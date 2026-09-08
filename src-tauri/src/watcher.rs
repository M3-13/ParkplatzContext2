//! Event-driven watcher for `.git/HEAD` changes.
//!
//! The watcher runs on a background thread and uses the `notify` crate to watch
//! the `.git/HEAD` file of every known repository (event-driven, no polling).
//! When a repository's checked-out branch changes, it remembers the repository
//! as the active one, resolves the new branch and, if a not-yet-done note is
//! parked on that branch, raises a system notification with its text.
//!
//! The watch list is (re)built from `config::load().workspace_root` plus every
//! repository returned by `db::list_known_repos`, and is refreshed every time a
//! `()` arrives on `repos_changed_rx` (which `db::save_note` sends).

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use notify::{
    recommended_watcher, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};

use crate::models::AppState;
use crate::{config, db, notification};

/// Start the `.git/HEAD` watcher on a dedicated background thread.
///
/// `state` is the shared application state; it must outlive the watcher and is
/// shared via `Arc` so that the Tauri command handlers and the watcher operate
/// on the same `db`, `active_repo` and `repos_changed_tx`. `repos_changed_rx`
/// is the receiving half of the channel whose sender lives in `state`; every
/// `()` on it triggers a rebuild of the watch list.
pub fn start_watcher(state: Arc<AppState>, repos_changed_rx: Receiver<()>) {
    std::thread::spawn(move || {
        run_watcher(state, repos_changed_rx);
    });
}

enum Control {
    Notify(notify::Result<Event>),
    Resync,
}

fn run_watcher(state: Arc<AppState>, repos_changed_rx: Receiver<()>) {
    let (control_tx, control_rx) = std::sync::mpsc::channel::<Control>();

    // Forward repo-change signals into the single control channel so the main
    // loop handles notify events and resync requests from one place.
    {
        let control_tx = control_tx.clone();
        std::thread::spawn(move || {
            while repos_changed_rx.recv().is_ok() {
                if control_tx.send(Control::Resync).is_err() {
                    break;
                }
            }
        });
    }

    let mut watcher = match recommended_watcher(move |res| {
        let _ = control_tx.send(Control::Notify(res));
    }) {
        Ok(w) => w,
        Err(_) => return,
    };

    // head path -> repo path
    let mut entries: HashMap<PathBuf, String> = HashMap::new();
    let mut watched_dirs: Vec<PathBuf> = Vec::new();

    resync(&state, &mut watcher, &mut entries, &mut watched_dirs);

    for control in control_rx {
        match control {
            Control::Resync => {
                resync(&state, &mut watcher, &mut entries, &mut watched_dirs);
            }
            Control::Notify(Ok(event)) => {
                if !is_head_event(&event) {
                    continue;
                }
                for path in &event.paths {
                    if is_head_path(path) {
                        if let Some(repo) = entries.get(path).cloned() {
                            handle_head_change(&state, &repo, path);
                        }
                    }
                }
            }
            Control::Notify(Err(_)) => {}
        }
    }
}

/// Rebuild the set of watched `.git` directories from the current configuration
/// and known repositories.
fn resync(
    state: &AppState,
    watcher: &mut RecommendedWatcher,
    entries: &mut HashMap<PathBuf, String>,
    watched_dirs: &mut Vec<PathBuf>,
) {
    let new_entries = build_entries(state);

    let mut new_dirs: Vec<PathBuf> = new_entries
        .keys()
        .filter_map(|head| head.parent().map(|p| p.to_path_buf()))
        .collect();
    new_dirs.sort();
    new_dirs.dedup();

    for dir in watched_dirs.iter() {
        if !new_dirs.contains(dir) {
            let _ = watcher.unwatch(dir);
        }
    }
    for dir in &new_dirs {
        if !watched_dirs.contains(dir) {
            let _ = watcher.watch(dir, RecursiveMode::NonRecursive);
        }
    }

    *entries = new_entries;
    *watched_dirs = new_dirs;
}

/// Collect the `HEAD` path -> repository path mapping for every repository the
/// watcher must observe: the configured workspace root plus all known repos.
fn build_entries(state: &AppState) -> HashMap<PathBuf, String> {
    let mut entries = HashMap::new();

    let cfg = config::load();
    collect_repo(&mut entries, cfg.workspace_root);

    if let Ok(conn) = state.db.lock() {
        if let Ok(repos) = db::list_known_repos(&conn) {
            for repo in repos {
                collect_repo(&mut entries, PathBuf::from(repo));
            }
        }
    }

    entries
}

