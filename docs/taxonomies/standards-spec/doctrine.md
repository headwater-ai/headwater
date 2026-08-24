# The standards-spec taxonomy

The internal-standard ladder. A standard binds many components. A functional specification states what one component does, and a technical specification states how that component is realized.

This entry declares three kinds, one purpose, one facet, two relations, two shelves, three identifier schemes and three obligations. It requires no other entry. This file states what the tradition is, why each declaration is here, and what this draft assumed where the specification is silent.

## The prior art

**MIL-STD-498**, issued by the United States Department of Defense in December 1994. It merged DOD-STD-2167A, which governed defense system software, with DOD-STD-7935A, which governed automated information systems. Its Data Item Descriptions name the documents directly: the System/Subsystem Specification, the Software Requirements Specification and the Software Design Description. The standard was later withdrawn, and the ladder it wrote down outlived it.

**ISO/IEC/IEEE 29148**, second edition 2018. The family replaced IEEE 830-1998, which is the requirements-specification document most engineers of that period met. It names a ladder rather than one document: business requirements, then stakeholder requirements, then system requirements, then software requirements. Each rung is a specification, each one is regulated by the rung above it, and the standards that bind the whole ladder sit outside it.

**Joel Spolsky, "Painless Functional Specifications", four parts on Joel on Software, October 2000.** This is the source of the two names this entry uses. A functional specification describes the product from the side of the person who uses it. A technical specification describes the internal realization: the data structures, the algorithms and the choices an engineer makes. Spolsky's argument for keeping them apart is that the two have different readers and different rates of change.

