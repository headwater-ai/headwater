---
id: HW-LIB-requirement-verification
status: current
status_since: 2026-11-24
summary: "Why the requirement-verification entry types a requirement, the acceptance criterion that settles it and the verification that applies it, and why each declaration is there."
last_verified: 2026-11-24
title: "The requirement-verification taxonomy"
---

# The requirement-verification taxonomy

Three kinds and the edges between them. A requirement states what must hold before a thing is built. An acceptance criterion states the measure that decides whether it holds. A verification is the thing that applies the measure, usually a test. This repository typed itself with these declarations in its own overlay first. They moved into this entry so that an adopter takes them without a copy of about 200 lines of overlay ([#2017](https://github.com/headwater-ai/headwater/issues/2017)).

## The prior art

**ISO/IEC/IEEE 29148** (Systems and software engineering, Life cycle processes, Requirements engineering) treats a requirement as an identified object. It names four verification methods: test, demonstration, inspection and analysis. The facet values and the `verified_by` relation come from it.

**Volere** (Robertson and Robertson, *Mastering the Requirements Process*) supplies the fit criterion. An [evaluation of this repository](https://github.com/headwater-ai/headwater/blob/main/docs/evaluations/specifying-the-engine.md) quotes it: "a quantification of the requirement written beside it, which is what makes a subjective statement testable". That is prose, so it is a section of the acceptance criterion and not a facet.

**[HW-DR-0073](https://github.com/headwater-ai/headwater/blob/main/docs/decisions/0073-a-verification-is-a-kind-and-its-identity-is-minted-rather-than-found-in-the-code-that-cites-it.md)** rules that a verification is a kind with a minted identity, and not a path in the code that cites it. A rename must not break the link, and one criterion can be proved in several repositories. The same ruling ships the `comment-scan` resolver and keeps the pattern it reads in configuration.

**[Q35](https://github.com/headwater-ai/headwater/blob/main/docs/decisions/0035-q35-whether-one-requirement-kind-holds-an-imported-requirement-and-an-authored-one.md)** rules that `requirement` and `acceptance_criterion` serve the authored population, and that a separate kind carries a transcribed requirement. This entry declares no transcribed kind. That is what lets both kinds require a facet and require sections, because every document of either kind is authored.

## The three kinds

| Kind | Purpose | Shelf | Voice | Identifier |
|---|---|---|---|---|
| `requirement` | `verifiable_requirement` | `verified_requirements`, at `docs/verified-requirements/**` | `requirement_prospective` | `{namespace}-REQ-{seq:04d}` |
| `acceptance_criterion` | `verifiable_requirement` | `acceptance_criteria`, at `docs/acceptance-criteria/**` | `requirement_prospective` | `{namespace}-AC-{seq:04d}` |
| `verification` | `evidence` | `verifications`, at `docs/verifications/**` | `declarative` | `{namespace}-VER-{seq:04d}` |

A requirement requires the sections `Context`, `Requirement` and `Verification`. An acceptance criterion requires `Fit criterion` and `Method`, and the facet `verification_method`. A verification requires `Approach` and `What this does not cover`. Each kind requires `title`, which the base declares in the `name` role.

**Three homogeneous shelves, and not one heterogeneous shelf with a discriminator.** The shelf states the kind, so the front matter stays short, and [the taxonomy model](https://github.com/headwater-ai/headwater/blob/main/docs/spec/taxonomy-model.md) refuses a facet that restates placement. The `standards-spec` entry measured what a heterogeneous shelf costs. A discriminator value that the shelf does not admit stops kind resolution, and no rule then reads the document. The three paths are disjoint from each other and from every shelf of the base and of every admitted entry.

**A verification serves `evidence` and takes the base's `declarative` voice.** It reports what was checked rather than what must hold. `evidence-and-obligation` declares that purpose, and this entry requires that bundle for it.

## The purpose, and why it is new

Each purpose that the base and the admitted entries declare was read first. `rationale` is retrospective and belongs to a choice already made. `behavior` states what the system does now, and a requirement states what a thing must do before it is built. `evidence` is wrong for an acceptance criterion, which is the standard that a measurement is read against and not the measurement. `obligation` is what the corpus owes.

The `brd-prd` entry declares `requirement`, which is the same reader intent scoped to one initiative. Criterion 6 refuses that name here, so this entry declares `verifiable_requirement`. Both kinds serve it.

## The voice regime

The base declares `declarative`, which forbids `future_intent`. A requirement is prospective by construction, so `declarative` on these kinds would report the tradition itself. `requirement_prospective` keeps the other two categories: it forbids `change_narration` and `phased_rollout`. It states what `brd-prd`'s `prospective` regime states, under a name that `brd-prd` does not hold. The [fixture corpus](https://github.com/headwater-ai/headwater/blob/main/docs/taxonomies/requirement-verification/fixtures/README.md) holds a requirement sentence that says a replay "will reuse" a key, and no voice finding names it.

## The language regime is the consumer's

No kind of this entry binds a language regime. A language regime is a house rule of one corpus, such as a sentence limit or a list of retired terms. So a consumer writes `kinds.requirement.language` in its own overlay, and this repository binds its `ste_house` regime on the requirement and the criterion.

**An EARS rule arrives as a regime, and never as a value on another regime.** A kind binds one regime, and `controlled` is one scalar. So a later EARS rule is a separate regime that repeats the house rules, or a new member on `regimes.language`. Either one is a meta-schema change. Nothing in a house regime fights EARS today. A 25-word limit, no semicolon, no contraction and American spelling do not touch `shall`, `When`, `While`, `Where` or `If`.

## The relations

**`verified_by` runs from a requirement to an acceptance criterion or a check rule.** Its family is `evidence`. Q35 holds its `from` end at `requirement`. A consumer that has another verifying kind appends it to `to` with `add_to`, because the consumer's overlay applies after every bundle ([#2022](https://github.com/headwater-ai/headwater/issues/2022)). This repository appends its `probe` kind that way.

**`proven_by` runs from an acceptance criterion to a verification.** It is a distinct relation because the `from` end of `verified_by` is fixed. It copies the shape of `verified_by` clause by clause.

**Both declare `created_by: scaffold` and `reciprocal: required`.** So `headwater new acceptance_criterion --relates verifies=<requirement>` writes the criterion's half, and `headwater new verification --relates proves=<criterion>` writes the verification's half. The far half is owed once the new document leaves `draft`, and `headwater check --fix` writes it then. `relation.reciprocity.missing` is an error, so no half-written pair passes a strict run, whoever wrote the half.

**`cited_in` runs from a verification to a test file.** Its target is the `test_site` anchor kind, which the `comment-scan` resolver reads. The edge binds when a comment in that file cites the verification's identifier. An anchor cannot declare the other half of an edge, so `cited_in` declares no inverse and no reciprocal, as `governs` declares none.

**Four things can verify a requirement, and this entry reaches each one.** An acceptance criterion and a verification are documents. A test is a file, and `cited_in` reaches it through the comment that cites the verification. A check rule is not a document, so `anchors.check_rule` declares it as an anchor. The `check-rule` resolver ships in the binary and binds a string that the engine's own rule list holds.

**`verified_by` reaches no code path.** Reciprocity is silent over an anchor target. So one `reciprocal: required` declaration over two document targets and an anchor would state one contract over two populations. Q35 refused that shape.

## The facet and the section

**`verification_method` is a facet, and the fit criterion is a section.** 29148 supplies a closed method set, and an expectation reads it, so it is a facet. The four values are lower case, as every closed set in this language is. The facet is `volatility: mutable`, because a criterion checked by inspection can later be checked by a test.

**`headwater new acceptance_criterion` refuses to guess the method.** The facet is required, it declares a closed value set, and no role determines a value for it. So the scaffolder writes nothing and names `--facet verification_method=<value>`.

**`section.required.missing` reads a heading and never a word under it.** A criterion that writes both headings and `TODO` under each one passes. The section contract is a rule for a person and for the scaffolder, which writes the headings in this order.

## The participation expectations

**`requirement-verified`** reports a `current` requirement with no `verified_by` edge after 90 days, at `warn`. It names no `to_kind`, so any kind that `verified_by` admits at its target end settles it. [HW-OBL-0039](https://github.com/headwater-ai/headwater/blob/main/docs/obligations/0039-a-participation-expectation-names-one-target-kind.md) is the record of why. The expectation named `acceptance_criterion` until 2026-09-11, and a requirement that only a probe verified was then reported as reaching nothing.

**`acceptance-criterion-proven`** reports a `current` criterion whose method is `test` and that no verification proves after 90 days, at `warn`. It names `to_kind: verification`, because `proven_by` admits one target kind. Its `when` names `verification_method: test`, because HW-DR-0073 leaves the cadence of an inspection or a demonstration open. Neither carries a transport that a verification could observe, so an expectation over every criterion would warn one with no remedy.

## The identifier schemes

`{seq:04d}` and `reconcile-first`, the shape that the decision and the obligation record take. 29148 treats a requirement as an identified object, and the allocation rule of [authoring and lifecycle](https://github.com/headwater-ai/headwater/blob/main/docs/spec/authoring-and-lifecycle.md) never renumbers, so a citation stays stable. `brd-prd` chose `minted-once` with a slug, on the argument that a business document is cited outside the corpus. That argument does not carry here, because a numbered requirement is cited by its number.

**No scheme carries a namespace.** A bundle ships to adopters, and a constant written here would be minted by all of them at once. `taxonomy validate` refuses a resolved scheme with no namespace, so the consumer declares one.

## The traceability matrix

The entry declares one `relation_view` ([HW-DR-0114](https://github.com/headwater-ai/headwater/blob/main/docs/decisions/0114-a-relation-view-follows-an-ordered-chain-of-relations-from-every-document-of-one-shelf.md)). It starts from each requirement and follows `verified_by`, then `proven_by`, then `cited_in`. `headwater generate` writes it to `traceability.md` in the directory of the requirement shelf, and `generate --check` holds it.

A row is one path through the chain. A requirement that nothing verifies is one row with empty cells, and so is a criterion that no verification proves. A verification cell states whether a snapshot in `.headwater/observations.yml` observed it, and whether its criterion changed after that snapshot. `projections` is a list in the base, so the entry appends to it with `add_to`.

## What a consumer writes

The whole overlay that an adopter writes to take the three kinds, with `ACME` as its namespace:

    add:
      identifier_schemes.decision_id.namespace: ACME
      identifier_schemes.obligation_record_id.namespace: ACME
      identifier_schemes.requirement_id.namespace: ACME
      identifier_schemes.acceptance_criterion_id.namespace: ACME
      identifier_schemes.verification_id.namespace: ACME
      anchors.test_site.pattern: "ACME-VER-"

The selection is `bundles: [evidence-and-obligation, decision-record, requirement-verification]`. [Finding 3](#findings) says why it names `decision-record`. `anchors.test_site.pattern` is the literal prefix that a test comment opens with. With no pattern, the engine registers no `comment-scan` resolver and `cited_in` binds nothing.

## The core, declared

Criterion 5 of the [library index](../README.md#admission-criteria) asks each entry to state its relationship to the invariant core. This is a full entry, and it satisfies the core through the base for every clause, as the `brd-prd` and `decision-record` entries do ([HW-DR-0079](https://github.com/headwater-ai/headwater/blob/main/docs/decisions/0079-criterion-5-admits-naming-or-extending-a-core-serving-kind-and-a-facet-only-entry-ships-no-templates-directory-under-criterion-2.md)).

| Core requirement | What satisfies it | Declared by |
|---|---|---|
| `facet_role: state` | `facets.status` | the base, inherited through `governed_document` |
| `facet_role: freshness` | `facets.last_verified` | the base, the same way |
| `facet_role: scent` | `facets.summary` | the base, the same way |
| `purpose: rationale` | `kinds.decision` | the base. This entry declares no rationale-serving kind |
| `purpose: behavior` | `kinds.specification` | the base. This entry declares no behavior-serving kind |
| `relation_family: succession`, lifecycle-sensitive | `relations.supersedes` | the base, untouched |

**This entry declares no facet that carries an engine role.** `verification_method` takes no `role`.

## The rename, and why it is the only route

Criterion 6 asks for an address set disjoint from every admitted entry. Three addresses of this tradition are already held by `brd-prd`: `purposes.requirement`, `regimes.voice.prospective`, and `shelves.requirements` at `docs/requirements/**`. The criterion offers a rename or a declared dependency.

**The dependency is refused.** `requires: [brd-prd]` would bring in the `brd` and `prd` kinds, which a reader of this entry does not ask for. The two shelves also cannot merge. `brd-prd` declares `docs/requirements/**` heterogeneous over `[brd, prd]`, and the shelf here is homogeneous over one kind.

**So this entry renames all three.** The purpose is `verifiable_requirement`, the regime is `requirement_prospective`, and the shelf is `verified_requirements`. The shelf path must also differ, because two shelves at one path resolve a kind by accident when an adopter selects both entries. `docs/verified-requirements/**` is the default. A consumer that selects no `brd-prd` can move the path back with an overlay `override`. This repository does that, so no document moved. `engine/crates/cli/tests/requirement_bundle.rs` resolves a selection of both entries on every run of the workspace suite.

The other route merges this entry with `brd-prd` and rules the tier question first. That needs an owner ruling, and nobody has asked for one.

## What this entry deliberately does not declare

**29148's owner, priority, source and status attributes, and its nine well-formedness characteristics.** Nothing reads them. A kind for each characteristic would repeat the mistake that the design-spec doctrine names.

**A transcribed requirement kind.** Q35 leaves that kind to an adopter who has an upstream.

**A language regime on any kind**, for the reason [above](#the-language-regime-is-the-consumers).

## What this entry assumed

These go to [HW-OBL-0192](https://github.com/headwater-ai/headwater/blob/main/docs/obligations/0192-assumptions-that-a-library-entry-takes-where-the-specification-does-not-speak.md), which collects the assumptions that each entry takes where the specification does not speak.

**That both expectation windows are 90 days.** The number comes from no source. The severity is `warn`, so an adopter who works on a different cadence overrides it.

**That an identifier scheme may carry no namespace.** The base declares none for `decision_id`, and this entry follows it.

**That `volatility: mutable` is right for `verification_method`.** The claim is that a criterion changes its method more often than it changes its identity.

**That a requirement is one per document.** Identity in this language is per document. So a requirement that needs its own identifier needs its own document, and the template says so.

## Findings

These follow the rule that the [library index](../README.md#where-a-finding-goes) states. None of them is tier 1, so none of them holds the entry.

**1. Tier 2. A second entry meets a shelf path that an earlier declaration closed.** [HW-OBL-0189](https://github.com/headwater-ai/headwater/blob/main/docs/obligations/0189-a-base-shelf-path-is-closed-to-every-later-entry-over-the-same-ground.md) records the base form of this, where `docs/specifications/**` was closed to the specification tradition. Here an admitted entry closed `docs/requirements/**`. This entry works around it with a different default path and an override that a consumer writes.

**2. Tier 2. An admitted entry requires a bundle that admission refuses.** This entry requires `evidence-and-obligation`, which the library index does not admit. It is the second case of [HW-OBL-0185](https://github.com/headwater-ai/headwater/blob/main/docs/obligations/0185-whether-an-admitted-library-entry-may-require-a-bundle-that-admission-refuses.md), after `decision-record`.

**3. Tier 2. The bundle this entry requires does not resolve alone.** `evidence-and-obligation` declares `purposes.obligation` and ships no kind that serves it. So a selection of that bundle and this entry is refused for purpose completeness, and an adopter must also select `decision-record` or `design-spec`. The adopter's overlay is still six lines, and the selection is three entries for a reason that the two entries in it do not state. The test above measured the refusal on 2026-11-24.

**4. Tier 3. A requirement here has an identity, and that is a partial answer to [HW-OBL-0195](https://github.com/headwater-ai/headwater/blob/main/docs/obligations/0195-a-requirement-has-no-identity-so-a-descoped-requirement-has-no-form.md).** That record is about the `brd-prd` tier, where a requirement is a numbered line inside a document. Here a requirement is a document, so it has an identifier and a state, and a descoped requirement is a document at `deprecated`. The record stays open, because the `brd-prd` form is unchanged.
