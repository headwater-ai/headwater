---
id: AUD-FIX-0004
doc_type: manual
status: current
status_since: 2025-05-01
last_verified: 2025-06-01
summary: a manual that nobody has verified for two seasons, so every half it declares is past the declared window
relations:
  catalogues:
    - AUD-FIX-0001
---

# The stale manual

This is the document that makes `catalogues` a finding. Move `last_verified`
forward and the finding goes, which is what the regression test asserts.
