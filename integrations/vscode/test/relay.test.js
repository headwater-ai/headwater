// SPDX-License-Identifier: Apache-2.0
//
// What holds `integrations/vscode/relay.js`, the MCP server the extension
// registers for Copilot Chat. Each case starts the relay as VS Code would, over
// `test/fake-server.js` in place of `headwater mcp`, and speaks JSON-RPC to it
// on standard input. The fake writes down every session it serves, so a case
// can count the servers a request started. Run the suite with
// `node --test integrations/vscode/test/relay.test.js`.

'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const { spawn } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const readline = require('node:readline');

const relay = require('../relay.js');

const REPO = path.resolve(__dirname, '..', '..', '..');
const RELAY = path.join(__dirname, '..', 'relay.js');
const FAKE = path.join(__dirname, 'fake-server.js');
const FIXTURES = path.join(__dirname, 'fixtures');

// A relay over the fake server replaying `fixture`. `send` writes one message
// and resolves to the next response line; `sessions` reads what the fake
// recorded, one entry for each server the relay started.
function start(fixture, extra = []) {
  const record = path.join(fs.mkdtempSync(path.join(os.tmpdir(), 'hw-relay-')), 'record.jsonl');
  const child = spawn(
    process.execPath,
    [RELAY, '--bin', process.execPath, '--bin-arg', FAKE, '--root', REPO, ...extra],
    {
      env: { ...process.env, FAKE_FIXTURE: path.join(FIXTURES, fixture), FAKE_RECORD: record },
      stdio: ['pipe', 'pipe', 'inherit'],
    },
  );
  const waiting = [];
  readline.createInterface({ input: child.stdout }).on('line', (line) => waiting.shift()(JSON.parse(line)));
  const next = () => new Promise((resolve) => waiting.push(resolve));
  const send = (message) => {
    child.stdin.write(JSON.stringify(message) + '\n');
    return next();
  };
  const sessions = () => {
    if (!fs.existsSync(record)) return [];
    const entries = fs.readFileSync(record, 'utf8').trim().split('\n').map((line) => JSON.parse(line));
    const started = [];
    for (const entry of entries) {
      if (entry.argv) started.push({ argv: entry.argv, messages: [] });
      else if (entry.message) started[started.length - 1].messages.push(entry.message);
    }
    return started;
  };
  const stop = () => child.stdin.end();
  return { send, next, sessions, stop, child };
}

const CALL = {
  jsonrpc: '2.0',
  method: 'tools/call',
  params: { name: 'route', arguments: { task: 'add a VS Code extension that calls the MCP server for routing' } },
};

test('each call starts a fresh server and carries its result back under the caller\'s id', async (t) => {
  const r = start('route-pointers.jsonl');
  t.after(r.stop);
  const first = await r.send({ ...CALL, id: 'a' });
  const second = await r.send({ ...CALL, id: 7 });

  assert.equal(first.id, 'a');
  assert.equal(second.id, 7);
  // The result is the server's own, untouched: the text and structuredContent.
  const recorded = JSON.parse(fs.readFileSync(path.join(FIXTURES, 'route-pointers.jsonl'), 'utf8').split('\n')[1]);
  assert.deepEqual(first.result, recorded.result);

  const started = r.sessions();
  assert.equal(started.length, 2, 'two calls must start two servers, so neither answers from an old walk');
  for (const session of started) {
    assert.deepEqual(session.argv, ['mcp', '--root', REPO]);
    assert.deepEqual(
      session.messages.map((m) => m.method),
      ['initialize', 'notifications/initialized', 'tools/call'],
    );
  }
});

test('the relay never passes --write', async (t) => {
  const r = start('route-pointers.jsonl');
  t.after(r.stop);
  await r.send({ ...CALL, id: 1 });
  assert.ok(!r.sessions()[0].argv.includes('--write'));
});

