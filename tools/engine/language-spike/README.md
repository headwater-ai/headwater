# Q1 risk-retirement spike

Not an engine, and not the start of one. This is the four-item spike that
[Q1](../../../docs/spec/09-decisions.md#q1--implementation-language) requires before the Rust decision stands, built far enough that each item passes or fails. The results are in [the results document](../../../docs/evaluations/language-spike-results.md), and the argument they test is in [the evaluation](../../../docs/evaluations/language-choice.md).

Run everything:

    ./build.sh

Needs a Rust toolchain with the `wasm32-unknown-unknown` target, and node. The script runs item 2 first, because it is the only one that could have falsified the decision.

| Item | What it retires | Where |
|---|---|---|
| 1 | Spanned front-matter parse, positions reaching rendered findings | `crates/core/src/frontmatter.rs`, `src/spans_test.rs` |
| 2 | A scope leak fails to compile | `crates/core/src/view.rs`, `src/check.rs`, `tests/ui/` |
| 3 | One crate as a native Node addon and as `wasm32` | `crates/node/`, `crates/wasm/` |
| 4 | Warm change-scoped run inside the 200 ms budget | `crates/core/src/runner.rs`, `examples/bench.rs` |

Two things here are deliberately not production shape, and the results document
says so again: the content hash is FNV rather than a collision-resistant hash,
and the Markdown scan handles ATX headings and inline links rather than
CommonMark. Both would change the cold-parse number and neither changes any
verdict.
