//! Claude Code JSONL parser

use crate::services::normalizer::{display_name, normalize_model_name};
use crate::services::PricingService;
use crate::types::{Result, SessionDetailEntry, SessionInfo, ToktrackError, UsageEntry};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use super::CLIParser;

/// Claude Code JSONL line structure (assistant messages with usage)
#[derive(Deserialize)]
struct ClaudeJsonLine<'a> {
    timestamp: &'a str,
    #[serde(rename = "requestId")]
    request_id: Option<&'a str>,
    message: Option<ClaudeMessage<'a>>,
    #[serde(rename = "costUSD")]
    cost_usd: Option<f64>,
}

#[derive(Deserialize)]
struct ClaudeMessage<'a> {
    model: Option<&'a str>,
    id: Option<&'a str>,
    usage: Option<ClaudeUsage>,
}

#[derive(Deserialize)]
struct ClaudeUsage {
    input_tokens: u64,
    output_tokens: u64,
    cache_creation_input_tokens: Option<u64>,
    cache_read_input_tokens: Option<u64>,
}

/// Lightweight struct for extracting session metadata from user-type JSONL lines
#[derive(Deserialize)]
struct SessionMetadataLine {
    #[serde(rename = "type")]
    line_type: Option<String>,
    #[serde(rename = "sessionId", default)]
    session_id: Option<String>,
    timestamp: Option<String>,
    #[serde(rename = "gitBranch", default)]
    git_branch: Option<String>,
    cwd: Option<String>,
    message: Option<SessionMetadataMessage>,
}

#[derive(Deserialize)]
struct SessionMetadataMessage {
    role: Option<String>,
    content: Option<serde_json::Value>,
}

/// Parser for Claude Code usage data
pub struct ClaudeCodeParser {
    data_dir: PathBuf,
}

impl ClaudeCodeParser {
    /// Create a new parser with default data directory (~/.claude/projects/)
    pub fn new() -> Self {
        let home = directories::BaseDirs::new()
            .map(|d| d.home_dir().to_path_buf())
            .unwrap_or_else(|| {
                eprintln!("[toktrack] Warning: Could not determine home directory");
                PathBuf::from(".")
            });
        Self {
            data_dir: home.join(".claude").join("projects"),
        }
    }

    /// Create a parser with a custom data directory (for testing)
    #[allow(dead_code)] // Used in tests
    pub fn with_data_dir(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    /// Parse a single JSONL line (zero-copy with borrowed strings)
    fn parse_line(&self, line: &mut [u8]) -> Option<UsageEntry> {
        if line.is_empty() {
            return None;
        }

        let data: ClaudeJsonLine = simd_json::from_slice(line).ok()?;

        // Only process lines with message and usage data
        let message = data.message.as_ref()?;
        let usage = message.usage.as_ref()?;

        // Skip synthetic responses (no actual API call)
        if message.model == Some("<synthetic>") {
            return None;
        }

        let timestamp = match DateTime::parse_from_rfc3339(data.timestamp) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => {
                eprintln!(
                    "[toktrack] Warning: Invalid timestamp '{}', skipping entry",
                    data.timestamp
                );
                return None;
            }
        };

        Some(UsageEntry {
            timestamp,
            model: message.model.map(String::from),
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            cache_read_tokens: usage.cache_read_input_tokens.unwrap_or(0),
            cache_creation_tokens: usage.cache_creation_input_tokens.unwrap_or(0),
            thinking_tokens: 0,
            cost_usd: data.cost_usd,
            message_id: message.id.map(String::from),
            request_id: data.request_id.map(String::from),
            source: Some("claude".into()),
            provider: None,
        })
    }
}

/// Sessions index file structure
#[derive(Deserialize)]
struct SessionsIndex {
    entries: Vec<SessionsIndexEntry>,
}

