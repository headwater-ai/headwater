---
status: current
status_since: 2026-02-03
last_verified: 2026-07-01
summary: Every decision the Beacon design settled, and the evidence that settled it.
doc_type: decision_register
sequence: 9
cites_evidence:
  - docs/evaluations/queue-durability.md
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: evidenced
---

# 9 — The decision register

This file records decisions that are settled. It accepts no open questions. Work that a decision left behind goes to spec 13.

## Q1 — Which broker carries the queue

**The decision.** The durable log, not the in-memory broker.

**Why.** An attempt that a restart loses is an attempt that no operator can account for, and the record of delivery is the product.

**The evidence.** [Queue durability](../evaluations/queue-durability.md), which measured recovery after a forced restart on both candidates.

**What it left open.** The retention window is a cost question that nobody has priced. Spec 13 holds it.
