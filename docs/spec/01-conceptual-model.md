# 1 — Conceptual model

The vocabulary the rest of the specification uses. Each term is a first-class
concept in the schema and in the engine; where they differ, this document says so.

## Two layers: terminology and assertions

The split this model rests on is the one description logic already names. A
knowledge base has a **TBox** — the terminology: what kinds of thing exist, what
relations may hold between them, what values are permitted — and an **ABox** — the
assertions: the actual individuals and the relations actually asserted.

| docgov | Description logic |
|---|---|
| Taxonomy | TBox |
| Resolved taxonomy lock | Compiled TBox |
| Corpus — documents and their declared edges | ABox |
| `taxonomy validate` | TBox-internal consistency |
| `check`, `taxonomy audit` | ABox against TBox |

Using the standard names is not decoration: it is why validating a taxonomy and
checking a corpus are genuinely different operations rather than two halves of one
([spec 6](06-engine-architecture.md#taxonomy-validate-versus-taxonomy-audit)).

**Where the ABox stops.** docgov's assertions are about *documents* — this document
exists, is of this kind, governs that code path, supersedes that other document. They
are not about the claims *inside* the prose. "The service returns 404 on a missing
key" is a sentence; the system knows the document that contains it and what that
document governs, but not what it asserts about the world. Reasoning stops at the
document boundary, and any promise of semantic consistency checking beyond it would
be a promise we cannot keep.

## The corpus

A **corpus** is the set of documents governed by one taxonomy, rooted at one
directory (conventionally `docs/`) in one repository. A repository has exactly one
corpus. A corpus knows its taxonomy by reference (id plus version), so a document
never has to state what kind of thing it is more than once.

The corpus is loaded into a **corpus graph**: typed nodes with typed edges, built
in a single pass, cached, and reused by every downstream operation. The graph — not
the file tree — is the engine's working representation.

## Nodes

### Document

A file in the corpus. Markdown body, YAML front matter. Every document resolves to
exactly one **kind** (below). A document that resolves to no kind, or to more than
one, is a defect the engine reports rather than guesses at.

### Shelf

A named region of the corpus — in practice a directory or glob — that carries a
purpose. `specifications/`, `decisions/`, `standards/` are shelves. A shelf is the
**primary classification axis**: placement is the loudest signal a document sends,
so the schema treats it as authoritative, and metadata never contradicts it.

A shelf is **homogeneous** when every document on it is of one kind, and
**heterogeneous** when it legitimately holds several. This distinction drives
whether documents on the shelf must carry a kind discriminator (see
[taxonomy model](02-taxonomy-model.md)).

### Kind

What a document *is*: a decision record, a component specification, a runbook, an
incident record, a standard. A kind carries:

- the **purpose** it serves — the reader intent it exists to satisfy, declared, not
  implied. A kind without one is invalid: purpose plus form is what makes a kind a
  genre rather than a shape;
- the **section contract** — headings the document must or may have;
- the **facet schema** — which metadata it must, may, and must not carry;
- the **voice regime** and **lifecycle regime** it obeys;
- its **identifier scheme**, if it mints one;
- the **relations** it may (or must) participate in;
- its **template**.

A kind is resolved from a document's position (shelf + path pattern) and, on
heterogeneous shelves, from a discriminator facet. Resolution is deterministic and
explainable: the engine can always say *why* a document was typed as it was.

### External anchor

A node representing something outside the corpus that documents point at: a source
path, a component, a work item, a service, a released artefact, a URL. Anchors make
otherwise-dangling references first-class, so the engine can check them, traverse
them, and answer "what governs this code path?" without special-casing every kind
of pointer.

An anchor carries an identifier, a name, and an owner — never a purpose or a
lifecycle; every substantive claim about the thing itself lives in a document.
Anchor kinds are declared like everything else: the taxonomy's `anchors`
declaration names each type and the single resolver that owns it, so a relation
endpoint is always either a declared kind or a declared anchor kind — never a
bare string. Anchor identity is declared, not guessed: anchor strings normalise
before comparison so two spellings of one target are one node, and an anchor no
resolver claims is a finding
([spec 2](02-taxonomy-model.md#behaviour-at-the-limits)).

## Edges

### Relation

A **typed, named, directed link** between documents (or from a document to an
anchor), declared in front matter. Relations are the system's connective tissue and
its main source of checkable structure. A relation type declares:

- **family** — one of six fixed families (`succession`, `derivation`, `governance`,
  `evidence`, `composition`, `association`) supplying default semantics;
- **nuclearity** — whether both ends stand alone (multinuclear) or one end supports
  the other and cannot stand without it (nucleus–satellite), and if so which end is
  the nucleus. The family supplies the default; a relation that contradicts it says
  so explicitly, and the override is reported by `taxonomy audit`;
- **endpoints** — which kinds may sit at each end;
- **cardinality** — how many are allowed, and whether one is required;
- **reciprocity** — whether the target must acknowledge the source, and with which
  inverse relation;
- **directionality constraints** — for example, a reference may only point at the
  same abstraction tier or higher;
- **lifecycle interaction** — for example, a live document may not depend on a
  superseded one.

Because relation semantics live in the schema, the checks over them are generic.
Adding a new relation type to a taxonomy adds validation for free; it does not add
a linter.

Relation types an adopter is likely to want (all shipped with the default
package, none hard-coded in the engine): `supersedes` / `superseded_by`,
`derives_from`, `governs`, `verifies`, `implements`, `cites`, `owns`, `refines`,
`conflicts_with`. Shipped is not enabled: the default *enables* a minimal four,
and the rest are complete declarations an overlay pulls in by reference
([spec 2](02-taxonomy-model.md#the-decision-relation-vocabulary)).

A kind may also declare a participation **expectation**: its documents, in a given
state, are expected to acquire a named relation within a window measured from a
declared origin date ([spec 2](02-taxonomy-model.md#participation-expectations)).
Expectations are the only construct that finds a document which **should exist and
does not** — every other check validates artefacts that are present. They model
what genre theory calls a *genre system*: proposal → decision → specification →
evidence.

Which end governs the *reading* when two documents are linked is derived, never
declared. On a nucleus–satellite relation the nucleus governs: a document that
cannot stand alone cannot govern the reading of the one it depends on. On
succession the successor governs — that is what succession means. Other
multinuclear relations carry no reading order, and none has yet needed one; a
routing or projection outcome that demands it is the evidence that would reopen
this ([spec 2](02-taxonomy-model.md#reading-precedence-is-derived)).

### Facet

A named metadata dimension attached to documents. A facet declares its value space
(free scalar, date, enum with a controlled vocabulary, or list of any of those),
where it applies, whether it is required, and how strictly it is enforced.

A facet value is never a reference to another node. An earlier draft permitted
that, and it was a second, ungoverned edge mechanism: a reference-valued facet
asserts exactly what a relation asserts — this document is connected to that
node — while carrying no family, no nuclearity, no reciprocity, no lifecycle
interaction, no `created_by`, and generating none of the graph checks. The same
fact would get two levels of governance depending on which syntax a taxonomy
author happened to reach for, and the ungoverned syntax is the cheaper one, so
under deadline pressure it wins. A connection is a relation; an opaque external
identifier (a ticket number, a framework control id) is a scalar, and resolving
it into the graph is what external anchors are for.

Facets are how a taxonomy expresses everything the directory cannot: lifecycle
state, freshness, ownership, scope, audience, provenance, confidentiality, kind
discriminators on heterogeneous shelves. The vocabulary of an enum facet is part of
the schema, so extending it is a deliberate, reviewable, versioned change.

Three facet roles are structurally special because the engine reasons about them:

- the **state facet**, which the lifecycle regime interprets;
- the **state-entry date**, stamped by each transition — the origin that windowed
  participation expectations are measured from;
- the **freshness facet**, which staleness detection interprets.

Which facet plays each role is declared, not assumed — a corpus may call its state
facet `status`, `stage`, or `état`.

## Regimes

A **regime** is a reusable, named bundle of rules that a kind opts into. Regimes
exist so that rules are declared once and referenced many times.

- **Voice regime** — the register a document is written in, with the machine-
  checkable part of it: e.g. *declarative present-state* (no future intent, no
  narration of change, no phased-rollout language), *narrative* (time-boxed
  exploration, changelogs), or *unconstrained*.
- **Lifecycle regime** — a state machine over the state facet: states, legal
  transitions, terminal states, and what each state implies (e.g. superseded
  documents are retained and delinked from live dependency paths).

An earlier draft had two more regimes, and each was one parameter wearing a
wrapper. **Freshness policy** — the staleness threshold, drift weighting, and
posture — lives on the freshness facet itself, and is mandatory: a facet carrying
the freshness role without applicable policy is invalid. **Size budgets** live on
the agent-facing kinds and projections they meter, and are mandatory there: an
agent-facing projection without an applicable budget is invalid. The protections
moved to their enforcement points; only the wrappers were deleted.

## Obligations and controls

### Obligation

A statement the corpus commits to, expressed as an **invariant** with a stable
identifier. "Behaviour-changing code updates its specification in the same change."
"Every live decision record is reachable from what it constrains." Obligations are
data in the corpus, not prose in a document.

### Control

A mechanism that discharges an obligation: an engine check, a CI job, a git hook,
an agent behaviour, a scheduled scan, a human audit. A control declares what it
verifies, when it runs, and its **posture** — advisory, blocking, or detective.

### The register

The generated, checked view of how obligations and controls bind: which controls
discharge which obligations, and the explicit disposition of every obligation
with none — *gap* (tracked, wanted) or *unverifiable* (no mechanism can exist;
accepted). Every obligation carries exactly one disposition.

Obligations and controls are authored; the register never is. The binding lives
on the control (`discharges:`) and the disposition on the obligation, so the
register is a projection of the other two — mandatory, regenerated, failed in CI
when it differs. Coverage claims are generated from it, never asserted in prose.

See [assurance model](04-assurance-model.md).

## Projections

A **projection** is a derived artefact computed from the graph: a shelf index, a
decision-lineage summary, a component-to-specification matrix, site navigation, an
agent instruction file, a JSON export of the graph. Projections are:

- **generated** — never hand-authored;
- **checked** — CI fails when a committed projection differs from a regenerated one;
- **declared** — the schema names them, so a taxonomy can add projections without
  engine changes, within the set of projection kinds the engine implements.

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
