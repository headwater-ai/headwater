---
doc_type: guide
id: GD-DIF-listed-audience
status: current
status_since: 2026-03-07
summary: a guide whose audience list holds one declared value and one outside the set
audience: [operator, nobody]
---

# A guide for a list of audiences, one undeclared

The engine reads each scalar item of a list against the set (#1873). A schema that admitted any list, as the guard did before, misses this document, and a schema that enumerated the list as one value would reject the guide beside it.
