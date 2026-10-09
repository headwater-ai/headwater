---
id: HW-OBL-0238
status: current
status_since: 2026-10-09
summary: "HW-DR-0106 declares no edge onto the test that holds it and its reopening condition reads as met by its own examples. HW-DR-0063 is suspect on derived.rs, and one mutant of the credit check survives."
last_verified: 2026-10-09
title: "Four findings that the spec 6 trim of #1572 left in two decisions and their test"
waiting_on: build
provenance:
  warrant: asserted
---

# Four findings that the spec 6 trim of #1572 left in two decisions and their test

## Context

The maintainer and the verifiers of #1572 in run `20261003-1026` wrote these findings as intake lines 1, 2, 24 and 25. None names a reader outside this repository, so the product owner ruled each one RECORD at the end-of-run pass on 2026-10-09. #1572 is closed. Each item was checked on `66dec0f9`. The product owner read the front matter and *Consequences* of HW-DR-0106, the `governs` entry of HW-DR-0063, and `clause_words` in `engine/crates/cli/tests/spec_six_inbound_links.rs`.

## Obligation

- [HW-DR-0106](../decisions/0106-a-governing-document-repoints-a-link-to-a-rule-that-moved-and-a-record-of-a-moment-stays-as-written.md) declares no relation. The test that enforces it, `engine/crates/cli/tests/spec_six_inbound_links.rs`, has no `governs` edge from it. So `headwater explain` on that path names no document.
- [HW-DR-0063](../decisions/0063-every-required-facet-of-a-generated-document-is-derived-and-the-emitter-composes-the-summary.md) governs `engine/crates/generate/src/derived.rs` with a `verified_revision` that differs from the digest of the file today. PR #1676 moved its `last_verified` to 2026-10-03 for a change of a link only. A re-read of `derived.rs` against the decision is owed.
- HW-DR-0106 says the negator rule opens again "when a governing sentence of either shape appears in the corpus". Read as written, its own two quoted examples meet that condition. The condition should exclude a quoted example.
- Mutant M2 of the spec 6 credit check survives. Removing `last.ends_clause |= ends_clause;` in `clause_words` leaves the suite green. A case in which a semicolon, spaced as a separate token, follows the credited clause and one credit is expected with the rule words that precede it would hold it.

## Discharge

Each item discharges alone. The first does when HW-DR-0106 declares the edge, and the second when HW-DR-0063 is re-read and its edge stamped. The third does when the reopening condition excludes a quoted example, and the fourth when a case kills M2. The record discharges when every item has.
