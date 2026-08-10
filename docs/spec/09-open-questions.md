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

This question is closed. The [first-run walkthrough](../evaluations/default-taxonomy-first-run.md) wrote the candidate base package out as real YAML, then ran five adopters through their first day against it.

**The decision.** The base package is minimal and derived from the core: two concrete kinds under one abstract kind, four facets, five relations, one anchor, two shelves. Optional content ships as **bundles**, and a bundle is a publisher overlay that only adds. The doctrine starter kit is a named bundle selection over that base, plus the prose that explains the selection. The interview composes a different selection, and it is `headwater infer` with a second evidence source ([spec 7](07-distribution-and-federation.md#the-interview)).

**The three options never competed.** This entry named a minimal core, a batteries-included default, and an interview as alternatives. All three survive as layers, and each one needs the layer below it. The minimal core is the base. Batteries-included is a selection over it. The interview reaches every other selection. What does not survive is the framing that made them compete.

**Nothing new was needed to package this.** [Spec 7](07-distribution-and-federation.md#profiles-are-publisher-overlays) already ruled that a profile is a publisher overlay rather than a mechanism. An optional package is the same mechanism pointed the other way. Spec 2 removed the `profiles` declaration for this reason, and a `packages` declaration would repeat the mistake under a new name.

**The reason for a minimal base is the resolver, not adoption feel.** Add-only overlays over disjoint addresses commute, so the existing confluence check proves that every subset of bundles resolves. Only a minimal base lets every bundle stay add-only. A large base forces bundles and profiles to remove, and `remove` carries dependent-key deletion and the most failure modes. The size of the base is thus a property of the resolver.

**The leaning here failed the core.** It named decisions, standards, and guides as the small core. Those serve `rationale`, `constraint`, and `procedure`, and the [immutable core](02-taxonomy-model.md#the-immutable-core) requires `behavior`. What that list actually describes is the starter kit, which the walkthrough shows is a different artifact.

**What it found on the way.** Six defects, and three of them sit in [spec 2](02-taxonomy-model.md) with one cause between them. The smallest column of the worked example was drawn as an impression and never derived from the core beside it. So it omits the `behavior` purpose that the core requires. Its four default relations all run between decisions, so no default edge attaches the corpus to code. And two of those four have no mechanical creator, which contradicts what spec 2 claims for its own default set. All six are applied, and the [walkthrough](../evaluations/default-taxonomy-first-run.md#consequences-for-the-specification) records where each one landed.

**What stays open.** The bundle set is a guess about how adopters cluster, and no adopter exists yet. It is data in a package, so the first real adopter revises it at the cost of a release. The license half of this question belongs to [Q11](#q11--license-and-distribution-posture), and nothing above depends on the answer.

## Q4 — Relation storage

This question is closed. Front matter is authoritative, and the [evaluation](../evaluations/relation-storage.md) confirmed the leaning. It did not confirm it in the shape that this entry expected, and three of its findings change the specification.

**The question was not the one that this entry asked.** The entry reads as a choice between three files. [Q18](#q18--recording-adjudicated-disagreements) wants an adjudication recorded on the declared edge. [Q20](#q20--where-scent-lives) wants an optional cue on a relation, and says that it blocks here. Both need an edge that carries data of its own, so the prior question is whether a relation instance is a pointer or an object. It is an object.

**The decision.** A relation is declared under a `relations:` block in front matter and nowhere else. An entry is either a target reference or a mapping with `to:` and instance attributes. The scalar is sugar for the mapping, and both produce the same edge. Targets are identifiers, never paths. An edge is identified by the source identifier, the relation name, and the normalized target. List order therefore carries no meaning, and a repeated triple is an error.

**The sidecar loses on ownership rather than on convenience.** DITA relationship tables are this option, shipped for two decades in the same domain. A DITA relationship belongs to the map, so the same topic under a second map has different relationships. A Headwater relation asserts something about two documents, and it holds in every context that contains them. Standoff annotation supplies the second argument. It exists because inline markup cannot express overlapping hierarchies, which is a problem that a relation does not have. Its cost is pointer fragility, which a relation would still pay.

**The annotated prose link is cut, and that is the largest change here.** The old leaning allowed a prose link to carry relation semantics "unless they are annotated". <!-- ste-lint: allow passive # quoting the superseded leaning --> An annotation syntax gives one edge two authoring locations, and three questions then have no good answer. Which location wins when they disagree? Which span does a finding anchor to? What does `--fix` write when it adds a reciprocal? [Spec 1](01-conceptual-model.md#facet) already refused a second edge syntax once, when it removed reference-valued facets, and the reasoning transfers without change.

**The friction that the old leaning worried about becomes a check.** A prose link that resolves to a corpus document with no declared relation raises an advisory finding. The author writes the link once, and the fix writes the declaration. The fix is mechanical only when exactly one enabled relation type permits the pair of kinds at the two ends. Otherwise the finding lists the candidates and carries no patch. The check has no converse, because a succession edge belongs in no paragraph.

**Instance attributes are governed the way facets are.** A relation type declares which attributes its instances may carry, and an undeclared attribute is a finding. An attribute takes a facet's value space and is never a reference. An edge that must point at a node is a request to make the edge a node. [Q18](#q18--recording-adjudicated-disagreements) owns that change if it wants it. Each attribute declares an owning end. A source-owned attribute on a symmetric relation gives one value per direction. An edge-owned attribute with two different values at the two ends is a finding, and no fix resolves it. `created_by` stays on the relation type, because `taxonomy audit` measures the declared intent against a real corpus.

**Two consequences land outside this entry.** The internal model is a property graph. So the RDF projection of [Q6](#q6--where-the-corpus-graph-lives-at-rest) reifies any edge that carries an attribute. Q6 has since closed, and it replaced the round-trip test with a declared loss set and a projection census. The reification is one entry in RDF's loss set. And [Q20](#q20--where-scent-lives) now has a home for its cue, plus an answer to one of its three questions: the referring end owns it.

**What stays open.** One claim here is unmeasured, as [principle 11](00-vision-and-scope.md#design-principles) requires it to be. The promotion fix should raise author-attributable edges without a rise in hand entry, and the assisted fraction plus the audit report are the instruments. The friction signal also survives in a narrower form. If authors still declare the same link twice at a rate the fix does not absorb, this ruling is wrong and the annotation question returns.

## Q5 — Voice checking depth

Lexical pattern matching is cheap, explainable, and imprecise. A small local classifier is more accurate and much less explainable. A finding that an author cannot understand is a finding that they suppress.

**Leaning:** lexical, with a well-curated pattern set, per-category severities, and reasoned escape hatches. Revisit only with measured false-positive data.

## Q6 — Where the corpus graph lives at rest

This question is closed, and the answer is that the graph never rests. One [evaluation](../evaluations/graph-export-and-federation.md) settles it with [Q13](#q13--linkml-and-shacl-as-substrate) and [Q9](#q9--multi-repository-corpora), because the three were one question asked at three radii.

**The question was not the one that this entry asked.** The entry offered four storage options and asked which one wins. But nothing in the design permits a stored graph to be authoritative for anything. [Spec 0](00-vision-and-scope.md#non-negotiables) makes the Markdown the corpus, [principle 2](00-vision-and-scope.md#design-principles) puts truth per fact rather than per store, and [Q17](#q17--governed-access-and-the-solution-layer) refused the inversion on four grounds. So the storage question was answered before this entry was written. What stayed open was a different one. Which artifacts derive from the graph, how long does each one live, and what may each one claim?

**The decision.** Every run rebuilds the graph. Three artifacts derive from it, and none of them is canonical for anything ([spec 6](06-engine-architecture.md#nothing-stores-the-graph)). The in-memory graph lives for one run. The cache lives until its inputs change, and version control ignores it. An export lives until a run regenerates it, and the taxonomy decides whether it is committed by declaring an output path for it.

**The cache is disposable by test.** `headwater check --no-cache` produces output byte-identical to `headwater check`. A cache that can change a verdict is a store under another name, and only the test keeps that distinction true under maintenance.

**No embedded database, and the trigger is named.** The [Q1 spike](../evaluations/language-spike-results.md) ran a warm change-scoped pass over 1,000 documents in about 2 ms, against a 200 ms budget. A named query workload that misses one of spec 6's targets reopens this, and nothing else does. The industry does not agree, and spec 6 now records the disagreement rather than omitting it.

**The largest correction: a round trip is the wrong instrument.** The leaning promised round-trip fidelity tests for an RDF view, and the same paragraph called RDF the lossy direction. Both cannot hold. A projection that is lossy by design cannot round-trip.

What replaces it is the coverage doctrine of [spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for), one layer out. Each emitter declares a **loss set** — the node classes, edge classes, and attributes that its target cannot carry, each with a reason. Each export run emits a **projection census**. Every node and every edge is either present in the output, or accounted for by a declared reason. An uncovered omission fails the run. The native graph export keeps its round trip, because an empty loss set is what a round trip proves.

That answers the trust problem that the [SHACL evaluation](../evaluations/shacl-worked-example.md#problem-one-everything-downstream-trusts-the-projection-and-shacl-does-not-check-it) raised. The projector was the component that everything downstream trusted and nothing could check. Now the engine checks it against the graph, with no help from any consumer.

**Two classes of export, and only one preserves fidelity.** The native export carries the property graph whole, including the edge attributes that [Q4](#q4--relation-storage) introduced. Every interoperability export is lossy by construction. So the Q4 consequence recorded here still holds — an RDF projection reifies an attributed edge, as Wikidata reifies a statement before it attaches a qualifier. It is now one entry in RDF's declared loss set rather than a special case.

**What it found on the way.** Three findings, and two of them are about other documents. [Spec 1](01-conceptual-model.md#the-corpus) said that a repository has exactly one corpus, which named the wrong container and quietly blocked the monorepo half of Q9. The command list in [spec 6](06-engine-architecture.md#cli) never carried `headwater export`, although the pipeline diagram and [spec 2](02-taxonomy-model.md#mapping-between-taxonomies) both used it. And the projector had no check of its own, only a warning that it needed one.

**What stays open.** One claim here is unmeasured, as [principle 11](00-vision-and-scope.md#design-principles) requires it to be. The rebuild-and-cache design is measured at spike scale, on generated documents. A real corpus and a real harvesting tier are the instruments, and neither exists yet.

## Q7 — Scope of the MCP surface

Read-only tools are obviously right. May an agent *write* through the MCP server — for example, create a decision record or update a facet? That is a question of trust and workflow as much as a technical question.

**Leaning:** read-only in the first release. Writes arrive later, behind explicit opt-in, and they produce a change proposal rather than a commit.

## Q8 — Probe cost and cadence

Efficacy probes are the adaptive layer and they cost real money per run. These points stay open: which categories run on which cadence, and what the monthly envelope is. Also open: whether probes run against pull requests or only on a schedule. A last open point: how results are stored so that trends are visible over quarters rather than runs.

**Leaning:** scheduled weekly, pinned model, deterministic rotation, results committed as evidence records inside the corpus so that the trend is itself governed content.

## Q9 — Multi-repository corpora

This question is closed. One [evaluation](../evaluations/graph-export-and-federation.md) settles it with [Q6](#q6--where-the-corpus-graph-lives-at-rest) and [Q13](#q13--linkml-and-shacl-as-substrate). The cross-*taxonomy* half was already answered by declared SKOS mapping relations ([spec 2](02-taxonomy-model.md#mapping-between-taxonomies)). All three of the remaining points now close, and one of them closes by fixing a sentence.

**The monorepo half was answered all along, in the wrong words.** [Spec 1](01-conceptual-model.md#the-corpus) said that a repository has exactly one corpus. The binding constraint is not the repository. A corpus is one taxonomy and one root, and a repository holds one or more of them. A monorepo with several independent documentation sets is not one corpus under strain. It is several corpora that share a working tree, each with its own lock. A path resolves to exactly one of them, under the rule that shelf patterns already obey. Namespaced identifiers stop two corpora in one tree from colliding. Nothing in the engine changes, because the sentence named the wrong container and that was the whole defect.

**The aggregator is in scope, and it is not a component.** It is a solution corpus plus one anchor kind ([spec 7](07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it)). The subsection below already made the solution layer an ordinary corpus. What was missing is how it reaches the corpora underneath, and [spec 2](02-taxonomy-model.md#behavior-at-the-limits) already carries the machinery. An anchor kind is declared, and exactly one resolver owns it. That resolver reads pinned corpus exports, in the way that `code_path`'s resolver reads a source tree.

**There is no merged graph, so the question of where merged graphs live dissolves.** Merging *is* anchor resolution, and anchor resolution leaves nothing behind when a run ends. This entry already concluded that a merged graph is canonical for nothing. The step that it did not take is the obvious one. An artifact that is canonical for nothing, that nobody reviews, and that one rebuild reproduces does not need to exist.

**Query fan-out does not happen. The tier harvests.** Each source corpus carries a pin: an identity, a content hash, and a location. A scheduled job fetches the export out of band and commits it, and the resolver reads the committed copy. Three arguments agree, and the specification already made all three. [Spec 0](00-vision-and-scope.md#non-negotiables) forbids the network at check time. [Spec 6](06-engine-architecture.md#performance-targets) budgets 100 ms for a route query, and a fan-out across estates does not fit. And a fan-out that meets an unreachable source either fails whole or returns less with no notice. That second outcome is the silent pass that [spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for) forbids. So a pinned export that the tier cannot read is a finding that names the pin.

**What the prior art settled.** SPARQL's `SILENT` keyword is the documented form of the failure above. The digital-library field ran the fan-out experiment for two decades, and its aggregators harvest through tiers of intermediaries. GraphQL federation is the counter-example that proves the rule, because runtime fan-out works there on three conditions that Headwater cannot meet ([spec 11 §N.7](11-adjacent-work.md#n7-harvest-beat-fan-out-and-graphql-federation-says-why)).

### The aggregator authors its own facts

The earlier leaning called this "an aggregator that merges exported graphs". That understates it, and the omission matters when anyone tries to build one.

Some facts belong to no repository. Examples: two services share an interface. One system's failure mode is another system's operating assumption. A capability is implemented across four estates and owned by none of them. These are claims about the *space between* repositories. No repository's export can carry them, and an aggregator that only merges can never state one.

So the federation layer is not a merge target at all. It is a **corpus at a higher altitude**: it authors the facts that genuinely live there, and it reads exports for everything else. When that is admitted, the model is unchanged rather than strained. The solution layer is one more corpus that happens to be about other corpora. It has a taxonomy, its documents are Markdown, and its cross-estate edges are declared in front matter like any other.

This also decides where authority sits, in the terms that [principle 2](00-vision-and-scope.md#design-principles) already sets: one source of truth **per fact**, not per store. A document's content is canonical in its own repository. A cross-estate edge is canonical in the solution corpus that declares it. The merged graph is canonical for nothing, and the ruling above removes it entirely. The question "is the graph or the Markdown authoritative?" has no answer because it is the wrong question — nothing is authoritative *as a store*.

Backstage is this shape in production. Its catalog re-derives entities from descriptors that live beside the code, and it generates the `relations` field rather than accepting one. It also admits entities registered as static configuration ([spec 11 §N.3](11-adjacent-work.md#n3-backstage--the-catalog-is-a-read-model-that-authors-its-own-entries)).

### What this fixes for Q17, and what it leaves there

The harvest ruling constrains [Q17](#q17--governed-access-and-the-solution-layer) in three ways, and decides none of it.

The export is the serving artifact, so a filter acts at export and never at graph build. Checks therefore stay privileged and total, which Q17 required and could not point at. The projection census is the mechanism for Q17's tombstone rule. A redaction is a loss with a reason, and the census already reports that shape. And a harvesting tier holds bytes that a publishing corpus gave it. A filter applied when the tier *reads* is a filter applied after the bytes crossed the boundary. Filtering belongs to the publishing corpus's export step.

**What stays open.** Whether a solution corpus vendors each source export or references it. A vendored copy keeps checks offline and grows the repository, and a reference does the reverse. The size of one real harvest is the evidence that closes it, and no such tier exists yet. Also open: whether a harvesting tier owes conformance rules of its own, because `headwater conformance` evaluates one repository. One claim is unmeasured. Harvest should keep a solution-tier route query inside the same 100 ms budget, and route latency at that tier is the instrument.

## Q10 — Naming

This question is closed. The name is **Headwater**. It is no longer a working name, and the design phase does not reopen it.

The casing has two forms, and they do not mix. In prose, the name of the system is *Headwater*, capitalized. As an identifier, it is `headwater` in lower case. It names the CLI verb that people type dozens of times a day, and the package name. It also names the `.headwater/` directory, the `headwater:` annotation prefix, and the `https://headwater.dev/` namespace.

## Q11 — License and distribution posture

The options are open source, source-available, or internal-only. A related question is whether the base package, the bundles, and the doctrine ship under the same terms as the engine. That half of [Q3](#q3--how-much-of-the-default-taxonomy-ships-in-the-box) is now the only part of it left open, and it also affects Q7. Decide it early, because it is easier to open something later than to close it.

## Q12 — Migration path for an existing corpus

An organization that already runs a comparable framework needs an on-ramp. The on-ramp includes a taxonomy inferred from an existing corpus, and a report of what does not fit. It also includes an incremental adoption mode, where checks apply only to newly touched documents. Whether this is a first-release feature or a follow-on determines how much the schema must tolerate a half-conformant corpus. That tolerance is a design constraint, not a feature request.

**Leaning:** `headwater infer` (propose a taxonomy from an existing tree) and a `--since <ref>` mode are first-release. Adoption friction is the thing most likely to kill this, and both of these directly attack it.

[Q3](#q3--how-much-of-the-default-taxonomy-ships-in-the-box) settled the shape of `infer` while it settled the first-run surface. `infer` and the `init` interview emit the same artifact, so they are one command with two evidence sources: the tree, and the answers. An empty repository is then the degenerate case rather than a second code path.

The between-majors half of this question is no longer open. The core-concepts review established that it constrains the validity model itself, not the migration UX. The lock now records a migration state with `migration-pending` findings ([spec 7](07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states)). What remains open here is first contact — a corpus that was never valid, which the migration state (defined against a known-good starting point) does not cover.

## Q13 — LinkML and SHACL as substrate

This question is closed. The ownership half of the leaning survives untouched, and the architecture around it does not. Headwater owns the language, and standard formats come out of it. But LinkML is not the substrate, and it is not the compiler either. It is the last of six sibling emitters, and the one with the weakest case.

One [evaluation](../evaluations/graph-export-and-federation.md) settles this with [Q6](#q6--where-the-corpus-graph-lives-at-rest) and [Q9](#q9--multi-repository-corpora). Two worked examples supply the substance underneath, on [LinkML](../evaluations/linkml-worked-example.md) and on [SHACL](../evaluations/shacl-worked-example.md).

**What the worked examples established, and what does not change.** LinkML already ships three of the twenty research-derived changes: SKOS mapping slots, PROV `slot_uri` alignment, and `recommended` as advisory severity. Its `designates_type` is our heterogeneous-shelf discriminator. The boundary is not structural against governance. LinkML, SHACL and JSON Schema all validate one instance against a shape. Everything else that Headwater does is a property of the whole graph, or of the corpus over time. SHACL reaches the graph layer that LinkML cannot, and every interesting constraint there is embedded SPARQL that nobody reads because a generator wrote it.

Two objections recorded here were withdrawn, and three survive. The error-message objection fell to `sh:message` interpolation, and the SPARQL-engine objection fell to embeddable Rust engines. Line numbers, remediation and fixability still do not survive the RDF round trip, and [spec 4](04-assurance-model.md#findings) needs all three.

**Emitters never chain, and that is now measured rather than feared.** This entry already named the chaining trap, and framed it as a loss of the graph layer. The stronger fact sits inside LinkML's own shape layer. Its SHACL generator does not translate `any_of` or `equals_string_in`, which LinkML itself expresses ([spec 11 §N.5](11-adjacent-work.md#n5-spdx-30-and-what-linkmls-own-generator-drops)). A chained pipeline inherits every loss of every hop and declares none of them. So every emitter reads the resolved lock and the graph directly.

**The staging order, and why LinkML is last.**

| Order | Emitter | Ships when | Consumer |
|---|---|---|---|
| 1 | JSON Schema for front matter | first release | the adopter's own editor, through a language server |
| 2 | Native graph JSON | first release | the solution corpus of [Q9](#q9--multi-repository-corpora), and any local tool |
| 3 | SHACL | a named external consumer asks | a validation stack with no Headwater installation |
| 4 | RDF and SKOS | the same trigger | knowledge-organization tooling |
| 5 | OKF | the same trigger | LeanCTX, and whatever reads its bundles |
| 6 | LinkML | the same trigger | LinkML's own fan-out to OWL, Pydantic and SQL |

Only the first two pay off with no external adopter, and that is the whole reason for their position. The title of this entry is what hid the answer. It named LinkML and SHACL together and put LinkML first, which made a pipeline look natural. Once emitters may not chain, LinkML stops being the route to anything else. It becomes a sibling whose distinctive value is a fan-out that nobody has asked for.

SPDX 3.0 is the observed application of the model-first half, at standards scale. One model yields an OWL ontology with SHACL restrictions, a JSON-LD context, and a JSON Schema, all derived rather than authored in parallel.

**`exportable_as` is a set, with a partition rule and an equivalence bar.** [Spec 12](12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule) owns the rules. A check declares a set of emitter targets, and `none` is the common value. The exported and unexported sets partition the check registry, and the engine generates both. A target may appear only when the emitted constraint catches exactly what the native check catches, which a differential test establishes. The declaration travels with the artifact, because a consumer who copies an export copies its limits too. OGC API and STAC ship that convention already ([spec 11 §N.6](11-adjacent-work.md#n6-ogc-api-and-stac--the-subset-declaration-shipped)).

**The export is for three things, and the list is unchanged.** External validation to a declared depth. LinkML's generator fan-out, of which JSON Schema is the immediately useful part and now arrives without LinkML. And a differential-testing oracle, which is no longer an extra. It is the admission test for any claim of coverage.

**The engine still never runs on SHACL's validation machinery**, and the disqualifications hold for any authoring surface. SHACL defines conformance as "no validation results" and has no notion of completeness. Nothing in SHACL checks that a projection represents the corpus. Source positions and fixability do not survive the round trip. Document-body, corpus-scope and temporal checks never reach the graph at all.

**The OKF half is not a separate question, and this entry misfiled it.** Q13 separated OKF from the substrate question correctly. One is about the TBox and the other about the ABox, and to conflate them imports weight that the smaller decision does not carry. What it failed to notice is that the native graph export is an ABox emitter too, and [Q6](#q6--where-the-corpus-graph-lives-at-rest) owns it. So OKF is a second ABox emitter beside the native one, on identical terms. It reads the graph, declares a loss set, and emits a census.

That placement makes the standing worry concrete rather than hypothetical. OKF's own conformance check is four advisory warnings, so a consumer that validates a bundle verifies almost nothing about it. The census is the answer, because it is the emitter's own account of what it dropped, checked where the emitter runs. OKF carries unrecognized front-matter keys through a parse-emit cycle untouched, so Headwater facets ride along under a `headwater_*` prefix and the loss set stays small.

**Counter-evidence, still standing.** [OpenGEO](11-adjacent-work.md#e-opengeo--same-substrate-opposite-direction) declines RDF, OWL and SHACL for a neighboring problem. [TrustGraph](11-adjacent-work.md#j-trustgraph--the-same-pitch-the-opposite-mechanism) chose the whole standards stack and ships it. The staging order is what respects both. Nothing standards-based is refused, and nothing is built before a consumer exists.

**What stays open.** No named consumer exists for emitters 3 through 6, and that is the trigger rather than an oversight. If none appears, four emitters are never written and nothing upstream changes. One claim is unmeasured, as [principle 11](00-vision-and-scope.md#design-principles) requires. Emitted JSON Schema should lower the rate of invalid front matter that reaches a check. The instrument is the coverage report's finding rate for Shape-origin rules. If that rate does not move, the staging order is wrong and SHACL has as good a claim to the first slot.

## Q14 — Discovery surface

**Blocks:** nothing yet. It becomes urgent when a corpus is consumed by anything that did not clone the repository.

[Spec 7](07-distribution-and-federation.md) covers distribution to repositories that already know about the publisher. Nothing covers an agent or tool that encounters a corpus cold. Open: how it discovers that a corpus exists, what taxonomy governs it, what version, and where to start to read.

Prior art exists to copy rather than reinvent. Examples: a well-known file at a predictable path, link relations from rendered pages, and an MCP server that advertises the corpus as a capability. All three are cheap, and the first two work without any Headwater installation at all.

**Leaning:** a small machine-readable descriptor at a fixed path, plus the MCP surface for agents that can use it. The descriptor holds: taxonomy identity and version, corpus root, entry points, and the graph export location.

**The reason to defer is gone.** This entry deferred until the graph export format was stable, because the descriptor points at it. [Q6](#q6--where-the-corpus-graph-lives-at-rest) fixed that format, and [Q13](#q13--linkml-and-shacl-as-substrate) fixed the emitter set. Two things follow for whoever takes this entry up. A repository holds one or more corpora ([spec 1](01-conceptual-model.md#the-corpus)), so a descriptor at one fixed path must be able to name several roots. And the export carries its own coverage statement ([spec 12](12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule)). The descriptor points at an artifact that already declares its own limits, so it does not repeat them.

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

**Blocks:** the discovery surface ([Q14](#q14--discovery-surface)), which currently assumes a reader entitled to see everything. It becomes urgent the first time that an adopter wants a contractor to read one shelf and not another.

**The graph export format no longer waits on this entry, and it gave this entry three things.** [Q6](#q6--where-the-corpus-graph-lives-at-rest), [Q13](#q13--linkml-and-shacl-as-substrate) and [Q9](#q9--multi-repository-corpora) closed together. The export is the serving artifact, so a filter acts there and never at graph build. That is the seam that "checks are privileged and total" needed. The projection census is the tombstone mechanism, because a redaction is a loss with a reason and the census reports exactly that. And the harvesting tier holds bytes that a publishing corpus gave it. A filter at the reading end arrives after the bytes crossed the boundary. Filtering belongs to the publishing corpus's export step, and that is a constraint on any design that this entry produces.

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

**Leaning:** the solution layer proceeds now as an ordinary corpus. Access control is specified now and built late. The export format that it waited on is now fixed. The remaining reason to wait is the category change stated above, and not a missing dependency. It never arrives as a side effect when federation ships. The four constraints above are the acceptance criteria for the design, not a wish list. A filtered view that does not announce its filtering is not a partial implementation of this — it is a defect.

## Q18 — Recording adjudicated disagreements

**Blocks:** nothing yet. It becomes live the first time that an agent must choose between two current sources that a human already judged.

Spec 2 formerly assigned kinds a scalar authority rank, consumed by an `on_disagreement` rule. The core-concepts review cut it ([spec 2](02-taxonomy-model.md#disagreement-is-adjudicated-not-ranked)), for three reasons. The trigger is a judgment that the ABox boundary says the system cannot make. The rank pre-answers a question that nobody asked. And a global scalar cannot express the scoped precedence that the prose demanded.

What the cut leaves open is the *positive* half. Suppose that a human adjudicated a specific disagreement, or accepted a coherence-sweep finding that did. Where does that judgment live, so that agents and readers inherit it and do not decide it again? The options are: a resolution recorded on the `conflicts_with` edge itself, a correction or succession of the document that lost, or a dedicated scoped-precedence declaration. Be suspicious of the last option — it re-grows the authority rank with more syntax.

This belongs beside [Q15](#q15--a-synthesized-content-tier)'s provenance questions. Both are about a record of who vouched for what. An adjudication without a named adjudicator is a rank with extra steps.

**Leaning:** record adjudication per-conflict as data on the declared edge, with the adjudicator named. No per-kind ranks, and no new declaration until a real corpus shows that the edge form fails.

[Q4](#q4--relation-storage) built the place for it and set one limit on it. An instance attribute takes a facet's value space, so the adjudicator is a scalar and no edge runs from the edge to a person. If that is not enough, the answer is to make the edge a node, and this question owns that change. It does not arrive as an attribute type.

## Q19 — Inbound integration: an external system of record

**Blocks:** nothing in the engine. It becomes live the first time that an adopter authors requirements in an RM tool. [Spec 11 §K](11-adjacent-work.md#k-modern-requirements--the-first-candidate-where-the-arrow-reverses) records that this is already scheduled to happen.

Every integration specified so far points out of the corpus. Spec 11 §K records the first candidate that points in: Modern Requirements. There, requirements live in Azure DevOps as work items, and Headwater documents must trace to them. The model needs nothing new — work-item anchors, `traces_to`, and `created_by: import` are all specified. What is unspecified is the operational half: what an importer is, what it may touch, and what its output is worth.

These constraints are already settled. The fetch runs out-of-band and commits a snapshot, so check time stays offline ([spec 0](00-vision-and-scope.md#non-negotiables)). The resolver binds anchors against the committed snapshot, not against the live service. Upstream drift raises a change proposal, never a mutation ([spec 7](07-distribution-and-federation.md#upstream-awareness)). Requirement content stays canonical upstream ([principle 2](00-vision-and-scope.md#design-principles)).

[Q9](#q9--multi-repository-corpora) has since made that shape a named pattern with three instances. A taxonomy pin, this snapshot pin, and a source-export pin all behave alike. Each one fetches out of band, commits, checks the committed copy, and compares on a schedule ([spec 7](07-distribution-and-federation.md#upstream-awareness)). Spec 2 also now rules that an anchor resolver reads repository content or a committed snapshot, and never a live service. So the second constraint above is no longer a promise particular to this entry.

Open, in order of consequence, least first:

- **Snapshot format and home.** ReqIF (an OMG standard, tool-neutral, verbose) or the native JSON of the API (simpler, vendor-specific)? And does the snapshot live inside the governed repository or beside it? The snapshot is an input to anchor resolution, so its format is a compatibility surface, not an implementation detail.
- **Does imported prose enter the corpus at all?** The minimal integration imports identities and edges only. Documents point at requirement anchors, and a reader follows the pointer into the RM tool. The larger integration materializes requirement text as marked, read-only documents. Then the corpus is self-contained for offline readers and agents. The larger integration is more useful, and it imports a maintenance obligation with the text.
- **What is an imported edge worth?** May a `traces_to` edge that an importer created satisfy a participation expectation, or let a document claim `evidenced`? If yes, a system that nobody here governs discharges obligations in a corpus that claims to be checkable. If no, imports are decoration. The honest middle: imported edges satisfy nothing blocking until the fidelity of the importer has an evidence trail. That is [principle 4](00-vision-and-scope.md#design-principles), applied to a pipeline instead of a rule.
- **The tier question.** Imported text is regenerable against the pinned snapshot, so it fails the [Q15](#q15--a-synthesized-content-tier) definition of synthesized. But its source is ungoverned, so it is not a projection in the spec-6 sense either. Whether that is a fourth tier or a qualifier on `generated` decides what its provenance record carries. The per-fact receipts of TrustGraph (source, timestamp, method) fit as they are. The addition is the snapshot pin.

**Leaning:** reference-first. Anchors and imported edges ship first — they change no content and are cheap to audit. Imported requirement text arrives later, clearly marked and never canonical, under whatever rule Q15 lands on. The snapshot pin goes into its provenance. Imported edges start advisory and walk the same evidence-driven promotion path as every other control. And the importer is an adapter in the sense that [spec 6](06-engine-architecture.md#ci-adapters) already uses: thin, swappable, and with no RM vendor privileged in the core.

## Q20 — Where scent lives

**Unblocked.** [Q4](#q4--relation-storage) closed, and a relation instance is an object with declared attributes. A cue now has somewhere to sit, and one of the three questions below has an answer.

[Spec 5](05-ai-integration.md#scent-is-the-thing-being-engineered) puts the corpus's scent in the `summary` facet of each document, and calls that facet the entire scent surface. Serena's shipped convention ([spec 11 §L.3](11-adjacent-work.md#l3-where-scent-lives--the-first-substantive-disagreement)) states the opposite rule. A memory must not say when to read it, and the memory that refers to it carries that guidance instead.

Foraging theory supports the node for one moment and the edge for the other, because scent is the proximal cue at the point of decision. A routing result has no referring edge, so the cue has to sit on the node. A traversal has one, and there the referring text is proximal while the target's summary is distal.

Headwater serves both moments. Routing answers a task description, and relations answer a reader who already holds a document. So one placement covers half of the surface, and spec 5 currently claims it covers all of it.

Open: whether a relation instance may carry its own cue. If it may, three questions follow, and Q4 answered the first two of them. **Where does the cue live?** It is a declared instance attribute on the relation type, in the `relations:` block of the referring document ([spec 2](02-taxonomy-model.md#instance-attributes-and-which-end-owns-each-one)). **Who writes it?** The owning end is `source`, so the referring document writes it, which is what Serena's convention asks for and what the proximal-cue argument requires. A symmetric relation then gets one cue per direction, correctly. The authoring cost is still real, and it is the reason that the cue stays optional. **How does anything grade it?** That question stands untouched. Spec 5 grades distinctiveness by comparison of siblings in one place, and edge cues have no such place.

**Leaning:** an optional cue on a relation, with the summary still required. Absence falls back to the target's summary, so current behavior stands and no corpus regresses. Grade the cue only where a probe records a failed traversal. Do not make it mandatory. A required prose field on every edge is exactly the bookkeeping burden that [spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) warns kills a corpus.

One consequence arrives from outside. A cue is an instance attribute, so an edge that carries one reifies in any RDF export. It appears in that emitter's declared loss set ([Q6](#q6--where-the-corpus-graph-lives-at-rest)). The native graph export carries a cue with no loss. Nothing here changes, and the cost of the cue is now visible in the place that pays it.
## Q21 — Terminological succession, and validity under merge

**Blocks:** nothing today. It becomes live the first time that two authors change one corpus at the same time. That is the normal case, not an edge case.

Two questions arrive together here and separate cleanly. One is about vocabulary. The other is about when a check result is trustworthy, and it is the larger of the two.

### The observed case

This project produced the case itself, which [principle 8](00-vision-and-scope.md#design-principles) makes the test that matters.

One change removed a framing from the specification. A second change, written from a commit before the first one landed, used that framing in new prose. Git merged the two without a conflict, because they touched different lines. The linter passed, because no rule knew that the framing was retired. A human caught it while reading the diff.

Nothing here is unusual, and that is the point. The judgment "we no longer describe it that way" existed only as prose and a diff. So no mechanism could inherit it.

### Part one: what an author declares to retire a term

[Spec 4](04-assurance-model.md#declaration-moves-the-boundary) already gives the rule that decides this. Do not ask how to detect the reintroduction. Ask what an author could declare that makes detection unnecessary.

| Option | For | Against |
|---|---|---|
| A **retired-term lexicon** in the taxonomy: the term, its replacement, and the reason | Reads like the regime vocabularies that exist already. Generates a document-scoped check, and the replacement makes the fix mechanical | One more vocabulary to maintain. Lexical matching brings the false positives that [Q5](#q5--voice-checking-depth) warns about |
| **Terms become documents**, and succession is an ordinary `supersedes` edge | Reuses the whole machinery. Provenance, adjudication and lineage come free | Promotes every phrase to a document. The glossary is a projection today, and this inverts that |
| **SKOS labels** on concepts: `prefLabel` for the current term, `hiddenLabel` for the retired one | The standard instrument for this exact problem. [Spec 2](02-taxonomy-model.md#mapping-between-taxonomies) uses SKOS already, so the export rides along | A retired *framing* is not always a concept. It covers the vocabulary half and not the rest |

The reason column is not decoration. A retired term with no recorded reason is the rank with extra steps that [Q18](#q18--recording-adjudicated-disagreements) rejects.

This repository built a version of this already, which is evidence about the need rather than about the design. `tools/ste-lint.py` carries a stock-phrase list, a per-line escape hatch with a reason, and a baseline for what predates the rule. That is a hand-rolled retired-term check. It exists because the need was real enough to write one.

### Part two: validity is not preserved under merge

The general statement is larger than vocabulary, and the specification does not make it anywhere:

> Two changes that are each valid against the merge base can produce an invalid corpus.

Git reports nothing, because the conflict is semantic and not textual. The name for this is a **semantic conflict**, and it is not ours to invent. Every incremental checker has it.

It presses harder here than in most systems, for a reason worth stating plainly. Change-scoped evaluation is the performance bet of [spec 6](06-engine-architecture.md#performance-targets) and [spec 12](12-check-layer.md). Semantic conflict is the failure class that change-scoped evaluation is worst at, because neither change looks wrong inside the scope that evaluated it.

Two things in the design help already, and neither was built for this.

**Corpus-scoped checks cannot be narrowed.** [Spec 12](12-check-layer.md#scope--the-declaration-everything-else-rests-on) makes them barriers that re-run on every change. So the moment a retired-term declaration exists, the machinery catches a reintroduction with no new mechanism. The model is unchanged rather than strained.

**The prior version is anchored at the merge base.** Spec 12 states this for transition legality. What it does not state is the general form. A run against a branch tip is *provisional*, and the run that counts is the one against the merge result.

Open: whether the specification says that, and what follows from it. A merge queue answers it completely, by testing the merged result before it lands. That answer is well known, and it belongs to the forge rather than to us. [Spec 6](06-engine-architecture.md#ci-adapters) says that no forge is privileged in the core. So the engine emits what a merge queue consumes, and it does not become one.

**Leaning:** declare it, and say the larger thing out loud.

Retired terms become data, in the lexicon form. Terms do not become documents, because the cost is out of proportion to a phrase. Revisit SKOS labels when the taxonomy export has a consumer. The check ships advisory under [principle 4](00-vision-and-scope.md#design-principles), because it is lexical and Q5's warning applies to it directly.

For the merge half, the specification states that a branch-tip result is provisional and that the merge result is what counts. The engine builds no merge queue. It emits findings that a gate can consume, which is what [spec 6](06-engine-architecture.md#ci-adapters) promises already.
