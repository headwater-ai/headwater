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
// It also reads `withheld` from a `route` answer: how many ranked pointers the
// budget held back. `route` resolves to `{ pointers, withheld }`, and
// `withheldNote` is the line the extension shows beside the list, so a reader
// can tell three answers from three of fifteen. It reads `silence.says` too:
// the engine's own sentence for a route it answered with no pointer, so a
// person who asked can tell a heard answer of nothing from no answer. `route`
// resolves to `{ pointers, withheld, silence }`. `governing_docs_for_path` has
// no budget and no silence, and `governing` resolves to the pointers alone.
//
// It fails open. A spawn error, a non-zero exit, a timeout, `isError: true`, a
// JSON-RPC error, an answer without `structuredContent` (an engine older than
// #1248), output past one megabyte or output that does not parse all resolve
// to no pointers, a withheld count of 0 and no silence. No call throws or
// rejects, except `ask` with a tool outside the allowlist, which is a defect in
// the caller and not a state of the workspace.
//
// It also decides what the Copilot surfaces of `extension.js` say (#1583).
// `present` and `toolText` turn one answer into the lines of the `@headwater`
// participant and the text of a language model tool. `resolveBin` decides
// whether there is an engine to register as an MCP server. `exchange` is the
// one spawn of a server session, which `ask` and `relay.js` both run.

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

/** The answer that carries nothing: no pointers, nothing withheld, no silence. */
function none() {
  return { pointers: [], withheld: 0, silence: null };
}

/**
 * The pointers, the withheld count and the silence of one answer's
 * `structuredContent`. An answer that `readPointers` refuses is no pointers,
 * nothing withheld and no silence, so nothing is shown for an answer nobody
 * could read. A `withheld` that is not a non-negative safe integer is 0. The
 * silence is `silence.says` when the answer has no pointer and that member is a
 * non-empty string, and null otherwise.
 */
function readAnswer(structured) {
  if (!structured || !Array.isArray(structured.pointers)) return none();
  const pointers = readPointers(structured);
  if (pointers.length !== structured.pointers.length) return none();
  const n = structured.withheld;
  const says = structured.silence && structured.silence.says;
  return {
    pointers,
    withheld: Number.isSafeInteger(n) && n >= 0 ? n : 0,
    silence: pointers.length === 0 && typeof says === 'string' && says !== '' ? says : null,
  };
}

/** The line shown beside a route's pointers, or null when nothing was withheld. */
function withheldNote(answered) {
  const n = answered ? answered.withheld : 0;
  return n > 0 ? `${n} more withheld by the budget` : null;
}

/**
 * One session: spawn, initialize, one `tools/call`, read the answer, stop.
 * Throws synchronously when `tool` is not in `TOOLS`; resolves to
 * `{ pointers, withheld }` otherwise, and to no pointers and 0 on every failure.
 */
function ask(tool, args, options = {}) {
  if (!TOOLS.includes(tool)) {
    throw new Error(`\`${tool}\` is not a tool this client calls; it calls ${TOOLS.join(', ')}`);
  }
  const root = options.root;
  if (typeof root !== 'string' || !fs.existsSync(path.join(root, '.headwater'))) {
    return Promise.resolve(none());
  }
  const messages = [
    {
      jsonrpc: '2.0',
      id: 1,
      method: 'initialize',
      params: {
        protocolVersion: '2024-11-05',
        capabilities: {},
        clientInfo: { name: 'headwater-vscode', version: '0.2.0' },
      },
    },
    { jsonrpc: '2.0', method: 'notifications/initialized' },
    { jsonrpc: '2.0', id: 2, method: 'tools/call', params: { name: tool, arguments: args } },
  ];
  return exchange(messages, options).then((ended) => (ended.ok ? answer(ended.stdout) : none()));
}

/**
 * One server process: spawn `<bin> [binArgs...] mcp --root <root>`, write
 * `messages` and close standard input, then collect standard output until the
 * process exits. Resolves to `{ ok: true, stdout }` when it exits 0, and to
 * `{ ok: false, reason }` on a spawn error, a non-zero exit, a timeout or
 * output past `maxOutput`. It never rejects. `ask` and `relay.js` both run
 * every session through it, so a server never outlives the request it served.
 */
