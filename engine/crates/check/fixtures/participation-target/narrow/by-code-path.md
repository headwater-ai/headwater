---
id: PT-FIX-narrow-by-code-path
status: current
status_since: 2026-01-05
summary: verified by a bound code-path anchor, which is not the acceptance_criterion this arm names
relations:
  verified_by:
    - participation-target/code/widget.rs
---

# Verified by a bound anchor

`verified_by` admits `code_path` at its target end, and this edge names a real file the source-tree resolver binds. A bound anchor is a neighbour ([#855](https://github.com/headwater-ai/headwater/issues/855)), and this arm's `to_kind: acceptance_criterion` still does not accept it: an anchor kind descends from nothing this taxonomy declares under `kinds:`.
