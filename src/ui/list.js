// Parkplatz — Zettelliste (search, grouping by repo, done-toggle, delete).
//
// This module renders the list of parked notes for the Parkplatz tray app.
// It talks to the Rust backend exclusively through Tauri `invoke` and renders
// every string read from the database via `textContent`, never `innerHTML`
// (AC-13). The search term is forwarded to the backend as an argument, where
// it is bound as a SQL parameter (AC-11).
(function () {
  'use strict';

  // Resolve the Tauri `invoke` function. Tauri v2 exposes it under
  // `window.__TAURI__.core.invoke` when running without a bundler.
  function getInvoke() {
    const tauri = window.__TAURI__;
    if (!tauri) {
      return null;
    }
    if (tauri.core && typeof tauri.core.invoke === 'function') {
      return tauri.core.invoke.bind(tauri.core);
    }
    if (typeof tauri.invoke === 'function') {
      return tauri.invoke.bind(tauri);
    }
    return null;
  }

  // Set the text of an element from a possibly untrusted string.
  // textContent guarantees the value is treated as text, not markup (AC-13).
  function setText(el, value) {
    el.textContent = value == null ? '' : String(value);
  }

  // Build an element with a tag, CSS class and optional text.
  function el(tag, className, text) {
    const node = document.createElement(tag);
    if (className) {
      node.className = className;
    }
    if (text != null) {
      setText(node, text);
    }
    return node;
  }

  // Group notes by `repo_path`, preserving the overall newest-first ordering.
  // Each group is itself sorted newest-first by `created_at`.
  function groupByRepo(notes) {
    const groups = [];
    const index = new Map();
    for (const note of notes) {
      const repo = note.repo_path || '';
      let group = index.get(repo);
      if (!group) {
        group = { repo_path: repo, notes: [] };
        index.set(repo, group);
        groups.push(group);
      }
      group.notes.push(note);
    }
    return groups;
  }

  function formatTimestamp(createdAt) {
    if (createdAt == null) {
      return '';
    }
    const date = new Date(createdAt * 1000);
    if (Number.isNaN(date.getTime())) {
      return '';
    }
    return date.toLocaleString();
  }

  function renderList(root, invoke, notes, onToggle, onDelete) {
    root.replaceChildren();

    if (!notes || notes.length === 0) {
      root.appendChild(el('div', 'note-list-empty', 'Keine Zettel gefunden.'));
      return;
    }

    const groups = groupByRepo(notes);

    for (const group of groups) {
      const section = el('section', 'note-group');
      const header = el('h3', 'note-group-header', group.repo_path);
      section.appendChild(header);

      for (const note of group.notes) {
        const item = el('div', note.done ? 'note-item note-item-done' : 'note-item');

        const body = el('div', 'note-body');
        body.appendChild(el('div', 'note-text', note.note_text));
        body.appendChild(
          el(
            'div',
            'note-meta',
            [note.branch, formatTimestamp(note.created_at)].filter(Boolean).join(' · '),
          ),
        );
        item.appendChild(body);

        const actions = el('div', 'note-actions');

        const toggleBtn = el(
          'button',
          'note-toggle',
          note.done ? 'Wieder öffnen' : 'Abhaken',
        );
        toggleBtn.type = 'button';
        toggleBtn.addEventListener('click', () => {
          if (typeof onToggle === 'function') {
            onToggle(note);
          }
        });
        actions.appendChild(toggleBtn);

        const deleteBtn = el('button', 'note-delete', 'Löschen');
        deleteBtn.type = 'button';
        deleteBtn.addEventListener('click', () => {
          if (typeof onDelete === 'function') {
            onDelete(note);
          }
        });
        actions.appendChild(deleteBtn);

        item.appendChild(actions);
        section.appendChild(item);
      }

      root.appendChild(section);
    }
  }

  // Initialize the list view inside `root`, wiring up search, toggle and
  // delete. Returns a cleanup function that removes the search listener.
  function initZettelliste(root, options) {
    const invoke = getInvoke();
    const opts = options || {};

    if (!invoke) {
      root.replaceChildren(
        el('div', 'note-list-error', 'Tauri API nicht verfügbar.'),
      );
      return function cleanup() {};
    }

    const container = el('div', 'note-list');
    const searchRow = el('div', 'note-search-row');
    const searchInput = el('input', 'note-search');
    searchInput.type = 'search';
    searchInput.placeholder = 'Zettel durchsuchen…';
    searchRow.appendChild(searchInput);
    container.appendChild(searchRow);

    const listRoot = el('div', 'note-list-items');
    container.appendChild(listRoot);

    root.replaceChildren(container);

    let currentSearch = '';

    async function load() {
      try {
        const notes = await invoke('list_notes', { search: currentSearch });
        const sorted = (notes || []).slice().sort((a, b) => (b.created_at || 0) - (a.created_at || 0));
        renderList(listRoot, invoke, sorted, onToggle, onDelete);
      } catch (err) {
        listRoot.replaceChildren(
          el('div', 'note-list-error', 'Zettel konnten nicht geladen werden.'),
        );
      }
    }

    async function onToggle(note) {
      try {
        await invoke('toggle_note_done', { id: note.id, done: !note.done });
        await load();
      } catch (err) {
        // Keep the previous list; nothing to render as text from a raw error.
      }
    }

    async function onDelete(note) {
      try {
        await invoke('delete_note', { id: note.id });
        await load();
      } catch (err) {
        // Keep the previous list.
      }
    }

    let debounceTimer = null;
    searchInput.addEventListener('input', () => {
      currentSearch = searchInput.value;
      if (debounceTimer) {
        clearTimeout(debounceTimer);
      }
      debounceTimer = setTimeout(load, 200);
    });

    load();

    return function cleanup() {
      if (debounceTimer) {
        clearTimeout(debounceTimer);
      }
    };
  }

  // Public API. Works both as a plain script and an ES module: the module
  // scope still sees the global `window`, so the entry point can always reach
  // `window.ParkplatzList.init`.
  window.ParkplatzList = {
    init: initZettelliste,
  };
})();
