---
id: SPEC-FIX-untyped
---

# Untyped

No `doc_type` on a heterogeneous shelf, so the census gives it no kind. It gets
no instance of a kind's template, and it is **not** an OB-COV-2 finding, because
coverage reads typed rows. It is one `shelf.discriminator.unresolved` error at
line 1: spec 2 states that a missing discriminator is a finding, and without
that rule this document left every check.
