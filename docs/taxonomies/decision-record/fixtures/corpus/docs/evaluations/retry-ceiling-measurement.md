---
id: EVAL-repo-retry-ceiling
title: How far apart two attempts of one delivery fall
status: current
status_since: 2026-06-15
last_verified: 2026-07-01
summary: A measurement of inter-attempt intervals in production, against the one-hour deduplication window.
provenance:
  warrant: accepted
  agency: mixed
  accepted_by: fixture-acceptor
  evidence_basis: evidenced
relations:
  discharges:
    - HW-OBL-0001
    - HW-OBL-0003
---

# How far apart two attempts of one delivery fall

## Method

Every attempt in one week of production, grouped by delivery, with the interval between the first attempt and the last.

## Result

The ninety-ninth percentile interval is eleven minutes. The maximum is fifty-two minutes, which sits inside the one-hour window and close to its edge.

## What it does not settle

One week is one week. A destination that goes down for a day produces intervals this measurement never saw.

This document also claims to discharge `HW-OBL-0003`, which it does not measure and which does not name it back. That claim is planted, and the reciprocity rule is what reports it.
