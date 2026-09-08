use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::Mutex;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// A parked note ("Zettel") captured at a point in time for a repository/branch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub repo_path: String,
    pub branch: String,
    pub commit_hash: String,
    pub changed_files: Vec<String>,
    pub note_text: String,
    pub created_at: i64,
    pub done: bool,
}

/// The current working context captured from a git repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInfo {
    pub repo_path: String,
    pub branch: String,
    pub commit_hash: String,
    pub changed_files: Vec<String>,
}

/// Shared application state held by Tauri and passed to every command.
pub struct AppState {
    pub db: Mutex<Connection>,
    pub active_repo: Mutex<Option<PathBuf>>,
    pub repos_changed_tx: Sender<()>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_roundtrips_through_json() {
        let note = Note {
            id: 7,
            repo_path: "/home/dev/myproject".to_string(),
            branch: "feature/x".to_string(),
            commit_hash: "abc123def".to_string(),
            changed_files: vec!["src/main.rs".to_string(), "Cargo.toml".to_string()],
            note_text: "Weiter an der Login-Seite arbeiten".to_string(),
            created_at: 1700000000,
            done: false,
        };

        let json = serde_json::to_string(&note).unwrap();
        let parsed: Note = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, 7);
        assert_eq!(parsed.repo_path, "/home/dev/myproject");
        assert_eq!(parsed.branch, "feature/x");
        assert_eq!(parsed.commit_hash, "abc123def");
        assert_eq!(parsed.changed_files.len(), 2);
        assert_eq!(parsed.note_text, "Weiter an der Login-Seite arbeiten");
        assert_eq!(parsed.created_at, 1700000000);
        assert!(!parsed.done);
    }

    #[test]
    fn context_info_roundtrips_through_json() {
        let ctx = ContextInfo {
            repo_path: "/repo".to_string(),
            branch: "main".to_string(),
            commit_hash: "0123456".to_string(),
            changed_files: vec!["a.txt".to_string()],
        };

        let json = serde_json::to_string(&ctx).unwrap();
        let parsed: ContextInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.repo_path, "/repo");
        assert_eq!(parsed.branch, "main");
        assert_eq!(parsed.commit_hash, "0123456");
        assert_eq!(parsed.changed_files, vec!["a.txt".to_string()]);
    }

    #[test]
    fn context_info_serializes_to_expected_json_shape() {
        let ctx = ContextInfo {
            repo_path: "/repo".to_string(),
            branch: "main".to_string(),
            commit_hash: "abc".to_string(),
            changed_files: vec![],
        };
        let value: serde_json::Value = serde_json::to_value(&ctx).unwrap();
        assert!(value.get("repo_path").is_some());
        assert!(value.get("branch").is_some());
        assert!(value.get("commit_hash").is_some());
        assert!(value.get("changed_files").is_some());
    }
}
