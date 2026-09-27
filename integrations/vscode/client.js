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
// It reads a pointer by the shape `docs/interfaces/headwater-mcp.md` calls the
// wire format a caller may rely on: `path (name) — summary [asserted: …]`. Any
// other line is not a pointer. The sentence `no document governs <path>` is
// the one exception, recognized by its prefix before anything is parsed,
// because the path it repeats can hold a pointer's shape.
//
// It fails open. A spawn error, a non-zero exit, a timeout, `isError: true`, a
// JSON-RPC error or output that does not parse all resolve to `[]`. No call
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

// `path (name) — summary [asserted: warrant]`, where the name, the summary and
// the warrant are each optional and a pointer carries a name or a summary. A
// path may hold a space, so the path runs to the first ` (` or ` — `.
const POINTER = /^(.+?)(?: \((.+?)\))?(?: — (.+?))?(?: \[asserted: ([^\]]*)\])?$/;

// Where a pointer's head line may start: a path, then the name, the summary or
// nothing. The server folds a long route line at 80 columns and hangs the rest
// two columns deeper, and this is what tells a folded pointer from a header.
const HEAD = /^(?:\S+$|.+? \(|.+? — )/;

// The lines `route` writes under a pointer, two columns deeper, as it writes
// a fold. They end the pointer rather than continue it.
const EVIDENCE = /^(?:matched |governs |suspect: )/;

function indentOf(line) {
  return line.length - line.trimStart().length;
}

/**
 * The pointers in one answer's text, in order. A fold is rejoined with one
 * space. Every line that does not parse as a pointer is dropped.
 */
function parsePointers(text) {
  if (typeof text !== 'string') return [];
  const heads = [];
  let current = null;
  for (const raw of text.split('\n')) {
    if (raw.trim() === '') {
      current = null;
      continue;
    }
    const indent = indentOf(raw);
    const body = raw.trim();
    // Only a line that starts like a pointer takes a fold or evidence under
    // it. Under any other line, a deeper line is a line of its own.
    if (current && current.pointer && indent === current.indent + 2) {
      if (current.open && EVIDENCE.test(body)) current.open = false;
      if (current.open) current.text += ' ' + body;
      continue;
    }
    if (current && current.pointer && indent > current.indent + 2) continue;
    const pointer = HEAD.test(body);
    current = { indent, text: body, pointer, open: pointer };
    heads.push(current);
  }
  return heads.map((head) => pointerOf(head.text)).filter((p) => p !== null);
}

// One line as a pointer, or null. A line that carries neither a name nor a
// summary is not a pointer. Every sentence the server writes around its
// pointers has that shape, so this guard keeps a sentence from being shown as
// a document.
function pointerOf(line) {
  const match = POINTER.exec(line);
  if (!match) return null;
  const [, p, name, summary, asserted] = match;
  if (name === undefined && summary === undefined) return null;
  return { path: p, name: name ?? null, summary: summary ?? null, asserted: asserted ?? null };
}

// How `governing_docs_for_path` answers a path that no document governs, as
// `engine/crates/query/src/mcp.rs` writes it. This is the one sentence of the
// server that this file copies.
const UNGOVERNED = 'no document governs ';

// How each tool's answer text is read. The server repeats the caller's own
// words in two places, and neither is ever read for a pointer.
const READERS = Object.freeze({
  // The first line is `route "<task>"`, the task the user typed. The report
  // is everything under it.
  route: (text) => parsePointers(text.split('\n').slice(1).join('\n')),
  // Either the sentence `no document governs <path>`, which repeats the path
  // the caller sent, or one pointer per line and nothing else. The sentence
  // is no pointers whatever the path holds, and any other answer is all
  // pointers or none.
  governing_docs_for_path: (text) => {
    // The sentence is recognized before anything is parsed, because the path
    // it repeats can itself hold ` — ` or `(x)` and so parse as a pointer.
    if (text.startsWith(UNGOVERNED)) return [];
    const lines = text.split('\n').filter((line) => line !== '');
    const pointers = lines.map(pointerOf);
    return pointers.length > 0 && pointers.every((p) => p !== null) ? pointers : [];
  },
});

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
      finish(answer(stdout, tool));
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
function answer(stdout, tool) {
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
    const block = Array.isArray(result.content) ? result.content[0] : null;
    if (!block || block.type !== 'text') return [];
    return READERS[tool](block.text);
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

module.exports = { TOOLS, ask, route, governing, parsePointers };
