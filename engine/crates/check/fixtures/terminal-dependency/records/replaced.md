---
id: NOTE-FIX-replaced
status: superseded
status_since: 2026-08-01
summary: a record a successor replaced, standing at the state that edge wrote
---

# The record a successor replaced

`live/succeeds-replaced.md` declares `supersedes` at this document, and
`supersedes` declares `on_target: {set_state: superseded}`. This edge is the
cause of the state and never a dependency on it.
