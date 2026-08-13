---
id: SPEC-HW-glossary
status: current
status_since: 2026-08-10
last_verified: 2026-08-11
summary: One line for every named concept in the specification, with a link to the section that defines it.
doc_type: design_spec
sequence: 14
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cites_evidence:
    - EVAL-HW-default-taxonomy-first-run
    - EVAL-HW-graph-export-and-federation
    - EVAL-HW-the-measurement-layer
    - EVAL-HW-the-serving-boundary
    - EVAL-HW-warrant-and-adjudication
---

# Glossary

Every named concept in this specification, with one line about what it is and a link to the section that defines it.

**The linked section is canonical.** This page is a lookup surface, not a second definition. Where this page and a specification disagree, the specification is right and this page has a defect. Entries stay to one or two sentences for that reason. A definition that needs a paragraph belongs to the specification that owns it.

**A human writes this page, and no engine can generate it.** A [projection](01-conceptual-model.md#projections) of this repository's taxonomy emits one entry for each declared kind, facet, relation and purpose. That is 27 entries, with names like `governed_document`, `status` and `supersedes`. This page defines the words of the schema language rather than the declarations that are written in it. Of the 175 terms below, exactly one is the name of a declaration, and it is [summary](#summary). The two lists are almost disjoint, which is the measurement that [13 — Open obligations](13-open-obligations.md#a-human-maintains-this-list-by-hand) records.

**On the count.** The list below holds about 175 terms, and that number needs an honest reading. Most of them belong to the engine, the check layer, or the publisher, and no author ever meets them. An author who files a document meets the eight in the next table. A taxonomy author meets roughly thirty-five. The [core-concepts review](../reviews/) treats concept count as a live risk to adoption. This page is the inventory that makes the count visible, instead of leaving it to be felt.

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
| [Relation](#relation) | A typed link to another document, declared under `relations:` in front matter |

`headwater new <kind>` supplies most of them. `headwater explain <path>` states what the engine expects of a document, and why.

## Terms

### ABox

The assertion layer of the knowledge base: the documents and the edges that they declare. The corpus is the ABox. See [spec 1](01-conceptual-model.md#two-layers-terminology-and-assertions).

### Absence finding

A report that a document which should exist does not. [Participation expectations](#participation-expectation) are the only construct that produces one. See [spec 4](04-assurance-model.md#absence-is-a-finding-class-of-its-own).

### Abstract kind

A [kind](#kind) that no document ever is. It carries what a group of concrete kinds share, and a concrete kind inherits from it with `is_a`. Relation endpoints may name one, and kind resolution never returns one. See [spec 2](02-taxonomy-model.md#abstract-kinds).

### Accuracy audit

A periodic human or supervised-agent sample that asks one semantic question: does this specification still describe the system? Its output is a typed, tracked document inside the corpus. It was called a conformance audit until the name collided with [conformance](#conformance). See [spec 4](04-assurance-model.md#accuracy-audit).

### Adaptive

The control class that retunes the other three, through efficacy probes, false-positive rates, and promotion decisions. See [spec 4](04-assurance-model.md#assurance-not-enforcement).

### Adjudication

A human ruling that settles a disagreement between two documents. It is itself a decision, and it carries `overrides` against the document whose effect it displaces. The adjudicator is the name in `accepted_by`. There is no rank and no scalar. See [spec 2](02-taxonomy-model.md#disagreement-is-adjudicated-not-ranked).

### Advisory

A posture. The finding reports and does not block. Every new check starts here. See [spec 4](04-assurance-model.md#promotion-advisory-to-blocking).

### Aggregator

The tier that answers questions across taxonomies which share no vocabulary. It is a solution corpus plus one anchor kind, and it holds no merged graph. It owns the [mappings](#mapping), normatively rather than conveniently. It harvests [pinned exports](#pinned-export) and never queries a live endpoint. See [spec 7](07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it).

### Anchor

See [external anchor](#external-anchor).

### Anchor resolver

The single component that owns identity for one anchor type. It normalizes anchor strings, so that two spellings of one target become one node. See [spec 2](02-taxonomy-model.md#behavior-at-the-limits).

### Arm

Whether a probe run had the corpus present or absent. Every probe runs in one arm, and a published efficacy claim needs the pair. See [spec 5](05-ai-integration.md#probe-categories).

### Asserted content

Content with the `asserted` [warrant](#warrant): nobody accepted it, and no regeneration proves it. Headwater admits it, marks it positively, and never lets it govern the reading of warranted content or discharge an evidence obligation. See [spec 1](01-conceptual-model.md#warrant).

### Assisted fraction

The measured share of required front matter, sections, identifiers, and relations that the tooling supplied rather than the author. A fall in it is an assurance finding, not a dashboard curiosity. See [spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric).

### Audit

`headwater taxonomy audit` measures a taxonomy against a real corpus. Its findings are about the schema, and they are advisory by construction. Compare the [accuracy audit](#accuracy-audit), which asks a different question. See [spec 6](06-engine-architecture.md#taxonomy-validate-versus-taxonomy-audit).

### Backfill

A write-time hook. At creation, when it is cheap, the engine completes the front matter, identifier, and required sections of a new document. See [spec 5](05-ai-integration.md#write-time-hooks).

### Base package

The minimal taxonomy that everything else composes over, derived from the [core](#core) rather than chosen. Nobody is expected to run it bare. See the [first-run walkthrough](../evaluations/default-taxonomy-first-run.md).

### Blocking

A posture. The finding fails the build. A check reaches it only through [promotion](#promotion) against evidence. See [spec 4](04-assurance-model.md#promotion-advisory-to-blocking).

### Bundle

A named overlay that the publisher ships, which adds optional content and declares its dependency closure. It holds no `override` and no `remove`, so any subset of bundles resolves. Compare a [profile](#profile), which is the same mechanism pointed the other way. See [spec 7](07-distribution-and-federation.md#bundles-are-publisher-overlays-in-the-other-direction).

### Cache

Content-addressed per file, plus the taxonomy lock hash, so that an incremental run costs what the change costs rather than what the corpus costs. It is disposable by test: a run with the cache and a run without it produce byte-identical output. See [spec 6](06-engine-architecture.md#nothing-stores-the-graph).

### Campaign

A powered, paired probe run for one named claim, executed as a single batch at one model version. Compare the regression tier, which runs one arm on a schedule and estimates no effect. See [spec 5](05-ai-integration.md#two-tiers-and-the-cadence-follows-the-purpose).

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

The six measures that decide a version bump: classification, instance validity, consequence, projection, identifier, and addressability. The first five measure against a corpus and the last against overlays. The set belongs to the engine, and no taxonomy may vary it. See [spec 2](02-taxonomy-model.md#versioning-by-measured-compatibility).

### Confidence gate

The threshold below which routing says nothing. A wrong pointer costs more than a missing one, so the gate errs toward silence. See [spec 5](05-ai-integration.md#intent-time-routing).

### Confluence

The property that a set of overlays, applied in any legal order, gives the same resolved taxonomy. The resolver checks it statically, before it applies anything. See [spec 2](02-taxonomy-model.md#customization-by-composition).

### Conformance

The evaluated question of whether a consumer wired the method, rather than only copied it. It checks the [core](#core), not the whole taxonomy, which is the difference between a method and a monoculture. See [spec 7](07-distribution-and-federation.md#conformance).

### Consumer

A repository that adopts a published taxonomy package, and that measures compatibility against its own documents before an upgrade. See [spec 7](07-distribution-and-federation.md#consuming).

### Contract sidecar

Machine-verifiable artifacts beside the prose, such as schemas and identified acceptance criteria. The prose stays canonical for meaning, and the sidecar is canonical for shape. Structured criteria make a specification a test oracle. See [spec 2](02-taxonomy-model.md#contract-sidecars-the-specification-as-oracle).

### Control

The mechanism that discharges an [obligation](#obligation): an engine check, a CI job, a hook, an agent behavior, a scheduled scan, or a human audit. It declares what it verifies, when it runs, and its posture. See [spec 4](04-assurance-model.md#controls-are-data).

### Core

The semantics that an overlay may extend but never remove or redefine. It constrains roles and purposes, never names or paths, and the resolver checks it against the resolved result. See [spec 2](02-taxonomy-model.md#the-immutable-core).

### Corpus

The set of documents that one taxonomy governs, rooted at one directory. One taxonomy and one root make a corpus, and a repository holds one or more. A path resolves to exactly one of them. See [spec 1](01-conceptual-model.md#the-corpus).

### Corpus descriptor

The generated document that a machine reads when it arrives at a location with nothing else. It names every corpus root in the repository, with the taxonomy identity, the version, the lock hash, the entry points, and each [export profile](#export-profile). It is a [projection](#projection), so it cannot go stale against the roots. Its path is the engine's rather than the taxonomy's, because a reader who must read the taxonomy to find it already knows what it says. See [spec 7](07-distribution-and-federation.md#arriving-at-a-corpus-cold).

### Corpus graph

Typed nodes and typed edges, built in one pass and cached. The graph, not the file tree, is the engine's working representation. Every run rebuilds it, and nothing stores it. See [spec 1](01-conceptual-model.md#the-corpus).

### Correction

An edit in place, made when a document was wrong about the present. It preserves truth. Contrast [succession](#succession), which preserves lineage. To conflate the two destroys the record. See [spec 3](03-authoring-and-lifecycle.md#lifecycle).

### Corrective

The control class that repairs what detection found: auto-fix, generated remediation tasks, and agent-raised change proposals. See [spec 4](04-assurance-model.md#assurance-not-enforcement).

### Counterfactual probe

A pair of runs of one probe, in the `present` [arm](#arm) and the `absent` arm. It is the only evidence that the instruction surface earns its context cost. It is not a probe category. See [spec 5](05-ai-integration.md#probe-categories).

### Coverage

Two related reports. Over obligations, the fraction discharged by severity, with the gap list. Over a run, the documents seen, classified, checked, and skipped, each with a reason. Clean runs report it too. See [spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for).

### `created_by`

The required declaration of who creates each edge, from a closed set: `author`, `scaffold`, `generator`, `hook`, `agent`, `import`. It forces the question of who pays for a link at design time, not after the corpus stops maintaining it. An `import` edge carries the same weight as any other, because its producer is a [correctness root](12-check-layer.md#the-correctness-roots) rather than a rule. See [spec 2](02-taxonomy-model.md#who-creates-each-edge).

### Cue

An optional source-owned attribute on a relation instance, which says why this link, from here. Absent, a reader gets the target's [summary](#summary) instead. See [spec 5](05-ai-integration.md#what-a-cue-may-do-and-where-it-is-served).

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

The measured effect of the instruction surface on agent behavior, taken from the probe suite and reported as an interval. A rule that measurably changes nothing is a candidate for deletion. See [spec 4](04-assurance-model.md#the-adaptive-layer-reports-cost-not-just-coverage).

### Emitter

The generator for one [export](#export) target. Every emitter reads the resolved [lock](#lock) and the graph directly, and no emitter reads another emitter's output. See [spec 6](06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped).

### Engine

The deterministic program that parses the corpus once, builds one graph, and runs every check against it. No LLM sits in its validation path. See [spec 6](06-engine-architecture.md#why-one-engine).

### Escape hatch

A stated, reasoned exemption from a rule, scoped to a file or a block. Voice checking and [suppression](#suppression) both use them. A shelf that collects them is a finding about the shelf, not about the documents. See [spec 3](03-authoring-and-lifecycle.md#voice) and [spec 4](04-assurance-model.md#suppression).

### Evidence basis

One of three honest states for the support behind a decision: `evidenced`, `reconstructed`, or [`unevidenced`](#unevidenced). `reconstructed` is not a soft `evidenced`, and it never promotes silently. See [spec 3](03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two).

### Expectation

The predicate that grades a probe run, drawn from a closed set of forms over the transcript and the produced artifacts. A question that needs a rubric instead is not a probe. See [spec 5](05-ai-integration.md#a-probe-is-a-document-with-a-declared-expectation).

### Explain

`headwater explain` prints why the engine typed a document as it did. It names the shelf that matched, the rule that fired, the declared purpose, and what is consequently required. Classification is never a black box. See [spec 2](02-taxonomy-model.md#kind-resolution).

### Export

A [projection](#projection) of the graph for a consumer outside the engine. The native graph export carries the property graph with no loss. Every interoperability export is lossy, and each one declares a [loss set](#loss-set). It is also the serving boundary, so an [export profile](#export-profile) is where a corpus decides what leaves it. An emitter that cannot carry the [warrant](#warrant) withholds the content rather than shipping it unmarked. No export is canonical for anything. See [spec 6](06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped).

### Export profile

An entry under `projections` that names an audience, an emitter target, an output path, a filter over facet values, and a [tombstone grain](#tombstone-grain). It is not a fourteenth declaration. A corpus with one audience declares one profile and no filter. See [spec 6](06-engine-architecture.md#an-export-profile-carries-a-filter).

### External anchor

A node for something outside the corpus that documents point at: a code path, a work item, a service, a URL. It carries an identity, and never a purpose or a lifecycle. Resolution has three outcomes: resolved, unresolved, and [withheld](#withholding). See [spec 1](01-conceptual-model.md#external-anchor).

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

The date when a human last confirmed that a document agrees with reality. It is deliberately not the last-edited date, which says nothing about truth. [Asserted content](#asserted-content) carries no value here, because nobody confirmed it. See [spec 3](03-authoring-and-lifecycle.md#freshness-and-staleness).

### Front matter

The YAML block that carries the facets which a kind requires. It is the machine's only guaranteed read of a document, so the schema is strict about it. See [spec 3](03-authoring-and-lifecycle.md#front-matter-is-the-contract).

### Gap

A [disposition](#disposition). No control discharges the obligation yet, it is wanted, and an owner tracks it. The evidence sense of this word is now [`unevidenced`](#unevidenced). See [spec 4](04-assurance-model.md#every-obligation-has-exactly-one-disposition).

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

### Interview

The first-run surface. `headwater init` composes a [bundle](#bundle) selection from answers about the corpus, and emits an overlay. It is declared as package data, and it is `headwater infer` with a second evidence source. See [spec 7](07-distribution-and-federation.md#the-interview).

### Kind

What a document *is*. A kind carries a purpose, a section contract, a facet schema, voice and lifecycle regimes, an identifier scheme, and a template. Its permitted relations are derived from relation endpoints and never declared on the kind. See [spec 1](01-conceptual-model.md#kind).

### Kind resolution

The four-step derivation of a kind from shelf match, shelf homogeneity, discriminator facet, and path-pattern refinement. It is deterministic and explainable. See [spec 2](02-taxonomy-model.md#kind-resolution).

### Lifecycle regime

A state machine over the state facet: states, legal transitions, terminal states, and what each state implies. It is declared in the taxonomy and interpreted by the engine. See [spec 3](03-authoring-and-lifecycle.md#lifecycle).

### Lifecycle-sensitive

A property of a relation and of its family. A live document may not depend on a terminal one through a lifecycle-sensitive relation. See [spec 3](03-authoring-and-lifecycle.md#lifecycle).

### Lock

The resolved taxonomy, written with a content hash and committed. Everything downstream reads the lock and never the sources, so a check result depends on a hash that a reviewer can see in a diff. See [spec 6](06-engine-architecture.md#pipeline).

### Loss set

What an [emitter](#emitter)'s target vocabulary cannot carry: node classes, edge classes, and attributes, each with a reason. The [projection census](#projection-census) is what proves the declaration complete. See [spec 6](06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped).

### Maintainer subagent

A context-isolated agent that owns documentation upkeep across a change, with its own bounded instruction subset. It does not inflate the always-on prompt of every session. See [spec 5](05-ai-integration.md#agent-surfaces).

### Mapping

A declared SKOS correspondence between two taxonomies: `exactMatch`, `closeMatch`, `broadMatch`, `narrowMatch`, `relatedMatch`. The [aggregator](#aggregator) tier owns them, because peers do not map to peers at scale. See [spec 2](02-taxonomy-model.md#mapping-between-taxonomies).

### MCP server

The agent-facing surface of the engine library. Its tools fall in three classes: query, working-tree write, and landed write. The first two ship, and the third never does, because acceptance is a human act. It applies no filter to a corpus that its reader already holds. See [spec 5](05-ai-integration.md#what-the-server-may-do-and-the-axis-that-decides-it).

### Meta-schema

The formal schema of the taxonomy language, published and versioned with the engine. `taxonomy validate` checks a taxonomy against it, and the engine never applies a taxonomy that fails. See [spec 2](02-taxonomy-model.md#the-meta-schema).

### Migration payload

The machine-readable steps that ship with a major version, split into what the engine applies mechanically and what needs human or agent judgment. It covers the consumer's overlay as well as the corpus, and carries the rename map that an overlay rewrite reads. See [spec 7](07-distribution-and-federation.md#upgrading).

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

The declared customization of a base taxonomy, through `override`, `add`, and `remove`. An adopter never edits a base taxonomy. Merge semantics are strict, total, and order-independent, and each operation asserts a precondition about the base that an upgrade can falsify. See [spec 2](02-taxonomy-model.md#customization-by-composition).

### Participation expectation

A kind's declaration that its documents, in a given state, acquire a named relation to another kind within a window. It is the only construct that finds a document which should exist and does not. Detective only. See [spec 2](02-taxonomy-model.md#participation-expectations).

### Pin

The taxonomy version that a consumer holds. A scheduled check compares it against the publisher's latest release. See [spec 7](07-distribution-and-federation.md#upstream-awareness).

### Pinned export

A source corpus's [export](#export), named by identity, content hash, and location, and committed where a harvesting tier can read it. An export that the tier cannot read is a finding that names the pin. See [spec 7](07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it).

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

### Probe

An authored document that declares a category, a task statement, and an [expectation](#expectation). It is the unit that the probe suite runs. See [spec 5](05-ai-integration.md#a-probe-is-a-document-with-a-declared-expectation).

### Probe result

The document that grades one run: a projection over the [probe transcript](#probe-transcript), the declared expectations and the grader version. Its [warrant](#warrant) is `regenerated`, and it reports an interval rather than a point. See [spec 5](05-ai-integration.md#a-run-produces-a-snapshot-and-a-document).

### Probe suite

The probes that a corpus declares, run against it in a controlled session and graded from the tool-call transcript rather than from the model's self-report. It runs in two tiers, regression and [campaign](#campaign). See [spec 5](05-ai-integration.md#measuring-whether-any-of-this-works).

### Probe transcript

The committed snapshot that one run emits: the ordered tool-call events, the produced artifacts, the final answer, and the [run identity](#run-identity). It holds no model prose, which is what keeps the grader out of the system under test. See [spec 5](05-ai-integration.md#a-run-produces-a-snapshot-and-a-document).

### Profile

A named overlay that the publisher ships for a repository archetype, which removes what that archetype does not have. It is a use of the overlay mechanism, not a mechanism of its own. Compare a [bundle](#bundle), which adds. See [spec 7](07-distribution-and-federation.md#profiles-are-publisher-overlays).

### Projection

A derived artifact computed from the graph: a shelf index, site navigation, an agent rule file, a graph export, a [transcription](#transcription), the [corpus descriptor](#corpus-descriptor). They are generated, checked against regeneration, and declared in the schema. A projection that leaves the repository may carry a filter, and it then says so. See [spec 1](01-conceptual-model.md#projections).

### Projection census

The account that an [export](#export) run gives of itself. Every node and every edge is present in the output, or covered by a declared [loss set](#loss-set) reason. An uncovered omission fails the run. It is the [census](#census) doctrine, one layer out. See [spec 6](06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped).

### Promotion

The evidence-bound move of a check from advisory to blocking. It needs an observation window, a false-positive rate under a declared threshold, a mechanical remediation path, and an adjudicated sample. Demotion is the inverse. A control with one unrecoverable error class does not walk this path, and a [withholding](#withholding) rule is the one instance. See [spec 4](04-assurance-model.md#promotion-advisory-to-blocking).

### Provenance

The record of who drafted a document, by which activity, who accepted it, and what [warrant](#warrant) it carries. An agent may draft. Acceptance is a human act, and the record names the human. The block's shape is the engine's rather than a taxonomy choice, because `accepted_by` enforces a boundary. PROV supplies the derivation half and has no vocabulary for the endorsement half. See [spec 3](03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed).

### Publisher

The organization that releases a taxonomy package, its doctrine, and its migrations, and that measures compatibility against its own reference corpora. See [spec 7](07-distribution-and-federation.md#publishing).

### Purpose

The reader intent that a kind serves, declared once at the taxonomy level. A kind without a purpose fails validation, because purpose plus form is what makes a kind a genre rather than a shape. See [spec 2](02-taxonomy-model.md#purpose-is-declared-not-implied).

### Read set

The inputs that a run's results depended on. It holds the content hash of every document and edge that an instance read, plus the lock hash, the check versions, and the injected values. A run reports it, so a later holder of a merge result can decide whether the verdict still applies. See [spec 12](12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict).

### Reading precedence

Which of two linked documents governs the reading. It is derived from nuclearity, succession, and the governance family, and never declared. See [spec 2](02-taxonomy-model.md#reading-precedence-is-derived).

### Reciprocity

Whether the target of a relation must acknowledge the source, and with which inverse relation. Two halves that disagree are a finding, and the engine prefers neither side. See [spec 1](01-conceptual-model.md#relation).

### Regime

A reusable, named bundle of rules that a kind opts into, so that a rule is declared once and referenced many times. Two survive: [voice](#voice-regime) and [lifecycle](#lifecycle-regime). See [spec 1](01-conceptual-model.md#regimes).

### Register projection

The generated view of which controls discharge which obligations, with every disposition, control health, suppressions, and waivers. It is engine-defined, non-optional, and never authored. See [spec 4](04-assurance-model.md#every-obligation-has-exactly-one-disposition).

### Relation

A typed, named, directed link between two documents, or from a document to an anchor. It is declared under the `relations:` key of front matter and nowhere else. The type carries a family, nuclearity, endpoints, cardinality, reciprocity, lifecycle interaction, and the [instance attributes](#relation-instance-attribute) that its edges may take. An instance is identified by the source identifier, the relation name, and the normalized target. See [spec 1](01-conceptual-model.md#the-authored-form).

### Relation instance attribute

Data that one edge carries, as distinct from data that the relation type declares for all of them. A relation type names each attribute, its value space, and its owning end, which is `source`, `target`, or `edge`. An undeclared attribute is a finding, and an attribute value is never a reference. See [spec 2](02-taxonomy-model.md#instance-attributes-and-which-end-owns-each-one).

### Remediation

The instruction that every finding carries. A finding that cannot say what to do next is noise. See [spec 4](04-assurance-model.md#findings).

### Resolve

`headwater taxonomy resolve` fetches the base, merges the overlays, validates the result, and writes the [lock](#lock). See [spec 7](07-distribution-and-federation.md#consuming).

### Retired term

A term that the corpus no longer uses, declared in the language regime with a required reason and an optional replacement. With a replacement the fix is a substitution. Without one the finding carries prose, because a retired framing has nothing to substitute. See [spec 2](02-taxonomy-model.md#the-language-regime-carries-the-terms-that-the-corpus-retired).

### Rigidity

A kind is rigid, because a document cannot lose it and stay the same document. A state is anti-rigid. So the validator rejects a kind that collides with a state value, or that is named with a bare phase adjective. See [spec 2](02-taxonomy-model.md#kinds-are-rigid-states-are-not).

### Routing

The intent-time match of a task description against declared purposes, before any match on text. It returns a ranked, budget-capped set of [pointers](#pointer), and it fails open below its confidence gate. See [spec 5](05-ai-integration.md#intent-time-routing).

### Run identity

What a probe run records about itself. It holds the served model version, the corpus tree, the lock hash, the probe selection, the seed, the harness version, the [arm](#arm), and the time. A model name alone is not a pin. See [spec 5](05-ai-integration.md#a-run-produces-a-snapshot-and-a-document).

### Sampler

The non-deterministic path that carries the [coherence sweep](#coherence-sweep) and the [probe suite](#probe-suite). It shares the finding shape and the reporting pipeline, and it never enters the cached, reproducible path. See [spec 12](12-check-layer.md#where-the-llm-coherence-sweep-fits).

### Satellite

The end of a nucleus–satellite relation that cannot stand alone. It inherits declared facets from its nucleus, and a context budget drops it before the nucleus. See [spec 2](02-taxonomy-model.md#nuclearity).

### Scaffolding

`headwater new` creates a document with correct placement, front matter, sections, and identifier, and prints the relations that the document is expected to declare. It also proposes an edge that the taxonomy assigns to a scaffold, and writes the reciprocal half into the document at the far end. What it writes is authored rather than generated, so every check reads it. See [spec 3](03-authoring-and-lifecycle.md#templates-and-scaffolding).

### Scent

The proximal cue that predicts distal value, and the thing that routing trades in. It sits in the `summary` facet at a routing result, and in an optional [cue](#cue) at a traversal. Each measure of it names the alternatives it compares against. See [spec 5](05-ai-integration.md#scent-is-the-thing-being-engineered).

### Scope

What a check declares that it needs to see. The values are `Document`, `Edge`, `Neighbourhood`, `Shelf`, and `Corpus`, plus flags for the body, the clock, and the prior version. See [spec 12](12-check-layer.md#scope--the-declaration-everything-else-rests-on).

### Scoped view

The only thing that a check can read. The engine enforces it, and that enforcement is what makes cache keys sound and change-scoped runs exact. See [spec 12](12-check-layer.md#scope--the-declaration-everything-else-rests-on).

### Section contract

The headings that a kind must or may have. See [spec 1](01-conceptual-model.md#kind).

### Semantic conflict

Two changes that are each valid against the merge base, whose merge is invalid. Git reports nothing, because the conflict is not textual. Databases call the same anomaly write skew. See [spec 4](04-assurance-model.md#a-verdict-is-about-one-state-of-the-corpus).

### Severity

The check's report of how bad a finding is. Whether that severity blocks is the control's business, so promotion needs no code change. See [spec 12](12-check-layer.md#severity-is-the-checks-posture-is-the-controls).

### Shelf

A named region of the corpus, in practice a directory or glob, that carries a purpose. Placement is the loudest signal that a document sends, so the shelf is the primary classification axis. See [spec 1](01-conceptual-model.md#shelf).

### Shift ratio

The share of edges that are [focus shifts](#focus-shift), reported as a corpus-health metric. It stays permanently advisory, because a gate on it causes link padding and destroys the measure. See [spec 4](04-assurance-model.md#measuring-coherence-where-we-can-continuity-across-links).

### Size budget

The declared context limit on an agent-facing kind or projection. It is mandatory, and an agent-facing projection without an applicable budget fails validation. See [spec 5](05-ai-integration.md#read-time-rule-loading).

### Snapshot pin

A committed copy of an external system of record, with its fetch time and the upstream identity and revision of every item in it. An [anchor resolver](#anchor-resolver) reads it, so check time stays offline. It is the second of the three instances of one pin pattern. See [spec 7](07-distribution-and-federation.md#upstream-awareness).

### Staleness

The condition of a document past its freshness threshold, weighted by drift risk rather than by calendar time alone. Detective, never blocking. See [spec 3](03-authoring-and-lifecycle.md#freshness-and-staleness).

### Starter kit

The [base package](#base-package), a named [bundle](#bundle) selection over it, and the doctrine prose that explains the selection. It is what an adopter gets who answers nothing. See [spec 7](07-distribution-and-federation.md#the-starter-kit-is-a-selection).

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

The declared structure of a corpus, in thirteen declarations: purposes, facets, regimes, relations, anchors, shelves, kinds, identifier schemes, core, mappings, projections, obligations, and controls. It is data, not code. See [spec 2](02-taxonomy-model.md#the-thirteen-declarations).

### Taxonomy package

The released, versioned unit that a publisher ships: taxonomy, doctrine, templates, plugins, profiles, bundles, an interview, and migrations. The taxonomy is a package, not a copy. See [spec 7](07-distribution-and-federation.md#publishing).

### TBox

The terminology layer of the knowledge base: which kinds of thing exist, which relations may hold, and which values are legal. The taxonomy is the TBox. See [spec 1](01-conceptual-model.md#two-layers-terminology-and-assertions).

### Template

The starting document for a kind, generated from the kind declaration. A template that drifted from its kind is impossible by construction. See [spec 3](03-authoring-and-lifecycle.md#templates-and-scaffolding).

### Tier

One layer of a federation: the generic method, a divisional taxonomy, or a repository overlay. See [spec 7](07-distribution-and-federation.md#federation).

### Tombstone grain

What a filtered [export profile](#export-profile) tells a reader about what it withheld. `counted` gives the number by declared reason, and `sealed` gives only the fact of the filter. No profile may present a filtered view as total. See [spec 6](06-engine-architecture.md#an-export-profile-carries-a-filter).

### Transcription

A [projection](#projection) that copies text out of a [snapshot pin](#snapshot-pin), byte for byte. Its [warrant](#warrant) is `transcribed`, which W3C PROV calls `prov:Quotation`. Truth stays upstream, and `generate --check` proves the copy. See [Q19](09-decisions.md#q19--inbound-integration-an-external-system-of-record).

### Unevidenced

An [evidence basis](#evidence-basis). No evidence exists behind a decision, and none is claimed. Whether the register that collects these is the obligation [gap](#gap) register, or a second register of the same name, is unsettled. See [spec 3](03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two).

### Unverifiable

A [disposition](#disposition). No mechanism can discharge the obligation, the organization accepts that, and the reasoning is recorded. See [spec 4](04-assurance-model.md#every-obligation-has-exactly-one-disposition).

### Upstream awareness

The scheduled comparison of the [pin](#pin) against the publisher's latest release. It raises a draft change proposal, not a notification that nobody acts on. See [spec 7](07-distribution-and-federation.md#upstream-awareness).

### Validate

`headwater taxonomy validate` decides the schema alone: referential integrity, determinism, purpose completeness, rigidity, edge provenance, overlay confluence, and core satisfiability. It needs no documents, and it gates everything. See [spec 6](06-engine-architecture.md#taxonomy-validate-versus-taxonomy-audit).

### Vocabulary

A named value set that facets reference. It is authoring syntax, validated as part of `facets`, and not a concept that anyone must learn first. See [spec 2](02-taxonomy-model.md#the-thirteen-declarations).

### Voice regime

The voice that a kind writes in, with the machine-checkable part of it: declarative, narrative, or unconstrained. Enforcement is lexical, over author-owned text alone, and every escape hatch states a reason from the [suppression](#suppression) set. A category blocks only when its remediation is mechanical, so a category that needs a rewrite is permanently advisory. See [spec 3](03-authoring-and-lifecycle.md#voice).

### Volatility

A facet's declaration of permanence. A `mutable` facet may not appear in an identifier, a path, or a shelf pattern. See [spec 2](02-taxonomy-model.md#facet-acceptance-tests).

### Waiver

A consumer's deliberate deviation from a conformance rule, with a rule name, a reason, an owner, and an expiry. Deviation is fine. Invisible deviation is not. See [spec 7](07-distribution-and-federation.md#waivers).

### Warrant

What the corpus can point at to defend that a document is what it claims to be. One of `accepted`, `regenerated`, `transcribed`, or `asserted`. The set is closed, the engine owns it, and an absent value is a finding rather than a default. See [spec 1](01-conceptual-model.md#warrant).

### Window

The time bound on a [participation expectation](#participation-expectation), measured from a declared [origin](#origin). An expectation with no window is a wish. See [spec 2](02-taxonomy-model.md#participation-expectations).

### Withholding

The removal of a document from an [export profile](#export-profile) by its declared filter. It is a [loss set](#loss-set) reason, so the [projection census](#projection-census) accounts for it. The rule that performs it never ships advisory, and no suppression or waiver reaches it, because its false negative is a disclosure that nothing recalls. See [spec 6](06-engine-architecture.md#an-export-profile-carries-a-filter).

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
| [Projection](#projection) | Authored document | A projection is regenerable, and CI checks it against regeneration. [Asserted content](#asserted-content) is neither, and it carries no warrant at all |
| [Warrant](#warrant) | Agency | An agent may draft a document that a human accepts, and that document is `accepted`. `asserted` marks what nobody accepted, whoever wrote it |
| [Transcription](#transcription) | [Asserted content](#asserted-content) | One is a function of a pin that the repository holds, so a run proves it. The other is a function of nothing that the corpus can check |
| [Withholding](#withholding) | An unresolved anchor | One is somebody's declared decision, reported at a declared grain. The other is a defect. Counted as one class, the defects disappear |
| [Loss set](#loss-set) | [Withholding](#withholding) | A loss is what a target vocabulary cannot carry. A withholding is what a corpus chose not to send. Both are census reasons, and only one is a policy |

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
| [Warrant](#warrant) | Toulmin, *The Uses of Argument* (1958) | What licenses the step from data to a claim, which here is the step from what a document says to what the corpus asserts |
| PROV | W3C | Lineage and provenance alignment for `derives_from`, `supersedes`, and generated artifacts. `prov:Quotation` names the `transcribed` warrant |
| SKOS [mapping](#mapping) relations | W3C | `exactMatch`, `closeMatch`, `broadMatch`, `narrowMatch`, `relatedMatch` between taxonomies |
| [Confluence](#confluence), overlay operations | Delta-oriented programming (Schaefer et al.) | Order independence of overlays, checked statically before application |
| [Capture cost](#capture-cost) | IBIS, gIBIS, QOC, and the traceability literature | The failure that killed fifty years of design-rationale tools, now a tracked metric |
| RFC 2119 | IETF | The normative keyword set that voice checking assumes by default |
| LinkML, SHACL | The [evaluations](../evaluations/) | Export targets for Shape and Graph checks. Emitted, never authored, and each one waits for a named consumer ([spec 13](13-open-obligations.md#what-waits-on-a-first-adopter)) |
| SARIF | OASIS | One of the finding output formats |
| MCP | Model Context Protocol | The agent-facing surface of the engine library |

## Terms that used to collide

An index of every term is what makes a name collision visible, so this page found four. All four are now resolved, and the [terminology decisions table](../reviews/ste-editorial-pass-findings.md) records what each one was and why the rename went the way it did. The rule that settled them: rename the sense that is not embedded in a schema key, a CLI verb, or a generated artifact name.

- **register** — the voice sense is now *voice*. The word is left to the [register projection](#register-projection) and the registers that it generates.
- **gap** — the evidence sense is now [`unevidenced`](#unevidenced). The word is left to the obligation [disposition](#disposition).
- **audit** and **conformance** — spec 4's conformance audit is now the [accuracy audit](#accuracy-audit). `taxonomy audit` keeps its CLI verb, and [conformance](#conformance) keeps the adoption sense that owns `headwater conformance`.

One question survives the renames. Spec 3 says that an `unevidenced` document appears in the gap register, and spec 4 says that an uncovered obligation appears in the gap register. Whether that is one artifact with two entry classes, or two artifacts that share a name, is unsettled and recorded.

One further usage is deliberate rather than accidental. Routing **fails open** when it stays silent below its confidence gate. Conventional security usage would call that failing closed. Design principle 7 in [spec 0](00-vision-and-scope.md#design-principles) fixes the project meaning.
