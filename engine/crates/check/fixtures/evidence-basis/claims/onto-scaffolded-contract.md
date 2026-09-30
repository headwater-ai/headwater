---
id: NOTE-FIX-onto-scaffolded-contract
status: current
status_since: 2026-08-02
summary: an evidenced claim whose pointer reaches a contract exactly as headwater new writes it
provenance:
  warrant: accepted
  agency: human
  accepted_by: a.person
  evidence_basis: evidenced
relations:
  traces_to:
    - NOTE-FIX-scaffolded-contract
---

# The evidenced claim resting on a scaffolded contract

The target is `interfaces/scaffolded.md`, whose front matter is what `headwater new` writes for an `interface_contract`. The scaffolder writes `warrant: asserted`, so this edge is reported rather than skipped (#1409). `every_kind_the_scaffolder_writes_states_warrant_asserted` in `headwater-scaffold` holds the target's provenance block to the bytes the scaffolder renders.
