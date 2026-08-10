# 9 — Open questions

These are decisions that we deliberately deferred. Each one blocks something. Each entry lists the options that are actually on the table, and a current leaning. Thus the design phase argues the options and does not rediscover them.

## Q1 — Implementation language

This question is closed. The language is **Rust**, with a WebAssembly build of the same crate for editor and browser embedding. The full argument is a separate [evaluation](../evaluations/language-choice.md), and the spike that tested it [passed on all four items](../evaluations/language-spike-results.md). Go is no longer the fallback.

Python and TypeScript are out on distribution. Spec 6 requires a single binary and no toolchain per check, and it gives 200 ms to the change-scoped run that a commit hook performs. An interpreter start spends a large part of that budget before any work begins.

Rust and Go were the real question, and the table that this entry used to carry decided it on the wrong axis. **Performance does not separate them.** A thousand Markdown documents is a small corpus, and both languages clear every target in [spec 6](06-engine-architecture.md#performance-targets) comfortably. The performance argument is completely spent on the exclusion of the two interpreted candidates, and to carry it forward is to count it twice.

Three arguments decide it, and each one comes from a constraint that the specification already made.

**Embedding.** [Spec 6](06-engine-architecture.md#library) requires that editor integrations, the MCP server, and CI adapters consume the library in-process and do not start a subprocess. Rust reaches a Node, Python, or JVM host as a shared library that adds no runtime to it, and reaches a browser through WebAssembly. Go reaches those hosts by putting its scheduler and collector inside them. The esbuild project is the proof of this rather than the counter-example. It is the most successful Go tool in the JavaScript ecosystem, and it arrived by the subprocess route that spec 6 declines.

**Scope enforcement.** [Spec 12](12-check-layer.md#scope--the-declaration-everything-else-rests-on) says that a `Document`-scoped check must be unable to read a sibling. It adds that the enforcement is the feature, and that a leak silently corrupts every cache key derived from it. In Rust, a view is a borrow that holds no path back to the graph, so the leak is a compile error. The same signature makes check purity and parallel safety properties of the build. In Go, the same design compiles and the enforcement returns to code review. Spec 12 lists this component beside the census walker as one whose defect produces systematically green results.

**Sum types.** `Scope`, `Severity`, provenance, the taxonomy syntax, and the classification outcome in the census are all closed sets. Exhaustive matching turns a change to one of them into a compiler-generated list of the sites to fix. Spec 12 already expects two such changes. Taxonomy-as-data means that the engine is mostly interpreters over closed sets, so this is a recurring cost rather than a one-time one.

Three arguments that look decisive are not, and are recorded so that nobody re-runs them. The **RDF stack** is thicker in Rust, but [Q13](#q13--linkml-and-shacl-as-substrate) already puts SHACL off the check path, so an external validator may run as a subprocess. **WebAssembly plugin hosting** is available in both. **Determinism** needs the same discipline in both.

The costs are real and stated in the evaluation. The YAML crate ecosystem is in poor repair, which matters less than it looks because spec 12 requires source spans that no convenient deserializer supplies. Iteration speed and the contributor pool are genuine, and the architecture shrinks the surface where anyone writes engine code at all.

**The counter-evidence, kept in view:** [Vale](00-vision-and-scope.md#what-we-do-not-build) is the closest observed analog to this engine, and it is Go. Go builds this command-line tool. What Go builds poorly is the embeddable library underneath it, which is where the whole investment goes.

**The spike changed, and then it ran.** A parse-classify-graph path in two candidates would have measured the one axis on which the languages are equal. The replacement was a risk-retirement spike in Rust alone, in [`spike/`](../../spike/), where any item could reopen this question. All four passed ([results](../evaluations/language-spike-results.md)).

| Item | Result |
|---|---|
| A front-matter parse that reports line and column for every key | 10 assertions pass, front matter and body |
| A scope leak that fails to compile | 6 leaks rejected, 3 positive controls compile |
| One crate built as a native Node addon and as `wasm32` | 600 KiB addon, 113 KiB wasm, identical output |
| A warm change-scoped run over 1,000 documents, under 200 ms | about 2 ms |

The scope-leak item ran first, because it carried the most weight and was the only claim that Rust might not have delivered. It delivered, and it also caught the spike's own author. The first Node wrapper tried to construct a view from outside the crate, and it did not compile.

**One result belongs in [spec 12](12-check-layer.md) rather than here.** Scope is better carried as a *type* than as a value that `scope()` returns. One trait per scope makes the declared scope and the argument type one fact, so they cannot disagree. Spec 12 records this in its own terms.

## Q2 — Schema format

This question is closed. The [cognitive-dimensions walkthrough](../evaluations/schema-format-walkthrough.md) that this entry prescribed has run, over the five authoring scenarios and the seven dimensions that it named.

**The decision.** YAML 1.2 is the concrete syntax. The schema language, the reference sublanguage, the overlay language and the meta-schema are Headwater's. JSON Schema is an emitted export and never the validator.

The question decomposed into three, and only two of them were open. Who owns the schema language? [Q13](#q13--linkml-and-shacl-as-substrate) answered that already: Headwater owns it, and standard formats come out of it. What syntax do authors type, and what checks partial work? Those two are what the walkthrough settled.

The earlier leaning read "YAML plus JSON Schema for the authored surface". That phrasing implies an answer to the first question, and the implied answer is wrong. [Spec 2](02-taxonomy-model.md) already carries references that no JSON Schema keyword resolves: `$vocabularies.lifecycle_state`, `$package.optional.forbids`, and every dotted overlay address. JSON Schema checks that a reference has the shape of one. Only the engine checks that it points at something. The authored surface was thus always YAML plus a Headwater resolver.

This is the same finding that Q13 reached from the other end. Q13 found that LinkML and SHACL validate one instance against a shape. Q2 finds that JSON Schema validates one document against a shape. Two questions, opposite directions, one architecture.

**Why the typed configuration languages lost.** CUE unifies, and unification only narrows. It expresses `add` well, and it expresses neither `override` nor `remove`. The workaround is a default inside a disjunction, which makes the publisher pre-enumerate every value that an adopter might choose. That is premature commitment, placed on the party least able to carry it. Dhall does express override, and an overlay there becomes a function. But spec 2 requires a static confluence check over the paths that each overlay addresses. No reader can take that address set off a function that branches on its input.

**What the walkthrough derived rather than assumed.** A statically checkable confluence property needs the address set of an overlay to be readable off its syntax. That leaves a first-order path-addressed patch language as the only available shape, which is the shape that spec 2 already sketched. The overlay design is now derived, not preferred.

**Loader rulings.** YAML 1.2 core schema, so `no` stays the string `no`. Duplicate keys are an error. Anchors, aliases and merge keys are forbidden in taxonomy sources. The `$`-reference is the sanctioned reuse mechanism, and an alias is a second one that no overlay can address. Scalar types come from the meta-schema and never from the YAML resolver.

**The implementation cost is known, and the spike retired it.** [Q1](#q1--implementation-language) closed on Rust, where the YAML crate ecosystem is in poor repair. That matters less than it looks, because spec 12 requires source spans that no convenient deserializer supplies. The engine writes its own parse either way. The spike built one for front matter, and a taxonomy file is the same problem ([results](../evaluations/language-spike-results.md)).

**The lock.** The lock is generated, so authorability is not one of its criteria. "A stricter representation" thus means a canonical serialization rather than a typed language. The lock is JSON with sorted keys, one spelling per value, every reference resolved, and a hash over the bytes.

**CUE as an optional front-end.** Yes, outside the engine and in one direction. An organization writes CUE, runs its own build, and commits generated Headwater YAML. The engine never reads CUE. This costs nothing, because it follows from the format being ordinary YAML.

The trade that this entry named still holds, and the decision does not soften it. **Overlays deliberately trade viscosity for hidden dependencies.** Customization by overlay makes change cheap, at the cost of a resolved result that nobody authored directly. Thus `explain`, `resolve`, and a readable lock file are not conveniences here. They are the mitigation.

**What it found on the way.** The walkthrough found five defects in [spec 2](02-taxonomy-model.md) that no notation fixes, and all five are now applied. The kind-to-relation permission was declared twice, with nothing to make the two directions agree. `abstract` appeared in the meta-schema with no semantics. Reading precedence had no clause for the governance family. Compatibility had no dimension for the overlay address surface. And a major version migrated documents but never overlays. The [walkthrough](../evaluations/schema-format-walkthrough.md#consequences-for-the-specification) records where each one landed.

One point stays open in this entry. The `$`-reference sublanguage has three uses and no grammar, and it needs one definition before the meta-schema ships.

**A note on the method.** This entry predicted that viscosity and hidden dependencies would decide the question. They did not. All three candidates scored near-equal on both, because most of the viscosity lives in the model rather than in the notation. Premature commitment, abstraction gradient and a new dimension for machine authors decided it instead. The framework earned its place by contradiction of the prediction that chose it.

## Q3 — How much of the default taxonomy ships in the box

A taxonomy that is too opinionated repels adopters that have an existing culture. A taxonomy that is too thin leaves them with a blank schema. The options are: a minimal core plus optional packages, one batteries-included default, or a `headwater init` interview that composes a taxonomy from answers.

**Leaning:** a small core (decisions, standards, guides) plus optional packages (specifications, evidence, operations, compliance), with an interview that composes them. The interview matters more than the packages — the blank-schema problem is a first-run problem.

## Q4 — Relation storage

Relations are declared in front matter today. The alternatives are: relations inline in prose with extractable syntax, or a sidecar edge file per document. Front matter is simple and diffs well. Prose links are where humans actually write references, and they easily become dangling links.

**Leaning:** front matter is authoritative. Prose links are *extracted* and checked for resolvability, but they do not carry relation semantics unless they are annotated. If authors find it tedious to declare the same link twice, revisit this decision — that friction is a real signal.

## Q5 — Voice checking depth

Lexical pattern matching is cheap, explainable, and imprecise. A small local classifier is more accurate and much less explainable. A finding that an author cannot understand is a finding that they suppress.

**Leaning:** lexical, with a well-curated pattern set, per-category severities, and reasoned escape hatches. Revisit only with measured false-positive data.

## Q6 — Where the corpus graph lives at rest

The options are: rebuild from the cache each run (simplest, no sync problem), persist to a committed JSON artifact (reviewable in diffs, enables tooling without the engine), or use an embedded database (fast queries, another thing to keep in step).

A fourth option arrived with Q13: an **RDF projection**, which is the input to SHACL validation. It carries a question that the others do not carry — *fidelity*. When checks read a projection rather than the documents, the projector becomes the most trusted component in the pipeline. Nothing downstream can detect its mistakes. A projector that drops or mistypes a document yields a graph that validates cleanly and does not represent the corpus ([evaluation](../evaluations/shacl-worked-example.md#problem-one-everything-downstream-trusts-the-projection-and-shacl-does-not-check-it)).

**Leaning:** cache by default, with `headwater export`, which produces a committed JSON graph for anyone who wants to build on it. Avoid a database until a query workload justifies it. If RDF is emitted, it is a **derived view and never canonical**, and it ships with round-trip fidelity tests. The Markdown is the corpus. Any projection that disagrees with the Markdown is the projection's bug.

## Q7 — Scope of the MCP surface

Read-only tools are obviously right. May an agent *write* through the MCP server — for example, create a decision record or update a facet? That is a question of trust and workflow as much as a technical question.

**Leaning:** read-only in the first release. Writes arrive later, behind explicit opt-in, and they produce a change proposal rather than a commit.

## Q8 — Probe cost and cadence

Efficacy probes are the adaptive layer and they cost real money per run. These points stay open: which categories run on which cadence, and what the monthly envelope is. Also open: whether probes run against pull requests or only on a schedule. A last open point: how results are stored so that trends are visible over quarters rather than runs.

**Leaning:** scheduled weekly, pinned model, deterministic rotation, results committed as evidence records inside the corpus so that the trend is itself governed content.

## Q9 — Multi-repository corpora

The model assumes one corpus per repository. Monorepos with several independent documentation sets, and organizations that want a single query surface across many repositories, both push against that.

**Leaning:** one corpus per repository stays the model. Cross-repository views are a *federation* concern — an aggregator that merges exported graphs — not a change to the corpus model. Confirm this before the graph export format is frozen, because that format is the aggregator's input.

The cross-*taxonomy* half of this is now answered: declared SKOS mapping relations ([spec 2](02-taxonomy-model.md#mapping-between-taxonomies)), not a merge. What remains open is the aggregator itself. Open points: whether the aggregator is part of this project at all, and where merged graphs live. Also open: how a query fans out across repositories that are not all checked out at once.

### The aggregator authors its own facts

"An aggregator that merges exported graphs" understates it, and the omission matters when anyone tries to build one.

Some facts belong to no repository. Examples: two services share an interface. One system's failure mode is another system's operating assumption. A capability is implemented across four estates and owned by none of them. These are claims about the *space between* repositories. No repository's export can carry them, and an aggregator that only merges can never state one.

So the federation layer is not only a merge target. It is a **corpus at a higher altitude**: it authors the facts that genuinely live there, and it consumes exports for everything else. When that is admitted, the model is unchanged rather than strained. It is still one corpus per repository, and the solution layer is one more corpus that happens to be about other corpora. It has a taxonomy, its documents are Markdown, and its cross-estate edges are declared in front matter like any other.

This also decides where authority sits, in the terms that [principle 2](00-vision-and-scope.md#design-principles) already sets: one source of truth **per fact**, not per store. A document's content is canonical in its own repository. A cross-estate edge is canonical in the solution corpus that declares it. The merged graph is canonical for nothing. The question "is the graph or the Markdown authoritative?" has no answer because it is the wrong question — nothing is authoritative *as a store*.

**Leaning:** the solution layer is in scope and is an ordinary corpus, not a new mechanism. Its access rules, however, are not ordinary — see [Q17](#q17--governed-access-and-the-solution-layer).

## Q10 — Naming

This question is closed. The name is **Headwater**. It is no longer a working name, and the design phase does not reopen it.

The casing has two forms, and they do not mix. In prose, the name of the system is *Headwater*, capitalized. As an identifier, it is `headwater` in lower case. It names the CLI verb that people type dozens of times a day, and the package name. It also names the `.headwater/` directory, the `headwater:` annotation prefix, and the `https://headwater.dev/` namespace.

## Q11 — License and distribution posture

The options are open source, source-available, or internal-only. A related question is whether the default taxonomy package ships under the same terms as the engine. This affects Q3 and Q7. Decide it early, because it is easier to open something later than to close it.

## Q12 — Migration path for an existing corpus

An organization that already runs a comparable framework needs an on-ramp. The on-ramp includes a taxonomy inferred from an existing corpus, and a report of what does not fit. It also includes an incremental adoption mode, where checks apply only to newly touched documents. Whether this is a first-release feature or a follow-on determines how much the schema must tolerate a half-conformant corpus. That tolerance is a design constraint, not a feature request.

**Leaning:** `headwater infer` (propose a taxonomy from an existing tree) and a `--since <ref>` mode are first-release. Adoption friction is the thing most likely to kill this, and both of these directly attack it.

The between-majors half of this question is no longer open. The core-concepts review established that it constrains the validity model itself, not the migration UX. The lock now records a migration state with `migration-pending` findings ([spec 7](07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states)). What remains open here is first contact — a corpus that was never valid, which the migration state (defined against a known-good starting point) does not cover.

## Q13 — LinkML and SHACL as substrate

**Blocks:** Q1. It also blocked [Q2](#q2--schema-format), which is now decided and confirms the leaning below. The choice stays close to irreversible after the schema format ships.

[LinkML](https://linkml.io/) is a YAML-authored schema language that compiles to JSON Schema, SHACL, RDF/OWL, Pydantic, and SQL DDL. It covers a substantial part of what [spec 2](02-taxonomy-model.md) specifies structurally — classes, slots, ranges, cardinality, enums, inheritance — and none of the governance half (regimes, expectations, overlays, core). SHACL, similarly, is the standard to validate a graph against declared shapes, which is what our schema-derived checks do by hand.

| Option | For | Against |
|---|---|---|
| 1. Author in LinkML | Meta-schema, validator, and multi-format output for free. A standard that others already read | Everything Headwater-specific lands in untyped `annotations` that LinkML never validates, so the meta-schema benefit disappears for exactly our half. Two languages in one file. Its Python tooling pulls against a Rust core (Q1) |
| 2. Borrow the design, stay independent | One coherent language. Full control of authoring ergonomics | We rebuild a validator and a compiler that already exist, and forfeit interoperability |
| 3. **Author in Headwater's language, emit LinkML** | One validated authoring surface. LinkML's generators then produce JSON Schema, SHACL, OWL and Pydantic for free. Reversible, since a generator can be changed or dropped. Dissolves the Q1 tension | A generator to build and maintain, plus fidelity tests that prove that the emitted schema accepts exactly what Headwater accepts |

A [worked example](../evaluations/linkml-worked-example.md) decides two things that were guesses when this question was raised.

**LinkML already ships more than expected.** `recommended` is advisory severity. `designates_type` is our heterogeneous-shelf discriminator. SKOS mapping slots and PROV `slot_uri` alignment are native. That is three of the twenty research-derived changes, for free.

**The boundary is not structural-versus-governance.** *Reciprocity* is not expressible in LinkML, and reciprocity is as structural as anything in spec 2. The real line is that LinkML, SHACL and JSON Schema all validate **one instance against a shape**. Everything that Headwater does and that they cannot do is a property of the **whole graph or of the corpus over time**. Examples: reciprocity, satellite inheritance, cross-endpoint conflict rules, and participation expectations.

**SHACL reaches the layer that LinkML cannot.** A second [worked example](../evaluations/shacl-worked-example.md) shows that reciprocity — the constraint that defeated LinkML — is routine in SHACL. So are cross-node conflict detection, satellite inheritance, and even windowed sequence expectations. All of them require a fallback to SPARQL. The reason is that SHACL Core can traverse but cannot refer back to the focus node from the far end of a traversal.

Two corrections apply to what this question originally recorded. The **error-message objection is withdrawn**: `sh:message` with variable interpolation makes messages as good as they are authored. Generated shapes are as good as our generator. The **SPARQL-engine objection is weaker than stated**: embeddable Rust SPARQL engines exist. Thus the cost needs measurement against the change-scoped budget, not an assumption.

The objections that survive are different and sharper: **line numbers, remediation and fixability do not survive the RDF round trip**. Those three are what make a finding actionable under [spec 4](04-assurance-model.md#findings).

**Leaning:** option 3, which the [Q2 walkthrough](../evaluations/schema-format-walkthrough.md) now confirms — and extended: emit **LinkML for the shape layer and SHACL for the graph layer**. Then an external consumer can validate a Headwater corpus to useful depth with no Headwater installation. This choice makes the shape/graph split an explicit architectural seam rather than an accident. Every mature validation stack in this space already has that shape.

The decisive argument is the same for both, and stronger for SHACL: every interesting constraint is embedded SPARQL. Hand-authored SPARQL is *less* readable than an engine predicate. But nobody reads generated SPARQL, so readability is no longer a cost.

These stay Headwater-native either way: schema operations, statistical measures, instrumentation, and the finding shape. The emitted shape set is deliberately a **subset**, and it must declare itself as one. An external validator that reports a clean run, while it believes that it checked everything, is worse than one that knows what it skipped.

Counter-evidence to keep in view: [OpenGEO](11-adjacent-work.md#e-opengeo--same-substrate-opposite-direction) solves a neighboring problem on Markdown and YAML, and it explicitly declines RDF/OWL/SHACL. Thus a standards-based route is not self-evidently correct. [TrustGraph](11-adjacent-work.md#j-trustgraph--the-same-pitch-the-opposite-mechanism) is the same evidence in the other direction: a production platform on a neighboring problem that chose the full standards stack and ships it. Thus the choice is genuinely open, and it does not trend either way.

### What the export is for — and what it is not

A later review of this question settled a confusion that deserves a record: the emitted LinkML and SHACL are an **export product, never the validator**. The engine loses nothing when it does not run on SHACL's validation machinery. SHACL's disqualifications as a check engine hold for any authoring surface:

- **Conformance** — SHACL defines it as "no validation results", with no notion of completeness ([spec 4](04-assurance-model.md)).
- **Projection fidelity** — nothing in SHACL checks that the projection represents the corpus ([Q6](#q6--where-the-corpus-graph-lives-at-rest)).
- **Source positions and fixability** — the RDF round trip loses both.
- **Document-body, corpus-scope, and temporal checks** — these never reach the graph ([spec 12](12-check-layer.md)).

The properties that SHACL has were already disqualifying for the engine. The properties that the engine needs, SHACL never had. Thus native authoring costs nothing that was ever usable.

The export is for three concrete things. **External validation to a declared depth** — a consumer with a standard RDF stack checks a Headwater corpus with no Headwater installation. The `exportable_as` mechanism ([spec 12](12-check-layer.md)) keeps the claimed coverage honest. **LinkML's generator fan-out** — the immediately useful output is JSON Schema, which gives front-matter validation in any editor via yaml-language-server, with no Headwater tooling installed. OWL, Pydantic, and SQL ride along. **A differential-testing oracle** — the fidelity tests listed as the cost of option 3 are also a correctness harness. A stock SHACL validator runs over the RDF projection, and a diff of its findings against the engine's generated checks catches bugs in the generator. This is the one place that uses SHACL's validation machinery — as a cross-check, not an authority.

One trap hides in the phrasing "compile to LinkML and onward to SHACL": the emitters must not be chained. LinkML's own SHACL generator emits only what LinkML expresses — one instance against a shape. Thus a pipeline routed through LinkML produces SHACL that silently covers the shape subset, while every graph-layer constraint (reciprocity, live conflicts, satellite inheritance, windowed expectations) vanishes. That is exactly the "clean run while believing it checked everything" failure that the subset declaration exists to prevent. So the two-emitter split in the leaning above is a requirement, not a preference.

The honest risk: the taxonomy generates the shape checks and the graph checks either way, so the export saves no implementation work. It is additive interoperability at the cost of a generator and its fidelity tests. If no external consumer appears, it is maintenance for nothing beyond the editor story and the test oracle. That suggests a staging order. **JSON Schema emission comes first**, because its payoff needs no external adopter. **SHACL emission comes only when someone asks to validate a corpus externally.** Both stay reversible on the terms that option 3 already claims.

### The OKF question is a different layer

The question above is about the **TBox**: whether an external language expresses our schema and our checks. [OKF](11-adjacent-work.md#i1-okf--the-same-substrate-arrived-at-independently) — LeanCTX's Markdown-plus-front-matter knowledge format — is about the **ABox**: whether the *corpus itself* is exportable to something that another tool already reads. The two share the word "export" and nothing else. To conflate them imports the substrate question's weight onto a decision that does not carry it.

It does not carry it for three reasons. It is **additive**: an emitter that nobody uses costs a generator and a fidelity test, and its deletion later breaks nothing upstream. It is **already most of the way done**: OKF is a directory of Markdown files with YAML front matter, `type` required, and Markdown-link relations. That describes what we already write. And it is **lossless in the direction that matters**: OKF carries unrecognized front-matter keys through a parse-emit cycle untouched. Thus Headwater facets with no OKF meaning ride along under a `headwater_*` prefix and are not dropped.

If it is built, two constraints apply. The emitted bundle is a **projection and never canonical**, on the same terms that Q6 sets for any RDF view. The Markdown corpus is the truth, and a disagreement is the projector's bug.

And OKF's own conformance checking is four advisory warnings. Thus a consumer that validates an exported bundle verifies almost nothing about it. Do not mistake the export for a second opinion on the corpus. That is the same "declare yourself a subset" obligation that the emitted shape set carries above, for the same reason.

**Leaning:** yes, but late — after the graph export format is stable ([Q9](#q9--multi-repository-corpora)) and well after the substrate decision. It is a day of work at the right moment and a distraction at the wrong one.

## Q14 — Discovery surface

**Blocks:** nothing yet. It becomes urgent when a corpus is consumed by anything that did not clone the repository.

[Spec 7](07-distribution-and-federation.md) covers distribution to repositories that already know about the publisher. Nothing covers an agent or tool that encounters a corpus cold. Open: how it discovers that a corpus exists, what taxonomy governs it, what version, and where to start to read.

Prior art exists to copy rather than reinvent. Examples: a well-known file at a predictable path, link relations from rendered pages, and an MCP server that advertises the corpus as a capability. All three are cheap, and the first two work without any Headwater installation at all.

**Leaning:** a small machine-readable descriptor at a fixed path, plus the MCP surface for agents that can use it. The descriptor holds: taxonomy identity and version, corpus root, entry points, and the graph export location. Defer until the graph export format is stable, because the descriptor will point at it.

## Q15 — A synthesized content tier

**Blocks:** the provenance model, if the answer is yes.

Headwater recognizes two kinds of content: **authored** (a human wrote it, and it is canonical) and **generated** (a projection, verified by regeneration and comparison). Karpathy's LLM Wiki pattern ([spec 11](11-adjacent-work.md#f2-karpathys-llm-wiki)) is built on a third: **synthesized** — an agent's interpretation of sources. The interpretation evolves, and the agent revises it as new sources arrive.

It fits neither existing tier, and the difference is not cosmetic. A projection is verifiable by regeneration, but a synthesis is not. Two runs over the same sources produce different prose, both defensible.

The tier is not hypothetical. [TrustGraph](11-adjacent-work.md#j-trustgraph--the-same-pitch-the-opposite-mechanism) runs an entire platform on it. Its per-fact receipts — source document, ingestion timestamp, extraction method — are a working design for the provenance record that this tier needs. What it lacks is exactly what this question adds: a rule that prevents anyone from treating the synthesis as canonical.

[Serena](11-adjacent-work.md#l5-onboarding-ships-the-synthesized-tier-and-marks-nothing) shows the cost of no rule at all, and it is the sharper case of the two. Its onboarding pass writes agent-synthesized knowledge into the same directory, in the same file shape, as memories that a human wrote. Nothing marks the difference, and no field records the source or the date. A reader cannot separate the two tiers by inspection, so the corpus quietly loses the distinction that this question exists to keep. That is the outcome a refusal to model the tier produces, observed in a widely adopted tool rather than predicted.

[Q19](#q19--inbound-integration-an-external-system-of-record) brings the neighboring case that sharpens the definition: imported requirement text. That text *is* regenerable, against a pinned upstream snapshot, so it is not synthesized. The boundary that this question draws is a verification method, not an author. The answer has to place both cases.

Open: whether Headwater admits synthesized content at all. If it does, more questions follow. Does it need its own staleness rules? How does the system make sure that it can never become canonical for anything? Does a human acceptance step promote it to authored, or does it stay permanently second-class?

**Leaning:** admit it, permanently non-canonical, clearly marked, and never a valid target for a `governs` or `verifies` relation. Promotion to authored requires an explicit human acceptance that changes its provenance record. It is how most organizations will actually want to use this, and a refusal to model it just means that it happens unmarked.

## Q16 — Public presence

**Blocks:** nothing technical. It blocks adoption entirely, and later than is comfortable. By the time that it obviously matters, the first impressions are already made.

[Q14](#q14--discovery-surface) covers how a *machine* finds a corpus cold. This is the human half, and it is currently unplanned. There is no site and no sitemap. There is no positioning for someone who heard the name once and has four minutes.

[LeanCTX](11-adjacent-work.md#i5-the-presentation-is-the-lesson) is the standard to match. It is a useful standard precisely because it is not a large company. It is one developer's project, with a site that nonetheless assembles, coherently, what most open specifications never manage:

| Facet | What it answers |
|---|---|
| How it works, architecture | What is this, and what is the shape of it? |
| Benchmarks, metrics | Does it do what it claims, in numbers that someone can re-run? |
| Comparisons | Why this and not the adjacent thing I already know? |
| Use cases | Which of these is *me*? |
| Compatibility, integrations | Will it fit what I already run? |
| Docs, getting started | Can I make it work before I lose interest? |
| Pricing, enterprise, consulting | How does this survive, and what does it cost me? |
| Compliance, audits, self-assessment | What can I show the person who must approve it? |
| Changelog, community, open-source posture | Is it alive, and is anyone else here? |
| `llms.txt`, AI-crawler-friendly `robots.txt` | Can a machine reader find and cite it? |

The last row is where this question touches Q14, and it is the one that an ordinary marketing site omits. The entire thesis of this project is that machines are readers in their own right. For such a project, unreadability to the machines that can recommend it is a self-inflicted wound.

Two constraints are particular to Headwater. The site should be **generated from the corpus that documents Headwater**. Anything else is a governance system whose own public documentation is ungoverned — the first thing that a skeptical reader will check. And the benchmark and self-assessment rows must be **honest before they are impressive**. §I.4 records claims that move between README versions as the thing that made an otherwise strong project harder to trust. A governance tool that inflates its own numbers and is caught has nothing left to sell.

**Leaning:** deferred, deliberately, until there is an engine that makes a site worth a visit. But draft the sitemap early. It is a forcing function for positioning, and every column above is a question that the specification should already answer. Where it cannot, that is a gap in the design rather than in the marketing.

## Q17 — Governed access and the solution layer

**Blocks:** the graph export format ([Q9](#q9--multi-repository-corpora)) and the discovery surface ([Q14](#q14--discovery-surface)), both of which currently assume a reader entitled to see everything. It becomes urgent the first time that an adopter wants a contractor to read one shelf and not another.

Three proposals arrive bundled and separate cleanly. To keep them apart is most of the analysis, because they have very different merits and only one of them is hard.

| Proposal | Verdict |
|---|---|
| A cross-repository **solution layer** | Yes — [Q9](#the-aggregator-authors-its-own-facts)'s aggregator, extended to author its own facts |
| **Access control** over it | Yes — the substantive question, and the one that this entry is about |
| **Graph authoritative, Markdown projected** | No — and unnecessary for either of the above |

### Why authority does not move

The case for inversion is that no one can enforce access control on files that someone already cloned. That is true, and it is the right instinct pointed at the wrong layer.

Inversion costs four things that the design currently gets free.

**Capture cost**: spec 3's survival argument is that authoring is a file edit in the same change as the code. A route through a graph store rebuilds the tool-mediated capture step that killed gIBIS.

**Review**: [spec 4](04-assurance-model.md)'s controls trigger on pull requests, because documents diff there. A graph store does not.

**Detectability**: Q6 already warns that a projector that nothing can check becomes the most trusted component in the pipeline. Today a projector bug is caught by comparison against the Markdown. Inversion removes the comparison target, so the bug corrupts what humans read instead.

**Provenance**: blame, history and signed commits are free from git. An inverted design must rebuild them.

Against that, inversion buys nothing that the serving boundary, below, does not already give.

There is a coherent version of the proposal. Name it, so that no one adopts it by accident. An organization for whom a repository clone is *itself* the leak wants documentation that is never committed to the repository at all. That is a real market. It also abandons "documents are files in the repository, next to the code they describe". That is [spec 0](00-vision-and-scope.md)'s central bet, and the reason that capture is cheap. It is a **pivot, not an extension**.

### Access control is a property of the serving boundary

Enforcement belongs where a reader is *served*, not where an author writes. Per-repository Markdown stays canonical and carries the host platform's repository permissions. The federated graph is a filtered view, and the filtering happens there.

[Serena](11-adjacent-work.md#l6-a-filter-in-the-tool-layer-is-advisory-and-the-documentation-says-so) demonstrates the failure of the other placement. Its `ignored_memory_patterns` hides a memory from every memory tool, and its own documentation then explains how to read the file with a general file tool. The filter sits in the tool surface while the bytes sit in the repository, so it controls one reading path and no other. Serena states the limit plainly for a related feature: the trust gate is "a functionality boundary, not a containment boundary". Any filter that Headwater places short of the serving boundary inherits the same weakness.

This puts the control exactly where the need is and nowhere else. When someone who can clone a repository reads that repository, that is intended behavior. Every case that motivates the question — contractor, partner, adjacent business unit, "show the topology but not the internals" — is cross-corpus, which is the federated layer by definition. Sensitive material lives in a tightly-permissioned repository and is federated in. The graph serves filtered views over the union.

The declaration surface already exists: `confidentiality` is a named facet ([spec 1](01-conceptual-model.md)). This promotes it from descriptive metadata to an enforced security control. That is a small schema change that carries a large change in obligation. A mislabelled facet is no longer a lint — it becomes a leak.

### Four constraints, if it is built

**A filtered view must be legibly filtered.** The doctrine already exists twice — spec 4's [no silent passes](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for) and Q13's declare-yourself-a-subset rule. Redaction obeys it: tombstones, never silent omission, so that a view reports *3 documents withheld* and does not look complete. Without this, an agent traverses a redacted graph, finds nothing, and reports absence with confidence. That is the failure that [spec 5](05-ai-integration.md) names at its start, caused this time by our own security layer.

**Checks are privileged and total. Serving is filtered.** An evaluation of reciprocity on a partial graph invents findings. Validation therefore runs at full visibility regardless of who triggered it, and only the results are filtered on the way out. That seam is clean, and a written statement of it prevents the obvious mistake: checks that run as the user who made the request.

**Topology leaks even when content does not.** Shelf and kind names leak organizational structure, and node counts and edge shapes leak product structure. A hidden document with a kept inbound edge leaks its existence. When both are hidden, the graph's shape changes in ways that a determined reader can difference. This is the multi-level-security inference problem, and it has no clean solution. What the specification owes is honesty that this is mitigation rather than a guarantee.

**Do not invent an identity system.** Derive from the platform's existing identity and team model. Two permission systems that disagree mean that the documentation system is the wrong one, and it is the one that leaks. Spec 7 already has the pattern for rules that it cannot decide from the repository tree. The pattern: degrade to a recorded attestation with an owner and a date.

### The one place "visibility before blocking" cannot apply

[Principle 4](00-vision-and-scope.md#design-principles) says that a new rule ships advisory and earns its way to blocking. Access control is the single mechanism in the system where that is wrong. To ship it advisory is to ship it broken. Every other control may be wrong for a while, precisely because that state is recoverable. A leak is not.

This exception belongs in the specification rather than in someone's judgment. The promotion machinery is otherwise uniform, and it will happily process a permission check like any other.

### The sub-question that arrives silently: what may be a node

Access is the loud half of the solution layer. The quiet half is what the layer may contain. That decision occurs at the moment that someone writes its schema, not when anyone argues about it.

[Spec 11 §A](11-adjacent-work.md#a1-the-solution-layer-presses-on-that-boundary) sets out the choice. The ABox currently stops at the document boundary: the corpus knows that a document exists, its kind, and what it governs — never what it asserts. A solution layer with a `Service` node that describes an actual service crosses that line. It takes on an obligation to stay true to the estate. Nothing in the design currently carries that obligation. Its drift is worse than stale prose, because a wrong node reads as structural rather than editorial.

**Leaning:** declared anchors. A solution-layer node carries an identifier, a name and an owner, and asserts nothing further. Every substantive claim stays inside a document, where freshness and the check layer already reach it. This is what `code_path` already does — an external anchor kind that is referenced and never described — and its generalization costs no new machinery. Revisit only against a concrete need that the anchor form cannot meet.

### What it changes about the project

This is worth a plain statement, because it is a category change rather than a feature. Documentation tooling with no access model is a developer tool. Documentation tooling with one is security software. It acquires a threat model, an audit obligation, a disclosure process, and a class of bug that no one can fix forward. That is a defensible business, and it is the natural shape of an enterprise tier ([Q16](#q16--public-presence)). But it is not a facet that someone adds on a quiet afternoon.

**Leaning:** the solution layer proceeds now as an ordinary corpus. Access control is specified now and built late, after the graph export format is stable. It never arrives as a side effect when federation ships. The four constraints above are the acceptance criteria for the design, not a wish list. A filtered view that does not announce its filtering is not a partial implementation of this — it is a defect.

## Q18 — Recording adjudicated disagreements

**Blocks:** nothing yet. It becomes live the first time that an agent must choose between two current sources that a human already judged.

Spec 2 formerly assigned kinds a scalar authority rank, consumed by an `on_disagreement` rule. The core-concepts review cut it ([spec 2](02-taxonomy-model.md#disagreement-is-adjudicated-not-ranked)), for three reasons. The trigger is a judgment that the ABox boundary says the system cannot make. The rank pre-answers a question that nobody asked. And a global scalar cannot express the scoped precedence that the prose demanded.

What the cut leaves open is the *positive* half. Suppose that a human adjudicated a specific disagreement, or accepted a coherence-sweep finding that did. Where does that judgment live, so that agents and readers inherit it and do not decide it again? The options are: a resolution recorded on the `conflicts_with` edge itself, a correction or succession of the document that lost, or a dedicated scoped-precedence declaration. Be suspicious of the last option — it re-grows the authority rank with more syntax.

This belongs beside [Q15](#q15--a-synthesized-content-tier)'s provenance questions. Both are about a record of who vouched for what. An adjudication without a named adjudicator is a rank with extra steps.

**Leaning:** record adjudication per-conflict as data on the declared edge, with the adjudicator named. No per-kind ranks, and no new declaration until a real corpus shows that the edge form fails.

## Q19 — Inbound integration: an external system of record

**Blocks:** nothing in the engine. It becomes live the first time that an adopter authors requirements in an RM tool. [Spec 11 §K](11-adjacent-work.md#k-modern-requirements--the-first-candidate-where-the-arrow-reverses) records that this is already scheduled to happen.

Every integration specified so far points out of the corpus. Spec 11 §K records the first candidate that points in: Modern Requirements. There, requirements live in Azure DevOps as work items, and Headwater documents must trace to them. The model needs nothing new — work-item anchors, `traces_to`, and `created_by: import` are all specified. What is unspecified is the operational half: what an importer is, what it may touch, and what its output is worth.

These constraints are already settled. The fetch runs out-of-band and commits a snapshot, so check time stays offline ([spec 0](00-vision-and-scope.md#non-negotiables)). The resolver binds anchors against the committed snapshot, not against the live service. Upstream drift raises a change proposal, never a mutation ([spec 7](07-distribution-and-federation.md#upstream-awareness)). Requirement content stays canonical upstream ([principle 2](00-vision-and-scope.md#design-principles)).

Open, in order of consequence, least first:

- **Snapshot format and home.** ReqIF (an OMG standard, tool-neutral, verbose) or the native JSON of the API (simpler, vendor-specific)? And does the snapshot live inside the governed repository or beside it? The snapshot is an input to anchor resolution, so its format is a compatibility surface, not an implementation detail.
- **Does imported prose enter the corpus at all?** The minimal integration imports identities and edges only. Documents point at requirement anchors, and a reader follows the pointer into the RM tool. The larger integration materializes requirement text as marked, read-only documents. Then the corpus is self-contained for offline readers and agents. The larger integration is more useful, and it imports a maintenance obligation with the text.
- **What is an imported edge worth?** May a `traces_to` edge that an importer created satisfy a participation expectation, or let a document claim `evidenced`? If yes, a system that nobody here governs discharges obligations in a corpus that claims to be checkable. If no, imports are decoration. The honest middle: imported edges satisfy nothing blocking until the fidelity of the importer has an evidence trail. That is [principle 4](00-vision-and-scope.md#design-principles), applied to a pipeline instead of a rule.
- **The tier question.** Imported text is regenerable against the pinned snapshot, so it fails the [Q15](#q15--a-synthesized-content-tier) definition of synthesized. But its source is ungoverned, so it is not a projection in the spec-6 sense either. Whether that is a fourth tier or a qualifier on `generated` decides what its provenance record carries. The per-fact receipts of TrustGraph (source, timestamp, method) fit as they are. The addition is the snapshot pin.

**Leaning:** reference-first. Anchors and imported edges ship first — they change no content and are cheap to audit. Imported requirement text arrives later, clearly marked and never canonical, under whatever rule Q15 lands on. The snapshot pin goes into its provenance. Imported edges start advisory and walk the same evidence-driven promotion path as every other control. And the importer is an adapter in the sense that [spec 6](06-engine-architecture.md#ci-adapters) already uses: thin, swappable, and with no RM vendor privileged in the core.

## Q20 — Where scent lives

**Blocks:** relation storage ([Q4](#q4--relation-storage)), because a cue attached to an edge needs somewhere to sit.

[Spec 5](05-ai-integration.md#scent-is-the-thing-being-engineered) puts the corpus's scent in the `summary` facet of each document, and calls that facet the entire scent surface. Serena's shipped convention ([spec 11 §L.3](11-adjacent-work.md#l3-where-scent-lives--the-first-substantive-disagreement)) states the opposite rule. A memory must not say when to read it, and the memory that refers to it carries that guidance instead.

Foraging theory supports the node for one moment and the edge for the other, because scent is the proximal cue at the point of decision. A routing result has no referring edge, so the cue has to sit on the node. A traversal has one, and there the referring text is proximal while the target's summary is distal.

Headwater serves both moments. Routing answers a task description, and relations answer a reader who already holds a document. So one placement covers half of the surface, and spec 5 currently claims it covers all of it.

Open: whether a relation instance may carry its own cue. If it may, three questions follow. Where does the cue live, given that Q4 has not settled how relations are stored? Who writes it, given that every reference then gains an authoring cost? How does anything grade it, given that spec 5 grades distinctiveness by comparison of siblings in one place, and edge cues have no such place?

**Leaning:** an optional cue on a relation, with the summary still required. Absence falls back to the target's summary, so current behavior stands and no corpus regresses. Grade the cue only where a probe records a failed traversal. Do not make it mandatory. A required prose field on every edge is exactly the bookkeeping burden that [spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) warns kills a corpus.
