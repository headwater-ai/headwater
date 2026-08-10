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

**The question was not the one that this entry asked.** The entry reads as a choice between three files. [Q18](#q18--recording-adjudicated-disagreements) wanted an adjudication recorded on the declared edge. [Q20](#q20--where-scent-lives) wants an optional cue on a relation, and says that it blocks here. Both need an edge that carries data of its own, so the prior question is whether a relation instance is a pointer or an object. It is an object.

**The decision.** A relation is declared under a `relations:` block in front matter and nowhere else. An entry is either a target reference or a mapping with `to:` and instance attributes. The scalar is sugar for the mapping, and both produce the same edge. Targets are identifiers, never paths. An edge is identified by the source identifier, the relation name, and the normalized target. List order therefore carries no meaning, and a repeated triple is an error.

**The sidecar loses on ownership rather than on convenience.** DITA relationship tables are this option, shipped for two decades in the same domain. A DITA relationship belongs to the map, so the same topic under a second map has different relationships. A Headwater relation asserts something about two documents, and it holds in every context that contains them. Standoff annotation supplies the second argument. It exists because inline markup cannot express overlapping hierarchies, which is a problem that a relation does not have. Its cost is pointer fragility, which a relation would still pay.

**The annotated prose link is cut, and that is the largest change here.** The old leaning allowed a prose link to carry relation semantics "unless they are annotated". <!-- ste-lint: allow passive # quoting the superseded leaning --> An annotation syntax gives one edge two authoring locations, and three questions then have no good answer. Which location wins when they disagree? Which span does a finding anchor to? What does `--fix` write when it adds a reciprocal? [Spec 1](01-conceptual-model.md#facet) already refused a second edge syntax once, when it removed reference-valued facets, and the reasoning transfers without change.

**The friction that the old leaning worried about becomes a check.** A prose link that resolves to a corpus document with no declared relation raises an advisory finding. The author writes the link once, and the fix writes the declaration. The fix is mechanical only when exactly one enabled relation type permits the pair of kinds at the two ends. Otherwise the finding lists the candidates and carries no patch. The check has no converse, because a succession edge belongs in no paragraph.

**Instance attributes are governed the way facets are.** A relation type declares which attributes its instances may carry, and an undeclared attribute is a finding. An attribute takes a facet's value space and is never a reference. An edge that must point at a node is a request to make the edge a node. [Q18](#q18--recording-adjudicated-disagreements) owned that change and declined it, because an adjudication that needs an author, a date and a reason is a document. So [Q20](#q20--where-scent-lives)'s cue is the one live instance attribute in this specification. Each attribute declares an owning end. A source-owned attribute on a symmetric relation gives one value per direction. An edge-owned attribute with two different values at the two ends is a finding, and no fix resolves it. `created_by` stays on the relation type, because `taxonomy audit` measures the declared intent against a real corpus.

**Two consequences land outside this entry.** The internal model is a property graph. So the RDF projection of [Q6](#q6--where-the-corpus-graph-lives-at-rest) reifies any edge that carries an attribute. Q6 has since closed, and it replaced the round-trip test with a declared loss set and a projection census. The reification is one entry in RDF's loss set. And [Q20](#q20--where-scent-lives) now has a home for its cue, plus an answer to one of its three questions: the referring end owns it.

**What stays open.** One claim here is unmeasured, as [principle 11](00-vision-and-scope.md#design-principles) requires it to be. The promotion fix should raise author-attributable edges without a rise in hand entry, and the assisted fraction plus the audit report are the instruments. The friction signal also survives in a narrower form. If authors still declare the same link twice at a rate the fix does not absorb, this ruling is wrong and the annotation question returns.

## Q5 — Voice checking depth

This question is closed. One [evaluation](../evaluations/what-a-check-can-know.md) settles it with [Q21](#q21--terminological-succession-and-validity-under-merge), because a retired-term rule is a lexical rule and the two entries share every problem. The leaning survives. What decides it is not what this entry expected, and the measurement that shows this was available in the repository all along.

**Lexical, confirmed.** The curated pattern set, the per-category posture and the reasoned escape hatch all stand.

**The measurement, because this project runs a lexical checker on its own specification.** `tools/ste-lint.py` reports 0 errors and 613 warnings over 14 files, 5,432 sentences and about 82,000 words. A hand-adjudicated sample gives three false-positive rates, under the two labels that [spec 4](04-assurance-model.md#suppression) already declares. The passive rule reaches 8% (60 of 449 sampled), the auxiliary rule 56% (25 of 54), and the progressive rule 83% (a census of all 12). The history adds the landing cost of a blocking rule. Of 58 sentence-length errors on the first run, 32 were defects in the sentence splitter rather than long sentences.

**The errors do not come from where the entry assumes.** Three sources, in order of size. Segmentation and span, which decides what is a sentence and which text belongs to the author. Part of speech, which decides whether the matched word is the verb that the rule assumed. And the lexicon, which produced approximately no errors across four rules and one landing.

**So the classifier is refused on aim rather than on accuracy.** It attacks the smallest of the three sources. It does not touch segmentation, and it does not improve a lexicon that is already right. The trigger to reopen is named: a voice category with a mechanical remediation, whose measured errors come mostly from part of speech.

**The largest correction is to the revisit condition.** "Measured false-positive data" names an instrument that cannot answer this question. The passive rule measures 92% precise on its sample, 449 findings stand, and not one of them should block. What decides posture is the [fixability bar](12-check-layer.md#fixability): a category may block only when its remediation is mechanical and total. A category whose remediation is a rewrite is permanently advisory. [Spec 4](04-assurance-model.md#where-promotion-cannot-finish) now states that as a general rule, beside the case where promotion is skipped.

**And the instrument is worse than imprecise. It is empty.** Against 613 advisory findings this corpus holds four escape hatches, so the suppression-derived rate has almost no denominator. That is spec 4's stated blind spot in its extreme form. The adjudicated sample is therefore the only instrument that reaches an advisory rule, and the sample above is one.

**Two obligations land on the parser rather than on the rules** ([spec 3](03-authoring-and-lifecycle.md#what-a-lexical-rule-gets-wrong-and-where-posture-comes-from)). A voice check reads author-owned text, so quotations, code, citations and generated blocks are outside every voice rule by construction. And sentence segmentation joins the [correctness roots](12-check-layer.md#the-correctness-roots), because that is where the errors were.

**The [principle 4](00-vision-and-scope.md#design-principles) exception does not reach here.** A voice finding that fires wrongly is visible and cheap. One that fails to fire costs a sentence that a later reader or a later run still catches. Both error classes recover, so the ordinary promotion path applies and only the fixability bar stops it.

**What stays open.** The measurement above is of three ASD-STE100 structural rules, used as proxies. Nobody has measured `future_intent`, `change_narration` or `phased_rollout`, which are the categories that the declarative regime forbids, because no implementation of them exists. One claim is unmeasured, as [principle 11](00-vision-and-scope.md#design-principles) requires. [Spec 8](08-design-departures.md) calls declarative voice "mechanically detectable at useful precision". The instrument is a run of the three categories over this corpus, with an adjudicated sample of at least 50 findings each.

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

This question is closed. One [evaluation](../evaluations/the-serving-boundary.md) settles it with [Q14](#q14--discovery-surface) and [Q17](#q17--governed-access-and-the-solution-layer). All three are about one boundary, where a corpus meets a reader that it does not control.

**The entry's axis is wrong, and that is why the leaning came out half right.** It asks whether an agent may write, and calls that "a question of trust and workflow". `headwater check --fix` writes files today and nobody calls it a write surface, because the result lands in a diff that a human commits. The axis is **whose review the result passes through**, not whether bytes move.

**The decision.** Three classes of tool, and the boundary between the second and the third is where the whole question lived ([spec 5](05-ai-integration.md#what-the-server-may-do-and-the-axis-that-decides-it)).

| Class | Ships | Why |
|---|---|---|
| Query | first release | It changes nothing |
| Working-tree write (`new`, `fix`) | first release, off by default per server | The human reviews at commit, and the [fixability bar](12-check-layer.md#fixability) forbids a judgment-bearing patch |
| Landed write | never | Acceptance is a human act, and no forge is privileged in the core |

**The specification had already answered the commit half, and the entry did not cite it.** [Spec 3](03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed) requires `accepted_by` and states that acceptance is a human act. [Spec 5](05-ai-integration.md#what-we-do-not-do) forbids an agent-authored document merged without review. A server-side commit produces a document with no `accepted_by`, or an invented one. No judgment about trust is needed to reach that.

**So the third row is a refusal and not a deferral.** "Writes arrive later" describes a thing that never arrives. Headwater emits what a change proposal needs, and an adapter opens the proposal ([spec 7](07-distribution-and-federation.md#upstream-awareness)). That is the boundary that keeps the engine out of the merge-queue business ([Q21](#q21--terminological-succession-and-validity-under-merge)).

**And the second row corrects the leaning the other way.** "Read-only in the first release" would ship the agent surface without the authoring half. [Spec 0](00-vision-and-scope.md#what-we-build) puts that half in the first release for a stated reason. The two working-tree tools are the mechanical, total operations that the fixability bar already admits.

**Three findings from the prior art change how this is stated** ([spec 11 §O](11-adjacent-work.md#o--the-serving-boundary-descriptors-redaction-and-the-write-path)). The protocol's own annotations are hints, and a client must not trust one from an untrusted server. So the enforcement is an unregistered tool and never a flag. The published attacks arrive at discovery time, before any call, so a confirmation prompt at each call is not a safety argument. And an observed exploit used a write tool as an exfiltration channel, which makes this entry an access-control ruling as well as a workflow one.

**What stays open.** What a hosted server is, operationally: who runs it, and how it is deployed. One claim is unmeasured, as [principle 11](00-vision-and-scope.md#design-principles) requires. Working-tree write tools should raise the assisted fraction ([spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric)), and that metric is the instrument.

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

### What this fixed for Q17

The harvest ruling constrained [Q17](#q17--governed-access-and-the-solution-layer) in three ways, and Q17 has since closed on them.

The export is the serving artifact, so a filter acts at export and never at graph build. Checks therefore stay privileged and total, which Q17 required and could not point at. The projection census is the mechanism for Q17's tombstone rule. A redaction is a loss with a reason, and the census already reports that shape. And a harvesting tier holds bytes that a publishing corpus gave it. A filter applied when the tier *reads* is a filter applied after the bytes crossed the boundary. Filtering belongs to the publishing corpus's export step. That third constraint changed Q17's answer rather than confirming it, because Q17 had placed the boundary at the tier.

**What stays open.** Whether a solution corpus vendors each source export or references it. A vendored copy keeps checks offline and grows the repository, and a reference does the reverse. The size of one real harvest is the evidence that closes it, and no such tier exists yet. Also open: whether a harvesting tier owes conformance rules of its own, because `headwater conformance` evaluates one repository. One claim is unmeasured. Harvest should keep a solution-tier route query inside the same 100 ms budget, and route latency at that tier is the instrument.

## Q10 — Naming

This question is closed. The name is **Headwater**. It is no longer a working name, and the design phase does not reopen it.

The casing has two forms, and they do not mix. In prose, the name of the system is *Headwater*, capitalized. As an identifier, it is `headwater` in lower case. It names the CLI verb that people type dozens of times a day, and the package name. It also names the `.headwater/` directory, the `headwater:` annotation prefix, and the `https://headwater.dev/` namespace.

## Q11 — License and distribution posture

The options are open source, source-available, or internal-only. A related question is whether the base package, the bundles, and the doctrine ship under the same terms as the engine. That half of [Q3](#q3--how-much-of-the-default-taxonomy-ships-in-the-box) is now the only part of it left open. Decide it early, because it is easier to open something later than to close it.

The link to [Q7](#q7--scope-of-the-mcp-surface) that this entry used to name is gone. Q7 closed on the ruling that a landed write never ships, and no license term changes that.

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

This question is closed. One [evaluation](../evaluations/the-serving-boundary.md) settles it with [Q17](#q17--governed-access-and-the-solution-layer) and [Q7](#q7--scope-of-the-mcp-surface), because the three describe one boundary from three sides.

**The entry bundled two questions, and only one of them is ours.** It asked how a machine "discovers that a corpus exists, what taxonomy governs it, what version, and where to start to read". **Registration** is how a machine learns of a corpus when it holds no pointer at all. No file inside a corpus answers that, and none ever has. Every convention in the field presumes a client that already resolved a name. Three package ecosystems put a capability document inside an index and discover the index in none of them. The human half of registration is [Q16](#q16--public-presence), and the machine half is the publisher's own channel. **Resolution** is what closes: a machine holds a location and learns what governs it.

**The decision.** The **corpus descriptor** is a projection ([spec 7](07-distribution-and-federation.md#arriving-at-a-corpus-cold)). `generate --check` holds it to regeneration. It names every corpus root in the repository, with the taxonomy identity, the version, the lock hash, the entry points, and each export profile. One descriptor serves several transports. A file at a fixed path *plus* a separate MCP statement is two copies of one fact ([principle 2](00-vision-and-scope.md#design-principles)).

**Its path is the engine's and not the taxonomy's, and the entry did not notice that it had to be.** Every other projection takes its output path from the schema. Apply that rule here and the descriptor is unreachable, because a reader who must read the taxonomy to find it already knows what it says. So the descriptor is engine-defined and non-optional, at `.headwater/corpus.json` relative to the repository root. The [register projection](04-assurance-model.md#every-obligation-has-exactly-one-disposition) already holds that standing for a different reason.

**The canonical location is inside the repository, and that is a ruling.** A reserved path at the root of an origin fixes one service to one site. It also misdescribes a host that serves several publishers, and it needs control of the apex. A corpus meets all three objections, so a pointer reaches the served copy instead.

**Three rules make it usable.** A version carries a stated client behavior, because a version with no rule attached is a string. A success response that does not parse to the declared shape means **absent** rather than malformed. The reason is that a host which answers every path with a default page is the ordinary case. And a filter reaches the descriptor first, which is the real dependency that this entry had on Q17.

**The descriptor is a disclosure, not only a convenience.** It names roots, entry points and profiles, which is organizational structure. The robots convention states the same thing about itself in its own standard. The file grants no authorization, and a path becomes discoverable by being named.

**What the prior art contributes, including against us.** Four conventions that resemble this keep the index bare and put identity on each collection. They do so to avoid a central file that describes roots which somebody else edits. Headwater centralizes anyway, and may, because the descriptor is generated and regeneration catches drift. The sharper warning is `llms.txt`: about 137,000 domains measured, 97% of valid files unread in a month, and no provider obliged to read one. A descriptor is worth what its obliged consumer is worth, and Headwater's first consumer is its own tooling ([spec 11 §O](11-adjacent-work.md#o--the-serving-boundary-descriptors-redaction-and-the-write-path)).

**What stays open.** Registration. One claim is unmeasured, as [principle 11](00-vision-and-scope.md#design-principles) requires. A descriptor should let a cold agent reach a governing document that it otherwise misses, and the Discovery and Navigability probe categories are the instrument.

## Q15 — A synthesized content tier

This question is closed. One [evaluation](../evaluations/warrant-and-adjudication.md) settles it with [Q19](#q19--inbound-integration-an-external-system-of-record) and [Q18](#q18--recording-adjudicated-disagreements). All three ask what a provenance record must carry for content that the corpus did not author, and what such content may then do.

**The entry named the wrong field, and its own text says so without following through.** It says that "the boundary that this question draws is a verification method, not an author". It then argues about the author for the rest of its length. Agency was never the boundary. [Spec 3](03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed) already admits an agent-drafted document in full: `agency: agent`, `drafted_by`, and a human in `accepted_by`. What this entry actually names is content that **nobody accepted**, at a volume where acceptance does not scale.

**The decision.** Every document carries a **warrant**: the mechanism by which the corpus can defend that the document is what it claims to be ([spec 1](01-conceptual-model.md#warrant)). The set is closed and it has four values.

| Warrant | What stands behind the content |
|---|---|
| `accepted` | A named human accepted it |
| `regenerated` | It is a function of inputs inside the repository, and `generate --check` proves it |
| `transcribed` | It is a byte-faithful copy of a pinned external snapshot ([Q19](#q19--inbound-integration-an-external-system-of-record)) |
| `asserted` | Nothing |

**Yes, Headwater admits `asserted` content.** The mark is positive and never an absent field, which is what SPDX settled when it made `NOASSERTION` a value beside `NONE`. Wikipedia is the observed case at the largest available scale. Its 2025 speedy-deletion criterion for machine output fires on the absence of review, and not on the presence of a model ([spec 11 §P](11-adjacent-work.md#p--provenance-endorsement-and-the-record-of-a-judgment)).

**What it may not do is derived, and the leaning's list is replaced.** Two rules cover the cases that "never a valid target for `governs` or `verifies`" enumerated, plus every relation that an adopter adds later.

- **No edge may let unwarranted content govern the reading of warranted content.** [Reading precedence](02-taxonomy-model.md#reading-precedence-is-derived) is already derived from nuclearity, succession and the governance family, so no per-relation list is needed. Anchors carry no reading precedence, so an asserted document may still declare `governs` against a code path. That edge is what gives it write-time impact detection, which is the only thing that will report its decay.
- **An asserted document does not discharge an evidence obligation.** `evidenced` obliges an external, auditable artifact, and an asserted document is neither ([spec 3](03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two)).

**Staleness does not apply, and there is no date to bump.** `last_verified` records that a human confirmed a document. An asserted document has none and cannot acquire one without becoming `accepted`. Its decay is reported as drift from its production date, and the remedy is to regenerate or to delete.

**Promotion is acceptance, and nothing new is built.** A person reads the document, sets the warrant, and names themselves. Bulk stamping produces identical bytes, so no mechanism detects it. `taxonomy audit` reports promotions per change instead, and a change that promotes forty documents is a finding about the review. That posture is advisory permanently, for the reason that the shift ratio is.

**The Q17 constraint changed the answer rather than confirming it.** This entry proposed that an emitter which cannot carry the mark declares that in its loss set. A loss-set entry reaches the consumer who reads it and nobody else. C2PA's own threat model records that ordinary tooling strips a mark that travels beside content. So an emitter that cannot carry the warrant **withholds** the content at the profile's declared tombstone grain ([spec 6](06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped)). That costs nothing today, because the native export carries the warrant with no loss.

**What stays open.** Whether anything is ever promoted. One claim is unmeasured, as [principle 11](00-vision-and-scope.md#design-principles) requires. Asserted content should move to `accepted` rather than accumulate, and the promotion rate against the asserted count is the instrument. If it never moves, the tier is a dumping ground and the honest response is to say so.

## Q16 — Public presence

**Blocks:** nothing technical. It blocks adoption entirely, and later than is comfortable. By the time that it obviously matters, the first impressions are already made.

[Q14](#q14--discovery-surface) has closed, and it left this entry more work rather than less. Q14 answers **resolution**: a machine that holds a location learns what governs it. It refuses **registration**, because no file inside a corpus makes that corpus findable. Registration is therefore wholly this entry's problem, on both the human and the machine side. There is no site and no sitemap. There is no positioning for someone who heard the name once and has four minutes.

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

The last row is where this question touches Q14, and it is the one that an ordinary marketing site omits. The entire thesis of this project is that machines are readers in their own right. For such a project, unreadability to the machines that can recommend it is a self-inflicted wound. One caution came out of Q14 and belongs here. About 137,000 domains publish an `llms.txt`, and 97% of the valid files went unread for a month. A file that nobody promised to read is not a discovery surface, however cheap it is to write.

Two constraints are particular to Headwater. The site should be **generated from the corpus that documents Headwater**. Anything else is a governance system whose own public documentation is ungoverned — the first thing that a skeptical reader will check. And the benchmark and self-assessment rows must be **honest before they are impressive**. §I.4 records claims that move between README versions as the thing that made an otherwise strong project harder to trust. A governance tool that inflates its own numbers and is caught has nothing left to sell.

**Leaning:** deferred, deliberately, until there is an engine that makes a site worth a visit. But draft the sitemap early. It is a forcing function for positioning, and every column above is a question that the specification should already answer. Where it cannot, that is a gap in the design rather than in the marketing.

## Q17 — Governed access and the solution layer

This question is closed. One [evaluation](../evaluations/the-serving-boundary.md) settles it with [Q14](#q14--discovery-surface) and [Q7](#q7--scope-of-the-mcp-surface). The constraint that [Q9](#q9--multi-repository-corpora) handed this entry changed its answer rather than confirming it.

**The three proposals still separate cleanly, and two of the three verdicts stand.**

| Proposal | Verdict |
|---|---|
| A cross-repository **solution layer** | Yes — [Q9](#the-aggregator-authors-its-own-facts)'s aggregator, extended to author its own facts |
| **Access control** over it | Yes, and much smaller than this entry expected |
| **Graph authoritative, Markdown projected** | No — and unnecessary for either of the above |

### Why authority does not move

The case for inversion is that no one can enforce access control on files that someone already cloned. That is true, and it is the right instinct pointed at the wrong layer. Inversion costs four things that the design gets free. The low capture cost of spec 3. Review on a pull request, where documents diff. The detectability of a projector defect against the Markdown. And provenance from git. Against that it buys nothing that the ruling below does not give.

There is a coherent version of the proposal. Name it, so that no one adopts it by accident. An organization for whom a repository clone is *itself* the leak wants documentation that is never committed to the repository at all. That is a real market. It also abandons "documents are files in the repository, next to the code they describe". That is [spec 0](00-vision-and-scope.md)'s central bet, and the reason that capture is cheap. It is a **pivot, not an extension**.

### The boundary is the export step, not the tier

The entry placed enforcement at the federated layer: "the federated graph is a filtered view, and the filtering happens there". That is one tier too far out, on the entry's own argument. A harvesting tier holds pinned, committed exports, so a filter that the tier applies acts on bytes that already crossed the boundary. That is [Serena's failure](11-adjacent-work.md#l6-a-filter-in-the-tool-layer-is-advisory-and-the-documentation-says-so) at one remove, and this entry diagnosed that failure and then reproduced it.

So the serving boundary is the **export step of each publishing corpus** ([spec 6](06-engine-architecture.md#an-export-profile-carries-a-filter)). A corpus decides what leaves it, and what reaches a tier is already what that tier may hold.

### The unit is a destination, and there are no principals

A filter that runs at export runs when nobody is reading. There is no request, no session, and no reader to identify. So a corpus filters for an **audience** and never for a person, and Headwater has no principals.

That is the model with an enforcement story rather than a limitation accepted reluctantly. The bytes of a filtered export live in a repository. The platform's permissions on that repository decide who reads them, exactly as they do for the Markdown. Two permission systems become one, which is what this entry asked for and could not reach while it imagined a filter at request time. The identity branch closes with it. Capability systems and centralized authorization services answer whether a principal may act on an object now, and Headwater never asks that ([spec 11 §O](11-adjacent-work.md#o--the-serving-boundary-descriptors-redaction-and-the-write-path)).

### The mechanism, and what it did not need

An **export profile** names an audience, an emitter target, an output path, a filter over facet values, and a tombstone grain. It is an entry under `projections`, so the declaration count stays at eleven. Six rules make it honest, and three already hold elsewhere. Carried and withheld partition the corpus and the engine generates both. A withholding is a census reason. A document is withheld whole. The filter is default-deny over classes, so a later schema addition does not widen a profile that nobody re-read. Every projection inside a profile regenerates from the filtered graph. And the declaration travels with the artifact, along with the time that the export ran.

**Two of those six came from the prior art rather than from the argument.** The attenuating-credential literature records that a credential which lists what it forbids widens silently the first time its target grows an operation. And one observed case had a correct text redaction defeated by an alphabetized word index that shipped beside it. A shelf index built at full visibility is that failure, in our own artifact set.

**No facet role was needed.** This entry said that `confidentiality` is a named facet and proposed to promote it to an enforced control. [Spec 1](01-conceptual-model.md) lists it as an example of what facets express, and no role carries it. A filter that names its own facet and values inside the profile is [principle 1](00-vision-and-scope.md#design-principles) working, and the closed role registry stays closed. What survives is this entry's best observation, sharpened. A facet that a filter reads is a facet whose every change is a disclosure decision.

### The tension that this entry stated twice and never noticed

The entry requires a view that reports "3 documents withheld" and does not look complete. Four paragraphs later it reports that node counts leak product structure. A count of withheld documents is a node count. The tombstone that the first constraint demands is the leak that the second one reports, and the inference literature says that no design reconciles them.

So the grain is declared per profile. `sealed` gives only the fact of the filter. `counted` puts a placeholder where each withheld node would have sat, carrying the identifier of the rule that withheld it. That shape is not ours. Freedom-of-information law asks for the amount, the position, and the rule, marked at the site of the cut. It omits the marking only where that would harm the interest which the exemption protects. That is the same conditional that the inference literature reached, a century apart. A reason comes from a closed set, because free prose in a tombstone is a second channel.

**What no profile may declare is a view that presents as total.** That invariant holds under both grains because it leaks nothing, and it is what prevents the harm. The harm is an agent that traverses a filtered graph, finds nothing, and reports absence.

### Checks stay privileged and total, and one outcome is new

A check runs inside the publishing repository, over the full graph, on a runner that holds every byte. Filtering is strictly downstream, in a projection, so no configuration exists in which a check sees a partial graph. This entry's worry about "checks that run as the user who made the request" dissolves along with the request.

What is new is a third resolution outcome. An anchor whose target a profile withheld is **withheld**, not unresolved ([spec 2](02-taxonomy-model.md#behavior-at-the-limits)). Without that distinction, every filtered harvest produces a wall of dangling-anchor findings, and operators learn to ignore the class that also carries real defects.

### Why "visibility before blocking" cannot apply, derived

[Principle 4](00-vision-and-scope.md#design-principles) is right to except this, and the exception follows from an error asymmetry rather than from the subject matter. A withholding that fires wrongly is visible, cheap and reversible. A withholding that fails to fire is invisible to both promotion instruments, and it is not reversible at all. So the general rule lands in [spec 4](04-assurance-model.md#where-promotion-does-not-apply). A control walks the promotion path when both error classes are recoverable, and otherwise it ships at its final posture. A withholding rule is the one instance today, it is not suppressible, and it is not waivable.

[Principle 7](00-vision-and-scope.md#design-principles) resolves the same way. Its positional form is shorthand for a rule about cost, so an exporter that cannot evaluate its filter emits nothing and fails the run.

### Topology, stated as mitigation

Shelf and kind names leak organizational structure. Counts and edge shapes leak product structure. A hidden document with a kept inbound edge leaks its existence. None of this is fixable, and the field that spent decades on it under a larger budget reached the same verdict. `sealed` removes the count, and withholding a document's inbound edges removes the dangling reference. Neither is a guarantee. The honest instruction is the one that this entry implied and did not state. A fact whose *existence* is the secret does not belong in a corpus that is exported at all.

**Revocation is late, and the specification says how late.** A harvesting tier reads a pinned export, so a document withheld today stays in the tier's copy until the next harvest. The authorization systems that solve this in the other direction carry a freshness token on every answer for exactly this reason. A pinned export has none, so the export carries its generation time and the harvest cadence is declared. The lag is then a number rather than a surprise.

### What may be a node

**Declared anchors, and there are now two arguments.** The first is this entry's, and it is about truth. A node that asserts a service's properties takes on an obligation to stay true, and nothing in the design carries it. Its drift then reads as structural rather than editorial ([spec 11 §A.1](11-adjacent-work.md#a1-the-solution-layer-presses-on-that-boundary)).

The second is about enforcement, and it did not exist before the ruling above. A filter has nothing to attach to on a node that carries properties. A document has facets that a predicate reads, and an anchor is carried whole or withheld whole. Declared anchors are what make a solution-layer export filterable at all ([spec 7](07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it)). Revisit only against a concrete need that the anchor form cannot meet, argued as the model change that it would be.

### What the project takes on, and what it declines

This entry says that documentation tooling with an access model is security software. Four things follow, by its account: a threat model, an audit obligation, a disclosure process, and a class of bug that nobody can fix forward. Under the destination model the first three shrink and the fourth stays whole.

Headwater authenticates nobody, holds no session, evaluates no policy at request time, issues and revokes no credential, and records no read. Those obligations stay with the platform that already discharges them for the Markdown. [Spec 0](00-vision-and-scope.md#what-we-do-not-build) now says so in the table of what we do not build. What Headwater does is generate an artifact from a declared rule and account for what it left out. That is a redaction tool, and it keeps the worst property of an authorization system. A leak cannot be fixed forward.

So the project takes on three things, and they arrive with the first filtered profile.

| Obligation | What discharges it |
|---|---|
| The exporter emits exactly the declared set | The projection census, plus a differential fixture set per profile ([spec 12](12-check-layer.md#the-correctness-roots)) |
| Somebody outside the project can report a defect | A stated coordinated-disclosure process |
| The control never ships in a state where it may be wrong for a while | The [principle 4](00-vision-and-scope.md#design-principles) exception, recorded in the register |

**The trigger is a published claim, which is more useful than a category.** One vendor's servicing criteria decide whether a report earns a security fix by asking whether it violates a **published** boundary. The same document lists what is deliberately not one. A project does not become security software by writing a filter. It becomes security software by publishing a sentence that says a boundary holds. So [spec 6](06-engine-architecture.md#what-a-filtered-export-claims-and-what-it-does-not) states one claim and five non-claims. The non-claims are the more useful half, because they turn the tombstone channel, the shape leak and the revocation lag into stated limits.

**One hole under the premise, recorded rather than answered.** Enforcement rests on the platform's repository permissions, and commits in a fork network stay reachable across that network by the platform's own account ([spec 11 §O.12](11-adjacent-work.md#o12-the-platform-permission-that-this-design-leans-on-has-a-documented-hole)). The premise holds for the current tip of a repository that was never forked and never changed visibility. It is qualified otherwise, and no alternative placement is better.

**Sub-repository filtering is refused, not deferred.** Within one repository a clone is total, so any filter placed there controls one reading path while the bytes stay readable along another. An adopter who needs a contractor to read one shelf and not another puts the other shelf in a second repository and federates it in. The cost is real and stated. The alternative is a control that we would have to call advisory in the one place where advisory is a defect.

**What stays open.** Whether any corpus ever needs a second profile. The first release ships one, unfiltered, and a real adopter with a real second audience is what builds the rest. Also open: what a hosted server is operationally, and whether a withheld anchor needs a class beside its count. One claim is unmeasured, as [principle 11](00-vision-and-scope.md#design-principles) requires. A `counted` tombstone should stop an agent reporting absence with confidence, and a probe over a withheld answer is the instrument.

This entry's closing sentence survives and belongs in the specification. A filtered view that does not announce its filtering is not a partial implementation of this. It is a defect.

## Q18 — Recording adjudicated disagreements

This question is closed. One [evaluation](../evaluations/warrant-and-adjudication.md) settles it with [Q15](#q15--a-synthesized-content-tier) and [Q19](#q19--inbound-integration-an-external-system-of-record), because all three are about who vouched for what.

**[Q4](#q4--relation-storage) handed this entry one question to decide, and the answer is no.** The edge does not become a node. An adjudication that carries what this entry asks for — a named adjudicator, a date, a scope, and the reason that [Q21](#q21--terminological-succession-and-validity-under-merge) adds — is a document. The corpus has documents, and it gives them shelves, kinds, lifecycles, identifiers, and `accepted_by`. An edge promoted to a node is a second and weaker copy of all of that. No shelf holds it, and no acceptance binds it. [Spec 1](01-conceptual-model.md#facet) removed reference-valued facets on that argument, and Q4 cut the annotated prose link on it.

**The decision, and the edge already exists.** An adjudication is a decision record, and it carries `overrides` against the document whose effect it displaces. [Spec 2](02-taxonomy-model.md#the-decision-relation-vocabulary) has listed `overrides` in the Kruchten set from the start, with the claim "displaces a prior decision's effect and does not retire it". That is an adjudication, defined before this question was asked.

Everything the entry wanted then follows from rulings that exist.

- **The adjudicator is named and resolved.** The adjudicating document carries `accepted_by`, and its [warrant](01-conceptual-model.md#warrant) is `accepted`. No attribute holds an unresolved name.
- **Readers and agents inherit it.** `overrides` is in the succession family, and [reading precedence](02-taxonomy-model.md#reading-precedence-is-derived) says that the successor governs. Routing already offers a successor first ([spec 5](05-ai-integration.md#intent-time-routing)).
- **Scope comes from the endpoints and the prose.** A scoped precedence is one document with one edge, which is the thing that a scalar rank could never express.
- **The adjudication can be wrong, and then a later document supersedes it.** An attribute pair has no lifecycle.

**The three listed options each fail on one sentence.** A resolution on the `conflicts_with` edge cannot name a resolvable adjudicator, which is this entry's own test. A correction or succession of the loser destroys the record, because the loser was not wrong. A scoped-precedence declaration regrows the authority rank, which the entry already suspected.

**One consequence for the base package.** `overrides` declares an inverse with required reciprocity, in the way that `supersedes` does. Without it, a reader who arrives at the losing document learns nothing, and `check --fix` has no back-link to write.

**The prior art argued the other way first** ([spec 11 §P](11-adjacent-work.md#p--provenance-endorsement-and-the-record-of-a-judgment)). Legal citators put a treatment signal on the citing relationship, which is the edge, and they have done so for over a century. Two facts turn it around. The flag is derived from a published opinion, so the opinion is the record and the flag is a projection. And two citators over the same case law disagree at a measured rate. Retraction practice lands where this ruling lands: a separate citable object that points at a work which stays in place.

**What stays open.** Whether an adjudication is ever partial, with one document governing one axis and another governing a second. Today that is two edges, or one document whose prose carries the split. No corpus shows the need. One claim is unmeasured, as [principle 11](00-vision-and-scope.md#design-principles) requires. An agent that meets the losing document first should reach the adjudication, and a probe over a settled pair is the instrument.

## Q19 — Inbound integration: an external system of record

This question is closed. One [evaluation](../evaluations/warrant-and-adjudication.md) settles it with [Q15](#q15--a-synthesized-content-tier) and [Q18](#q18--recording-adjudicated-disagreements). Four of the five open points close, and the first one dissolves.

**The tier question was Q15's question, and it has one answer.** Imported requirement text is not a fourth tier and not a qualifier on `generated`. It is `transcribed`, one of the four [warrant](01-conceptual-model.md#warrant) values, and W3C PROV already names it `prov:Quotation`. Q15 reached for the boundary and did not follow through. It is this: **is the content a function of a pinned input that the repository holds?** If it is, `generate --check` proves the equality on every run. If somebody edits, summarizes or merges it with local prose, no mechanism proves anything and the warrant is `asserted`. Regenerability was never a disqualifier from a tier. It is the tier axis.

**Snapshot format dissolves, and home closes.** [Spec 2](02-taxonomy-model.md#behavior-at-the-limits) already rules that exactly one resolver owns each anchor kind, and that a resolver reads repository content or a committed snapshot. So the shape of a snapshot is a property of the resolver, and the specification privileges neither ReqIF nor a vendor's API export. ReqIF exists so that two requirements tools can exchange a set without either owning the format, which is the adopter's asset rather than the engine's. What the specification does owe is three properties. The snapshot is committed inside the governed repository, because check time is a function of repository content and because a reviewer reads the diff. It carries its fetch time and the upstream identity and revision of every requirement. And it is a pin, so [spec 7](07-distribution-and-federation.md#upstream-awareness)'s one pattern with three instances covers it unchanged.

**Imported prose may enter, as a projection, and reference-first survives with a better reason.** A transcribed document is generated, held to regeneration against the pin, and marked as generated. A hand edit to it is the finding that a hand edit to a shelf index is. Truth stays upstream, which satisfies [principle 2](00-vision-and-scope.md#design-principles) rather than straining it. The leaning said that text "imports a maintenance obligation". The sharper reason is cost. Anchors and edges need only the anchor machinery, which exists. A transcription needs a resolver that reads text, a projection kind that writes it, and a comparison over the result.

**An imported edge is worth what any generated edge is worth, and the leaning was wrong.** It proposed that imported edges "start advisory and walk the same evidence-driven promotion path as every other control". [Principle 4](00-vision-and-scope.md#design-principles) promotes a **rule** against evidence about that rule's false-positive rate. An importer is not a rule. It is a producer of graph facts, and a wrong imported edge produces a correct check result over a wrong graph. No advisory posture ever finds that. The instrument is a fixture set over the importer, which is what [spec 12](12-check-layer.md#the-correctness-roots) already demands of the scaffolder for the identical reason. So an importer joins the correctness roots, an imported edge satisfies a participation expectation, and an imported pointer supports `evidenced`. A work item in a requirements tool is the clearest external auditable artifact that this specification has. `evidenced` never claimed that Headwater governs the artifact.

**Imported text does not leave, and the mechanism landed one group earlier.** [Q17](#q17--governed-access-and-the-solution-layer) made the export filter default-deny over classes, and a transcribed document is a class. So an adopter carries imported prose to an audience only by naming it in a profile, and that is a line that a reviewer reads. What this ruling adds is a statement rather than a mechanism, and it sits with the other non-claims. Headwater checks nothing about a license. A profile that carries transcribed content is the adopter's redistribution decision ([spec 6](06-engine-architecture.md#what-a-filtered-export-claims-and-what-it-does-not)). Debian segregates by archive area for the same reason, and carries the rest by an explicit act.

**Drift is reported on the edge, not on the snapshot.** When a scheduled comparison advances the pin, the requirements whose revision changed are known. Every `traces_to` edge into one of them is a finding until a person re-verifies it. That is the suspect-link mechanism of requirements practice, and it is [spec 4](04-assurance-model.md#absence-is-a-finding-class-of-its-own)'s report-at-the-origin rule. A proposal against the whole snapshot names a file. A finding on an edge names the document whose author can act.

**What stays open.** Whether a transcription projection ships at all, because no adopter has asked for imported text. Also open: the size of a committed snapshot with full requirement text, which is the same question that [Q9](#q9--multi-repository-corpora) holds about a vendored source export. One claim is unmeasured, as [principle 11](00-vision-and-scope.md#design-principles) requires. Imported edges should not decay faster than scaffolded ones, and staleness by `created_by` in `taxonomy audit` is the instrument.

## Q20 — Where scent lives

**Unblocked.** [Q4](#q4--relation-storage) closed, and a relation instance is an object with declared attributes. A cue now has somewhere to sit, and one of the three questions below has an answer.

[Spec 5](05-ai-integration.md#scent-is-the-thing-being-engineered) puts the corpus's scent in the `summary` facet of each document, and calls that facet the entire scent surface. Serena's shipped convention ([spec 11 §L.3](11-adjacent-work.md#l3-where-scent-lives--the-first-substantive-disagreement)) states the opposite rule. A memory must not say when to read it, and the memory that refers to it carries that guidance instead.

Foraging theory supports the node for one moment and the edge for the other, because scent is the proximal cue at the point of decision. A routing result has no referring edge, so the cue has to sit on the node. A traversal has one, and there the referring text is proximal while the target's summary is distal.

Headwater serves both moments. Routing answers a task description, and relations answer a reader who already holds a document. So one placement covers half of the surface, and spec 5 currently claims it covers all of it.

Open: whether a relation instance may carry its own cue. If it may, three questions follow, and Q4 answered the first two of them. **Where does the cue live?** It is a declared instance attribute on the relation type, in the `relations:` block of the referring document ([spec 2](02-taxonomy-model.md#instance-attributes-and-which-end-owns-each-one)). **Who writes it?** The owning end is `source`, so the referring document writes it, which is what Serena's convention asks for and what the proximal-cue argument requires. A symmetric relation then gets one cue per direction, correctly. The authoring cost is still real, and it is the reason that the cue stays optional. **How does anything grade it?** That question stands untouched. Spec 5 grades distinctiveness by comparison of siblings in one place, and edge cues have no such place.

**Leaning:** an optional cue on a relation, with the summary still required. Absence falls back to the target's summary, so current behavior stands and no corpus regresses. Grade the cue only where a probe records a failed traversal. Do not make it mandatory. A required prose field on every edge is exactly the bookkeeping burden that [spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) warns kills a corpus.

One consequence arrives from outside. A cue is an instance attribute, so an edge that carries one reifies in any RDF export. It appears in that emitter's declared loss set ([Q6](#q6--where-the-corpus-graph-lives-at-rest)). The native graph export carries a cue with no loss. Nothing here changes, and the cost of the cue is now visible in the place that pays it.
## Q21 — Terminological succession, and validity under merge

This question is closed. One [evaluation](../evaluations/what-a-check-can-know.md) settles it with [Q5](#q5--voice-checking-depth). Both halves survive in substance, and the argument for each one changes.

### The observed case, with its commits

This project produced the case itself, which [principle 8](00-vision-and-scope.md#design-principles) makes the test that matters.

Commit `684a153` removed a framing from the specification and named the retired phrase, "reference system". Commit `a1aecc1` records the repair. A concurrent branch, written from an earlier commit, used the retired phrase in new prose. Git merged both without a conflict, and the linter passed because no rule knew the phrase was retired. A human found it while reading a diff.

The judgment "we no longer describe it that way" existed only as prose and a diff, so no mechanism could inherit it.

### Part one: the retired-term lexicon, and where it lives

**A retired-term lexicon in the taxonomy, under the language regime** ([spec 2](02-taxonomy-model.md#the-language-regime-carries-the-terms-that-the-corpus-retired)). Terms do not become documents, because the glossary is a projection today and that would invert it. SKOS labels wait for the taxonomy export to have a consumer, which is the trigger that [Q13](#q13--linkml-and-shacl-as-substrate) already set for emitter 4.

**The entry's argument against the lexicon is the one part of it that the measurement contradicts.** This entry said that lexical matching brings the false positives Q5 warns about. Q5 measured them, and they come from segmentation and from part of speech. The lexicon produced approximately none. A retired-term list is a closed, authored, small set of exact strings, which is the highest-precision shape a lexical rule takes. It inherits Q5's two parser obligations and not Q5's warning.

**It belongs to the language regime and never to a voice regime, and the reason is scope.** A voice regime binds per kind, and its `narrative` value exempts a kind entirely. A retired term is retired in a proposal as much as in a specification. The declaration count stays at eleven.

**The replacement decides fixability, and that reverses the leaning on posture.** With a replacement the fix is a substitution, which meets the [fixability bar](12-check-layer.md#fixability), so the check offers a patch and can finish the promotion path. The leaning said it ships advisory "because it is lexical". It ships advisory under the ordinary [principle 4](00-vision-and-scope.md#design-principles) rule, and being lexical is not what holds it there. The observed case is the other shape. "Reference system" had no replacement term, and the repair rewrote the clause.

**What happens when a lexicon lands on a live corpus needs no new mechanism.** A new entry makes checks that passed fail, which breaks the `consequence` [compatibility dimension](02-taxonomy-model.md#versioning-by-measured-compatibility) and forces a major version. A major ships a migration payload, which knows which rules it broke for which documents. Those are `migration-pending` findings at `(document, rule)` grain, with an owner and an expiry ([spec 7](07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states)). `.ste-lint-baseline.json` is a hand-rolled version of that with neither an owner nor an expiry.

**One consequence from the [warrant](01-conceptual-model.md#warrant).** A term is retired under every warrant and the check does not vary. The escape does. An `asserted` document has nobody who accepted anything, so it carries no `accepted_deviation`, and its remedy is the one that [spec 3](03-authoring-and-lifecycle.md#freshness-and-staleness) gives it already.

### Part two: validity is not preserved under merge

The general statement is the larger of the two, and [spec 4](04-assurance-model.md#a-verdict-is-about-one-state-of-the-corpus) now makes it as a property of a verdict.

> Two changes that are each valid against the merge base can produce an invalid corpus, and no run against either branch tip reports it.

**The anomaly has a name outside version control, and the name comes with a solution.** Under snapshot isolation, two transactions that read overlapping data and write disjoint data each preserve an invariant that the pair violates. That is **write skew**, and git permits it for the identical reason: both detect a write-write overlap and neither holds a read set. Serializable snapshot isolation detects it by tracking what each transaction read, and it pays with a false abort ([spec 10 §F.6](10-theoretical-foundations.md#f6-write-skew-names-the-anomaly-and-read-sets-detect-it)).

**We already have the read set, and that is the whole ruling.** [Spec 12](12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) keys every result on a hash of exactly the in-scope inputs. A run therefore computes its read set as a byproduct of scope enforcement. It now reports that set, with the corpus tree and the lock hash. A merge is then an ordinary change. The engine derives the invalidated instances from it as from any diff, and a verdict survives only when nothing it read has moved.

**The correction to this entry's own text.** It said that corpus-scoped barriers are what catch a reintroduced term. They are not. A retired-term check reads one body and the lexicon, so it is `Document`-scoped. What makes a new lexicon entry retroactive is the lock. The lexicon sits in the taxonomy, the lock hash is in every cache key, and a lock change voids every cached result at once.

**Corpus scope is the honest cost of the performance bet.** A corpus-scoped instance reads everything, so any concurrent change voids it and no incremental test rescues one. Their count is now readable as the work that every merge repeats.

**The engine emits and never orders** ([spec 6](06-engine-architecture.md#ci-adapters)). A merge queue answers the question completely and pays with a serialized landing, and that trade belongs to the forge. This is the boundary that [Q7](#q7--scope-of-the-mcp-surface) drew for the write path, met a second time. The invalidation test fails toward re-running, because a false invalidation costs one run and a false survival ships an invalid corpus with a green report.

**How exposed this repository is.** Across the 29 merge commits on `main`, the mainline had moved past the merge base in 16 of them. In 11 of the 29, both sides changed at least one file under `docs/spec/` from that base. The rate does not show that the failure is common. It shows that the window is the normal case, which is what this entry claimed and could not measure.

**What stays open.** Whether any corpus other than this one needs a retired-term lexicon at all. Also open: whether the read set of a real corpus is small enough that publishing it is free. One claim is unmeasured, as [principle 11](00-vision-and-scope.md#design-principles) requires. Publishing the read set should let a gate skip a full re-run on most merges. The instrument is the fraction of merges whose read set the other side never touched.
