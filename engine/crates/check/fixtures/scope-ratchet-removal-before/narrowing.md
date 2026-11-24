---
id: SRR-FIX-narrowing
relations:
  governs:
    - to: scope-ratchet-removal-src/kept.rs
    - to: ./scope-ratchet-removal-src/dropped.rs
    - scope-ratchet-removal-src/shared.rs
    - to: [scope-ratchet-removal-src/narrowed*.rs]
  cites:
    - to: scope-ratchet-removal-src/stray.rs
  governs_elsewhere:
    - to: scope-ratchet-removal-src/stray.rs
  governed_by:
    - to: scope-ratchet-removal-src/old_debt.rs
---

# The document that a change narrowed, as it stood before the change

It governed `kept.rs`, `dropped.rs`, `shared.rs` and every file that `narrowed*.rs` matches, and it cited `stray.rs` and governed it as an `other_path`. The entries use three shapes: a `to:` scalar, a bare scalar and a `to:` list, and the `dropped.rs` entry opens with `./`. The `governed_by` entry is the inverse name, which names a document, and its value only looks like a path.
