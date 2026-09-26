---
"headwater:generated": "state_pages. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: NOTE-FIX-gen-clash
status: retired
summary: a generated page that `supersedes` and `retires` tell two states
---

# gen-clash

The decisive case. `notes/clash-superseder.md` declares `supersedes` here and
`notes/clash-retirer.md` declares `retires` here. Generate writes `retired`,
the first in sorted order, and `superseded` is lost.
