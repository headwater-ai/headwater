---
id: NOTE-FIX-asserted-discharges-evidenced
status: current
status_since: 2026-08-02
summary: an evaluation nobody has read, which declares that it discharges an evidenced obligation
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  evidence_basis: unevidenced
relations:
  discharges:
    - NOTE-FIX-evidenced-obligation
---

# The asserted evaluation that discharges an evidenced obligation

This document makes no `evidenced` claim of its own, so a rule that read the claim at the source end passes it. The claim is at the target end, and this document is the evidence for it. A rule that read `evidence_at: from` as "skip" also passes it, and this is the case that reports that mistake.
