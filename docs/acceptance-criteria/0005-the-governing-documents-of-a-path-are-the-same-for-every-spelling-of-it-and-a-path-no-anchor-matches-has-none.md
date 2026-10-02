---
id: HW-AC-0005
status: current
status_since: 2026-10-02
summary: "Over the query fixture tree, each path names the same governors in every spelling, and an ungoverned path names none."
last_verified: 2026-10-02
title: "The governing documents of a path are the same for every spelling of it, and a path no anchor matches has none"
verification_method: test
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  verifies:
    - HW-REQ-0005
  proven_by:
    - HW-VER-0004
---

# The governing documents of a path are the same for every spelling of it, and a path no anchor matches has none

## Fit criterion

In the query fixture tree, `SPEC-FIX-ingest` governs `src/ingest/**`. `DR-FIX-0031` governs the list `[src/ingest/rate_limit.rs, src/ingest/mod.rs]`. Over that tree, `governing_docs_for_path` gives these results:

- `src/ingest/rate_limit.rs` names both documents.
- `src/ingest/mod.rs` names both documents. It matches the pattern and a member of the list.
- `src/ingest` names `SPEC-FIX-ingest` only.
- `src/nothing.rs` names no document.
- Through the MCP tool, `./src/ingest/mod.rs`, `src/../src/ingest/mod.rs` and the absolute path under the root each give the same text and the same structured answer as `src/ingest/mod.rs`.
- Through the MCP tool, `../outside.md` and `/etc/passwd` each get the one outside sentence. The structured answer holds an empty `pointers` list.

The criterion is held on every change, because CI runs the workspace suite on every pull request.

## Method

Test. Two tests prove it, and [HW-VER-0004](../verifications/0004-the-ingest-stubs-of-the-query-fixture-tree-prove-what-governs-a-path.md) states the design of both.

`the_fixture_tree_answers_the_recorded_reads` in `engine/crates/query/tests/reads.rs` writes the answer for each of the four paths into a report. It compares that report byte for byte with `engine/crates/query/fixtures/query.reads`.

`the_related_and_governing_tools_read_every_spelling_of_a_path_as_explain_does` in `engine/crates/query/tests/mcp.rs` asks the MCP server about each spelling and each outside path.

    cargo test -p headwater-query --test reads --manifest-path engine/Cargo.toml --locked
    cargo test -p headwater-query --test mcp the_related_and_governing_tools_read_every_spelling_of_a_path_as_explain_does --manifest-path engine/Cargo.toml --locked
