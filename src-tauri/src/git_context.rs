use std::path::PathBuf;

use git2::Repository;

use crate::models::{AppState, ContextInfo};

/// Captures the current working context from the active git repository.
///
/// The active repository is read from `AppState.active_repo` (set by the
/// watcher when HEAD changes). When no active repository is known, we fall back
/// to discovering a repository from the directory the app was started in. When
/// no repository can be recognised at all, every field is left empty.
///
/// No shell and no subprocess is used: everything is read through the `git2`
/// (libgit2) crate.
pub fn capture_context(state: &AppState) -> ContextInfo {
    let active_repo: Option<PathBuf> = state
        .active_repo
        .lock()
        .ok()
        .and_then(|guard| (*guard).clone());

    let repo = match active_repo {
        Some(path) => Repository::open(&path).ok(),
        None => Repository::discover(start_dir()).ok(),
    };

    context_from_repo(repo)
}

fn context_from_repo(repo: Option<Repository>) -> ContextInfo {
    let repo = match repo {
        Some(repo) => repo,
        None => return empty_context(),
    };

    ContextInfo {
        repo_path: repo
            .workdir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default(),
        branch: current_branch(&repo),
        commit_hash: head_commit_hash(&repo),
        changed_files: changed_files(&repo),
    }
}

fn empty_context() -> ContextInfo {
    ContextInfo {
        repo_path: String::new(),
        branch: String::new(),
        commit_hash: String::new(),
        changed_files: Vec::new(),
    }
}

fn start_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn current_branch(repo: &Repository) -> String {
    repo.head()
        .ok()
        .and_then(|head| head.shorthand().map(|s| s.to_string()))
        .unwrap_or_default()
}

fn head_commit_hash(repo: &Repository) -> String {
    repo.head()
        .ok()
        .and_then(|head| head.target())
        .map(|oid| oid.to_string())
        .unwrap_or_default()
}

fn changed_files(repo: &Repository) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(statuses) = repo.statuses(None) {
        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                files.push(path.to_string());
            }
        }
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::sync::Mutex;

    fn app_state() -> AppState {
        let (tx, _rx) = mpsc::channel::<()>();
        AppState {
            db: Mutex::new(
                rusqlite::Connection::open_in_memory().expect("in-memory connection"),
            ),
            active_repo: Mutex::new(None),
            repos_changed_tx: tx,
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("parkplatz_test_{}_{}", name, std::process::id()));
        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
        std::fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    /// Initialises a repository and creates one commit so that `head()` points
    /// at a real branch with a real commit hash.
    fn init_repo_with_commit(dir: &std::path::Path) -> git2::Repository {
        let repo = Repository::init(dir).expect("init repo");
        let sig = git2::Signature::now("test", "test@example.com").expect("signature");
        let mut index = repo.index().expect("index");
        let tree_id = index.write_tree().expect("write tree");
        let tree = repo.find_tree(tree_id).expect("find tree");
        repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
            .expect("commit");
        repo
    }

    #[test]
    fn empty_context_when_no_repository() {
        let ctx = context_from_repo(None);
        assert!(ctx.repo_path.is_empty());
        assert!(ctx.branch.is_empty());
        assert!(ctx.commit_hash.is_empty());
        assert!(ctx.changed_files.is_empty());
    }

    #[test]
    fn active_repo_fields_are_filled() {
        let tmp = temp_dir("capture_fields");
        init_repo_with_commit(&tmp);

        let state = app_state();
        *state.active_repo.lock().unwrap() = Some(tmp.clone());

        let ctx = capture_context(&state);
        assert_eq!(ctx.repo_path, tmp.to_string_lossy());
        assert!(!ctx.branch.is_empty());
        assert!(!ctx.commit_hash.is_empty());
        assert!(ctx.changed_files.is_empty());
    }

    #[test]
    fn missing_active_repo_falls_back_to_discovery() {
        // The worktree this test runs in is a git repository, so discovery
        // from the current directory must yield a non-empty branch.
        let state = app_state();
        let ctx = capture_context(&state);
        assert!(!ctx.branch.is_empty());
    }
}
