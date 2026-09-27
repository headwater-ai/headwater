---
"headwater:generated": "state_pages. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: NOTE-FIX-gen-page
status: current
summary: a generated page that a successor supersedes
---

# gen-page

This engine writes the state of a generated page, so `lifecycle.state.set_twice` and `generate --check` read it and this rule does not.
