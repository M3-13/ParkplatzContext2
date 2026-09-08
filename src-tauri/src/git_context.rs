use std::path::Path;

use crate::models::{AppState, ContextInfo};

/// Captures the git context (repo path, branch, commit hash, changed files) of
/// the given repository directory.
///
/// Stub: the real libgit2-based capture is implemented by ticket #1.
#[allow(dead_code)]
pub fn capture_context(_repo: &Path) -> Result<ContextInfo, String> {
    Ok(ContextInfo {
        repo_path: String::new(),
        branch: String::new(),
        commit_hash: String::new(),
        changed_files: Vec::new(),
    })
}

/// Reads the currently active repository (set by the watcher on HEAD change)
/// and captures its context.
///
/// Stub: implemented by ticket #1.
pub fn get_context(_state: &AppState) -> Result<ContextInfo, String> {
    Ok(ContextInfo {
        repo_path: String::new(),
        branch: String::new(),
        commit_hash: String::new(),
        changed_files: Vec::new(),
    })
}