/// A single entry in sessions-index.json
#[derive(Deserialize)]
struct SessionsIndexEntry {
    #[serde(rename = "sessionId")]
    session_id: String,
    #[serde(rename = "fullPath")]
    full_path: String,
    #[serde(rename = "firstPrompt", default)]
    first_prompt: String,
    #[serde(default)]
    summary: String,
    #[serde(rename = "messageCount", default)]
    message_count: u64,
    #[serde(default)]
    created: String,
    #[serde(default)]
    modified: String,
    #[serde(rename = "gitBranch", default)]
    git_branch: String,
    #[serde(rename = "projectPath", default)]
    project_path: String,
}

impl ClaudeCodeParser {
    /// Scan all sessions-index.json files and return session metadata with
    /// aggregated cost/token data from quick-parsing each session's JSONL.
    pub fn parse_sessions_index(&self, pricing: Option<&PricingService>) -> Vec<SessionInfo> {
        let pattern = self.data_dir.join("*/sessions-index.json");
        let index_files: Vec<PathBuf> = glob::glob(&pattern.to_string_lossy())
            .map(|paths| paths.filter_map(|e| e.ok()).collect())
            .unwrap_or_default();

        let mut sessions = Vec::new();
        let mut indexed_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

        for index_path in &index_files {
            let content = match std::fs::read_to_string(index_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let index: SessionsIndex = match serde_json::from_str(&content) {
                Ok(i) => i,
                Err(_) => continue,
            };

            for entry in index.entries {
                indexed_paths.insert(entry.full_path.clone());

                let created = DateTime::parse_from_rfc3339(&entry.created)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_default();
                let modified = DateTime::parse_from_rfc3339(&entry.modified)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_default();

                // Extract project name from last path segment
                let project = entry
                    .project_path
                    .rsplit('/')
                    .next()
                    .unwrap_or(&entry.project_path)
                    .to_string();

                // Quick-parse the JSONL to get cost/token/model aggregates
                let (total_cost_usd, total_tokens, primary_model) =
                    self.quick_parse_session_jsonl(&entry.full_path, pricing);

                sessions.push(SessionInfo {
                    session_id: entry.session_id,
                    project,
                    project_path: entry.project_path,
                    summary: entry.summary,
                    first_prompt: entry.first_prompt,
                    message_count: entry.message_count,
                    created,
                    modified,
                    git_branch: entry.git_branch,
                    jsonl_path: entry.full_path,
                    total_cost_usd,
                    total_tokens,
                    primary_model,
                    metadata: None,
                });
            }
        }

        // Discover JSONL files not present in any index (fallback for stale indexes)
        let jsonl_pattern = self.data_dir.join("*/*.jsonl");
        let jsonl_files: Vec<PathBuf> = glob::glob(&jsonl_pattern.to_string_lossy())
            .map(|paths| paths.filter_map(|e| e.ok()).collect())
            .unwrap_or_default();

        for jsonl_path in jsonl_files {
            let path_str = jsonl_path.to_string_lossy().to_string();
            if indexed_paths.contains(&path_str) {
                continue;
            }

            if let Some(session) = self.session_from_jsonl(&jsonl_path, pricing) {
                sessions.push(session);
            }
        }

        // Sort by created descending (most recent first)
        sessions.sort_by(|a, b| b.created.cmp(&a.created));
        sessions
    }

    /// Build a SessionInfo by extracting metadata directly from a JSONL file.
    /// Used as fallback when the session isn't in any sessions-index.json.
    fn session_from_jsonl(
        &self,
        jsonl_path: &Path,
        pricing: Option<&PricingService>,
    ) -> Option<SessionInfo> {
        let file = File::open(jsonl_path).ok()?;
        let reader = BufReader::new(file);

        let mut session_id = String::new();
        let mut git_branch = String::new();
        let mut project_path = String::new();
        let mut first_prompt = String::new();
        let mut first_timestamp: Option<DateTime<Utc>> = None;
        let mut last_timestamp: Option<DateTime<Utc>> = None;
        let mut message_count: u64 = 0;
        let mut total_cost: f64 = 0.0;
        let mut total_tokens: u64 = 0;
        let mut model_counts: HashMap<String, u64> = HashMap::new();

        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) if !l.is_empty() => l,
                _ => continue,
            };

