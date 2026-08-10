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

> **Decided: a solution-layer node is a declared anchor ([spec 7](07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it)).** The [evaluation](../evaluations/the-serving-boundary.md) adds a second argument that this section did not have, and it is the one that settles the case. A filter has nothing to attach to on a node that carries properties. A document has facets that a predicate reads, and an anchor is carried whole or withheld whole.

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

> **Applied, then cut.** An `authority` ordering on kinds was adopted here and removed by the core-concepts review. The trigger (detection of factual disagreement) needs a judgment that the ABox boundary refuses to make. A global scalar cannot carry the scoped precedence that their prompt actually encodes. What Headwater keeps is the part of their design that works. It keeps the *decision rule* ("note the discrepancy") as an agent instruction, and the adjudication recorded as data ([spec 2](02-taxonomy-model.md#disagreement-is-adjudicated-not-ranked)). [Q18](09-open-questions.md#q18--recording-adjudicated-disagreements) has since closed and said where that data lives. An adjudication is a decision document, and `overrides` is the edge that carries it ([§P.8](#p8-legal-citators-argue-against-q18s-ruling-and-then-for-it)).

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

When you read [spec 2](02-taxonomy-model.md) and then the LinkML documentation, the overlap is substantial. The shared features are classes with slots, ranges, cardinality, enums with permissible values, inheritance, and schema-level imports. The Q2 leaning of spec 2 read "YAML plus a published JSON Schema, with the resolved lock in a stricter representation". That describes something that LinkML already implements. LinkML also ships the multi-format compilation that we would otherwise write.

**[Q2](09-open-questions.md#q2--schema-format) has since closed, and it withdrew that leaning as wrong.** Spec 2 already carries references that no JSON Schema keyword resolves, so the authored surface was always YAML plus a Headwater resolver. The overlap with LinkML survives the correction, because it is a fact about the two languages and never followed from our leaning. What the correction removes is option 1's cheapest argument, which was that our own leaning already described LinkML.

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
2. **Stay independent, borrow the design.** LinkML's target is data models for research and biomedical data. A documentation taxonomy is a different animal, and a bolted-together schema — half LinkML, half ours — may be worse to author than either alone. The cognitive-dimensions [walkthrough](../evaluations/schema-format-walkthrough.md) that Q2 prescribed has since run, and "two languages in one file" scores badly on role-expressiveness.

3. **Emit it, do not author in it.** Author in Headwater's language and compile the shape layer *to* LinkML, which then generates JSON Schema, SHACL, OWL and Pydantic through LinkML's own toolchain. One authoring surface, fully validated, with a standards-based export. This option only became visible when we wrote the example out, and it is now the leading candidate.

The decisive evidence against option 1 is mundane: everything Headwater-specific lands in LinkML `annotations`, which are untyped pass-through. LinkML carries them and validates none of them. So for exactly the half that is ours, the meta-schema benefit disappears. Authors face two languages in one file, with no visual cue for which half is checked.

I will not decide this unilaterally. It changes what we build, and it is close to irreversible under option 1. [Q1](09-open-questions.md#q1--implementation-language) has since closed on Rust, so LinkML's Python tooling now pulls against a settled core rather than a candidate one. Option 3 dissolves that tension, which is part of its appeal.

> **Recorded as [Q13](09-open-questions.md#q13--linkml-and-shacl-as-substrate), which has since closed on option 3 with one correction.** Emitters never chain, so LinkML is not the route to SHACL or to JSON Schema. It is the last of six sibling emitters, and §N.5 records the measurement that decided it.

## D. SHACL — the name for schema-derived checks

[Spec 6](06-engine-architecture.md) says that schema-derived checks are *generated from the taxonomy*. SHACL (Shapes Constraint Language, W3C) is the standard that does this for graphs: you declare shapes, validate a graph against them, and get structured violation reports. The corpus graph is expressible as RDF, because it is typed nodes with typed edges. If it is expressed as RDF, then SHACL shapes can *be* the check layer rather than something that we hand-write.

A [worked example](../evaluations/shacl-worked-example.md) confirms that SHACL reaches the whole-graph layer that LinkML cannot. Reciprocity, conflict between two live decisions, satellite inheritance, and windowed sequence expectations are all expressible. Every one of them needs a drop to SPARQL. SHACL Core can traverse, but it cannot refer back to the focus node from the far end of a traversal.

My reservation here — that SHACL's reports are hard to read — was too strong and is withdrawn. `sh:message` with variable interpolation makes messages as good as they are authored. The objections that survive are sharper. **Line numbers, remediation and fixability do not survive the RDF round trip**, and [spec 4](04-assurance-model.md) requires all three to make a finding actionable. Temporal checks also need the evaluation time injected rather than read from the clock, or determinism breaks. This constraint applies to whatever engine we build, not just to this one.

The conclusion is the same as for LinkML, by a different route: a compilation target, not an authoring surface. Folded into Q13, which closed and put SHACL third in the emitter order, behind JSON Schema and the native graph export.

## E. OpenGEO — same substrate, opposite direction

[OpenGEO](https://github.com/donhaji/opengeo) is a specification for publishers to declare canonical meaning to AI systems. It uses Markdown bodies with YAML front matter, organized as Discovery → Semantic → Context → Execution. Assurance is treated as a concern *around* the chain rather than a layer in it.

Two points of contact:

- **The substrate is identical.** Markdown plus YAML front matter as a semantic contract for machine readers, deliberately *not* RDF/OWL/SHACL. That is an independent data point that the heavyweight stack is not required for this class of problem. It is useful evidence for the Q13 decision, and a caution against the assumption that the standards-based route is obviously correct.
- **Assurance is not a layer.** Their framing puts provenance, authorship, freshness, auditability and ownership as oversight around the whole chain rather than a stage within it. That framing matches [spec 4](04-assurance-model.md) and is a cleaner statement of it than ours.

But the direction is opposite. OpenGEO points **outward**: a publisher declares meaning to third-party engines that it does not control, with execution explicitly out of scope. Headwater points **inward**: an organization governs its own corpus, for its own agents. We control the whole pipeline, and can therefore *check* things rather than merely declare them. Their context layer (tone, persona, interpretation envelope) follows from a lack of control over the consumer. We do control the consumer, so we constrain behavior directly, and do not merely request it.

The transferable gap is **discovery**. OpenGEO takes seriously how a machine reader arrives cold and finds out what a corpus is. [Spec 7](07-distribution-and-federation.md) covers distribution to repositories that already know about us, and says nothing about an agent that encounters the corpus for the first time.

> **Recorded as [Q14](09-open-questions.md#q14--discovery-surface), which has since closed.** The gap named here is real and it is narrower than this section states. A corpus can tell an arriving machine what governs it, and that is the [corpus descriptor](07-distribution-and-federation.md#arriving-at-a-corpus-cold). A corpus cannot make itself found by a machine that holds no pointer to it, and no file inside a corpus ever will.

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

> **Recorded as [Q15](09-open-questions.md#q15--a-synthesized-content-tier), which has since closed, and it renamed the tier.** The axis is not the author but the **warrant**: what stands behind the content ([spec 1](01-conceptual-model.md#warrant)). This pattern ships content with no warrant at all, because lint is a cohesion mechanism and establishes nothing about truth ([§P.5](#p5-the-two-observed-cases-in-this-survey-read-against-the-standards)).

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

OKF is an **export**. The durable store is a `knowledge.json` under the user's config directory — the only format that round-trips losslessly. Markdown is a projection *out* of it, for portability and hand-editing. Headwater is the exact inverse: the Markdown is the corpus, and the graph, indexes and rules are projections out of *that*. [Q6](09-open-questions.md#q6--where-the-corpus-graph-lives-at-rest) closed on exactly that, and it added the obligation that every projection declares what it dropped.

The inversion explains their validation, and a concrete statement of it is worthwhile: it is the sharpest available illustration of what a taxonomy is *for*. `lint_okf_bundle` returns warnings only — its own doc comment says that the checks are advisory and "a partially-malformed bundle should still import what it can". The complete set is: not a directory, unreadable, missing front matter, missing `type`, empty body. Four checks. The importer then does `get_str(fm, "type").unwrap_or("fact")`.

**`type` is a free string with a default.** There is no closed vocabulary, no per-type required facets, no cardinality, no reciprocity, nothing whole-graph.

This is not a criticism. For an export format, lenience is correct engineering: the obligation is to survive a round trip. A format that rejects its own bundles serves nobody. But it decides the overlap question. LeanCTX has our file format and none of our TBox. [§C](#c-linkml--the-uncomfortable-one) established that everything that Headwater does beyond one-instance-against-a-shape is the interesting part. So a shared serialization costs us nothing and threatens nothing.

### I.3 What transfers

**OKF as an export target.** We already emit Markdown with typed front matter. An OKF bundle is close to free, and it buys interoperation with a tool that a large number of people already installed. Their `leanctx_*` convention — producer-owned prefixed keys that a consumer carries but never validates — is the right pattern for the reverse direction too. Their round-trip test asserts exactly that: unknown keys survive a parse-emit cycle.

> **Folded into [Q13](09-open-questions.md#q13--linkml-and-shacl-as-substrate), as a separate and much smaller question than the substrate one.** Q13 has since closed and placed OKF fifth in the emitter order. The terms are the ones that every emitter takes: it declares a loss set and emits a census.

**`contradicts` as a declared edge — and an inconsistency that it exposed in our own specification.** OKF carries `contradicts` as a declared relation, which prompted the question of where a declared contradiction sits on the cohesion/coherence line. The answer was embarrassing rather than novel. [spec 2](02-taxonomy-model.md#the-decision-relation-vocabulary) contains `conflicts_with` with `invalid_when: {both: {status: current}}`, and it was there all along — a deterministic, blocking-eligible check. But [spec 4](04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep) listed "pages that contradict each other while both remain current" as work for the sampled LLM sweep. Two documents assigned the same job to two different mechanisms, and one of them was needlessly the expensive one.

The fix generalizes past the bug. What decides whether an obligation is cohesion or coherence is not its subject but **whether the judgment that it requires is recorded as data**. So a coherence concern that recurs is a prompt to ask what an author can declare, rather than how to detect it better. That reframes the sweep's purpose: its best output is not a finding but a declared edge, after which the engine owns the constraint permanently.

> **Applied:** the declaration boundary, the corrected sweep scope, and a fourth sweep constraint, in [spec 4](04-assurance-model.md#declaration-moves-the-boundary).

### I.4 Two things to be careful about

**The Context Governance Benchmark.** LeanCTX publishes a self-assessment against a 32-control, 6-family, 3-tier benchmark and claims "C2 — Managed". The structure is recognisably that of [spec 4](04-assurance-model.md) — named controls, families, maturity tiers, a published assessment. If our control catalog ever wants an existing numbering to point at, it is a candidate.

But the spec lives on a private GitLab instance that belongs to the same author, and its independence is **unverified**. It should be read as self-published until shown otherwise. The controls themselves govern agent behavior rather than corpus quality, so little of the content transfers even if the framing does.

**The claims move.** The repository description, the README and cached earlier versions give the MCP tool count as 76, 82 and 62 respectively. The compression percentages and the "4-layer verification engine" are unmeasured by anyone outside the project. Roughly nine-tenths of the commits are from one author in under five months. The OKF specification itself is small, legible and backed by round-trip tests, and can be depended on directly. That judgment does not extend to the numbers around it.

### I.5 The presentation is the lesson

The most transferable thing here is not technical. LeanCTX ships a product site that covers, coherently and in eighteen languages, what most open specifications never assemble. The site has how-it-works, architecture, benchmarks, compatibility, competitor comparisons, six use-case pages, pricing, an enterprise tier, docs, changelog, community, and a compliance self-assessment. Its `robots.txt` explicitly welcomes AI crawlers under a "GEO" heading, and it serves an `llms.txt` that describes it to machine readers.

That last detail read at the time as [Q14](09-open-questions.md#q14--discovery-surface) already shipped by someone else, and the measurement since says otherwise. A study of about 137,000 domains found that 97% of valid `llms.txt` files received no requests at all in one month. Most of the requests that did arrive came from audit tools. No major model provider has stated that it reads the file. The lesson survives the correction and changes shape. A published descriptor is worth what its obliged consumer is worth, and this one has none ([§O.3](#o3-llmstxt-is-the-measured-failure-of-a-descriptor-with-no-obliged-reader)). What the site does establish stands: the discovery surface has a human half that we did not plan for at all. A corpus that nobody can evaluate from the outside is not adopted, however well it validates.

> **Recorded as [Q16](09-open-questions.md#q16--public-presence).**

## J. TrustGraph — the same pitch, the opposite mechanism

[TrustGraph](https://github.com/trustgraph-ai/trustgraph) (Apache-2.0, v1.2 shipped August 2025) is a containerized context-engineering platform. Documents flow through configurable pipelines in which **LLM agents extract entities and relationships** into a graph store (Cassandra, Neo4j, Memgraph or FalkorDB). Embeddings live in Qdrant, messages flow over Pulsar, and retrieval returns to agents as document-, graph-, or ontology-driven RAG. Its semantic layer is the standards stack — RDF, OWL, SKOS, SHACL. Every answer carries a per-fact provenance receipt: source document, ingestion timestamp, extraction method.

The pitch overlaps ours almost word for word — typed graphs that ground agents in verifiable knowledge, with provenance. The implementations are close to opposites. In Headwater, authors declare the graph: documents are the nodes, front-matter references are the typed edges, and no LLM issues a verdict. In TrustGraph, an LLM extracts the graph: documents are feedstock, dissolved into triples, and the 1.2 release headline is an agent that "autonomously populates the knowledge graph". Nothing in the platform governs the source documents — it mines them. Its retrieval modes manage exactly the reconstitution loss that §G describes. That makes TrustGraph the subject of that section, in production form. The overlap is at the slogan, not the layer beneath it.

Three things deserve a record:

- **Evidence the other way on Q13.** §E cites OpenGEO, which declines RDF/OWL/SHACL, as evidence that the standards stack is not required for this class of problem. TrustGraph is the counterweight: a production open-source system that chose that stack and ships it. One data point on each side is a more honest input to the decision than one.
- **A preview of Q15's provenance burden.** TrustGraph's extracted graph is precisely the tier that Q15 asks about — LLM-maintained, never verifiable by regeneration. Its per-fact receipts (source, timestamp, derivation method) are a concrete, operational design for the provenance record that such a tier needs. Q15 has since closed, and it read the receipts against the standard that they follow. A receipt records derivation and never endorsement, so it supplies the shape of the provenance block and none of the [warrant](01-conceptual-model.md#warrant) ([§P.1](#p1-w3c-prov-records-derivation-and-it-has-no-word-for-endorsement)).
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

> **Recorded as [Q19](09-open-questions.md#q19--inbound-integration-an-external-system-of-record), which has since closed.** The last sentence above had the fact right and the conclusion backwards. Regenerability against a pinned input is the tier axis rather than a disqualifier, and imported text is the `transcribed` [warrant](01-conceptual-model.md#warrant). W3C PROV already names it `prov:Quotation` ([§P.1](#p1-w3c-prov-records-derivation-and-it-has-no-word-for-endorsement)). An importer joins the correctness roots on the scaffolder's terms, so an imported edge carries full weight ([§P.6](#p6-requirements-practice-already-has-the-pin-and-a-better-drift-signal)).

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

> **Applied:** [Q15](09-open-questions.md#q15--a-synthesized-content-tier) closed on that leaning, and it moved the boundary off the author. The failure here is not that an agent wrote the content. It is that nobody accepted it, and nothing says so. The `asserted` [warrant](01-conceptual-model.md#warrant) is the positive mark, and [spec 5](05-ai-integration.md#the-stop-rules) forbids an agent to stamp its own output as accepted.

### L.6 A filter in the tool layer is advisory, and the documentation says so

Two regex controls sit in Serena's configuration. `read_only_memory_patterns` marks memories that the agent must not change, and the agent is told which ones. `ignored_memory_patterns` removes memories from every memory tool.

The second is the instructive one. The documentation states that an ignored memory is reachable through no memory tool. It then tells the reader to open that memory with the general `read_file` tool, on the raw path. The filter sits in the tool surface. The bytes sit in the repository. Serena's security page states the same limit about a different feature. The trust gate is "a functionality boundary, not a containment boundary".

[Q17](09-open-questions.md#q17--governed-access-and-the-solution-layer) places enforcement at the serving boundary rather than at the authoring layer. Here that position is confirmed by counterexample. A control that filters one reading path, while the bytes stay readable along another, is a convenience. `read_only_memory_patterns` is the honest half of the pair, because it constrains a cooperative agent and claims nothing beyond that.

This entry then decided more than it looked like it would. Q17 first read the serving boundary as the federated tier, and a tier that serves holds bytes that a publishing corpus already gave it. That is this same failure at one remove. So the boundary moved inward, to the export step of each publishing corpus. The Headwater MCP server then inherits the honest half of Serena's pair. It applies no filter to a corpus that its reader already holds, and it says so ([spec 5](05-ai-integration.md#what-the-server-may-do-and-the-axis-that-decides-it)).

> **Applied:** the filter placement in [spec 6](06-engine-architecture.md#an-export-profile-carries-a-filter), and the no-filter rule for the MCP server in [spec 5](05-ai-integration.md#what-the-server-may-do-and-the-axis-that-decides-it). [Q17](09-open-questions.md#q17--governed-access-and-the-solution-layer) closed.

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

> **Applied:** the grader constraint and the counterfactual obligation in [spec 5](05-ai-integration.md#measuring-whether-any-of-this-works), and a new [principle 11](00-vision-and-scope.md#design-principles), *efficacy is measured, not inherited*. Principle 10 keeps its original scope. The split between the two is the point: one tests the design, and the other tests whether it works.

## N — Transmission, harvest, and the declared subset

This section sits after the capstone above rather than before it. Its sources arrived with the [Q6, Q13 and Q9 evaluation](../evaluations/graph-export-and-federation.md) and not with the original survey, and §M binds them in full. Not one of the seven below measured whether structured knowledge changes what a reader or an agent does. What they settle is architecture, where a wrong answer costs a migration.

### N.1 A derived index is a transmission format, not a store

Sourcegraph replaced LSIF with SCIP, and the SCIP design document states the distinction outright. SCIP carries data from producers to consumers, and it is not a storage format for queries. LSIF failed on the other half of the same rule. It encoded a graph with opaque global integer identifiers, which forced an order on how symbols entered the index. Partial update of one document then became impractical, and Sourcegraph replaced the integers with human-readable string symbols.

That last detail confirms a ruling that Headwater reached for an unrelated reason. [Q4](09-open-questions.md#q4--relation-storage) made an edge identity a triple of stable strings, so that the `Edge` scope had a computable key. Sourcegraph arrived at the same shape from incremental indexing.

Software Heritage shows the same layering with two derived representations. The archive exports its tables as Apache ORC, and a separate pipeline compiles a compressed WebGraph representation *from that export*. The fast representation is regenerated rather than maintained, and the archive stays the authority.

> **Applied:** the three-artifact table and the no-database ruling in [spec 6](06-engine-architecture.md#nothing-stores-the-graph).

### N.2 Git is not a database for a derived index

Four package registries put an index in a git repository, and every large one migrated away. Cargo moved to a sparse HTTP protocol, and about 99% of crates.io requests used it by April 2025. Homebrew moved to JSON downloads in 4.0.0, after `.git` directories reached about 1 GB. CocoaPods moved to a content delivery network in 1.8, with 16,000 directories in one folder as the cause. Go made `GOPROXY` the default in 1.13, and one reported resolution fell from 18 minutes to 12 seconds.

This contradicts the casual half of the old [Q6](09-open-questions.md#q6--where-the-corpus-graph-lives-at-rest) leaning, which offered a committed graph with no cost attached. The cost is churn and size rather than principle. It does not bite at a thousand documents. It bites at the tier that harvests many corpora, which is why that tier holds pins rather than a merged graph.

### N.3 Backstage — the catalog is a read model that authors its own entries

Backstage keeps entity descriptors in the repositories that they describe. A processing loop re-derives entities continuously, and an edge that a later pass no longer emits is severed. The `relations` field is read-only, and processors generate it. The catalog also admits entities registered as static configuration rather than harvested from a repository.

That last property is [Q9](09-open-questions.md#the-aggregator-authors-its-own-facts)'s "the aggregator authors its own facts", already in production. The rest confirms rebuild over store, at a scale that Headwater will not reach soon.

### N.4 CodeQL — the contradiction worth keeping in view

A CodeQL database is created from source, uploaded as an artifact, and it *is* the query surface. That is a shipped design at very large scale, so the refusal in spec 6 is not a general truth.

What CodeQL never does is treat the database as canonical for the code, and nobody reviews a database in a pull request. Headwater declines the pattern on its own stated constraints: offline, deterministic, and reviewable in a diff. It does not decline it on a claim that the pattern fails.

### N.5 SPDX 3.0, and what LinkML's own generator drops

SPDX 3.0 is [Q13](09-open-questions.md#q13--linkml-and-shacl-as-substrate)'s option 3 at standards scale. One model generates an OWL ontology with SHACL restrictions, a JSON-LD context, and a JSON Schema through `shacl2code`. The serializations are derived rather than authored in parallel, by a standards body with many independent consumers.

The sharper result is a limit inside LinkML. A 2024 report on a semantic data link records that LinkML's SHACL generator translates its own schema inadequately. It names `any_of` and `equals_string_in` as constructs that do not survive.

Q13 argued that a pipeline routed through LinkML drops the *graph* layer, which LinkML never expressed. The measured fact is stronger. Such a pipeline also drops parts of the *shape* layer that LinkML does express, and it declares nothing. That is the reason for the rule that emitters never chain.

> **Applied:** the staging order and the no-chaining rule in [Q13](09-open-questions.md#q13--linkml-and-shacl-as-substrate) and [spec 6](06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped).

### N.6 OGC API and STAC — the subset declaration, shipped

An OGC API implementation serves a conformance endpoint that lists the conformance classes it supports, and a listed class obliges the whole capability behind it. STAC moved the same list onto the landing page, so that one request tells a client what it is talking to.

They sharpen the requirement rather than supply it. Both declare a positive set against a fixed, published universe of classes. Headwater's universe is the check registry that an adopter's own taxonomy generates, so an outside reader cannot compute the complement. The export therefore states both halves, which is a real difference and not a copy.

> **Applied:** the partition rule and the equivalence bar in [spec 12](12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule).

### N.7 Harvest beat fan-out, and GraphQL federation says why

SPARQL federation is the standards-track version of query fan-out. Under SPARQL 1.1 and 1.2, a `SERVICE` pattern that cannot reach its endpoint fails the whole query. The remedy in the specification is the `SILENT` keyword, which lets the query succeed with a silently incomplete answer. Research on public endpoints reports that low reliability pushes serious consumers onto dataset dumps and local reinstallation. Wikidata maintains a page of federation issues that lists downtime, timeouts, and protocol incompatibilities.

The digital-library field ran the same experiment for two decades. The candidates were distributed broadcast search over Z39.50 and metadata harvesting over OAI-PMH. Europeana aggregates from more than 3,700 providers through its Metis service, partly through intermediary aggregators, and DPLA harvests over the same protocol. The tiered shape matters here, because providers feed intermediaries and intermediaries feed the top. That is [spec 7](07-distribution-and-federation.md#federation)'s tiers, reached by an unrelated community under load.

GraphQL federation is the counter-example, and it proves the rule. Apollo's router does fan out at query time, on three conditions. Composition of subgraph schemas is a build-time step that yields a static supergraph artifact. Entities join on a declared `@key`, and a cross-subgraph reference carries only the key fields. Every subgraph is a live service under one operator. Headwater meets none of the three, and what transfers is the build-time half.

> **Applied:** harvest over fan-out, and the pinned export, in [spec 7](07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it).

## O — The serving boundary: descriptors, redaction, and the write path

This section sits with §N, after the capstone, and for the same reason. Its sources arrived with the [serving-boundary evaluation](../evaluations/the-serving-boundary.md), which closed [Q14](09-open-questions.md#q14--discovery-surface), [Q17](09-open-questions.md#q17--governed-access-and-the-solution-layer) and [Q7](09-open-questions.md#q7--scope-of-the-mcp-surface). §M binds them in full. Two of the sources below measure adoption rather than efficacy, and the distinction is worth holding. They say what a mechanism achieved in the field. They say nothing about whether the knowledge it carried helped anyone.

### O.1 The well-known convention is narrower than its reputation

RFC 8615 reserves a path prefix at the **root of an origin**, and an IANA registry admits suffixes under a specification-required policy. Its author's current guidance names three costs. The approach locks a service into a one-to-one relation with a site. It misdescribes a host that represents several publishers. And it needs control of the apex. The recommendation is direct: where a protocol can carry a full URL, a well-known location earns nothing.

A corpus meets all three objections at once. A repository holds one or more corpora. A documentation site is often one part of a larger host. And the party who writes a descriptor rarely owns the root. RFC 9727 registers a **link relation** beside its well-known path for the same reason, and that half is the one a corpus can always use.

> **Applied:** the canonical descriptor location is repository-relative, and a pointer reaches the served copy ([spec 7](07-distribution-and-federation.md#arriving-at-a-corpus-cold)).

### O.2 Registration is configuration, in every registry that ships a descriptor

Three package ecosystems put a capability document at a fixed path inside an index. None of them discovers the index. The Python simple-repository API takes the index URL from configuration and reports its own version inside each response. Cargo takes a scheme-tagged index URL from configuration and reads `config.json` at the index root. The npm client takes the registry URL from configuration and negotiates capability by content type.

That settles what a fixed-path descriptor is for. It describes an already-known location, and it does not make a location known. The Python contract is the part worth copying whole. Every response carries a `Major.Minor` version. A major above what the client understands is a hard failure, and a minor mismatch is a warning.

The sitemap protocol adds the two-level shape and one detail that is easy to miss. An index names up to 50,000 sitemaps, and each of those names up to 50,000 URLs. The index carries a location and a modification date and nothing else. The fixed path is not the sitemap. It is the robots file, which carries a *pointer*.

> **Applied:** the split between registration and resolution, and the version contract, in [spec 7](07-distribution-and-federation.md#arriving-at-a-corpus-cold) and [Q14](09-open-questions.md#q14--discovery-surface).

### O.3 `llms.txt` is the measured failure of a descriptor with no obliged reader

The convention was proposed in September 2024, and it is close in spirit to what Q14 leaned toward. A study of about 137,000 domains reports that 28% publish one. Of roughly 38,000 valid files, **97% received no requests at all in one month**. About 96% of the requests that did arrive came from bots, and only about a fifth of those were named AI tools. No major model provider has stated that it consumes the file.

The published diagnosis is structural rather than aesthetic. The format cannot work without cooperation from parties who never agreed to cooperate. The sample is self-selected and the adoption figure is biased upward, which makes the zero-request figure the reliable half.

This corrects [§I.5](#i5-the-presentation-is-the-lesson), which read the file as Q14 already shipped by somebody else. What survives is sharper than what it replaces. A descriptor is worth exactly what its obliged consumer is worth. Headwater's first consumer is its own tooling, which it controls and can oblige, and that difference is the whole reason to build one.

> **Applied:** the obliged-consumer argument in [Q14](09-open-questions.md#q14--discovery-surface), and the caution carried into [Q16](09-open-questions.md#q16--public-presence).

### O.4 Absence reads as presence, and a declaration needs a verifier

A survey of seventy-four API providers for the standardized catalog path found four real documents. **Sixty-eight returned a success status with an unrelated HTML page**, because a catch-all route answers every path. Two returned a clean not-found. The sample is curated and point-in-time, and the failure mode it exposes is not.

The declaration side has the same shape. An OGC API implementation serves the conformance classes that it supports, and a listed class obliges the whole capability behind it. STAC moved that list to its landing page so that one request answers the question. A separate validator exists to test each declared class against the live service, and its documentation records real classes of wrong declaration in the wild. This survey found no published statement that conformance lists routinely lie. The weaker evidence carries the point anyway. Nobody writes a rule that a declared class must be implemented unless the rule has been broken.

One header from a neighboring domain belongs here rather than with the redaction sources, because it is the same idea in miniature. When an OCI registry applies a server-side filter to a referrers query, it must return a header saying that a filter was applied. A partial answer announces itself in band, so a client can separate "no results" from "I did not look".

> **Applied:** the required media type and shape check in [spec 7](07-distribution-and-federation.md#arriving-at-a-corpus-cold), and the announce-the-filter rule in [spec 6](06-engine-architecture.md#an-export-profile-carries-a-filter).

### O.5 The annotations are hints, and the attacks arrive before the call

The Model Context Protocol gained tool annotations in its March 2025 revision, and they survive unchanged into the current one: `readOnlyHint`, `destructiveHint`, `idempotentHint` and `openWorldHint`. The normative sentence is that clients **must** consider tool annotations untrusted unless they come from trusted servers, and the schema repeats it. The properties are hints, and they are not guaranteed to describe behavior faithfully. The specification never defines a trusted server.

Two details are worth keeping. An omitted `readOnlyHint` means false, and an omitted `destructiveHint` means true. Silence means "assume destructive", which is the same error asymmetry that [principle 7](00-vision-and-scope.md#design-principles) states for an exporter. And the shipped read-only modes of a large forge's reference server filter at registration, so the flag describes a tool that is not there.

The published attacks defeat the obvious safety argument. The line-jumping result is the sharpest. A payload in a tool description enters a model's context at *discovery* time, before any call. It therefore bypasses invocation-time approval entirely, and the malicious tool never has to run. Tool poisoning and the mutable-description case are the same family. All of these are vendor disclosures and preprints, and this survey found no peer-reviewed source among them.

One observed exploit ties this to §L.6 and to the access question. An attacker filed an issue on a public repository. A user asked their agent a benign question, and the agent read private content and published it **by opening a proposal on the public repository**. The write tool was the exfiltration channel. Two supply-chain incidents have the same shape at another layer. One compromised a bot credential that held write access. One exfiltrated secrets by creating a public repository and committing to it.

The protocol supplies no approval primitive. Its human-in-the-loop language is a *should* addressed to clients, and it concedes that it cannot enforce its security principles at the protocol level. Its elicitation feature is a structured-input channel rather than an authorization one. So propose-rather-than-commit is a Headwater decision with no protocol vocabulary behind it.

> **Applied:** the three tool classes, the annotation-is-not-enforcement rule, and the disclosure-channel argument in [spec 5](05-ai-integration.md#what-the-server-may-do-and-the-axis-that-decides-it).

### O.6 Propose and do not land, and what actually confines a proposer

A production coding agent at a large forge implements the ruling that [Q7](09-open-questions.md#q7--scope-of-the-mcp-surface) reached. It pushes to one namespaced branch and no other. It opens a draft proposal, cannot mark that proposal ready, cannot approve it, and cannot merge it. Continuous integration does not run until a person with write access approves. Several dependency bots follow the same shape. One governance tool takes the cheapest posture of all, and reports an issue rather than authoring a change.

**The permission split alone does not deliver it, and that is the sharpening.** On the platform whose generated permission data this survey checked directly, creating a proposal needs one permission and merging needs another. That looks like the separation. It is not, because creating the branch that a proposal points at needs the same permission that merging needs. A proposer that authors its own branch is therefore not confined by its credential. Two mechanisms confine it: a branch rule that requires review and grants the proposer no exemption, or a separate repository that the proposer owns.

**A proposal channel is priced by selectivity, and this part is peer reviewed.** A 2017 study of 7,470 projects found that automated proposals raised upgrade frequency by about 1.6 times. Only about a third of those proposals merged, against roughly four fifths for ordinary ones within three days. A 2021 study of 2,904 projects found that about 65% of *security* proposals were accepted, often within a day. A 2024 journal study over 9.9 million proposal-related issues confirms the scale. The mechanism is identical in all three, so what moves the number is what the proposer chooses to propose. Notification fatigue was the top reported complaint in 2017, and the standard remedy is a cap on open proposals.

**Two field results about shared write access.** A long-running collaborative packaging organization closed in March 2026. It reported that only about one in ten machine-generated proposals met project standards. It also reported that an organization which gives push access to everyone can no longer operate safely. A widely used project ended its bug bounty in January 2026. Low-quality machine-generated reports had driven its confirmation rate below 5%, from a historical figure above 15%. Its intake later reopened and the bounty did not. Neither result is about documentation, and both are about what an unselective write channel does to the people who maintain the thing.

> **Applied:** the proposal budget and the confinement statement in [spec 7](07-distribution-and-federation.md#upstream-awareness).

### O.7 Filter where the data is written, not where it is read

Multilevel-secure database research ran this comparison in the 1980s and wrote down the answer. Lunt's 1989 paper on aggregation and inference sets SeaView against LDV. SeaView applies classification once, when data enters. LDV applies it on every access. Her verdict on the second is that a low user may infer high information from the results of their own queries. Different information is released depending on the context in which the query was posed. She adds that the read-side design drags a large part of the database mechanism into the trusted base.

That is the ruling of [Q17](09-open-questions.md#q17--governed-access-and-the-solution-layer), reached forty years earlier in another discipline. It also names a second cost that this design would otherwise have to discover. A filter that runs on every read makes the *behavior of the filter* an inference channel.

The redaction literature says the same thing in the crudest possible form. Four dated public cases exist in which a black rectangle covered text that was still in the file. A copy-and-paste recovered it in every one. The authoritative guidance is unambiguous. Information hidden or covered in a computer document can almost always be recovered, so the item must be **deleted** rather than obscured. Where deletion breaks the layout, the guidance says to replace it with meaningless content of the same size. That is a tombstone, specified as engineering.

> **Applied:** filtering at the export step, and a placeholder rather than a cover, in [spec 6](06-engine-architecture.md#an-export-profile-carries-a-filter).

### O.8 "3 documents withheld" is a named channel, and the field named it in 1986

The withheld count has a name in this literature. Denning called it the **missing data inference channel** in 1986, and Lunt records the conditional that governs it. Holes in a relation are correct when the existence of the hidden item is not itself sensitive, and they are a channel when it is.

The field's own answer to the second case is the one that Headwater declines. Multilevel databases invented **polyinstantiation** and **cover stories** so that a filtered view would look complete. The standard reference then documents the cost. Real-world entity integrity is lost. High users receive a mix of real data and cover stories with no explanation. And nobody can later tell reality from cover story from data-entry error.

So the two branches are both known and both costly, and the choice between them is conditional. That is the `counted` and `sealed` grain, and it is a switch rather than a preference.

The freedom-of-information statute reaches the identical conditional from the other direction. It requires three things at a deletion. The amount deleted. The place in the record where the deletion is made. And the rule under which it was made. Its exception is the same one. The indication is omitted where including it would harm the interest that the exemption protects. A 1973 appellate decision supplies the reason for the default. The party who wants a record cannot argue about content that it cannot see, so only an itemized account restores the argument. The civil-procedure counterpart requires a description that lets another party assess the claim "without revealing information itself privileged or protected". A protocol precedent exists too. The HTTP status code for a legal obstacle asks for the demanding party and the applicable rule. It then concedes that a client cannot rely on receiving either.

Three consequences follow, and the third is the one that a footer count misses.

- The default is legible, because that is the usual obligation in every source above.
- The exception is narrow and separately justified. Courts treat a refusal to confirm or deny as an answer that must itself be defended, and not as a default.
- **Amount, position, and reason** is the specification. A count in a footer gives one of the three. A placeholder at the position, carrying the rule identifier, gives all three. The reason comes from a closed set, because free prose in a tombstone is a second channel.

The clearest working example of the switch is one vendor's own product. It answers a request for a private repository with "not found" rather than "forbidden", so the reply does not confirm existence. The same product answers a legal takedown with a status code that names the rule, and it publishes the notice. Illegible for permission, legible for rule, in one system, deliberately.

**And the honest limit.** The 1996 standards report on inference and aggregation states that eliminating inference is difficult if not impossible. It calls classification rules a constant trade-off. And it carries the sentence that this design should adopt: inference controls mean that complete data correctness is not always possible for the low reader. The covert-channel guidance of the same tradition sets bandwidth thresholds rather than a prohibition. The posture is to bound and declare a channel, never to claim its absence.

> **Applied:** the declared tombstone grain, the closed reason set, and the non-claims in [spec 6](06-engine-architecture.md#what-a-filtered-export-claims-and-what-it-does-not).

### O.9 The derived artifact is where a correct redaction leaks

One case is worth more than the four in §O.7 because the redaction *held*. A deposition shipped with its text correctly removed. It also shipped with an alphabetized word index that covered redacted and unredacted words alike, so the alphabetical neighbors bracketed every hidden term. Electronic-discovery guidance carries the general form of the rule. When a redaction is burned into an image, the extracted searchable text layer has to be withheld or regenerated. Otherwise it reproduces exactly what the image hides.

A database vendor supplies the same failure in a supported feature. Creating a table from a query over a row-filtered table copies the filtered rows and does **not** carry the policy. The data crosses and the rule does not.

A governed corpus produces exactly the metadata that betrayed that deposition: indexes, counts, sort orders, link degrees, navigation trees. So a filtered profile regenerates every projection that it carries, from the filtered graph. A projection built once at full visibility and then shipped inside a filtered profile is the leak. It is the class of leak that survives a correct redaction of the documents.

> **Applied:** the regenerate-from-the-filtered-graph rule in [spec 6](06-engine-architecture.md#an-export-profile-carries-a-filter).

### O.10 What the filtered result may say, and what a permission model would have cost

**Any diagnostic that a full-visibility component emits is a channel.** PostgreSQL states the general rule in its own documentation for row-level security. A function is `LEAKPROOF` only when it reveals nothing about its arguments except through its return value. A function that puts argument values into an error message is therefore not leakproof. Only a superuser may make the declaration. The same documentation records that referential-integrity checks bypass row security by design, and warns in those words about covert-channel leaks through them.

That is the reason a withholding reason comes from a closed set. A component that sees everything and writes free prose is a leakproof violation with better grammar. The cloud vendors supply the rest of a threat model without being asked. Query duration, billing amount, and the errors raised when a query exceeds a limit are each named as an observable channel over repeated attempts.

**The permission model that Headwater does not build has a documented price.** One large collaboration product breaks permission inheritance per page. Its own support base records inheritance that silently stops applying after an import. The derived table that the computation reads has diverged from the visible tree. Another product caps unique permission scopes per list, warns that query performance degrades as they grow, and refuses to break inheritance past the cap. Both are evidence for the ruling that Headwater keeps one permission system, and that it is the platform's.

**And the systems that answer the question we never ask.** Centralized authorization services model access as relation tuples with rewrite rules and answer at request time. Object capabilities make designation and authority one thing and answer at the moment of use. Bearer credentials with attenuating caveats let a holder narrow a token offline and let a service verify it without a callback. All three answer whether a principal may act on an object *now*. Headwater has no principal and no now, so all three are recorded here as considered and declined, and nobody has to run the comparison again.

Two of them leave a mark on the design anyway.

- **The staleness objection is real and it is theirs.** The centralized service exists partly to stop an old permission from reaching new content. It carries a freshness token on every answer to prove that it did not. A pinned export is that failure by construction, because a permission revoked after the pin holds in the tier until the next harvest. No version of harvest-not-fan-out removes it, so the specification declares the cadence instead ([spec 7](07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it)).
- **The attenuating-credential literature names a bug that a schema change would otherwise cause.** A credential that lists what it forbids silently widens when the target grows a new operation. The stated remedy is default-deny on anything new, plus a version. An export filter written as a list of exclusions has the same defect the first time a taxonomy adds a node class.

**One acronym is a red herring, recorded so that nobody spends an afternoon on it.** The messaging protocol called MLS is a group key-agreement protocol and shares only its initials with multilevel security. Its own architecture document places authorization out of scope.

> **Applied:** the closed reason set and the default-deny filter in [spec 6](06-engine-architecture.md#an-export-profile-carries-a-filter), and the declared harvest cadence in [spec 7](07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it).

### O.11 The obligation attaches to the published claim

One vendor's servicing criteria give the clearest available answer, and it is a decision procedure rather than a threshold. A report earns a security fix when it violates the goal or intent of a **security boundary** that the vendor has published. The same document then enumerates what is deliberately **not** a boundary, and a bypass of a non-boundary is not a vulnerability.

So the obligation does not attach when a project writes a filter. It attaches when the project publishes a sentence that says a boundary holds. [Spec 6](06-engine-architecture.md#what-a-filtered-export-claims-and-what-it-does-not) therefore states one claim and five non-claims, and the non-claims are the more useful half. They convert the declared tombstone channel, the shape leak, and the revocation lag from latent defects into stated limits.

Around that sit the ordinary process standards. Two ISO standards split the external disclosure interface from the internal handling process. A long-running coordination guide describes the roles, and an identifier-assignment scheme defines what a supplier may assign against. A published policy and a stated contact are the cheap form of all of it. There is also a date that does not care what the project calls itself. European product regulation binds vulnerability-reporting duties for products with digital elements from September 2026, and full handling duties from December 2027.

> **Applied:** the claim and the non-claims in [spec 6](06-engine-architecture.md#what-a-filtered-export-claims-and-what-it-does-not), and the disclosure obligation in [Q17](09-open-questions.md#q17--governed-access-and-the-solution-layer).

### O.12 The platform permission that this design leans on has a documented hole

[Q17](09-open-questions.md#q17--governed-access-and-the-solution-layer) rests enforcement on the hosting platform's repository permissions. One published analysis qualifies that, and the vendor confirms the behavior as intended rather than fixing it.

Commits in a fork network stay reachable across that network. A commit pushed to a fork that somebody later deleted remains reachable through the upstream by its hash. Code committed to a private fork before the upstream became public becomes public with it. Short commit prefixes are guessable. The vendor's own documentation states that commits to any repository in a fork network are accessible to all repositories in it.

So the premise holds for the current tip of a repository that has never been forked and never changed visibility. It is qualified otherwise. That does not move the ruling, because no alternative placement is better. It does mean that the export step is not merely where filtering is best done. For a corpus with that history, it may be the only place where filtering happens at all.

> **Recorded** as a qualification on [Q17](09-open-questions.md#q17--governed-access-and-the-solution-layer)'s reliance on platform permissions.

## P — Provenance, endorsement, and the record of a judgment

This section sits with §N and §O, after the capstone, and for the same reason. Its sources arrived with the [warrant evaluation](../evaluations/warrant-and-adjudication.md), which closed [Q15](09-open-questions.md#q15--a-synthesized-content-tier), [Q19](09-open-questions.md#q19--inbound-integration-an-external-system-of-record) and [Q18](09-open-questions.md#q18--recording-adjudicated-disagreements). §M binds them in full. One source below carries a measured result about a mark, and none of them measures whether the mark changes what a reader does.

### P.1 W3C PROV records derivation, and it has no word for endorsement

PROV models an **entity**, an **activity**, and an **agent**, and it relates them through `prov:wasGeneratedBy`, `prov:wasDerivedFrom` and `prov:wasAttributedTo`. Its derivation subtypes are `prov:Revision`, `prov:Quotation` and `prov:PrimarySource`. `prov:Quotation` is the repeat of part or all of an entity by somebody who may not be its original author.

That last definition is Q19's imported requirement text, under a standard name that predates the question. It supplies the `transcribed` value of the [warrant](01-conceptual-model.md#warrant) whole.

The contradiction is larger than the confirmation. PROV says what happened to an entity and who took part. It has no vocabulary at all for the claim that a party stands behind a result. All three entries treat "record the provenance" as the answer, and a provenance record is a record of derivation. `accepted_by` is the addition, and [spec 2](02-taxonomy-model.md#lineage-aligns-with-prov) claimed alignment with PROV without saying that the alignment stops one field short.

One further point serves Q18. A PROV record is itself an entity, so it may carry provenance of its own. An adjudication that names an adjudicator wants exactly that recursion, and the cheapest artifact that already has it is a document.

> **Applied:** the warrant in [spec 1](01-conceptual-model.md#warrant), and the correction to the PROV claim in [spec 2](02-taxonomy-model.md#lineage-aligns-with-prov).

### P.2 SPDX makes "nobody asserted anything" a value

A license field in SPDX may hold `NONE` or `NOASSERTION`, and the two mean different things. `NONE` says that the document states there is no license. `NOASSERTION` says that no party made a claim either way. SPDX 3.0 goes further and puts `creationInfo` on every element with no exception.

That settles what would otherwise be a coin toss. An unwarranted document carries a positive mark and never an absent field, because a consumer cannot tell an absent field from an unknown value. A standard with a very large installed base reached the same conclusion for the same reason.

> **Applied:** an absent warrant is a finding rather than a default ([spec 1](01-conceptual-model.md#warrant)).

### P.3 C2PA measures what happens to a mark in transit

Content Credentials bind a signed manifest to an asset, with assertions about capture, editing, and the use of generative tools. The specification's own threat model names manifest removal as a live case. The ecosystem's answer is a durable binding through watermarking and fingerprinting, because ordinary tooling strips metadata that it never intended to strip.

That sharpens the constraint that [Q17](09-open-questions.md#q17--governed-access-and-the-solution-layer) handed this group, and it moves the ruling. Q15 proposed that an emitter which cannot carry the mark declares that in its loss set. A loss-set entry serves the consumer who reads it and nobody else. The consumer who does not read it receives unwarranted content that looks vouched. So the emitter withholds the content instead.

C2PA's second contribution is the signer. A manifest names an accountable party and validates against a trust list. A provenance mark with no named party is decoration, which is Q18's own test generalized past adjudication.

> **Applied:** the withhold-rather-than-drop rule in [spec 6](06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped).

### P.4 Wikipedia enforces on the review, not on the author

The core content policy is verifiability: material must be attributable to a reliable published source, and the burden sits with the editor who adds it. A companion policy forbids original research. The slogan that the community used for years was "verifiability, not truth", which is the separation that this group needed. The project does not ask whether a sentence is true. It asks what stands behind it.

The response to machine-generated content is the transferable half. The community first tagged suspected model output with a template. In 2025 it added a speedy-deletion criterion for unreviewed model output, aimed at hallucinated citations and unedited chatbot artifacts. The trigger of that criterion is not that a model wrote the text. It is that nobody reviewed it.

That is the reframing of [Q15](09-open-questions.md#q15--a-synthesized-content-tier), confirmed by the largest observed instance of the problem, and by a community that tried the weaker control first.

**The contradiction is the more useful half.** Most Wikipedia prose carries no inline citation. The policy demands attributability rather than attribution, and the encyclopedia is useful anyway. So a corpus that holds unwarranted content is not worthless. It is useful in proportion to how cheaply a reader can check it, and the inline `citation needed` mark is what keeps that cost visible. That contradicts any ruling that would forbid unwarranted content, and it supports the ruling that admits it and marks it.

One thing that Wikipedia does and this design declines. The `citation needed` mark sits inside a paragraph, at sub-document grain. [Q17](09-open-questions.md#q17--governed-access-and-the-solution-layer) already refused a filter that reaches inside a body, and the same reasoning holds here. A warrant is per document.

> **Applied:** the `asserted` warrant, admitted and marked, in [spec 1](01-conceptual-model.md#warrant) and [spec 3](03-authoring-and-lifecycle.md#the-warrant-and-what-each-value-requires).

### P.5 The two observed cases in this survey, read against the standards

[§J](#j-trustgraph--the-same-pitch-the-opposite-mechanism) records TrustGraph's per-fact receipts of source, ingestion timestamp and extraction method. Read against §P.1, a receipt is a derivation record and not an endorsement record. It supplies the shape of the provenance block and none of the warrant. It also works at per-fact grain, which §P.4 declines.

[§L.5](#l5-onboarding-ships-the-synthesized-tier-and-marks-nothing) records the opposite failure in a tool with a very large installed base. Onboarding writes synthesized content into the same directory, in the same file shape, as memories that a human wrote. Nothing marks the difference. That is the outcome that a refusal to model the tier produces, observed rather than predicted.

[§F.2](#f2-karpathys-llm-wiki) records ingest, query and lint as the operations of the pattern that raised Q15. Lint is a cohesion mechanism. It finds orphans, undefined concepts and contradictions between pages, and it establishes nothing about whether a page is true. So the pattern ships no warrant at all, which confirms the entry that it raised.

### P.6 Requirements practice already has the pin, and a better drift signal

Baselining is the pinned snapshot under the name that the field gave it. A requirements set is frozen and identified, and traceability is evaluated against the baseline rather than against the live set. [§K](#k-modern-requirements--the-first-candidate-where-the-arrow-reverses) records that the tool in question mints baselines as work items. So the pin of [Q9](09-open-questions.md#q9--multi-repository-corpora)'s one-pattern rule is the upstream's own concept here.

The sharper result is the **suspect link**. DOORS-family tooling flags every trace link into a requirement when that requirement changes, and the flag clears only when a person confirms the link. Q19 proposed a change proposal against the snapshot, which is correct and too coarse. A finding on each affected edge names the document whose author can act, and that is [spec 4](04-assurance-model.md#absence-is-a-finding-class-of-its-own)'s report-at-the-origin rule.

ReqIF is a real OMG standard, and this survey places it on the adopter's side rather than the engine's. It exists so that two requirements tools can exchange a set without either one owning the format. Headwater reads identities and text and writes nothing back, so the value accrues to an organization that wants to leave its current tool.

> **Applied:** the snapshot properties and the drift rule in [Q19](09-open-questions.md#q19--inbound-integration-an-external-system-of-record) and [spec 7](07-distribution-and-federation.md#upstream-awareness).

### P.7 Debian settles redistribution by segregating the archive

Software whose terms Debian cannot pass on lives in a separate archive area, and a default installation carries only the free one. An administrator enables the rest explicitly, and the act is visible in a configuration file. Machine-readable per-file copyright records sit beside that, and an embedded copy of another project must be declared.

That is the shape of the answer to Q19's redistribution point, and Headwater needs no new mechanism for it. The [export filter is already default-deny over classes](06-engine-architecture.md#an-export-profile-carries-a-filter), so imported text stays inside the repository unless a profile names it.

> **Applied:** the license non-claim in [spec 6](06-engine-architecture.md#what-a-filtered-export-claims-and-what-it-does-not).

### P.8 Legal citators argue against Q18's ruling, and then for it

Shepard's and KeyCite attach a treatment to a **pair** of cases. A later opinion overrules, distinguishes, criticizes or follows an earlier one. That is an edge attribute, shipped for over a century, in the field that thought hardest about how a later judgment binds an earlier one. Read alone it argues for Q18's leaning and against the ruling that closed it.

Two further facts reverse it.

**The signal is an index over a document and never the authority.** An editorial process derives the flag from a published opinion. The opinion is the record, the flag is a lookup surface, and no practitioner treats the flag as the holding. In Headwater's vocabulary the opinion is the document and the flag is a projection.

**Two citators over the same case law disagree at a measured rate.** Published comparisons of the two major services report substantial divergence in how they treat the same decisions. A derived scalar produced by a party who did not make the judgment is not reliable enough to be the record. That is Q18's own sentence about the authority rank, confirmed at the largest deployed instance of the pattern.

**Retraction practice lands where the ruling lands.** A retraction in scholarly publishing is a separate citable object with its own identifier, and it points at the work that it retracts. The retracted work is marked and kept. Crossref carries the relationship as metadata between two registered items.

CrossMark adds the detail worth taking whole. It does not try to make a flag survive a copy. It puts a resolvable pointer to the authority inside the copy. A reader can then go back and ask whether the copy in hand is current. A transcribed document carries its pin for the same reason.

> **Applied:** the adjudication as a document, and `overrides` as its edge, in [spec 2](02-taxonomy-model.md#disagreement-is-adjudicated-not-ranked). The exported pin in [spec 6](06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped).

---

## Q — What a lexical rule gets wrong, and what a merge queue buys

This section arrived with the [Q5 and Q21 evaluation](../evaluations/what-a-check-can-know.md). §M binds it: convergence on a shape is not evidence that the shape works. Two of the sources below carry measured numbers, and one of the two is our own repository.

### Q.1 Every controlled-language checker ships the same shape, and none of them blocks on voice

Vale, `textlint`, `proselint`, `write-good` and `alex` are rule sets over text, with per-rule severity and a per-line disable comment. Vale ships three levels, `error`, `warning` and `suggestion`, and its published style packages put readability and voice at the lower two. That is the design that [Q5](09-open-questions.md#q5--voice-checking-depth) leans toward, arrived at five times independently.

`alex` is the closest thing to a retired-term lexicon that ships. It carries terms with replacements and reasons, it is advisory by default, and it is best known for its findings on quoted and technical text. Its documented remedy is a scoped ignore comment.

The convergence confirms the shape and says nothing about the posture. Not one of these tools blocks a build by default, and their maintainers say why in the same words that this evaluation reaches. A finding that needs a rewrite is a prompt and not a gate.

> **Applied:** posture by fixability rather than by precision ([spec 3](03-authoring-and-lifecycle.md#what-a-lexical-rule-gets-wrong-and-where-posture-comes-from)), and the permanently advisory class ([spec 4](04-assurance-model.md#where-promotion-cannot-finish)).

### Q.2 The static-analysis field measured what a developer tolerates, and it is less than intuition

Bessey and colleagues ("A Few Billion Lines of Code Later", *CACM* 2010) report a decade of commercial deployment. Two results transfer. A checker whose false-positive rate climbs past roughly a third gets switched off, whatever it finds. And a correct finding with no clear remediation is ignored, which costs the same as a wrong one.

Google's Tricorder (ICSE 2015) states a stricter operating point. An analyzer stays in the review pipeline only while its "not useful" rate stays under about a tenth. That rate comes from the reviewers who press the button, and never from an auditor.

Tricorder's instrument is the sharper one for us, and the reason is the blind spot that [spec 4](04-assurance-model.md#promotion-advisory-to-blocking) already names. It measures the reaction of a reader, and a reader who ignores an advisory finding leaves no label behind. Both numeric claims here come from memory. The session that wrote this evaluation had no web budget left to re-verify them, and that is recorded rather than hidden.

> **Applied:** the adjudicated sample as the only instrument that reaches an advisory rule ([spec 4](04-assurance-model.md#where-promotion-cannot-finish)).

### Q.3 Merge queues are the complete answer, and they charge for it

The "not rocket science rule", stated by Graydon Hoare for Rust's `bors`, has one clause. Never merge a commit that has not passed its tests in the merged state. GitHub merge queue, Zuul and `bors` all implement it.

Zuul is the instructive one, because it shows the cost and the recovery. Serialized landing destroys throughput on a busy repository, so Zuul gates a speculative future state in which several changes have landed together. When an earlier change fails, it discards the speculation behind it and re-runs. That is optimistic concurrency with a rollback, and the whole design turns on control of the landing order.

Headwater does not control the landing order and should not want to. A queue needs one cheap answer from a checker: does the earlier verdict still apply? The read set supplies it.

> **Applied:** the engine emits and never orders ([spec 6](06-engine-architecture.md#ci-adapters)). [Q7](09-open-questions.md#q7--scope-of-the-mcp-surface) drew the same line for the write path.

### Q.4 Incremental build and incremental typecheck solved the cheap half already

Bazel keys an action on a hash of its declared inputs, and reuse is correct exactly while that declaration is complete. Salsa, and the red-green algorithm inside `rustc`, memoize a query together with the queries that it read. A recorded dependency that changes invalidates the result.

One rule sits under both. A result carries the set of inputs that it depended on, and it survives a change only while that set is untouched. [Spec 12](12-check-layer.md#scope--the-declaration-everything-else-rests-on) already says the hard half of it, in the sentence that calls an omitted input a correctness bug rather than a performance bug.

> **Applied:** the read set of a run, and the merge as an ordinary change ([spec 12](12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict)).

### Q.5 Terminology retirement at scale carries by tooling, never by memory

Git's own default-branch rename in 2020 is the clearest observed case. The judgment took one discussion. Its propagation took a configuration key (`init.defaultBranch`), changes across every forge and every CI product, and years of residue. The inclusive-language migrations across the Linux kernel, the large clouds and the language ecosystems ran the same way. Each one shipped a lexicon with replacements and reasons.

The durable pattern in all of them is the split that [spec 2](02-taxonomy-model.md#the-language-regime-carries-the-terms-that-the-corpus-retired) adopts. A retirement with a replacement becomes a rewrite that tooling performs, at scale, with no argument. A retirement of a framing has no replacement to substitute, so it produces discussion instead of change. It needs a reason, recorded where a later author will meet it.

> **Applied:** `retired_terms` in the language regime, with a required reason and an optional replacement ([spec 2](02-taxonomy-model.md#the-language-regime-carries-the-terms-that-the-corpus-retired)).

---

## Summary

| Source | Verdict | Outcome |
|---|---|---|
| TBox/ABox framing | Adopt the vocabulary | Applied — specs 1, 2, 6 |
| testerstories, spec → ontology → implementation | Strongest supporting evidence found | Applied — oracle and citation (specs 2, 4, 5). Authority adopted, then cut by review |
| LinkML | May already be half of spec 2 | Closed — **Q13**. Emitted, never authored, and last in the emitter order |
| SHACL | May already be the check layer | Closed — **Q13**. A compilation target, and never the validator |
| OpenGEO | Different direction, same substrate | Discovery gap recorded — **Q14** |
| KG chunking for RAG | Framing useful, chunking not applicable | Non-adoption reasoned and recorded |
| r/OntologyEngineering | Ontology-first methodology, further than we go | Noted. Oracle work is the shared ground |
| Karpathy, *LLM Wiki* | Best anti-RAG argument, independent capture-cost confirmation | Applied — coherence sweep (spec 4). **Q15** closed: the tier is real and its axis is the warrant |
| LeanCTX / OKF | Same substrate, opposite arrow. No taxonomy to collide with | Applied — declaration boundary (spec 4). OKF export → **Q13**. Presentation gap → **Q16** |
| TrustGraph | Same pitch, opposite mechanism — extracts the graph we declare | Standards-stack counterweight → **Q13**. A receipt records derivation and not endorsement → **Q15**. Ingestion integration candidate |
| Modern Requirements / Azure DevOps | First inbound candidate — the arrow reverses | Model already fits. Applied — the `transcribed` warrant and the importer as a correctness root. **Q19** closed |
| Serena | Fourth arrival at the substrate. The first to disagree with us | Applied — third anti-retrieval argument (spec 5). Scent placement → **Q20**. Evidence → **Q15**, **Q17** |
| W3C PROV, SPDX, C2PA | A provenance standard records derivation. None of them records who vouched | Applied — the warrant, and a positive mark rather than an absent field (specs 1, 3) |
| Wikipedia verifiability, and its rule for machine output | The trigger is the absence of review, and never the presence of a model | Applied — agency is not the warrant. **Q15** closed |
| Baselines, suspect links, ReqIF | The pin has an industry name, and drift belongs on the edge | Applied — snapshot properties and edge-level drift (spec 7). **Q19** closed |
| Debian archive areas | Content whose terms you do not control travels by an explicit act | Confirms default-deny. Applied as a license non-claim (spec 6) |
| Shepard's and KeyCite, Crossref retractions | A treatment flag is a projection over a document, and two citators disagree | Applied — the adjudication is a document, and `overrides` is its edge. **Q18** closed |
| **The survey as a whole** | Convergence on the substrate, and near-zero measurement of the claim | Applied — grader and counterfactual constraints (spec 5), and a new principle 11, *efficacy is measured, not inherited* |
| SCIP and LSIF, Software Heritage | An index is a transmission format, not a store | Applied — the three artifacts and the no-database ruling (spec 6). **Q6** closed |
| Package-manager indexes in git | A committed derived index is priced by churn, not by principle | Applied — the pin at the federation tier, not a merged graph (spec 7) |
| Backstage | Rebuild over store, and the aggregator authors its own entries | Confirms **Q9**. No change needed |
| CodeQL | A derived store *is* a query surface at scale | Contradiction kept in view, in spec 6 |
| SPDX 3.0, LinkML's SHACL generator | Model-first works. A chained emitter drops what it never declares | Applied — staging order and no chaining. **Q13** closed |
| OGC API, STAC conformance | The declared subset, already an industry convention | Applied — partition rule and equivalence bar (spec 12) |
| SPARQL federation, OAI-PMH aggregators, GraphQL federation | Harvest, and never fan out | Applied — the harvesting tier (spec 7). **Q9** closed |
| Well-known URIs, robots.txt, sitemaps | A fixed path describes a known location, and never makes one known | Applied — the descriptor and the registration split (spec 7). **Q14** closed |
| Package-registry indexes, OGC and STAC conformance | The index is configuration. A version needs a client rule, and a declaration needs a verifier | Applied — the version contract and the shape check (spec 7) |
| `llms.txt` | A descriptor with no obliged reader goes unread, measurably | Corrects §I.5. Carried into **Q16** |
| MCP tool annotations, tool poisoning, line jumping | The annotation is a hint, and injection lands before any call | Applied — the three tool classes (spec 5). **Q7** closed |
| Vale, `textlint`, `proselint`, `alex` | Five arrivals at the same shape, and not one of them blocks on voice | Confirms the shape, and decides nothing about posture. **Q5** closed |
| Coverity in the field, Google Tricorder | A developer tolerates less than intuition says, and ignores what has no fix | Applied — posture by fixability, and the permanently advisory class (specs 3, 4) |
| `bors`, GitHub merge queue, Zuul | Testing the merged state is complete, and it costs control of the landing order | Applied — the engine emits and never orders (spec 6). **Q21** closed |
| Bazel, Salsa, `rustc` red-green | A result carries the inputs it read, and survives only while they hold | Applied — the read set of a run (spec 12) |
| The git branch rename, inclusive-language migrations | A retirement carries by tooling. A replacement is what makes it mechanical | Applied — `retired_terms` in the language regime (spec 2) |
| Forge coding agents, dependency bots, Allstar | Propose and never land. The credential alone does not confine a proposer | Applied — the proposal budget and the confinement statement (spec 7) |
| MLS databases: SeaView against LDV, cover stories, the inference reports | Classify once at write. A visible hole is a named channel, and its acceptability is conditional | Applied — export-step filtering and the declared tombstone grain (spec 6). **Q17** closed |
| Vaughn v. Rosen, 5 U.S.C. § 552(b), FRCP 26(b)(5), HTTP 451 | Amount, position and reason, at the site of the cut — unless the marking causes the harm | Applied — the `counted` placeholder and the `sealed` exception (spec 6) |
| Redaction failures, NSA redaction guidance, the indexed deposition | Delete rather than cover. The derived artifact is where a correct redaction leaks | Applied — no partial-document redaction, and regenerate every projection (spec 6) |
| PostgreSQL RLS, BigQuery, Snowflake | A diagnostic is a channel. A copy carries the data and drops the policy | Applied — the closed reason set, and the declaration travels with the artifact (spec 6) |
| Zanzibar, object capabilities, macaroons | They answer whether a principal may act now. We have no principal and no now | Declined, with the reason recorded. Their staleness and vocabulary objections applied (specs 6, 7) |
| Confluence and SharePoint permission models | A second permission model diverges in its derived structure, and does not scale | Confirms one permission system, and it is the platform's |
| Microsoft security servicing criteria, ISO 29147/30111, the CRA | The obligation attaches to the published claim. Publish the non-boundaries too | Applied — the claim and the five non-claims (spec 6) |
| Cross-fork object reference on a large forge | Repository permissions hold for an unforked tip, and are qualified otherwise | Recorded as a qualification on **Q17** |
