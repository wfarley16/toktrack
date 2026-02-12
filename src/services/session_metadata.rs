//! Session metadata sidecar service
//!
//! Manages per-session metadata stored as JSON sidecar files
//! in `~/.toktrack/sessions/<session-id>.json`.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use directories::BaseDirs;
use regex::Regex;

use crate::types::{Result, SessionMetadata, ToktrackError};

/// Service for managing session metadata sidecar files
pub struct SessionMetadataService {
    sessions_dir: PathBuf,
}

impl SessionMetadataService {
    /// Create a new service using the default sidecar directory (`~/.toktrack/sessions/`)
    pub fn new() -> Result<Self> {
        let base_dirs = BaseDirs::new()
            .ok_or_else(|| ToktrackError::Config("Cannot determine home directory".into()))?;
        let sessions_dir = base_dirs.home_dir().join(".toktrack").join("sessions");
        fs::create_dir_all(&sessions_dir)?;
        Ok(Self { sessions_dir })
    }

    /// Create a service with a custom directory (for testing)
    #[cfg(test)]
    pub fn with_dir(sessions_dir: PathBuf) -> Self {
        Self { sessions_dir }
    }

    /// Get the sidecar directory path
    #[allow(dead_code)] // Used by hooks and CLI annotate command
    pub fn sidecar_dir(&self) -> &PathBuf {
        &self.sessions_dir
    }

    /// Load metadata for a single session by ID
    pub fn load(&self, session_id: &str) -> Option<SessionMetadata> {
        let path = self.sessions_dir.join(format!("{}.json", session_id));
        if !path.exists() {
            return None;
        }
        let content = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Save metadata to a sidecar file
    pub fn save(&self, metadata: &SessionMetadata) -> Result<()> {
        let path = self
            .sessions_dir
            .join(format!("{}.json", metadata.session_id));
        let content = serde_json::to_string_pretty(metadata)
            .map_err(|e| ToktrackError::Cache(format!("Failed to serialize metadata: {}", e)))?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Load all metadata files from the sidecar directory
    pub fn load_all(&self) -> HashMap<String, SessionMetadata> {
        let mut map = HashMap::new();

        let entries = match fs::read_dir(&self.sessions_dir) {
            Ok(entries) => entries,
            Err(_) => return map,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(metadata) = serde_json::from_str::<SessionMetadata>(&content) {
                    map.insert(metadata.session_id.clone(), metadata);
                }
            }
        }

        map
    }
}

/// Extract an issue ID (e.g., `ISE-123`, `PROJ-456`) from a git branch name.
///
/// Matches the first occurrence of `[A-Z]+-\d+` in the branch string.
pub fn extract_issue_id(branch: &str) -> Option<String> {
    let re = Regex::new(r"[A-Z]+-\d+").expect("valid regex");
    re.find(branch).map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AutoDetected;
    use chrono::Utc;
    use tempfile::TempDir;

    fn make_metadata(session_id: &str) -> SessionMetadata {
        let now = Utc::now();
        SessionMetadata {
            session_id: session_id.to_string(),
            issue_id: Some("ISE-123".to_string()),
            tags: vec!["bug-fix".to_string()],
            notes: Some("test notes".to_string()),
            skills_used: vec!["clarify".to_string(), "implement".to_string()],
            auto_detected: Some(AutoDetected {
                git_branch: Some("feature/ISE-123-fix-bug".to_string()),
                issue_id_source: Some("branch".to_string()),
            }),
            created_at: now,
            updated_at: now,
        }
    }

    // ========== extract_issue_id tests ==========

    #[test]
    fn test_extract_issue_id_feature_branch() {
        assert_eq!(
            extract_issue_id("feature/ISE-123-foo"),
            Some("ISE-123".to_string())
        );
    }

    #[test]
    fn test_extract_issue_id_bare() {
        assert_eq!(extract_issue_id("ISE-456"), Some("ISE-456".to_string()));
    }

    #[test]
    fn test_extract_issue_id_main() {
        assert_eq!(extract_issue_id("main"), None);
    }

    #[test]
    fn test_extract_issue_id_no_issue() {
        assert_eq!(extract_issue_id("bugfix/no-issue"), None);
    }

    #[test]
    fn test_extract_issue_id_different_prefix() {
        assert_eq!(
            extract_issue_id("fix/PROJ-789-bar"),
            Some("PROJ-789".to_string())
        );
    }

    #[test]
    fn test_extract_issue_id_empty() {
        assert_eq!(extract_issue_id(""), None);
    }

    #[test]
    fn test_extract_issue_id_lowercase_ignored() {
        assert_eq!(extract_issue_id("feature/ise-123-foo"), None);
    }

    // ========== Serialize/deserialize round-trip ==========

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let metadata = make_metadata("abc-123");
        let json = serde_json::to_string_pretty(&metadata).unwrap();
        let deserialized: SessionMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(metadata.session_id, deserialized.session_id);
        assert_eq!(metadata.issue_id, deserialized.issue_id);
        assert_eq!(metadata.tags, deserialized.tags);
        assert_eq!(metadata.notes, deserialized.notes);
        assert_eq!(metadata.skills_used, deserialized.skills_used);
        assert_eq!(metadata.auto_detected, deserialized.auto_detected);
    }

