// SPDX-License-Identifier: Apache-2.0
// Item 3, first half: proof that Node loads the core crate in-process.
//
// `require` of a .node file is a dlopen into this process. If this script
// prints findings, there is no subprocess anywhere in the path.

const path = require('node:path');
const assert = require('node:assert');

// Node only treats `.node` as a native addon, so the cdylib is copied to that
// extension by `build.sh`. The file itself is the unmodified shared library.
const addonPath = path.join(__dirname, '../../target/release/headwater.node');
const hw = require(addonPath);

console.log('loaded in-process, pid', process.pid);
console.log('exports:', Object.keys(hw).sort().join(', '));

const DOC = `---
id: DR-0001
kind: decision
status: currrent
owner: platform
---

# Decision DR-0001

## Context

Some prose.
`;

const findings = hw.checkDocument('docs/decisions/dr-0001.md', DOC);
console.log('\nfindings from checkDocument:');
for (const f of findings) {
  console.log(`  ${f.path}:${f.line}:${f.column} [${f.severity}] ${f.check}: ${f.message}`
    + (f.fix ? `  (fix: ${f.fix})` : ''));
}

// The span assertions are the point: line numbers survive the FFI boundary.
const status = findings.find((f) => f.check === 'status_enum');
assert.ok(status, 'expected a status_enum finding');
assert.strictEqual(status.line, 4, 'status_enum must report file line 4');
assert.strictEqual(status.fix, 'current', 'near-miss fix must cross the boundary');

const missing = findings.find((f) => f.message.includes('last_verified'));
assert.ok(missing, 'expected a missing-facet finding');
assert.strictEqual(missing.line, 1, 'absent facet anchors to the block');

// Corpus entry point, same library, same process.
const corpus = hw.checkCorpus([
  ['a.md', '---\nid: A\nkind: decision\nstatus: current\nowner: t\nlast_verified: 2026-01-01\nsupersedes:\n  - B\n---\n\n## Consequences\n\nx\n'],
  ['b.md', '---\nid: B\nkind: decision\nstatus: current\nowner: t\nlast_verified: 2026-01-01\n---\n\n## Consequences\n\nx\n'],
]);
console.log('\nfindings from checkCorpus:');
for (const f of corpus) {
  console.log(`  ${f.path}:${f.line}:${f.column} [${f.severity}] ${f.check}: ${f.message}`);
}
const recip = corpus.find((f) => f.check === 'reciprocity');
assert.ok(recip, 'expected a reciprocity finding across two documents');
assert.strictEqual(recip.line, 8, 'reciprocity anchors at the declaring line');

console.log('\nOK — all assertions passed, no subprocess used');
