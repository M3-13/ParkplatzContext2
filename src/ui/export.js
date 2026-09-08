(function () {
  "use strict";

  window.Parkplatz = window.Parkplatz || {};
  window.Parkplatz.ui = window.Parkplatz.ui || {};

  window.Parkplatz.ui.export = {
    run: async function () {
      const ipc = window.Parkplatz.ipc;
      if (!ipc) {
        return;
      }

      try {
        const path = await ipc.exportNotes();
        if (window.Parkplatz.setStatus) {
          window.Parkplatz.setStatus(
            path ? "Exportiert nach: " + path : "Export abgeschlossen."
          );
        }
      } catch (err) {
        if (window.Parkplatz.setStatus) {
          window.Parkplatz.setStatus("Export nicht verfügbar.");
        }
      }
    }
  };
})();
