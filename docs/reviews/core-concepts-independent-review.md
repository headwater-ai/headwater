# Core-concepts independent review

Instrument: [core-concepts-review-prompt.md](core-concepts-review-prompt.md), run 2026-08-05 against revision `b7ca526`. The existing findings were not read until after the independent pass was complete. Claims marked *(inferred)* rest on absent definitions or predicted adoption behaviour rather than an implemented corpus.

## Verdict table

| Concept | Verdict | Reason |
|---|---|---|
| corpus | keep | The Phase A census and no-silent-passes coverage denominator require a bounded file set; federation also needs an adoption unit. |
| shelf | keep | Heterogeneous placement, shelf-scoped comparisons, profiles, indexes, and criticality cannot be represented by kind alone without recreating grouping. |
| kind | keep | Removing it eliminates section contracts, facet applicability, regimes, endpoint constraints, templates, and deterministic classification. |
| document | keep | It is the governed node and unit of parsing, findings, lifecycle, coverage, and authoring. |
| external anchor | define edges | Code-impact routing and `governs` targets require non-document nodes, but identity, duplication, resolution failure, purpose, and lifecycle semantics are incomplete. |
| relation | define edges | Reciprocity and traversal are load-bearing, but cycles, self-links, conflicting inverses, duplicate edges, multiple nuclei, and anchor endpoints are underdefined. |
| sequence expectation | merge into relation participation | It is a delayed, state-conditional required relation; its unique absence finding survives the merge, but its time origin is missing. |
| facet | define edges | Scalar, date, and enum metadata are necessary; reference-valued facets duplicate relations while bypassing relation validation. |
| voice regime | keep | Reusable lexical checks and exemption-concentration reporting are concrete outcomes. |
| lifecycle regime | define edges | The state machine is necessary, but transition checks cannot run from the current graph because prior state is not a declared check input. |
| freshness regime | merge policy into the freshness facet | Keep the engine-significant role, threshold, drift weighting, posture, reporting, and auditability; remove only a redundant author-facing wrapper. |
| size regime | merge policy into agent-facing kinds and projections | Keep mandatory budgets, enforcement, findings, and trend reporting where context is produced; remove only a detached wrapper. |
| obligation | keep | It records why a rule exists and explicitly accounts for gaps and unverifiable commitments. |
| control | keep | Non-engine mechanisms, trigger, posture, handoff, and promotion cannot live on a check alone. |
| register | merge authored bindings into obligations and controls; keep a generated register | The register remains the mandatory inspectable coverage and degradation view, but need not be a third independently authored object. |
| projection | keep | Declared outputs plus regeneration comparison provide a concrete drift check unavailable elsewhere. |

## Arguments, ordered by consequence

### 1. Temporal checks are not evaluable from the declared inputs

`03-authoring-and-lifecycle.md` says illegal lifecycle transitions are rejected. `02-taxonomy-model.md` says sequences fire after a window. But `12-check-layer.md` gives checks only the current scoped graph, body access, and an injected clock.

A current `status` value cannot prove whether the transition into it was legal. Likewise, `within: 90d` is meaningless without a declared start event and timestamp. Document creation time is not in the graph, state-entry time is not a facet, and git history is neither a declared input nor reliable after vendoring or squash merges.

This makes two advertised checks impossible while preserving the appearance of deterministic, cache-safe evaluation. The checks need an explicit historical input, or their guarantees must be reduced. (Confidence: high; *inferred from absence* in specs 2, 3, and 12.)

### 2. Authority rank should be removed

`02-taxonomy-model.md` assigns scalar authority to kinds while also saying authority is scoped: a specification outranks a standard only within its component. No scope operand appears in the declaration.

More fundamentally, `01-conceptual-model.md` says headwater does not model claims inside prose, and `04-assurance-model.md` says semantic disagreement requires judgement. `on_disagreement` therefore cannot fire until a human or coherence sweep has already identified the disagreement. At that point a global kind rank is a poorer representation than the adjudication itself.

Removing authority rank loses no currently executable check. Conflict reporting and the instruction to cite both sources survive. This settled part of spec 2 should be reopened. (Confidence: high.)

### 3. Sequence expectation is a relation constraint with an undefined clock

Every sequence in `02-taxonomy-model.md` is an origin kind/state, an expected relation, a target kind, a window, and detective posture. That is delayed relation participation rather than a separate graph primitive. Folding it into the relation or kind-relation contract preserves the unique result: reporting that a required target remains absent after a deadline.

