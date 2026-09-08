use std::fs;
use std::path::{Path, PathBuf};

use tauri::State;

use crate::config;
use crate::db;
use crate::models::{AppState, Note};

/// Writes `notes` as a pretty-printed JSON array into `dir` using a
/// timestamped file name and returns the path of the written file.
///
/// Every field of `Note` (id, repo_path, branch, commit_hash, changed_files,
/// note_text, created_at, done) is serialized because `Note` derives
/// `Serialize`.
fn export_to_dir(notes: &[Note], dir: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(dir)
        .map_err(|e| format!("Export-Verzeichnis konnte nicht angelegt werden: {e}"))?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let path = dir.join(format!("parkplatz_notizen_{timestamp}.json"));

    let json = serde_json::to_string_pretty(notes)
        .map_err(|e| format!("Zettel konnten nicht serialisiert werden: {e}"))?;

    fs::write(&path, json).map_err(|e| format!("Export konnte nicht geschrieben werden: {e}"))?;

    Ok(path)
}

/// Tauri command: exports every parked note as a JSON array into a file under
/// the user data directory and returns the written file's path.
#[tauri::command]
pub fn export_notes(state: State<'_, AppState>) -> Result<String, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "Die Datenbank ist gerade gesperrt.".to_string())?;

    let notes: Vec<Note> = db::list_notes(&conn, "").map_err(|e| e.to_string())?;

    let dir = config::data_dir().join("exports");
    let path = export_to_dir(&notes, &dir)?;

    Ok(path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::export_to_dir;
    use crate::models::Note;

    fn sample_notes() -> Vec<Note> {
        vec![
            Note {
                id: 1,
                repo_path: "/home/dev/repo".to_string(),
                branch: "main".to_string(),
                commit_hash: "abc123".to_string(),
                changed_files: vec!["src/a.rs".to_string(), "src/b.rs".to_string()],
                note_text: "Work in progress".to_string(),
                created_at: 1_700_000_000,
                done: false,
            },
            Note {
                id: 2,
                repo_path: "/home/dev/repo".to_string(),
                branch: "feature/x".to_string(),
                commit_hash: "def456".to_string(),
                changed_files: vec![],
                note_text: "Done here".to_string(),
                created_at: 1_700_000_100,
                done: true,
            },
        ]
    }

    #[test]
    fn exports_every_note_with_all_fields_as_json_array() {
        let dir = std::env::temp_dir().join(format!("parkplatz_test_{}", std::process::id()));
        let path = export_to_dir(&sample_notes(), &dir).unwrap();

        let raw = std::fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();

        let arr = parsed.as_array().expect("top-level JSON must be an array");
        assert_eq!(arr.len(), 2);

        let first = &arr[0];
        assert_eq!(first["id"], 1);
        assert_eq!(first["repo_path"], "/home/dev/repo");
        assert_eq!(first["branch"], "main");
        assert_eq!(first["commit_hash"], "abc123");
        assert_eq!(first["changed_files"].as_array().unwrap().len(), 2);
        assert_eq!(first["note_text"], "Work in progress");
        assert_eq!(first["created_at"], 1_700_000_000);
        assert_eq!(first["done"], false);

        assert_eq!(arr[1]["done"], true);
        assert!(arr[1]["changed_files"].as_array().unwrap().is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