**This repository already wrote the same ladder down.** [Spec 2](../../spec/02-taxonomy-model.md#contract-sidecars-the-specification-as-oracle) sketches a contract sidecar over a layout of `specifications/ingest/parser/functional.md` and `technical.md`. It glosses the first as "prose: what it does and why — canonical for meaning". It glosses the second as "prose: how it is realized". That is in-corpus prior art, and it arrived from the sidecar question rather than from any of the three sources above. Four independent arrivals at one shape is the strongest argument this entry has for criterion 1.

## What the tradition converges on

**Three levels, not two.** Every source separates the rule that binds many components from the document about one component. The word for the first is a standard, and it is the word all four use. The 29148 family adds rungs inside the specification level and never merges them into the standard.

**The functional level is canonical for meaning.** All four sources give the reader who asks "what may I rely on" the functional document and never the technical one. When the two disagree, the tradition says the functional one is right and the technical one is stale.

**The technical level exists because the two change at different rates.** A realization changes when the implementation changes. Behavior changes when somebody decides to change it. To hold both in one document makes every implementation change look like a behavior change to every reader of the history.

**Conformance runs upward and traceability runs downward.** A specification answers the requirements of the standards that bind it. A standard asks which specification meets each of its requirements. Both directions are stated in all four sources, and neither one is a link that a corpus derives.

**What the tradition does not converge on is the file layout.** MIL-STD-498 numbers its deliverables, 29148 names them by scope, and Spolsky writes one document per feature. This entry takes the layout spec 2 already sketched, because it is the one the corpus that hosts this library wrote.

## Why three kinds and not two or four

**Not two.** A taxonomy that merges the standard into the specification loses the one edge that matters. A standard binds many components and a specification is about one. To merge them makes `regulates` a self-edge on one kind. The reading precedence of the governance family then says a document governs the reading of itself.

**Not four.** The 29148 ladder has four rungs and this entry declares two specification kinds. A fourth kind would name one standards body rather than the convergence, and criterion 1 refuses a structure invented for the library. An adopter who needs the stakeholder rung and the system rung apart adds two values and two kinds in an overlay of their own. This entry declares the two rungs all four sources share.

**The names are the plain words.** `standard`, `functional_spec` and `technical_spec`. `governing_standard` was considered and refused. Spec 2's rigidity rule refuses a kind named with a bare phase adjective and says nothing against a plain noun. The hedge would be this entry disagreeing with the tradition it models. `srs`, `sdd` and `requirements_spec` were refused for naming one standards body.

**Two kinds serve one purpose, and that is legal.** [Spec 2](../../spec/02-taxonomy-model.md#purpose-is-declared-not-implied) says several kinds may serve one purpose, and it refuses only a kind that serves two unrelated purposes. The precedent inside this library is exact. The design-spec entry gives `review_prompt` and `review_record` the one `evidence` purpose, and it splits them because a relation runs between them. Here `realizes` runs between the two specification kinds, and a relation endpoint names a kind.

## The relations, and who creates each edge

**Neither relation is `created_by: author`**, so this entry owes no author-edge sentence under the rule the [library index](../README.md#one-rule-of-the-base-is-not-a-criterion-here) states. Both proposers are named below anyway, because a reader who has to write an edge by hand should know that nobody meant them to.

### `regulates`, from the standard to the specification

The direction is forced rather than chosen, and it is the sharpest point of this entry. [Spec 2](../../spec/02-taxonomy-model.md#reading-precedence-is-derived) derives reading precedence for the governance family in one clause: "On governance between two documents, the source governs." An edge spelled `conforms_to`, from the specification to the standard, would therefore derive the precedence backwards. The specification would govern the reading of the standard that binds it, and an agent that pruned by precedence would read the wrong document first.

The name avoids two addresses the base holds. `governs` runs from a document to a `code_path`, and `constrains` runs from a decision to a decision. Both declare their endpoints as lists, and an `add` cannot reach into a list that exists, so neither is reachable.

`created_by: agent`, which matches the base's `constrains`. Nothing mechanical knows which standard binds which component. A coherence sweep proposes the edge and a person accepts it.

### `realizes`, from the technical specification to the functional one

The family is `derivation` and the nucleus is the functional specification. This is where the entry earns its keep, and [spec 2](../../spec/02-taxonomy-model.md#relation-families-and-nuclearity) lists four things nuclearity buys. Three of them land here exactly.

- **Lifecycle inheritance.** A technical specification whose functional specification is superseded is stale the moment the succession lands, and the engine sees it structurally with no separate check.
- **Context pruning.** An agent under a context budget drops the technical specification and keeps the functional one. That is the right way round, and the tradition says so.
- **Orphan detection.** An unlinked functional specification is a real orphan, and an unlinked technical specification is a generation defect. The fixture corpus reports the first and stays silent about the second, which is the measured form of this claim.

`composition` was refused. It would say the technical specification is part of the functional one, and it is not. It is a second document about one subject at a second level.

`created_by: scaffold`, on the argument the design-spec entry used for `applied_in`. The run that opens a technical specification from a functional one writes both halves of the edge.

### The third relation this entry does not declare

The tradition's central negative edge is the registered deviation, and this entry cannot supply it. [Finding 1](#findings) states why.

## The core, declared

This is a full entry rather than a partial one. It declares kinds, shelves, a purpose, relations and identifier schemes, so the partial-entry paragraph of criterion 5 does not apply to it. It satisfies the core the way the design-spec entry does, which is **through the base** for every clause except one.

| Core requirement | What satisfies it | Declared by |
|---|---|---|
| `facet_role: state` | `facets.status` | the base, inherited through `governed_document` |
| `facet_role: freshness` | `facets.last_verified` | the base, the same way |
| `facet_role: scent` | `facets.summary` | the base, the same way |
| `purpose: rationale` | `kinds.decision` | **the base.** This entry declares no rationale-serving kind |
| `purpose: behavior` | `kinds.functional_spec` and `kinds.technical_spec` | **this entry** |
| `relation_family: succession`, lifecycle-sensitive | `relations.supersedes` | the base, untouched |

`facets.status_since` is not a clause of the core, and both participation expectations read it, so it is listed here for a reader who checks. The base requires it on `governed_document`, which is what makes the expectations well-formed.

**This entry declares no facet that carries an engine role.** `spec_layer` takes no `role`, for the reason the diataxis entry's `reader_mode` takes none. The role registry is closed at six and none of them is this.

**The purpose map, stated rather than implied.**

| Kind | Purpose | The reader intent |
|---|---|---|
| `standard` | `constraint`, new and declared by this entry | what must hold across everything this governs, and what it rules out |
| `functional_spec` | `behavior`, from the base | what does this component do, and what may I rely on |
| `technical_spec` | `behavior`, from the base | what does this component do, at the level of the realization |

`purposes.constraint` is the one new purpose. [Spec 2](../../spec/02-taxonomy-model.md) already uses the word for it, in the product-suite column of its worked example. No entry and no package declares the address.

**One correction, because the issue that asked for this entry stated the core wrongly.** The framing said the immutable core requires `rationale`, `constraint`, `procedure` and `behavior`. It does not. `core.requires` in the base package names the three facet roles, the purposes `rationale` and `behavior`, and a lifecycle-sensitive succession family. `procedure` is declared only by this repository's own consumer overlay, which is not an entry. `constraint` was declared nowhere at all before this entry.

## What this entry deliberately does not declare

**A rationale-serving kind.** The base's `decision` serves `rationale` and this entry adds nothing there. A kind invented to tick criterion 5 is a shape invented for the library, which criterion 1 refuses.

**A deviation relation.** The address `does_not_comply_with` is reserved package content, and [finding 1](#findings) states the whole of it.

**A normative-keyword voice regime.** The worked example promises one and no declaration in this language holds one. [Finding 4](#findings) states it. All three kinds bind the base's `declarative` regime instead.

**A `standard_class` facet.** A facet that names a standard as internal, industry or regulatory reads well and no check, projection, routing rule or expectation would read it. The relevance canon of [spec 2](../../spec/02-taxonomy-model.md#facet-acceptance-tests) refuses a facet that nothing reads, and `taxonomy validate` enforces it.

**A shelf layout.** Neither shelf declares one. [Finding 8](#findings) records the reason.

**`purposes.procedure` and `purposes.attestation`.** Both appear in spec 2's worked example and neither belongs to this tradition. An entry that models the guides column or the regulated column declares them.

## What this draft assumed

**That the discriminator value of a heterogeneous shelf is the kind name.** It is, and this is a measurement rather than a reading. `engine/crates/census/src/resolve.rs` matches the value against the shelf's `kinds` list by identity and stops when it finds no match. So `facets.spec_layer` declares `functional_spec` and `technical_spec` as its values, and a facet named for a layer carries a value named for a kind. The design-spec entry's `doc_type` has the same shape, and no rule and no document states the constraint anywhere a schema author would find it.

**That `volatility: stable` is right for `spec_layer`.** The claim is that a document does not move from one rung to the other. A document that reads as though it did has been rewritten into a second document. The design-spec entry recorded the same assumption for `doc_type` and nothing has ruled on it. The claim is what permits an adopter to put the value in a shelf path or a projection output.

**That an identifier scheme may carry no namespace.** The base declares none for `decision_id` and states why: a published package that named one would have every adopter mint under it. The decision-record entry followed the base and the design-spec entry declared no scheme at all, so two live patterns exist and no ruling separates them. This entry follows the base.

**That `minted-once` is right for all three schemes.** A standard and a specification are cited from outside the corpus, in a contract, an audit trail and a ticket. An identifier that a later reconciliation moved would break a citation that nobody inside the corpus can rewrite. The `{slug}` in each pattern is what makes this workable, because the name comes from the subject rather than from a counter.

**That the two expectation windows are 180 days and 90 days.** Neither number comes from a source. A standard that regulates nothing after half a year is a standard nobody applied. A functional specification that nothing realizes after a quarter is a requirement nobody built. Both are `severity: warn`, so both are advisory, and an adopter who measures a different cadence overrides them.

## Findings

All eight below go to [13 — Open obligations](../../spec/13-open-obligations.md) as a separate change after this entry merges. That is the rule the [library index](../README.md#where-a-finding-goes) states. Finding 6 was found by running the engine rather than by reading the specification.

**1. Tier 2 — the tradition's registered-deviation check cannot be declared.** [Spec 2](../../spec/02-taxonomy-model.md#the-decision-relation-vocabulary) names one check of four that the decision-relation vocabulary brings:

    a `does_not_comply_with` edge that points at a `current` standard, which is a registered deviation and must carry an owner and an expiry

The relation is defined-but-unenabled package content, reached by `$package.optional.does_not_comply_with`, and `package.optional` holds nothing. So this entry cannot enable it. It does not declare the name either, because the package would collide with the address the day `package.optional` holds the relation. The central negative edge of the tradition is unreachable, and `OB-SS-3` carries the gap.

**2. Tier 2 — the negative edge and the positive edge point opposite ways, and reading precedence cannot hold both.** Spec 2 spells `does_not_comply_with` from the deviating document to the standard, inside the governance family, whose derived precedence makes the source govern. That says the deviating document governs the reading of the standard it violates. This entry spells its positive edge from the standard, which is the only spelling the precedence clause permits. Either the family assignment of the negative edge or the precedence clause is wrong, and no reading makes both right.

**3. Tier 2 — a base shelf's path is closed to every entry that models the same territory.** The layout the tradition uses is spec 2's own contract-sidecar sketch, `specifications/<component>/functional.md`. The base holds `shelves.specifications` at `docs/specifications/**`, and shelf determinism refuses a second pattern over it. So the entry that models the specification tradition cannot use the word the tradition uses, and it declares `docs/component-specs/**` instead. This is the design-spec entry's second finding seen from the shelf side rather than the section side. An opinionated default in a minimal base costs every later entry over the same ground.

**4. Tier 2 — the normative-keyword voice regime has no declarable form.** Spec 2's worked example promises "normative keywords on standards" for the product-suite column and "mandatory normative keyword usage" for the regulated one. A voice regime carries a `forbid` list and nothing else. A requirement that every normative sentence uses one of a fixed keyword set is unexpressible. This entry binds `declarative` on all three kinds and states the gap here. A standard whose requirements avoid every normative keyword passes every rule.

**5. Tier 2 — a standard's requirements are sub-document objects and nothing can name one.** This is the design-spec entry's third finding arriving from a third tradition. A conformance claim wants to say that one technical specification meets requirement 3 of one standard, and identity in this language is per document. `OB-SS-2` carries the gap and names contract sidecars as the place the answer probably lives. Nothing declares how a criterion identifier reaches the graph, and every citation in the fixture corpus is prose that no rule reads.

**6. Tier 2 — an unknown discriminator value removes a document from every check, and no gate reports it.** This one was measured rather than reasoned. The fixture corpus plants `spec_layer: interface_spec` on one document. Kind resolution stops, the census carries the row, and no rule instantiates over the document. A run with that document alone reports 0 findings and `headwater check --strict` exits 0. `engine/crates/check/src/coverage.rs` states the ruling deliberately, so this is not an engine defect. It is a property of every heterogeneous shelf, and the design-spec entry's `spec_series` shelf has carried it since the library opened. A typo in one metadata value is indistinguishable from a clean document at the gate. The [fixture README](fixtures/README.md#planted-defect-4-reports-nothing-and-that-is-the-measurement) holds the run.

**7. Tier 3 — the bundle-set sketch names `standards` and this entry is `standards-spec`.** [Spec 7](../../spec/07-distribution-and-federation.md) lists `standards: {requires: []}` and `compliance: {requires: [standards, evidence]}` in its bundle-set example. Spec 13 says the bundle set waits on a first adopter, and the library index says every admitted entry is a revision of that guess. This entry is that revision for the standards cluster, and it is named for the whole ladder rather than for one shelf. The two must not drift.

**8. Tier 3 — assumptions this draft took.** Neither shelf declares a `layout`, because a layout has no form for a per-component subdirectory and the tradition's files are `<component>/functional.md`. `volatility: stable` on `spec_layer` is the same assumption the design-spec entry recorded for `doc_type`. Whether an entry may declare an identifier scheme with no namespace still has two live patterns in this library and no ruling. The two expectation windows are judgments rather than measurements. The section on assumptions above states each one with its reasoning.
