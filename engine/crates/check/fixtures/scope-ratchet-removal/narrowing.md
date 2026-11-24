---
id: SRR-FIX-narrowing
relations:
  governs:
    - to: scope-ratchet-removal-src/kept.rs
    - to: scope-ratchet-removal-src/narrowed1*.rs
---

# The document that a change narrowed

It governs `kept.rs` and the files that `narrowed1*.rs` matches. The version before the change also governed `dropped.rs`, `shared.rs` and every file that `narrowed*.rs` matches, and it cited `stray.rs`.
