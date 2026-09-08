// Parkplatz — JSON export of all parked notes.
//
// Wires the "Export" button to the Tauri `export_notes` command and shows the
// resulting file path as a brief confirmation in the window. It talks to the
// Rust backend exclusively through Tauri `invoke` and renders every string via
// `textContent`, never `innerHTML` (AC-13).
(function () {
  'use strict';

  // The button the Tray-App scaffold renders for the export action.
  const EXPORT_BUTTON_ID = 'export-btn';
  const CONFIRMATION_ID = 'export-confirmation';
  // How long (ms) the confirmation stays visible before it clears itself.
  const CONFIRMATION_TTL_MS = 5000;

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

  let hideTimer = null;

  // Return the (single, reused) confirmation element, creating it on first use.
  function confirmationElement() {
    let node = document.getElementById(CONFIRMATION_ID);
    if (!node) {
      node = el('div', 'export-confirmation');
      node.id = CONFIRMATION_ID;
      node.setAttribute('role', 'status');
      document.body.appendChild(node);
    }
    return node;
  }

  // Show a transient, user-readable message, cleared after CONFIRMATION_TTL_MS.
  function showConfirmation(message) {
    const node = confirmationElement();
    setText(node, message);
    if (hideTimer) {
      clearTimeout(hideTimer);
    }
    hideTimer = setTimeout(() => setText(node, ''), CONFIRMATION_TTL_MS);
  }

  // Initialize the export wiring: attach a click handler to the export button
  // that invokes `export_notes` and shows the written file path. Idempotent and
  // re-runnable: the wired state lives on the button element itself, so a
  // re-rendered button is wired again, and the returned cleanup function undoes
  // the wiring.
  function initExport(root) {
    const invoke = getInvoke();
    if (!invoke) {
      return function cleanup() {};
    }

    const button =
      document.getElementById(EXPORT_BUTTON_ID) ||
      (root && root.querySelector && root.querySelector('#' + EXPORT_BUTTON_ID)) ||
      (root && root.querySelector && root.querySelector('button[data-export]'));

    if (!button) {
      return function cleanup() {};
    }

    if (button.dataset.exportWired === 'true') {
      return function cleanup() {};
    }
    button.dataset.exportWired = 'true';

    const onClick = async () => {
      try {
        const path = await invoke('export_notes');
        showConfirmation('Export gespeichert: ' + path);
      } catch (err) {
        showConfirmation('Export fehlgeschlagen: ' + err);
      }
    };

    button.addEventListener('click', onClick);

    return function cleanup() {
      button.removeEventListener('click', onClick);
      delete button.dataset.exportWired;
    };
  }

  // Public API. Works both as a plain script and an ES module: the module
  // scope still sees the global `window`, so the entry point can always reach
  // `window.ParkplatzExport.init`.
  window.ParkplatzExport = {
    init: initExport,
  };
})();