test('initialize is forwarded, and its parameters open every later session', async (t) => {
  const r = start('route-pointers.jsonl');
  t.after(r.stop);
  const params = { protocolVersion: '2025-06-18', capabilities: {}, clientInfo: { name: 'vscode', version: '1' } };
  const opened = await r.send({ jsonrpc: '2.0', id: 0, method: 'initialize', params });
  assert.equal(opened.id, 0);
  assert.equal(opened.result.serverInfo.name, 'headwater');

  r.child.stdin.write(JSON.stringify({ jsonrpc: '2.0', method: 'notifications/initialized' }) + '\n');
  await r.send({ ...CALL, id: 1 });
  const started = r.sessions();
  // The notification started no server: only the two requests did.
  assert.equal(started.length, 2);
  assert.deepEqual(started[0].messages.map((m) => m.method), ['initialize']);
  assert.deepEqual(started[1].messages[0].params, params);
});

test('ping is answered without a server', async (t) => {
  const r = start('route-pointers.jsonl');
  t.after(r.stop);
  assert.deepEqual(await r.send({ jsonrpc: '2.0', id: 3, method: 'ping' }), { jsonrpc: '2.0', id: 3, result: {} });
  assert.equal(r.sessions().length, 0);
});

test('a server error answer is carried back as the server wrote it', async (t) => {
  const r = start('rpc-error.jsonl');
  t.after(r.stop);
  const answered = await r.send({ ...CALL, id: 5 });
  const recorded = JSON.parse(fs.readFileSync(path.join(FIXTURES, 'rpc-error.jsonl'), 'utf8').trim().split('\n').pop());
  assert.equal(answered.id, 5);
  assert.deepEqual(answered.error, recorded.error);
});

test('a session that ends without an answer is a JSON-RPC error, not a hang', async (t) => {
  // The server answers `initialize`, never answers the call, and exits 0.
  const r = start('initialize-only.jsonl');
  t.after(r.stop);
  const answered = await r.send({ ...CALL, id: 9 });
  assert.equal(answered.id, 9);
  assert.equal(answered.error.code, -32603);
  assert.match(answered.error.message, /ended without an answer/);
});

test('a server that never answers is a JSON-RPC error after the timeout', async (t) => {
  // A fixture path that names no file is a fake that never answers.
  const r = start('no-such-fixture.jsonl', ['--timeout-ms', '300']);
  t.after(r.stop);
  const answered = await r.send({ ...CALL, id: 'slow' });
  assert.equal(answered.id, 'slow');
  assert.match(answered.error.message, /no answer within 300 ms/);
});

test('an engine that cannot start is a JSON-RPC error', async (t) => {
  const child = spawn(process.execPath, [RELAY, '--bin', '/nonexistent/headwater', '--root', REPO], {
    stdio: ['pipe', 'pipe', 'inherit'],
  });
  t.after(() => child.stdin.end());
  const line = new Promise((resolve) => readline.createInterface({ input: child.stdout }).once('line', resolve));
  child.stdin.write(JSON.stringify({ ...CALL, id: 2 }) + '\n');
  const answered = JSON.parse(await line);
  assert.equal(answered.id, 2);
  assert.match(answered.error.message, /could not start/);
});

test('parseArgs needs --bin and --root, and refuses an argument it does not know', () => {
  assert.deepEqual(relay.parseArgs(['--bin', 'b', '--root', 'r', '--bin-arg', 'x', '--timeout-ms', '5']), {
    bin: 'b',
    root: 'r',
    binArgs: ['x'],
    timeoutMs: 5,
  });
  assert.throws(() => relay.parseArgs(['--bin', 'b']), /both required/);
  assert.throws(() => relay.parseArgs(['--bin', 'b', '--root', 'r', '--write']), /unknown argument `--write`/);
});

test('responseOf finds the answer by id and skips every other line', () => {
  const stdout = ['not json', '{"jsonrpc":"2.0","id":1,"result":{}}', '{"jsonrpc":"2.0","id":2,"result":{"x":1}}'].join('\n');
  assert.deepEqual(relay.responseOf(stdout, 2), { jsonrpc: '2.0', id: 2, result: { x: 1 } });
  assert.equal(relay.responseOf(stdout, 3), null);
});
