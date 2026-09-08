use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, Result};

use crate::models::{ContextInfo, Note};

/// Name of the SQLite database file inside the data directory.
const DB_FILE: &str = "notes.db";

/// Returns the path of the SQLite database file within the given data
/// directory (`dirs::data_dir()/parkplatz`).
pub fn db_path(data_dir: &Path) -> PathBuf {
    data_dir.join(DB_FILE)
}

/// Creates the database file (if missing) with owner-only permissions so that
/// SQLite opens a file that is not readable or writable by other users.
fn precreate_owner_only(path: &Path) -> std::io::Result<()> {
    let file = fs::OpenOptions::new().write(true).create(true).open(path)?;
    drop(file);
    set_owner_only_permissions(path)
}

/// Sets file permissions so that only the executing user may read and write
/// the file (mode 0600 under Unix). On non-Unix platforms this is a no-op.
#[cfg(unix)]
fn set_owner_only_permissions(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn set_owner_only_permissions(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

/// Opens (creating if necessary) the SQLite database at
/// `<data_dir>/notes.db` with owner-only permissions, and ensures the schema
/// exists. Called once at startup.
pub fn open(data_dir: &Path) -> Result<Connection> {
    fs::create_dir_all(data_dir)
        .map_err(|e| rusqlite::Error::InvalidParameterName(format!("create data dir: {e}")))?;

    let path = db_path(data_dir);
    precreate_owner_only(&path)
        .map_err(|e| rusqlite::Error::InvalidParameterName(format!("create db file: {e}")))?;

    let conn = Connection::open(&path)?;
    // Re-apply after opening in case SQLite re-created or touched the file.
    let _ = set_owner_only_permissions(&path);

    init(&conn)?;
    Ok(conn)
}

/// Creates the `notes` table if it does not already exist.
pub fn init(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            repo_path     TEXT NOT NULL,
            branch        TEXT NOT NULL,
            commit_hash   TEXT NOT NULL,
            changed_files TEXT NOT NULL,
            note_text     TEXT NOT NULL,
            created_at    INTEGER NOT NULL,
            done          INTEGER NOT NULL DEFAULT 0
        )",
        params![],
    )?;
    Ok(())
}

/// Inserts a new note together with the captured context and returns it.
///
/// All free-text values are bound as parameters; nothing is interpolated into
/// the SQL string. The caller is responsible for signalling the watcher via
/// `repos_changed_tx` after a successful save.
pub fn save_note(conn: &Connection, text: &str, ctx: &ContextInfo) -> Result<Note> {
    let changed_files = serde_json::to_string(&ctx.changed_files)
        .map_err(|e| rusqlite::Error::InvalidParameterName(format!("serialize changed_files: {e}")))?;
    let created_at = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO notes (repo_path, branch, commit_hash, changed_files, note_text, created_at, done)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)",
        params![
            ctx.repo_path,
            ctx.branch,
            ctx.commit_hash,
            changed_files,
            text,
            created_at
        ],
    )?;

    let id = conn.last_insert_rowid();
    Ok(Note {
        id,
        repo_path: ctx.repo_path.clone(),
        branch: ctx.branch.clone(),
        commit_hash: ctx.commit_hash.clone(),
        changed_files: ctx.changed_files.clone(),
        note_text: text.to_string(),
        created_at,
        done: false,
    })
}

/// Lists all notes matching the given search term (matched against note text,
/// repository path and branch), newest first. An empty search returns all.
pub fn list_notes(conn: &Connection, search: &str) -> Result<Vec<Note>> {
    let pattern = format!("%{}%", search);
    let mut stmt = conn.prepare(
        "SELECT id, repo_path, branch, commit_hash, changed_files, note_text, created_at, done
         FROM notes
         WHERE note_text LIKE ?1 OR repo_path LIKE ?1 OR branch LIKE ?1
         ORDER BY created_at DESC, id DESC",
    )?;

    let rows = stmt.query_map(params![pattern], |row| {
        Ok(Note {
            id: row.get(0)?,
            repo_path: row.get(1)?,
            branch: row.get(2)?,
            commit_hash: row.get(3)?,
            changed_files: deserialize_changed_files(&row.get::<_, String>(4)?),
            note_text: row.get(5)?,
            created_at: row.get(6)?,
            done: row.get::<_, i64>(7)? != 0,
        })
    })?;

    rows.collect()
}

