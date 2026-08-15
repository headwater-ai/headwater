---
id: HW-OBL-0115
status: current
status_since: 2026-08-14
summary: A consumer that vendors a package for the first time takes the publisher's identity on the channel that carried the digest, because nothing signs a release and no key reaches an adopter.
last_verified: 2026-08-14
title: "A pinned digest authenticates the pin and never the publisher"
provenance:
  warrant: proposed
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-distribution-and-federation
    - HW-DR-0022
---

# A pinned digest authenticates the pin and never the publisher

## Context

`headwater taxonomy vendor` checks a fetched artifact against a digest that the consumer wrote into `.headwater/taxonomy.yml`. [Q22](../decisions/0022-q22-the-integrity-posture-of-a-published-package.md) states what that buys. An artifact that any party changed between the pin and the use is refused, and the refusal names each file that moved.

**The first fetch is outside the check, and it is the fetch that produced the pin.** A consumer who is binding a package for the first time holds no earlier digest to compare against. What they hold is a number that arrived from the publisher's release notes or from a registry entry. So the identity of the publisher rests on that channel, and this engine states nothing about it.

**Nothing here signs a release.** A signature would close the gap, and it needs three things that no decision of this repository has settled. It needs a key pair that a publisher holds. It needs a channel that distributes the public half to an adopter who has never met the publisher. And it needs a rule for what a consumer does when a key is revoked after a pin was written.

[Spec 7](../spec/07-distribution-and-federation.md#publishing) names an integrity digest and names no signature, so the specification does not yet ask for one. The gap is recorded here rather than filled with a field. A manifest key named `signature` that held a digest would read as an answer to anybody who did not open the code.

## Obligation

This corpus owes an answer to one question. **What does a consumer rely on for the first fetch of a package from a publisher they have not met?**

Three answers would discharge it, and each is a different amount of work.

1. **The channel is enough, stated as a requirement on the publisher.** Spec 7 would require a publisher to carry the digest of each release on a channel that a consumer already trusts. It would also require the artifact and the digest to travel apart. This is the cheapest answer, and it moves the obligation onto the publisher rather than removing it.
2. **A signature, argued as the dependency it costs.** A signing scheme, a key distribution route, and a revocation rule, each recorded. The engine gains a verifier, and [Q22](../decisions/0022-q22-the-integrity-posture-of-a-published-package.md)'s reasoning about third-party crates is reopened for a component that nobody should write.
3. **A trust root that an organization already runs.** Many adopters distribute packages through an artifact store that authenticates the publisher on their behalf. Where that holds, the engine adds nothing and the specification says which property it is relying on.

The answer is a ruling rather than an edit, because the first two make opposite claims about what the engine owes an adopter.

## Discharge

Nothing discharges this yet. The measurement below states what was true on 2026-08-14, so that no later reader mistakes the check that exists for the one that does not.

**What runs.** `headwater_resolve::release::verify` reads the release record, compares the release digest with the pin, and then compares the bytes of every member with the record. `engine/crates/resolve/tests/publish.rs` provokes each refusal. The case named `a_consistent_forgery_is_refused_by_the_pin` shows what the pin adds over the record. An artifact republished whole passes every internal check, and the pin refuses it because the consumer wrote a digest down first.

**What no test can show.** No fixture in this repository can fail on the first fetch, because nothing in the design decides what the first fetch relies on. A test written now would assert a rule that no document states. That is the same posture `requires_engine` held until it earned a reader, and it is the reason this record exists instead of a check.

**Who this reaches.** No adopter has bound a package from a publisher outside their own repository. Until one does, the question is answerable from the design rather than from a report. The cheapest of the three answers above may well be right.

**A second verb met the same gap and took the first answer for its own case.** `headwater import` checks a committed snapshot against a pin. A snapshot has the property a package has: the record travels inside the artifact it describes. So an imported edge inherits the authority of whatever carried the digest, rather than the authority of the digest. The verb refuses an import whose declaration names no `channel`, which puts the requirement on the consumer rather than on the publisher. That is a narrower move than answer 1 above, and it does not settle this record. It states who a reader should ask, and it verifies nothing about the answer.
