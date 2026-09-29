---
id: HW-DR-0100
status: current
status_since: 2026-09-29
summary: "Under the counted grain, each tombstone of a filtered export lists the SHA-256 digest of each identifier it withheld. A tier that pins the export then binds an anchor to a withheld document as withheld, and it reports a typo as unresolved. Under sealed the export lists nothing, and both stay unresolved. An export older than version 1.3 lists nothing either."
last_verified: 2026-09-29
title: "A counted tombstone lists a digest of each withheld identifier, and a sealed one lists nothing"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0017
---

# A counted tombstone lists a digest of each withheld identifier, and a sealed one lists nothing

## Context

[HW-DR-0017](0017-governed-access-and-the-solution-layer.md#consequences) makes withheld a third outcome of an anchor, beside resolved and unresolved. It gives the reason. Without the distinction, every filtered harvest produces a wall of unresolved findings. Operators then learn to ignore the class that also carries real defects. [Spec 7](../spec/07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it) says that an anchor into a document that the publisher withheld resolves to `withheld`.

Before this decision, nothing produced that outcome. The resolver over a pinned export matched an anchor against the identifiers in the export, and the export held no withheld identifier. So an anchor into a withheld document and a typo gave the same miss, and both were reported as unresolved ([#1309](https://github.com/headwater-ai/headwater/issues/1309)).

The export could not help. Under `counted`, each tombstone stated a rule and a count of documents. Under `sealed`, the export stated only that it is filtered. Neither form lets a reader tell one withheld identifier from an identifier that never existed.

Three statements pull against each other here:

- HW-DR-0017 and [spec 1](../spec/01-conceptual-model.md#external-anchor) say that withheld is never reported as unresolved.
- [Spec 6](../spec/06-engine-architecture.md#an-export-profile-carries-a-filter) says that under `counted` the existence of the item is not the secret, and under `sealed` it is. It also says that the rule identifier is all that a reader of a tombstone gets.
- A filtered export must not carry the identifier or the path of a document it withheld. The fixture `a_filtered_profile_withholds_a_document_and_its_edges` in `engine/crates/generate/tests/fixtures.rs` holds that.

## Decision

**Under `counted`, each tombstone lists a digest of each identifier it withheld.** The new member is `identifiers`, inside each entry of `tombstones`. It holds the sorted, deduplicated list of `sha256:<hex>` digests, one for each withheld document that has an identifier. The digest is over the bytes of the identifier, with the same function that digests every other artifact of this engine. `documents` still counts every withheld document. A document with no identifier is counted and not listed. An anchor names a document by its identifier, so no anchor can name that document. The list is written even when it is empty, so that a reader can tell a current export from an older one.

**The resolver over a pinned export reads the list.** When an anchor misses the list of documents, the resolver digests the anchor and looks for the digest in the tombstones. A hit binds the anchor as withheld, under the name of the export's profile. A miss stays unresolved, and the reason says that the string is not one of the identifiers that the export withheld.

**Under `sealed`, the export lists nothing, and a miss stays unresolved.** The reason names the `sealed` grain. The publisher chose that the existence of a document is the secret, so this tier cannot tell a withheld document from a typo. This is the one case where the rule "withheld is never reported as unresolved" does not hold. The reason is the error asymmetry that HW-DR-0017 applies to a withholding. A typo reported as withheld is silent, and nothing shows it again. A withheld anchor reported as unresolved is visible, and a person can clear it. So the tier takes the visible error.

**Under an export older than version 1.3, a miss stays unresolved.** No tombstone of such an export carries `identifiers`. The reason says that the export predates the list.

**The export version moves from 1.2 to 1.3.** The change adds a member and removes none, so it is a minor version.

**What this discloses, stated plainly.** A reader who holds an identifier can learn that the identifier exists and that the filter withheld it. The digest is not salted, so a reader who can guess an identifier can test the guess. An identifier from a pattern with a short slug is easy to guess. A reader who holds nothing learns nothing more than the count. This is consistent with the `counted` grain, where existence is not the secret. It amends one sentence of spec 6, which says that the rule identifier is all that a reader gets. Under `counted`, the reader also gets a membership test for an identifier that they already hold. Where the existence of a document is the secret, the publisher declares `sealed`. HW-DR-0017's instruction also stands: a fact whose existence is the secret does not belong in a corpus that is exported at all.

**The list is a membership test, and not the placeholder that spec 6 describes.** Spec 6 says that under `counted` a placeholder sits where each withheld node would have been. The export writes one aggregate entry for each rule, with no position. This decision does not close that gap.

**What reopens this decision.** This is a product choice, and it is reversible. Reopen the `sealed` ruling when an adopter who harvests a `sealed` export reports that its unresolved findings hide real defects. Reopen the `counted` ruling when a publisher reports that the digest list disclosed an identifier that the `counted` grain was meant to keep.

## Consequences

`Binding::Withheld` has its first producer, the resolver in `engine/crates/import/src/harvest.rs`. The edges module, the scope module and the target check already handle the binding. So a withheld anchor raises no finding, and the graph summary counts it in the withheld class.

The test `a_withheld_identifier_binds_withheld_and_a_typo_stays_unresolved` in `engine/crates/cli/tests/harvest.rs` holds the `counted` ruling on an export that the binary wrote. The test `under_a_sealed_grain_a_withheld_identifier_and_a_typo_both_stay_unresolved` beside it holds the `sealed` ruling.

A reader of a `counted` export who reads version 1.2 sees one more member in each tombstone. A reader that rejects an unknown member refuses it. The native export has no schema that forbids a new member, so no reader in this repository does that.

[HW-OBL-0098](../obligations/0098-whether-a-withheld-anchor-needs-a-class-beside-its-count.md) asks whether a withheld anchor needs a class beside its count. This decision produces the withheld outcome for the first time. No adopter has read the count yet, so the obligation stays open.
