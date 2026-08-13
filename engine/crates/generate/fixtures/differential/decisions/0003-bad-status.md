---
id: DR-DIF-0003
status: retired
status_since: 2026-03-03
summary: a decision whose state is outside the declared set
---

# A decision with a state nobody declared

The taxonomy declares `draft`, `current` and `superseded`, and `retired` is none of them. The engine reports `facet.value.not_permitted`, and the emitted schema fails the `enum` keyword on the same member of the same document.
