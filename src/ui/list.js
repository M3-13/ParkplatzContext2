(function () {
  "use strict";

  window.Parkplatz = window.Parkplatz || {};
  window.Parkplatz.ui = window.Parkplatz.ui || {};

  // All database text is rendered as plain text (textContent, never
  // innerHTML), so note content cannot inject HTML/JavaScript.
  function createNoteElement(note) {
    const li = document.createElement("li");
    li.className = "note-item";

    const text = document.createElement("span");
    text.className = "note-text";
    text.textContent = note.note_text;
    li.appendChild(text);

    return li;
  }

  window.Parkplatz.ui.list = {
    render: function (notes) {
      const listEl = document.getElementById("notes-list");
      const emptyEl = document.getElementById("empty-state");
      if (!listEl) {
        return;
      }

      listEl.textContent = "";

      if (!notes || notes.length === 0) {
        if (emptyEl) {
          emptyEl.hidden = false;
        }
        return;
      }

      if (emptyEl) {
        emptyEl.hidden = true;
      }

      notes.forEach(function (note) {
        listEl.appendChild(createNoteElement(note));
      });
    },

    setRecent: function (note) {
      const recentEl = document.getElementById("recent-note");
      if (!recentEl) {
        return;
      }
      recentEl.textContent = note ? note.note_text : "\u2013";
    }
  };
})();
