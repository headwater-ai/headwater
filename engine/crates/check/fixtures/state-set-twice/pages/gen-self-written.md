---
"headwater:generated": "state_pages. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: NOTE-FIX-gen-self-written
status: current
relations:
  superseded_by:
    - NOTE-FIX-self-superseder
  retired_by:
    - NOTE-FIX-self-retirer
summary: a generated page whose own inverse halves would clash, which generate never reads
---

# gen-self-written

This page writes `superseded_by` and `retired_by`. Generate reads no edge the
page wrote, because the page would then be a function of its own last version,
and it reads no inverse half. So generate writes the `live` state here and
nothing clashes.
