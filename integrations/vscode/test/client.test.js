// SPDX-License-Identifier: Apache-2.0
//
// What holds `integrations/vscode/client.js`, the one file of the extension
// that decides anything. `extension.js` is glue over the `vscode` module, which
// `node --test` cannot load, so it is kept thin enough to carry no decision.
//
// The first test runs against the real engine: `HEADWATER_BIN` names it. With
// the variable unset it skips with a printed reason on a workstation, and it
// fails in CI, where a skip would hide the one case that reads a real corpus.
//
// Every other test drives `test/fake-server.js`, which replays a session that
// the real server recorded under `test/fixtures/` and writes down what it was
// asked. Run the suite with `node --test integrations/vscode/test/client.test.js`.

'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const client = require('../client.js');

const REPO = path.resolve(__dirname, '..', '..', '..');
const FAKE = path.join(__dirname, 'fake-server.js');
const FIXTURES = path.join(__dirname, 'fixtures');

// A client aimed at the fake server: `node fake-server.js mcp --root <root>`.
function fake(fixture, extra = {}) {
  const record = path.join(fs.mkdtempSync(path.join(os.tmpdir(), 'hw-vscode-')), 'record.jsonl');
  const options = {
    root: REPO,
    bin: process.execPath,
    binArgs: [FAKE],
    env: { ...process.env, FAKE_FIXTURE: path.join(FIXTURES, fixture), FAKE_RECORD: record },
    ...extra,
  };
  const asked = () =>
    fs.existsSync(record)
      ? fs.readFileSync(record, 'utf8').trim().split('\n').map((line) => JSON.parse(line))
      : [];
  return { options, asked };
}

test('governing answers the contract\'s pointers, silence for an ungoverned path, and nothing when the server is unreachable', async (t) => {
  const bin = process.env.HEADWATER_BIN;
  if (!bin) {
    if (process.env.CI) {
      assert.fail('HEADWATER_BIN is unset in CI; this case must run against the built engine');
    }
    t.skip('HEADWATER_BIN is unset: build the engine and point HEADWATER_BIN at it to run this case');
    return;
  }
  const options = { root: REPO, bin };

  const governed = await client.governing('engine/crates/query/src/mcp.rs', options);
  assert.equal(governed.length, 1);
  assert.equal(governed[0].path, 'docs/interfaces/headwater-mcp.md');
  assert.equal(governed[0].name, 'headwater mcp');

  // The server answers the sentence `no document governs <path>`, and a
  // sentence is not a pointer: it is never shown as a guess.
  assert.deepEqual(await client.governing('integrations/vscode/extension.js', options), []);

  // No engine at the configured path is silence, not an error.
  const missing = { root: REPO, bin: '/nonexistent/headwater' };
  assert.deepEqual(await client.governing('engine/crates/query/src/mcp.rs', missing), []);
  assert.deepEqual(await client.route('what governs the mcp server', missing), []);
});

test('route parses the pointers out of the report, folded lines rejoined, and nothing else', async () => {
  const { options } = fake('route-pointers.jsonl');
  const pointers = await client.route('add a VS Code extension that calls the MCP server for routing', options);
  assert.deepEqual(
    pointers.map((p) => p.path),
    [
      'docs/probes/the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens.md',
      'docs/how-to/mine-the-shadow-mode-routing-log.md',
      'docs/requirements/0002-every-document-of-a-kind-that-requires-sections-carries-all-of-them.md',
      'docs/interfaces/headwater-mcp.md',
      'docs/obligations/0206-hw-run-policy-names-a-worktree-add-workaround-that-write-edit-refuses-under-this-harness.md',
    ],
  );
  // The server folds a pointer at 80 columns, here inside the name.
  const folded = pointers[1];
  assert.equal(folded.name, 'Mine the shadow-mode routing log');
  assert.equal(
    folded.summary,
    'Join the shadow log to the session transcripts by prompt identifier, and read the deterministic route against the embedding path one prompt at a time.',
  );
  const mcp = pointers[3];
  assert.equal(mcp.name, 'headwater mcp');
  assert.equal(mcp.asserted, null);
  assert.equal(
    mcp.summary,
    'How the stdio MCP server exposes read tools, optional working-tree writes, and a one-write session seal.',
  );
});

