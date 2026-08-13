---
doc_type: guide
id: GD-DIF-bad-audience
status: current
status_since: 2026-03-07
summary: a guide that names an audience the taxonomy does not declare
audience: nobody
---

# A guide for an audience nobody declared

The `audience` facet is enumerated and it is not required. A schema that emitted `enum` only for the required facets misses this document, and the differential then reports a finding that the engine has and the validator does not.
