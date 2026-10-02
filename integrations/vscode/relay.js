// SPDX-License-Identifier: Apache-2.0
//
// The MCP server that `extension.js` registers with VS Code for Copilot Chat
// (#1583). VS Code keeps a registered server alive for the whole chat session.
// `headwater mcp` walks the corpus once when it starts, so a server kept alive
// across the agent's own edits would answer from an old walk. This relay is the
// long-lived process instead, and it starts one `headwater mcp` for each
// request it forwards, as HW-DR-0102 rules for every query of an editor
// integration:
//
//   node relay.js --bin <path> --root <folder> [--bin-arg <arg>]... [--timeout-ms <n>]
//
// It answers `initialize` by forwarding it to a fresh server, and it keeps the
// client's `initialize` parameters for every later session. Each other request
// that carries an `id` runs as one session: `initialize`, `initialized`, the
// request, then the server's answer back under the client's own `id`. It
// answers `ping` itself and drops every notification, because no session
// outlives the request it served. It never passes `--write`, so the server it
// starts registers no tool that writes.
//
// A session that fails, or that ends without an answer, comes back as a
// JSON-RPC error and never as a hang: the model then reads that the engine did
// not answer, and not that nothing governs the file.

'use strict';

const readline = require('node:readline');
const client = require('./client.js');

const DEFAULT_TIMEOUT_MS = 60000;
const MAX_OUTPUT = 16 << 20;
const DEFAULT_INITIALIZE = {
  protocolVersion: '2024-11-05',
  capabilities: {},
  clientInfo: { name: 'headwater-vscode-relay', version: '0.2.0' },
};

function parseArgs(argv) {
  const parsed = { bin: null, root: null, binArgs: [], timeoutMs: DEFAULT_TIMEOUT_MS };
  for (let i = 0; i < argv.length; i += 1) {
    const value = argv[i + 1];
    switch (argv[i]) {
      case '--bin':
        parsed.bin = value;
        i += 1;
        break;
      case '--root':
        parsed.root = value;
        i += 1;
        break;
      case '--bin-arg':
        parsed.binArgs.push(value);
        i += 1;
        break;
      case '--timeout-ms':
        parsed.timeoutMs = Number(value);
        i += 1;
        break;
      default:
        throw new Error(`relay.js: unknown argument \`${argv[i]}\``);
    }
  }
  if (!parsed.bin || !parsed.root) throw new Error('relay.js: --bin and --root are both required');
  return parsed;
}

// The response line with id `want` in a session's output, parsed, or null.
function responseOf(stdout, want) {
  for (const line of stdout.split('\n')) {
    let message;
    try {
      message = JSON.parse(line);
    } catch {
      continue;
    }
    if (message && message.id === want && ('result' in message || 'error' in message)) return message;
  }
  return null;
}

function failure(id, reason) {
  return { jsonrpc: '2.0', id, error: { code: -32603, message: `headwater mcp did not answer: ${reason}` } };
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  const session = { ...options, maxOutput: MAX_OUTPUT };
  let initialize = DEFAULT_INITIALIZE;

  const send = (message) => process.stdout.write(JSON.stringify(message) + '\n');

  async function forward(request) {
    const opening = { jsonrpc: '2.0', id: 1, method: 'initialize', params: initialize };
    const isInitialize = request.method === 'initialize';
    const messages = isInitialize
      ? [opening]
      : [opening, { jsonrpc: '2.0', method: 'notifications/initialized' }, { ...request, id: 2 }];
    const ended = await client.exchange(messages, session);
    if (!ended.ok) return failure(request.id, ended.reason);
    const response = responseOf(ended.stdout, isInitialize ? 1 : 2);
    if (!response) return failure(request.id, 'the session ended without an answer');
    return { ...response, id: request.id };
  }

  const input = readline.createInterface({ input: process.stdin });
  input.on('line', (line) => {
    if (line.trim() === '') return;
    let message;
    try {
      message = JSON.parse(line);
    } catch {
      send({ jsonrpc: '2.0', id: null, error: { code: -32700, message: 'Parse error' } });
      return;
    }
    if (!message || message.id === undefined || message.id === null) return;
    if (message.method === 'ping') {
      send({ jsonrpc: '2.0', id: message.id, result: {} });
      return;
    }
    if (message.method === 'initialize' && message.params) initialize = message.params;
    forward(message).then(send);
  });
}

if (require.main === module) main();

module.exports = { parseArgs, responseOf };