test('an [asserted: ...] pointer keeps its warrant', async () => {
  const { options } = fake('route-pointers.jsonl');
  const [first] = await client.route('anything', options);
  assert.equal(first.asserted, 'nobody accepted this document');
  assert.ok(!first.summary.includes('[asserted'));
});

test('the no-purpose-matched answer is no pointers', async () => {
  const { options } = fake('route-no-purpose.jsonl');
  assert.deepEqual(await client.route('zzqx', options), []);
});

test('isError: true is no pointers', async () => {
  const { options } = fake('is-error.jsonl');
  assert.deepEqual(await client.governing('a/b.rs', options), []);
});

test('garbage on stdout is no pointers', async () => {
  const { options } = fake('garbage.jsonl');
  assert.deepEqual(await client.governing('a/b.rs', options), []);
});

test('a JSON-RPC error answer is no pointers', async () => {
  const { options } = fake('rpc-error.jsonl');
  assert.deepEqual(await client.route('x', options), []);
});

test('a server that exits non-zero is no pointers, even after an answer', async () => {
  const { options } = fake('route-pointers.jsonl', {
    env: { ...fake('route-pointers.jsonl').options.env, FAKE_EXIT: '3' },
  });
  assert.deepEqual(await client.route('x', options), []);
});

test('a server that never answers is no pointers within the timeout', async () => {
  const { options } = fake('silent', { timeoutMs: 300 });
  const started = Date.now();
  assert.deepEqual(await client.governing('a/b.rs', options), []);
  assert.ok(Date.now() - started < 3000, 'the client waited past its timeout');
});

test('a workspace with no .headwater/ spawns nothing and answers nothing', async () => {
  const empty = fs.mkdtempSync(path.join(os.tmpdir(), 'hw-vscode-empty-'));
  const { options, asked } = fake('route-pointers.jsonl', { root: empty });
  assert.deepEqual(await client.route('x', options), []);
  assert.deepEqual(asked(), []);
});

test('the client speaks only the query class: initialize, initialized, and an allowlisted tool, never --write', async () => {
  const { options, asked } = fake('route-pointers.jsonl');
  await client.route('what governs this', options);
  await client.governing('engine/crates/query/src/mcp.rs', options);
  const records = asked();
  const argvs = records.filter((r) => r.argv).map((r) => r.argv);
  assert.equal(argvs.length, 2, 'one session per query');
  for (const argv of argvs) {
    assert.deepEqual(argv, ['mcp', '--root', REPO]);
    assert.ok(!argv.includes('--write'));
  }
  const messages = records.filter((r) => r.message).map((r) => r.message);
  assert.deepEqual(
    messages.map((m) => m.method),
    ['initialize', 'notifications/initialized', 'tools/call', 'initialize', 'notifications/initialized', 'tools/call'],
  );
  const calls = messages.filter((m) => m.method === 'tools/call').map((m) => m.params);
  assert.deepEqual(calls, [
    { name: 'route', arguments: { task: 'what governs this' } },
    { name: 'governing_docs_for_path', arguments: { path: 'engine/crates/query/src/mcp.rs' } },
  ]);
  assert.deepEqual([...client.TOOLS], ['route', 'governing_docs_for_path']);
  assert.ok(Object.isFrozen(client.TOOLS));
});

test('asking for any tool outside the allowlist throws before anything spawns', () => {
  const { options, asked } = fake('route-pointers.jsonl');
  for (const tool of ['new', 'fix', 'check', 'explain', 'related', 'resolve_identifier', '']) {
    assert.throws(() => client.ask(tool, { path: 'x' }, options), /not a tool this client calls/);
  }
  assert.deepEqual(asked(), []);
});

test('parsePointers takes a pointer line by the contract shape alone', () => {
  assert.deepEqual(client.parsePointers('no document governs a/b.rs\n'), []);
  assert.deepEqual(client.parsePointers('docs/a.md (A (b) c) — sum — more\n'), [
    { path: 'docs/a.md', name: 'A (b) c', summary: 'sum — more', asserted: null },
  ]);
  assert.deepEqual(client.parsePointers('docs/a.md — only a summary\n'), [
    { path: 'docs/a.md', name: null, summary: 'only a summary', asserted: null },
  ]);
  assert.deepEqual(client.parsePointers(''), []);
});
