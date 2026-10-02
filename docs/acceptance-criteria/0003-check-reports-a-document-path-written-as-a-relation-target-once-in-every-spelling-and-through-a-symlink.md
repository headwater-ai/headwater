---
id: HW-AC-0003
status: current
status_since: 2026-10-02
summary: "The rule reports each document path once with the identifier to write, in every spelling and through symlinks, and no other path."
last_verified: 2026-10-02
title: "Check reports a document path written as a relation target once, in every spelling and through a symlink"
verification_method: test
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  verifies:
    - HW-REQ-0003
  proven_by:
    - HW-VER-0002
---

# Check reports a document path written as a relation target once, in every spelling and through a symlink

## Fit criterion

Over the fixture tree at `engine/crates/check/fixtures/document-path-target/`, `relation.target.unresolved` gives these results:

- A relation entry that writes the path of `NOTE-FIX-b` gets one finding, and the finding names `NOTE-FIX-b`.
- A path with `./` or a `..` segment gets the same finding as the plain path.
- One entry that writes the path in two spellings gets one finding and one repeated target.
- A path through a symlinked file, and a path through a symlinked directory, each get the same finding.
- A path beside the identifier gets one finding, and the identifier still binds to the document.
- The identifier form binds to the document, and the document has the reverse edge.
- A source file, a symlink onto one, a wildcard, and a Markdown file with no identifier each stay an anchor with no finding.
- A file with an identifier and no kind, a target under a resolver that is not `source-tree`, and a path under `governs` also stay anchors.

For each entry that gets a finding, the graph holds no anchor.

The criterion is held on every change, because CI runs the workspace suite on every pull request.

## Method

Test. `engine/crates/check/tests/document_path_target.rs` builds the graph over the fixture tree and runs the check. It has one test for each result in the list above. [HW-VER-0002](../verifications/0002-a-fixture-tree-of-document-paths-symlinks-and-identifiers-proves-the-document-path-finding.md) states the test design.

    cargo test -p headwater-check --test document_path_target --manifest-path engine/Cargo.toml --locked

The fixture documents decide the verdict, and no person reads a tree. That is what makes the method `test` and not `inspection`.
