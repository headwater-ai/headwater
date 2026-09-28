// SPDX-License-Identifier: Apache-2.0
//
// The one file of the VS Code extension that decides anything. It imports no
// `vscode`, so `node --test` holds every decision it makes.
//
// It spawns `<bin> mcp --root <root>` once per query. `headwater mcp` is stdio
// only, so there is no running server to find. It walks the corpus once when it
// starts, so a server kept alive across edits would answer from a stale walk.
// One session costs about 0.16 s on this repository.
//
// It calls two tools of the query class and no other: `route` and
// `governing_docs_for_path`. It never passes `--write`. `ask` refuses any other
// tool name before it spawns anything.
//
// It reads the pointers out of `structuredContent`, which
// `docs/interfaces/headwater-mcp.md` names as the machine contract of both
// tools, and never out of the text block. The text is for a person: a path may
// hold ` (` and a summary any word, so no parse of it is exact.
//
// It fails open. A spawn error, a non-zero exit, a timeout, `isError: true`, a
// JSON-RPC error, an answer without `structuredContent` (an engine older than
// #1248) or output that does not parse all resolve to `[]`. No call
// throws or rejects, except `ask` with a tool outside the allowlist, which is a
// defect in the caller and not a state of the workspace.

'use strict';

const { spawn } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');

/** The only tools this client sends. Nothing else can reach the server. */
const TOOLS = Object.freeze(['route', 'governing_docs_for_path']);

const DEFAULT_TIMEOUT_MS = 5000;
const MAX_OUTPUT = 1 << 20;

// The sentence `route` and `governing_docs_for_path` write after a pointer to a
// document nobody accepted, as `engine/crates/query/src/lib.rs` writes it. The
// structured pointer states the fact as `unwarranted`, and this is the one
// sentence of the server that this file copies.
const UNWARRANTED = 'nobody accepted this document';

// One element of `structuredContent.pointers` as the pointer this client
// hands on, or null when the element is not one.
function pointerOf(element) {
  if (!element || typeof element !== 'object' || typeof element.path !== 'string') return null;
  const text = (value) => (typeof value === 'string' ? value : null);
  return {
    path: element.path,
    name: text(element.name),
    summary: text(element.summary),
    asserted: element.unwarranted === true ? UNWARRANTED : null,
  };
}

/**
 * The pointers of one answer's `structuredContent`, in order. An answer with
 * no such member, or with any element that is not a pointer, is no pointers.
 */
function readPointers(structured) {
  if (!structured || !Array.isArray(structured.pointers)) return [];
  const pointers = structured.pointers.map(pointerOf);
  return pointers.every((p) => p !== null) ? pointers : [];
}

/**
 * One session: spawn, initialize, one `tools/call`, read the answer, stop.
 * Throws synchronously when `tool` is not in `TOOLS`; resolves to the pointers
 * otherwise, and to `[]` on every failure.
 */
function ask(tool, args, options = {}) {
  if (!TOOLS.includes(tool)) {
    throw new Error(`\`${tool}\` is not a tool this client calls; it calls ${TOOLS.join(', ')}`);
  }
  const root = options.root;
  if (typeof root !== 'string' || !fs.existsSync(path.join(root, '.headwater'))) {
    return Promise.resolve([]);
  }
  const bin = options.bin || 'headwater';
  const argv = [...(options.binArgs || []), 'mcp', '--root', root];
  const timeoutMs = options.timeoutMs ?? DEFAULT_TIMEOUT_MS;

  return new Promise((resolve) => {
    let done = false;
    let child;
    let timer;
    const finish = (pointers) => {
      if (done) return;
      done = true;
      clearTimeout(timer);
      if (child && child.exitCode === null && child.signalCode === null) child.kill('SIGKILL');
      resolve(pointers);
    };

    try {
      child = spawn(bin, argv, {
        cwd: root,
        env: options.env || process.env,
        stdio: ['pipe', 'pipe', 'ignore'],
        windowsHide: true,
      });
    } catch {
      finish([]);
      return;
    }
    timer = setTimeout(() => finish([]), timeoutMs);
    child.on('error', () => finish([]));
    child.stdin.on('error', () => {});

    let stdout = '';
    child.stdout.setEncoding('utf8');
    child.stdout.on('data', (chunk) => {
      stdout += chunk;
      if (stdout.length > MAX_OUTPUT) finish([]);
    });
    child.on('close', (code) => {
      if (code !== 0) {
        finish([]);
        return;
      }
      finish(answer(stdout));
    });

    const messages = [
      {
        jsonrpc: '2.0',
        id: 1,
        method: 'initialize',
        params: {
          protocolVersion: '2024-11-05',
          capabilities: {},
          clientInfo: { name: 'headwater-vscode', version: '0.1.0' },
        },
      },
      { jsonrpc: '2.0', method: 'notifications/initialized' },
      { jsonrpc: '2.0', id: 2, method: 'tools/call', params: { name: tool, arguments: args } },
    ];
    child.stdin.end(messages.map((m) => JSON.stringify(m)).join('\n') + '\n');
  });
}

// The pointers in the response to the `tools/call`, or `[]`.
function answer(stdout) {
  for (const line of stdout.split('\n')) {
    let message;
    try {
      message = JSON.parse(line);
    } catch {
      continue;
    }
    if (!message || message.id !== 2) continue;
    const result = message.result;
    if (!result || result.isError === true) return [];
    return readPointers(result.structuredContent);
  }
  return [];
}

/** The documents that govern the task the user typed, as pointers. */
function route(task, options) {
  return ask('route', { task: String(task) }, options);
}

/** The documents that govern a workspace-relative path, as pointers. */
function governing(relativePath, options) {
  return ask('governing_docs_for_path', { path: String(relativePath) }, options);
}

module.exports = { TOOLS, ask, route, governing, readPointers };
