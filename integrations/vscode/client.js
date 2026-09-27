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
// other line is not a pointer. So the sentence `no document governs <path>` is
// no pointers, and this file does not copy that sentence.
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
// the warrant are each optional and a pointer carries a name or a summary.
const POINTER = /^(\S+)(?: \((.+?)\))?(?: — (.+?))?(?: \[asserted: ([^\]]*)\])?$/;

// Where a pointer's head line may start: a path, then the name, the summary or
// nothing. The server folds a long route line at 80 columns and hangs the rest
// two columns deeper, and this is what tells a folded pointer from a header.
const HEAD = /^\S+(?: \(| — |$)/;

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
  const pointers = [];
  for (const head of heads) {
    const match = POINTER.exec(head.text);
    if (!match) continue;
    const [, p, name, summary, asserted] = match;
    if (name === undefined && summary === undefined) continue;
    pointers.push({
      path: p,
      name: name ?? null,
      summary: summary ?? null,
      asserted: asserted ?? null,
    });
  }
  return pointers;
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
    const block = Array.isArray(result.content) ? result.content[0] : null;
    if (!block || block.type !== 'text') return [];
    return parsePointers(block.text);
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
