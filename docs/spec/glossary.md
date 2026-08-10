# Glossary

Every named concept in this specification, with one line about what it is and a link to the section that defines it.

**The linked section is canonical.** This page is a lookup surface, not a second definition. Where this page and a specification disagree, the specification is right and this page has a defect. Entries stay to one or two sentences for that reason. A definition that needs a paragraph belongs to the specification that owns it.

In an implemented system this page is a projection. Kinds, facets, relations, and purposes all carry their own declarations, so the engine can emit the list. It is written by hand today because the engine does not exist yet. When it does, this page is generated and checked like any other [projection](01-conceptual-model.md#projections).

**On the count.** The list below holds just over 140 terms, and that number needs an honest reading. Most of them belong to the engine, the check layer, or the publisher, and no author ever meets them. An author who files a document meets the eight in the next table. A taxonomy author meets roughly thirty-five. The [core-concepts review](../reviews/) treats concept count as a live risk to adoption. This page is the inventory that makes the count visible, instead of leaving it to be felt.

## The eight an author needs

| Term | What it means when you file a document |
|---|---|
| [Shelf](#shelf) | The directory that you put the document in. It states which region of the corpus the document belongs to |
| [Kind](#kind) | What the document is. The shelf usually settles it, and the engine derives it |
| [Front matter](#front-matter) | The YAML block at the top. It carries the metadata that the kind requires |
| [Facet](#facet) | One named metadata field inside that block |
| [State](#state) | Where the document sits in its lifecycle |
| [Freshness](#freshness) | The date when a human last confirmed that the document is true |
| [Summary](#summary) | One sentence. It is how routing, indexes, and agents find the document |
| [Relation](#relation) | A typed link to another document, declared in front matter |

`headwater new <kind>` supplies most of them. `headwater explain <path>` states what the engine expects of a document, and why.

## Terms

### ABox

The assertion layer of the knowledge base: the documents and the edges that they declare. The corpus is the ABox. See [spec 1](01-conceptual-model.md#two-layers-terminology-and-assertions).

### Absence finding

A report that a document which should exist does not. [Participation expectations](#participation-expectation) are the only construct that produces one. See [spec 4](04-assurance-model.md#absence-is-a-finding-class-of-its-own).

### Adaptive

The control class that retunes the other three, through efficacy probes, false-positive rates, and promotion decisions. See [spec 4](04-assurance-model.md#assurance-not-enforcement).

### Advisory

A posture. The finding reports and does not block. Every new check starts here. See [spec 4](04-assurance-model.md#promotion-advisory-to-blocking).

### Aggregator

The tier that answers questions across taxonomies which share no vocabulary. It owns the [mappings](#mapping), normatively rather than conveniently. See [spec 7](07-distribution-and-federation.md#across-taxonomies-not-under-them).

### Anchor

See [external anchor](#external-anchor).

### Anchor resolver

The single component that owns identity for one anchor type. It normalizes anchor strings, so that two spellings of one target become one node. See [spec 2](02-taxonomy-model.md#behaviour-at-the-limits).

### Assisted fraction

The measured share of required front matter, sections, identifiers, and relations that the tooling supplied rather than the author. A fall in it is an assurance finding, not a dashboard curiosity. See [spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric).

### Audit

`headwater taxonomy audit` measures a taxonomy against a real corpus. Its findings are about the schema, and they are advisory by construction. Compare the [conformance audit](#conformance-audit), which asks a different question. See [spec 6](06-engine-architecture.md#taxonomy-validate-versus-taxonomy-audit).

### Backfill

A write-time hook. At creation, when it is cheap, the engine completes the front matter, identifier, and required sections of a new document. See [spec 5](05-ai-integration.md#write-time-hooks).

### Blocking

A posture. The finding fails the build. A check reaches it only through [promotion](#promotion) against evidence. See [spec 4](04-assurance-model.md#promotion-advisory-to-blocking).

### Cache

Content-addressed per file, plus the taxonomy lock hash, so that an incremental run costs what the change costs rather than what the corpus costs. See [spec 6](06-engine-architecture.md#pipeline).

### Capture cost

The author pays to record an edge or a rationale, and a later reader collects the benefit. This asymmetry killed every prior design-rationale system, so the system measures it instead of assuming it away. See [spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric).

### Census

The Phase A record of every file under the corpus root, and what became of it. It fixes the denominator for coverage before any check runs. See [spec 12](12-check-layer.md#two-phases-and-why-the-order-matters).

### Check

A pure function from a [scoped view](#scoped-view) of the graph to findings. No file access, no network, no clock, no mutation. That purity is what makes results cacheable and safe to run in parallel. See [spec 12](12-check-layer.md#what-a-check-is).

### Check instance

One check applied to one target. `facet_required` is not one check but several hundred instances. Findings, cache entries, timings, and coverage all attach to instances. See [spec 12](12-check-layer.md#instances-and-why-coverage-needs-them).

### Check origin

One of five sources for a check: Shape, Graph, Corpus, Document, and Plugin. Shape and Graph checks are generated from the taxonomy, and they hold most of the check count. See [spec 12](12-check-layer.md#the-five-origins-of-a-check).

### Clock

The injected time value that a check reads as `ctx.now`. It is never a system call, because a check must stay reproducible. See [spec 12](12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version).

### Coherence

The reader's experience that the corpus adds up to one account of the system. It needs judgment, so sampled controls discharge it. See [spec 4](04-assurance-model.md#cohesion-and-coherence-are-different-obligations).

### Coherence sweep

A periodic LLM-assisted pass over the *undeclared* half of the corpus. It reports findings, never verdicts, and it is not a [check](#check). Its best outcome is a declared edge, after which the engine owns the problem permanently. See [spec 4](04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep).

### Cohesion

The surface ties that bind the corpus: links that land, relations that reciprocate, identifiers that bind, enums that hold. Decidable, total, and eligible to block. See [spec 4](04-assurance-model.md#cohesion-and-coherence-are-different-obligations).

### Compatibility dimensions

The five measures that decide a version bump: classification, instance validity, consequence, projection, and identifier. The set belongs to the engine, and no taxonomy may vary it. See [spec 2](02-taxonomy-model.md#versioning-by-measured-compatibility).

### Confidence gate

The threshold below which routing says nothing. A wrong pointer costs more than a missing one, so the gate errs toward silence. See [spec 5](05-ai-integration.md#intent-time-routing).

### Confluence

The property that a set of overlays, applied in any legal order, gives the same resolved taxonomy. The resolver checks it statically, before it applies anything. See [spec 2](02-taxonomy-model.md#customisation-by-composition).

### Conformance

The evaluated question of whether a consumer wired the method, rather than only copied it. It checks the [core](#core), not the whole taxonomy, which is the difference between a method and a monoculture. See [spec 7](07-distribution-and-federation.md#conformance).

### Conformance audit

A periodic human or supervised-agent sample that asks one semantic question: does this specification still describe the system? Its output is a typed, tracked document inside the corpus. See [spec 4](04-assurance-model.md#conformance-audit).

### Consumer

A repository that adopts a published taxonomy package, and that measures compatibility against its own documents before an upgrade. See [spec 7](07-distribution-and-federation.md#consuming).

### Contract sidecar

Machine-verifiable artifacts beside the prose, such as schemas and identified acceptance criteria. The prose stays canonical for meaning, and the sidecar is canonical for shape. Structured criteria make a specification a test oracle. See [spec 2](02-taxonomy-model.md#contract-sidecars-the-specification-as-oracle).

### Control

The mechanism that discharges an [obligation](#obligation): an engine check, a CI job, a hook, an agent behavior, a scheduled scan, or a human audit. It declares what it verifies, when it runs, and its posture. See [spec 4](04-assurance-model.md#controls-are-data).

### Core

The semantics that an overlay may extend but never remove or redefine. It constrains roles and purposes, never names or paths, and the resolver checks it against the resolved result. See [spec 2](02-taxonomy-model.md#the-immutable-core).

### Corpus

The set of documents that one taxonomy governs, rooted at one directory in one repository. A repository has exactly one corpus. See [spec 1](01-conceptual-model.md#the-corpus).

### Corpus graph

Typed nodes and typed edges, built in one pass and cached. The graph, not the file tree, is the engine's working representation. See [spec 1](01-conceptual-model.md#the-corpus).

### Correction

An edit in place, made when a document was wrong about the present. It preserves truth. Contrast [succession](#succession), which preserves lineage. To conflate the two destroys the record. See [spec 3](03-authoring-and-lifecycle.md#lifecycle).

### Corrective

The control class that repairs what detection found: auto-fix, generated remediation tasks, and agent-raised change proposals. See [spec 4](04-assurance-model.md#assurance-not-enforcement).

### Counterfactual probe

An A/B run of one scenario with the corpus present and absent. It is the only evidence that the instruction surface earns its context cost. See [spec 5](05-ai-integration.md#measuring-whether-any-of-this-works).

### Coverage

Two related reports. Over obligations, the fraction discharged by severity, with the gap list. Over a run, the documents seen, classified, checked, and skipped, each with a reason. Clean runs report it too. See [spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for).

### `created_by`

The required declaration of who creates each edge, from a closed set: `author`, `scaffold`, `generator`, `hook`, `agent`, `import`. It forces the question of who pays for a link at design time, not after the corpus stops maintaining it. See [spec 2](02-taxonomy-model.md#who-creates-each-edge).

### Declarative regime

The [voice regime](#voice-regime) for present-state documents. It forbids future intent, change narration, and phased-rollout language. See [spec 3](03-authoring-and-lifecycle.md#voice).

### Detective

A posture. The control acts after the mistake lands, and never blocks. Staleness and participation expectations are permanently detective. See [spec 4](04-assurance-model.md#assurance-not-enforcement).

### Discriminator

The facet that names the kind on a [heterogeneous shelf](#heterogeneous-shelf). A [homogeneous shelf](#homogeneous-shelf) forbids it, because the directory already says what the document is. See [spec 2](02-taxonomy-model.md#placement-is-primary-metadata-fills-the-gap).

### Disposition

What an obligation's coverage amounts to: verified, gap, or unverifiable. Every obligation carries exactly one, and an obligation with none is itself a finding. See [spec 4](04-assurance-model.md#every-obligation-has-exactly-one-disposition).

### Doctrine

The prose that explains the method. The publisher ships it inside the taxonomy package, and consumers vendor or link it. See [spec 7](07-distribution-and-federation.md#what-is-shared-and-what-is-not).

### Document

A file in the corpus, with a Markdown body and YAML front matter. It resolves to exactly one [kind](#kind). No kind, or more than one, is a defect that the engine reports rather than guesses at. See [spec 1](01-conceptual-model.md#document).

### Dwell

The time that a document spends in one state, computed from the [state-entry date](#state-entry-date). The audit reports the distribution per state. Dwell is observed, not policed. See [spec 3](03-authoring-and-lifecycle.md#lifecycle).

### Edge

One relation instance in the graph. The edge, not the file, is the unit that change-scoped evaluation invalidates. See [spec 1](01-conceptual-model.md#edges).

### Efficacy

The measured effect of the instruction surface on agent behavior, taken from the probe suite. A rule that measurably changes nothing is a candidate for deletion. See [spec 5](05-ai-integration.md#measuring-whether-any-of-this-works).

### Engine

The deterministic program that parses the corpus once, builds one graph, and runs every check against it. No LLM sits in its validation path. See [spec 6](06-engine-architecture.md#why-one-engine).

### Escape hatch

A stated, reasoned exemption from a rule, scoped to a file or a block. Voice checking and [suppression](#suppression) both use them. A shelf that collects them is a finding about the shelf, not about the documents. See [spec 3](03-authoring-and-lifecycle.md#voice) and [spec 4](04-assurance-model.md#suppression).

### Evidence basis

One of three honest states for the support behind a decision: `evidenced`, `reconstructed`, or `gap`. `reconstructed` is not a soft `evidenced`, and it never promotes silently. See [spec 3](03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two).

### Explain

`headwater explain` prints why the engine typed a document as it did. It names the shelf that matched, the rule that fired, the declared purpose, and what is consequently required. Classification is never a black box. See [spec 2](02-taxonomy-model.md#kind-resolution).

### External anchor

A node for something outside the corpus that documents point at: a code path, a work item, a service, a URL. It carries an identity, and never a purpose or a lifecycle. See [spec 1](01-conceptual-model.md#external-anchor).

### Facet

A named metadata dimension on documents, with a declared value space, applicability, requiredness, and enforcement strength. A facet value is never a reference to another node, because a connection is a [relation](#relation). See [spec 1](01-conceptual-model.md#facet).

### Facet canons

Five acceptance tests for a facet: relevance, ascertainability, permanence, differentiation, and orthogonality. The first three are decidable from the schema, and the last two need documents. See [spec 2](02-taxonomy-model.md#facet-acceptance-tests).

### Facet role

One of five engine-significant roles that a facet may carry: `state`, `state_entered`, `created`, `freshness`, and `scent`. The registry is closed, and a role outside it is a validation error. See [spec 2](02-taxonomy-model.md#the-meta-schema).

### Family

One of six fixed classes that every relation belongs to: `succession`, `derivation`, `governance`, `evidence`, `composition`, and `association`. The family supplies default semantics, so a new relation type inherits checking for free. See [spec 2](02-taxonomy-model.md#family).

### Federation

The layered arrangement of a generic method, a divisional taxonomy, and a repository overlay. References run upward only, and overlays compose in one direction. See [spec 7](07-distribution-and-federation.md#federation).

### Finding

The uniform record that every mechanism emits: rule, severity, obligation, path, line, message, remediation, and fixability. Every finding names the obligation that it serves. See [spec 4](04-assurance-model.md#findings).

### Fixability

Whether a check may return a patch beside its finding. It may, only when the fix is mechanical and total: one correct outcome, derivable without judgment. See [spec 12](12-check-layer.md#fixability).

### Focus shift

An edge whose two endpoints share nothing, so that the reader must reorient on arrival. Individual shifts are fine. The distribution is the signal. See [spec 4](04-assurance-model.md#measuring-coherence-where-we-can-continuity-across-links).

### Freshness

The date when a human last confirmed that a document agrees with reality. It is deliberately not the last-edited date, which says nothing about truth. See [spec 3](03-authoring-and-lifecycle.md#freshness-and-staleness).

### Front matter

The YAML block that carries the facets which a kind requires. It is the machine's only guaranteed read of a document, so the schema is strict about it. See [spec 3](03-authoring-and-lifecycle.md#front-matter-is-the-contract).

### Gap

Two senses, both deliberate. As a [disposition](#disposition), no control discharges the obligation yet, and an owner tracks it. As an [evidence basis](#evidence-basis), no evidence exists and none is claimed. See [spec 4](04-assurance-model.md#every-obligation-has-exactly-one-disposition) and [spec 3](03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two).

### Generated-file marker

The marker that a projection writes into its output. The engine refuses to overwrite a file that lacks it, so a projection can never silently destroy an authored document. See [spec 6](06-engine-architecture.md#projections).

### Heterogeneous shelf

A shelf that legitimately holds several kinds. Its documents carry a [discriminator](#discriminator), and the audit watches the distribution, because a self-asserted kind is a route out of an expensive contract. See [spec 1](01-conceptual-model.md#shelf).

### Homogeneous shelf

A shelf where every document is of one kind. It forbids the discriminator facet, so that front matter cannot create a second truth about what a document is. See [spec 2](02-taxonomy-model.md#placement-is-primary-metadata-fills-the-gap).

### Identifier scheme

The declared pattern, namespace, and allocation policy of a stable identifier. Identifiers are globally unique, resolvable without their document, and never reused. See [spec 3](03-authoring-and-lifecycle.md#identifiers).

### Impact detection

A write-time hook. An edit to code that a document governs raises an advisory prompt that names the documents at risk. It stays advisory, because a gate here teaches people to write "no doc impact" by reflex. See [spec 5](05-ai-integration.md#write-time-hooks).

### Kind

What a document *is*. A kind carries a purpose, a section contract, a facet schema, voice and lifecycle regimes, an identifier scheme, permitted relations, and a template. See [spec 1](01-conceptual-model.md#kind).

### Kind resolution

The four-step derivation of a kind from shelf match, shelf homogeneity, discriminator facet, and path-pattern refinement. It is deterministic and explainable. See [spec 2](02-taxonomy-model.md#kind-resolution).

### Lifecycle regime

A state machine over the state facet: states, legal transitions, terminal states, and what each state implies. It is declared in the taxonomy and interpreted by the engine. See [spec 3](03-authoring-and-lifecycle.md#lifecycle).

### Lifecycle-sensitive

A property of a relation and of its family. A live document may not depend on a terminal one through a lifecycle-sensitive relation. See [spec 3](03-authoring-and-lifecycle.md#lifecycle).

### Lock

The resolved taxonomy, written with a content hash and committed. Everything downstream reads the lock and never the sources, so a check result depends on a hash that a reviewer can see in a diff. See [spec 6](06-engine-architecture.md#pipeline).

### Maintainer subagent

A context-isolated agent that owns documentation upkeep across a change, with its own bounded instruction subset. It does not inflate the always-on prompt of every session. See [spec 5](05-ai-integration.md#agent-surfaces).

### Mapping

A declared SKOS correspondence between two taxonomies: `exactMatch`, `closeMatch`, `broadMatch`, `narrowMatch`, `relatedMatch`. The [aggregator](#aggregator) tier owns them, because peers do not map to peers at scale. See [spec 2](02-taxonomy-model.md#mapping-between-taxonomies).

### MCP server

The agent-facing surface of the engine library. It exposes `route`, `governing_docs_for_path`, `resolve_identifier`, `related`, `explain`, and `check`. It is read-only by default. See [spec 5](05-ai-integration.md#agent-surfaces).

### Meta-schema

The formal schema of the taxonomy language, published and versioned with the engine. `taxonomy validate` checks a taxonomy against it, and the engine never applies a taxonomy that fails. See [spec 2](02-taxonomy-model.md#the-meta-schema).

### Migration payload

The machine-readable steps that ship with a major version, split into what the engine applies mechanically and what needs human or agent judgment. See [spec 7](07-distribution-and-federation.md#upgrading).

### Migration state

The record in the lock while judgment-bearing migration tasks stay open: from-version, to-version, owner, expiry, and the task list. The expiry is what stops a corpus from parking there. See [spec 7](07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states).

### `migration-pending`

The label on a finding whose (document, rule) pair the migration payload expects to fail. Such findings are counted, visible in coverage, never blocking, and never suppressed one at a time. See [spec 7](07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states).

### Multinuclear

A relation where both ends stand alone. See [nuclearity](#nuclearity).

### Namespace

The scope in which an identifier is unique, applied at the moment of minting. To retrofit one later is expensive, so the default is to always have one. See [spec 3](03-authoring-and-lifecycle.md#identifiers).

### Narrative regime

The [voice regime](#voice-regime) for time-bound kinds such as proposals, evidence, and incident records. They are exempt from the declarative rules by nature. See [spec 3](03-authoring-and-lifecycle.md#voice).

### `Neighbourhood`

A check scope: a node and its n-hop neighbors. The identifier keeps the spelling that spec 12 gives it. Spec 12 marks it speculative and offers to cut it if no real check needs depth greater than one. See [spec 12](12-check-layer.md#scope--the-declaration-everything-else-rests-on).

### Normative language

The declared requirement keywords, usually RFC 2119. The engine checks the casing, the interpretation boilerplate, and hedged pseudo-requirements such as "should probably". See [spec 3](03-authoring-and-lifecycle.md#normative-language).

### Nuclearity

Whether a relation is [multinuclear](#multinuclear) or nucleus–satellite. It drives lifecycle inheritance, context pruning, orphan detection, and deletion safety. The family supplies the default. See [spec 2](02-taxonomy-model.md#nuclearity).

### Nucleus

The end of a nucleus–satellite relation that stands alone. It governs the reading of the pair, because a document that cannot stand alone cannot govern one that it depends on. See [spec 2](02-taxonomy-model.md#reading-precedence-is-derived).

### Obligation

A stable, identified invariant that the corpus commits to, declared as data rather than as prose. Each one cites the observed defect class that it prevents. See [spec 4](04-assurance-model.md#obligations-are-data).

### Origin

The declared date role that a participation window measures from: `state_entered`, or `created` where no state condition applies. A window that cannot say where it starts is not a window. See [spec 2](02-taxonomy-model.md#participation-expectations).

### Overlay

The declared customization of a base taxonomy, through `override`, `add`, and `remove`. An adopter never edits a base taxonomy. Merge semantics are strict, total, and order-independent. See [spec 2](02-taxonomy-model.md#customisation-by-composition).

### Participation expectation

A kind's declaration that its documents, in a given state, acquire a named relation to another kind within a window. It is the only construct that finds a document which should exist and does not. Detective only. See [spec 2](02-taxonomy-model.md#participation-expectations).

### Pin

The taxonomy version that a consumer holds. A scheduled check compares it against the publisher's latest release. See [spec 7](07-distribution-and-federation.md#upstream-awareness).

### Plugin

Adopter check code behind a narrow interface. It receives the same scoped view as a built-in check, obeys the same scope enforcement, and must name the obligation that it serves. See [spec 12](12-check-layer.md#the-plugin-interface).

### Pointer

A path plus a one-line summary. Routing returns pointers and never content, so the corpus stays the single source and the context cost stays near zero. See [spec 5](05-ai-integration.md#intent-time-routing).

### Posture

How a control acts: advisory, blocking, or detective. Posture belongs to the control. [Severity](#severity) belongs to the check. See [spec 12](12-check-layer.md#severity-is-the-checks-posture-is-the-controls).

### Preventive

The control class that acts before the mistake lands: scaffolding, editor validation, agent behavior, and commit hooks. See [spec 4](04-assurance-model.md#assurance-not-enforcement).

### Prior version

A change-scoped check input, supplied by the diff. It is the merge-base version for a proposed change, and the committed `HEAD` version for a working-tree hook. Transition legality cannot be decided without it. See [spec 12](12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version).

### Probe suite

Scenarios run against the corpus in a controlled session, graded from the tool-call transcript rather than from the model's self-report. Probes run with a pinned model and a cost envelope. See [spec 5](05-ai-integration.md#measuring-whether-any-of-this-works).

### Profile

A named overlay that the publisher ships for a repository archetype. It is a use of the overlay mechanism, not a mechanism of its own. See [spec 7](07-distribution-and-federation.md#profiles-are-publisher-overlays).

### Projection

A derived artifact computed from the graph: a shelf index, a lineage view, site navigation, an agent rule file, a graph export. Projections are generated, checked against regeneration, and declared in the schema. See [spec 1](01-conceptual-model.md#projections).

### Promotion

The evidence-bound move of a check from advisory to blocking. It needs an observation window, a false-positive rate under a declared threshold, a mechanical remediation path, and an adjudicated sample. Demotion is the inverse. See [spec 4](04-assurance-model.md#promotion-advisory-to-blocking).

### Provenance

The PROV-aligned record of who drafted a document, by which activity, and who accepted it. An agent may draft. Acceptance is a human act, and the record names the human. See [spec 3](03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed).

### Publisher

The organization that releases a taxonomy package, its doctrine, and its migrations, and that measures compatibility against its own reference corpora. See [spec 7](07-distribution-and-federation.md#publishing).

### Purpose

The reader intent that a kind serves, declared once at the taxonomy level. A kind without a purpose fails validation, because purpose plus form is what makes a kind a genre rather than a shape. See [spec 2](02-taxonomy-model.md#purpose-is-declared-not-implied).

### Reading precedence

Which of two linked documents governs the reading. It is derived from nuclearity and succession, and never declared. See [spec 2](02-taxonomy-model.md#reading-precedence-is-derived).

### Reciprocity

Whether the target of a relation must acknowledge the source, and with which inverse relation. Two halves that disagree are a finding, and the engine prefers neither side. See [spec 1](01-conceptual-model.md#relation).

### Register projection

The generated view of which controls discharge which obligations, with every disposition, control health, suppressions, and waivers. It is engine-defined, non-optional, and never authored. See [spec 4](04-assurance-model.md#every-obligation-has-exactly-one-disposition).

### Regime

A reusable, named bundle of rules that a kind opts into, so that a rule is declared once and referenced many times. Two survive: [voice](#voice-regime) and [lifecycle](#lifecycle-regime). See [spec 1](01-conceptual-model.md#regimes).

### Relation

A typed, named, directed link between two documents, or from a document to an anchor, declared in front matter. It carries a family, nuclearity, endpoints, cardinality, reciprocity, and lifecycle interaction. See [spec 1](01-conceptual-model.md#relation).

### Remediation

The instruction that every finding carries. A finding that cannot say what to do next is noise. See [spec 4](04-assurance-model.md#findings).

### Resolve

`headwater taxonomy resolve` fetches the base, merges the overlays, validates the result, and writes the [lock](#lock). See [spec 7](07-distribution-and-federation.md#consuming).

### Rigidity

A kind is rigid, because a document cannot lose it and stay the same document. A state is anti-rigid. So the validator rejects a kind that collides with a state value, or that is named with a bare phase adjective. See [spec 2](02-taxonomy-model.md#kinds-are-rigid-states-are-not).

### Routing

The intent-time match of a task description against declared purposes, before any match on text. It returns a ranked, budget-capped set of [pointers](#pointer), and it fails open below its confidence gate. See [spec 5](05-ai-integration.md#intent-time-routing).

### Sampler

The non-deterministic path that carries the [coherence sweep](#coherence-sweep). It shares the finding shape and the reporting pipeline, and it never enters the cached, reproducible path. See [spec 12](12-check-layer.md#where-the-llm-coherence-sweep-fits).

### Satellite

The end of a nucleus–satellite relation that cannot stand alone. It inherits declared facets from its nucleus, and a context budget drops it before the nucleus. See [spec 2](02-taxonomy-model.md#nuclearity).

### Scaffolding

`headwater new` creates a document with correct placement, front matter, sections, and identifier, and prints the relations that the document is expected to declare. See [spec 3](03-authoring-and-lifecycle.md#templates-and-scaffolding).

### Scent

The proximal cue that predicts distal value, and the thing that routing trades in. The `summary` facet is the corpus's whole scent surface, so its quality is measured rather than assumed. See [spec 5](05-ai-integration.md#intent-time-routing).

### Scope

What a check declares that it needs to see. The values are `Document`, `Edge`, `Neighbourhood`, `Shelf`, and `Corpus`, plus flags for the body, the clock, and the prior version. See [spec 12](12-check-layer.md#scope--the-declaration-everything-else-rests-on).

### Scoped view

The only thing that a check can read. The engine enforces it, and that enforcement is what makes cache keys sound and change-scoped runs exact. See [spec 12](12-check-layer.md#scope--the-declaration-everything-else-rests-on).

### Section contract

The headings that a kind must or may have. See [spec 1](01-conceptual-model.md#kind).

### Severity

The check's report of how bad a finding is. Whether that severity blocks is the control's business, so promotion needs no code change. See [spec 12](12-check-layer.md#severity-is-the-checks-posture-is-the-controls).

### Shelf

A named region of the corpus, in practice a directory or glob, that carries a purpose. Placement is the loudest signal that a document sends, so the shelf is the primary classification axis. See [spec 1](01-conceptual-model.md#shelf).

### Shift ratio

The share of edges that are [focus shifts](#focus-shift), reported as a corpus-health metric. It stays permanently advisory, because a gate on it causes link padding and destroys the measure. See [spec 4](04-assurance-model.md#measuring-coherence-where-we-can-continuity-across-links).

### Size budget

The declared context limit on an agent-facing kind or projection. It is mandatory, and an agent-facing projection without an applicable budget fails validation. See [spec 5](05-ai-integration.md#read-time-rule-loading).

### Staleness

The condition of a document past its freshness threshold, weighted by drift risk rather than by calendar time alone. Detective, never blocking. See [spec 3](03-authoring-and-lifecycle.md#freshness-and-staleness).

### State

The facet role that the lifecycle regime interprets. The default taxonomy calls it `status`, and a corpus may call it anything, because the role is declared. See [spec 1](01-conceptual-model.md#facet).

### State-entry date

The date when a document entered its current state. Whatever performs the transition stamps it, the same diff enforces the stamp, and participation windows measure from it. See [spec 3](03-authoring-and-lifecycle.md#front-matter-is-the-contract).

### Stop rules

The four refusals that an assistant working in the corpus must show. No rationale without external evidence. No hand edit of a generated file. No new shelf, kind, or facet invented in place. No duplication of a fact that exists elsewhere. See [spec 5](05-ai-integration.md#the-stop-rules).

### Succession

The relation family where one artifact replaces another, and the path that preserves lineage when the decision itself changed. The successor governs the reading. See [spec 2](02-taxonomy-model.md#family).

### Summary

The facet that carries the `scent` role in the default taxonomy. One sentence, for machine consumption. It is the public interface of the document, not decoration. See [spec 3](03-authoring-and-lifecycle.md#front-matter-is-the-contract).

### Suppression

A bounded exemption scoped to a file or a block, with an expiry and a reason from a closed set: `false_positive` or `accepted_deviation`. Only `false_positive` counts toward promotion and demotion statistics. See [spec 4](04-assurance-model.md#suppression).

### Taxonomy

The declared structure of a corpus, in eleven declarations: purposes, facets, regimes, relations, anchors, shelves, kinds, identifier schemes, core, mappings, and projections. It is data, not code. See [spec 2](02-taxonomy-model.md#the-eleven-declarations).

### Taxonomy package

The released, versioned unit that a publisher ships: taxonomy, doctrine, templates, plugins, profiles, and migrations. The taxonomy is a package, not a copy. See [spec 7](07-distribution-and-federation.md#publishing).

### TBox

The terminology layer of the knowledge base: which kinds of thing exist, which relations may hold, and which values are legal. The taxonomy is the TBox. See [spec 1](01-conceptual-model.md#two-layers-terminology-and-assertions).

### Template

The starting document for a kind, generated from the kind declaration. A template that drifted from its kind is impossible by construction. See [spec 3](03-authoring-and-lifecycle.md#templates-and-scaffolding).

### Tier

One layer of a federation: the generic method, a divisional taxonomy, or a repository overlay. See [spec 7](07-distribution-and-federation.md#federation).

### Unverifiable

A [disposition](#disposition). No mechanism can discharge the obligation, the organization accepts that, and the reasoning is recorded. See [spec 4](04-assurance-model.md#every-obligation-has-exactly-one-disposition).

### Upstream awareness

The scheduled comparison of the [pin](#pin) against the publisher's latest release. It raises a draft change proposal, not a notification that nobody acts on. See [spec 7](07-distribution-and-federation.md#upstream-awareness).

### Validate

`headwater taxonomy validate` decides the schema alone: referential integrity, determinism, purpose completeness, rigidity, edge provenance, overlay confluence, and core satisfiability. It needs no documents, and it gates everything. See [spec 6](06-engine-architecture.md#taxonomy-validate-versus-taxonomy-audit).

### Vocabulary

A named value set that facets reference. It is authoring syntax, validated as part of `facets`, and not a concept that anyone must learn first. See [spec 2](02-taxonomy-model.md#the-eleven-declarations).

### Voice regime

The register that a kind writes in, with the machine-checkable part of it: declarative, narrative, or unconstrained. Enforcement is lexical, and every escape hatch must state a reason. See [spec 3](03-authoring-and-lifecycle.md#voice).

### Volatility

A facet's declaration of permanence. A `mutable` facet may not appear in an identifier, a path, or a shelf pattern. See [spec 2](02-taxonomy-model.md#facet-acceptance-tests).

### Waiver

A consumer's deliberate deviation from a conformance rule, with a rule name, a reason, an owner, and an expiry. Deviation is fine. Invisible deviation is not. See [spec 7](07-distribution-and-federation.md#waivers).

### Window

The time bound on a [participation expectation](#participation-expectation), measured from a declared [origin](#origin). An expectation with no window is a wish. See [spec 2](02-taxonomy-model.md#participation-expectations).

## Distinctions that the design depends on

Each pair below is two concepts that read as one. The specification treats each difference as structural, so a reader who collapses a pair will misread the design.

| This | Is not this | Why the difference matters |
|---|---|---|
| [Cohesion](#cohesion) | [Coherence](#coherence) | One is decidable and can block. The other needs judgment and stays sampled. A green run means cohesive, never coherent |
| [`taxonomy validate`](#validate) | [`taxonomy audit`](#audit) | `validate` decides the schema alone and gates. `audit` measures the schema against documents and only advises |
| [Correction](#correction) | [Succession](#succession) | An edit in place preserves truth. A successor preserves lineage. To conflate them destroys the record |
| [Severity](#severity) | [Posture](#posture) | The check states how bad a finding is. The control decides whether it blocks. So promotion is a configuration change |
| [Suppression](#suppression) | [Waiver](#waiver), [`migration-pending`](#migration-pending) | Three escape mechanisms of different width. Coverage counts each finding under the widest one that applies |
| [Gap](#gap) | [Unverifiable](#unverifiable) | A gap is wanted and tracked. An unverifiable obligation has no possible mechanism, and the organization accepts that |
| Defined relation | Enabled relation | A package defines a full vocabulary. A taxonomy enables a subset. An edge that names a defined but unenabled relation is an unknown-relation finding |
| [Nucleus](#nucleus) | [Satellite](#satellite) | The nucleus governs the reading, and a context budget drops the satellite first. Orphan findings differ in severity and owner |
| [Kind](#kind) | [State](#state) | A kind is rigid and a state is not. Lifecycle state must never become a kind, a shelf, or a directory |
| [Obligation](#obligation) | [Control](#control) | The obligation is the commitment. The control is the mechanism. The binding between them is generated, never authored |
| [Facet](#facet) | [Relation](#relation) | A connection between nodes is always a relation. A facet value is a scalar, and never a reference |
| [Projection](#projection) | Authored document | A projection is regenerable, and CI checks it against regeneration. Synthesized content is neither ([Q15](09-open-questions.md#q15--a-synthesised-content-tier)) |

## Where the borrowed terms come from

Several terms are not ours. [Spec 10](10-theoretical-foundations.md) records the confrontation with each source, and this table gives the attribution in one place.

| Term | Source | What it contributes |
|---|---|---|
| [TBox](#tbox), [ABox](#abox) | Description logic | Why taxonomy validation and corpus checking are two operations, not two halves of one |
| [Nuclearity](#nuclearity), [multinuclear](#multinuclear) | Rhetorical Structure Theory (Mann & Thompson, 1988) | The asymmetry that drives inheritance, context pruning, orphan severity, and deletion safety |
| [Cohesion](#cohesion), [coherence](#coherence) | Halliday & Hasan, *Cohesion in English* (1976) | The split between what the engine can decide and what needs a reader |
| [Focus shift](#focus-shift) | Centering Theory (Grosz, Joshi & Weinstein, 1995) | Local coherence as continuity of focus, which is what makes the shift ratio measurable |
| [Facet canons](#facet-canons) | Ranganathan, and Vickery, *Faceted Classification* (1960) | Acceptance tests for a facet, in place of taste |
| [Rigidity](#rigidity) | OntoClean (Guarino & Welty) | Why a lifecycle state may never be modeled as a kind |
| [Purpose](#purpose), genre systems | Yates & Orlikowski (1992–2002) | A kind is a genre, so purpose is required. Genre systems became participation expectations |
| [Core](#core) as a boundary object | Star & Griesemer (1989) | Plastic enough to adapt locally, strong enough to keep a common identity across sites |
| [Scent](#scent) | Pirolli & Card, *Information Foraging* (1999) | Routing has a theory, and the confidence gate errs toward silence |
| The decision relations | Kruchten, *An Ontology of Architectural Design Decisions* (2004) | The default decision-relation vocabulary, adopted rather than invented |
| PROV | W3C | Lineage and provenance alignment for `derives_from`, `supersedes`, and generated artifacts |
| SKOS [mapping](#mapping) relations | W3C | `exactMatch`, `closeMatch`, `broadMatch`, `narrowMatch`, `relatedMatch` between taxonomies |
| [Confluence](#confluence), overlay operations | Delta-oriented programming (Schaefer et al.) | Order independence of overlays, checked statically before application |
| [Capture cost](#capture-cost) | IBIS, gIBIS, QOC, and the traceability literature | The failure that killed fifty years of design-rationale tools, now a tracked metric |
| RFC 2119 | IETF | The normative keyword set that voice checking assumes by default |
| LinkML, SHACL | The [evaluations](../evaluations/) | The export target for Shape and Graph checks, and an open substrate question ([Q13](09-open-questions.md#q13--linkml-and-shacl-as-substrate)) |
| SARIF | OASIS | One of the finding output formats |
| MCP | Model Context Protocol | The agent-facing surface of the engine library |

## Terms with a known collision

This page makes three name clashes visible. Each one is recorded here, and none is resolved here.

- **register** — the voice sense (the register that a document writes in) and the [register projection](#register-projection) use one word for two concepts. The [STE pass](../reviews/ste-editorial-pass-findings.md) flagged it and asked for a ruling.
- **gap** — a [disposition](#disposition) on an obligation, and a value of [`evidence_basis`](#evidence-basis) on a document. The two are related in spirit and separate in mechanism.
- **audit** — [`taxonomy audit`](#audit) measures a schema against a corpus. A [conformance audit](#conformance-audit) is a human sample that asks whether a specification is still true of the system.

One further usage is deliberate rather than accidental. Routing **fails open** when it stays silent below its confidence gate. Conventional security usage would call that failing closed. Design principle 7 in [spec 0](00-vision-and-scope.md#design-principles) fixes the project meaning.
