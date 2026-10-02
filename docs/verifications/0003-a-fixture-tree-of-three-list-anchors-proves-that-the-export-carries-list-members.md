---
id: HW-VER-0003
status: current
status_since: 2026-10-02
summary: "One fixture decision governs a single path and three lists, one of them holding a member named with the separator, and one test reads the export back against HW-AC-0004."
last_verified: 2026-10-02
title: "A fixture tree of three list anchors proves that the export carries list members"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  proves:
    - HW-AC-0004
  governs:
    - engine/crates/generate/fixtures/listanchor/**
    - engine/crates/generate/fixtures/listanchor.taxonomy.yml
  cited_in:
    - engine/crates/generate/tests/fixtures.rs
---

# A fixture tree of three list anchors proves that the export carries list members

## Approach

The fixture tree at `engine/crates/generate/fixtures/listanchor/` holds one decision, `docs/decisions/0001-govern-two-lists.md`, with the identifier `DR-FIX-0001`. `engine/crates/generate/fixtures/listanchor.taxonomy.yml` declares its kind and its relations.

The decision governs four entries, and each one is a case of [HW-AC-0004](../acceptance-criteria/0004-each-list-anchor-in-the-native-export-carries-its-sorted-members-and-a-single-pattern-carries-none.md):

- `src/c.rs` is a single pattern, which carries no `patterns` field.
- `[src/a.rs, src/b.rs]` is a list of two members.
- `[src/c.rs, "src/a, b.txt"]` holds a member whose name holds `, `. An emitter that split a joined display string on `, ` reads three members there and fails.
- `[src/c.rs, src/b.rs, src/a.rs]` holds three members out of order. An emitter that wrote members only for a list of exactly two drops it, and one that did not sort them writes the wrong order.

The four files under `src/` hold no code. They exist because a `code_path` anchor binds only when its pattern matches at least one entry on the tree.

The test `the_native_export_carries_the_members_of_a_list_anchor` in `engine/crates/generate/tests/fixtures.rs` builds the corpus and emits the native JSON export with the default profile. It reads the export back, and it first checks that there are four anchor nodes and four anchor edges. It then compares the `patterns` field of each node and edge target with the expected members.

    cargo test -p headwater-generate --test fixtures the_native_export_carries_the_members_of_a_list_anchor --manifest-path engine/Cargo.toml --locked

## What this does not cover

The test reads the native JSON export only. It does not read any other emitter, and it does not read the text form of `headwater explain`.

This document governs the fixture tree and its taxonomy, and not the test file. `engine/crates/generate/tests/fixtures.rs` holds many other tests, and this document states the design of one of them. The doc comment of that test cites `HW-VER-0003`, and that comment binds the `cited_in` edge.
