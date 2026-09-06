---
id: HW-SPEC-conceptual-model
status: current
status_since: 2026-08-01
last_verified: 2026-08-11
summary: The vocabulary that the rest of the specification uses, from corpus and shelf to warrant, projection, and the TBox and ABox split.
doc_type: design_spec
sequence: 1
title: "Conceptual model"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: [claude-fable-5, claude-opus-5]
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cites_evidence:
    - HW-EVAL-adjacent-work
    - HW-EVAL-graph-export-and-federation
    - HW-EVAL-relation-storage
    - HW-EVAL-the-serving-boundary
    - HW-EVAL-warrant-and-adjudication
---

# 1 — Conceptual model

This document gives the vocabulary that the rest of the specification uses. Each term is a named concept in the schema and in the engine. Where they differ, this document says so.

The [glossary](glossary.md) indexes every term in the specification, including the ones that later documents introduce. It gives one line for each and points back to the section that owns the definition.

## Two layers: terminology and assertions

The split that this model rests on is the one that description logic already names. A knowledge base has a **TBox** and an **ABox**. The **TBox** is the terminology: which kinds of thing exist, which relations may hold between them, and which values are legal. The **ABox** is the assertions: the actual individuals and the relations actually asserted.

| Headwater | Description logic |
|---|---|
| Taxonomy | TBox |
| Resolved taxonomy lock | Compiled TBox |
| Corpus — documents and their declared edges | ABox |
| `taxonomy validate` | TBox-internal consistency |
| `check`, `taxonomy audit` | ABox against TBox |

