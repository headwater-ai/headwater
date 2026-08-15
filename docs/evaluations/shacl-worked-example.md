---
id: HW-EVAL-shacl-worked-example
status: current
status_since: 2026-08-02
last_verified: 2026-08-11
summary: The Headwater checks written out in SHACL, and the whole-graph line where the constraint language stops.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: [claude-fable-5, claude-opus-5]
  activity: evaluate+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cited_by:
    - HW-REG-decisions
    - HW-SPEC-assurance-model
    - HW-SPEC-engine-architecture
    - HW-SPEC-adjacent-work
    - HW-SPEC-check-layer
---

# The Headwater checks in SHACL — a worked example

The companion to the [LinkML worked example](linkml-worked-example.md), and the second half of the evidence for [Q13](../spec/09-open-questions.md#q13--linkml-and-shacl-as-substrate).

The two are not alternatives. LinkML is a **schema** language — it says what a document is. SHACL is a **constraint** language over **graphs** — it says what must hold across them. So SHACL maps onto Headwater's *checks*, not its taxonomy, and it lands exactly where the LinkML exercise found the boundary: whole-graph invariants.

**It gets further than I claimed in Q13, and one objection I recorded there does not survive contact with the spec.**

## First, the corpus as RDF

SHACL validates RDF, so the corpus graph needs a triple projection. This is mechanical — the graph is already typed nodes and typed edges.

```turtle
@prefix dg:   <https://w3id.org/headwater/taxonomy/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .

<file:///docs/decisions/dr-0042.md>
    a              dg:Decision ;
    dg:id          "ACME-DR-0042" ;
    dg:status      dg:current ;
    dg:lastVerified "2026-07-15"^^xsd:date ;
    dg:summary     "Overlay-based customization keeps consumers on upstream fixes" ;
    dg:evidenceBasis dg:evidenced ;
    dg:supersedes  <file:///docs/decisions/dr-0031.md> ;
    dg:governs     <file:///src/taxonomy/resolve.rs> .
```

One useful property: `sh:targetClass` matches *SHACL instances*, which follows `rdfs:subClassOf`. Declaring `dg:Decision rdfs:subClassOf dg:Document` makes a shape targeting `dg:Document` apply to every kind, without a reasoner. Kind inheritance comes free.

## Tier 1 — shape constraints, which SHACL does trivially

Everything LinkML covered, SHACL also covers, with better message and severity control.

```turtle
@prefix sh:  <http://www.w3.org/ns/shacl#> .
@prefix dgs: <https://w3id.org/headwater/shapes/> .

dgs:DocumentShape
    a sh:NodeShape ;
    sh:targetClass dg:Document ;

    sh:property [
        sh:path      dg:status ;
        sh:minCount  1 ;
        sh:maxCount  1 ;
        sh:in        ( dg:draft dg:current dg:superseded dg:deprecated ) ;
        sh:message   "status is required and must be a declared lifecycle state" ;
    ] ;

    sh:property [
        sh:path      dg:lastVerified ;
        sh:minCount  1 ;
        sh:datatype  xsd:date ;
        sh:message   "last_verified must be a date a human asserted, not an edit time" ;
    ] ;

    sh:property [
        sh:path      dg:summary ;
        sh:minCount  1 ;
        sh:minLength 20 ;
        sh:maxLength 200 ;
        sh:severity  sh:Warning ;
        sh:message   "summary is the corpus's scent surface; too short to discriminate" ;
    ] ;

    sh:property [
        sh:path      dg:id ;
        sh:minCount  1 ;
        sh:maxCount  1 ;
        sh:pattern   "^DR-[A-Z]{2,6}-[0-9]{4}$" ;
    ] .
```

`sh:closed true` additionally reports facets nobody declared — the unknown-key check — which LinkML has no direct equivalent for.

Relation endpoints are equally direct:

```turtle
dgs:DecisionShape
    a sh:NodeShape ;
    sh:targetClass dg:Decision ;
    sh:property [
        sh:path     dg:supersedes ;
        sh:class    dg:Decision ;      # endpoint kind
        sh:nodeKind sh:IRI ;
        sh:message  "supersedes must point at a decision" ;
    ] .
```

## Tier 2 — the graph invariants LinkML could not reach

This is the interesting part. **Every one of these requires SPARQL.** SHACL Core's property paths — including `sh:inversePath` — let you *traverse*, but Core has no way to refer back to the focus node from the far end of a traversal, which is precisely what these constraints need.

### Reciprocity

The check LinkML definitively could not express:

```turtle
dgs: sh:declare [ sh:prefix "dg" ;
                  sh:namespace "https://w3id.org/headwater/taxonomy/"^^xsd:anyURI ] .

dgs:DecisionShape
    sh:sparql [
        a sh:SPARQLConstraint ;
        sh:severity sh:Violation ;
        sh:message  "{$this} supersedes {?target}, which does not link back" ;
        sh:prefixes dgs: ;
        sh:select """
            SELECT $this ?target
            WHERE {
                $this dg:supersedes ?target .
                FILTER NOT EXISTS { ?target dg:supersededBy $this }
            }
        """ ;
    ] .
```

`$this` is pre-bound to the focus node, so the round trip closes. This is the single most important result in this document: **the constraint that defeated LinkML is routine in SHACL-SPARQL.**

### Conflict between two live decisions

Reads the *target's* state — impossible per-instance, trivial here:

```turtle
    sh:sparql [
        sh:message "{$this} and {?other} are both current and declared in conflict" ;
        sh:prefixes dgs: ;
        sh:select """
            SELECT $this ?other
            WHERE {
                $this  dg:conflictsWith ?other ;
                       dg:status dg:current .
                ?other dg:status dg:current .
            }
        """ ;
    ] .
```

### Satellite inheritance

A live satellite whose nucleus has gone terminal:

```turtle
    sh:sparql [
        sh:severity sh:Warning ;
        sh:message  "{$this} derives from {?nucleus}, which is no longer current" ;
        sh:prefixes dgs: ;
        sh:select """
            SELECT $this ?nucleus
            WHERE {
                $this dg:derivesFrom ?nucleus ;
                      dg:status dg:current .
                ?nucleus dg:status ?ns .
                FILTER( ?ns IN ( dg:superseded, dg:deprecated ) )
            }
        """ ;
    ] .
```

### Sequence expectations — expressible, with two caveats

```turtle
    sh:sparql [
        sh:severity sh:Info ;
        sh:message  "{$this} is current but nothing implements it" ;
        sh:prefixes dgs: ;
        sh:select """
            SELECT $this
            WHERE {
                $this dg:status dg:current ;
                      dg:lastVerified ?since .
                FILTER NOT EXISTS { ?spec dg:implements $this }
                FILTER( xsd:dateTime(?since) < ( $now - "P90D"^^xsd:dayTimeDuration ) )
            }
        """ ;
    ] .
```

Two things this exposes that no amount of SHACL fixes:

1. **Date arithmetic is fiddly.** `?since` is an `xsd:date` and duration arithmetic wants `xsd:dateTime`, so a cast is required, and engines vary in what they accept.
2. **`NOW()` would make the check non-deterministic**, which collides head-on with [spec 6](../spec/06-engine-architecture.md)'s requirement that the same corpus and the same lock produce byte-identical output. The evaluation time has to be *injected* as a bound variable — written as `$now` above — rather than read from the clock. That is a real design constraint on any temporal check, and it is not SHACL-specific: it applies to whatever engine we build.

## Does this help with the actual documents?

The sharper question, and the answer is narrower than the sections above suggest.

SHACL validates **RDF**. Headwater's instances are Markdown files with YAML front matter. So nothing above touches a document directly — it touches a *projection* of one, and everything depends on what that projection contains.

### What is in the graph, and what is not

| In the projection — SHACL sees it | Not in the projection — SHACL is blind |
|---|---|
| Front-matter facets: status, dates, ids, enums | The section contract — required headings |
| Declared relations and their endpoints | Voice regime — declarative present-state |
| Identifier format and uniqueness | Normative language usage and its boilerplate |
| Graph invariants: reciprocity, conflicts, inheritance | Prose links, as distinct from declared relations |
| Kind, shelf, provenance | Summary distinctiveness against siblings |
| | Size and token budgets |
| | The prose itself |

For a typical decision record that is roughly ten lines of front matter against two hundred of body. **SHACL covers the ten.**

That is not as damning as it sounds — the ten lines are where the *graph* lives, and graph invariants are the checks that are hardest to hand-write and most valuable to have. It is the small half by volume and a valuable half by weight. But it is decidedly not "document validation", and calling it that would mislead.

You *could* lift more in: project headings as triples and `sh:qualifiedValueShape` can check a section contract. Each lift makes the projection larger, lossier, and more of a parallel re-encoding of the document that has to be kept in step. Projecting the body as a literal and running `sh:pattern` over it is technically possible and a bad idea — that is regex over prose with extra steps.

### Problem one: everything downstream trusts the projection and SHACL does not check it

The Markdown-to-RDF projector becomes the most trusted component in the pipeline, and nothing in SHACL validates it. A projector that drops a document, mistypes it, or misparses front matter produces a graph that does not represent the corpus — and SHACL will happily report that graph as conformant.

This is a new trust boundary that did not exist when checks read the documents directly, and it needs its own fidelity tests. It also bears on [Q6](../spec/09-open-questions.md#q6--where-the-corpus-graph-lives-at-rest): if RDF is a derived view, the derivation is a component; if it is stored, it is a second copy that can drift from the Markdown.

### Problem two: silent passes

This is the serious one. The SHACL specification is explicit that it **provides no mechanism to report coverage gaps or detect unvalidated nodes** — the language has no concept of completeness. Conformance means "no validation results were produced", and a node that no target selects produces none.

Combine that with `sh:targetClass` requiring explicit `rdf:type` triples in the data graph, and the failure mode writes itself:

> A document whose front matter fails to parse, or whose kind cannot be resolved, yields no type triple → is selected by no target → produces no violations → **the corpus conforms.**

For an assurance system that is the worst available failure mode, because absence of data is indistinguishable from absence of problems, and it fails *quiet* and *green*. The document that is most broken is the one most likely to escape.

The fix is not in SHACL. Headwater must establish, before validation runs, that every file in the corpus was classified and routed to at least one check — and treat a document that matched nothing as a finding in its own right. That guarantee generalizes past SHACL to any check layer, so it belongs in the assurance model rather than here.

> **Applied:** every-document-accounted-for as an explicit obligation in [spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for).

### Problem three: change-scoped validation cuts across graph constraints

[Spec 6](../spec/06-engine-architecture.md) budgets under 200 ms for a change-scoped hook. SHACL targets are graph-wide, and the graph constraints need neighbours: checking only the touched document is not sound.

Worse, *which* node reports a violation depends on how the constraint was written. The reciprocity shape above is authored on the superseding decision, so editing **A** to add `supersedes B` surfaces the violation on A. Author it the other way — as a shape on the target requiring an inbound link — and the same edit surfaces it on B, which was not touched and would not be in a naive changed-files set.

So change-scoped validation needs the affected *subgraph*, and its extent is a function of the constraint set rather than of the diff. That is tractable, and it is a real design constraint that only becomes visible at instance level.

## Where SHACL stops

| Headwater capability | SHACL |
|---|---|
| Shape constraints, enums, cardinality, patterns | Core, directly |
| Unknown-facet detection | `sh:closed` |
| Relation endpoints, lifecycle-sensitivity | Core |
| Reciprocity, cross-node conflict, satellite inheritance | SPARQL |
| Sequence expectations | SPARQL, with injected time |
| Conditional constraints ("only when this document is current") | Awkward in Core; SPARQL or SHACL-AF targets |
| Overlay algebra, core satisfiability, compatibility measurement | **No** — operations *on* the schema, not statements in it |
| Projections and document generation | **No** — SHACL-AF `sh:rule` infers triples, not documents |
| Facet orthogonality, continuity distribution | **No** — SPARQL aggregates count and group; correlation and mutual information are outside it |
| Capture-cost instrumentation | **No** — not validation at all |

The pattern is consistent with the LinkML finding. SHACL covers the **graph** layer that LinkML could not, and neither touches the layer above — operations on the schema itself — or the measurement layer below.

## Two corrections to what Q13 recorded

**The error-message objection was too strong.** I wrote that SHACL's violation reports are "famously hard to read", which is true of raw reports and irrelevant here. `sh:message` with `{$this}` / `{?var}` interpolation in SPARQL-based constraints makes messages exactly as good as they are authored — and since Headwater would *generate* the shapes, they would be as good as our generator. That objection should be withdrawn.

**The real objections are different, and sharper:**

- **Line numbers are lost.** A SHACL `ValidationResult` carries `focusNode`, `resultPath`, `value`, `sourceShape`, `resultSeverity` and `resultMessage`. RDF has no notion of a byte offset in a Markdown file, so the `line` field that [spec 4](../spec/04-assurance-model.md#findings) requires cannot survive the round trip. Findings would need to be re-anchored to source positions by the Headwater side.
- **No remediation, no fixability.** Spec 4 requires every finding to carry a remediation and a `fixable` flag. SHACL has no slot for either. They can be hung off the shape as custom properties and looked up via `sourceShape`, but that is a convention we would define, not something a stock SHACL consumer would understand.
- **Posture is not severity.** `sh:Violation` / `sh:Warning` / `sh:Info` map onto Headwater's severities, but Headwater's *posture* — advisory versus blocking, and the promotion criteria attached to it — is a property of the control, not of the shape. It lives in the runner either way.

**And one objection dissolves.** Q13 noted that a SPARQL engine pulls against the single-binary, offline, sub-second constraints of spec 6. Embeddable Rust SPARQL engines exist, so an in-memory store over a corpus of this size is not obviously a problem. It needs measuring against the change-scoped budget rather than assuming, but it is not the blocker it looked like.

## What this means for Q13

It reinforces **option 3**, and extends it. The LinkML example concluded: author in Headwater's language, emit LinkML. This one adds the other half.

> Emit **LinkML for the shape layer and SHACL for the graph layer**. Between them, an external consumer can validate a Headwater corpus to a genuinely useful depth without installing Headwater at all.

The argument that decides it is the same one, and it is stronger here. Every interesting SHACL constraint above is embedded SPARQL. Hand-authored, that is *less* readable than the equivalent engine predicate and considerably harder to test. **Generated, nobody reads it** — and readability stops being a cost at all. That is the difference between adopting SHACL as the authoring surface (bad) and as a compilation target (good), and it is exactly the conclusion the LinkML exercise reached by a different route.

Two things stay Headwater-native regardless, and they should be stated as such rather than discovered later:

- the checks SHACL cannot express — schema operations, statistical measures, instrumentation;
- the finding shape, because line anchoring, remediation and fixability are what make a finding actionable, and none of them survive the RDF round trip.

So the emitted SHACL is deliberately a **subset**, and should say so in its own metadata. An external validator that reports a clean run against a partial shape set, while believing it checked everything, is worse than one that knows what it skipped.
