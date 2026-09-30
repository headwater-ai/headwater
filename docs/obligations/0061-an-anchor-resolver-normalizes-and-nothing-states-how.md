---
id: HW-OBL-0061
title: "An anchor resolver normalizes, and nothing states how"
status: current
status_since: 2026-08-12
waiting_on: build
last_verified: 2026-09-30
summary: "Spec 2 requires that anchor strings normalize before comparison, and it states no rule that does it."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-taxonomy-model
    - HW-SPEC-check-layer
---

# An anchor resolver normalizes, and nothing states how

## Context

[Spec 2](../spec/02-taxonomy-model.md#behavior-at-the-limits) requires that anchor strings normalize before comparison, so that two spellings of one target are one node. It states no rule that does it. [Spec 12](../spec/12-check-layer.md#the-correctness-roots) then names the resolver a correctness root, because a resolver that mis-normalizes makes `governs` edges miss with no error anywhere.

## Obligation

The engine had to decide the rules, and a decision of that class belongs to the meta-schema.

## Discharge

`engine/crates/graph/src/anchors.rs` states them. A backslash reads as a separator. A `.` segment goes, and a `..` segment cancels the segment before it. An absolute path, and a path that climbs above the repository, are refused rather than clamped. Every rule is lexical, because a resolver that asks the filesystem follows a symlink out of the repository. One comparison asks the filesystem after normalization, and not as part of it. `Resolver::real_path` follows each symlink on a path to find a typed document that the path reaches by a second name. It gives no path where the real path leaves the repository (#1417).

A second reader normalizes a path. The MCP tools `explain`, `related` and `governing_docs_for_path` read a typed path with `headwater_census::walk::within`, which is `walk::typed` with a symlink check added. The rule between the two readers is this: on a spelling that both accept, they give the same path (#1314). So `./a/b`, `a/./b`, `a//b`, `a/b/` and `x/../a/b` are `a/b` on both sides, and both refuse `../x` and an absolute path outside the repository. The two readers differ on five spellings, and each difference has a reason:

- **An absolute path under the repository root.** The tools accept it and the anchor resolver refuses it. A reader types an absolute path from a shell, but an anchor is written into a portable document.
- **A path through a symlink that leads out of the repository.** The anchor resolver keeps `link/x` lexically, and the tools refuse it. A tool answers about a file on the host, and that file is not in the repository (#1249).
- **A backslash.** The anchor resolver reads `\` as a separator, because a path written on Windows names the same file. On POSIX, a typed `\` is a character of a file name, so the tools keep it.
- **Surrounding whitespace.** The anchor resolver trims it as a tolerance for authors. The tools read the path as typed.
- **The empty path, and `.`.** The anchor resolver refuses both, because an anchor that names nothing is a defect. The tools read both as the repository root.

The test `the_anchor_resolver_and_the_path_tools_agree_on_every_spelling_both_accept` in `engine/crates/query/tests/reads.rs` holds this rule. It fails on a difference that this section does not name.

This is a meta-schema decision of the same class as [the shelf order](0060-the-most-specific-shelf-wins-names-no-order.md).
