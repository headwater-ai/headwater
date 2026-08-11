---
status: current
status_since: 2026-03-09
last_verified: 2026-07-01
summary: Three findings from the first pass over the core model, two accepted.
doc_type: review_record
applies: docs/reviews/first-pass-prompt.md
assesses:
  - docs/spec/01-model.md
provenance:
  warrant: accepted
  agency: mixed
  accepted_by: fixture-acceptor
  evidence_basis: evidenced
---

# First pass over the core model: findings

## The run

Run on 2026-03-09 against the revision of spec 1 current on that date.

## Findings

### F1 — "Outcome" is defined twice

**What.** The term is defined in the opening paragraph and again under the attempt, with different wording.

**Where.** [1 — The core model](../spec/01-model.md).

**Why it matters.** A reader who meets the second definition first reads the terminal set as open.

**Disposition.** Accepted. One definition survives.

### F2 — The ordering claim has no evidence

**What.** The model says ordering between destinations does not matter, and nothing measured it.

**Where.** [1 — The core model](../spec/01-model.md), Limits.

**Why it matters.** The claim moved to spec 4 and was never tested there either.

**Disposition.** Deferred to the obligation register.

## What the instrument missed

The prompt asked nothing about the attempt identifier, which is the part a receiver depends on.