The use of the standard names is not decoration. It is why the validation of a taxonomy and the check of a corpus are two different operations, not two halves of one operation ([spec 6](06-engine-architecture.md#taxonomy-validate-versus-taxonomy-audit)).

**Where the ABox stops.** Headwater's assertions are about *documents* — this document exists, is of this kind, governs that code path, supersedes that other document. They are not about the claims *inside* the prose. "The service returns 404 on a missing key" is a sentence. The system knows the document that contains it and what that document governs, but not what it asserts about the world. Reasoning stops at the document boundary. A promise of semantic consistency checking beyond that boundary is a promise that we cannot keep.

## The corpus

A **corpus** is the set of documents that one taxonomy governs, rooted at one directory (usually `docs/`). A corpus knows its taxonomy by reference (id plus version). Thus the system never requires a document to state what kind of thing it is more than once.

**One taxonomy and one root make a corpus, and a repository holds one or more.** An earlier draft said that a repository has exactly one corpus, which named the wrong container. A monorepo with several independent documentation sets is not one corpus under strain. It is several corpora that share a working tree, and each one carries its own lock.

Two rules keep that cheap, and both reuse machinery that exists. A path resolves to exactly one corpus. The rule is the one that shelf patterns already obey: the most specific root wins, and a tie is a schema-validation error. Identifiers carry a namespace from the moment of minting ([spec 3](03-authoring-and-lifecycle.md#identifiers)), so two corpora in one tree cannot collide ([Q9](09-decisions.md#q9--multi-repository-corpora)).

The engine loads the corpus into a **corpus graph**: typed nodes with typed edges. The engine builds the graph in one pass and caches it. Every downstream operation reuses it. The graph — not the file tree — is the engine's working representation.

**The graph is rebuilt, and nothing stores it.** Every run derives the graph from the Markdown, and the cache only makes that derivation cheap. A cache that can change a verdict is a store under another name, and the engine tests the difference ([spec 6](06-engine-architecture.md#nothing-stores-the-graph)). Every artifact that leaves the graph is a projection, and each one declares what it could not carry ([Q6](09-decisions.md#q6--where-the-corpus-graph-lives-at-rest)).

## Nodes

### Document

A file in the corpus. Markdown body, YAML front matter. Every document resolves to exactly one **kind** (below). A document that resolves to no kind, or to more than one, is a defect. The engine reports the defect and does not guess.

### Shelf

A named region of the corpus — in practice a directory or glob — that carries a purpose. `specifications/`, `decisions/`, `standards/` are shelves. A shelf is the **primary classification axis**. Placement is the loudest signal that a document sends. Thus the schema treats placement as authoritative, and metadata never contradicts it.

A shelf is **homogeneous** when every document on it is of one kind, and **heterogeneous** when it legitimately holds several. This distinction controls if documents on the shelf must carry a kind discriminator (see [taxonomy model](02-taxonomy-model.md)).

### Kind

What a document *is*: a decision record, a component specification, a runbook, an incident record, a standard. A kind carries:

- the **purpose** that it serves — the reader intent that it exists to satisfy, declared, not implied. A kind without one is invalid: purpose plus form is what makes a kind a genre rather than a shape.
- the **section contract** — headings that the document must or may have.
- the **facet schema** — which metadata it must, may, and must not carry.
- the **voice regime** and **lifecycle regime** that it obeys.
- its **identifier scheme**, if it mints one.
- the **relations** that it may (or must) participate in.
- its **template**.

The engine resolves a kind from a document's position (shelf + path pattern) and, on heterogeneous shelves, from a discriminator facet. Resolution is deterministic and explainable: the engine can always say *why* it typed a document as it did.

### External anchor

A node that represents something outside the corpus that documents point at. Examples: a source path, a component, a work item, a service, a released artifact, a URL. Anchors turn otherwise-dangling references into typed nodes. The engine can then check them, traverse them, and answer "what governs this code path?" without a special case for every kind of pointer.

An anchor carries an identifier, a name, and an owner — never a purpose or a lifecycle. Every substantive claim about the thing itself lives in a document. Anchor kinds are declared like everything else: the taxonomy's `anchors` declaration names each type and the single resolver that owns it. Thus a relation endpoint is always either a declared kind or a declared anchor kind — never a bare string. Anchor identity is declared, not guessed. Anchor strings normalize before comparison, so two spellings of one target are one node. An anchor that no resolver claims is a finding ([spec 2](02-taxonomy-model.md#behavior-at-the-limits)).

Resolution has three outcomes and not two. An anchor resolves, or it fails to resolve, or its target sits behind a declared withholding. The third outcome is **withheld**, and it is never reported as the second. A withheld target is a fact about a filter that somebody declared. An unresolved target is a defect. To report them as one class teaches a reader to ignore the class that carries the defects ([Q17](09-decisions.md#q17--governed-access-and-the-solution-layer)).

## Edges

### Relation

A **typed, named, directed link** between documents (or from a document to an anchor), declared in front matter. Relations are the system's connective tissue and its main source of checkable structure. A relation type declares:

- **family** — one of six fixed families (`succession`, `derivation`, `governance`, `evidence`, `composition`, `association`). The family supplies default semantics.
- **nuclearity** — if both ends stand alone (multinuclear), or if one end supports the other and cannot stand without it (nucleus–satellite). For nucleus–satellite relations, the declaration says which end is the nucleus. The family supplies the default. A relation that contradicts it says so explicitly, and `taxonomy audit` reports the override.
- **endpoints** — which kinds may sit at each end.
- **cardinality** — how many are legal, and if one is required.
- **reciprocity** — if the target must acknowledge the source, and with which inverse relation.
- **directionality constraints** — for example, a reference may only point at the same abstraction tier or higher.
- **lifecycle interaction** — for example, a live document may not depend on a superseded one.
- **instance attributes** — the data that one instance of this relation may carry, and which end owns each one.

Because relation semantics live in the schema, the checks over them are generic. A new relation type added to a taxonomy adds validation for free. It does not add a linter.

#### The authored form

Every relation instance is declared under one `relations:` key in front matter ([Q4](09-decisions.md#q4--relation-storage)). A relation name and a facet name come from separate declarations. Under a flat front matter, a taxonomy that declares both a facet and a relation called `owns` is unresolvable. One block also gives `check --fix` one region to rewrite.

An entry is a target reference, or a mapping with `to:` and instance attributes. The scalar is sugar for a mapping whose only key is `to`, and both forms produce the same edge.

```yaml
relations:
  supersedes: ACME-DR-0031
  conflicts_with: ACME-DR-0044
  cites:
    - STD-ACME-0007
    - to: STD-ACME-0012
      cue: the retry budget, not the error taxonomy
```

A target is an identifier and never a path. Document identifiers are namespaced, globally resolvable, and never reused ([spec 3](03-authoring-and-lifecycle.md#identifiers)), and a path is a location that moves. An anchor target is the anchor string, which its resolver normalizes.

An edge is identified by the source identifier, the relation name, and the normalized target. List order therefore carries no meaning, and a repeated triple in one document is an error. That identity is what the `Edge` scope of [spec 12](12-check-layer.md#scope--the-declaration-everything-else-rests-on) keys on.

An instance attribute takes a facet's value space, and it is never a reference. A connection is a relation, and the rule that removed reference-valued facets reaches edges before anyone finds the loophole ([spec 2](02-taxonomy-model.md#instance-attributes-and-which-end-owns-each-one)).

#### Prose links are not relations

A prose link is evidence that an author found a reference worth making at one point in a text. A relation asserts something about two documents that holds wherever a reader meets them. The engine extracts every prose link and checks that it resolves. No syntax promotes one into a relation. Two authoring locations for one edge leave three questions unanswered: which location wins, which span a finding anchors to, and what a fix writes.

The cost of that ruling is a reference written twice. [Spec 4](04-assurance-model.md#declaration-moves-the-boundary) absorbs most of it: an extracted link with no declared relation is an advisory finding, and the fix writes the declaration.

Relation types that an adopter is likely to want (all shipped with the default package, none hard-coded in the engine): `supersedes` / `superseded_by`, `derives_from`, `governs`, `verifies`, `implements`, `cites`, `owns`, `refines`, `conflicts_with`. Shipped does not mean enabled: the default *enables* a minimal four. The rest are complete declarations that an overlay pulls in by reference ([spec 2](02-taxonomy-model.md#the-decision-relation-vocabulary)).

A kind may also declare a participation **expectation**. The expectation says that its documents, in a given state, get a named relation within a window measured from a declared origin date ([spec 2](02-taxonomy-model.md#participation-expectations)). Expectations are the only construct that finds a document that **should exist and does not**. Every other check validates artifacts that are present. They model what genre theory calls a *genre system*: proposal → decision → specification → evidence.

The end that governs the *reading* of two linked documents is derived, never declared. On a nucleus–satellite relation the nucleus governs: a document that cannot stand alone cannot govern the reading of the one that it depends on. On succession the successor governs — that is what succession means. Other multinuclear relations carry no reading order, and none needed one to date. A routing or projection outcome that requires one is the evidence that reopens this ([spec 2](02-taxonomy-model.md#reading-precedence-is-derived)).

### Facet

A named metadata dimension attached to documents. A facet declares its value space (free scalar, date, enum with a controlled vocabulary, or list of any of those), where it applies, if it is required, and how strictly it is enforced.

A facet value is never a reference to another node. That was legal in an earlier draft, and it became a second, ungoverned edge mechanism. A reference-valued facet asserts exactly what a relation asserts — this document is connected to that node. But it carries no family, no nuclearity, no reciprocity, no lifecycle interaction, and no `created_by`, and it generates none of the graph checks. Under that draft, the same fact got two levels of governance, decided only by which syntax a taxonomy author used. The ungoverned syntax is the cheaper one, so under deadline pressure it wins.

A connection is a relation. An opaque external identifier (a ticket number, a framework control id) is a scalar. External anchors exist to resolve such an identifier into the graph.

Facets are how a taxonomy expresses everything that the directory cannot: lifecycle state, freshness, ownership, scope, audience, provenance, confidentiality, kind discriminators on heterogeneous shelves. The vocabulary of an enum facet is part of the schema, so an extension to it is a deliberate, reviewable, versioned change.

Six facet roles are engine-significant, and the registry is closed ([spec 2](02-taxonomy-model.md#the-meta-schema) owns it):

- **`state`** — the lifecycle regime interprets it.
- **`state_entered`** — each transition stamps it. It is the origin for state-conditional participation windows.
- **`created`** — set at scaffold time. It is the origin for windows with no state condition.
- **`freshness`** — staleness detection interprets it.
- **`scent`** — what routing, indexes, and agent-facing pointers show (the `summary` facet in the default taxonomy).
- **`name`** — the text that a projection writes as a heading or as a row (the `title` facet in the default taxonomy). A projection may not read a facet by the name of the facet ([spec 6](06-engine-architecture.md#projections)).

The facet that plays each role is declared, not assumed. A corpus may call its state facet `status`, `stage`, or `état`. But the roles themselves come only from the registry.

## Regimes

A **regime** is a reusable, named bundle of rules that a kind opts into. Regimes exist so that rules are declared once and referenced many times.

- **Voice regime** — the voice in which a document is written, with the machine-checkable part of it: e.g. *declarative present-state* (no future intent, no narration of change, no phased-rollout language), *narrative* (time-boxed exploration, changelogs), or *unconstrained*.
- **Lifecycle regime** — a state machine over the state facet: states, legal transitions, terminal states, and what each state implies (e.g. superseded documents are retained and delinked from live dependency paths).

An earlier draft had two more regimes, and each was one parameter in a wrapper. **Freshness policy** — the staleness threshold, drift weighting, and posture — lives on the freshness facet itself, and it is mandatory. A facet that carries the freshness role without applicable policy is invalid. **Size budgets** live on the agent-facing kinds and projections that they meter, and they are mandatory there. An agent-facing projection without an applicable budget is invalid. The protections moved to their enforcement points. Only the wrappers were deleted.

## Obligations and controls

### Obligation

A statement that the corpus commits to, expressed as an **invariant** with a stable identifier. "Behavior-changing code updates its specification in the same change." "Every live decision record is reachable from what it constrains." Obligations are data in the corpus, not prose in a document.

### Control

A mechanism that discharges an obligation: an engine check, a CI job, a git hook, an agent behavior, a scheduled scan, a human audit. A control declares what it verifies, when it runs, and its **posture** — advisory, blocking, or detective.

These are the section's only two concepts. Their binding is visible in the **register projection**: the generated, checked view of which controls discharge which obligations. The view also gives the explicit disposition of every obligation with none — *gap* (tracked, wanted) or *unverifiable* (no mechanism can exist, accepted) — and control health, suppressions, and waivers. Every obligation carries exactly one disposition.

The register projection is a projection like any other (below), distinguished only in that it is engine-defined and non-optional. The binding lives on the control (`discharges:`), and the disposition lives on the obligation. The view is never authored, and coverage claims are generated from it, never asserted in prose.

See [assurance model](04-assurance-model.md).

## Projections

A **projection** is a derived artifact computed from the graph. Examples: a shelf index, a decision-lineage summary, a component-to-specification matrix, site navigation, an agent instruction file, a JSON export of the graph. Projections are:

- **generated** — never hand-authored.
- **checked** — CI fails when a committed projection differs from a regenerated one.
- **declared** — the schema names them, so a taxonomy can add projections without engine changes, within the set of projection kinds that the engine implements.

**A projection may be filtered, and a filtered projection says so.** An [export profile](06-engine-architecture.md#an-export-profile-carries-a-filter) names an audience and a filter over facet values. What it carries and what it withholds partition the corpus. A withholding is a loss with a declared reason, so the projection census reports it like any other loss. No profile may produce a view that presents itself as total ([Q17](09-decisions.md#q17--governed-access-and-the-solution-layer)).

**The corpus descriptor is a projection too.** A machine that arrives at a location reads it to learn which corpora live there, what taxonomy governs each one, and where to start ([spec 7](07-distribution-and-federation.md#arriving-at-a-corpus-cold)). The engine generates it from the roots, holds it to regeneration, and filters it like any other served artifact ([Q14](09-decisions.md#q14--discovery-surface)). Like the register projection, it is engine-defined and non-optional, and its path is the engine's rather than the taxonomy's. A reader who must consult the taxonomy to find the descriptor already knows what it would say.

## Warrant

A **warrant** is the mechanism by which the corpus can defend that a document is what it claims to be. The term comes from Toulmin's model of argument, where the warrant licenses the step from data to a claim. Here it licenses the step from *this document says X* to *the corpus asserts X*.

Every document carries exactly one warrant. The set is closed, the engine owns it, and the value lives in the provenance block ([spec 3](03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed)).

| Warrant | What stands behind the content | What fails loudly |
|---|---|---|
| `accepted` | A named human accepted it | Nothing mechanical. A human re-reads, and the freshness facet records that they did |
| `regenerated` | It is a function of inputs inside the repository | `generate --check`, on every run |
| `transcribed` | It is a copy of a pinned external snapshot, byte for byte | `generate --check` against the pin, and the drift comparison when the pin advances |
| `asserted` | Nothing | Nothing |

**The first three are three mechanisms, and no order runs between them.** A projection is not weaker than an accepted document. It restates one, and it takes that document's standing through the derivation family. The one order that the design needs is that `asserted` sits below the other three, and the difference is not a matter of taste. It is the difference between a defect that some mechanism finds and a defect that no mechanism finds.

**An absent value is a finding, never a default.** A reader cannot tell an unknown value from a missing field. SPDX made the same ruling when it gave a license field both `NONE` and `NOASSERTION` ([HW-EVAL-adjacent-work §P](../evaluations/adjacent-work.md#p--provenance-endorsement-and-the-record-of-a-judgment)).

**Agency and warrant are different fields.** An agent may draft a document that a human then accepts, and the warrant of that document is `accepted`. `asserted` marks content that nobody accepted, whoever wrote it. To read the author as the boundary loses the case that matters, which is unreviewed content at a volume where review does not scale ([Q15](09-decisions.md#q15--a-synthesized-content-tier)).

Two rules constrain `asserted` content, and each one reuses a distinction that this document already makes.

- **No edge may let unwarranted content govern the reading of warranted content.** [Reading precedence](02-taxonomy-model.md#reading-precedence-is-derived) is derived from nuclearity, from succession, and from the governance family. Where the end that governs is `asserted` and the other end is not, the edge is a finding. An edge that ends on an external anchor carries no reading precedence, so an asserted document may declare `governs` against a code path. That edge is what gives the document write-time impact detection.
- **An asserted document does not discharge an evidence obligation.** `evidence_basis: evidenced` obliges an external, auditable artifact, and an asserted document is neither ([spec 3](03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two)).

## How the pieces fit

```
taxonomy schema  ──────────┐
                           ▼
corpus files ──▶ [ parse ] ──▶ corpus graph ──┬──▶ checks    ──▶ findings
                                              ├──▶ queries   ──▶ pointers
                                              ├──▶ projections ──▶ derived files
                                              └──▶ export    ──▶ graph JSON / MCP
```

One parse. One graph. Everything downstream is a function of it.