The merge also exposes the missing timestamp instead of hiding it behind genre-system terminology. (Confidence: high on structural equivalence; medium on the best declaration location.)

### 4. Relation semantics should shrink, but family cannot infer everything

Family, nuclearity, dominance, lifecycle sensitivity, and authority create an excessive relation-design surface. Family alone, however, does not identify an endpoint: derivation may be named `derives_from` or expressed in the opposite direction; composition may be `part_of` or `comprises`. Without canonical direction rules, family cannot determine which endpoint is nucleus or dominant.

Cut authority. Derive dominance from explicit succession semantics and from the nucleus for nucleus-satellite relations. Use no dominance for remaining multinuclear relations until a routing or projection outcome demonstrates a need. Keep family as grouping and audited defaults rather than the sole semantic source. (Confidence: medium-high; the counterexample follows from taxonomy-defined relation direction.)

### 5. Reference-valued facets are a second, ungoverned edge mechanism

`01-conceptual-model.md` permits facets that reference nodes. Such a value creates the same graph fact as a relation but has no family, endpoint rules, reciprocity, lifecycle interaction, provenance, or relation cardinality. Under deadline pressure the cheaper syntax will win. Reference values should compile into a declared relation or be limited to non-graph identifiers. (Confidence: high.)

### 6. The declaration surface is fourteen, not twelve

`02-taxonomy-model.md` lists twelve declarations and then adds projections and profiles. At least four are not orthogonal:

- `sequences` are delayed relation participation;
- `profiles` are named publisher overlays;
- `compatibility` is an engine-owned set of five measurements unless taxonomies may actually vary it;
- `vocabularies` are reusable enum definitions referenced by facets, useful syntax but not a separate domain concept.

The same simplification applies inside regimes and assurance without deleting their protections:

- freshness policy remains mandatory and engine-significant on the freshness facet;
- size budgets remain mandatory on every agent-facing kind or projection, with failures and trends visible to the assurance layer;
- obligation-control bindings and dispositions remain authored, while the register remains a generated, checked projection that makes uncovered or degrading controls impossible to hide.

This is relocation of policy to its enforcement point, not removal or optional nesting. A taxonomy may still reuse named policy blocks as schema syntax. The engine must still reject an agent-facing projection without an applicable size budget and a freshness-bearing kind without applicable freshness policy. (Confidence: high for sequence, profile, compatibility, and the preservation rule; medium for vocabulary.)

### 7. Relation limits will become accidental implementation policy

The following are not settled in specs 1, 2, or 12:

- `A supersedes A`, mutual succession, and longer succession cycles;
- derivation and composition cycles;
- duplicate identical relations;
- both inverse fields being authored but disagreeing;
- a satellite with two nuclei providing contradictory inherited facets;
- a satellite declaring a value different from an inherited value;
- nuclearity or dominance when an endpoint is an external anchor;
- a missing anchor resolver, two resolvers claiming one anchor, or two anchor strings normalising to one identity.

Succession, derivation, and composition need explicit cycle and self-link rules. Inheritance needs conflict and precedence behaviour. Anchor relations need semantics that do not pretend anchors have document purpose or lifecycle. (Confidence: high that these cases are undefined.)

### 8. Partial migration contradicts the all-or-nothing pipeline

`02-taxonomy-model.md` says an invalid taxonomy is never applied and there is no partial-load mode. `07-distribution-and-federation.md` allows migrations containing human-judgement tasks, during which the corpus necessarily violates either the old or new taxonomy.

An ordinary major upgrade can therefore create a period where no valid resolved state represents the repository. Q12 recognises migration as open, but the settled resolver language assumes atomicity. Q12 is mis-framed if treated only as migration UX: it constrains the core validity model. (Confidence: high.)

### 9. Gaming concentrates at classification and delayed enforcement boundaries

- On heterogeneous shelves, selecting a cheaper kind avoids its sections, facets, voice, relations, and sequences.
- Parking a document in `draft` avoids live-state obligations and state-triggered expectations.
- Choosing `cites` instead of a lifecycle-sensitive relation can avoid stronger checks.
- Suppressions may expire while waivers must expire, making the easier individual escape less controlled.
- A vague or keyword-stuffed summary can manipulate routing while aggregate probes miss the individual defect.

The design cannot mechanically determine prose genre without crossing its stated semantic boundary. Same-shelf kind contracts therefore should not create large compliance arbitrage, and every exception must be countable and expiring. (Confidence: high on the escape routes; medium on frequency.)

### 10. Maintenance machinery is a required part of the adoption model

