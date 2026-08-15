---
id: HW-EVAL-language-spike-results
status: current
status_since: 2026-08-10
last_verified: 2026-08-10
summary: The results of the Q1 risk-retirement spike, which are four items, all passing, and three findings that the argument did not predict.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cited_by:
    - HW-REG-decisions
    - HW-SPEC-engine-architecture
    - HW-SPEC-check-layer
  traces_to:
    - spike/
---

# The Q1 spike — results

The risk-retirement spike that [Q1](../spec/09-open-questions.md#q1--implementation-language) requires, run. The argument under test is in the [language evaluation](language-choice.md), and the code is in [`spike/`](../../spike/). Reproduce with `spike/build.sh`.

**All four items pass. Q1 stands: the language is Rust.** Three findings came out of the work that the argument did not predict, and one of them is a correction to the specification rather than to the code.

Measured on Linux 6.8, x86_64, rustc 1.97.1, node 24.18.

| Item | Claim | Verdict | Number |
|---|---|---|---|
| 2 | A scope leak fails to compile | **pass** | 6 leaks rejected, 3 positive controls compile |
| 1 | Spans reach rendered findings | **pass** | 10 assertions, front matter and body |
| 3 | One crate as Node addon and as `wasm32` | **pass** | 600 KiB addon, 113 KiB wasm |
| 4 | Warm change-scoped run under 200 ms | **pass** | 1.4–2.2 ms |

## Item 2 — scope enforcement

This ran first, because it was the only item that could have falsified the decision. It did not.

Six leak attempts, each the thing an author would actually reach for, and each rejected by the compiler:

| Attempt | Rejected by |
|---|---|
| `view.graph()`, `view.siblings()`, `view.corpus()` from a document check | `E0599` no such method |
| Constructing a `CorpusView` from a `&Graph` you hold | `E0624` constructor is private |
| Returning `&'static str` borrowed from the view | lifetime error |
| Downcasting the view through `Any` to recover the graph | lifetime error: `Any` needs `'static`, a view borrows for `'g` |
| `self.seen += 1` inside `evaluate` | `E0594` cannot assign behind `&` |
| A `RefCell` field on a registered check | `E0277` `RefCell` is not `Sync` |

Three positive controls compile: an ordinary document check, an edge check, and a corpus check that reads siblings legitimately. They matter as much as the failures, because without them a crate that simply did not build would score six out of six.

**The design changed under the test, and this is the finding to carry back into [spec 12](../spec/12-check-layer.md).** Spec 12 models scope as a value that a check returns from `scope()`. With one trait and a returned value, a check declares one scope and can still be handed a view that reads more, because nothing connects the declaration to the argument. The spike uses **one trait per scope** — `DocumentCheck`, `EdgeCheck`, `CorpusCheck` — so the declared scope *is* the argument type. A check that wants siblings has to implement `CorpusCheck`, which registers it in the bucket where the runner already keys the cache on the whole corpus. There is one fact instead of two, so the two cannot disagree. `Scope` survives as a reporting and cache-keying value derived from the trait, never supplied by the implementer.

**Two honest limits.** First, `EdgeView` holds a `&Graph` internally to answer `reciprocal_exists()`. Nothing outside the crate can reach it, because the field is private — but that is enforcement by privacy, which Go matches with an unexported field. The genuinely Rust-only guarantees are the lifetime and `Sync` ones, rows three, four and six above. Second, the compile-fail expectations are pinned to rustc's exact diagnostic text, so a toolchain upgrade will require regenerating them. That is a real maintenance cost, small and recurring.

## Item 1 — spans

Ten assertions pass, over both halves of a document. Top-level keys, sequence elements, and nested keys each report their own line. Body headings and inline links report file lines, not body-relative ones. Then the half that matters: a finding from a real check renders as

    docs/decisions/dr-0001.md:4:1: [error] status_enum: `currrent` is not a declared status

with the position surviving to output, and — through item 3 — across both embedding boundaries unchanged.

Three cases in particular:

- **A key that is present but empty** anchors to that key's line.
- **A key that is absent** has no span of its own, and anchors to the front-matter block rather than to line 0. An editor cannot jump to line 0, and this is the defect a naive implementation ships.
- **CRLF input** does not shift line numbers.

The evaluation predicted that the archived-YAML-crate problem would land on a layer that is custom work regardless. That held. `saphyr-parser` supplies an event stream with a span per event and nothing else, and the tree builder that turns it into spanned front matter is about 150 lines. What the evaluation did not predict is that **the parser's own documentation is wrong in a way only an empirical check catches**:

- `Marker::col` is documented as 1-indexed and is **0-indexed**. Trusting the comment puts every column off by one.
- `key:` with nothing after it arrives as a *plain* `~` scalar, while `key: ""` arrives as a double-quoted empty string. Only the scalar style separates "the author left this blank" from "the author wrote an empty string", and a required-facet check has to tell them apart. A deserializer collapses both.

Both are recorded in `spike/crates/core/examples/yaml_events.rs`, which is the probe that found them. This is the concrete form of the cost the evaluation named: the YAML layer is yours to own, including its surprises.

## Item 3 — embedding

The same core crate, in a Node process and on `wasm32`, with no subprocess in either path.

| Artifact | Bytes | Note |
|---|---|---|
| `headwater.node` (native addon) | 613,936 | N-API, loaded by `require` |
| `headwater_wasm.wasm` (`release`) | 181,708 | opt-level 3 |
| `headwater_wasm.wasm` (`release-small`) | 115,524 | opt-level "z" |

The Node addon is `dlopen`ed into the process and returns structured findings, including the mechanical fix, across the boundary. The wasm module is instantiated with the plain `WebAssembly` API and no `wasm-bindgen`, deliberately: glue would inflate the measured artifact and hide whether the core itself is portable. Both report identical positions for the same input.

**The item-2 enforcement caught its own author here, which is the most useful thing that happened in the spike.** The first draft of the Node wrapper tried to construct a `DocumentView` and did not compile, because the wrapper is a *separate crate* and the constructor is `pub(crate)`. The repair is the correct layering: an embedder asks for a task and receives findings, and the view type never crosses the crate boundary. So no embedder can widen a check's scope by building a view for it. The constraint held against the person who wrote it, in the place where foreign code actually lives.

One concession is real: wasm32 gets no threads, so the runner's fan-out compiles out there behind a feature flag. For an editor checking one buffer that costs nothing.

## Item 4 — the budget

A synthetic 1,000-document corpus, 1,017 KiB, six checks (four document, one edge, one corpus), 6,979 check instances, 1,170 findings.

| Phase | Median | Budget |
|---|---|---|
| Cold: parse, classify, build graph | 34.9 ms | |
| Cold: all checks | 6.1 ms | |
| **Cold total** | **41.9 ms** | 5 s |
| Warm: re-parse the one edited file | 0.030 ms | |
| Warm: rebuild graph and index | 0.879 ms | |
| Warm: 11 invalidated instances | 0.494 ms | |
| **Warm change-scoped total** | **1.41 ms** | 200 ms |

The warm total moved between 1.4 ms and 2.2 ms across repeated runs on an otherwise busy machine, so read it as roughly two milliseconds rather than as a precise figure. That is a margin near a hundredfold on the number that matters. The margin is the useful result, not the number, because three things in the spike are cheaper than the real engine will be:

- The Markdown scan handles ATX headings and inline links, not CommonMark. A real parser is the dominant cost in the 34.9 ms cold figure and will multiply it.
- The content hash is FNV, not collision-resistant. A real hash costs more, on a phase that is already sub-millisecond.
- The corpus is generated, so it is more uniform than a real one.

Two costs scale with corpus size rather than change size, and both are visible in the warm breakdown. The graph rebuild is O(corpus) at 0.879 ms per change for 1,000 documents, and corpus-scoped checks re-run on every change because their scope cannot be narrowed. At ten thousand documents both are roughly ten times larger, which is still an order of magnitude inside the budget. Neither is a language question — the same arithmetic applies in Go — but both are worth carrying into the engine design as the things to watch.

## What the spike did not test

Stated plainly, because an unstated gap reads as a covered one:

- No LSP server, no MCP server, no plugin hosting. The WebAssembly component model that [spec 12](../spec/12-check-layer.md) now points at for plugins is untested here.
- Linux and x86_64 only. No macOS, no Windows, no aarch64.
- No Python binding, although the evaluation cites `pyo3` as part of the embedding argument.
- No real corpus. The generator is representative by construction, which is not the same as representative.
- The taxonomy is hard-coded rather than resolved from a schema, so nothing here tests the overlay resolver or the lock.

## Verdict

Every item passes, so Q1 stands and Go is no longer the fallback. Two changes go back into the specification: spec 12's `scope()` becomes one trait per scope rather than a returned value, and its note about plugin hosting keeps the WebAssembly option that item 3 makes plausible but does not test.
