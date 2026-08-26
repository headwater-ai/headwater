---
id: HW-OBL-0144
status: draft
status_since: 2026-08-26
last_verified: 2026-08-26
title: "The cache-identity test reads the live working tree, so an edit during the run reports the correctness root as violated"
summary: "The test proving cache and no-cache agree on a verdict reads the live working tree, and a concurrent edit reports the correctness root as violated."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
waiting_on: adopter
---

# The cache-identity test reads the live working tree, so an edit during the run reports the correctness root as violated

## Context

This repository's own test suite proves that a cached run and an uncached run report the same findings. `this_repository_reports_the_same_run_from_a_cache_as_from_none`, at engine/crates/check/tests/fixtures.rs:460, does this by running the corpus three times over `repository_root()`, the live working tree rather than a copy. Nothing pins the tree between the three runs, so a document edited while the suite runs can move under any of them.

Measured on 2026-08-21, a paragraph added to docs/spec/07-distribution-and-federation.md during `cargo test --workspace` produced one failure among twenty-six tests. The failing assertion, at fixtures.rs:476, compared the second cached run against the uncached one, and both sides rendered in full. They differed only in the two spec 7 line numbers the inserted paragraph shifted. The same test alone, with no edit in flight, passed in 10.72 seconds. The failure cannot occur in continuous integration, where the runner's tree is static for the life of the job. So only a person or an agent editing while testing ever sees it, and building issue #314 is where this one did.

This is the same class of failure as issue #302, a blocking local failure that never fires in continuous integration. It reads as the change under test breaking the strongest invariant this repository states. An agent once recorded, in durable memory, that the cache invariant was false, on a weaker signal than this one. That is the cost a misdiagnosis of this kind extracts from a reader who trusts the correctness root.

## Obligation

The corpus owes a fix to `this_repository_reports_the_same_run_from_a_cache_as_from_none` so the test no longer reads a tree that a concurrent edit can move. Today it compares three renders of `repository_root()`. Either of the two assertions, at fixtures.rs:475 or fixtures.rs:476, can fail for an edit landing in its window rather than for a cache defect.

The cache property itself is not in question. Every measurement here is of a test reading a moving input, not of a cache changing a verdict. `tests/cache.rs` already proves the same property over its own fixture tree, which nothing outside the test can move. Its reasoning for reading the live corpus here still holds and is not disputed. The report and the property then describe one set of numbers.

The other corpus-reading assertions in the same file compare against recorded totals and can fail the same way for the same reason. Only this one test is measured here, and a fix to it does not have to cover them.

## Discharge

This closes when the test proves the same claim without reading a tree an editor can move. A snapshot taken once and compared against itself three times would satisfy it. So would a check that names a mid-run edit as the cause, rather than rendering two disagreeing reports. Either fix must keep the tie the test's own doc comment protects. The report it compares stays a report of the same corpus as the recorded `corpus.checks` fixture beside it. `cargo test --workspace` staying green with an edit landing during the run is the practical proof the fix holds.

This waits on an adopter to choose the fix and land it.
