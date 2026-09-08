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
