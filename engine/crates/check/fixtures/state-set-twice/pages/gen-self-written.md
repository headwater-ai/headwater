---
"headwater:generated": "state_pages. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: NOTE-FIX-gen-self-written
status: superseded
relations:
  retires:
    - NOTE-FIX-gen-self-written
summary: a generated page that one edge supersedes and that names itself under `retires`
---

# gen-self-written

`notes/self-superseder.md` declares `supersedes` here. This page itself writes
`retires` at its own identifier. Generate reads no edge the page wrote, because
the page would then be a function of its own last version, so only
`superseded` reaches it and nothing clashes.
