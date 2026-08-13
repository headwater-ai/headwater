---
id: DR-DIF-0002
status: current
status_since: 2026-03-02
audience: author
---

# A decision with no summary

Every kind that descends from `governed_document` requires `summary`, and this document does not state it. The engine reports `facet.required.missing`, and the emitted schema fails the `required` keyword. The two sets must agree.
