---
id: PT-FIX-narrow-by-probe
status: current
status_since: 2026-01-05
summary: verified by a probe alone, which is the target kind a scalar to_kind cannot reach
relations:
  verified_by:
    - PT-FIX-probe-one
---

# Verified by a probe

The relation admits a probe at its target end, so this edge is legitimate and complete. The narrow arm reports it as unverified anyway, and that is the defect this tree pins.
