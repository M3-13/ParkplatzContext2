// src/ui/export.js
//
// Wires the "Export" button to the Tauri `export_notes` command and shows the
// resulting file path as a brief confirmation in the window.

// The button the Tray-App scaffold renders for the export action.
const EXPORT_BUTTON_ID = "export-btn";

// How long (ms) the confirmation stays visible before fading.
const CONFIRMATION_TTL_MS = 5000;

let wired = false;
let hideTimer = null;

function confirmationElement() {
  let el = document.getElementById("export-confirmation");
  if (!el) {
    el = document.createElement("div");
    el.id = "export-confirmation";
    el.setAttribute("role", "status");
    document.body.appendChild(el);
  }
  return el;
}

function showConfirmation(message) {
  const el = confirmationElement();
  // textContent, never innerHTML: note text / paths are treated as plain text.
  el.textContent = message;

  if (hideTimer) {
    clearTimeout(hideTimer);
  }
  hideTimer = setTimeout(() => {
    el.textContent = "";
  }, CONFIRMATION_TTL_MS);
}

/**
 * Attaches the export click handler. Idempotent: safe to call more than once
 * (e.g. from the scaffold's main.js after a re-render).
 */
export function initExport() {
  if (wired) {
    return;
  }

  const button = document.getElementById(EXPORT_BUTTON_ID);
  if (!button) {
    return;
  }

  wired = true;

  button.addEventListener("click", async () => {
    try {
      const path = await window.__TAURI__.core.invoke("export_notes");
      showConfirmation(`Export gespeichert: ${path}`);
    } catch (error) {
      showConfirmation(`Export fehlgeschlagen: ${error}`);
    }
  });
}
