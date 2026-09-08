# Parkplatz

Parkplatz ist ein Tray-Tool für Entwickler, das per globalem Hotkey den aktuellen
Arbeitskontext (Repository, Branch, Commit-Hash, geänderte Dateien) zusammen mit
einer Zeile Freitext als „Zettel" in einer lokalen SQLite-Datei parkt. Wechselt man
später zurück auf denselben Branch, erinnert eine unaufdringliche Benachrichtigung
an den geparkten Zettel. Die App läuft dauerhaft im System-Tray, benötigt kein
Konto, keinen Server und initiiert keinerlei Netzwerkverbindungen — alle Daten
bleiben ausschließlich auf dem lokalen Gerät.

## Tech-Stack

- **Sprache:** Rust
- **Framework:** Tauri 2 (Rust-Backend in `src-tauri/`, Vanilla-Frontend in `src/`)
- **Speicherung:** SQLite (rusqlite)
- **Git:** libgit2
- **Watcher:** notify

## Installation

Voraussetzung ist eine Rust-Toolchain (mindestens 1.77) mit Cargo sowie die
Systemabhängigkeiten von Tauri (siehe <https://v2.tauri.app/start/prerequisites/>).

Darüber hinaus sind keine Schritte nötig: Abhängigkeiten werden beim ersten Build
automatisch geladen. Es sind **keine Umgebungsvariablen** erforderlich.

## Ausführen (Entwicklung)

```bash
cargo run --manifest-path src-tauri/Cargo.toml
```

## Build für Produktion

```bash
cargo build --release --manifest-path src-tauri/Cargo.toml
```

Das Binärprogramm liegt danach unter `src-tauri/target/release/`.

## Bedienung

- Die App startet minimiert als Tray-Icon und läuft dauerhaft im Tray.
- **Globaler Hotkey `Strg+Alt+P`:** öffnet/fokussiert das Fenster und zeigt das
  Eingabefeld zum Parken eines Zettels.
- **Eingabe + Enter** parkt den Zettel; das Eingabefeld verschwindet wieder.
- Der Bereich **„Zuletzt geparkt"** zeigt den neuesten Zettel.
- Das **Suchfeld** filtert die Liste; Zettel lassen sich abhaken und löschen.
- Der Button **„Exportieren"** exportiert alle Zettel als JSON-Datei.
- **Tray-Eintrag → „Beenden"** beendet die App.

## Features

- Globaler Hotkey zum schnellen Parken eines Kontext-Zettels
- Automatische Kontexterfassung (Repository, Branch, Commit-Hash, geänderte Dateien)
- Lokale SQLite-Ablage mit Benutzer-Berechtigungen (Modus 0600)
- Durchsuchbare Zettelliste mit Abhaken und Löschen
- Benachrichtigung beim Wechsel auf einen Branch mit geparktem Zettel
- JSON-Export aller Zettel
- Kein Konto, kein Server, keine Netzwerkverbindungen

## Datenschutz

Die Anwendung initiiert keine Netzwerkverbindungen. Zetteltext, Repository-Pfad,
Branch, Commit-Hash und geänderte Dateien bleiben ausschließlich lokal und erscheinen
nicht in Logdateien oder auf der Standardausgabe.
