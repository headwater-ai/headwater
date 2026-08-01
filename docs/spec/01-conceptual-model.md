# 1 — Conceptual model

The vocabulary the rest of the specification uses. Each term is a first-class
concept in the schema and in the engine; where they differ, this document says so.

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

## Edges

### Relation

A **typed, named, directed link** between documents (or from a document to an
anchor), declared in front matter. Relations are the system's connective tissue and
its main source of checkable structure. A relation type declares:

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

Relation types an adopter is likely to want (all defined in the default taxonomy,
none hard-coded in the engine): `supersedes` / `superseded_by`, `derives_from`,
`governs`, `verifies`, `implements`, `cites`, `owns`, `refines`.

### Facet

A named metadata dimension attached to documents. A facet declares its value space
(free scalar, date, enum with a controlled vocabulary, reference to another node,
or list of any of those), where it applies, whether it is required, and how
strictly it is enforced.

Facets are how a taxonomy expresses everything the directory cannot: lifecycle
state, freshness, ownership, scope, audience, provenance, confidentiality, kind
discriminators on heterogeneous shelves. The vocabulary of an enum facet is part of
the schema, so extending it is a deliberate, reviewable, versioned change.

Two facets are structurally special because the engine reasons about them:

- the **state facet**, which the lifecycle regime interprets;
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
- **Freshness regime** — how a document declares it was last confirmed true, the
  staleness threshold, and what happens on expiry.
- **Size regime** — token or byte budgets, applied mainly to agent-facing
  instruction files where context is metered.

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

The binding of obligations to controls, plus explicit dispositions for those with
none: *gap* (tracked, wanted) or *unverifiable* (no mechanism can exist; accepted).
Every obligation carries exactly one disposition. Coverage claims are generated
from the register, never asserted in prose.

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
