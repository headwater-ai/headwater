---
id: HW-REQ-0004
status: current
status_since: 2026-10-02
summary: "Each list anchor in the native graph export carries its sorted members as `patterns` beside its encoded id."
last_verified: 2026-10-02
title: "The graph export carries the member patterns of a list anchor"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  verified_by:
    - HW-AC-0004
---

# The graph export carries the member patterns of a list anchor

## Context

[HW-DR-0074](../decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md) rules that a `governs` entry written as a flow list is one anchor over the union of its members. The graph gives that anchor an `id` in a length-prefixed encoding, such as `8:src/a.rs8:src/b.rs`.

Before [#1247](https://github.com/headwater-ai/headwater/issues/1247), the native export carried that `id` and nothing else. A consumer that read the `id` as a path saw a path that matches no file. A consumer that split a joined display string on `, ` read the wrong members when a member name holds `, `.

A consumer outside this repository, such as a coverage dashboard, reads the export to show which files a document governs. It needs the members, and it must not parse the encoding to get them.

## Requirement

In the native graph export, each list anchor carries a `patterns` field. The field holds the normalized members of the list, in sorted order.

The anchor node carries the field, and so does every edge target that is bound to that anchor.

The `id` stays as it is, so a consumer that keys on it sees no change.

An anchor with a single pattern carries no `patterns` field, because its `id` is already its one pattern.

## Verification

[HW-AC-0004](../acceptance-criteria/0004-each-list-anchor-in-the-native-export-carries-its-sorted-members-and-a-single-pattern-carries-none.md) is the acceptance criterion. Its method is `test`, and [HW-VER-0003](../verifications/0003-a-fixture-tree-of-three-list-anchors-proves-that-the-export-carries-list-members.md) is the test design that proves it.
