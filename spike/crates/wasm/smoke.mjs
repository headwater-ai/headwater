// Item 3, second half: instantiate the wasm module and check that findings —
// including line numbers — come back correctly through linear memory.
//
// No wasm-bindgen, no JS glue beyond the string marshalling below. A browser
// or an editor host would do exactly this.

import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import assert from 'node:assert';

const here = dirname(fileURLToPath(import.meta.url));
const profile = process.argv[2] ?? 'release-small';
const wasmPath = join(here, `../../target/wasm32-unknown-unknown/${profile}/headwater_wasm.wasm`);

const bytes = await readFile(wasmPath);
const { instance } = await WebAssembly.instantiate(bytes, {});
const ex = instance.exports;
const mem = () => new Uint8Array(ex.memory.buffer);

console.log(`module: ${wasmPath.split('/').slice(-3).join('/')}  (${bytes.length} bytes)`);
console.log('exports:', Object.keys(ex).sort().join(', '));

function writeString(s) {
  const enc = new TextEncoder().encode(s);
  const ptr = ex.hw_alloc(enc.length);
  mem().set(enc, ptr);
  return [ptr, enc.length];
}

function checkDocument(path, source) {
  const [pPtr, pLen] = writeString(path);
  const [sPtr, sLen] = writeString(source);
  const len = ex.hw_check_document(pPtr, pLen, sPtr, sLen);
  const out = new TextDecoder().decode(mem().subarray(ex.hw_result_ptr(), ex.hw_result_ptr() + len));
  ex.hw_dealloc(pPtr, pLen);
  ex.hw_dealloc(sPtr, sLen);
  return out ? out.split('\n') : [];
}

// Self-test needs no marshalling at all. The fixture is missing `owner` and
// `last_verified`, has an undeclared status, and has no Consequences section:
// one finding from each of the four document checks.
assert.strictEqual(ex.hw_selftest(), 4, 'selftest should report four findings');
console.log('\nhw_selftest():', ex.hw_selftest());

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

const lines = checkDocument('docs/decisions/dr-0001.md', DOC);
console.log('\nfindings from wasm:');
for (const l of lines) console.log('  ' + l);

// The same assertions the Node addon makes, against the same core crate.
assert.ok(
  lines.some((l) => l.startsWith('docs/decisions/dr-0001.md:4:1:') && l.includes('status_enum')),
  'status_enum must report file line 4 through wasm too',
);
assert.ok(
  lines.some((l) => l.startsWith('docs/decisions/dr-0001.md:1:1:') && l.includes('last_verified')),
  'absent facet must anchor to the block',
);

console.log('\nOK — wasm32 runs the same core and reports the same positions');