fn collect_repo(entries: &mut HashMap<PathBuf, String>, repo_path: PathBuf) {
    if let Some(head) = head_path(&repo_path) {
        entries
            .entry(head)
            .or_insert_with(|| repo_path.to_string_lossy().into_owned());
    }
}

/// React to a `.git/HEAD` change: remember the active repo, resolve the new
/// branch and, if a pending note exists for it, notify.
fn handle_head_change(state: &AppState, repo_path: &str, head_path: &Path) {
    if let Ok(mut active) = state.active_repo.lock() {
        *active = Some(PathBuf::from(repo_path));
    }

    let branch = match branch_from_head(head_path) {
        Some(b) => b,
        None => return, // detached HEAD or unreadable file — no branch to match
    };

    let note = match state.db.lock() {
        Ok(conn) => match db::latest_for_branch(&conn, repo_path, &branch) {
            Ok(Some(n)) => n,
            _ => return,
        },
        Err(_) => return,
    };

    notification::notify(&branch, &note.note_text);
}

/// True when an event touches a file literally named `HEAD`.
fn is_head_event(event: &Event) -> bool {
    match &event.kind {
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Any => {
            event.paths.iter().any(|p| is_head_path(p))
        }
        _ => false,
    }
}

fn is_head_path(path: &Path) -> bool {
    path.file_name().map_or(false, |n| n == OsStr::new("HEAD"))
}

/// Resolve the `.git` directory of a repository, following the `gitdir:` indirection
/// used by linked worktrees (where `.git` is a plain file, not a directory).
fn git_dir(repo_path: &Path) -> Option<PathBuf> {
    let dot_git = repo_path.join(".git");
    let meta = std::fs::metadata(&dot_git).ok()?;
    if meta.is_dir() {
        return Some(dot_git);
    }
    let content = std::fs::read_to_string(&dot_git).ok()?;
    let line = content
        .lines()
        .find(|l| l.trim_start().starts_with("gitdir:"))?;
    let gitdir = line
        .trim_start()
        .trim_start_matches("gitdir:")
        .trim();
    if gitdir.is_empty() {
        return None;
    }
    Some(PathBuf::from(gitdir))
}

/// The `.git/HEAD` path for a repository, if it can be located.
fn head_path(repo_path: &Path) -> Option<PathBuf> {
    git_dir(repo_path).map(|d| d.join("HEAD"))
}

/// Read the branch name referenced by a `HEAD` file.
///
/// A normal `HEAD` reads `ref: refs/heads/<branch>`; a detached `HEAD` holds a
/// raw commit hash and yields `None` because there is no branch to match a note
/// against.
fn branch_from_head(head_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(head_path).ok()?;
    let content = content.trim();
    content
        .strip_prefix("ref: refs/heads/")
        .map(|b| b.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_repo() -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("parkplatz_watcher_test_{}", nanos));
        fs::create_dir_all(dir.join(".git")).unwrap();
        dir
    }

    #[test]
    fn resolves_branch_from_head() {
        let repo = temp_repo();
        fs::write(repo.join(".git/HEAD"), "ref: refs/heads/feature/foo\n").unwrap();
        let head = head_path(&repo).unwrap();
        assert_eq!(branch_from_head(&head).as_deref(), Some("feature/foo"));
        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn detached_head_has_no_branch() {
        let repo = temp_repo();
        fs::write(repo.join(".git/HEAD"), "0123456789abcdef0123456789abcdef01234567\n").unwrap();
        let head = head_path(&repo).unwrap();
        assert_eq!(branch_from_head(&head), None);
        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn missing_head_is_handled() {
        let repo = temp_repo();
        assert_eq!(branch_from_head(&repo.join(".git/HEAD")), None);
        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn follows_worktree_gitdir_file() {
        let repo = temp_repo();
        fs::remove_dir_all(repo.join(".git")).unwrap();
        fs::write(repo.join(".git"), "gitdir: /tmp/somewhere/.git/worktrees/foo\n").unwrap();
        let dir = git_dir(&repo).unwrap();
        assert!(dir.to_string_lossy().ends_with(".git/worktrees/foo"));
        fs::remove_file(repo.join(".git")).unwrap();
        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn is_head_path_matches_only_head() {
        assert!(is_head_path(Path::new("/a/b/HEAD")));
        assert!(is_head_path(Path::new("HEAD")));
        assert!(!is_head_path(Path::new("/a/b/HEAD~")));
        assert!(!is_head_path(Path::new("/a/b/index")));
    }
}
