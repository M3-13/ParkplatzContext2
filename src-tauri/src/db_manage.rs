use rusqlite::Connection;

/// Marks a note as done or not done.
///
/// Stub: implemented by ticket #4 (note list).
pub fn toggle_note_done(_conn: &Connection, _id: i64, _done: bool) -> Result<(), String> {
    Ok(())
}

/// Permanently deletes a note from the database.
///
/// Stub: implemented by ticket #4 (note list).
pub fn delete_note(_conn: &Connection, _id: i64) -> Result<(), String> {
    Ok(())
}
