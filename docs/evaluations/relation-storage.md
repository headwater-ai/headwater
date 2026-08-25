---
id: HW-EVAL-relation-storage
status: current
status_since: 2026-08-10
last_verified: 2026-08-10
summary: The Q4 evidence, which makes a relation instance an object in front matter and refuses the annotated prose link as a second edge syntax.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: evaluate+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cited_by:
    - HW-REG-decisions
    - HW-SPEC-conceptual-model
    - HW-SPEC-taxonomy-model
    - HW-SPEC-assurance-model
    - HW-SPEC-check-layer
---

# Relation storage — the Q4 evaluation

This evaluation closes [Q4](../spec/09-open-questions.md#q4--relation-storage). The question asked where a relation instance lives. The three candidates were front matter, prose with an extractable syntax, and a sidecar edge file per document.

The recorded leaning was front matter, with prose links extracted and checked for resolvability. That leaning survives. The evaluation did not confirm it in the shape that the entry expected, and three of its findings change the specification.

## The question is not the one that Q4 asked

Q4 reads as a choice between three files. Two later questions had already made it a choice about the *shape of an edge*, and neither of them noticed.

[Q18](../spec/09-open-questions.md#q18--recording-adjudicated-disagreements) leans toward a record of adjudication "as data on the declared edge, with the adjudicator named". [Q20](../spec/09-open-questions.md#q20--where-scent-lives) leans toward "an optional cue on a relation". It states that it blocks on Q4, because "a cue attached to an edge needs somewhere to sit".

Both leanings need an edge to carry data of its own. A bare pointer cannot carry any. So Q4 must first settle whether a relation instance is a pointer or an object. That answer decides most of what follows. A file format that cannot hold an attribute rules out both leanings without argument.

## What the specification already fixed

Six rulings constrain Q4, and it may not revisit them.

**A relation endpoint is never a bare string.** [Spec 1](../spec/01-conceptual-model.md#external-anchor) requires that each endpoint is a declared kind or a declared anchor kind. Anchor strings normalize before comparison, so two spellings of one target become one node.

**A facet value is never a reference.** [Spec 1](../spec/01-conceptual-model.md#facet) removed reference-valued facets because they were a second, ungoverned edge mechanism. The reasoning transfers to Q4 without change: "the same fact got two levels of governance, decided only by which syntax a taxonomy author used. The ungoverned syntax is the cheaper one, so under deadline pressure it wins."

**The edge is a unit of checking.** [Spec 12](../spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on) declares `Edge` as a scope, defined as "one relation instance and both endpoints". Change-scoped evaluation re-runs an edge check for every edge incident to a changed file. Cache keys hash exactly the inputs in scope. Both need an edge identity that a reader can compute.

**Findings anchor to a line.** [Spec 12](../spec/12-check-layer.md#findings) requires the parser to retain spans for front-matter keys, headings, and links. Every candidate location can supply a span, so this ruling excludes nothing. It does tell us that the parser already reads all three places.

**The engine writes edges back.** [Spec 3](../spec/03-authoring-and-lifecycle.md#authoring-surfaces) gives `headwater check --fix` the job of adding a missing reciprocal link, and [spec 12](../spec/12-check-layer.md#fixability) confirms that the fix is mechanical. So relations live somewhere that a program may rewrite without a judgment about prose.

**Most edges are machine-written.** [Spec 2](../spec/02-taxonomy-model.md#who-creates-each-edge) reserves `created_by: author` for overlay additions, and every relation that the default taxonomy enables comes from a scaffold, a generator, or a hook. This weakens the strongest argument for the prose option. Prose is where humans write references, and by design humans write a minority of the edges.

## Two traditions, and what each one learned

Hypertext research answered this question twice, in opposite directions, and both answers are informative.

**Links outside the document.** Open hypermedia systems such as Microcosm and Hyper-G held links in a separate link database. The document stayed clean, the system could enumerate every link, and an editor could link over text that a reader could not change. Halasz set out the agenda for that work in *Reflections on NoteCards* and its later revisit. The Web then won with links embedded in the document, for many reasons that have nothing to do with this decision. The lesson that does transfer is narrow and reliable: two artifacts that must travel together eventually do not.

**Annotation outside the text.** Corpus linguistics and the TEI community use standoff annotation, where the annotation sits in a second file and addresses the text by offset. Standoff exists for one reason, which is that inline markup cannot express overlapping hierarchies. Its documented cost is pointer fragility, because an edit to the base text invalidates the offsets.

That pair decides the sidecar on its own. A relation is not a span, so the problem that standoff solves does not arise here. Only its cost would arrive.

## What the industry shipped

[Principle 10](../spec/00-vision-and-scope.md#design-principles) asks for at least one observed application. Four exist, and each option has one.

**DITA relationship tables are the sidecar, shipped for two decades.** OASIS DITA keeps topic-to-topic relationships in a `<reltable>` inside the map rather than in the topics. It buys one place to read and edit the link structure, and topics that stay reusable across maps. It costs the topic its own relationships. An author cannot see them while they write, and the same topic in a second map has different ones.

That last property is the decisive one, and it is a correct design for DITA. A DITA relationship belongs to the *assembly*. A Headwater relation is an assertion about two documents, and it holds in every context that contains them. The sidecar models the wrong owner.

**Semantic MediaWiki is the typed prose option, shipped since 2005.** Its `[[property::Target]]` syntax annotates a link in place, and the wiki text is also the entry form for the database. It works. It also ships `{{#set:}}` for a fact with no prose home. Two surfaces therefore state the same class of fact, and either one may carry it. That is the exact condition that spec 1 outlawed for facets.

**OKF is the prose option arrived at independently, and it dissolves on inspection.** [HW-EVAL-adjacent-work §I.1](../evaluations/adjacent-work.md#i1-okf--the-same-substrate-arrived-at-independently) records relations as Markdown links in the body, written as `- depends_on: [category/key](path.md)`. [§I.2](../evaluations/adjacent-work.md#i2-the-arrow-points-the-other-way-and-that-is-the-whole-difference) explains why this is not evidence for the prose option. OKF is an export, and its durable store is a `knowledge.json`. Its prose links are a serialization of edges that a database owns. They are not a place where an author declares one.

**The note-taking tools converge on the split we are about to make.** Obsidian, Roam, Dendron, and Foam all put untyped links in prose and derive backlinks from them. The tools that then want a query surface, Dataview among them, read structure out of YAML front matter instead. Untyped reference in the body, typed assertion in the front matter, is where that ecosystem arrived without planning to.

## Front matter is the third position

Neither tradition had it. The links sit inside the document, so they travel with it and no second artifact can go missing. They also sit in a separate machine-owned region, so a program may add a reciprocal edge without a decision about anyone's prose.

The two costs are real and neither is decisive. A document with many edges carries a large block at the top. [`taxonomy audit`](../spec/06-engine-architecture.md#taxonomy-validate-versus-taxonomy-audit) already reports edge counts, so an unreadable block is a taxonomy finding before it is an authoring complaint. An author writes a reference twice when the same target belongs in the prose and in the graph. The next section removes most of that second cost.

## Prose links are a different fact

The leaning said that prose links "do not carry relation semantics unless they are annotated". This evaluation cuts the escape hatch, and the cut is the largest change that it makes.

An annotation syntax gives one edge two authoring locations. Three questions follow immediately, and none of them has a good answer. Which location wins when the two disagree? Which span does a finding anchor to? What does `--fix` write when it adds the reciprocal of an annotated link?

The `headwater:` annotation prefix that [Q10](../spec/09-open-questions.md#q10--naming) names loses this use. Where else that prefix applies is not Q4's business.

The cut is also a statement about what the two things are. A prose link is evidence that an author found a reference worth making at one point in a text. A relation is an assertion about two documents that holds wherever they are read. To make every incidental "see also" into a governed edge misstates most of them.

**The friction becomes a check instead of a second declaration.** Parse already extracts every link. A prose link that resolves to a corpus document with which the source declares no relation raises an advisory finding. The author writes the link once, and the fix writes the declaration.

That fix is mechanical only when the relation type is unambiguous, which [spec 12](../spec/12-check-layer.md#fixability) requires. It is unambiguous when exactly one enabled relation type permits the pair of kinds at the two ends. The endpoint declaration is the single source of permitted pairs, which [spec 2](../spec/02-taxonomy-model.md#endpoints-are-the-only-permission) settled when it removed `relations.may`. When two or more types permit the pair, the finding lists the candidates and carries no patch.

**The check has no converse.** A declared relation whose target the prose never names is not a finding. A succession edge belongs in no paragraph, and most structural edges are the same.

**Resolvability stands unchanged.** A prose link that resolves to nothing is a structural finding, exactly as the original leaning said.

## The instance is an object

Q18 and Q20 both need per-instance data, so a relation entry needs a long form. The short form stays, because most edges carry nothing.

```yaml
relations:
  supersedes: ACME-DR-0031
  cites: [STD-ACME-0007, STD-ACME-0012]
  conflicts_with:
    - to: ACME-DR-0044
      adjudicated_by: J. Baxter
      adjudicated_on: 2026-07-14
```

The scalar is sugar for a mapping whose only key is `to`. Both forms produce the same edge, and the engine reports on one shape.

The adjudication pair in that example is superseded, and the example stays as written because this document is a record of what Q4 decided. [Q18](../spec/09-open-questions.md#q18--recording-adjudicated-disagreements) later refused the edge-as-node change that Q4 offered it, and an adjudication is now a document with an `overrides` edge ([evaluation](warrant-and-adjudication.md)). The long form and the owning-end rule are unaffected.

**Relations sit under one key.** A relation name and a facet name come from separate declarations. Nothing stops a taxonomy from declaring both a facet and a relation called `owns`. Under a flat front matter, that collision is unresolvable, and it appears in one adopter's corpus rather than in the meta-schema. A single `relations:` block also gives `--fix` one region to rewrite and gives edge findings one span root.

**Targets are identifiers, never paths.** [Spec 8](../spec/08-design-departures.md#7-identifier-namespacing-arrives-late) makes every identifier namespaced and globally resolvable, and [spec 3](../spec/03-authoring-and-lifecycle.md#identifiers) never reuses one. A path is a location, and a location moves. An anchor target is the anchor string, which its resolver normalizes.

**Edge identity is a triple.** The source identifier, the relation name, and the normalized target identify an edge. List order does not affect it. A repeated triple in one document is an error, for the reason that [Q2](../spec/09-open-questions.md#q2--schema-format) makes a duplicate YAML key an error. This is what makes the `Edge` scope of spec 12 well-defined and its cache key stable.

**Instance attributes are declared, not free.** A relation type declares the attributes that its instances may carry, as a kind declares its facets. An undeclared attribute is a finding. Without that rule, the long form becomes the ungoverned side channel that the previous section just closed.

**An attribute takes a facet's value space, and never a reference.** The choices are a free scalar, a date, an enum with a controlled vocabulary, or a list of those. Spec 1 removed reference-valued facets as a second ungoverned edge mechanism, and a reference-valued attribute would be a third. A connection is a relation.

That rule constrains Q18 rather than serving it, and the constraint is deliberate. An adjudicator named in an attribute is text that no resolver claims, which is the same status that spec 1 gives a ticket number. An edge that must point at a node is a request to make the edge a node. That is a change to the model, and it belongs in Q18 as one, rather than arriving as an attribute type that nobody argued.

**An attribute has an owning end.** A relation type declares whether an attribute belongs to the source, to the target, or to the edge. A source-owned attribute on a symmetric relation gives two values, one per direction, which is correct for a cue. An edge-owned attribute declared with two different values at the two ends is a finding, and no fix resolves it, because that is a judgment.

That last ruling unblocks Q20 without deciding it. A cue is a source-owned attribute, so the referring end writes it, which is what Serena's convention asks for and what foraging theory supports.

**`created_by` stays on the type.** [Spec 2](../spec/02-taxonomy-model.md#who-creates-each-edge) declares it per relation type, and `taxonomy audit` measures whether the declared intent holds in a real corpus. A per-instance value would answer a different question and would make that measurement meaningless. [Q19](../spec/09-open-questions.md#q19--inbound-integration-an-external-system-of-record) may still want per-instance provenance for an imported edge. The instance-attribute surface is where it would go, and Q4 does not put it there.

## The projection has to reify

An edge with attributes is a property graph edge. A plain RDF triple cannot hold one. That is why Wikidata reifies every statement into a node before it attaches a qualifier or a reference. ISO GQL and the openCypher family give edges properties natively, and the RDF 1.2 work adds triple terms for the same need.

So the RDF projection of [Q6](../spec/09-open-questions.md#q6--where-the-corpus-graph-lives-at-rest) and [Q13](../spec/09-open-questions.md#q13--linkml-and-shacl-as-substrate) must reify any edge that carries an attribute, and the round-trip fidelity tests must cover attributes. This is a consequence rather than a new decision. It also sharpens the standing ruling that the Markdown is the corpus. The internal model is a property graph, and RDF is the lossy direction.

## What this predicts, and how to measure it

[Principle 11](../spec/00-vision-and-scope.md#design-principles) forbids an inherited claim of efficacy, and this evaluation makes one testable claim.

The prediction is that promotion of a prose link raises the count of author-attributable edges without a rise in hand entry. The instruments exist. [Spec 3](../spec/03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) records the assisted fraction over required relations, and [`taxonomy audit`](../spec/06-engine-architecture.md#taxonomy-validate-versus-taxonomy-audit) reports edge counts by creator. Until a corpus runs long enough to show the trend, the claim is unmeasured.

The original leaning asked for a different signal, and it stays open in a narrower form. Suppose that authors declare the same link twice at a rate that the promotion fix does not absorb. Then the front-matter ruling is wrong, and the annotation question comes back.

## Consequences for the specification

Seven changes follow, and all are applied.

| Where | Change |
|---|---|
| [Spec 1](../spec/01-conceptual-model.md#relation) | The authored form of a relation instance: the `relations:` block, the short and long forms, identifier targets, and edge identity |
| [Spec 1](../spec/01-conceptual-model.md#relation) | Prose links are extracted and never declare a relation |
| [Spec 2](../spec/02-taxonomy-model.md#the-decision-relation-vocabulary) | A relation type declares its instance attributes, each with an owning end |
| [Spec 4](../spec/04-assurance-model.md#declaration-moves-the-boundary) | The undeclared-prose-reference check, at advisory posture |
| [Spec 12](../spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on) | Edge identity is the triple, which is what the `Edge` scope keys on |
| [Q6](../spec/09-open-questions.md#q6--where-the-corpus-graph-lives-at-rest) | The RDF projection reifies an edge that carries an attribute |
| [Q20](../spec/09-open-questions.md#q20--where-scent-lives) | The cue has a home, and the owning-end rule answers one of its three questions |
