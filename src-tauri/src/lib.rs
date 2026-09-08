mod config;
mod db;
mod db_manage;
mod export;
mod git_context;
mod models;
mod notification;
mod watcher;

use std::sync::Mutex;

use models::{AppState, ContextInfo, Note};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

#[tauri::command]
fn save_note(note_text: String, state: tauri::State<AppState>) -> Result<Note, String> {
    let ctx = git_context::get_context(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let note = db::save_note(&conn, note_text, &ctx)?;
    let _ = state.repos_changed_tx.send(());
    Ok(note)
}

#[tauri::command]
fn list_notes(search: String, state: tauri::State<AppState>) -> Result<Vec<Note>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::list_notes(&conn, &search)
}

#[tauri::command]
fn toggle_note_done(id: i64, done: bool, state: tauri::State<AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db_manage::toggle_note_done(&conn, id, done)
}

#[tauri::command]
fn delete_note(id: i64, state: tauri::State<AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db_manage::delete_note(&conn, id)
}

#[tauri::command]
fn get_context(state: tauri::State<AppState>) -> Result<ContextInfo, String> {
    git_context::get_context(&state)
}

#[tauri::command]
fn export_notes(state: tauri::State<AppState>) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    export::export_notes(&conn)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Channel: a note save signals the watcher to resync its watch list.
            let (tx, rx) = std::sync::mpsc::channel::<()>();

            let data_dir = crate::config::data_dir();
            let conn = db::open(&data_dir)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            db::init(&conn);

            let state = AppState {
                db: Mutex::new(conn),
                active_repo: Mutex::new(None),
                repos_changed_tx: tx,
            };

            watcher::start_watcher(&state, rx);
            app.manage(state);

            setup_tray(app)?;
            setup_hotkey(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            save_note,
            list_notes,
            toggle_note_done,
            delete_note,
            get_context,
            export_notes
        ])
        .run(tauri::generate_context!())
        .expect("error while running parkplatz");
}

fn setup_tray(app: &mut tauri::App) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::TrayIconBuilder;

    let quit_i = MenuItem::with_id(app, "quit", "Beenden", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit_i])?;

    TrayIconBuilder::with_id("parkplatz-tray")
        .icon(app.default_window_icon().expect("app icon").clone())
        .tooltip("Parkplatz")
        .menu(&menu)
        .on_menu_event(|app, event| {
            if event.id.as_ref() == "quit" {
                app.exit(0);
            }
        })
        .build(app)?;

    Ok(())
}

fn setup_hotkey(app: &mut tauri::App) -> tauri::Result<()> {
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyP);

    app.global_shortcut()
        .on_shortcut(shortcut, |app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.emit("park-hotkey", ());
                }
            }
        })?;

    Ok(())
}
