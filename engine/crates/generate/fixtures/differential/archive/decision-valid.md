---
doc_type: decision
id: DR-DIF-0005
status: current
status_since: 2026-03-05
summary: a valid decision on the shelf whose kind a front-matter member states
---

# The document that catches a root `oneOf`

This document is valid, and it is valid against more than one branch of the schema, because a kind and the abstract kind above it accept the same front matter. An emitter that selected with `oneOf` reports an error here that the engine does not report. The differential fails in that direction, and that is the direction a loss set cannot redeem.
