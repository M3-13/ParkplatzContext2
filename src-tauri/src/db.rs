use std::path::Path;

use rusqlite::Connection;

use crate::models::{ContextInfo, Note};

/// Opens (or creates) the SQLite database under `data_dir/notes.db`.
///
/// The database file is created with access permissions that grant read/write
/// only to the current user (0600 on Unix). The schema is created by
/// `init`/ticket #1.
pub fn open(data_dir: &Path) -> Result<Connection, String> {
    std::fs::create_dir_all(data_dir).map_err(|e| e.to_string())?;
    let db_path = data_dir.join("notes.db");
    create_file_0600(&db_path).map_err(|e| e.to_string())?;
    Connection::open(&db_path).map_err(|e| e.to_string())
}

/// Creates the database file with user-only permissions (0600 on Unix) if it
/// does not exist yet. Leaves an existing file untouched.
#[cfg(unix)]
fn create_file_0600(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;

    if path.exists() {
        return Ok(());
    }
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .mode(0o600)
        .open(path)?;
    Ok(())
}

/// Creates the database file if it does not exist yet (permissions are not
/// controllable on non-Unix platforms).
#[cfg(not(unix))]
fn create_file_0600(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        return Ok(());
    }
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)?;
    Ok(())
}

/// Initializes the database schema.
///
/// Stub: the schema creation is implemented by ticket #1 (Zettel parken).
pub fn init(_conn: &Connection) {}

/// Persists a note for the given context.
///
/// Stub: real persistence is implemented by ticket #1. Returns a placeholder
/// note so the command wiring stays observable without crashing.
pub fn save_note(_conn: &Connection, text: String, ctx: &ContextInfo) -> Result<Note, String> {
    Ok(Note {
        id: 0,
        repo_path: ctx.repo_path.clone(),
        branch: ctx.branch.clone(),
        commit_hash: ctx.commit_hash.clone(),
        changed_files: ctx.changed_files.clone(),
        note_text: text,
        created_at: 0,
        done: false,
    })
}

/// Lists notes, optionally filtered by a search string.
///
/// Stub: real querying is implemented by ticket #4. Returns an empty list, which
/// is exactly the empty state the skeleton UI must render.
pub fn list_notes(_conn: &Connection, _search: &str) -> Result<Vec<Note>, String> {
    Ok(Vec::new())
}

/// Lists the distinct repository paths present in the database.
///
/// Stub: implemented by ticket #4.
#[allow(dead_code)]
pub fn list_known_repos(_conn: &Connection) -> Result<Vec<String>, String> {
    Ok(Vec::new())
}

/// Returns the most recent note for a repository/branch pair, if any.
///
/// Stub: implemented by ticket #2 (branch-change notification).
#[allow(dead_code)]
pub fn latest_for_branch(
    _conn: &Connection,
    _repo: &str,
    _branch: &str,
) -> Result<Option<Note>, String> {
    Ok(None)
}
