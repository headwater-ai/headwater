---
id: HW-REQ-0005
status: current
status_since: 2026-10-02
summary: "Asked about a code path in any spelling, the engine names every document whose governs anchor matches it."
last_verified: 2026-10-02
title: "A code path names every document that governs it"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  verified_by:
    - HW-AC-0005
---

# A code path names every document that governs it

## Context

A person or an agent who is about to change a file asks which documents govern it. `governing_docs_for_path` answers that question in the query library, and the MCP server gives it to an agent as a tool of the same name.

A `governs` anchor is a pattern ([HW-DR-0074](../decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md)). It can be a single path, a pattern such as `src/ingest/**`, or a list of members. So one file can match the anchors of more than one document, and each of those documents governs it.

An answer that leaves out one of those documents is the worst failure here. The caller then changes the file and does not read the rule that applied to it.

## Requirement

Asked about a code path, the engine names every document that has a `governs` anchor that matches the path. That includes a match through a pattern and a match through any member of a list.

The answer is the same for every spelling of a path inside the repository. A path with `./`, a path with a `..` segment, and an absolute path under the root each get the answer of the plain path.

A path that no anchor matches gets an answer that names no document.

A path outside the repository gets one fixed sentence, and no list of documents. [#1249](https://github.com/headwater-ai/headwater/issues/1249) states that sentence.

## Verification

[HW-AC-0005](../acceptance-criteria/0005-the-governing-documents-of-a-path-are-the-same-for-every-spelling-of-it-and-a-path-no-anchor-matches-has-none.md) is the acceptance criterion. Its method is `test`, and [HW-VER-0004](../verifications/0004-the-ingest-stubs-of-the-query-fixture-tree-prove-what-governs-a-path.md) is the test design that proves it.
