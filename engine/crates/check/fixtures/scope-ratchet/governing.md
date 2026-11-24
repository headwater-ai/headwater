---
id: SR-FIX-governing
relations:
  governs:
    - to: scope-ratchet-src/a.rs
    - to: scope-ratchet-src/new_governed.rs
  cites:
    - to: scope-ratchet-src/new_ungoverned.rs
  governs_elsewhere:
    - to: scope-ratchet-src/new_ungoverned.rs
---

# The document that governs two files

It governs `a.rs` and `new_governed.rs`, and no other file under the scope. It cites `new_ungoverned.rs`, and it governs that file as an `other_path`, and neither edge governs it in the scope.
