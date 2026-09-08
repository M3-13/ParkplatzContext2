use rusqlite::{params, Connection, Result};

/// Marks a note as done/undone by its primary key.
///
/// The `done` flag is bound as a parameter so no user-controlled value is ever
/// interpolated into the SQL string (AC-11).
pub fn toggle_note_done(conn: &Connection, id: i64, done: bool) -> Result<()> {
    conn.execute(
        "UPDATE notes SET done = ?1 WHERE id = ?2",
        params![done, id],
    )?;
    Ok(())
}

/// Permanently removes a note from the `notes` table (AC-08).
///
/// The row is deleted by primary key, bound as a parameter.
pub fn delete_note(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(
            "CREATE TABLE notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                done INTEGER NOT NULL DEFAULT 0
            );",
        )
        .expect("schema");
        conn
    }

    #[test]
    fn toggle_note_done_flips_flag() {
        let conn = test_conn();
        conn.execute("INSERT INTO notes (done) VALUES (?1)", params![false])
            .unwrap();
        let id = conn.last_insert_rowid();

        toggle_note_done(&conn, id, true).unwrap();

        let done: bool = conn
            .query_row("SELECT done FROM notes WHERE id = ?1", params![id], |r| {
                r.get(0)
            })
            .unwrap();
        assert!(done);
    }

    #[test]
    fn toggle_note_done_can_uncheck() {
        let conn = test_conn();
        conn.execute("INSERT INTO notes (done) VALUES (?1)", params![true])
            .unwrap();
        let id = conn.last_insert_rowid();

        toggle_note_done(&conn, id, false).unwrap();

        let done: bool = conn
            .query_row("SELECT done FROM notes WHERE id = ?1", params![id], |r| {
                r.get(0)
            })
            .unwrap();
        assert!(!done);
    }

    #[test]
    fn delete_note_removes_row_permanently() {
        let conn = test_conn();
        conn.execute("INSERT INTO notes (done) VALUES (?1)", params![false])
            .unwrap();
        let id = conn.last_insert_rowid();

        delete_note(&conn, id).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM notes WHERE id = ?1", params![id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn delete_nonexistent_id_is_a_noop_success() {
        let conn = test_conn();
        assert!(delete_note(&conn, 9999).is_ok());
    }
}
