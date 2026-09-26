---
"headwater:generated": "state_pages. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: NOTE-FIX-gen-inverse
status: superseded
summary: a generated page that sets a state through an inverse half another file wrote
---

# gen-inverse

`notes/inverse-superseder.md` declares `supersedes` here.
`notes/retired-by-inverse.md` writes `retired_by` toward this page, the inverse
of `retires`, so the relation runs from this page to that file. Generate reads
no state from that edge. A rule that read the writing file as the source
reports a clash here.
