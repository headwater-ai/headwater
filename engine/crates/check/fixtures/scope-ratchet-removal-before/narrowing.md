---
id: SRR-FIX-narrowing
relations:
  governs:
    - to: scope-ratchet-removal-src/kept.rs
    - to: scope-ratchet-removal-src/dropped.rs
    - scope-ratchet-removal-src/shared.rs
    - to: [scope-ratchet-removal-src/narrowed*.rs]
  cites:
    - to: scope-ratchet-removal-src/stray.rs
---

# The document that a change narrowed, as it stood before the change

It governed `kept.rs`, `dropped.rs`, `shared.rs` and every file that `narrowed*.rs` matches, and it cited `stray.rs`. The entries use three spellings: a `to:` scalar, a bare scalar and a `to:` list.