    #[test]
    fn test_deserialize_minimal() {
        let json = r#"{
            "session_id": "minimal-session",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }"#;
        let metadata: SessionMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.session_id, "minimal-session");
        assert_eq!(metadata.issue_id, None);
        assert!(metadata.tags.is_empty());
        assert!(metadata.skills_used.is_empty());
        assert_eq!(metadata.notes, None);
        assert_eq!(metadata.auto_detected, None);
    }

    // ========== Service tests ==========

    #[test]
    fn test_load_nonexistent_returns_none() {
        let tmp = TempDir::new().unwrap();
        let service = SessionMetadataService::with_dir(tmp.path().to_path_buf());
        assert!(service.load("nonexistent-session").is_none());
    }

    #[test]
    fn test_save_and_load() {
        let tmp = TempDir::new().unwrap();
        let service = SessionMetadataService::with_dir(tmp.path().to_path_buf());
        let metadata = make_metadata("test-session-1");

        service.save(&metadata).unwrap();

        let loaded = service.load("test-session-1").unwrap();
        assert_eq!(loaded.session_id, "test-session-1");
        assert_eq!(loaded.issue_id, Some("ISE-123".to_string()));
        assert_eq!(loaded.tags, vec!["bug-fix".to_string()]);
    }

    #[test]
    fn test_save_creates_file() {
        let tmp = TempDir::new().unwrap();
        let service = SessionMetadataService::with_dir(tmp.path().to_path_buf());
        let metadata = make_metadata("file-check");

        service.save(&metadata).unwrap();

        let path = tmp.path().join("file-check.json");
        assert!(path.exists());
    }

    #[test]
    fn test_load_all_multiple() {
        let tmp = TempDir::new().unwrap();
        let service = SessionMetadataService::with_dir(tmp.path().to_path_buf());

        service.save(&make_metadata("session-a")).unwrap();
        service.save(&make_metadata("session-b")).unwrap();

        let all = service.load_all();
        assert_eq!(all.len(), 2);
        assert!(all.contains_key("session-a"));
        assert!(all.contains_key("session-b"));
    }

    #[test]
    fn test_load_all_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let service = SessionMetadataService::with_dir(tmp.path().to_path_buf());
        let all = service.load_all();
        assert!(all.is_empty());
    }

    #[test]
    fn test_load_all_ignores_invalid_json() {
        let tmp = TempDir::new().unwrap();
        let service = SessionMetadataService::with_dir(tmp.path().to_path_buf());

        service.save(&make_metadata("valid")).unwrap();
        fs::write(tmp.path().join("invalid.json"), "not json").unwrap();

        let all = service.load_all();
        assert_eq!(all.len(), 1);
        assert!(all.contains_key("valid"));
    }

    #[test]
    fn test_load_all_ignores_non_json_files() {
        let tmp = TempDir::new().unwrap();
        let service = SessionMetadataService::with_dir(tmp.path().to_path_buf());

        service.save(&make_metadata("valid")).unwrap();
        fs::write(tmp.path().join("readme.txt"), "not a sidecar").unwrap();

        let all = service.load_all();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_sidecar_dir() {
        let tmp = TempDir::new().unwrap();
        let service = SessionMetadataService::with_dir(tmp.path().to_path_buf());
        assert_eq!(service.sidecar_dir(), &tmp.path().to_path_buf());
    }
}
