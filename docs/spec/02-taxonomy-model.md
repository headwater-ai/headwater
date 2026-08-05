# 2 — The taxonomy model

**This is the central design of the system.** Everything else is downstream of it.

> **Revision note.** This document incorporates the five structural changes proposed
> by [theoretical foundations](10-theoretical-foundations.md): nuclearity on
> relations, purpose as a first-class declaration with reading precedence derived
> from nuclearity and succession, windowed participation expectations, an immutable
> semantic core, and versioning by measured compatibility. Relation families arrived
> as a dependency of the precedence change. The
> [core-concepts review](../reviews/) subsequently cut the per-relation `dominance`
> declaration — redundant where nuclearity or succession already determined it,
> unused where they did not.

## The problem being solved

In the systems this replaces, the taxonomy exists in at least three places at once:
as prose in a standards document, as constants in each linter, and as path globs in
the AI instruction files. Those three copies drift. Worse, the third copy is
*executable*, so a customer whose documentation culture differs — different shelf
names, an extra document kind, a lifecycle with an extra state — must fork the
tooling to express it, and then loses the ability to take upstream fixes.

The taxonomy must therefore be:

- **declarative** — data, not code;
- **complete** — nothing structural known only to the engine;
- **validated** — the schema itself has a schema;
- **composable** — customisation by overlay, never by fork;
- **versioned** — a change is a release, with a migration path;
- **explainable** — the engine can justify every classification decision.

## Shape

A taxonomy is one logical document, assembled from a base package plus zero or more
overlays, resolving to a single validated object. The illustrative form below is
YAML; the binding decision is deferred (see [open questions](09-open-questions.md)).

```yaml
taxonomy: acme-engineering
version: 3.2.0
extends: docgov/standard@2.1.0        # base package, or null for from-scratch

vocabularies:                          # authoring sugar: named value sets facets reference
  lifecycle_state:
    - {value: draft,      role: initial}
    - {value: current,    role: live}
    - {value: superseded, role: terminal-retained}
    - {value: deprecated, role: terminal-retained}
  audience: [engineer, operator, integrator, auditor]

purposes:                              # reader intents the corpus serves
  rationale:
    intent: explain why a choice was made and what it forecloses
    answers: ["why is it this way", "what was rejected", "what does this constrain"]
  behaviour:
    intent: state what the system does, as it is now
    answers: ["what does this component do", "what may I rely on"]
  procedure:
    intent: enable a reader to carry out a task correctly
    answers: ["how do I do this", "what do I do when X happens"]

facets:
  status:
    role: state                        # engine-significant role
    values: $vocabularies.lifecycle_state
    required: true
  status_since:
    role: state_entered                # stamped on transition; the origin windows are measured from
    type: date
    required: true
  last_verified:
    role: freshness
    type: date
    stale_after_days: 180
  summary:
    role: scent                        # what routing and indexes surface
    type: string
    required: true
  owner:
    type: string
    required: false
  audience:
    values: $vocabularies.audience
    required: false
    severity: warn

regimes:
  voice:
    declarative:
      forbid: [future_intent, change_narration, phased_rollout]
    narrative: {}
  lifecycle:
    standard:
      initial: draft
      transitions:
        draft:      [current, deprecated]
        current:    [superseded, deprecated]
        superseded: []
      retain_terminal: true            # superseded documents are never deleted

relations:
  supersedes:
    family: succession
    from: [decision]
    to:   [decision]
    inverse: superseded_by
    reciprocal: required
    nuclearity: multinuclear           # both ends stand alone
    on_target: {set_state: superseded}
    created_by: scaffold               # who pays for this edge

  derives_from:
    family: derivation
    from: [agent_rule]
    to:   [standard]
    nuclearity: nucleus-satellite
    nucleus: to                        # the standard is the nucleus
    inherits: [status, last_verified]  # the satellite tracks its nucleus
    created_by: generator

  governs:
    family: governance
    from: [standard, specification]
    to:   [code_path]                  # an external anchor kind
    cardinality: many
    created_by: author

  conflicts_with:
    family: association
    from: [decision]
    to:   [decision]
    reciprocal: symmetric
    nuclearity: multinuclear
    invalid_when: {both: {status: current}}   # two live conflicting decisions

shelves:
  decisions:
    path: docs/decisions/**
    homogeneous: true
    kind: decision
    group_by: domain                   # sub-directory is a facet value
  specifications:
    path: docs/specifications/**
    homogeneous: true
    kind: specification
    layout: "{domain}/{component}/{aspect}.md"
  governance:
    path: docs/governance/**
    homogeneous: false
    discriminator: doc_type            # facet naming the kind
    kinds: [standard, methodology, runbook, reference, register, values]

kinds:
  decision:
    purpose: rationale                 # required; a kind without one is invalid
    identifier: {scheme: decision_id}
    voice: declarative
    lifecycle: standard
    facets:
      require: [status, last_verified, domain, summary]
      forbid:  [doc_type]              # placement already states the kind
    sections:
      require: [Context, Decision, Consequences]
      optional: [Alternatives considered, Related]
    relations:
      may: [supersedes, superseded_by, refines, conflicts_with]
      expect:                          # windowed participation: finds what should exist and does not
        - id: decision-realised
          relation: implemented_by
          to_kind: specification
          when: {status: current}
          within: 90d
          since: state_entered         # window origin: the state-entry date facet
          severity: warn
          rationale: a decision nothing implements is either not a decision or not done

identifier_schemes:
  decision_id:
    pattern: "DR-{namespace}-{seq:04d}"
    namespace: repo                    # globally unique when vendored
    allocation: reconcile-first        # never reuse; scan before minting

core:                                  # what overlays may never remove or redefine
  requires:
    - facet_role: state
    - facet_role: freshness
    - facet_role: scent
    - purpose: rationale               # some kind must serve it
    - purpose: behaviour
    - relation_family: succession
      lifecycle_sensitive: true

projections:
  - kind: shelf_index
    for: [decisions, governance]
    output: "{shelf}/README.md"
  - kind: agent_rules
    output: .agent/rules/
  - kind: site_nav
    output: .docgov/nav.yml
```

