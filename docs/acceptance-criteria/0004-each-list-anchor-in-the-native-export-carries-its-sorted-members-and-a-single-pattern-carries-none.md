---
id: HW-AC-0004
status: current
status_since: 2026-10-02
summary: "Over one fixture decision, each of three list anchors carries its sorted members in the export, and a single anchor carries none."
last_verified: 2026-10-02
title: "Each list anchor in the native export carries its sorted members, and a single pattern carries none"
verification_method: test
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  verifies:
    - HW-REQ-0004
  proven_by:
    - HW-VER-0003
---

# Each list anchor in the native export carries its sorted members, and a single pattern carries none

## Fit criterion

The fixture decision `DR-FIX-0001` governs four entries: one single path and three lists. The native JSON export of that corpus gives these results:

- The export holds four anchor nodes and four `governs` edges bound to an anchor, one for each entry.
- The single path `src/c.rs` carries no `patterns` field.
- Each of the three lists carries `patterns`, in the anchor node and in its edge target.
- `patterns` holds the members in sorted order, also for the list that is written out of order.
- A member whose name holds `, ` stays one member.
- A list of three members carries all three.

The criterion is held on every change, because CI runs the workspace suite on every pull request.

## Method

Test. The test `the_native_export_carries_the_members_of_a_list_anchor` in `engine/crates/generate/tests/fixtures.rs` builds the fixture corpus and emits the native export. It reads the export back and compares each anchor node and edge target with the expected members. [HW-VER-0003](../verifications/0003-a-fixture-tree-of-three-list-anchors-proves-that-the-export-carries-list-members.md) states the test design.

    cargo test -p headwater-generate --test fixtures the_native_export_carries_the_members_of_a_list_anchor --manifest-path engine/Cargo.toml --locked

The test also checks the counts of nodes and edges first. Without that check, an export with no anchors would pass every comparison and prove nothing.
