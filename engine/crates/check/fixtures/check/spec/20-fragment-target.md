---
id: SPEC-FIX-fragment-target
doc_type: design_spec
status: current
status_since: 2026-01-05
summary: The document whose heading another document cites by name.
---

# Fragment target

`00-both-halves.md` writes a link at the heading below, by path and by fragment. Nothing else in this tree points here, so a test may rename that heading without moving any other verdict.

## A heading that another document cites

`tests/renamed_heading.rs` renames this heading in a copy of this tree, and the citation in `00-both-halves.md` is what turns red when it does.
