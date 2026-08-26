---
id: HW-EVAL-graph-export-and-federation
status: current
status_since: 2026-08-10
last_verified: 2026-08-10
summary: The Q6, Q13 and Q9 evidence, which is where the corpus graph lives, what it exports, and how a second repository consumes it.
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
    - HW-SPEC-engine-architecture
    - HW-SPEC-distribution-and-federation
    - HW-SPEC-check-layer
    - HW-SPEC-glossary
---

# The graph, its export, and the tier above it

This evaluation closes three entries at once: [Q6](../spec/09-open-questions.md#q6--where-the-corpus-graph-lives-at-rest) (where the corpus graph lives at rest), [Q13](../spec/09-open-questions.md#q13--linkml-and-shacl-as-substrate) (LinkML and SHACL as substrate), and [Q9](../spec/09-open-questions.md#q9--multi-repository-corpora) (multi-repository corpora).

The house pattern is one evaluation per question, and this one departs from it. The reason is the first finding below.

## Why the three did not close separately

Read the three leanings in sequence and a cycle appears.

Q6 defers: an RDF view is acceptable, but the shape of it waits on Q13. Q13 defers twice: the emitter staging is open, and the OKF half arrives "after the graph export format is stable ([Q9](../spec/09-open-questions.md#q9--multi-repository-corpora))". Q9 defers back: confirm the federation model "before the graph export format is frozen, because that format is the aggregator's input".

Each entry waits for the other two. No entry owns the format that all three want. The deadlock is not an accident of drafting. It happened because the three are one question, asked at three radii.

- **Q6** asks it inside the engine. What does the engine hold between the parse and the check?
- **Q13** asks it at the corpus boundary. What may leave, in whose vocabulary?
- **Q9** asks it above the corpus. Who consumes what left, and what may they conclude from it?

The single question underneath is this: **what leaves the corpus graph, and what does the departing artifact owe about what it left behind?** Every ruling below is a corollary of one answer to that. Three separate documents would have restated the answer three times and drifted.

## What the specification already fixed

Twelve rulings constrain this evaluation, and it may not revisit any of them.

**The Markdown is the corpus.** [Spec 0](../spec/00-vision-and-scope.md#non-negotiables) forbids a proprietary store and requires that the corpus degrades to readable Markdown. [Q17](../spec/09-open-questions.md#q17--governed-access-and-the-solution-layer) refused the inversion of that on four grounds and named the coherent alternative a pivot.

**No network at check time**, and the same verdict on a laptop as in continuous integration.

**Determinism.** Same corpus, same lock, byte-identical output ([spec 6](../spec/06-engine-architecture.md#implementation-constraints)).

**One source of truth per fact, not per store.** [Q9](../decisions/0009-multi-repository-corpora.md#the-aggregator-authors-its-own-facts) already applied [principle 2](../spec/00-vision-and-scope.md#design-principles) to this exact case and concluded that nothing is authoritative *as a store*.

**Derived artifacts are regenerable, and the system checks them against regeneration** ([principle 3](../spec/00-vision-and-scope.md#design-principles)).

**Explicit incompleteness** ([principle 5](../spec/00-vision-and-scope.md#design-principles)), and its operational form in [spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for): the census fixes the denominator before any check runs.

**The internal model is a property graph.** [Q4](../spec/09-open-questions.md#q4--relation-storage) made a relation instance an object with declared attributes.

**An edge is identified by the source identifier, the relation name, and the normalized target** ([Q4](../spec/09-open-questions.md#q4--relation-storage)). No position and no path is part of that identity.

**Findings anchor to a line**, and [spec 12](../spec/12-check-layer.md#findings) states that spans do not survive an RDF round trip.

**Headwater owns the schema language, and standard formats come out of it** ([Q2](../spec/09-open-questions.md#q2--schema-format), [Q13](../spec/09-open-questions.md#q13--linkml-and-shacl-as-substrate)).

**The solution layer is an ordinary corpus** that authors its own cross-estate facts ([Q9](../decisions/0009-multi-repository-corpora.md#the-aggregator-authors-its-own-facts)).

**A projection is generated, checked, and declared** ([spec 1](../spec/01-conceptual-model.md#projections)).

Those twelve settle more of the three questions than any of the three entries noticed.

## Prior art, and what practitioners shipped

[Principle 10](../spec/00-vision-and-scope.md#design-principles) asks for the research literature and at least one observed industry application. Ten sources bear on this group. They are grouped by the radius that each one addresses.

### A derived index is a transmission format, not a store

**SCIP replaced LSIF at Sourcegraph, and its design document states the rule outright.** "SCIP is meant to be a *transmission* format for sending data from some producers to some consumers. It is not meant as a *storage* format for querying." LSIF failed on the other half of the same distinction. It encoded a graph with opaque global integer identifiers. That imposed an ordering constraint on how symbols entered the index and made partial update of one document impractical. Sourcegraph replaced the integers with human-readable string symbol identifiers.

That last detail confirms a Headwater ruling that arrived for an unrelated reason. [Q4](../spec/09-open-questions.md#q4--relation-storage) made an edge identity a triple of stable strings, to give the `Edge` scope a computable key. Sourcegraph reached the same shape from incremental indexing. Two derivations, one result.

**Software Heritage keeps two derived representations at two fidelities.** The archive exports its tables as Apache ORC, and a separate compression pipeline builds a WebGraph representation *from that export*. The compressed graph is regenerated rather than maintained. This is the layering that Q6 needed and did not have: an authority, a faithful export, and a lossy fast representation downstream of the export.

**Package registries put an index in git, and every large one migrated away.** Cargo moved to a sparse HTTP protocol, and by April 2025 about 99% of crates.io requests used it. Homebrew moved to JSON downloads in 4.0.0 after `.git` directories reached about 1 GB. CocoaPods moved to a content delivery network in 1.8, with 16,000 directories in one folder as the cause. Go added `GOPROXY` as the default in 1.13. One reported dependency resolution fell from 18 minutes to 12 seconds.

This contradicts the casual half of the Q6 leaning. It offered "a committed JSON graph for anyone who wants to build on it" with no cost attached. The cost is real, and it is churn and size rather than principle. It does not bite at a thousand documents. It bites at the tier that harvests many corpora, which is Q9.

**Backstage rebuilds its catalog rather than storing an authored one.** Entity descriptors live in the repositories that they describe. The catalog's processing loop re-derives entities continuously, and an edge that a later pass no longer emits is severed. The `relations` field is read-only and generated by processors. Backstage also admits entities registered as static configuration, which is [Q9](../decisions/0009-multi-repository-corpora.md#the-aggregator-authors-its-own-facts)'s "the aggregator authors its own facts" already in production.

**CodeQL is the honest contradiction.** A CodeQL database is created from source, uploaded as an artifact, and *is* the query surface. A derived store as the query surface is a shipped design at very large scale. What CodeQL never does is treat the database as canonical for the code, and nobody reviews a database in a pull request. Headwater declines the pattern on its own stated constraints — offline, deterministic, reviewable in a diff — and not on a claim that the pattern fails. That distinction belongs in the record.

**OKF and TrustGraph point the other way and are already recorded.** [HW-EVAL-adjacent-work §I.2](../evaluations/adjacent-work.md#i2-the-arrow-points-the-other-way-and-that-is-the-whole-difference) and [§J](../evaluations/adjacent-work.md#j-trustgraph--the-same-pitch-the-opposite-mechanism) hold a durable graph store with text projected out of it. Both work. Neither shares spec 0's non-negotiables.

### Model-first emission, and what happens when emitters chain

**SPDX 3.0 is option 3 at standards scale.** One model generates an OWL ontology with SHACL shape restrictions, a JSON-LD context, and, through `shacl2code`, a JSON Schema. The serializations are derived rather than authored in parallel. That is the architecture Q13 leans toward, shipped by a standards body with many independent consumers.

**LinkML's own SHACL generator is lossy inside the shape layer.** A 2024 report on building a semantic data link records that the generator "inadequately translates defined constraints within the schema". It names `any_of` and `equals_string_in` as constructs that do not survive. The project's own issue tracker carries further cases.

This is the strongest single result in the group, and it is stronger than Q13 claimed. Q13 argued that a pipeline routed through LinkML drops the *graph* layer, because LinkML never expressed it. The measured fact is that such a pipeline also drops parts of the *shape* layer that LinkML does express. A chain inherits every loss of every hop, and declares none of them.

**OGC API Features and STAC ship the subset declaration that Q13 asked for.** An OGC API implementation serves a `/conformance` endpoint that lists the conformance class identifiers it supports. A listed identifier obliges the full capability behind it. STAC moved the same list to the landing page as `conformsTo`, so that one request tells a client what it is talking to.

They sharpen the requirement rather than supply it. Both declare a positive set against a *fixed, published* universe of classes. Headwater's universe is the check registry that an adopter's own taxonomy generates, so an outside reader cannot compute the complement. The export therefore has to carry both halves, and that is a real difference rather than a copy.

### Harvest, not fan-out

**SPARQL federation is the standards-track version of query fan-out, and its documented behavior is the failure that spec 4 forbids.** Under SPARQL 1.1 and 1.2, a `SERVICE` pattern that cannot reach its endpoint fails the whole query. The remedy in the specification is the `SILENT` keyword, which makes the query succeed with a silently incomplete answer. Research on public endpoints reports that low reliability pushes serious consumers onto dataset dumps and local reinstallation. Wikidata maintains a page of federation issues that lists downtime, timeouts and protocol incompatibilities among its federated endpoints.

**Digital libraries ran this experiment for two decades and the harvesters won.** The two candidate architectures were distributed broadcast search over Z39.50 and metadata harvesting over OAI-PMH. Europeana aggregates from more than 3,700 providers through its Metis ingestion service, partly through intermediary aggregators. DPLA harvests over the same protocol. ResourceSync, built on sitemaps, is the successor for the cases that need faster synchronization.

The tiered shape of Europeana matters here. Providers feed intermediary aggregators, and those feed the top tier. That is [spec 7](../spec/07-distribution-and-federation.md#federation)'s federation tiers, arrived at by an unrelated community under load.

**GraphQL federation is the counter-example, and it proves the rule.** Apollo's router does fan out at query time, and it works. It works on three conditions. Composition of the subgraph schemas is a **build-time** step that produces a static supergraph artifact. Entities join on a declared `@key`, and a cross-subgraph reference carries only the key fields. Every subgraph is a live service under one operator.

Headwater meets none of the three. What transfers is the build-time half. Composition errors surface when the supergraph is composed, not when a query runs. The Headwater equivalent is resolution and mapping validation at the aggregator tier, before any question is asked.

## The decision

### Q6 — the corpus graph has no resting place

The entry asks where the graph lives at rest. The answer is that it never rests. The question that matters is a different one: which artifacts are derived from the graph? How long does each one live, and what may each one claim?

Three artifacts, and none of them is canonical for anything.

| Artifact | Lives for | Committed | Canonical for |
|---|---|---|---|
| The in-memory graph | one run | never | nothing |
| The cache | until its inputs change | never, and it is ignored by version control | nothing |
| An export | until it is regenerated | if the taxonomy declares an output path for it | nothing |

**The cache is disposable by test, not by intention.** `headwater check --no-cache` produces output byte-identical to `headwater check`. A cache that can change a verdict is a store wearing another name. The test is cheap, it runs in the engine's own suite, and it is the only thing that keeps the distinction true under maintenance.

**No embedded database, and the trigger to revisit is named.** The refusal is not on principle. The [Q1 spike](language-spike-results.md) ran a warm change-scoped pass over 1,000 documents in about 2 ms, against a 200 ms budget. Nothing in the current design asks a question that the in-memory graph cannot answer inside [spec 6](../spec/06-engine-architecture.md#performance-targets)'s budgets. The trigger is a named query workload that misses a budget, measured rather than anticipated.

**`headwater export` is a projection, declared like every other projection.** This moves the "should it be committed" question out of the engine and into the taxonomy, which is [principle 1](../spec/00-vision-and-scope.md#design-principles). An adopter who declares an output path gets an export that `generate --check` holds to regeneration. An adopter who declares none gets no file.

**Two classes of export, and only one of them preserves fidelity.** The native graph export carries the property graph without loss, edge attributes included. Every interoperability export is lossy by construction: RDF, SKOS, LinkML, SHACL, JSON Schema, OKF. Q6's leaning half-said this when it called RDF "the lossy direction". The ruling makes it a class distinction rather than a remark about one format.

**A round-trip test is the wrong instrument for a lossy export, and a census replaces it.** This is the largest correction that Q6 receives. You cannot round-trip a projection that is lossy by design, so the standing promise of "round-trip fidelity tests" for RDF was never satisfiable.

What replaces it is the doctrine that [spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for) already applies one layer in. Every emitter declares a **loss set**: the node classes, edge classes, and attributes that its target vocabulary cannot carry, each with a reason. Every export run emits a **projection census**. Every node and every edge in the graph is either present in the output or accounted for by a declared loss reason. An unaccounted omission is a projector defect, and it fails the run.

The native export keeps the round trip, because its declared loss set is empty and a round trip is exactly what proves that claim.

This answers the [SHACL evaluation](shacl-worked-example.md#problem-one-everything-downstream-trusts-the-projection-and-shacl-does-not-check-it) directly. The projector was the most trusted component in the pipeline, and nothing downstream could detect its mistakes. Now the projector is checked against the graph, by the engine, with no help from any consumer.

### Q13 — LinkML is not the substrate, and it is not the compiler either

The leaning survives on its ownership half and loses its architecture. Headwater owns the language, and standard formats come out of it. What the entry still assumed is a pipeline with LinkML near its head. That assumption is wrong.

**Every emitter reads the resolved lock and the graph. No emitter reads another emitter's output.** Q13 already stated a version of this as a trap to avoid, and the evidence makes it a rule with a measurement behind it. LinkML's SHACL generator drops constructs that LinkML itself expresses. A chained pipeline would inherit that loss and declare nothing. That is the "clean run while believing it checked everything" failure that the subset declaration exists to prevent.

**The staging order, derived rather than preferred.** An emitter ships when its consumer exists. Two consumers exist today and the rest do not.

| Order | Emitter | Ships when | Consumer |
|---|---|---|---|
| 1 | JSON Schema for front matter | first release | the adopter's own editor, through a language server |
| 2 | Native graph JSON | first release | the solution corpus of Q9, and any local tool |
| 3 | SHACL | a named external consumer asks | a validation stack with no Headwater installation |
| 4 | RDF and SKOS | the same trigger | knowledge-organization tooling |
| 5 | OKF | the same trigger | LeanCTX and what reads its bundles |
| 6 | LinkML | the same trigger | LinkML's own generator fan-out, for OWL, Pydantic and SQL |

**LinkML lands last, and the entry's title is what obscured that.** "LinkML and SHACL as substrate" put the two together and put LinkML first. Once emitters may not chain, LinkML stops being the route to JSON Schema and SHACL, and becomes one sibling among six. It is the sibling with the weakest case, because its distinctive value is a generator fan-out for formats that no named consumer has asked for.

**`exportable_as` is a set per target, with a partition rule and an equivalence bar.** [Spec 12](../spec/12-check-layer.md) promised that the declared subset "cannot drift from the truth". Three rules make that true.

- A check declares the emitter targets that it exports to. `none` is legal, and it is the common case.
- The exported set and the unexported set **partition** the check registry, and the engine generates both from it. Neither list is authored, so neither can drift from the other.
- A target may appear only when the emitted constraint is **equivalent** to the native check over the exported graph. Equivalence is established by the differential test that Q13 already wanted for other reasons. A stock validator over the projection produces the same finding set as the engine, for that check, over the fixture corpus. A partial translation is declared unexported. A partly-true claim of coverage is worse than a claim of none.
- The declaration travels **with** the artifact and never beside it. STAC moved `conformsTo` to the landing page so that one request answers the question. An export that a consumer copies has to carry its own limits.

**The OKF half is not a separate question, and the entry misfiled it.** Q13 separated OKF from the substrate question correctly, on the grounds that one is about the TBox and the other about the ABox. It then left OKF inside Q13. But the native graph export is an ABox emitter too, and Q6 owns it. So OKF is a second ABox emitter beside the native one, on the same terms as every other. It reads the graph directly, declares a loss set, and emits a census.

That placement makes the standing worry concrete. OKF's own conformance check is four advisory warnings, so a consumer that validates a bundle verifies almost nothing. The census is the answer. It is the emitter's own account of what it dropped, produced by the party that knows. It is checked in the emitting repository rather than trusted by the receiving one.

### Q9 — harvest, and there is no merged graph

Three points were open. All three close, and one of them closes by fixing a sentence in [spec 1](../spec/01-conceptual-model.md#the-corpus).

**The monorepo half was answered all along, in the wrong words.** Spec 1 said that a repository has exactly one corpus. The binding constraint is not the repository. A corpus is one taxonomy and one root, and a repository holds one or more of them. A monorepo with several independent documentation sets is not one corpus under strain. It is several corpora that share a working tree.

Nothing in the engine changes. The lock is per corpus root. The census walks one root. A path resolves to exactly one corpus by the determinism rule that shelf patterns already use. The most specific root wins, and a tie is a validation error. Identifiers are namespaced at minting ([spec 3](../spec/03-authoring-and-lifecycle.md#identifiers)), so two corpora in one tree cannot collide. The sentence named the wrong container, and that is the whole defect.

**The aggregator is in scope, and it is not a component.** It is a solution corpus plus one anchor kind. [Q9](../decisions/0009-multi-repository-corpora.md#the-aggregator-authors-its-own-facts) already admitted the solution layer as an ordinary corpus. What was missing is how it reaches the corpora below it, and the answer is machinery that [spec 2](../spec/02-taxonomy-model.md#behavior-at-the-limits) already has. An anchor kind is declared, and exactly one resolver owns it. That resolver reads pinned, committed exports, in the way that `code_path`'s resolver reads a source tree and [Q19](../spec/09-open-questions.md#q19--inbound-integration-an-external-system-of-record)'s resolver reads a committed snapshot.

**There is no merged graph, and the question about where merged graphs live dissolves.** Merging is anchor resolution. Anchor resolution produces nothing that outlives the run. The solution corpus holds its own documents and its own declared edges, and it resolves anchors against exports that it pinned. Q9 had already concluded that a merged graph is "canonical for nothing". The step it did not take is the obvious one. An artifact that is canonical for nothing, that nothing reviews, and that every run can rebuild, does not need to exist.

**Harvest, never fan-out.** The solution corpus pins each source export by identity and content hash. It never queries a live endpoint. Three independent arguments agree.

- [Spec 0](../spec/00-vision-and-scope.md#non-negotiables) forbids a network dependency at check time, and a fan-out query is exactly one.
- [Spec 6](../spec/06-engine-architecture.md#performance-targets) gives a route query 100 ms. A fan-out across estates does not fit, and an agent-facing call that does not fit is a call that developers remove.
- SPARQL's `SILENT` is the documented shape of what fan-out does when a source is unreachable. It returns a smaller answer and reports nothing. That is the silent pass that [spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for) exists to forbid.

So a source export that the aggregator cannot read is a **finding that names the pin**. It is never a narrower answer delivered without notice.

**A pin is the third instance of one pattern, and that is worth saying once.** A taxonomy pin ([spec 7](../spec/07-distribution-and-federation.md#consuming)), a requirements snapshot pin ([Q19](../spec/09-open-questions.md#q19--inbound-integration-an-external-system-of-record)), and a source-export pin are the same mechanism. Each fetches out of band, commits the result, checks against the committed copy, and compares on a schedule. Each raises a change proposal and never a mutation. [Spec 7](../spec/07-distribution-and-federation.md#upstream-awareness)'s upstream awareness covers all three with no change.

## What this fixes for Q17, which it does not decide

[Q17](../spec/09-open-questions.md#q17--governed-access-and-the-solution-layer) states that it blocks on the graph export format. The format is now fixed, and four things follow that Q17 may rely on.

**The export is the serving artifact.** Filtering acts at export, never at graph build. Q17 already required that checks stay privileged and total, and it now has a named seam rather than an intention.

**The census is the mechanism for the tombstone rule.** Q17 requires that a filtered view reports "3 documents withheld" and never looks complete. A redaction is a loss with a reason, and the projection census already reports exactly that shape. Q17 needs no new machinery for its most important constraint.

**Harvest moves the filtering point, and this is the constraint that Q17 most needs.** The aggregator holds bytes that a publishing corpus gave it. A filter applied at the aggregator's read step is applied after the bytes crossed the boundary, which is [Serena's failure](../evaluations/adjacent-work.md#l6-a-filter-in-the-tool-layer-is-advisory-and-the-documentation-says-so) restated at a different layer. Filtering belongs to the publishing corpus's export step.

**`exportable_as` and the loss set give a redaction somewhere generated and checked to live.** Q17 does not have to invent a declaration surface.

Q17 keeps everything else: whether to build access control at all, the identity model, the topology-leak analysis, and the exception to [principle 4](../spec/00-vision-and-scope.md#design-principles).

## What stays open

**No named consumer exists for emitters 3 through 6.** That is deliberate and it is the trigger, not an oversight. If none ever appears, four emitters are never written and nothing upstream changes.

**Whether a solution corpus vendors each source export or references it.** A vendored copy keeps checks offline and grows the repository. A reference keeps the repository small and puts a fetch before the check. The evidence that closes this is the size of one real harvest, and no aggregator exists yet.

**Whether the aggregator tier needs conformance rules of its own.** `headwater conformance` ([spec 7](../spec/07-distribution-and-federation.md#conformance)) evaluates one repository. Whether a tier that harvests owes a separate set is unargued, and nothing depends on it today.

**The `$`-reference grammar** stays where [Q2](../spec/09-open-questions.md#q2--schema-format) left it. It is not this group's to settle, and the emitters do not touch it.

## What this predicts, and how to measure it

[Principle 11](../spec/00-vision-and-scope.md#design-principles) forbids an inherited claim of efficacy. Four claims here are testable, and every one is unmeasured today.

| Claim | Instrument | Status |
|---|---|---|
| Rebuild plus cache meets spec 6's budgets, so no database is justified | the change-scoped benchmark from the [Q1 spike](language-spike-results.md), run over a real corpus | measured at spike scale only, on generated documents |
| Emitted JSON Schema lowers the rate of invalid front matter that reaches a check | finding rate for Shape-origin rules in the coverage report, before and after the emitter ships | unmeasured |
| The projection census catches projector defects that no consumer can catch | the differential test against a stock validator, plus the projector's fixture tree | unmeasured, and no emitter exists |
| Harvest keeps a solution-tier route query inside the 100 ms budget | route latency at the solution tier, against the same target | unmeasured |

The second claim is the one to watch, because it is the argument that puts JSON Schema first. If editor-side validation moves no finding rate, the staging order is wrong and SHACL has as good a claim to the first slot.

## Consequences for the specification

Fourteen changes follow, and all are applied.

| Where | Change |
|---|---|
| [Spec 1](../spec/01-conceptual-model.md#the-corpus) | A corpus is one taxonomy and one root. A repository holds one or more, and a path resolves to exactly one |
| [Spec 1](../spec/01-conceptual-model.md#the-corpus) | The graph is rebuilt every run. The cache is disposable, and a cache that changes a verdict is a defect |
| [Spec 2](../spec/02-taxonomy-model.md#behavior-at-the-limits) | An anchor resolver reads repository content or a committed snapshot, and never a live service |
| [Spec 6](../spec/06-engine-architecture.md#nothing-stores-the-graph) | The three derived artifacts, their lifetimes, and what each may claim |
| [Spec 6](../spec/06-engine-architecture.md#nothing-stores-the-graph) | No embedded database, with the revisit trigger stated |
| [Spec 6](../spec/06-engine-architecture.md#projections) | The export is a declared projection. Two classes of export, the loss set, and the projection census |
| [Spec 6](../spec/06-engine-architecture.md#checks) | The `Exportable as` column becomes a target set, and points at spec 12 |
| [Spec 6](../spec/06-engine-architecture.md#cli) | `headwater export` joins the command list, which never carried it |
| [Spec 7](../spec/07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it) | The aggregator is a solution corpus plus one anchor kind. There is no merged graph |
| [Spec 7](../spec/07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it) | Harvest and never fan-out, with an unreadable pin as a finding |
| [Spec 7](../spec/07-distribution-and-federation.md#upstream-awareness) | One pin pattern, three instances |
| [HW-EVAL-adjacent-work §N](../evaluations/adjacent-work.md#n--transmission-harvest-and-the-declared-subset) | The ten sources above, and what each confirms, sharpens, or contradicts |
| [Spec 12](../spec/12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule) | `exportable_as` as a target set, the partition rule, and the equivalence bar |
| [Spec 12](../spec/12-check-layer.md#the-correctness-roots) | The graph projector's correctness root is the census, and the Q6 reference is updated |

The [glossary](../spec/glossary.md) gains **export**, **loss set**, **projection census**, and **pinned export**, and its **corpus**, **cache**, and **aggregator** entries are corrected.
