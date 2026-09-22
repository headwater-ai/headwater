---
id: PT-FIX-broad-by-code-path
status: current
status_since: 2026-01-05
summary: verified by a bound code-path anchor, which this arm's absent to_kind accepts
relations:
  verified_by:
    - participation-target/code/widget.rs
---

# Verified by a bound anchor

`verified_by` admits `code_path` at its target end, and this edge names a real file the source-tree resolver binds. This arm names no `to_kind`, so a bound anchor neighbour satisfies it the same as any other ([#855](https://github.com/headwater-ai/headwater/issues/855)).
