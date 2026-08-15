---
id: HW-DR-0025
status: draft
status_since: 2026-08-15
summary: The namespace opens every identifier, and a published source declares no namespace at all, because any constant it wrote would be minted by every adopter of it at once.
last_verified: 2026-08-15
title: "Q25 — Where the namespace goes in an identifier, and who declares it"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - packages/headwater-standard/taxonomy.yml
    - docs/taxonomies/decision-record/bundle.yml
    - engine/crates/meta/meta-schema.yml
    - engine/crates/resolve/src/rules.rs
---

# Q25 — Where the namespace goes in an identifier, and who declares it

## Context

Nine identifier schemes serve this corpus. Seven are declared in this repository's own overlay and carried `namespace: HW`. Two are declared in shipped sources, and both carried `namespace: repo`. They are `decision_id` in `packages/headwater-standard/taxonomy.yml` and `obligation_record_id` in `docs/taxonomies/decision-record/bundle.yml`.

**The split fell exactly along what is published, and the published half held the one value a namespace cannot be.** `repo` is a word that every corpus adopting the package would also write. Two adopters who take the base unchanged therefore both mint decision number one under the namespace `repo`, and the two strings are the same string. `OB-ID-1` and [departure 7](../spec/08-design-departures.md#7-identifier-namespacing-arrives-late) both rest on the namespace being globally unique, and the shipped default guaranteed a collision. [Spec 2](../spec/02-taxonomy-model.md#the-thirteen-declarations) annotated the value as globally unique when vendored, which is the one thing a constant in a published package cannot be.

**A package cannot repair this by naming a better constant.** Any value it writes is minted by every adopter at once. Its own namespace is inherited by all of them, and a stand-in names nobody. So the question is not which value ships. It is whether a published source declares one at all.

**The rendered order was open at the same time, and it cannot stay open.** Every scheme wrote the type first, as `DR-{namespace}-{seq:04d}` and `SPEC-{namespace}-{slug}`. A rendered identifier travels into a commit message, a ticket and an agent prompt, so the order is fixed the moment anything cites one. This repository is private, has never been vendored, and carries about 1,100 identifier occurrences. That is the only window in which the order can be settled at all.

## Decision

**The namespace goes first. Every scheme renders `{namespace}-<TYPE>-<local part>`.**

Every system with a real namespace renders it outermost: Jira `PROJ-123`, GitHub `owner/repo#n`, a CURIE, a Java package, a Kubernetes API group. The type-first traditions usually cited against this carry no namespace at all. RFC 2119, `CVE-2021-44228` and `ADR-0001` name a series and never an owner, so none of them is evidence about where an owner goes.

Three things follow that the type-first order does not give. `^HW-` is an anchored match for everything this corpus owns. A lexical sort groups foreign identifiers by their owner rather than by their type. And the rendered form mirrors the IRI that a projection derives, where the namespace is outermost already.

The cost is real and small. In a single-corpus corpus the namespace is a constant on every occurrence, so a reader skips three characters before the discriminating token. That cost is learned once and the gain is permanent.

**A published source declares no namespace. The corpus that adopts it declares one in its overlay.**

This is forced rather than chosen, for the reason the context gives: a package has no value it can honestly write. So the base package and the decision-record bundle declare a pattern and an allocation and nothing else, and `.headwater/overlay.yml` declares `HW` for all nine schemes.

**The requirement moves from the source to the result.** `engine/crates/meta/meta-schema.yml` marked `namespace` required on an `identifier_scheme`, and one meta-schema serves the taxonomy root and the overlay root, so a package had no way to omit it. It is optional in a source now, and `taxonomy validate` refuses a resolved scheme that carries none.

**An absent namespace and a namespace declared as nothing are two findings.** A key that is absent is a package leaving the choice to its adopter. A key written as the empty string is an adopter answering with nobody. Each one gets its own message and its own address. The second resolved clean before this record, and the identifier check downstream then skipped every document under the scheme, for a template that it could not read.

**The prefix comparison expands the namespace before it compares.** `identifier integrity` refuses two schemes in one namespace when neither literal prefix rules the other out, and it took the pattern text before the first `{`. A namespace-first pattern opens with `{namespace}`, so every scheme yielded the empty string and all thirty-six pairs of nine were refused. `{namespace}-DR-{seq:04d}` and `{namespace}-SPEC-{slug}` admit no string in common, so those refusals were wrong on the rule's own terms. To expand the declared constant costs no soundness, because the result is still text that every admitted identifier begins with, and it is longer text.

## Consequences

**The rendered form is what a later grammar has to preserve.** [HW-OBL-0046](../obligations/0046-the-identifier-scheme-grammar.md) asks for the prefix, the namespace and the local part as separate declared fields, which makes disjointness decidable by construction. That change reaches `Template::parse`, minting, the identifier check, the IRI projection, and prose in spec 2 and spec 3. It lands after this record and has only to preserve this rendering, so no identifier is rewritten twice.

**The base package is at 2.0.0 and the change earns the major twice.** A consumer that declares no namespace resolves to a refusal until it declares one. An identifier minted under 1.0.0 no longer matches its pattern.

**The interview gains the one question that no package can answer.** [Spec 7](../spec/07-distribution-and-federation.md#the-interview) deleted "do you want identifiers" because an existing ruling answers it, and nothing replaced it. `headwater init` asks the adopter what the namespace is, and it is the one answer that selects no bundle.

**Git history is not rewritten, and the type-first form survives there.** Fifty-six commit messages and thirty-nine pull-request bodies cite an identifier that this record renames. Thirty-four issue bodies carry the new form, because a ticket is a live reference that a reader follows today. A commit message keeps what it said. A rewrite of it force-pushes the default branch, orphans every branch and proposal in flight, and invalidates the commits that this corpus cites by hash. So the type-first form reads in `git log` and nowhere in the working tree, and a reader who meets one maps it by moving the namespace to the front.

**Three claims in the corpus are corrected here.** [Spec 2](../spec/02-taxonomy-model.md#the-immutable-core) said that the invariant core requires a namespace on every scheme. It cannot: `core.requires` has an identifier form, and that form names a scheme an overlay may not remove, so no core entry can ask for a property of a scheme. `identifier integrity` is what requires it, and the refusal message that cited the core is gone. [Spec 7](../spec/07-distribution-and-federation.md#the-invariant-core) carried the same claim in two places, and [HW-OBL-0046](../obligations/0046-the-identifier-scheme-grammar.md) carried it in its Context.

**This record lands on an unchecked shelf.** [#205](https://github.com/headwater-ai/headwater/issues/205) reports that the house language regime binds four kinds and that `decision` is not one of them, so no lexical rule reads this prose. It is written to the same standard as a specification part, and nothing measures that claim.