            // Try to parse metadata from user/assistant lines
            if let Ok(meta) = serde_json::from_str::<SessionMetadataLine>(&line) {
                if let Some(ref ts) = meta.timestamp {
                    if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
                        let dt_utc = dt.with_timezone(&Utc);
                        if first_timestamp.is_none() {
                            first_timestamp = Some(dt_utc);
                        }
                        last_timestamp = Some(dt_utc);
                    }
                }

                if session_id.is_empty() {
                    if let Some(ref id) = meta.session_id {
                        session_id = id.clone();
                    }
                }

                let is_user = meta.line_type.as_deref() == Some("user");
                let is_assistant = meta.line_type.as_deref() == Some("assistant");

                if is_user {
                    message_count += 1;
                    if let Some(ref cwd) = meta.cwd {
                        if project_path.is_empty() {
                            project_path = cwd.clone();
                        }
                    }
                    if let Some(ref branch) = meta.git_branch {
                        if git_branch.is_empty() || git_branch == "HEAD" {
                            git_branch = branch.clone();
                        }
                    }
                    // Extract first user prompt
                    if first_prompt.is_empty() {
                        if let Some(ref msg) = meta.message {
                            if msg.role.as_deref() == Some("user") {
                                first_prompt = extract_text_content(&msg.content);
                            }
                        }
                    }
                }

                if is_assistant {
                    message_count += 1;
                }
            }

            // Also parse for cost/token data via the existing parser
            let mut line_bytes = line.into_bytes();
            if let Some(entry) = self.parse_line(&mut line_bytes) {
                let tokens = entry.input_tokens
                    + entry.output_tokens
                    + entry.cache_read_tokens
                    + entry.cache_creation_tokens;
                total_tokens = total_tokens.saturating_add(tokens);

                let cost = entry
                    .cost_usd
                    .unwrap_or_else(|| pricing.map_or(0.0, |p| p.calculate_cost(&entry)));
                total_cost += cost;

                if let Some(ref model) = entry.model {
                    *model_counts.entry(model.clone()).or_default() += 1;
                }
            }
        }

        // Skip files that don't look like real sessions
        let created = first_timestamp?;
        let modified = last_timestamp.unwrap_or(created);

        let project = project_path
            .rsplit('/')
            .next()
            .unwrap_or(&project_path)
            .to_string();

        let primary_model = model_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(model, _)| display_name(&normalize_model_name(&model)))
            .unwrap_or_default();

        Some(SessionInfo {
            session_id,
            project,
            project_path,
            summary: String::new(),
            first_prompt,
            message_count,
            created,
            modified,
            git_branch,
            jsonl_path: jsonl_path.to_string_lossy().to_string(),
            total_cost_usd: total_cost,
            total_tokens,
            primary_model,
            metadata: None,
        })
    }

    /// Quick-parse a session JSONL to get aggregated cost, tokens, and primary model.
    fn quick_parse_session_jsonl(
        &self,
        jsonl_path: &str,
        pricing: Option<&PricingService>,
    ) -> (f64, u64, String) {
        let path = Path::new(jsonl_path);
        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return (0.0, 0, String::new()),
        };
        let reader = BufReader::new(file);

        let mut total_cost: f64 = 0.0;
        let mut total_tokens: u64 = 0;
        let mut model_counts: HashMap<String, u64> = HashMap::new();

        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) if !l.is_empty() => l,
                _ => continue,
            };

            let mut line_bytes = line.into_bytes();
            if let Some(entry) = self.parse_line(&mut line_bytes) {
                let tokens = entry.input_tokens
                    + entry.output_tokens
                    + entry.cache_read_tokens
                    + entry.cache_creation_tokens;
                total_tokens = total_tokens.saturating_add(tokens);

                let cost = entry
                    .cost_usd
                    .unwrap_or_else(|| pricing.map_or(0.0, |p| p.calculate_cost(&entry)));
                total_cost += cost;

                if let Some(ref model) = entry.model {
                    *model_counts.entry(model.clone()).or_default() += 1;
                }
            }
        }

        let primary_model = model_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(model, _)| display_name(&normalize_model_name(&model)))
            .unwrap_or_default();

        (total_cost, total_tokens, primary_model)
    }

    /// Parse a session JSONL on-demand for the detail drill-down view.
    /// Returns individual request entries sorted by timestamp ascending.
    pub fn parse_session_detail(
        jsonl_path: &str,
        pricing: Option<&PricingService>,
    ) -> Vec<SessionDetailEntry> {
        let path = Path::new(jsonl_path);
        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };
        let reader = BufReader::new(file);
        let parser = ClaudeCodeParser::with_data_dir(PathBuf::from("."));

        let mut entries = Vec::new();

        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) if !l.is_empty() => l,
                _ => continue,
            };

            let mut line_bytes = line.into_bytes();
            if let Some(entry) = parser.parse_line(&mut line_bytes) {
                entries.push(SessionDetailEntry {
                    timestamp: entry.timestamp,
                    model: entry
                        .model
                        .as_deref()
                        .map(|m| display_name(&normalize_model_name(m)))
                        .unwrap_or_default(),
                    input_tokens: entry.input_tokens,
                    output_tokens: entry.output_tokens,
                    cache_read_tokens: entry.cache_read_tokens,
                    cache_creation_tokens: entry.cache_creation_tokens,
                    cost_usd: entry
                        .cost_usd
                        .unwrap_or_else(|| pricing.map_or(0.0, |p| p.calculate_cost(&entry))),
                });
            }
        }

        // Sort by timestamp ascending
        entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        entries
    }
}

