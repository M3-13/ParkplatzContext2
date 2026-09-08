(function () {
  "use strict";

  const tauri = window.__TAURI__ || null;
  const invoke =
    tauri && tauri.core
      ? tauri.core.invoke.bind(tauri.core)
      : null;
  const listen =
    tauri && tauri.event
      ? tauri.event.listen.bind(tauri.event)
      : null;

  // IPC wrapper around the Rust commands registered in src-tauri/src/lib.rs.
  const ipc = {
    saveNote: function (text) {
      return invoke
        ? invoke("save_note", { noteText: text })
        : Promise.reject(new Error("Tauri API nicht verfügbar"));
    },
    listNotes: function (search) {
      return invoke
        ? invoke("list_notes", { search: search || "" })
        : Promise.resolve([]);
    },
    toggleNoteDone: function (id, done) {
      return invoke
        ? invoke("toggle_note_done", { id: id, done: done })
        : Promise.resolve();
    },
    deleteNote: function (id) {
      return invoke
        ? invoke("delete_note", { id: id })
        : Promise.resolve();
    },
    getContext: function () {
      return invoke
        ? invoke("get_context")
        : Promise.resolve({
            repo_path: "",
            branch: "",
            commit_hash: "",
            changed_files: []
          });
    },
    exportNotes: function () {
      return invoke
        ? invoke("export_notes")
        : Promise.reject(new Error("Tauri API nicht verfügbar"));
    }
  };

  window.Parkplatz = window.Parkplatz || {};
  window.Parkplatz.ipc = ipc;

  function refresh() {
    const search = document.getElementById("search-input");
    const query = search ? search.value : "";

    return ipc
      .listNotes(query)
      .then(function (notes) {
        const list = window.Parkplatz.ui && window.Parkplatz.ui.list;
        if (list) {
          list.render(notes);
          const newest = notes && notes.length ? notes[0] : null;
          if (list.setRecent) {
            list.setRecent(newest);
          }
        }
      })
      .catch(function () {
        const list = window.Parkplatz.ui && window.Parkplatz.ui.list;
        if (list) {
          list.render([]);
        }
      });
  }
  window.Parkplatz.refresh = refresh;

  function setStatus(message) {
    const el = document.getElementById("status");
    if (!el) {
      return;
    }
    el.textContent = message;
    el.hidden = false;
    window.clearTimeout(setStatus.timer);
    setStatus.timer = window.setTimeout(function () {
      el.hidden = true;
    }, 4000);
  }
  window.Parkplatz.setStatus = setStatus;

  function init() {
    const exportBtn = document.getElementById("export-button");
    if (exportBtn) {
      exportBtn.addEventListener("click", function () {
        if (window.Parkplatz.ui && window.Parkplatz.ui.export) {
          window.Parkplatz.ui.export.run();
        }
      });
    }

    const search = document.getElementById("search-input");
    if (search) {
      search.addEventListener("input", refresh);
    }

    const noteInput = document.getElementById("note-input");
    if (noteInput) {
      noteInput.addEventListener("keydown", function (ev) {
        if (ev.key === "Enter") {
          ev.preventDefault();
          if (window.Parkplatz.ui && window.Parkplatz.ui.park) {
            window.Parkplatz.ui.park.submit(noteInput.value);
          }
        }
      });
    }

    // Hotkey listener: the backend emits "park-hotkey" when the global
    // shortcut is pressed; reveal the parking input field.
    if (listen) {
      listen("park-hotkey", function () {
        if (window.Parkplatz.ui && window.Parkplatz.ui.park) {
          window.Parkplatz.ui.park.show();
        }
      }).catch(function () {});
    }

    refresh();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
