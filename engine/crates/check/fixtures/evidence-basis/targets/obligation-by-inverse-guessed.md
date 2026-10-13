---
id: NOTE-FIX-obligation-by-inverse-guessed
status: current
status_since: 2026-10-13
summary: an evidenced obligation that names, through the inverse half, a discharging evaluation whose warrant is outside the closed set
provenance:
  warrant: accepted
  agency: human
  accepted_by: a.person
  evidence_basis: evidenced
relations:
  discharged_by:
    - NOTE-FIX-guessed-evaluation-by-inverse
---

# The obligation that writes the inverse half onto a guessed evaluation

The edge is `discharges` from the evaluation to this document, and this document is the one that wrote it, as `discharged_by`. The evaluation's warrant is `guessed`, which is not a value of spec 3's closed set, so the pair is reported. The finding anchors here, and its remediation names the `discharged_by` entry in this file. It does not name a `discharges` entry, because no such entry exists.
