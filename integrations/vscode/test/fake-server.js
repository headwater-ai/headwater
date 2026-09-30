// SPDX-License-Identifier: Apache-2.0
//
// A stand-in for `headwater mcp` that replays a recorded session.
//
//   FAKE_FIXTURE  a file under test/fixtures/ with one line per response, as the
//                 real server wrote them. A request with an `id` is answered with
//                 the recorded line of the same `id`. A line that is not JSON is
//                 written out on the first request, which is how the garbage
//                 case reaches the client. A path that names no file is a server
//                 that never answers and exits on its own after ten seconds.
//   FAKE_RECORD   a file this server appends to, one JSON object per line: first
//                 `{"argv": [...]}`, then `{"message": ...}` for each line read.
//   FAKE_EXIT     the status to exit with once standard input closes (default 0).
//   FAKE_FLOOD    a count of bytes: one line of that many `x` is written before
//                 the first answer, which is how the output cap is reached.

'use strict';

const fs = require('node:fs');
const readline = require('node:readline');

const fixture = process.env.FAKE_FIXTURE;
const record = process.env.FAKE_RECORD;
const status = Number(process.env.FAKE_EXIT || 0);
const flood = Number(process.env.FAKE_FLOOD || 0);

function note(entry) {
  if (record) fs.appendFileSync(record, JSON.stringify(entry) + '\n');
}

note({ argv: process.argv.slice(2) });

const silent = !fixture || !fs.existsSync(fixture);
const lines = silent ? [] : fs.readFileSync(fixture, 'utf8').split('\n').filter((l) => l.trim() !== '');
const byId = new Map();
const garbage = [];
for (const line of lines) {
  try {
    const parsed = JSON.parse(line);
    if (parsed && parsed.id !== undefined) byId.set(parsed.id, line);
  } catch {
    garbage.push(line);
  }
}

let answered = false;
const input = readline.createInterface({ input: process.stdin });
input.on('line', (line) => {
  if (line.trim() === '') return;
  let message;
  try {
    message = JSON.parse(line);
  } catch {
    note({ unparsed: line });
    return;
  }
  note({ message });
  if (silent || message.id === undefined) return;
  if (!answered) {
    if (flood > 0) process.stdout.write('x'.repeat(flood) + '\n');
    for (const g of garbage) process.stdout.write(g + '\n');
    answered = true;
  }
  if (byId.has(message.id)) process.stdout.write(byId.get(message.id) + '\n');
});
input.on('close', () => {
  if (silent) {
    // Outlive any client timeout, and still end on its own, so a client whose
    // timeout is broken fails its test rather than hanging the suite.
    setTimeout(() => process.exit(0), 10000);
    return;
  }
  process.exitCode = status;
});
