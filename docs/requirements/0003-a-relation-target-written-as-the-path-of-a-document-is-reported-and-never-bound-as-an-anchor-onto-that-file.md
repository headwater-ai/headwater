---
id: HW-REQ-0003
status: current
status_since: 2026-10-02
summary: "A relation entry that writes the path of a document gets a finding that names the identifier to write, and no anchor."
last_verified: 2026-10-02
title: "A relation target written as the path of a document is reported, and never bound as an anchor onto that file"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  verified_by:
    - HW-AC-0003
    - relation.target.unresolved
---

# A relation target written as the path of a document is reported, and never bound as an anchor onto that file

## Context

[Q4](../decisions/0004-relation-storage.md) rules that the target of a relation is an identifier. Some relations admit a document and also a `code_path` anchor, and `traces_to` is one of them. For such a relation, the engine asks the document index first, and the index matches an identifier only.

Before [#1410](https://github.com/headwater-ai/headwater/issues/1410), a path to a document fell through to the `source-tree` resolver. That resolver binds any file that exists. So the entry became an anchor onto a Markdown file. It passed `check --strict`, and the document that the author meant never showed the edge. [#1417](https://github.com/headwater-ai/headwater/issues/1417) closed the same gap for a path through a symlink.

The cost goes to a reader. A reader of `headwater explain` on the target document sees no incoming edge, and nothing tells the author that the edge is missing.

## Requirement

When a relation entry writes the path of a document that has an identifier, `headwater check` reports one finding. The finding names the identifier to write in place of the path.

The graph holds no anchor for that entry. The path does not become a `code_path` anchor onto the Markdown file.

The requirement applies to every spelling of the path. A path with `./`, a path with a `..` segment, and a path through a symlink each name the same document. So each gets the same finding.

The requirement does not apply to a path that names no document. A path to a source file, a wildcard, and a Markdown file with no identifier each stay an anchor with no finding. A file with an identifier and no kind also stays an anchor. A relation that admits only a `code_path` anchor, such as `governs`, also keeps the path as an anchor.

## Verification

Two artifacts verify this requirement.

`relation.target.unresolved` is the check rule that reports the finding. The entry above points at it through the anchor kind `check_rule`. `.githooks/pre-commit` runs the rule on every commit, and CI runs it on every pull request.

[HW-AC-0003](../acceptance-criteria/0003-check-reports-a-document-path-written-as-a-relation-target-once-in-every-spelling-and-through-a-symlink.md) is the acceptance criterion. Its method is `test`, and [HW-VER-0002](../verifications/0002-a-fixture-tree-of-document-paths-symlinks-and-identifiers-proves-the-document-path-finding.md) is the test design that proves it.
