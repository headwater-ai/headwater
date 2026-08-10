# 11 — Adjacent work and tooling

[Spec 10](10-theoretical-foundations.md) tested the design against the research literature. This document does the same against **practitioner work and existing tooling**. These are projects that solve adjacent problems, and standards that may already implement parts of what spec 2 describes.

The question here is narrower and more uncomfortable: *what did we specify that someone already built?*

---

## A. TBox and ABox — the right name for a split we already made

Description logic separates a knowledge base into two parts. The **TBox** (terminology box) holds the schema: classes, their hierarchy, and the relations that may hold between them. The **ABox** (assertion box) holds instances: individuals, their attributes, and the actual relations asserted between them.

That is exactly Headwater's split, and we described it in invented vocabulary:

| Headwater | Description logic |
|---|---|
| Taxonomy (kinds, facets, relations, regimes) | TBox |
| Resolved taxonomy lock file | Compiled TBox |
| Corpus (documents and their typed edges) | ABox |
| `taxonomy validate` | TBox-internal consistency |
| `taxonomy audit`, `check` | ABox against TBox |

Adoption of the standard names costs nothing and buys precision. It also explains why the `validate` / `audit` split arrived at [spec 6](06-engine-architecture.md#taxonomy-validate-versus-taxonomy-audit). That split is TBox reasoning versus ABox reasoning. This is a real distinction with decades of theory behind it, not a convenience that we invented.

**One boundary to state plainly.** Headwater's ABox is the graph of *documents and their declared edges* — not the claims inside the prose. "This service returns 404 on a missing key" is a sentence in a document. The system knows that the document exists, what kind it is, and what it governs, but not what it asserts. To pretend otherwise is to promise semantic consistency checks that we cannot deliver. The ABox stops at the document boundary, and that limit belongs in the specification rather than in a footnote that someone discovers later.

> **Applied:** TBox/ABox vocabulary adopted in specs 1, 2 and 6, with the ABox boundary stated explicitly.

### A.1 The solution layer presses on that boundary

The boundary above is easy to hold while every node is a document. A cross-repository **solution layer** ([Q9](09-open-questions.md#the-aggregator-authors-its-own-facts)) is where it comes under pressure. The pressure is quiet enough that people can cross the boundary by accident.

When you model the estate above individual repositories, the temptation is immediate: nodes for services, interfaces, data flows, capabilities. These are not documents *about* those things, but the things. That is a different system. A `Service` node that asserts a service's properties leaves the corpus and starts to model the world. The obligation follows the assertion: **if you model the world, you will be asked to keep it true.**

Nothing in the current design carries that obligation. The drift failure mode is worse than stale prose, because a wrong node looks structural rather than editorial.

The middle position is available and is probably right. Solution-layer nodes are **declared anchors** — identified things that documents may point at. Each anchor carries an identifier, a name, and ownership. There is no claim that any further property is accurate, and no pretense that the set is complete.

That buys the cross-estate edges that the layer exists for. Every substantive assertion stays inside a document, where the freshness machinery and the check layer already reach it. It is the same move that `code_path` already makes: an external anchor kind that is referenced, never described.

The decision belongs in the specification rather than in whichever schema someone writes first. The ontology-first thesis of §F.1 argues for a step further. An architecture layer is exactly where that step can pay. So this is a genuine choice, not a formality.

> **Recorded in [Q17](09-open-questions.md#q17--governed-access-and-the-solution-layer), alongside the access question that the same layer raises.**

## B. Specification → ontology → implementation (testerstories)

Two posts work a Z-Machine header specification into an RDF/Turtle ontology. They then generate an implementation from it and test the result. It is the most directly relevant practitioner work found. It runs the full pipeline that the "structured knowledge improves machine output" thesis depends on. It also reports honestly on what it did and did not achieve.

What the author does: prose spec → hand-crafted ontology (classes, datatype properties, object properties) → LLM-generated implementation → tests that **query the ontology at runtime** to derive what should be true. Version-conditional behavior that lived in table footnotes becomes explicit graph edges (`applicableFrom`, `supersededBy`). The generated code cites ontology individuals by name in its comments.

Four things transfer directly.

### B.1 The corpus can be a test oracle

Their tests do not hardcode expectations. They ask the ontology what fields should exist for the version found in the file. Coverage adapts because the schema is queried, not copied.

Headwater has the ingredients, but they are not connected. Acceptance criteria carry stable identifiers ([spec 3](03-authoring-and-lifecycle.md#identifiers)). Contract sidecars were credited as prior art but never actually specified. When they are connected, they give the strongest possible form of "the specification describes what is" — the specification *generates the check that proves it*.

> **Applied:** contract sidecars specified in [spec 2](02-taxonomy-model.md), with identified acceptance criteria as an oracle source.

### B.2 Source authority ordering

Their system prompt states a precedence: where the normative specification and the conventional practice disagree, prefer the normative one **and note the discrepancy**. The ontology carries `hasSourceAuthority`. The prompt turns it into a decision rule.

Headwater has no such ordering. Derived reading precedence records whose *purpose* governs, which is not the same question. When a standard and a specification disagree on a fact, nothing in the corpus says which one a reader should believe. That is a genuine hole, and it is exactly the situation where an agent will otherwise pick arbitrarily and sound confident.

> **Applied, then cut.** An `authority` ordering on kinds was adopted here and removed by the core-concepts review. The trigger (detection of factual disagreement) needs a judgment that the ABox boundary refuses to make. A global scalar cannot carry the scoped precedence that their prompt actually encodes. What Headwater keeps is the part of their design that works. It keeps the *decision rule* ("note the discrepancy") as an agent instruction, and the adjudication recorded as data ([spec 2](02-taxonomy-model.md#disagreement-is-adjudicated-not-ranked), [Q18](09-open-questions.md#q18--recording-adjudicated-disagreements)).

### B.3 Generated artifacts cite their source

Comments in the generated code name the ontology individual that justified each decision (`field_font_width_v5`). Any output can be walked back to the artifact that licensed it.

> **Applied:** citation-on-generation for agent-produced content, in [spec 5](05-ai-integration.md).

### B.4 The honest limit

The author is explicit that automated tests confirm *structure* while semantic correctness still needs a human. The font-swap test proves that the labels are present, not that the swap is right.

That is the cohesion/coherence boundary of [spec 4](04-assurance-model.md), reached independently from an entirely different direction. Independent arrival at the same line is the strongest evidence available that the line is real.

### B.5 What they do *not* claim

The author states plainly that local models are not deterministic, that identical prompts produce different outputs, and that the reference implementation exists *because* of that. Determinism is not claimed for the model. What the pipeline buys is that deviation becomes **visible and attributable**.

This matters enough to state in our own terms, below.

## C. LinkML — the uncomfortable one

[LinkML](https://linkml.io/) is a schema language authored in YAML that compiles to JSON Schema, SHACL, RDF/OWL, ShEx, Pydantic classes, SQL DDL, and GraphQL. It also has a validation runtime.

When you read [spec 2](02-taxonomy-model.md) and then the LinkML documentation, the overlap is substantial. The shared features are classes with slots, ranges, cardinality, enums with permissible values, inheritance, and schema-level imports. The Q2 leaning of spec 2 is "YAML plus a published JSON Schema, with the resolved lock in a stricter representation". That describes something that LinkML already implements. LinkML also ships the multi-format compilation that we will otherwise write.

Where it stops:

| Headwater declaration | LinkML |
|---|---|
| Kinds, facets, vocabularies, cardinality | Direct fit — this is what LinkML is |
| Relations with endpoints and cardinality | Direct fit |
| Nuclearity, family | Expressible as annotations, not native semantics |
| Voice and lifecycle regimes, freshness policy | Not a data-shape concern, outside its model |
| Sequences, projections, overlays, core | No equivalent |
| Compatibility measurement | No equivalent |

**That table is wrong, and a [worked example](../evaluations/linkml-worked-example.md) shows why.** An expression of the taxonomy in real LinkML establishes two things that the guess above missed. LinkML already ships three of the twenty research-derived changes — SKOS mappings, PROV alignment, and `recommended` as advisory severity. Its `designates_type` is our heterogeneous-shelf discriminator under another name.

But the boundary is not structural-versus-governance: *reciprocity* fails, and reciprocity is as structural as anything in spec 2. The real line is that LinkML, SHACL, and JSON Schema all validate **one instance against a shape**. Everything that Headwater does that they cannot is a property of the **whole graph, or of the corpus over time**.

That reframes the question from "does LinkML cover enough?" to "is a two-layer architecture — standard shape layer plus Headwater graph layer — better than one custom layer?" Three readings follow:

1. **Adopt it as the substrate.** Author the structural core as LinkML, layer Headwater's governance declarations alongside, and inherit the meta-schema, the validator, and SHACL/JSON-Schema/OWL output. Less to build, a standard that others already read, and automatic interoperability.
2. **Stay independent, borrow the design.** LinkML's target is data models for research and biomedical data. A documentation taxonomy is a different animal, and a bolted-together schema — half LinkML, half ours — may be worse to author than either alone. The cognitive-dimensions walkthrough (Q2) is the instrument for the decision, and "two languages in one file" scores badly on role-expressiveness.

3. **Emit it, do not author in it.** Author in Headwater's language and compile the shape layer *to* LinkML, which then generates JSON Schema, SHACL, OWL and Pydantic through LinkML's own toolchain. One authoring surface, fully validated, with a standards-based export. This option only became visible when we wrote the example out, and it is now the leading candidate.

The decisive evidence against option 1 is mundane: everything Headwater-specific lands in LinkML `annotations`, which are untyped pass-through. LinkML carries them and validates none of them. So for exactly the half that is ours, the meta-schema benefit disappears. Authors face two languages in one file, with no visual cue for which half is checked.

I will not decide this unilaterally. It changes what we build, it is close to irreversible under option 1, and it interacts with the language decision in Q1. LinkML's tooling is Python, which pulls against a Rust core. Option 3 dissolves that tension, which is part of its appeal.

> **Recorded as [Q13](09-open-questions.md#q13--linkml-and-shacl-as-substrate).**

## D. SHACL — the name for schema-derived checks

[Spec 6](06-engine-architecture.md) says that schema-derived checks are *generated from the taxonomy*. SHACL (Shapes Constraint Language, W3C) is the standard that does this for graphs: you declare shapes, validate a graph against them, and get structured violation reports. The corpus graph is expressible as RDF, because it is typed nodes with typed edges. If it is expressed as RDF, then SHACL shapes can *be* the check layer rather than something that we hand-write.

A [worked example](../evaluations/shacl-worked-example.md) confirms that SHACL reaches the whole-graph layer that LinkML cannot. Reciprocity, conflict between two live decisions, satellite inheritance, and windowed sequence expectations are all expressible. Every one of them needs a drop to SPARQL. SHACL Core can traverse, but it cannot refer back to the focus node from the far end of a traversal.

My reservation here — that SHACL's reports are hard to read — was too strong and is withdrawn. `sh:message` with variable interpolation makes messages as good as they are authored. The objections that survive are sharper. **Line numbers, remediation and fixability do not survive the RDF round trip**, and [spec 4](04-assurance-model.md) requires all three to make a finding actionable. Temporal checks also need the evaluation time injected rather than read from the clock, or determinism breaks. This constraint applies to whatever engine we build, not just to this one.

The conclusion is the same as for LinkML, by a different route: a compilation target, not an authoring surface. Folded into Q13.

## E. OpenGEO — same substrate, opposite direction

[OpenGEO](https://github.com/donhaji/opengeo) is a specification for publishers to declare canonical meaning to AI systems. It uses Markdown bodies with YAML front matter, organized as Discovery → Semantic → Context → Execution. Assurance is treated as a concern *around* the chain rather than a layer in it.

Two points of contact:

- **The substrate is identical.** Markdown plus YAML front matter as a semantic contract for machine readers, deliberately *not* RDF/OWL/SHACL. That is an independent data point that the heavyweight stack is not required for this class of problem. It is useful evidence for the Q13 decision, and a caution against the assumption that the standards-based route is obviously correct.
- **Assurance is not a layer.** Their framing puts provenance, authorship, freshness, auditability and ownership as oversight around the whole chain rather than a stage within it. That framing matches [spec 4](04-assurance-model.md) and is a cleaner statement of it than ours.

But the direction is opposite. OpenGEO points **outward**: a publisher declares meaning to third-party engines that it does not control, with execution explicitly out of scope. Headwater points **inward**: an organization governs its own corpus, for its own agents. We control the whole pipeline, and can therefore *check* things rather than merely declare them. Their context layer (tone, persona, interpretation envelope) follows from a lack of control over the consumer. We do control the consumer, so we constrain behavior directly, and do not merely request it.

The transferable gap is **discovery**. OpenGEO takes seriously how a machine reader arrives cold and finds out what a corpus is. [Spec 7](07-distribution-and-federation.md) covers distribution to repositories that already know about us, and says nothing about an agent that encounters the corpus for the first time.

> **Recorded as [Q14](09-open-questions.md#q14--discovery-surface).**

## F. Ontology-first, and the LLM-maintained wiki

There are two related sources: the r/OntologyEngineering community, and Karpathy's *LLM Wiki* pattern that the community identified as convergent with its own position.

### F.1 The ontology-first thesis

The community's stated position is that ontology should come **first**. You build the model of the domain and let agents derive the stack that supports it. Implementation is treated as a consequence of the model, rather than the other way round. Documentation generates the system. It does not describe one that already exists.

That is the testerstories pipeline (§B) generalized into a methodology, and it is further than Headwater currently goes. Our specs *describe* a system that exists. Theirs *generate* one. The two meet at the oracle idea in §B.1. A specification precise enough to test an implementation is most of the way to one precise enough to generate it. To be clear, Headwater's design does not preclude the stronger position, but it does not currently claim it either.

### F.2 Karpathy's LLM Wiki

The pattern is this: raw sources stay immutable. An LLM incrementally builds and maintains a wiki of interlinked Markdown between you and those sources. A schema file (`CLAUDE.md` / `AGENTS.md`) tells the agent how the wiki is structured and what workflows to follow. Operations are ingest, query, and **lint**.

Four things in it matter here, and one of them is a correction to something I wrote.

**The anti-RAG argument, better than mine.** The objection raised is not that retrieval is imprecise — it is that retrieval *accumulates nothing*. Every question re-derives its answer from fragments, and the synthesis is thrown away. If you ask again tomorrow, the work is done again from scratch. Knowledge should be **compiled once and kept current**, not reconstructed per query.

That is a stronger and more durable argument than the precision one that I gave in [spec 5](05-ai-integration.md#what-we-do-not-do). It is the argument that Headwater's whole design rests on: validation, relations, and projections are all compilation steps whose results persist.

A further data point: the pattern reports that a maintained index file works well into the hundreds of pages *without* embedding infrastructure. A governed documentation corpus is squarely in that range.

**Independent arrival at the capture-cost thesis.** The stated reason that wikis die is that the maintenance burden outgrows the value — the bookkeeping, not the thinking, is what people abandon. The reason that this pattern survives is that an LLM does the bookkeeping at near-zero cost. That is Grudin's capture bottleneck and the assisted-fraction answer of [spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric), reached from practice rather than from the literature. Two independent derivations of the same claim are the best support that it will get, short of measurement.

**Lint as a coherence control.** Their lint pass looks for contradictions between pages, claims superseded by newer sources, orphans, and concepts referenced but never defined. Most of that is cohesion, and the engine already does it deterministically. But *contradiction between pages* is not. It is exactly what [spec 4](04-assurance-model.md#cohesion-and-coherence-are-different-obligations) classes as coherence: undecidable structurally, so it needs judgment.

An LLM lint pass is therefore a legitimate **coherence control** — sampled, detective, non-blocking, exactly as spec 4 requires of that class. It does not violate "no LLM in the validation path", because that rule governs *verdicts*. A coherence finding is a prompt for human attention rather than a verdict. This is a real addition: spec 4 named the coherence class and left it thin.

> **Applied:** LLM-assisted coherence sweep added as a control class in spec 4.

**The gap that it exposes: a third content tier.** The pattern's wiki is neither hand-authored nor mechanically generated. It is *synthesized* — an agent's interpretation of sources, which evolves and is revised as new sources arrive. Headwater recognizes only two tiers: authored documents and deterministic projections. Synthesized content fits neither. The difference matters, because a projection can be verified through regeneration and a synthesis cannot.

Whether Headwater should admit a synthesized tier is a genuine question. Such a tier needs its own provenance, its own staleness rules, and a clear statement that it is never canonical for anything. It is also how most organizations will actually want to use this.

> **Recorded as [Q15](09-open-questions.md#q15--a-synthesized-content-tier).**

## G. Knowledge-graph chunking for RAG — a problem we do not have

The TBox/ABox framing (§A) is the valuable half and stands independently of retrieval. The chunking half solves a problem that Headwater does not have. That half covers how to slice a knowledge graph into embeddable pieces, and the trade between class-based and instance-based strategies. We record the reasoning because it also explains *why* we do not have the problem.

Chunking exists because embedding retrieval must reconstitute meaning at query time from fragments chosen by similarity. Every strategy in that article manages a loss that the approach introduces. Class-based chunking orphans cross-class relations, and instance-based chunking fragments schema reasoning. Hybrid approaches are recommended because neither loss is acceptable alone.

Headwater never incurs the loss. Retrieval returns *identified documents reached along declared edges*, so structure is not something to reconstruct — it was never dissolved. There is nothing to chunk because nothing is embedded.

This is the position stated properly rather than assumed, and it is recorded in [spec 5](05-ai-integration.md#what-we-do-not-do).

## H. What structured knowledge actually buys — stated precisely

The goal behind all of this is more deterministic machine behavior. The claim needs a careful statement, because the honest version is narrower than the marketing version and it changes what we should build.

**Structured, governed knowledge does not make a language model deterministic.** Sampling is stochastic, and identical prompts produce different outputs. That is a property of the model, not of its inputs, and no amount of schema fixes it. The testerstories author is explicit about this, and keeps a hand-written reference implementation exactly because of it.

What it does buy, and each buys something that we can build toward:

| Mechanism | Effect |
|---|---|
| **Ambiguity removal** | Fewer legitimate readings of the input, so fewer defensible-but-divergent outputs. Variance narrows. It does not vanish |
| **Oracles** | An output can be *checked* against a declared expectation rather than judged by eye |
| **Attribution** | When output deviates, the artifact that licensed it is identifiable — so the fix lands on the corpus or the prompt, not on a hunch |
| **Reproducible comparison** | A pinned corpus plus a pinned model gives a baseline that a later run can be diffed against |

The achievable target is **bounded, auditable non-determinism**. That is output that varies within a space that the corpus defines, deviations that are visible, and causes that are attributable. This is a substantially more useful goal than determinism, because it survives contact with how these models actually work.

It also sets the investment priority. Effort belongs in **oracles and traceability** — checkable expectations and citation of sources. It does not belong in prompt engineering whose aim is to make the model repeat itself. The first compounds and is measurable. The second is a treadmill.

> **Applied:** stated in [spec 5](05-ai-integration.md#what-structured-knowledge-buys).

## I. LeanCTX — our file format, none of our TBox

[LeanCTX](https://github.com/yvgude/lean-ctx) is a context-engineering layer for coding agents: a local Rust binary that sits between an agent and the model and compresses what passes through. Apache-2.0, ~3.5k stars, created March 2026, with near-daily releases. Its pitch is token economics. The features are read modes, AST-aware compression, and a shell hook that compresses `git` and `docker` output. It also has a proxy that compresses every request, and a property graph over *code* (imports, calls, exports).

It is not a documentation system, and its own `VISION.md` confirms this rather than merely omits it: no documentation corpora, no taxonomy, no schema validation. Where it says **governance** it means governance *of the agent*, not governance of a corpus. That means policy over what an agent may see, signed evidence of what it saw, and compliance reports. That distinction is important to hold, because the word will soon be contested and the two meanings have almost nothing in common.

### I.1 OKF — the same substrate, arrived at independently

The point of contact is the **Open Knowledge Format**, which LeanCTX defines itself and exports to:

- a directory of Markdown files, one concept per file
- YAML front matter, with `type` as the only required field
- relations as Markdown links — `- depends_on: [category/key](path.md)`
- a relation vocabulary of `depends_on`, `related_to`, `supports`, `contradicts`, `supersedes`
- written into the user's repository, byte-deterministic so exports diff cleanly

Typed nodes, typed edges, Markdown in the repository. That is Headwater's substrate, reached from an entirely different initial problem. This makes it a third independent arrival at the same choice, after OpenGEO (§E) and the LLM Wiki (§F.2). After three, we can no longer treat it as a preference.

### I.2 The arrow points the other way, and that is the whole difference

OKF is an **export**. The durable store is a `knowledge.json` under the user's config directory — the only format that round-trips losslessly. Markdown is a projection *out* of it, for portability and hand-editing. Headwater is the exact inverse: the Markdown is the corpus, and the graph, indexes and rules are projections out of *that* ([Q6](09-open-questions.md#q6--where-the-corpus-graph-lives-at-rest) exists exactly to keep it that way).

The inversion explains their validation, and a concrete statement of it is worthwhile: it is the sharpest available illustration of what a taxonomy is *for*. `lint_okf_bundle` returns warnings only — its own doc comment says that the checks are advisory and "a partially-malformed bundle should still import what it can". The complete set is: not a directory, unreadable, missing front matter, missing `type`, empty body. Four checks. The importer then does `get_str(fm, "type").unwrap_or("fact")`.

**`type` is a free string with a default.** There is no closed vocabulary, no per-type required facets, no cardinality, no reciprocity, nothing whole-graph.

This is not a criticism. For an export format, lenience is correct engineering: the obligation is to survive a round trip. A format that rejects its own bundles serves nobody. But it decides the overlap question. LeanCTX has our file format and none of our TBox. [§C](#c-linkml--the-uncomfortable-one) established that everything that Headwater does beyond one-instance-against-a-shape is the interesting part. So a shared serialization costs us nothing and threatens nothing.

### I.3 What transfers

**OKF as an export target.** We already emit Markdown with typed front matter. An OKF bundle is close to free, and it buys interoperation with a tool that a large number of people already installed. Their `leanctx_*` convention — producer-owned prefixed keys that a consumer carries but never validates — is the right pattern for the reverse direction too. Their round-trip test asserts exactly that: unknown keys survive a parse-emit cycle.

> **Folded into [Q13](09-open-questions.md#the-okf-question-is-a-different-layer), as a separate and much smaller question than the substrate one.**

**`contradicts` as a declared edge — and an inconsistency that it exposed in our own specification.** OKF carries `contradicts` as a declared relation, which prompted the question of where a declared contradiction sits on the cohesion/coherence line. The answer was embarrassing rather than novel. [spec 2](02-taxonomy-model.md#the-decision-relation-vocabulary) contains `conflicts_with` with `invalid_when: {both: {status: current}}`, and it was there all along — a deterministic, blocking-eligible check. But [spec 4](04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep) listed "pages that contradict each other while both remain current" as work for the sampled LLM sweep. Two documents assigned the same job to two different mechanisms, and one of them was needlessly the expensive one.

The fix generalizes past the bug. What decides whether an obligation is cohesion or coherence is not its subject but **whether the judgment that it requires is recorded as data**. So a coherence concern that recurs is a prompt to ask what an author can declare, rather than how to detect it better. That reframes the sweep's purpose: its best output is not a finding but a declared edge, after which the engine owns the constraint permanently.

> **Applied:** the declaration boundary, the corrected sweep scope, and a fourth sweep constraint, in [spec 4](04-assurance-model.md#declaration-moves-the-boundary).

### I.4 Two things to be careful about

**The Context Governance Benchmark.** LeanCTX publishes a self-assessment against a 32-control, 6-family, 3-tier benchmark and claims "C2 — Managed". The structure is recognisably that of [spec 4](04-assurance-model.md) — named controls, families, maturity tiers, a published assessment. If our control catalog ever wants an existing numbering to point at, it is a candidate.

But the spec lives on a private GitLab instance that belongs to the same author, and its independence is **unverified**. It should be read as self-published until shown otherwise. The controls themselves govern agent behavior rather than corpus quality, so little of the content transfers even if the framing does.

**The claims move.** The repository description, the README and cached earlier versions give the MCP tool count as 76, 82 and 62 respectively. The compression percentages and the "4-layer verification engine" are unmeasured by anyone outside the project. Roughly nine-tenths of the commits are from one author in under five months. The OKF specification itself is small, legible and backed by round-trip tests, and can be depended on directly. That judgment does not extend to the numbers around it.

### I.5 The presentation is the lesson

The most transferable thing here is not technical. LeanCTX ships a product site that covers, coherently and in eighteen languages, what most open specifications never assemble. The site has how-it-works, architecture, benchmarks, compatibility, competitor comparisons, six use-case pages, pricing, an enterprise tier, docs, changelog, community, and a compliance self-assessment. Its `robots.txt` explicitly welcomes AI crawlers under a "GEO" heading, and it serves an `llms.txt` that describes it to machine readers.

That last detail is not decoration — it is [Q14](09-open-questions.md#q14--discovery-surface) already shipped by someone else. It is evidence that the discovery surface has a human half that we did not plan for at all. A corpus that nobody can evaluate from the outside is not adopted, however well it validates.

> **Recorded as [Q16](09-open-questions.md#q16--public-presence).**

## J. TrustGraph — the same pitch, the opposite mechanism

[TrustGraph](https://github.com/trustgraph-ai/trustgraph) (Apache-2.0, v1.2 shipped August 2025) is a containerized context-engineering platform. Documents flow through configurable pipelines in which **LLM agents extract entities and relationships** into a graph store (Cassandra, Neo4j, Memgraph or FalkorDB). Embeddings live in Qdrant, messages flow over Pulsar, and retrieval returns to agents as document-, graph-, or ontology-driven RAG. Its semantic layer is the standards stack — RDF, OWL, SKOS, SHACL. Every answer carries a per-fact provenance receipt: source document, ingestion timestamp, extraction method.

The pitch overlaps ours almost word for word — typed graphs that ground agents in verifiable knowledge, with provenance. The implementations are close to opposites. In Headwater, authors declare the graph: documents are the nodes, front-matter references are the typed edges, and no LLM issues a verdict. In TrustGraph, an LLM extracts the graph: documents are feedstock, dissolved into triples, and the 1.2 release headline is an agent that "autonomously populates the knowledge graph". Nothing in the platform governs the source documents — it mines them. Its retrieval modes manage exactly the reconstitution loss that §G describes. That makes TrustGraph the subject of that section, in production form. The overlap is at the slogan, not the layer beneath it.

Three things deserve a record:

- **Evidence the other way on Q13.** §E cites OpenGEO, which declines RDF/OWL/SHACL, as evidence that the standards stack is not required for this class of problem. TrustGraph is the counterweight: a production open-source system that chose that stack and ships it. One data point on each side is a more honest input to the decision than one.
- **A preview of Q15's provenance burden.** TrustGraph's extracted graph is precisely the synthesized tier that Q15 asks about — LLM-maintained, never verifiable by regeneration. Its per-fact receipts (source, timestamp, derivation method) are a concrete, operational design for the provenance record that such a tier needs.
- **Attribution, implemented.** The receipt model — every answer traceable to the facts and the traversal that licensed it — is the attribution row of §H in production form. It is independent confirmation that a shipped system also put its effort into oracles and traceability.

And one relationship deserves a record as complementary rather than rival. Source quality bounds extraction quality, so a governed corpus is an unusually good input to an extraction platform. Kinds, facets, and declared edges arrive as structure that the extractor otherwise must guess at. Under the [better-together principle](00-vision-and-scope.md#design-principles) of spec 0, that makes TrustGraph ingestion a candidate integration — a Headwater corpus fed in as pre-structured source material. It is in the same family as the OKF export, and equally cheap to hold: the arrow points out of the corpus, and nothing flows back in.

> **Recorded:** counterweight evidence added to [Q13](09-open-questions.md#q13--linkml-and-shacl-as-substrate); provenance precedent added to [Q15](09-open-questions.md#q15--a-synthesized-content-tier).

## K. Modern Requirements — the first candidate where the arrow reverses

[Modern Requirements4DevOps](https://www.modernrequirements.com/products/modern-requirements4devops/) is a commercial requirements-management extension embedded in Azure DevOps. Everything that it creates is an ADO work item. Requirements, reviews, baselines, and trace links live in the ADO store. Its own documentation calls that store the single source of truth for project artifacts. Its interchange surfaces are the ADO REST API and ReqIF, the OMG Requirements Interchange Format, which a companion extension supplies. This entry exists for a practical reason, not for survey completeness. An organization that we work with decided to adopt the tool. Thus "can a Headwater corpus sit downstream of an RM tool?" is a question with a date on it.

The overlap is larger than a commercial ALM extension suggests, and it sits on our side of the boundary. Requirements management is traceability management, and [spec 10 §B.5](10-theoretical-foundations.md#b5-traceability-information-models--our-idea-has-a-name-and-a-literature) grounds Headwater's design in exactly that literature. The pieces that an RM integration needs are already specified:

- `traces_to` is an evidence relation, enabled by default ([spec 2](02-taxonomy-model.md#the-decision-relation-vocabulary)).
- Work items are a named external anchor kind. One resolver owns each anchor kind, and resolution is a [correctness root](02-taxonomy-model.md#behavior-at-the-limits).
- Requirements and acceptance criteria are two of the five artifacts that receive stable identifiers ([spec 3](03-authoring-and-lifecycle.md#identifiers)).
- Traceability matrices are a projection kind ([spec 6](06-engine-architecture.md#projections)).
- `evidence_basis` asks for exactly the pointer that an RM tool mints.

Even the exclusion is already written. [Spec 0](00-vision-and-scope.md#what-we-do-not-build) declines to build a ticketing system: "we refer to work items, we do not manage them". Everything in Modern Requirements is a work item, so that row covers requirements management verbatim. Nothing in the model changes to point at this tool.

What has no precedent is the direction. Every integration recorded so far points out of the corpus — TrustGraph ingestion, the OKF bundle, and the LinkML, SHACL, and SKOS emissions. §J prices TrustGraph as cheap on exactly that ground: nothing flows back in. Here the requirements come from a store that the corpus does not govern, and the corpus consumes them. This is the first inbound candidate, and it is a different family with a different cost.

It meets three stated positions of spec 0 head-on. Each one resolves into a design constraint, not a refusal:

- **"We refer to work items. We do not manage them."** This position holds untouched. The integration reads. On the day that it writes a work item, it becomes a requirements tool and leaves the scope table.
- **No network dependency at check time.** So the fetch is not part of validation. A scheduled job pulls a snapshot — a ReqIF bundle or an API export — and commits it. This is the [upstream-awareness pattern](07-distribution-and-federation.md#upstream-awareness) of spec 7, pointed at a second kind of upstream. An `ado_work_item` resolver then binds anchors against the committed snapshot, and that step is offline and deterministic. Check-time behavior stays a function of repository content, as everywhere else.
- **One source of truth per fact.** Requirement content is canonical in the RM tool and never here. What the corpus canonically owns is its own half of the join: which documents trace to which requirements. The corpus declares those edges, and the RM tool knows nothing of them.

What flows in is therefore deliberately small: anchor identities, `traces_to` edges, and drift findings. Upstream, someone can reword, close, or delete a requirement that documents trace to. When the snapshot shows such a change, the comparison raises a change proposal with the diff attached. That is exactly what spec 7 already does for taxonomy upstreams. It never mutates the corpus silently.

Two declarations that currently have no operational story acquire one here. `created_by: import` is a legal edge provenance [in spec 2](02-taxonomy-model.md#who-creates-each-edge), but nothing says what an import *is*. An RM importer is what the value is for, and the staleness-by-creator report of `taxonomy audit` is what keeps a decayed import visible. And [Q15](09-open-questions.md#q15--a-synthesized-content-tier) gains a sharper test case than the one that raised it. Imported requirement text is *not* synthesized, because regeneration against the pinned snapshot verifies it. That makes it a projection whose source the corpus does not govern. The authored/generated split does not name that case, and the tier question has to answer it.

> **Recorded as [Q19](09-open-questions.md#q19--inbound-integration-an-external-system-of-record).**

## L — Serena: the memory layer as a corpus, and where scent lives

[Serena](https://github.com/oraios/serena) is an MCP toolkit for coding agents, MIT-licensed, created in March 2025, with 27.8k stars. It gives an agent symbol-level navigation and editing through language servers, which is not our subject. The adjacent part is its **memory system**: Markdown files that an agent writes into `.serena/memories/` and commits with the code.

Scale changes how we should read it. LeanCTX (§I) has about 3.5k stars, and Serena has near eight times that number. This is not one team's local convention. It is a shape that a large number of agent-assisted repositories now carry.

### L.1 The fourth arrival, and the first one with its reasons written down

§I.1 counted three independent arrivals at Markdown-in-the-repository as the substrate for machine-readable knowledge: OpenGEO, the LLM Wiki, and OKF. Serena is the fourth. It is the first that states its criteria as a numbered list, and then names what those criteria exclude.

The seven criteria, compressed:

1. files stay human-readable and editable in any text editor
2. project memories live beside the code, and diff in a pull request
3. an agent receives the memory *name list*, and chooses what to read from it
4. references beat search
5. the agent decides each read, and the harness injects nothing on its behalf
6. the format is plain Markdown, with one Serena-specific convention
7. two scopes, project and global, compose freely

The exclusions are the more interesting half. Database-backed memory, which covers SQLite, graph stores and vector stores, fails criteria 1, 4 and 6. Single-file conventions of the `AGENTS.md` family fail criteria 3 and 5. Hooks and harness-internal memory fail criteria 5 and 6. The documentation then states that no existing system met the goal, and that Serena ships its own layer for that reason. It names Obsidian, Logseq and Foam as the nearest relatives. That is a documented search for an off-the-shelf answer, and a documented failure to find one.

The overlap stops where it stopped for OKF. A memory carries no front matter, no declared kind, and no schema. One relation exists, `mem:`, and it is untyped. OKF at least requires a `type` field and names five relations (§I.2). Serena carries less than that. So the verdict of §I.2 applies again and more strongly. This is our file format and none of our TBox, so a shared shape costs nothing and threatens nothing.

### L.2 "Prefer references to search" — a third argument against retrieval

[Spec 5](05-ai-integration.md#what-we-do-not-do) declines retrieval as the primary mechanism, and that position now rests on three independent arguments. Mine was precision. Karpathy's, which §F.2 records as the better one, is that retrieval accumulates nothing. Serena's is different again.

The criterion states that any retrieval method, lexical or semantic, produces both false positives and false negatives. An explicit reference, named and chosen by the agent, produces neither. Retrieval is therefore a source of variance, and a declared edge removes it.

That is §H in operational form. §H sets the target as bounded, auditable non-determinism, and it names ambiguity removal as the mechanism that narrows variance. A named edge in place of a similarity match is one concrete instance of that mechanism. Serena also keeps plain search as a complement and never as the authority, which is the posture spec 5 already takes toward embeddings.

> **Applied:** the third argument added to [spec 5](05-ai-integration.md#what-we-do-not-do).

### L.3 Where scent lives — the first substantive disagreement

This is the one finding that contradicts something we specified.

Serena ships a convention document to every onboarded project. One of its rules reads: "Memories themselves should not contain information about when to read them; this is the responsibility of the referring memory." <!-- ste-lint: allow semicolon # direct quotation from the shipped memory_maintenance template --> The convention also asks each reference to do more than name its target, and to say which aspects of the subject the target covers.

[Spec 5](05-ai-integration.md#scent-is-the-thing-being-engineered) puts that cue in the `summary` facet of the target, and calls the summary facet "the corpus's entire scent surface". Serena puts it on the edge. Both designs engineer scent deliberately, and they place it in opposite locations.

Foraging theory settles more of this than I expected, and it settles it against the stronger half of our claim. Scent is the **proximal** cue at the point of decision. When a reader follows a relation, the proximal cue is the referring text, and the target's own summary is distal. Spec 5 cites the theory correctly and then applies it to one moment only.

The two placements are not interchangeable, and each one fails where the other works.

- **Query-time routing needs the cue on the node.** `headwater route` builds a pointer list from a task description, so no referring edge exists to carry a cue. Serena has no routing, and never meets this case.
- **Traversal-time reading needs the cue on the edge.** A summary cannot say why *this* link, from *here*. One standard, referenced once by a decision and once by a specification, deserves a different cue in each place.
- **Orphans separate the two.** A document with no inbound edge still has node scent, and routing still finds it. Under edge-only scent it is invisible. Serena covers this with the up-front name list and a `mem:core` root, which works at the scale of a single project.
- **Measurement separates them again.** Spec 5 grades distinctiveness by a comparison of sibling summaries in one place. Edge cues have no such place, so they are harder to grade. Serena grades neither.

Headwater serves both moments, so it plausibly needs both placements, and it currently specifies one. The cost of the second is real. A cue on a relation instance means prose on an edge. It adds an authoring burden at every reference, and a new object for the checks to grade.

> **Recorded as [Q20](09-open-questions.md#q20--where-scent-lives).**

### L.4 The convention document is itself a memory, and nothing keeps it current

Serena seeds a `memory_maintenance` memory at first onboarding, under a stated precedence. A global copy wins, and suppresses the project copy. Otherwise an existing project copy stays untouched. Otherwise the shipped template lands in the project. Existing files are never overwritten.

That is [spec 7](07-distribution-and-federation.md)'s overlay shape, reached independently: a shipped default, an organization-wide override, and a local copy. Two details are worth keeping.

First, the precedence runs opposite to ours. [Spec 7](07-distribution-and-federation.md#federation) has overlays compose in one direction, where each tier may override the tier above it. A global convention here does the reverse. It **suppresses** the local copy, and no local copy is written at all. The stated reason is that a team wants one convention document across all of its projects. So a convention overlay and a content overlay want opposite precedence, which spec 7 does not currently allow for.

Second, the mechanism seeds once and then diverges. No pin exists, no version, and no drift report. The documented route to an improved template is deletion of the local copy. Spec 7's [upstream awareness](07-distribution-and-federation.md#upstream-awareness) and change proposals are the machinery this lacks, which is mild confirmation that they earn their cost.

### L.5 Onboarding ships the synthesized tier, and marks nothing

Serena runs an **onboarding** pass the first time it meets a project. It reads the code and writes what it learned into memories. That is [Q15](09-open-questions.md#q15--a-synthesized-content-tier)'s synthesized tier in production, and it is a sharper case than TrustGraph's.

TrustGraph holds its extracted graph in a separate store, and mints a per-fact receipt with source, timestamp and derivation method (§J). Serena writes synthesized content into the same directory, with the same file shape, as memories that a human wrote by hand. No front matter separates them, because no front matter exists. Nothing records which agent wrote a memory, from which sources, or when its claims were last true. The documented control is a recommendation to review the output after onboarding.

The result is a corpus in which a reader cannot tell authored content from synthesized content by inspection. Q15 already leans toward a tier that is permanently non-canonical and clearly marked. Serena shows what the alternative looks like at this scale of adoption, and that is the strongest available argument for the leaning.

> **Recorded:** evidence added to [Q15](09-open-questions.md#q15--a-synthesized-content-tier).

### L.6 A filter in the tool layer is advisory, and the documentation says so

Two regex controls sit in Serena's configuration. `read_only_memory_patterns` marks memories that the agent must not change, and the agent is told which ones. `ignored_memory_patterns` removes memories from every memory tool.

The second is the instructive one. The documentation states that an ignored memory is reachable through no memory tool. It then tells the reader to open that memory with the general `read_file` tool, on the raw path. The filter sits in the tool surface. The bytes sit in the repository. Serena's security page states the same limit about a different feature. The trust gate is "a functionality boundary, not a containment boundary".

[Q17](09-open-questions.md#access-control-is-a-property-of-the-serving-boundary) already places enforcement at the serving boundary rather than at the authoring layer. Here that position is confirmed by counterexample. A control that filters one reading path, while the bytes stay readable along another, is a convenience. `read_only_memory_patterns` is the honest half of the pair, because it constrains a cooperative agent and claims nothing beyond that.

> **Recorded:** evidence added to [Q17](09-open-questions.md#q17--governed-access-and-the-solution-layer).

### L.7 Two smaller things, and a gap in the evaluation

**A rule for what not to record.** The convention document sets an add threshold. It keeps stable, non-obvious project conventions that prevent expensive rediscovery. It then refuses quick-read facts, generic framework knowledge, one-off task notes, volatile line-level detail, and behavior that is likely to change soon. [Spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) argues that cheap capture is what saves a corpus. This is the other half of that argument. Once capture is cheap, over-capture becomes the failure mode, and something has to say what to leave out. Headwater's kinds and shelves say where a document belongs. Nothing yet says that a fact is too volatile to record at all.

**A voice regime for a machine reader.** The same document sets a style: "Dense agent notes, not prose docs". It asks for invariants and terse bullets, and it refuses rationale or examples unless they prevent a likely mistake. That is a voice regime whose primary reader is an agent. It is close to the opposite of the regimes spec 3 describes for human-facing kinds. The regime abstraction holds under the change of reader, which is the useful result. Headwater has not yet named a regime of this class.

**The closest part is the unmeasured part.** Serena publishes a real evaluation, with a methodology, a fixed prompt, five agent and model combinations, and a classification that admits neutral and negative results. It evaluates the code tools. It does not evaluate the memory system, which is the part that resembles Headwater. Reported user feedback is what carries that layer. The method is also agent-as-evaluator, where [spec 5](05-ai-integration.md#measuring-whether-any-of-this-works) grades from the tool-call transcript instead. Above all, no counterfactual run compares agent behavior with the memories present against the same behavior without them. Spec 5 names the counterfactual as the category that matters most, and the one most often skipped. The most widely adopted example of this pattern skips it.

## M — What the survey shows as a whole: convergence is not evidence

The entries above arrived one at a time, and each settled a local question. Read together they say something that no single entry says, and it raises the standard that this project owes its own claims.

### M.1 The convergence argument, and the question it cannot answer

This document leans on independent arrival repeatedly. §B.4 calls it "the strongest evidence available that the line is real". §F.2 rests the capture-cost thesis on two derivations, one from the literature and one from practice. §I.1 counts a third arrival at the substrate, and concludes that we can no longer treat it as a preference. §L.1 counts a fourth.

That argument is sound for one question, and it is worth stating which. Four projects that chose Markdown in a repository establish that the choice is ordinary, cheap and unsurprising. They lower the risk of a substrate decision that would be expensive to reverse.

They establish nothing about whether any of it works.

Convergence counts as evidence only when the arrivals are independent. These arrivals share a decade of tooling and a common set of posts and talks. They also share one blunt constraint: agents read text files, and git stores text files. A shared prior is not four measurements. Correlated error looks exactly like agreement, and it looks more convincing as it accumulates.

So the survey licenses a **substrate** conclusion, where a wrong answer costs a migration. It does not license an **efficacy** conclusion, where a wrong answer costs a project built on a belief. §F.2 already hedged toward this, with "the best support that it will get, short of measurement". This section states the limit for the whole document instead of one entry.

### M.2 What the field has actually measured

The claim under review is the one that every source here shares in some form. Structured knowledge improves machine output. The evidence offered for it:

| Source | The efficacy claim | What stands behind it |
|---|---|---|
| testerstories (§B) | An ontology yields better generated implementations | Tests that the generated code matches the ontology. No run without the ontology |
| LLM Wiki (§F.2) | A maintained wiki serves agents into the hundreds of pages | Reported experience |
| LeanCTX (§I) | Compression ratios, a verification engine, a maturity tier | Self-assessment, against a benchmark published by the same author (§I.4) |
| TrustGraph (§J) | Agents grounded in verifiable knowledge | Provenance receipts are implemented. This survey found no published effect on answer quality |
| OpenGEO (§E) | Declared meaning steers third-party engines | This survey found none published |
| Serena (§L) | Memory improves long-lived agent workflows | User feedback. The published evaluation covers other tools, grades with agents, and runs no counterfactual |

Two of those rows say "this survey found none", and that phrasing is deliberate. Absence of a result in a search is weaker than a null result. A section about rigor should not help itself to the stronger reading.

The pattern still holds across the rest. Not one source compares behavior with its structure present against the same behavior without it.

### M.3 The grader decides the answer, and the literature proves it

A systematic comparison of RAG and graph-based RAG supplies the finding that turns this from suspicion into evidence. The authors report that their reference-based evaluation **contradicted** the original graph-based RAG study. The stated cause is not the systems. It is the grading method. The earlier work used an LLM as judge with no ground-truth reference, and the later work scored against human-written summaries.

The same authors then tested the instrument itself. An LLM judge shows position bias. A reversal of the order in which two summaries are presented produces "substantially different, and in some cases opposite, judgments".

Read that against the table above. Serena grades with agents. LeanCTX grades itself against a benchmark it publishes. Both use an instrument that the reversal above discredits, and both apply it to a result they have an interest in.

The conclusion is not that these projects are dishonest. It is worse than that, and more ordinary. A weak instrument returns the answer that the author expected, and no one involved has to notice.

### M.4 What this obliges Headwater to do

[Spec 5](05-ai-integration.md#measuring-whether-any-of-this-works) already grades from the tool-call transcript rather than from any model's self-report. It already names the counterfactual as the category that matters most. §M.3 does not change that design. It changes its status. Those two choices are no longer hygiene. They are the part of the design with no prior art to copy, because every source above either skipped them or failed them.

Three requirements follow. Each is cheap to state now and expensive to retrofit.

- **The grader is never the system under test.** A probe verdict comes from the transcript and from a declared expectation. No model judges whether the corpus helped. The field has already published what that instrument returns.
- **A published efficacy claim carries its counterfactual.** Corpus present against corpus absent, with a pinned model and a recorded probe selection. Without that pair, Headwater holds an opinion of exactly the class tabulated in §M.2.
- **Unmeasured claims are declared as unmeasured.** [Principle 5](00-vision-and-scope.md#design-principles) requires the system to record what it does not check. The same rule binds what we have not measured. The evidence register of [spec 4](04-assurance-model.md) is where that mark belongs, so that a reader can separate a tested claim from an intended one.

One statement keeps this section honest. Headwater has no measurements either. It has a design for them, an unbuilt engine, and now a written obligation not to claim more than the design has earned.

> **Applied:** the grader constraint and the counterfactual obligation in [spec 5](05-ai-integration.md#measuring-whether-any-of-this-works), and the efficacy limit of prior-art agreement in [principle 10](00-vision-and-scope.md#design-principles).

---

## Summary

| Source | Verdict | Outcome |
|---|---|---|
| TBox/ABox framing | Adopt the vocabulary | Applied — specs 1, 2, 6 |
| testerstories, spec → ontology → implementation | Strongest supporting evidence found | Applied — oracle and citation (specs 2, 4, 5). Authority adopted, then cut by review |
| LinkML | May already be half of spec 2 | **Open — Q13**, needs a decision |
| SHACL | May already be the check layer | **Open — Q13** |
| OpenGEO | Different direction, same substrate | Discovery gap recorded — **Q14** |
| KG chunking for RAG | Framing useful, chunking not applicable | Non-adoption reasoned and recorded |
| r/OntologyEngineering | Ontology-first methodology, further than we go | Noted. Oracle work is the shared ground |
| Karpathy, *LLM Wiki* | Best anti-RAG argument, independent capture-cost confirmation | Applied — coherence sweep (spec 4). **Q15** raised |
| LeanCTX / OKF | Same substrate, opposite arrow. No taxonomy to collide with | Applied — declaration boundary (spec 4). OKF export → **Q13**. Presentation gap → **Q16** |
| TrustGraph | Same pitch, opposite mechanism — extracts the graph we declare | Standards-stack counterweight → **Q13**. Provenance precedent → **Q15**. Ingestion integration candidate |
| Modern Requirements / Azure DevOps | First inbound candidate — the arrow reverses | Model already fits. Import semantics open — **Q19** |
| Serena | Fourth arrival at the substrate. The first to disagree with us | Applied — third anti-retrieval argument (spec 5). Scent placement → **Q20**. Evidence → **Q15**, **Q17** |
| **The survey as a whole** | Convergence on the substrate, and near-zero measurement of the claim | Applied — grader and counterfactual constraints (spec 5), efficacy limit of prior-art agreement (principle 10) |
