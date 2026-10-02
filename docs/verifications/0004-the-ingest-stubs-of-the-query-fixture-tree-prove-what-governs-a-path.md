---
id: HW-VER-0004
status: current
status_since: 2026-10-02
summary: "Two stub files that a pattern and a list both reach, read by a recorded report and an MCP spelling test, prove HW-AC-0005."
last_verified: 2026-10-02
title: "The ingest stubs of the query fixture tree prove what governs a path"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  proves:
    - HW-AC-0005
  governs:
    - engine/crates/query/fixtures/src/ingest/**
  cited_in:
    - engine/crates/query/tests/reads.rs
    - engine/crates/query/tests/mcp.rs
---

# The ingest stubs of the query fixture tree prove what governs a path

## Approach

The query fixture tree at `engine/crates/query/fixtures/` is one small corpus. Its documents are under `query/`, and its code is the two stub files `src/ingest/mod.rs` and `src/ingest/rate_limit.rs`. The stubs hold no code. They exist because a `code_path` anchor binds only when its pattern matches at least one entry on the tree.

Two documents govern the stubs, and the overlap is the case that [HW-AC-0005](../acceptance-criteria/0005-the-governing-documents-of-a-path-are-the-same-for-every-spelling-of-it-and-a-path-no-anchor-matches-has-none.md) needs:

- `query/specs/ingest.md` (`SPEC-FIX-ingest`) governs the pattern `src/ingest/**`.
- `query/decisions/edge-throttling.md` (`DR-FIX-0031`) governs the list `[src/ingest/rate_limit.rs, src/ingest/mod.rs]`.

So each stub matches two anchors, and the directory `src/ingest` matches the pattern only. `src/nothing.rs` is not on the tree, and no anchor matches it.

Two tests read this design:

- `the_fixture_tree_answers_the_recorded_reads` in `engine/crates/query/tests/reads.rs` asks `governing_docs_for_path` about `src/ingest/rate_limit.rs`, `src/ingest/mod.rs`, `src/ingest` and `src/nothing.rs`. It records each answer in `engine/crates/query/fixtures/query.reads`, and it compares the report byte for byte.
- `the_related_and_governing_tools_read_every_spelling_of_a_path_as_explain_does` in `engine/crates/query/tests/mcp.rs` asks the MCP tool about `src/ingest/mod.rs` in three more spellings, and about two paths outside the root.

The recorded report is what makes a missing governor visible. A change that dropped a list member from the answer moves the bytes of `query.reads`, and the test then fails.

## What this does not cover

This document governs the two stubs only. The documents under `query/` and the other files of the fixture tree serve many other reads in the same two test files, so this document is not their design.

The tests do not cover a governor reached through a symlink, or a path that a `..` segment takes outside the root and back in. `headwater explain` on a code path gives the same governors from the command line, and this design does not test that route.

Comments in both test files cite `HW-VER-0004`, and either one binds the `cited_in` edge onto its file.