function exchange(messages, options = {}) {
  const root = options.root;
  const bin = options.bin || 'headwater';
  const argv = [...(options.binArgs || []), 'mcp', '--root', root];
  const timeoutMs = options.timeoutMs ?? DEFAULT_TIMEOUT_MS;
  const maxOutput = options.maxOutput ?? MAX_OUTPUT;

  return new Promise((resolve) => {
    let done = false;
    let child;
    let timer;
    const finish = (ended) => {
      if (done) return;
      done = true;
      clearTimeout(timer);
      if (child && child.exitCode === null && child.signalCode === null) child.kill('SIGKILL');
      resolve(ended);
    };

    try {
      child = spawn(bin, argv, {
        cwd: root,
        env: options.env || process.env,
        stdio: ['pipe', 'pipe', 'ignore'],
        windowsHide: true,
      });
    } catch (error) {
      finish({ ok: false, reason: `could not start ${bin}: ${error.message}` });
      return;
    }
    timer = setTimeout(() => finish({ ok: false, reason: `no answer within ${timeoutMs} ms` }), timeoutMs);
    child.on('error', (error) => finish({ ok: false, reason: `could not start ${bin}: ${error.message}` }));
    child.stdin.on('error', () => {});

    let stdout = '';
    child.stdout.setEncoding('utf8');
    child.stdout.on('data', (chunk) => {
      stdout += chunk;
      if (stdout.length > maxOutput) finish({ ok: false, reason: `output past ${maxOutput} bytes` });
    });
    child.on('close', (code) => {
      if (code !== 0) {
        finish({ ok: false, reason: `the server exited with status ${code}` });
        return;
      }
      finish({ ok: true, stdout });
    });

    child.stdin.end(messages.map((m) => JSON.stringify(m)).join('\n') + '\n');
  });
}

// The answer in the response to the `tools/call`, or the empty answer.
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
    if (!result || result.isError === true) return none();
    return readAnswer(result.structuredContent);
  }
  return none();
}

/**
 * The documents that govern the task the user typed, as
 * `{ pointers, withheld, silence }`: the pointers the budget let through, how
 * many more it held back, and the engine's sentence when it found none.
 */
function route(task, options) {
  return ask('route', { task: String(task) }, options);
}

/** The documents that govern a workspace-relative path, as pointers. */
async function governing(relativePath, options) {
  return (await ask('governing_docs_for_path', { path: String(relativePath) }, options)).pointers;
}

/**
 * The absolute path of the binary `bin` names, or null when there is none. A
 * `bin` with a directory part names a file; a bare name is looked up on
 * `env.PATH`, with `env.PATHEXT` on Windows. The MCP registration runs this
 * first, so a workspace with no engine registers no server and shows nothing.
 */
function resolveBin(bin, env = process.env, platform = process.platform) {
  if (typeof bin !== 'string' || bin === '') return null;
  const isFile = (candidate) => {
    try {
      return fs.statSync(candidate).isFile();
    } catch {
      return false;
    }
  };
  if (bin.includes('/') || (platform === 'win32' && bin.includes('\\'))) {
    return isFile(bin) ? path.resolve(bin) : null;
  }
  const extensions = platform === 'win32' ? ['', ...(env.PATHEXT || '.EXE;.CMD;.BAT').split(';')] : [''];
  const delimiter = platform === 'win32' ? ';' : ':';
  for (const directory of (env.PATH || '').split(delimiter)) {
    if (directory === '') continue;
    for (const extension of extensions) {
      const candidate = path.join(directory, bin + extension);
      if (isFile(candidate)) return candidate;
    }
  }
  return null;
}

// A `route` answer as it is, or the pointers of `governing` as an answer with
// nothing withheld, so one presentation serves both.
function asAnswer(answered) {
  if (Array.isArray(answered)) return { pointers: answered, withheld: 0 };
  return answered && Array.isArray(answered.pointers) ? answered : none();
}

/**
 * One answer, ready for a chat: a lead sentence, one entry for each pointer,
 * and the withheld note as a tail, or null. An empty answer that carries the
 * engine's silence quotes it, because the engine answered. Any other empty
 * answer says that the engine named nothing, and not that nothing governs,
 * because `ask` reads a failed session as an empty one.
 */
function present(answered, subject) {
  const { pointers, withheld, silence } = asAnswer(answered);
  const note = withheldNote({ withheld });
  let lead = `Documents that govern ${subject}:`;
  if (pointers.length === 0) {
    lead = silence
      ? `Headwater named no document for ${subject}. The engine says: ${silence}.`
      : `Headwater named no document for ${subject}. Either nothing governs it, or the engine did not answer.`;
  }
  const entries = pointers.map((p) => ({
    path: p.path,
    label: p.name || p.path,
    detail: [p.summary, p.asserted ? `(${p.asserted})` : null].filter(Boolean).join(' '),
  }));
  return { lead, entries, tail: note ? `${note}.` : null };
}

/**
 * The text a language model tool returns to the model: `present` as plain
 * lines, each entry with its path, so the model can open the document.
 */
function toolText(answered, subject) {
  const { lead, entries, tail } = present(answered, subject);
  const lines = [lead];
  for (const entry of entries) {
    const named = entry.label === entry.path ? entry.path : `${entry.path} (${entry.label})`;
    lines.push(`- ${named}${entry.detail ? `: ${entry.detail}` : ''}`);
  }
  if (tail) lines.push(tail);
  return lines.join('\n');
}

module.exports = {
  TOOLS,
  ask,
  exchange,
  route,
  governing,
  readPointers,
  readAnswer,
  withheldNote,
  resolveBin,
  present,
  toolText,
};
