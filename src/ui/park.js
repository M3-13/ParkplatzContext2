(function () {
  "use strict";

  window.Parkplatz = window.Parkplatz || {};
  window.Parkplatz.ui = window.Parkplatz.ui || {};

  function panel() {
    return document.getElementById("park-panel");
  }

  function input() {
    return document.getElementById("note-input");
  }

  window.Parkplatz.ui.park = {
    // Reveals and focuses the hidden input field (invoked by the hotkey).
    show: function () {
      const p = panel();
      if (p) {
        p.hidden = false;
      }
      const i = input();
      if (i) {
        i.focus();
      }
    },

    // Submits the note text via the save_note command, refreshes the list and
    // hides the input field again.
    submit: async function (text) {
      const ipc = window.Parkplatz.ipc;
      const trimmed = (text || "").trim();
      if (!trimmed) {
        return;
      }

      if (ipc) {
        try {
          const note = await ipc.saveNote(trimmed);
          if (window.Parkplatz.ui.list && window.Parkplatz.ui.list.setRecent) {
            window.Parkplatz.ui.list.setRecent(note);
          }
        } catch (err) {
          if (window.Parkplatz.setStatus) {
            window.Parkplatz.setStatus("Speichern fehlgeschlagen.");
          }
        }
      }

      const i = input();
      if (i) {
        i.value = "";
      }
      const p = panel();
      if (p) {
        p.hidden = true;
      }

      if (window.Parkplatz.refresh) {
        window.Parkplatz.refresh();
      }
    }
  };
})();
