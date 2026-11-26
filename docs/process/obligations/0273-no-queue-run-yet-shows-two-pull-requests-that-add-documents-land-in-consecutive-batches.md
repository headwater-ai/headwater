---
id: HW-OBL-0273
status: current
status_since: 2026-11-26
summary: "No merge-queue run shows a pull request that adds a document land behind another one with no hand update from main. Clause 5 of #1273 owes that evidence."
last_verified: 2026-11-26
title: "No queue run yet shows two pull requests that add documents land in consecutive batches"
waiting_on: measurement
provenance:
  warrant: asserted
relations:
  traces_to:
    - HW-DR-0049
---

# No queue run yet shows two pull requests that add documents land in consecutive batches

## Context

Issue #1273 removed the measured figures from the committed pages under `site/`. The merge queue can then land pull requests in batches. Its clauses 1 to 4 and 6 are met on `main` through #1277, #1380 and #1525. The owner closed #1273 on 2026-11-27 and moved clause 5 to this record ([ruling](https://github.com/headwater-ai/headwater/issues/1273#issuecomment-7200444015)).

Clause 5 asks for evidence from the queue, and a build agent cannot build that evidence. The cause that clause 5 tested is gone, and [HW-DR-0049](../../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md) states why no fold stays in the tree. The adjudication of 2026-11-27 read every merge to `main` from 2026-10-20 that added a document, and 0 of 35 formed the pair. To make the pair now, somebody must write two documents that no reader needs.

## Obligation

The corpus owes evidence of one pair of pull requests that each add a document. The two land through the merge queue, one batch after the other, with no hand update from `main`. The second pull request was open while the first one landed.

## Discharge

This record discharges when the integrator posts the merge-group run identifiers of the first such pair here. Reopen #1273 if the queue ejects a pull request that adds a document on a conflict in a derived file. Reopen it also if a parent must update such a pull request from `main` by hand.
