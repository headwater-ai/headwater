---
id: NOTE-FIX-obligation-by-inverse
status: current
status_since: 2026-08-02
summary: an evidenced obligation that names its own discharging evaluation through the inverse half
provenance:
  warrant: accepted
  agency: human
  accepted_by: a.person
  evidence_basis: evidenced
relations:
  discharged_by:
    - NOTE-FIX-evaluation-by-inverse
---

# The obligation that writes the inverse half

The edge is `discharges` from the evaluation to this document, and this document is the one that wrote it, as `discharged_by`. The evaluation is `asserted`, so the pair is reported. The finding anchors here, and its remediation names the `discharged_by` entry in this file. It does not name a `discharges` entry, because no such entry exists.
