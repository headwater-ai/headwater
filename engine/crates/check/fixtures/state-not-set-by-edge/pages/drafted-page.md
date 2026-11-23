---
"headwater:generated": "state_pages. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: NOTE-FIX-drafted-page
status: current
summary: a generated page that a draft successor supersedes
---

# drafted-page

This engine writes the state of a generated page, so `lifecycle.state.set_after_initial` skips it as `lifecycle.state.not_set_by_edge` does.
