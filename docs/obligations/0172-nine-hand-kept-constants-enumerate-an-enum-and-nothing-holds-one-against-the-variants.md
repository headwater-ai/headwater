---
id: HW-OBL-0172
status: current
status_since: 2026-09-07
summary: "The constants named ALL are now written from their enums. The length literals of the suite still copy a population size into a test, and nobody has sorted them."
last_verified: 2026-11-20
title: "Nine hand-kept constants enumerate an enum and nothing holds one against the variants"
waiting_on: build
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - engine/crates/adapter/src/lib.rs
    - engine/crates/generate/src/lib.rs
---

# Nine hand-kept constants enumerate an enum and nothing holds one against the variants

## Context

`Format::ALL` in `engine/crates/adapter/src/lib.rs` is declared `[Format; 4]`, and the four names inside it are typed by hand. Two lines below it, `Format::name` matches the same enum exhaustively, so a fifth variant refuses to compile there. The array refuses nothing, because its length is part of its type and a fifth variant leaves it at four. `Format::parse` reads the array, so the variant that the array omits has a name and no parse.

Nine constants of that shape sit in six files across five crates. The population comes from `(const|static) +ALL[A-Z_]*` over `engine/crates/**/*.rs` on `c574ddc`, and it holds `Shell::ALL`, `Emitter::ALL`, `Kind::ALL`, `Category::ALL`, `Expectation::ALL`, `Tier::ALL`, `Arm::ALL`, `Format::ALL` and `Class::ALL`. Four of the nine are in `engine/crates/probe/src/lib.rs`. No crate of this workspace depends on `strum` or on `enum-iterator`, so no derive produces a variant set for any of them.

The module comment of `engine/crates/generate/tests/spec_six_projections.rs` states the gap in terms: `Kind::ALL` is a hand-kept list the compiler does not hold complete. The test below it compares `Kind::ALL` against the projection-kinds block of [the engine architecture](../spec/engine-architecture.md), which is a second hand-kept list. Two lists agreeing says nothing about the enum that both describe.

The same grain runs through the suite. `assert_eq!(<expression>.len(), <literal>)` occurs 146 times in 51 files across 17 crates, measured with `assert_eq!\(.*\.len\(\), *[0-9]+` over the same tree. Each literal is a population size copied into a test, and none of them is derived from the declaration that produces the population.

## Obligation

The debt had two halves, and they are not the same size. The constants were a bounded set with one remedy each, and that half is closed (see Discharge). The length literals are the half that stays. Nothing ties a length literal to the declaration that sizes it.

On 2026-11-20, on `d359c244`, the same pattern found 391 length literals in 102 files. That is a grain question rather than a defect list. Many of them assert a fixture that the test itself built, and a literal is the honest form there. Nobody has measured how many of the 391 copy a population that a declaration produces.

## Discharge

**The constant half is discharged** by [#1961](https://github.com/headwater-ai/headwater/pull/1961), slice C of [#1494](https://github.com/headwater-ai/headwater/issues/1494). On `d359c244` there were ten constants in `src`, not nine, because `Block::ALL` in `engine/crates/probe/src/budget.rs` was new. A test file held an eleventh, `Shape::ALL` in `engine/crates/probe/tests/corpus.rs`. The macro `headwater_check::enumerated!` now declares each of these enums, except `Shell`, and writes its `ALL` from the same list of variants. So a variant that `ALL` omits cannot be written. The one exception is a variant after a `;` in the list, which is outside `ALL` on purpose and where a reader sees it. `Arm::Scenario` is that case. `Shell` derives `clap::ValueEnum`, and `Shell::all()` returns the list that the derive writes.

The text below is the condition as it was recorded. A constant is discharged when a value derived from the enum holds it. An exhaustive `match` in a function that returns the array does that at compile time. A test that maps every variant through `name` and compares the result against the array does it at test time. The remedy is per constant and no declaration of this taxonomy reaches it.

A length literal is discharged when the test names the source of the number. Where the population comes from a declaration, the assertion reads that declaration. Where the test built the input, the literal stays and no work is owed. Reading all 146 for that distinction is the measurement this record does not carry.