/// Extract text content from a user message's content field.
/// Content can be a plain string or an array of content blocks.
fn extract_text_content(content: &Option<serde_json::Value>) -> String {
    match content {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Array(arr)) => {
            for item in arr {
                if let Some(obj) = item.as_object() {
                    if obj.get("type").and_then(|t| t.as_str()) == Some("text") {
                        if let Some(text) = obj.get("text").and_then(|t| t.as_str()) {
                            return text.to_string();
                        }
                    }
                }
            }
            String::new()
        }
        _ => String::new(),
    }
}

impl Default for ClaudeCodeParser {
    fn default() -> Self {
        Self::new()
    }
}

impl CLIParser for ClaudeCodeParser {
    fn name(&self) -> &str {
        "claude-code"
    }

    fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    fn file_pattern(&self) -> &str {
        "**/*.jsonl"
    }

    fn parse_file(&self, path: &Path) -> Result<Vec<UsageEntry>> {
        let file = File::open(path).map_err(ToktrackError::Io)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        // Stream line-by-line to avoid loading entire file into memory
        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(_) => continue, // Skip lines with read errors
            };

            if line.is_empty() {
                continue;
            }

            // Convert to mutable bytes for simd-json
            let mut line_bytes = line.into_bytes();
            if let Some(entry) = self.parse_line(&mut line_bytes) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn test_parse_claude_jsonl() {
        let parser = ClaudeCodeParser::with_data_dir(PathBuf::from("tests/fixtures"));
        let entries = parser
            .parse_file(&fixture_path("claude-sample.jsonl"))
            .unwrap();

        // Should parse 3 assistant messages (skipping user message and invalid line)
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn test_parse_first_entry() {
        let parser = ClaudeCodeParser::with_data_dir(PathBuf::from("tests/fixtures"));
        let entries = parser
            .parse_file(&fixture_path("claude-sample.jsonl"))
            .unwrap();

        let first = &entries[0];
        assert_eq!(first.model, Some("claude-sonnet-4-20250514".to_string()));
        assert_eq!(first.input_tokens, 100);
        assert_eq!(first.output_tokens, 50);
        assert_eq!(first.cache_creation_tokens, 10);
        assert_eq!(first.cache_read_tokens, 20);
        assert_eq!(first.message_id, Some("msg-001".to_string()));
        assert_eq!(first.request_id, Some("req-001".to_string()));
    }

    #[test]
    fn test_parse_entry_with_cost() {
        let parser = ClaudeCodeParser::with_data_dir(PathBuf::from("tests/fixtures"));
        let entries = parser
            .parse_file(&fixture_path("claude-sample.jsonl"))
            .unwrap();

        let second = &entries[1];
        assert_eq!(second.model, Some("claude-opus-4-20250514".to_string()));
        assert_eq!(second.cost_usd, Some(0.025));
    }

    #[test]
    fn test_parse_entry_without_optional_fields() {
        let parser = ClaudeCodeParser::with_data_dir(PathBuf::from("tests/fixtures"));
        let entries = parser
            .parse_file(&fixture_path("claude-sample.jsonl"))
            .unwrap();

        // Third entry has no cache tokens, message_id, or request_id
        let third = &entries[2];
        assert_eq!(third.cache_creation_tokens, 0);
        assert_eq!(third.cache_read_tokens, 0);
        assert_eq!(third.message_id, None);
        assert_eq!(third.request_id, None);
    }

    #[test]
    fn test_skip_invalid_lines() {
        let parser = ClaudeCodeParser::with_data_dir(PathBuf::from("tests/fixtures"));
        let entries = parser
            .parse_file(&fixture_path("claude-sample.jsonl"))
            .unwrap();

        // Invalid JSON line should be skipped, not cause an error
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn test_skip_user_messages() {
        let parser = ClaudeCodeParser::with_data_dir(PathBuf::from("tests/fixtures"));
        let entries = parser
            .parse_file(&fixture_path("claude-sample.jsonl"))
            .unwrap();

        // User message has no usage, should be skipped
        // All entries should have input_tokens > 0
        assert!(entries.iter().all(|e| e.input_tokens > 0));
    }

    #[test]
    fn test_dedup_hash() {
        let parser = ClaudeCodeParser::with_data_dir(PathBuf::from("tests/fixtures"));
        let entries = parser
            .parse_file(&fixture_path("claude-sample.jsonl"))
            .unwrap();

        // First entry has both message_id and request_id
        assert_eq!(entries[0].dedup_hash(), Some("msg-001:req-001".to_string()));

        // Third entry has neither
        assert_eq!(entries[2].dedup_hash(), None);
    }

    #[test]
    fn test_parser_name() {
        let parser = ClaudeCodeParser::new();
        assert_eq!(parser.name(), "claude-code");
    }

    #[test]
    fn test_parser_file_pattern() {
        let parser = ClaudeCodeParser::new();
        assert_eq!(parser.file_pattern(), "**/*.jsonl");
    }

    #[test]
    fn test_parse_nonexistent_file() {
        let parser = ClaudeCodeParser::new();
        let result = parser.parse_file(Path::new("/nonexistent/file.jsonl"));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_file() {
        let parser = ClaudeCodeParser::with_data_dir(PathBuf::from("tests/fixtures"));
        let entries = parser.parse_file(&fixture_path("empty.jsonl")).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_skip_synthetic_model() {
        let parser = ClaudeCodeParser::with_data_dir(PathBuf::from("tests/fixtures"));
        let entries = parser
            .parse_file(&fixture_path("claude-sample.jsonl"))
            .unwrap();

        // <synthetic> model entries should be filtered out
        assert!(
            entries
                .iter()
                .all(|e| e.model != Some("<synthetic>".to_string())),
            "Synthetic model entries should be filtered out"
        );
    }
}
