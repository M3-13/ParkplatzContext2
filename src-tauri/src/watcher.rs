use std::sync::mpsc::Receiver;

use crate::models::AppState;

/// Starts the repository watcher.
///
/// It receives `()` on `rx` whenever a note is saved (via `repos_changed_tx`)
/// and resynchronizes its watch list. On a HEAD change it updates
/// `state.active_repo` so that `git_context` can capture the current context.
///
/// Stub: the real `notify`-based watcher is implemented by ticket #2
/// (branch-change notification).
pub fn start_watcher(_state: &AppState, _rx: Receiver<()>) {}
