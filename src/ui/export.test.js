// Tests for src/ui/export.js (JSON export button wiring).
//
// Dependency-free: uses Node's built-in test runner and a minimal hand-rolled
// DOM/window mock (no jsdom). The IIFE in export.js is evaluated inside a
// `vm` context so it can be exercised against the mock instead of a browser.
//
// Run with: node --test src/ui/export.test.js
'use strict';

const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

const source = readFileSync(path.join(__dirname, 'export.js'), 'utf8');

function createMockDom() {
  const byId = new Map();
  const listeners = new WeakMap();

  function makeElement(id) {
    return {
      id: id || '',
      dataset: {},
      className: '',
      textContent: '',
      attributes: {},
      children: [],
      appendChild(child) {
        this.children.push(child);
        if (child.id) {
          byId.set(child.id, child);
        }
        return child;
      },
      setAttribute(key, value) {
        this.attributes[key] = String(value);
      },
      addEventListener(type, fn) {
        let map = listeners.get(this);
        if (!map) {
          map = {};
          listeners.set(this, map);
        }
        map[type] = fn;
      },
      removeEventListener(type, fn) {
        const map = listeners.get(this);
        if (map && map[type] === fn) {
          delete map[type];
        }
      },
    };
  }

  const body = makeElement('body');
  const exportButton = makeElement('export-btn');
  byId.set('body', body);
  byId.set('export-btn', exportButton);

  const document = {
    body,
    getElementById(id) {
      return byId.get(id) || null;
    },
    createElement(tag) {
      return makeElement(tag);
    },
  };

  const invokeCalls = [];
  const window = {
    __TAURI__: {
      core: {
        invoke(cmd, args) {
          invokeCalls.push({ cmd, args });
          return Promise.resolve('C:/parkplatz/exports/parkplatz_notizen_20260908.json');
        },
      },
    },
  };

  function click(element) {
    const map = listeners.get(element);
    const fn = map && map.click;
    if (!fn) {
      throw new Error('no click listener registered');
    }
    return fn();
  }

  return { byId, document, window, exportButton, invokeCalls, click };
}

function load(ctx) {
  const sandbox = {
    window: ctx.window,
    document: ctx.document,
    setTimeout,
    clearTimeout,
  };
  vm.createContext(sandbox);
  vm.runInContext(source, sandbox, { filename: 'export.js' });
  return ctx.window.ParkplatzExport;
}

test('wires the export button and shows the written path', async () => {
  const ctx = createMockDom();
  const mod = load(ctx);

  mod.init(ctx.document.body);
  await ctx.click(ctx.exportButton);

  assert.equal(ctx.invokeCalls.length, 1);
  assert.equal(ctx.invokeCalls[0].cmd, 'export_notes');
  assert.equal(ctx.invokeCalls[0].args, undefined);

  const confirmation = ctx.byId.get('export-confirmation');
  assert.ok(confirmation, 'confirmation element created');
  assert.match(confirmation.textContent, /Export gespeichert:/);
  assert.match(confirmation.textContent, /parkplatz_notizen_20260908\.json/);
});

test('shows a readable error when the export fails', async () => {
  const ctx = createMockDom();
  ctx.window.__TAURI__.core.invoke = () => Promise.reject(new Error('boom'));
  const mod = load(ctx);

  mod.init(ctx.document.body);
  await ctx.click(ctx.exportButton);

  const confirmation = ctx.byId.get('export-confirmation');
  assert.ok(confirmation, 'confirmation element created');
  assert.match(confirmation.textContent, /Export fehlgeschlagen:/);
});

test('init is idempotent on the same button', async () => {
  const ctx = createMockDom();
  const mod = load(ctx);

  mod.init(ctx.document.body);
  mod.init(ctx.document.body);
  await ctx.click(ctx.exportButton);

  assert.equal(ctx.invokeCalls.length, 1);
});

test('cleanup allows re-wiring after a re-render', async () => {
  const ctx = createMockDom();
  const mod = load(ctx);

  const cleanup = mod.init(ctx.document.body);
  cleanup();
  mod.init(ctx.document.body);
  await ctx.click(ctx.exportButton);

  assert.equal(ctx.invokeCalls.length, 1);
});

test('no-op when the Tauri API is unavailable', () => {
  const ctx = createMockDom();
  delete ctx.window.__TAURI__;
  const mod = load(ctx);

  mod.init(ctx.document.body);

  assert.equal(ctx.invokeCalls.length, 0);
});
