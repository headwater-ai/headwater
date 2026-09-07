---
id: HW-OBL-0172
status: current
status_since: 2026-09-07
summary: "Nine constants named ALL type the variant list of an enum by hand, the array length is part of the type, and no assertion compares either against the exhaustive match beside it."
last_verified: 2026-09-07
title: "Nine hand-kept constants enumerate an enum and nothing holds one against the variants"
waiting_on: adopter
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

The module comment of `engine/crates/generate/tests/spec_six_projections.rs` states the gap in terms: `Kind::ALL` is a hand-kept list the compiler does not hold complete. The test below it compares `Kind::ALL` against the projection-kinds block of spec 6, which is a second hand-kept list. Two lists agreeing says nothing about the enum that both describe.

The same grain runs through the suite. `assert_eq!(<expression>.len(), <literal>)` occurs 146 times in 51 files across 17 crates, measured with `assert_eq!\(.*\.len\(\), *[0-9]+` over the same tree. Each literal is a population size copied into a test, and none of them is derived from the declaration that produces the population.

## Obligation

Nothing in this workspace ties a hand-kept list to the variants of the enum it enumerates. Nothing ties a length literal to the declaration that sizes it. A variant added to any of the nine enums compiles, passes the whole suite, and narrows every reader of the array.

The debt has two halves and they are not the same size. The nine constants are a bounded set with one remedy each. The 146 length literals are a grain question rather than a defect list. Many of them assert a fixture the test itself built, and a literal is the honest form there.

## Discharge

A constant is discharged when a value derived from the enum holds it. An exhaustive `match` in a function that returns the array does that at compile time. A test that maps every variant through `name` and compares the result against the array does it at test time. The remedy is per constant and no declaration of this taxonomy reaches it.

A length literal is discharged when the test names the source of the number. Where the population comes from a declaration, the assertion reads that declaration. Where the test built the input, the literal stays and no work is owed. Reading all 146 for that distinction is the measurement this record does not carry.