/// Lists all distinct repository paths that have at least one note.
pub fn list_known_repos(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT DISTINCT repo_path FROM notes ORDER BY repo_path")?;
    let rows = stmt.query_map(params![], |row| row.get::<_, String>(0))?;
    rows.collect()
}

/// Returns the newest un-done note for the given repository and branch, if any.
pub fn latest_for_branch(conn: &Connection, repo: &str, branch: &str) -> Result<Option<Note>> {
    let note = conn.query_row(
        "SELECT id, repo_path, branch, commit_hash, changed_files, note_text, created_at, done
         FROM notes
         WHERE repo_path = ?1 AND branch = ?2 AND done = 0
         ORDER BY created_at DESC, id DESC
         LIMIT 1",
        params![repo, branch],
        |row| {
            Ok(Note {
                id: row.get(0)?,
                repo_path: row.get(1)?,
                branch: row.get(2)?,
                commit_hash: row.get(3)?,
                changed_files: deserialize_changed_files(&row.get::<_, String>(4)?),
                note_text: row.get(5)?,
                created_at: row.get(6)?,
                done: row.get::<_, i64>(7)? != 0,
            })
        },
    );

    match note {
        Ok(n) => Ok(Some(n)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

fn deserialize_changed_files(raw: &str) -> Vec<String> {
    serde_json::from_str(raw).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let c = Connection::open_in_memory().expect("in-memory connection");
        init(&c).expect("init schema");
        c
    }

    fn ctx(repo: &str, branch: &str) -> ContextInfo {
        ContextInfo {
            repo_path: repo.to_string(),
            branch: branch.to_string(),
            commit_hash: "abc123".to_string(),
            changed_files: vec!["a.rs".to_string(), "b.rs".to_string()],
        }
    }

    #[test]
    fn save_and_list_round_trip() {
        let c = conn();
        let note = save_note(&c, "hello world", &ctx("/repo", "main")).expect("save");
        assert!(note.id > 0);
        assert!(note.created_at > 0);

        let all = list_notes(&c, "").expect("list");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].note_text, "hello world");
        assert_eq!(all[0].changed_files, vec!["a.rs", "b.rs"]);
        assert_eq!(all[0].done, false);
    }

    #[test]
    fn search_filters_by_text_repo_and_branch() {
        let c = conn();
        save_note(&c, "fix login", &ctx("/repo", "main")).unwrap();
        save_note(&c, "deploy notes", &ctx("/other", "release")).unwrap();

        assert_eq!(list_notes(&c, "login").unwrap().len(), 1);
        assert_eq!(list_notes(&c, "/other").unwrap().len(), 1);
        assert_eq!(list_notes(&c, "release").unwrap().len(), 1);
        assert_eq!(list_notes(&c, "missing").unwrap().len(), 0);
    }

    #[test]
    fn list_known_repos_returns_distinct_sorted() {
        let c = conn();
        save_note(&c, "a", &ctx("/beta", "main")).unwrap();
        save_note(&c, "b", &ctx("/alpha", "main")).unwrap();
        save_note(&c, "c", &ctx("/alpha", "dev")).unwrap();

        assert_eq!(
            list_known_repos(&c).unwrap(),
            vec!["/alpha".to_string(), "/beta".to_string()]
        );
    }

    #[test]
    fn latest_for_branch_returns_newest_unfinished() {
        let c = conn();
        save_note(&c, "first", &ctx("/repo", "main")).unwrap();
        // Manually age the first note so ordering is deterministic.
        c.execute("UPDATE notes SET created_at = 1 WHERE note_text = 'first'", params![])
            .unwrap();
        save_note(&c, "second", &ctx("/repo", "main")).unwrap();

        let latest = latest_for_branch(&c, "/repo", "main")
            .unwrap()
            .expect("a note");
        assert_eq!(latest.note_text, "second");

        // None for an unknown branch.
        assert!(latest_for_branch(&c, "/repo", "unknown").unwrap().is_none());
    }

    #[test]
    fn save_note_uses_bound_parameters() {
        // A note text containing SQL fragments and quotes must be stored and
        // read back verbatim, proving it is bound rather than interpolated.
        let c = conn();
        let text = "Robert'); DROP TABLE notes; --";
        save_note(&c, text, &ctx("/repo", "main")).unwrap();
        let all = list_notes(&c, "").unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].note_text, text);
    }
}
