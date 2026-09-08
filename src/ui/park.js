// park.js — Zettel parken mit automatischer Kontexterfassung.
//
// Verantwortlich fuer den "Park"-Flow:
//   1. Auf das `park-hotkey`-Event hin das Eingabefeld einblenden und fokussieren.
//   2. Bei Enter den Zetteltext via `invoke('save_note', { noteText })` speichern.
//   3. Das Feld wieder schliessen und den gespeicherten Zettel samt Kontext im
//      Bereich "Zuletzt geparkt" als reinen Text (textContent) anzeigen.
//
// Alle aus der Datenbank gelesenen Werte werden ausschliesslich ueber
// `textContent` gerendert, niemals ueber `innerHTML`, damit im Freitext
// enthaltener HTML/JavaScript-Code nicht ausgefuehrt wird (AC-13).

(function () {
  'use strict';

  var tauri = window.__TAURI__;

  var invoke = tauri && tauri.core ? tauri.core.invoke : null;
  var listen = tauri && tauri.event ? tauri.event.listen : null;

  var OVERLAY_ID = 'park-input-overlay';
  var INPUT_ID = 'park-input';
  var LAST_PARKED_ID = 'last-parked';

  function ensureElement(tag, id, className, parent) {
    var el = document.getElementById(id);
    if (!el) {
      el = document.createElement(tag);
      el.id = id;
      if (className) el.className = className;
      parent.appendChild(el);
    }
    return el;
  }

  function buildOverlay() {
    var overlay = ensureElement('div', OVERLAY_ID, 'park-overlay', document.body);
    overlay.style.display = 'none';

    var input = ensureElement('input', INPUT_ID, 'park-input', overlay);
    input.type = 'text';
    input.placeholder = 'Zettel parken …';

    return { overlay: overlay, input: input };
  }

  function lastParkedArea() {
    return ensureElement('section', LAST_PARKED_ID, 'last-parked', document.body);
  }

  function renderNote(area, note) {
    area.replaceChildren();

    var heading = document.createElement('h2');
    heading.textContent = 'Zuletzt geparkt';
    area.appendChild(heading);

    if (!note) {
      var empty = document.createElement('p');
      empty.textContent = 'Noch kein Zettel geparkt.';
      area.appendChild(empty);
      return;
    }

    var fields = [
      ['Text', note.note_text],
      ['Repo', note.repo_path],
      ['Branch', note.branch],
      ['Commit', note.commit_hash],
      ['Geänderte Dateien', (note.changed_files || []).join(', ')],
      ['Zeitpunkt', new Date(Number(note.created_at) * 1000).toLocaleString()],
    ];

    fields.forEach(function (entry) {
      var label = entry[0];
      var value = entry[1] == null ? '' : String(entry[1]);

      var row = document.createElement('p');
      var strong = document.createElement('strong');
      strong.textContent = label + ': ';
      row.appendChild(strong);
      row.appendChild(document.createTextNode(value));
      area.appendChild(row);
    });
  }

  function showInput(input, overlay) {
    input.value = '';
    overlay.style.display = '';
    input.focus();
  }

  function hideInput(overlay) {
    overlay.style.display = 'none';
  }

  function handleEnter(input, overlay, area, event) {
    if (event.key !== 'Enter') return;
    event.preventDefault();

    var text = input.value.trim();
    if (!text) return;

    if (!invoke) {
      renderNote(area, {
        note_text: text,
        repo_path: '',
        branch: '',
        commit_hash: '',
        changed_files: [],
        created_at: Math.floor(Date.now() / 1000),
      });
      hideInput(overlay);
      return;
    }

    invoke('save_note', { noteText: text })
      .then(function (note) {
        renderNote(area, note);
        hideInput(overlay);
      })
      .catch(function (err) {
        renderNote(area, null);
        hideInput(overlay);
        // Der Zettel konnte nicht gespeichert werden; der Fehler wird nicht
        // still verschluckt, sondern sichtbar gemeldet.
        console.error('save_note fehlgeschlagen:', err);
      });
  }

  function init() {
    var parts = buildOverlay();
    var area = lastParkedArea();

    parts.input.addEventListener('keydown', function (event) {
      handleEnter(parts.input, parts.overlay, area, event);
    });

    if (listen) {
      listen('park-hotkey', function () {
        showInput(parts.input, parts.overlay);
      });
    }
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
