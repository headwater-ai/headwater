---
id: SPEC-FIX-no-summary
doc_type: design_spec
status: current
status_since: 2026-01-05
---

# No summary

The failing fixture for the required-facet rule. `governed_document` requires `summary` and `design_spec` inherits that requirement, so this document owes a key it does not declare.

The finding carries no line, because the finding is that a line is absent, and no fix is offered: the value of a `summary` is a sentence somebody has to write.
