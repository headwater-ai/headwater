# The taxonomy in OWL and SKOS — a worked example

The third of the substrate worked examples, after [LinkML](linkml-worked-example.md) and [SHACL](shacl-worked-example.md). Those two asked whether an external language could hold the Headwater schema and the Headwater checks. This one asks the question that [Q13](../spec/09-decisions.md#q13--linkml-and-shacl-as-substrate) staged fourth: what happens when the taxonomy is emitted as an ontology, and the corpus as instance triples, and a reasoner is pointed at the result.

**Nothing here reopens Q13.** The staging order stands. RDF and SKOS are emitter 4, and they ship when a named external consumer asks. This document is evidence about what that emitter will owe when it is written, gathered by writing a throwaway version of it now.

The two earlier examples were written by hand. This one was run. The emitter is committed at [`tools/rdf-probe/emit.py`](../../tools/rdf-probe/emit.py), it reads the real base package and the real design-spec bundle, and it emits the real `docs/` tree. Every number below comes from that run.

## The method

The emitter resolves the taxonomy the way the resolver will: the base package `headwater/standard` plus the [design-spec bundle](../taxonomies/design-spec/bundle.yml) applied as add-only operations at dotted addresses. It then emits three graphs.

- **`taxonomy.ttl`** — the TBox. A kind becomes an `owl:Class`, `is_a` becomes `rdfs:subClassOf`, a facet becomes an `owl:DatatypeProperty` or `owl:ObjectProperty`, a controlled vocabulary becomes a `skos:ConceptScheme`, a relation becomes an `owl:ObjectProperty` with domain and range.
- **`corpus.ttl`** — the ABox. Kind resolution is by placement, which is what [spec 2](../spec/02-taxonomy-model.md#placement-is-primary-metadata-fills-the-gap) says is primary. Every Markdown link between two corpus documents becomes an edge.
- **`shapes.ttl`** — the same required-facet constraints written as SHACL, so that the two languages can be pointed at one graph and compared.

Three checks then run: OWL-RL closure over the combined graph, SHACL validation of the ABox against the shapes, and a serialize-parse round trip. The reasoner is `owlrl`, the validator is `pyshacl`, and both run offline.

**One thing had to be scraped rather than read.** The base package has no machine-readable home in this repository. Its only committed copy is a fenced YAML block inside [the first-run walkthrough](default-taxonomy-first-run.md#the-base-derived-rather-than-chosen), so the emitter parses it out of the Markdown. That is a finding in its own right and it is listed at the end.

## What the run reports

| Measure | Result |
|---|---|
| TBox | 293 triples |
| ABox | 433 triples, over 36 documents and 332 edges |
| Documents typed by placement | 14 of 36 |
| OWL-RL entailment | 1349 triples added |
| OWL-RL inconsistency | none |
| SHACL, no inference | 56 violations |
| SHACL, with RDFS inference | 140 violations |
| Round trip | isomorphic |
| Declared loss set | 28 constructs, 91 dropped declarations |

The corpus records no `status`, no `status_since`, no `last_verified` and no `summary`, because this repository is not typed yet: issue #4 is open and the dogfooding has not run. That is what makes the comparison worth having. Every document is missing every required facet, and the three checks disagree completely about whether that matters.

**The corpus includes this document.** The evaluations shelf is where this file lives, so the run that produced the table above counted it, and the figures moved when it was written. That is correct rather than awkward — a corpus that governs itself has no outside to stand on — but it means a reader who re-runs the probe after any commit to `docs/` gets different numbers. The findings below do not depend on the arithmetic.

## Finding 1 — the open-world assumption inverts the census

The reasoner finds no inconsistency. That is not the interesting part. The interesting part is what it concludes instead.

Take `docs/evaluations/first-contact.md`. It carries zero facet triples. After OWL-RL closure it is a member of four anonymous restriction classes:

    has min 1 of status
    has min 1 of status_since
    has min 1 of last_verified
    has min 1 of summary

The reasoner has concluded that the document **has** all four required facets. It reached that by the only route available to it: `hw:Evaluation` is a subclass of a restriction that says its members have at least one `hw:status`, this document is an evaluation, therefore this document has a status. The absence of the triple is not evidence of anything, because under the open-world assumption nothing is.

`owl:minCardinality` looks like a requirement and behaves as an entailment. Emit it and a consumer reads a constraint that the reasoner will satisfy by inventing the value rather than by reporting its absence.

[Spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for) fixes the denominator before any check runs, and requires that every document is either checked or accounted for by a declared reason. An OWL reasoner cannot participate in that doctrine at all. It is not that it does the census badly. It computes the opposite quantity: what must be true given what is stated, rather than what is missing from what is stated.

This is the sharpest form of the objection [Q13](../spec/09-decisions.md#q13--linkml-and-shacl-as-substrate) already records against SHACL, which "defines conformance as no validation results and has no notion of completeness". SHACL is merely silent about the denominator. OWL argues against it.

## Finding 2 — an endpoint declaration becomes a retyping rule

This one was not predicted anywhere in the specification, and it is the reason the exercise was worth running.

[Spec 2](../spec/02-taxonomy-model.md#endpoints-are-the-only-permission) titles a section "Endpoints are the only permission". A relation declares `from` and `to`, and an edge whose endpoint is not of a declared kind is a finding. The natural RDF spelling of `from` and `to` is `rdfs:domain` and `rdfs:range`. Both are inference rules. Neither is a permission.

`docs/spec/09-decisions.md` is the decision register. It sits on a heterogeneous shelf, so its kind resolves through the `doc_type` discriminator, which lives in front matter that this corpus does not have. It therefore enters the ABox with no type at all. After OWL-RL closure it has one:

    hw:Evaluation

The register is typed as an evaluation. It became one because some evaluation links to it, the link became a `cites_evidence` edge, and `cites_evidence` declares `to: [evaluation]`, which emits as `rdfs:range hw:Evaluation`. The reasoner did what RDFS range means: whatever appears in that position **is** of that class.

Across the corpus, **35 of 36 documents gained a kind purely by entailment**, and the kinds are wrong. A misfiled edge does not produce a finding in this export. It silently relabels the node at the other end.

That is a genuine impedance mismatch rather than an emitter defect, and no spelling of the emitter avoids it. `rdfs:range` is the only standard property that means what `to:` is trying to say, and it means something else. An emitter that wants endpoint checking has to emit `sh:class` on a SHACL property shape instead, and then the RDF export carries the ontology while the SHACL export carries the rule — two artifacts, and a consumer who takes only the first gets a graph that retypes itself.

## Finding 3 — `owl:inverseOf` manufactures the half that reciprocity wants reported

`cites_evidence` declares `inverse: cited_by` and `reciprocal: required`. Reciprocity in Headwater means the missing half is a finding: [spec 2](../spec/02-taxonomy-model.md#behavior-at-the-limits) says that inverses which disagree are reported as a pair and the corpus stays incoherent until an author resolves it.

The run entails **332 `hw:cited_by` triples**, exactly one per asserted `cites_evidence` edge. Not one of them was declared by any author. `owl:inverseOf` closes the gap that `reciprocal: required` exists to open.

Both halves of the model are reasonable and they are not compatible. RDF treats an inverse as a fact about the world that is true whether or not anybody wrote it down. Headwater treats a declared edge as an authoring act with a creator and a maintenance intent, which is why `created_by` is on the relation type and why `taxonomy audit` measures it. Emit `owl:inverseOf` and the audit's denominator is destroyed: every edge acquires a reciprocal that nobody maintains.

The honest emission is to leave `owl:inverseOf` out and declare it in the loss set. That costs a consumer the convenience of querying in either direction, and it is the only option that does not fabricate authorship.

## Finding 4 — a document with no type is checked by nothing

Run SHACL over the ABox with no inference and it reports **56 violations**. That is 14 typed documents times four required facets. The 22 documents on heterogeneous shelves report nothing, because a SHACL shape targets a class and an untyped node is targeted by no shape.

Turn RDFS inference on and the count becomes **140**, which is 35 documents times four. The extra 84 are the violations of the 21 documents that finding 2 retyped incorrectly.

The true figure is 36 documents times four required facets, which is 144. Neither run reaches it. One under-reports because the untyped documents are invisible, and the other under-reports differently while attributing the violations to the wrong kinds.

So the answer to "is this corpus conformant" depends on a reasoner setting, and no setting gives the right denominator. This is the silent-pass failure that [spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for) forbids, reproduced mechanically. The census has to be computed by the engine, before the export, and carried in the export as data. It cannot be recovered by a consumer who validates the export.

## Finding 5 — what the two languages carry without loss

The exercise is not one-sided. Six things map cleanly and would cost an emitter almost nothing.

- **Kind inheritance.** `is_a` to `rdfs:subClassOf` works, and the SHACL example already noted that `sh:targetClass` follows it, so a shape on the abstract kind reaches every concrete one.
- **Controlled vocabularies.** A vocabulary is a `skos:ConceptScheme` and each value a `skos:Concept`, with `skos:prefLabel` and `skos:inScheme`. This is what SKOS is for, and the fit is exact.
- **Purpose prose.** `intent` becomes `skos:definition` and each entry in `answers` becomes a `skos:scopeNote`. The reader intent that spec 2 requires a purpose to declare survives intact.
- **Facet guidance.** Per-value guidance becomes `skos:scopeNote` on the value concept.
- **Lineage.** `supersedes` emits as `rdfs:subPropertyOf prov:wasRevisionOf`, which is the alignment [spec 2](../spec/02-taxonomy-model.md#lineage-aligns-with-prov) already declared. It costs one triple and it is the single place where an external standard says what Headwater means.
- **Symmetry.** `reciprocal: symmetric` on `conflicts_with` emits as `owl:SymmetricProperty` and means the same thing on both sides.

The pattern is the one both earlier examples found, from a third direction. The **vocabulary** layer exports well. The **constraint** layer exports as something that looks like a constraint and behaves as an inference. The **governance** layer does not export at all.

## Finding 6 — SKOS has no slot for the role a value plays

The lifecycle vocabulary is not a flat list. Each value carries a role: `draft` is `initial`, `current` is `live`, `superseded` and `deprecated` are `terminal-retained`. Those roles are what the lifecycle regime's transition table is written against, and what the immutable core is stated in terms of.

SKOS has `broader`, `narrower`, `related`, and a notation. It has nothing for the role of a concept in a state machine. The emitter carries the role as `hw:lifecycleRole`, a Headwater annotation that no stock SKOS consumer reads:

```turtle
<https://w3id.org/headwater/taxonomy/lifecycle_state/current> a skos:Concept ;
    skos:inScheme <https://w3id.org/headwater/taxonomy/scheme/lifecycle_state> ;
    skos:prefLabel "current"@en ;
    hw:lifecycleRole "live" .
```

The same applies one level up. The transition table itself — `draft` may become `current` or `deprecated`, `current` may become `superseded` or `deprecated`, and nothing leaves a terminal state — has no expression in OWL or SKOS. A knowledge-organization consumer receives the four values and learns nothing about which order they come in.

The facet **role** has the same shape of problem. The core requires the `state`, `freshness` and `scent` roles, and OWL has no construct for the role a property plays in a schema. Four facet roles emit as annotations.

## The loss set

Twenty-eight constructs, 91 dropped declarations. This is what a real emitter would have to declare under the [loss-set doctrine](graph-export-and-federation.md), and the count is the argument for that doctrine rather than against the emitter.

| Layer | What is dropped |
|---|---|
| kind | abstract kinds, purposes, voice regime, required sections, identifier schemes, required facets, participation expectations |
| relation | family defaults, nuclearity, `created_by`, cardinality, required reciprocity, `on_target` state effects, `invalid_when` |
| facet | facet roles, `stale_after_days`, volatility |
| vocabulary | the role of each lifecycle value |
| regime | lifecycle transitions, voice, language |
| shelf | placement, layout, discriminator |
| instance | finding anchors, warrant |
| other | anchor resolvers, identifier allocation, projections, the immutable core |

Three entries deserve naming, because each one is a whole capability rather than an attribute.

**Nuclearity.** It drives satellite inheritance, context pruning under a budget, orphan severity, and deletion safety. No OWL property characteristic expresses which end of a relation stands alone.

**`invalid_when`.** `conflicts_with` is invalid when both endpoints are `current`. OWL can say the property is symmetric. It cannot say that a pair of instances is invalid under a condition on both of their states. The SHACL example already placed this in SPARQL, and that placement is confirmed.

**The immutable core.** Six requirements about what a taxonomy must declare. These are operations on the schema, not statements in it, which is the boundary both earlier examples found and this one does not move.

## The round trip proves nothing

The serialize-parse round trip is isomorphic. Two hundred and ninety-three triples out, 293 back, structurally identical.

That result is worth recording precisely because it is worthless. [Q6](../spec/09-decisions.md#q6--where-the-corpus-graph-lives-at-rest) says an empty loss set is what a round trip proves, and the converse holds: a round trip over an already-lossy projection proves only that Turtle serialization is deterministic. The 91 dropped declarations never entered the graph, so no round trip over the graph can detect them.

An emitter that reports a clean round trip as evidence of fidelity is reporting on the wrong artifact. The measurement that means something is the projection census, taken against the source graph, before serialization.

## What this means for Q13

The staging order is unchanged and better supported than before.

RDF and SKOS at position 4 is right, and the consumer named there — knowledge-organization tooling — is the consumer this export actually serves. The vocabulary layer exports cleanly, which is what such tooling wants. The constraint layer is where the export misleads, and a knowledge-organization consumer is not the one reading it.

**One sharpening.** Q13 records that emitters never chain, and the reason given is that a chained pipeline inherits every loss of every hop and declares none. Findings 1 and 2 add a second reason that is stronger for this emitter. The RDF export does not merely lose the constraint layer. It re-expresses it as inference, so a consumer who reads `owl:minCardinality` or `rdfs:range` as a check receives an answer that is confidently wrong rather than absent. A loss set covers what is missing. It does not cover a construct that survives the trip with its meaning inverted.

That suggests a rule the specification does not yet have: **where a target language has a construct that looks like a Headwater constraint and behaves as an entailment, the emitter either omits it or emits it beside an explicit statement that it is not a check.** Omission is cheaper and it is what `exportable_as` already implies, since [spec 12](../spec/12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule) admits a target only when the emitted constraint catches exactly what the native check catches. `owl:minCardinality` fails that bar and would therefore never have been emitted by a conformant emitter. The bar works. It just has not been read as forbidding this case, and the case is not obvious until a reasoner runs.

## Findings for 13 — Open obligations

Four items, and [the register](../spec/13-open-obligations.md) now carries all of them. The first two became entries under design work that nothing blocks. The last two joined one Q13 entry under what each decision left open, because both wait on emitter 4 rather than on a design choice that anybody can take today.

**The base package has no machine-readable home.** Its only committed copy is a fenced YAML block in an evaluation. Any tool that wants to resolve the taxonomy — this emitter, the confluence check that admission criterion 6 requires, the engine when it exists — has to parse Markdown to find it. The design-spec bundle has a real file at `docs/taxonomies/design-spec/bundle.yml` and extends a base that does not.

**`exportable_as` should be read as forbidding entailment-shaped constraints.** Spec 12's equivalence bar already excludes `owl:minCardinality` and `rdfs:range`, and nothing states the general case. One sentence would close it.

**`owl:inverseOf` conflicts with `created_by` measurement.** Emitting the inverse fabricates 332 unauthored edges in this corpus alone. Whether the RDF emitter omits inverses is a decision that emitter 4 will have to take, and finding 3 is the argument for omitting them.

**A heterogeneous shelf is untypeable without front matter.** Twenty-two of this repository's 36 documents cannot be typed by placement alone. That is not new — it is what `doc_type` is for — but it means the dogfooding in issue #4 is a precondition for any ABox export of this corpus, rather than a parallel activity.

## What this did not settle

**Whether a description-logic reasoner behaves differently on the forbidden-facet case.** `evaluation` forbids `doc_type`, which emits as `owl:maxCardinality 0`. Given a document that violates it, the OWL-RL rule set used here reports no inconsistency. A tableau reasoner over OWL DL should report one, and none was available offline to check. The finding is therefore about this profile and this implementation, not about OWL as such. Findings 1 through 4 do not depend on it: the open-world assumption and the meaning of `rdfs:range` are properties of the standard rather than of a reasoner.

**Whether the emitted ontology is useful to anybody.** No knowledge-organization consumer has looked at it, which is precisely the trigger that Q13 set for emitter 4. This exercise says what the emitter will owe. It says nothing about whether it should be written.

**The OKF and LinkML positions.** Emitters 5 and 6 are untouched here. The ABox findings apply to any triple-shaped target, so finding 3 and finding 4 probably transfer to OKF, and nobody has checked.

## Reproducing this

    python3 -m venv .venv
    .venv/bin/pip install rdflib pyshacl owlrl pyyaml
    .venv/bin/python tools/rdf-probe/emit.py --repo . --out out

Four files land in `out/`: the two graphs, the shapes, and `loss.json` with the census and the loss set. The dependencies are the reason this is not wired into the commit hook, and the reason the script is a probe rather than a tool.
