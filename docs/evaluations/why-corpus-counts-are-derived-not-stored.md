---
id: HW-EVAL-why-corpus-counts-are-derived
status: current
status_since: 2026-09-16
last_verified: 2026-09-16
title: "Why corpus counts are derived, not stored"
summary: "Counts over the whole corpus are computed when read, not kept in committed files. Stored counts cause silent merge failures when two branches write the same total but the merged tree holds a different number."
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-haiku-4-5
  activity: draft
  evidence_basis: grounded
relations:
  traces_to:
    - HW-DR-0049
    - HW-PD-0005
---

# Why corpus counts are derived, not stored

## The problem: silent merge failures

A count over the whole corpus is a fold—a single number that summarizes all records below it. Examples include the count of documents in the corpus, the count of edges in the graph, or the count of files under the corpus root.

When a count is stored in a committed file, two branches that each add a document create a merge conflict. One branch increments the count from 386 to 387. The other branch also increments it from 386 to 387. When git merges the two branches, it sees two identical changes and merges them as one.

The tree now holds 388 documents—the original 386 plus one new document in each branch—but the committed count says 387. The mismatch is silent. Git reports no conflict. A person reading the file sees a count that is wrong by one, with no warning.

This is called the "quiet form" of merge conflict. Two branches write the same value, but the merged tree holds a different total. A merge driver cannot catch it, because git never calls a driver when two identical blobs need no merge.

## The solution: derive counts when reading

Instead of storing a count, derive it. When you need to know how many documents the corpus holds, read all documents and count them. When you need the statistics, recompute them from the records. The count you get is always correct for the current state of the tree.

When two branches merge silently, the correct count is the number of records actually present. The count is not a number that either branch wrote.

## Why this matters for this repository

[HW-DR-0049](../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md) rules that "A corpus-wide fold is derived and never stored." This repository used to store counts in committed files: the census held a file count, and the graph held edge counts and anchor statistics. All three are now derived when needed.

The log ledger stores one record per iteration, never a total. When you need the count of open issues or the count of iterations, the integrator computes it from the log records.

## For adopters

If you run Headwater on your own corpus, the same rule applies. Any count over the whole corpus must be derived, never stored. A second person editing your corpus in parallel will not corrupt statistics by accident, because no statistics are committed to silently merge.

See [HW-DR-0049](../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md) for the full design reasoning and the measurement that led to it.
