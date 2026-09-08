---
id: SPEC-FIX-both-halves
doc_type: design_spec
status: current
status_since: 2026-01-05
summary: The complete pair, which is the passing fixture the reciprocity rule needs.
relations:
  cites_evidence:
    - EVAL-FIX-alpha
---

# Both halves

`alpha.md` writes `cited_by: SPEC-FIX-both-halves`, so this pair is complete and the reciprocity instance over it passes. It is the passing fixture that spec 12 requires beside a failing one.

It is the passing half for the prose rules too. The voice regime finds nothing, the house profile finds nothing, and the fragment below resolves.

## Where the pair is written

Both halves are in the front matter, and the link back to [this section](#where-the-pair-is-written) is the fragment that resolves.

The path in [the cited-only fixture](02-cited-only.md) resolves as well, which is the passing half that the path rule needs. `tests/renamed_target.rs` renames that file in a copy of this tree, and this line is what turns red when it does.

A fragment on a resolving path, at [a heading of another document](20-fragment-target.md#a-heading-that-another-document-cites), which is the passing half the cross-document arm of the fragment rule needs. `tests/renamed_heading.rs` renames that heading in a copy of this tree, and this line is what turns red when it does.

A query string is not part of a filename. [The fragment target with a version on the end](20-fragment-target.md?v=2) resolves to the file its path names.

A destination that opens with a separator names the root of the corpus. [The fragment target under that other spelling](/check/spec/20-fragment-target.md) is the same file again.