The specification correctly observes that author-maintained links decay, but treats scaffolding, hooks, and agent assistance chiefly as control mechanisms. For this model they are also the supply chain for the graph.

A viable release needs specialised, taxonomy-derived authoring machinery:

- an information-architecture skill that resolves kind and shelf, explains the applicable purpose and contracts, and refuses semantic classification it cannot justify;
- authoring and review skills that propose required relations, evidence, lifecycle changes, and summaries from the changed artefacts while retaining human acceptance;
- harness hooks at intent, write, and review time that invoke those skills or the equivalent deterministic scaffolding;
- telemetry that attributes every required datum and edge to scaffold, hook, agent, import, or author, then treats declining assisted coverage as an assurance finding;
- generated, bounded agent context whose freshness and size policy is checked before loading, so stale or oversized governance instructions cannot silently poison the agent.

This is not a proposal for new domain concepts or checks. It hardens the already specified `created_by`, control, projection, hook, routing, freshness, size, and assisted-fraction machinery into an adoption requirement. A corpus may be valid without these authoring aids, but the published method should not claim robust adoption without them. (Confidence: high on the architectural dependency; medium-high on the adoption prediction.)

### 11. Scale is dominated by maintained edges and mapping topology

At ten documents manual relation capture is plausible. At 500, missing-edge density and review noise determine whether traversal is useful. At 10,000, census walking, corpus checks, graph projection, and identifier reconciliation require measured incremental behaviour.

Fifty independently paired taxonomies permit 1,225 unordered mapping pairs, then multiply by versions and direction. The aggregator-owning-correspondences language in specs 2 and 7 should become the normative topology at scale. The largest non-linearity is coordination: every author-maintained edge and cross-taxonomy mapping becomes a stale assertion. (Confidence: high on growth shape; medium on the failure threshold.)

### 12. The correctness roots extend beyond kind resolution and projection

Specs 6 and 12 place silent trust in taxonomy and overlay resolution, lock production, the corpus census, scope enforcement, parsing and span retention, external-anchor resolvers, and scaffolding. A defect in any one can produce systematically green or misdirected downstream results. Check fixtures alone do not test those roots. (Confidence: high.)

## Forced 30% cut

Ranked in removal order:

1. Authority rank: no executable disagreement capability is lost.
2. Register as a separately authored concept: the mandatory generated register and its degradation visibility remain.
3. Freshness regime wrapper: freshness policy and enforcement remain on the engine-significant facet.
4. Size regime wrapper: mandatory budgets and enforcement remain on agent-facing outputs.
5. Sequence as a separate concept: absence detection remains as delayed relation participation.
6. Profile as a separate declaration: named overlays retain the capability.
7. Compatibility declaration: the five engine measurements remain.
8. Standalone vocabulary as a domain concept: reusable enum syntax remains.
9. Contract sidecars: defer; this loses the severable specification-as-test-oracle capability.
10. Cross-taxonomy mappings: defer until federation has an aggregator; this loses cross-taxonomy retrieval until then.

The first eight remove vocabulary or wrappers, not safeguards. Sidecars and mappings are the first cuts that lose substantive capabilities.

## Learnability gradient

- Filing with scaffolding: kind, required metadata, and required sections--three concepts. Relations become a fourth when machinery cannot derive them.
- Extending a taxonomy: currently about twelve concepts. The collapses reduce this to roughly eight, provided the IA skill and `explain` surface make inherited policy visible.
- Authoring from scratch: the full declaration set plus resolution, compatibility, projections, identifiers, and assurance. This is expert work. Mappings, sidecars, control registration, and advanced relation semantics should not enter the novice path.

The three-concept filing claim depends on scaffolding and specialised authoring skills shipping with the first usable release.

## Closing

1. **The single change that most reduces concept count without losing a capability:** collapse the separately authored register, freshness and size regime wrappers, profiles, compatibility declaration, and sequences into the declarations that already determine their behaviour, while preserving a mandatory generated register and mandatory freshness and context-budget enforcement. This removes six first-class concepts without removing their checks, visibility, or protections.

2. **The single thing most likely to break first in real adoption:** the corpus graph becomes too sparse or semantically weak because authors omit relations or choose cheaper kinds and relations. Routing, lifecycle propagation, impact detection, sequence checking, and projections then degrade together. Specialised IA and authoring skills plus enforced harness hooks are required countermeasures, and declining assisted coverage should be treated as evidence the adoption model is failing. (Confidence: medium-high; prediction, therefore *inferred*.)
