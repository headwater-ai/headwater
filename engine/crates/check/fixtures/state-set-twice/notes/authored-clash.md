---
id: NOTE-FIX-authored-clash
status: superseded
summary: an authored document that `supersedes` and `retires` tell two states
---

# authored-clash

`notes/authored-superseder.md` declares `supersedes` here and
`notes/authored-retirer.md` declares `retires` here. The author writes the state
of this document, and generate writes nothing onto it, so this rule reads it
not at all.
