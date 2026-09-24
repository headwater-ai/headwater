---
id: DR-FIX-0002
title: Retiring decision
status: current
status_since: 2026-04-01
last_verified: 2026-05-01
summary: the one document on the `flaggers` shelf, whose edge retires the written notice.
relations:
  flags_written:
    - WR-FIX-written
---

# Retiring decision

The setter of the written case. No projection reads the `flaggers` shelf, so its dates differ from those of the read set on purpose: `status_since` must come from here, and `last_verified` must not.
