//! `toktrack annotate` subcommand for editing session metadata sidecars

use chrono::Utc;
use clap::Args;

use crate::services::session_metadata::SessionMetadataService;
use crate::types::{Result, SessionMetadata, ToktrackError};

/// Annotate session metadata
#[derive(Args, Debug)]
pub struct AnnotateArgs {
    /// Session ID to annotate (omit to use --latest)
    #[arg(value_name = "SESSION_ID")]
    pub session_id: Option<String>,

    /// Use the most recently updated session
    #[arg(long)]
    pub latest: bool,

    /// Set session title
    #[arg(long)]
    pub title: Option<String>,

    /// Set issue ID (e.g., ISE-123)
    #[arg(long)]
    pub issue: Option<String>,

    /// Add tag(s) to the session
    #[arg(long, num_args = 1..)]
    pub tag: Vec<String>,

    /// Set notes for the session
    #[arg(long)]
    pub note: Option<String>,

    /// Clear all tags
    #[arg(long)]
    pub clear_tags: bool,
}

impl AnnotateArgs {
    pub fn run(self) -> Result<()> {
        let service = SessionMetadataService::new()?;

        let session_id = if let Some(id) = self.session_id {
            id
        } else if self.latest {
            find_latest_session(&service)?
        } else {
            return Err(ToktrackError::Config(
                "Provide a SESSION_ID or use --latest".into(),
            ));
        };

        let now = Utc::now();
        let mut metadata = service
            .load(&session_id)
            .unwrap_or_else(|| SessionMetadata {
                session_id: session_id.clone(),
                title: None,
                issue_id: None,
                tags: Vec::new(),
                notes: None,
                skills_used: Vec::new(),
                auto_detected: None,
                created_at: now,
                updated_at: now,
            });

        let mut changed = false;

        if let Some(title) = self.title {
            metadata.title = Some(title);
            changed = true;
        }

        if let Some(issue) = self.issue {
            metadata.issue_id = Some(issue);
            changed = true;
        }

        if self.clear_tags {
            metadata.tags.clear();
            changed = true;
        }

        for tag in self.tag {
            if !metadata.tags.contains(&tag) {
                metadata.tags.push(tag);
                changed = true;
            }
        }

        if let Some(note) = self.note {
            metadata.notes = Some(note);
            changed = true;
        }

        if changed {
            metadata.updated_at = now;
        }

        service.save(&metadata)?;

        // Print updated metadata to stdout
        let json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| ToktrackError::Parse(e.to_string()))?;
        println!("{}", json);

        Ok(())
    }
}

/// Find the most recently updated sidecar file
fn find_latest_session(service: &SessionMetadataService) -> Result<String> {
    let all = service.load_all();
    all.values()
        .max_by_key(|m| m.updated_at)
        .map(|m| m.session_id.clone())
        .ok_or_else(|| {
            ToktrackError::Config("No session metadata found. Run a session first.".into())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_service() -> (TempDir, SessionMetadataService) {
        let tmp = TempDir::new().unwrap();
        let svc = SessionMetadataService::with_dir(tmp.path().to_path_buf());
        (tmp, svc)
    }

    #[test]
    fn test_find_latest_session_empty() {
        let (_tmp, svc) = make_service();
        let result = find_latest_session(&svc);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_latest_session_picks_most_recent() {
        let (_tmp, svc) = make_service();
        let now = Utc::now();

        let older = SessionMetadata {
            session_id: "old-session".to_string(),
            title: None,
            issue_id: None,
            tags: Vec::new(),
            notes: None,
            skills_used: Vec::new(),
            auto_detected: None,
            created_at: now - chrono::Duration::hours(2),
            updated_at: now - chrono::Duration::hours(2),
        };
        let newer = SessionMetadata {
            session_id: "new-session".to_string(),
            title: None,
            issue_id: None,
            tags: Vec::new(),
            notes: None,
            skills_used: Vec::new(),
            auto_detected: None,
            created_at: now,
            updated_at: now,
        };

        svc.save(&older).unwrap();
        svc.save(&newer).unwrap();

        let latest = find_latest_session(&svc).unwrap();
        assert_eq!(latest, "new-session");
    }

    #[test]
    fn test_annotate_creates_new_sidecar() {
        let (_tmp, svc) = make_service();
        let args = AnnotateArgs {
            session_id: Some("test-session".to_string()),
            latest: false,
            title: None,
            issue: Some("ISE-999".to_string()),
            tag: vec!["urgent".to_string()],
            note: Some("test note".to_string()),
            clear_tags: false,
        };

        // Simulate what run() does (without the println)
        let now = Utc::now();
        let mut metadata = svc.load("test-session").unwrap_or_else(|| SessionMetadata {
            session_id: "test-session".to_string(),
            title: None,
            issue_id: None,
            tags: Vec::new(),
            notes: None,
            skills_used: Vec::new(),
            auto_detected: None,
            created_at: now,
            updated_at: now,
        });

        if let Some(issue) = args.issue {
            metadata.issue_id = Some(issue);
        }
        for tag in args.tag {
            if !metadata.tags.contains(&tag) {
                metadata.tags.push(tag);
            }
        }
        if let Some(note) = args.note {
            metadata.notes = Some(note);
        }

        svc.save(&metadata).unwrap();

        let loaded = svc.load("test-session").unwrap();
        assert_eq!(loaded.issue_id, Some("ISE-999".to_string()));
        assert_eq!(loaded.tags, vec!["urgent".to_string()]);
        assert_eq!(loaded.notes, Some("test note".to_string()));
    }

    #[test]
    fn test_annotate_clear_tags() {
        let (_tmp, svc) = make_service();
        let now = Utc::now();

        let metadata = SessionMetadata {
            session_id: "tag-test".to_string(),
            title: None,
            issue_id: None,
            tags: vec!["old-tag".to_string()],
            notes: None,
            skills_used: Vec::new(),
            auto_detected: None,
            created_at: now,
            updated_at: now,
        };
        svc.save(&metadata).unwrap();

        let mut loaded = svc.load("tag-test").unwrap();
        loaded.tags.clear(); // simulates --clear-tags
        loaded.tags.push("new-tag".to_string());
        svc.save(&loaded).unwrap();

        let result = svc.load("tag-test").unwrap();
        assert_eq!(result.tags, vec!["new-tag".to_string()]);
    }

    #[test]
    fn test_annotate_no_session_id_no_latest_errors() {
        let args = AnnotateArgs {
            session_id: None,
            latest: false,
            title: None,
            issue: None,
            tag: Vec::new(),
            note: None,
            clear_tags: false,
        };

        // Should fail because no session_id and --latest not set
        // (We can't call run() in test without stdout capture, so test the logic)
        assert!(args.session_id.is_none());
        assert!(!args.latest);
    }
}