## The ten declarations

| Declaration | Answers |
|---|---|
| `purposes` | What reader intents the corpus serves |
| `facets` | What metadata documents carry, its shape, and how hard it is enforced |
| `regimes` | Reusable rule bundles: voice and lifecycle |
| `relations` | What typed links exist, their family, endpoints, nuclearity, and reciprocity |
| `shelves` | How the corpus is partitioned, and what each partition means |
| `kinds` | What each species of document is, requires, may link to, and is expected in time to link to |
| `identifier_schemes` | How stable identifiers are shaped, namespaced, and allocated |
| `core` | What an overlay may never remove or redefine |
| `mappings` | How this taxonomy's concepts correspond to another's |
| `projections` | What derived artefacts are generated, and where they land |

An earlier draft counted twelve and then added two more in the next sentence.
Four of those fourteen are gone deliberately, and what each protected survives
without its name: `sequences` folded into windowed relation participation on
kinds (below); `profiles` are publisher-shipped overlays
([spec 7](07-distribution-and-federation.md#profiles-are-publisher-overlays));
`compatibility` named the engine's fixed measurement dimensions, which no
taxonomy could legally vary — a declaration with one legal value is an engine
constant; and `vocabularies` remains as authoring syntax — a named value set
that facets reference — validated as part of `facets`, not a concept anyone
must learn first.

## Purpose is declared, not implied

**Every kind declares the reader intent it serves.** A kind without a purpose fails
schema validation.

This is the genre-theoretic definition made operational: a genre is a socially
recognised type characterised by shared *purpose* and *form*. The rest of a kind
declaration — sections, facets, voice — is form. Without purpose, a kind is a shape
with no reason, and the first question anyone asks about a corpus ("what is this
shelf *for*?") has no answer in the schema.

Purposes are declared once, at the taxonomy level, and referenced by kinds. Several
kinds may serve one purpose — a decision record and an architecture note can both
serve `rationale` — but a kind serving two unrelated purposes is a signal that it
should be split, and the validator says so.

Purpose does real work downstream:

- **Routing** ([spec 5](05-ai-integration.md)) matches a task's intent against
  declared purposes before it matches text. "Why is it like this?" resolves to
  `rationale` kinds; "what does it do?" resolves to `behaviour` kinds. This is a
  search over intentional structure rather than over prose, and it is far cheaper
  and more precise than lexical ranking alone.
- **`docgov explain`** states a document's purpose alongside its kind, so a reader
  who lands on a document knows what it is *for* before reading it.
- **The core** (below) is expressed in terms of purposes, which is what lets an
  adopter rename everything and still be recognisably running the same method.

## Relation families and nuclearity

Two properties every relation type carries — one chosen, one usually inherited.
Together they turn a flat link set into a structure the engine can reason about.

### Family

Every relation belongs to exactly one of six families:

| Family | Meaning | Default nuclearity | Lifecycle-sensitive |
|---|---|---|---|
| `succession` | One artefact replaces or revises another | multinuclear | yes |
| `derivation` | One artefact is generated or distilled from another | nucleus–satellite | yes |
| `governance` | One artefact constrains another, or constrains code | — | yes |
| `evidence` | One artefact substantiates a claim in another | — | no |
| `composition` | One artefact is part of another | nucleus–satellite | yes |
| `association` | Related, with no stronger claim | multinuclear | no |

The fixed family set is deliberate. Open relation vocabularies sprawl, and readers
apply them inconsistently once the list passes about a dozen entries — the
consistent finding from decades of discourse annotation. A family supplies default
semantics, so a new relation type inherits sensible checking without declaring it,
and the validator can flag a taxonomy that has grown five near-synonymous relations
inside one family.

### Nuclearity

A relation is **multinuclear** (both ends stand alone) or **nucleus–satellite** (one
end supports the other and cannot stand without it). The family supplies the
default; a relation type that contradicts its family's default must say so
explicitly, and `taxonomy audit` reports every override, because a family whose
members mostly override it is misassigned.

One thing the family cannot supply is *which end* is the nucleus, because relation
direction is the taxonomy author's choice: a derivation may be spelled
`derives_from` or in the opposite direction, a composition `part_of` or
`comprises`, and the family is the same in both spellings. Nucleus–satellite
relation types therefore always name their nucleus end.

This asymmetry pays for itself in four places:

- **Lifecycle inheritance.** A satellite inherits declared facets from its nucleus.
  A generated rule derived from a superseded standard is stale the moment the
  standard is superseded — detected structurally, with no separate check.
- **Context pruning.** When an agent's context budget binds, satellites are dropped
  before nuclei ([spec 5](05-ai-integration.md)). Dropping the standard and keeping
  its teaser is exactly backwards, and without nuclearity the engine cannot tell.
- **Orphan detection.** An unlinked nucleus is a real finding: something exists that
  nothing points at. An unlinked satellite is a *generation* bug — different
  severity, different owner, different fix. Conflating them produces noise that gets
  suppressed wholesale.
- **Deletion safety.** Removing a nucleus that still has live satellites is a
  finding; removing a satellite is free.

### Reading precedence is derived

Which document a reader should treat as governing when two are linked is not a
declaration — it is entailed by properties already declared, and an earlier
per-relation `dominance` field was cut when the entailment was noticed:

- **On a nucleus–satellite relation, the nucleus governs.** A satellite supports
  the other end and cannot stand without it; a document that cannot stand alone
  cannot have its purpose govern the reading of the document it depends on.
- **On succession, the successor governs.** That is the whole meaning of the
  family: the successor is the one that governs behaviour now.
- **Other multinuclear relations carry no reading order.** `conflicts_with` and
  `is_alternative_to` assert exactly that neither end subordinates the other, and
  imposing an order would misstate the relation.

The engine uses the derived precedence to order routing results, to choose which
document a conflict is reported against, and to decide reading order in generated
indexes — nothing downstream changed when the declaration was removed, which is
the evidence it declared nothing. If a real corpus produces a multinuclear,
non-succession relation whose ends genuinely need an ordering, that outcome is
the case for reintroducing a declaration, and it should be argued then.

### The decision-relation vocabulary

Succession is the temporal axis, and on its own it is not enough. Two decisions can
both be current and contradict each other; one decision can constrain another
without replacing it. Neither is expressible with `supersedes` alone, and neither is
detectable by any reciprocity check.

The default taxonomy therefore adopts the decision-relation set from Kruchten's
ontology of architectural design decisions, each assigned to a family:

| Relation | Family | Claim |
|---|---|---|
| `supersedes` | succession | Replaces a prior decision |
| `overrides` | succession | Displaces a prior decision's effect without retiring it |
| `constrains` | governance | Narrows what the target may decide |
| `forbids` | governance | Rules out an option in the target's space |
| `enables` | governance | Makes the target decidable — the weak form of `constrains` |
| `does_not_comply_with` | governance | Records a known, deliberate violation |
| `subsumes` | composition | Is wider than, and implies, the target |
| `comprises` | composition | Is made up of the target decisions |
| `conflicts_with` | association | Contradicts the target |
| `is_alternative_to` | association | Was a considered alternative |
| `is_bound_to` | association | Must change together with the target |
| `traces_to` | evidence | Derives from a requirement, driver, or source |

Adopting a published vocabulary rather than inventing one is deliberate: this set
has been used and criticised for two decades, and the arguments about where its
edges sit have already been had.

Four checks come with it, none of which succession alone can express:

- two `current` decisions joined by `conflicts_with` — an incoherent corpus state;
- a decision `constrains`-linked to a superseded one — the constraint may be void;
- `forbids` and `enables` edges asserting opposite things about one option;
- a `does_not_comply_with` edge pointing at a `current` standard, which is a
  registered deviation and must carry an owner and an expiry.

A taxonomy may enable any subset. The default enables four: `supersedes`,
`conflicts_with`, `constrains`, and `traces_to` — which keeps the live-conflict
and void-constraint checks, at the cost of the `forbids`/`enables` and
`does_not_comply_with` checks, which belong to the regulated column of the
worked example anyway and arrive with the regulated platform's overlay as
*additions*. An earlier draft enabled all twelve and expected a small team's
overlay to remove most of them. That was backwards twice over: defaults are the
learnability surface for exactly the adopter
([spec 0](00-vision-and-scope.md#who-this-is-for)) with no taxonomist on staff,
and the design's own sprawl warning — readers apply relation sets inconsistently
past about a dozen entries — was aimed at its own default. A first taxonomy
experience that consists of writing `remove:` lines is friction spent deleting
things nobody asked for.

### Who creates each edge

**Every relation declares `created_by`**, from a closed set: `author`, `scaffold`,
`generator`, `hook`, `agent`, `import`. It is required, and a taxonomy that omits it
fails validation.

This exists because of the single most consistent finding in the traceability
literature: trace links decay when creating them costs the author and benefits
someone else later. The field forces the question at design time — *what creates
this edge, and who pays?* — rather than after the corpus has quietly stopped
maintaining it.

It is also measurable after the fact. `docgov taxonomy audit` reports edge counts
and staleness by creator, so a relation declared `created_by: author` that is
present on 4% of eligible documents is visibly not being maintained. The remedy is
usually to move it to `scaffold` or `generator`, not to exhort authors harder.

The default taxonomy assumes that remedy from the start: every relation it
enables is creatable by scaffold, generator, or hook, and `created_by: author`
is reserved for overlay additions an adopter explicitly chooses. The claim that
assisted authoring raises edge capture is the strongest and least-tested in the
system ([spec 10](10-theoretical-foundations.md#what-the-theory-did-not-settle));
a default that only works if the claim holds is a bet, and a default that
survives the claim failing is a design.

### Lineage aligns with PROV

The `derivation` and `succession` families map onto W3C PROV: `derives_from` to
`prov:wasDerivedFrom`, `supersedes` to `prov:wasRevisionOf`, and generated
projections to `prov:wasGeneratedBy`. Alignment is deliberate — provenance is a
solved modelling problem, and matching a standard costs nothing while making the
graph interoperable with tooling that already exists.

It also brings PROV's agent dimension, which now matters: documents are drafted by
humans, by agents, and by both. See
[authoring and lifecycle](03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed).

### Behaviour at the limits

A real corpus finds the edges of this machinery quickly, and every case below is
declared here rather than left for an implementation accident to settle. All of
them are generated Graph checks ([spec 12](12-check-layer.md)) — they come with
the family, not with adopter code.

- **Succession, derivation, and composition are acyclic.** `A supersedes A`, a
  mutual succession, or a longer cycle leaves a corpus with no live end; a
  `derives_from` loop makes satellite inheritance non-terminating; a `comprises`
  cycle is nonsense. All three families reject self-reference and cycles.
  Association may legitimately cycle; evidence and governance edges are not
  ordered, so the question does not arise.
- **Duplicate edges collapse to one, with a finding.** Two identical declarations
  of one relation between the same endpoints are a single edge and an advisory
  finding — usually a merge artefact, never a stronger claim.
- **Disagreeing inverses are a finding, not a choice.** Where both ends author
  their half and the halves disagree — B names a successor that is not the
  document naming B — neither side is preferred. The pair is reported, and the
  corpus is incoherent there until an author resolves it.
- **A satellite with two nuclei inherits nothing contested.** Where the inherited
  facet values agree, inheritance proceeds. Where they disagree — one nucleus
  `current`, the other `superseded` — no value is silently picked; the conflict
  is a finding against the satellite. A satellite that declares a value its
  nucleus also supplies keeps its local value, and the divergence is itself a
  finding: silent shadowing is how inherited staleness disappears.
- **Anchors are endpoints without document semantics.** Nuclearity, reading
  precedence, and lifecycle interaction are defined between documents. A relation
  ending on an external anchor carries none of them — an anchor has no purpose
  and no lifecycle, which is why the family table's nuclearity cells are blank
  for governance and evidence. What an anchor endpoint does carry is **identity**:
  each anchor type is owned by exactly one resolver, anchor strings are
  normalised before comparison so two spellings of one target are one node, and
  an anchor no resolver claims is a finding. Write-time impact detection
  ([spec 5](05-ai-integration.md)) fires on these identities, so anchor
  resolution is a correctness root, not an edge case.

### Disagreement is adjudicated, not ranked

Two documents can both be current, both be well-formed, and disagree on a fact — a
standard says one thing, a specification another. An earlier draft answered this
with a per-kind **authority rank** consumed by an `on_disagreement` rule. It is cut,
for three reasons that survive any rewording:

- **The trigger is unobservable.** Detecting that two documents disagree *on a
  fact* is reasoning about what prose asserts, which
  [spec 1](01-conceptual-model.md#two-layers-terminology-and-assertions) explicitly
  forswears and [spec 4](04-assurance-model.md#declaration-moves-the-boundary)
  confirms is undecidable structurally. The rule could only ever fire *after* a
  human or a coherence-sweep finding had already identified the disagreement.
- **At that point, the adjudication is the data.** Whoever identified the
  disagreement knows which side is right for this case. Spec 4's design rule says
  to record that judgement — as a declared edge, a correction, or a succession —
  not to pre-answer it with a scalar someone chose before the question existed.
- **A scalar cannot express the semantics the prose demanded.** "A specification
  outranks a standard about its own component and is silent about anything else"
  is scoped precedence; `authority: 20` is a global ordering, which is exactly the
  backwards behaviour the old prose warned against.

What survives needs no numbers: where sources conflict and no declared
adjudication exists, an agent cites both and flags the conflict
([spec 5](05-ai-integration.md)). Whether adjudications eventually need their own
declaration is reopened as [Q18](09-open-questions.md#q18--recording-adjudicated-disagreements).

## Contract sidecars: the specification as oracle

Prose is canonical for meaning and hopeless for precision. A component may therefore
carry a **contract sidecar** — machine-verifiable artefacts beside the prose:
schemas, interface descriptions, metric definitions, and structured acceptance
criteria carrying stable identifiers.

```
specifications/ingest/parser/
  functional.md          # prose: what it does and why — canonical for meaning
  technical.md           # prose: how it is realised
  contracts/
    input.schema.json    # canonical for shape
    acceptance.yml       # identified, structured criteria
```

The prose stays canonical for intent; the sidecar is canonical for the exact shape,
and the prose references it rather than restating it. This is decomposition by
**validation regime** — the two halves are checked by different means — not a new
shelf and not a file split for its own sake.

What this unlocks is the more interesting part. Structured acceptance criteria with
stable identifiers are a **test oracle**: a conformance check can be derived from
the specification rather than written alongside it and drifting from it. The check
asks the corpus what should be true, so coverage follows the specification
automatically instead of being maintained in parallel.

That closes the loop the whole system is built around. A specification that can
generate the check that proves it is a specification that cannot quietly stop being
true — the strongest available form of "documentation describes what is". It also
sets a boundary worth stating: a generated check confirms *structure*, never
meaning. A criterion can be present, well-formed, mechanically satisfied, and still
describe the wrong behaviour. Structural conformance is a floor, and the audit layer
([spec 4](04-assurance-model.md)) is the ceiling.

Sidecars are optional. A component with nothing mechanically checkable carries none,
and the taxonomy declares which kinds may have them.

## Participation expectations

A kind may declare that its documents are **expected to participate** in a
relation: a document of this kind, in a given state, should acquire the named
relation to a document of another kind within a window.

```yaml
kinds:
  decision:
    relations:
      expect:
        - id: decision-realised
          relation: implemented_by
          to_kind: specification
          when: {status: current}
          within: 90d
          since: state_entered
          severity: warn
          rationale: a decision nothing implements is either not a decision or not done
```

An earlier draft declared these as a separate top-level concept, `sequences`,
sold as chains — and every declared chain was in fact a single hop: *kind + state
⇒ expected relation, within window*. A chain is three expectations that share
endpoints. A single hop is a state-conditional, windowed, detective-posture
participation constraint — the `required` end of the cardinality spectrum a
relation already has, plus a clock. So it is declared where `may:` already
lives, and the separate concept is gone. What it models is unchanged: genre
theory's *genre system* — proposal → decision → specification → evidence;
incident → postmortem → standard change.

Expectations catch a failure class nothing else does. Every check in
[spec 4](04-assurance-model.md) validates artefacts that exist. An expectation
finds the artefact that **should exist and does not**: the accepted proposal
nobody implemented, the incident with no postmortem, the decision that never
reached a specification. That is the drift people actually complain about, and it
is invisible to link and front-matter validation because there is nothing
malformed to find.

**The window has a declared origin.** `within: 90d` is meaningless until the
question *ninety days from what?* has an answer in the graph, and the earlier
draft had none — no state-entry or creation date existed anywhere, which made the
flagship absence check uncomputable from declared data. So the origin is now
part of the declaration: `since:` names an engine-significant date role —
`state_entered` (stamped by the transition that put the document in the
triggering state) or `created` for expectations with no state condition. An
expectation whose origin facet is not required on the declaring kind fails
`taxonomy validate`. This keeps the check pure ([spec 12](12-check-layer.md)):
origin date plus injected clock, no history walk, no git archaeology.

Four constraints keep expectations honest:

- **Detective only.** An expectation is never blocking. The work may
  legitimately be in flight, deferred, or abandoned for good reason.
- **A window is required.** An expectation with no time bound is a wish. The
  window is what makes the finding actionable.
- **An origin is required.** A window that cannot say where it starts is not a
  window; it is a mood.
- **A rationale is required.** If you cannot say why the participation is
  expected, it is a convention, not an expectation, and it will generate noise.

Expectation findings are reported against the *originating* document, because
that is where the reader who can act will look.

## The immutable core

A taxonomy package declares a **core**: the semantics that overlays may extend but
never remove or redefine.

```yaml
core:
  requires:
    - facet_role: state
    - facet_role: freshness
    - facet_role: scent
    - purpose: rationale
    - purpose: behaviour
    - relation_family: succession
      lifecycle_sensitive: true
```

Without this, "the same taxonomy" means nothing. If a consumer may override or
remove anything, two consumers of one package can end up sharing no structure at
all, and the publisher has no answer to "are they still using the method?"

**The core is semantic, not lexical.** It constrains *roles and purposes*, never
names or paths. An adopter may rename every shelf, relocate every directory, change
every identifier pattern, and replace the lifecycle vocabulary — and still satisfy
the core, provided that after resolution *some* facet carries the state role, *some*
kind serves the `rationale` purpose, and lineage remains expressible and
lifecycle-sensitive.

That is the boundary-object property stated precisely: plastic enough to adapt to
local practice, robust enough to keep a common identity across sites. Local form is
entirely negotiable; shared meaning is not.

Core satisfaction is checked **after** overlay resolution, against the resolved
taxonomy — not by forbidding particular operations. An overlay is rejected when the
*result* fails to satisfy a core requirement, with an error naming which requirement
and which operation removed its last satisfier. Conformance ([spec
7](07-distribution-and-federation.md)) checks the core, not the whole taxonomy.

## Mapping between taxonomies

Two divisions with different taxonomies do not need a merged one. They need declared
correspondences, and that is a solved problem: SKOS mapping relations.

```yaml
mappings:
  - to: platform/docgov-taxonomy@2.0.0
    kinds:
      decision:     {relation: exactMatch,  target: adr}
      specification:{relation: broadMatch,  target: component_spec}
      runbook:      {relation: closeMatch,  target: operational_procedure}
    facet_values:
      status.current: {relation: exactMatch, target: state.active}
```

Four relations, with their standard meanings: `exactMatch` (interchangeable in
practice), `closeMatch` (interchangeable for retrieval, not for inference),
`broadMatch` / `narrowMatch` (one is wider than the other), `relatedMatch`
(associated, neither wider nor equivalent).

Mappings are what let a cross-repository aggregator answer "show me every decision
in the organisation" across taxonomies that share no vocabulary. They are declared
by whoever needs the correspondence — usually the aggregating tier — and are
directional, versioned, and validated: a mapping naming a kind that neither
taxonomy has is a finding.

The engine can also emit the resolved taxonomy as SKOS (`docgov export --format
skos`). That is partly interoperability with knowledge-organization tooling that
already exists, and partly a sanity check: a taxonomy that cannot be expressed in a
standard concept-scheme vocabulary has probably grown something idiosyncratic.

## Kind resolution

Given a document path and its front matter, the engine resolves a kind by:

1. matching the path against shelf patterns — most specific wins, ties are a
   schema-validation error, not a runtime coin-flip;
2. if the shelf is homogeneous, taking its declared kind;
3. if heterogeneous, reading the discriminator facet; a missing or unrecognised
   value is a finding whose severity the shelf declares;
4. applying any path-pattern refinement the shelf declares (for instance
   `functional.md` and `technical.md` resolving to different kinds within one
   component directory).

`docgov explain <path>` prints this derivation — which shelf matched, which rule
fired, the kind's declared purpose, which facets and sections are consequently
required, and which relations are permitted. Classification is never a black box,
for a human or an agent.

### Placement is primary; metadata fills the gap

Directory placement carries the primary classification, because it is the signal a
reader sees first and the one a path glob can act on. Metadata materialises only
what placement *cannot* express.

This produces one rule with real teeth: **a homogeneous shelf forbids the
discriminator facet.** If the directory already says what a document is, restating
it in front matter creates a second truth that will eventually disagree with the
first. The schema enforces the prohibition rather than trusting authors to notice.

## Facet acceptance tests

"Is this a good facet?" is usually settled by taste. Faceted-classification practice
supplies actual tests, and the engine applies them.

| Canon | Test | Where checked |
|---|---|---|
| **Relevance** | The facet is read by at least one check, projection, routing rule, or expectation | schema |
| **Ascertainability** | Every enum value carries guidance stating when it applies | schema |
| **Permanence** | The facet declares `volatility`; a `mutable` facet may not appear in an identifier, a path, or a shelf pattern | schema |
| **Differentiation** | The facet actually partitions the corpus — a value found on nearly every document distinguishes nothing | corpus |
| **Orthogonality** | No two facets are near-perfectly correlated across the corpus | corpus |

The first three are decidable from the schema alone and run under `docgov taxonomy
validate`. The last two require documents to measure against and run under `docgov
taxonomy audit`, which is advisory by construction: a young corpus will fail
differentiation simply for being small.

Orthogonality is the one worth dwelling on. If knowing a document's `shelf` tells
you its `doc_type` with near-certainty, one of them is doing no work — and the
redundant one will eventually disagree with the other. The audit reports correlated
facet pairs rather than rejecting them, because the right fix is a judgement:
sometimes you delete a facet, sometimes you discover the shelf split was wrong.

## Kinds are rigid; states are not

A kind is a property a document cannot lose while remaining the same document — a
specification does not stop being a specification. A lifecycle state is a phase
every document passes through. Formal-ontology practice calls the first **rigid**
and the second **anti-rigid**, and holds that an anti-rigid class may never subsume
a rigid one.

The practical rule: **lifecycle state must never be modelled as a kind, a shelf, or
a directory.** The validator enforces it — a kind whose name collides with a value
in the state vocabulary is rejected, as is a kind named with a bare phase adjective
(`draft`, `pending`, `proposed`, `deprecated`, `legacy`, `temporary`, `obsolete`).

This is the most common taxonomy mistake there is, it always looks reasonable at the
time (`docs/drafts/`, a `deprecated-standard` kind), and it is expensive to undo
because it forces a document to change identity as it matures. Naming the underlying
principle gives the argument a resolution instead of a stand-off.

## Customisation by composition

An adopter never edits a base taxonomy. They declare an overlay:

```yaml
taxonomy: acme-engineering
extends: docgov/standard@2.1.0

override:
  shelves.decisions.path: docs/adr/**            # we call them ADRs
  identifier_schemes.decision_id.pattern: "ADR-{seq:03d}"
  vocabularies.lifecycle_state:                  # our lifecycle, our names
    - {value: draft,   role: initial}
    - {value: active,  role: live}
    - {value: retired, role: terminal-retained}

add:
  shelves.playbooks:
    path: docs/playbooks/**
    homogeneous: true
    kind: playbook
  kinds.playbook: {purpose: procedure, ...}

remove:
  - shelves.proposals            # we do not do time-boxed proposals
  - relations.refines
```

Note what the lifecycle override does *not* break. The names change completely;
the roles survive; the core is satisfied. Had the overlay dropped the
`terminal-retained` role entirely, resolution would fail — not because a key went
missing, but because succession could no longer retain lineage, which the core
requires.

Merge semantics are strict and total:

- **`override`** replaces a value at an addressed path; the path must already exist.
- **`add`** introduces a new key; the key must not already exist.
- **`remove`** deletes a key and everything that depends on it — and the resolver
  **fails** if a surviving declaration still references the removed key. Removing a
  shelf that a projection targets is an error at resolve time, not a mystery later.
- Lists never silently merge. An overlay either replaces a list or uses explicit
  `add_to` / `remove_from` operations.
- Resolution is **order-independent** for disjoint paths and an **error** for
  conflicting ones: two overlays touching the same path is a conflict to resolve,
  not a last-writer-wins race.
- Overlay application must be **confluent**: applying a set of overlays in any
  legal order yields the same resolved taxonomy. This is checked statically, before
  anything is applied.
- The resolved taxonomy must satisfy the `core`. This is checked last, on the
  result.

Confluence is what makes order-independence a guarantee rather than a hope. The
resolver builds the set of paths each overlay addresses and checks pairwise
commutativity: two `add`s at disjoint paths commute; an `override` and a `remove`
on the same subtree do not; two `override`s on one path do not. Any non-commuting
pair is rejected at resolve time, naming both overlays and the contested path.

Delta-oriented software product lines worked this ground thoroughly, and the
requirement is theirs. Without it, a three-tier federation
([spec 7](07-distribution-and-federation.md#federation)) has a resolution order that
someone must remember, which is a bug waiting for the day two tiers are upgraded in
the wrong sequence.

The resolved taxonomy is written to a lock file with a content hash. The engine
checks the corpus against the lock, so a resolution result is reproducible and
reviewable in a diff.

## The meta-schema

The taxonomy language has a formal schema, published with the engine and versioned
with it. `docgov taxonomy validate` checks:

- structural conformance to the meta-schema;
- referential integrity — every referenced vocabulary, regime, kind, facet, and
  purpose exists; no dangling relation endpoints;
- coverage — every shelf resolves to at least one kind; every kind is reachable
  from at least one shelf, or is explicitly marked abstract;
- **purpose completeness** — every kind declares a purpose, and every declared
  purpose is served by at least one kind;
- determinism — no two shelf patterns can match the same path ambiguously;
- role uniqueness — at most one facet claims each engine-significant role;
- lifecycle soundness — the state machine is connected, has an initial state, and
  its terminal states are declared;
- **relation coherence** — every relation names a valid family; nucleus–satellite
  relations name their nucleus; `inherits` names facets that exist on both ends;
  a family's default is not contradicted without explicit override;
- **expectation well-formedness** — every participation expectation names an
  existing relation and reachable kinds, carries a window, a rationale, and an
  origin role that is required on the declaring kind;
- **core satisfiability** — the resolved taxonomy satisfies every core requirement;
- **facet canons** — relevance, ascertainability, and permanence hold for every
  facet (the corpus-measured canons run under `taxonomy audit`);
- **kind rigidity** — no kind collides with a lifecycle-state value or is named with
  a bare phase adjective;
- **edge provenance** — every relation declares a `created_by` from the closed set;
- **context safety** — every agent-facing kind or projection has an applicable
  size budget, and every facet carrying the freshness role has applicable
  staleness policy. The regimes that once wrapped these are gone; the mandates
  are not;
- **overlay confluence** — the overlay set commutes;
- **mapping integrity** — every mapping names kinds and facet values that exist in
  both taxonomies, with a valid SKOS relation;
- projection targets — every projection writes inside the corpus and does not
  collide with an authored path.

A taxonomy that does not validate is never applied. There is no partial-load mode.

## Versioning by measured compatibility

A taxonomy is a released, semantically versioned package — but the version number is
**derived from measured impact**, not chosen from a table of change categories.

The naive model (additive changes are minor, everything else is major) is
straightforwardly wrong for a schema carrying semantics. Adding an *optional* facet
is additive and can still change which documents a projection includes. Widening an
enum is additive and can still cause a completeness check to start failing.
Structural change and semantic consequence are not the same thing, and only one of
them matters to a consumer.

So compatibility is evaluated along five dimensions, against a real corpus. The
dimension set is the engine's, fixed and identical for every taxonomy — an
earlier draft made it a `compatibility` declaration, which every taxonomy would
have stated identically, and a declaration with one legal value declares
nothing:

| Dimension | Question |
|---|---|
| `classification` | Does every existing document still resolve to the same kind? |
| `instance_validity` | Does every existing document still validate? |
| `consequence` | Does every check that passed still pass, and every failing check still fail? |
| `projection` | Does every projection produce identical output? |
| `identifier` | Does every identifier still resolve to the same document? |

`docgov taxonomy diff --to <version>` runs all five and reports per dimension. The
required version bump is a *consequence* of the result: any dimension broken forces
a major version.

The publisher and the consumer play different roles here, and both are necessary:

- The **publisher** measures against its own reference corpora and publishes the
  result as a compatibility claim attached to the release. That is the best it can
  do; it does not have anyone else's documents.
- The **consumer** measures against its own corpus before upgrading. This
  *verifies* the publisher's claim rather than trusting it — and a claim that fails
  locally is exactly the interesting case, because it means the consumer's corpus
  uses something the publisher's reference corpora do not.

A major version ships a **migration payload**: machine-readable steps declaring what
moved, what was renamed, and what must be re-stated, split into what the engine can
apply mechanically (`docgov migrate --apply`) and what needs human or agent judgment
(emitted as a task list with the affected documents attached). Adopting a new major
version without running its migration is a hard failure, not a warning — the lock
file records the taxonomy version and the measured compatibility result each corpus
was validated against.

## Worked example: three taxonomies, one engine

| | Small team | Product suite | Regulated platform |
|---|---|---|---|
| Shelves | `decisions`, `guides` | + `specifications`, `standards`, `proposals`, `evidence` | + `controls`, `audits`, `risk` |
| Purposes | `rationale`, `procedure` | + `behaviour`, `constraint` | + `attestation` |
| Lifecycle | `draft` → `current` | + `superseded`, `deprecated` | + `approved`, with an approver facet |
| Identifiers | none | decision + requirement ids | + control ids, mapped to an external framework |
| Voice regime | unconstrained | declarative on specs and standards | + mandatory normative keyword usage |
| Relations | `supersedes` | + `governs`, `verifies`, `conflicts_with` | + `mitigates`, `attests` |
| Expectations | none | decision → spec; incident → postmortem | + control → audit → attestation |
| Engine changes | none | none | none |

The third column is the real test. If a regulated adopter can express control
mappings, approval states, and attestation relations without touching engine code,
the model is right. If they cannot, the schema is missing a primitive — and the fix
is a new primitive, not a special case.

## Deliberate limits

The taxonomy language is **not** a general programming language. It has no
conditionals, no user-defined functions, and no arbitrary expressions. Rules that
cannot be expressed declaratively are implemented as **check plugins** with a
documented interface (see [engine architecture](06-engine-architecture.md)), and
that boundary is defended: the moment the schema grows an `if`, the drift between
declared and actual structure comes back.

The relation family set is **closed**. An adopter may declare any number of relation
types, but every one must belong to one of the six families. A taxonomy that needs a
seventh family is telling us something about the model, and that conversation should
happen upstream rather than being settled locally by an escape hatch.
