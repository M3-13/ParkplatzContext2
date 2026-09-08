use rusqlite::Connection;

/// Exports all notes as a JSON file and returns the path to that file.
///
/// Stub: the real export is implemented by ticket #3 (JSON-Export).
pub fn export_notes(_conn: &Connection) -> Result<String, String> {
    Ok(String::new())
}
