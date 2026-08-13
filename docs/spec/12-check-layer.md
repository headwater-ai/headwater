---
id: SPEC-HW-check-layer
status: current
status_since: 2026-08-02
last_verified: 2026-08-12
summary: What a check is, how a scope constrains it, where it comes from, and what the check layer owes the engine.
doc_type: design_spec
sequence: 12
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: [claude-fable-5, claude-opus-5]
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cites_evidence:
    - EVAL-HW-graph-export-and-federation
    - EVAL-HW-language-spike-results
    - EVAL-HW-relation-storage
    - EVAL-HW-shacl-worked-example
    - EVAL-HW-the-measurement-layer
    - EVAL-HW-the-serving-boundary
    - EVAL-HW-warrant-and-adjudication
    - EVAL-HW-what-a-check-can-know
---

# 12 — The check layer

[Spec 6](06-engine-architecture.md) says that checks are pure functions over the corpus graph. That was sufficient at that level of detail. The LinkML and SHACL evaluations then found where the standards stop. Everything past that line lands here. Thus the check layer needs a design, not only a description.

The signatures below are pseudocode, given as examples. Nothing here depends on the implementation language, which is Rust ([Q1](09-decisions.md#q1--implementation-language)). Two requirements in this document are what decided that question.

## What a check is

```
check(view: ScopedView, ctx: Context) -> [Finding]
```

**Pure.** No file I/O, no network, no clock, no mutation of the graph. Everything that the check can read arrives through the view. Everything time-dependent arrives through `ctx` as an injected value. That purity is not for its own sake. It makes results cacheable, reproducible, and safe to run in parallel. It is also what the determinism requirement in spec 6 reduces to.

## The five origins of a check

Spec 6 sketched three tiers. The correct decomposition is five. It comes from the boundary that the two standards evaluations found, not from guesswork.

| Origin | Comes from | Examples | Exportable as |
|---|---|---|---|
| **Shape** | the TBox, generated | required facet, enum membership, identifier pattern, cardinality, unknown-facet detection | LinkML + SHACL |
| **Graph** | relation declarations, generated | reciprocity, endpoint kinds, lifecycle-sensitivity, satellite inheritance, live conflicts, windowed participation expectations | SHACL (via SPARQL) |
| **Corpus** | declarations that need many documents | facet orthogonality, continuity distribution, scent distinctiveness | — |
| **Document** | regimes, applied to the body | voice, section contract, normative language, size budgets, prose-link resolution | — |
| **Plugin** | adopter code | anything organization-specific | — |

This table settles three things.

**Shape and Graph checks are generated, not written.** A new facet or relation in the taxonomy produces its checks with no code. That is the full point of taxonomy-as-data. It is also where most of the check count lives.

**Document checks are the ones that no graph standard can reach**, because the body is not in the graph. That is the finding from the [SHACL instance-data evaluation](../evaluations/shacl-worked-example.md#does-this-help-with-the-actual-documents). Not by coincidence, they are also the checks that need source positions.

One example in that row asks for more than the row supplies. Prose-link resolution reads the destination of a link, and a fragment on that destination names a heading of another document. No scope below carries a second document's body. So the half that a document decides alone is a Document check. The other half waits for a grain that this list does not hold ([13 — Open obligations](13-open-obligations.md#design-work-that-nothing-blocks)).

**`exportable_as` is machine-checkable.** The emitted shapes are generated from exactly the checks that declare a target, and the next section states the rules that keep the claim honest.

### `exportable_as` is a set with a partition rule

A check declares the emitter targets that it exports to. The value is a **set** rather than one format, and `none` is legal and common. A check exportable to SHACL need not be exportable to JSON Schema.

Three rules make the declared subset of [Q13](09-decisions.md#q13--linkml-and-shacl-as-substrate) true rather than intended.

**The two lists partition the registry.** The engine generates the exported set and the unexported set from one check registry. Neither list is authored, so neither can drift from the other, and no check falls into both or into neither.

**A target needs equivalence, not resemblance.** A check may name a target only when the emitted constraint catches exactly what the native check catches, over the exported graph. A differential test establishes that. A stock validator runs over the projection, and its finding set for that check matches the engine's over the fixture corpus. A partial translation is declared unexported. A claim of coverage that is partly true is worse than a claim of none, because a consumer cannot see the difference.

**A construct that entails rather than checks is out, and a loss set is the wrong place for it.** A target language sometimes holds a construct that looks like a Headwater constraint and acts as an inference rule. `owl:minCardinality` reads as a requirement, and a reasoner meets it with a value that it invents rather than a finding. `rdfs:range` reads as an endpoint permission, and it retypes whatever stands in that position. Each one fails the equivalence bar above, so no conformant emitter carries either. The general rule is what that bar leaves unsaid. **An emitter omits a construct whose meaning inverts, and a loss set never declares one.** A loss set records what an export dropped, and a reader of one expects less coverage rather than a wrong answer ([evaluation](../evaluations/owl-skos-worked-example.md)).

**The declaration travels with the artifact.** An export carries its own coverage statement, so a consumer who copies the export copies the statement too. A statement that lives beside the artifact arrives separately, or not at all.

That last rule follows practice rather than invention. An OGC API implementation serves the conformance classes that it supports, and a listed class obliges the whole capability behind it. Headwater owes one thing more, because its check registry comes from an adopter's taxonomy rather than from a published universe. An outside reader cannot compute the complement, so the export states both halves ([evaluation](../evaluations/graph-export-and-federation.md)).

The differential is a test rather than a rule, and it lives at `engine/crates/generate/tests/differential.rs`. It runs a stock JSON Schema validator over the emitted schema. It compares the result with the finding set of this engine, document for document. The corpus that it reads breaks each claimed family on purpose. Two empty sets agree over every emitter, so a differential over a corpus that violates nothing establishes nothing.

That test took two constructs out of the JSON Schema emitter. Each one arrived with an inverted meaning, which the third rule above refuses. A kind that forbids a facet became `not`, and no check reports a document that states a forbidden facet. A facet with a declared value set became a bare `enum`, and the check declines a value that it cannot read as a scalar. Each construct rejected a document that this engine accepts. The first one is gone and the loss set records the drop. The second one now carries a guard that admits what the check admits.

## Scope — the declaration everything else rests on

Every check declares what it needs to see.

```
Scope =
  | Document                 // one document: front matter and body
  | Edge                     // one relation instance and both endpoints
  | Neighbourhood(depth)     // a node and its n-hop neighbours
  | Shelf                    // every document on one shelf — sibling comparison
  | Corpus                   // everything

  + needs_body:  bool
  + needs_clock: bool
  + needs_prior: bool        // change-scoped only: the prior committed version
```

Scope gives four things. The fourth makes the other three trustworthy.

**1. Change-scoped evaluation.** From a diff, the engine computes exactly which check instances are invalidated. `Document` checks re-run for changed files. `Edge` checks re-run for every edge *incident to* a changed file — in both directions.

That last clause is the subgraph problem that the SHACL evaluation surfaced. The scope declaration is what solves it. An edit to document A that adds `supersedes B` invalidates the reciprocity instance on that edge. This holds whichever endpoint the check reports against, because the edge is the unit, not the file. No guesswork from the diff is necessary.

The unit needs an identity, and [Q4](09-decisions.md#q4--relation-storage) supplies it. An edge is the source identifier, the relation name, and the normalized target. Position in a list is not part of it, so a reordered front-matter block invalidates nothing. A moved file invalidates nothing either, because a target is an identifier and never a path. An edge that carries instance attributes hashes them into its cache key. An attribute that only one end declares still belongs to the edge rather than to that end.

**2. Sound cache keys.** A result is keyed on a hash of exactly the inputs in scope, plus the taxonomy lock hash, the check's version, and any injected values. Nothing outside the scope can affect the result, so nothing outside it needs to be in the key.

**3. Parallelism, with visible serialization points.** `Document` and `Edge` checks are embarrassingly parallel. `Corpus` checks are the barriers. Because scope is declared, their count is a number that you can read, not a property that you discover under load.

**4. Enforcement, which is what makes the rest honest.** The view exposes *only* what the scope declared. A `Document`-scoped check physically cannot read a sibling. So a scope declaration cannot quietly rot into a lie. A scope that is declared but not enforced would silently corrupt every cache key derived from it. The enforcement is the feature, and the declaration alone would be a comment.

### The declaration is a type, not a returned value

The [Q1 spike](../evaluations/language-spike-results.md) tested point 4 and changed it. The `scope()` signature above is the wrong shape, for a reason that applies to any implementation language.

A scope that a check *returns* is a second fact beside the argument it receives. Nothing connects them, so a check can declare `Document` and still be handed a view that reads the corpus. The declaration is then a comment again, which is the failure that point 4 exists to prevent.

One trait per scope removes the second fact. A check that wants to compare siblings must implement the corpus-scoped trait, which is the only way to receive a corpus view. That trait also registers the check where the runner already keys the cache on the whole corpus. The declared scope and the argument type are one fact. `Scope` stays as a value for reporting and for cache keys, derived from the trait and never supplied by the implementer.

The same reasoning binds the plugin interface below. `scope()` there is a method a third party implements, so it is a claim the host would have to trust. As a type it is a claim the host cannot be given.

### The read set, and what a merge does to a verdict

Point 2 above gives a cache key. The same set has a second use, and it is the one that [spec 4](04-assurance-model.md#a-verdict-is-about-one-state-of-the-corpus) needs to make a verdict honest under merge.

The set is the view's and never the check's. The runner records what it handed an instance. So a check cannot report that it read less than it received, and the section above is what makes the record true. The **read set** of a run is the union of the in-scope inputs that produced its results. That is the content hash of every document and edge that an instance read. It also holds the taxonomy lock hash, the check versions, and the injected values. Every run already computes it, one instance at a time. The key is a hash of exactly those inputs, and a key that omits one is a correctness bug. What is new is that the run reports the union beside its coverage numbers.

**A merge is then an ordinary change.** Given the merge result and the tree that a run evaluated, the engine derives the invalidated instances the way it derives them from any diff. An empty result means that the verdict still applies. A non-empty result voids it, and the recomputation is change-scoped over the union of the two changes rather than a full pass.

This is where the design gets something free that a database has to add. A serializable database tracks read sets at run time to detect write skew. Git detects nothing of the kind, because it holds no read set at all. Scope enforcement built ours for a different purpose ([spec 10 §F.6](10-theoretical-foundations.md#f6-write-skew-names-the-anomaly-and-read-sets-detect-it)).

**Corpus-scoped checks are the barriers, and this gives their count a second meaning.** A corpus-scoped instance reads everything, so any concurrent change voids it and no incremental test rescues one. Their count is the work that every merge repeats, and it is readable from the declarations rather than discovered under load.

**The invalidation test fails toward re-running.** A false invalidation costs one run. A false survival ships an invalid corpus with a green report. That asymmetry decides every doubtful case, and it is the same rule that [principle 7](00-vision-and-scope.md#design-principles) gives an exporter.

## Temporal inputs: the clock and the prior version

Two inputs are about time. Both are injected, never fetched.

**The clock** (`needs_clock`) is a bound value. Windowed participation expectations read a declared origin date from the document ([spec 2](02-taxonomy-model.md#participation-expectations)) and compare it against `ctx.now`. There is no history walk and no search of the repository history.

**The prior version** (`needs_prior`) is the input that transition legality requires and that the earlier draft silently lacked. An illegal lifecycle transition is invisible in the current graph, because one `status` value cannot say how it was reached. A check that declares `needs_prior` receives the previously committed version of the changed document. The content hash of that version joins the cache key like any other input. The prior version is available **only in change-scoped evaluation**, because the diff is what supplies it. In a full-corpus run, instances of such a check are counted and reported as skipped, with the reason `change-scoped-only` — visible, never silent ([spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)).

**"Prior" is anchored, not assumed.** The prior version is the document as it stands on the branch where the change lands. That is the merge-base version for a proposed change, and the committed `HEAD` version for a working-tree hook. It is never an intermediate commit inside the incoming branch. A branch lands on the mainline as one state movement, whatever route it took internally, so intermediate flips are invisible by construction, not by accident. Two consequences are stated here so that nobody must discover them:

- **Legality is path-reachability, not edge membership.** A compound movement (`draft` at the merge-base, `superseded` in the result, via `current` inside the branch) is legal if and only if a path between the two states exists in the declared machine. A check of single-edge membership against the merge-base would reject movements that the machine permits.
- **Hook and CI can disagree only when the mainline moved** between the run of the hook and the merge. That is the ordinary race that every merge check has. The evaluation that counts is the one against the final merge-base, and that evaluation is deterministic: same merge-base, same incoming tree, same verdict.

That is a deliberately reduced guarantee, stated rather than implied. Transitions are verified when they land, and they are not re-derived from history later. Git history is not a check input. Vendoring and squash merges destroy it, and a guarantee that depends on a search of repository history is not a guarantee.

## Instances, and why coverage needs them

A check is a template. The engine instantiates it for each target. For example, `facet_required` is not one check but 412 instances. Findings, cache entries, and timings all attach to instances.

Coverage accounting ([spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)) then comes directly from this, with no added mechanism. Each run records, per document, which instances were created, which ran, which were served from cache, and which were skipped with a reason. **A document with zero instances is a finding.** It means that a shelf pattern is wrong or that a file is misplaced. Both facts are good to know.

## Two phases, and why the order matters

**Phase A — classify and build.** The engine parses every file, resolves its kind, builds edges, and indexes identifiers. Failures here are structural findings: an unparseable file, an unclassifiable path, a dangling edge, an ambiguous shelf match.

**Phase B — check.** All the checks above run over the graph that Phase A produced.

Phase A emits a **census**: every file under the corpus root and what became of it. The coverage report in Phase B is computed against that census, not against the set of documents that classified successfully.

That order is the direct answer to the silent-pass failure mode. The denominator is fixed before any checks start. Thus a document that fails to parse is counted, reported, and visibly unchecked. It does not drop out of the run and leave a clean result behind it.

## Findings

The shape of a finding comes from [spec 4](04-assurance-model.md#findings). The check layer carries two obligations to satisfy it:

- **Source positions.** Findings anchor to a line, which means that the parser retains spans for front-matter keys, headings, and links. This is a parser requirement that the check layer *drives*. It is also the concrete reason that an RDF projection cannot be the internal representation: spans do not survive the round trip.
- **Remediation and fixability.** Every finding states what to do. Checks that can fix mechanically say so.

## Fixability

A check may return a patch alongside a finding. The rule for whether it may return one:

> A fix is offered only when it is **mechanical and total** — one correct outcome, derivable without judgment.

To regenerate a stale projection, to add a missing reciprocal link, to normalize front-matter key order, to correct the format of an identifier: these are mechanical. To rewrite a section to satisfy a contract, to choose a summary, to resolve a conflict between two live decisions: these are not. Those carry remediation prose instead. A plausible automatic fix for them would be worse than none, because it would be applied unread.

## Severity is the check's; posture is the control's

A check reports severity. Whether that severity blocks is the **control's** business ([spec 4](04-assurance-model.md)), not the check's.

This separation means that the same check serves an advisory deployment and a blocking one unchanged. Promotion of a check from advisory to blocking is then a configuration change with an audit trail, not a code change. A check that knew whether it blocks would need an edit before each promotion, and the promotion criteria would lose their force.

## Suppression is the runner's

Checks know nothing about suppressions. The runner filters findings, records exactly what it filtered, and feeds the suppression inventory into the coverage report.

A check that handled its own suppressions could hide them. A suppression that nobody can count is indistinguishable from a rule that never fires.

## Determinism, concretely

- **The clock is injected.** `ctx.now` is a bound value, never a syscall. The SHACL sequence-expectation example forced that finding, and it applies to our own engine identically.
- **Stable ordering.** Findings sort by (path, line, check id, message). No iteration over an unordered map reaches output unsorted.
- **Complete cache keys.** The content hashes of the in-scope inputs, the taxonomy lock hash, the check version, and the injected values. A key that omits an input is a correctness bug, not a performance bug.

Same corpus, same lock, same injected clock, byte-identical output. That is what makes `generate --check` and projection freshness meaningful at all.

### A cache hit is a fact about a disk

Two rules above disagree, and this section states which one wins. Coverage records which instances a cache served. Determinism fixes the output at "same corpus, same lock, same injected clock, byte-identical output". A hit count in the report satisfies the first rule and breaks the second. The count follows from what one machine holds on disk, so two people with one tree would read two reports.

The run therefore reports the cache accounting on a second channel. The verdict goes to standard output, and the corpus, the lock and the injected values decide every byte of it. The hit count goes to standard error, which a differential over the verdict does not read.

A cache also never makes a run partial. The run creates every instance, every instance carries an outcome, and coverage counts what it counts without a cache. A run that evaluates part of a corpus is change-scoped evaluation, and that one owes an answer about coverage over a partial pass.

## The plugin interface

The interface is deliberately narrow:

```
DocumentCheck {                      // one trait per scope, and the trait is the declaration
  id()        -> CheckId
  severity()  -> Severity
  evaluate(view: &DocumentView) -> [Finding]
}
```

No filesystem, no network, no clock, no graph mutation. A plugin receives the same scoped view that a built-in check receives, and the same scope enforcement binds it. Thus a third-party check cannot break caching, cannot introduce non-determinism, and cannot see more of the corpus than it declared.

**The scope is the trait, so a host is never handed a scope claim to trust.** A plugin that wants to compare siblings implements the corpus-scoped trait, which is the only way to receive a corpus view. That is the rule of the section above, applied where it matters most.

**The obligation is not a method here, and the rule that made it one still holds.** Spec 4 admits no orphan check, and the binding between a rule and an obligation is data. A control names the mechanism `check:<id>` and the obligations that it discharges ([spec 4](04-assurance-model.md#controls-are-data)). The runner reads that register and stamps the finding. A check that named its own obligation would be a second source of truth for one binding. It could also name an identifier that no register holds. So the plugin surface is still not the place where a rule escapes the "every rule earns its place" discipline. A run names every rule that no control names. That is a report rather than a method signature. An adopter can then rebind a rule with no edit to code that they do not own.

## Where the LLM coherence sweep fits

The sweep is **not a check**, and every guarantee in this document depends on that distinction.

The sweep ([spec 4](04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep)) is non-deterministic. If it lived in the cached reproducible path, it would destroy every guarantee above. It runs as a separate **sampler**, with the same finding shape and the same reporting pipeline. Its provenance is marked `agent`, it never gates, and it is never cached as if it is reproducible.

That keeps "no LLM in the validation path" literally true, because the validation path is the one that produces verdicts. Coherence findings still flow through the same tooling that a human already reads.

## The correctness roots

Fixture discipline (below) covers checks. It does not cover the components that every check silently trusts. A defect in any of these produces systematically green or misdirected results, which is the silent-pass failure one level up. Each component therefore owes its own conformance fixtures, in the same spirit as "a check without a failing fixture does not ship":

- **The overlay resolver and the lock.** Every downstream verdict reads the lock. A resolver bug corrupts every check, projection, and conformance claim at once. A committed and diffable lock decreases the risk but does not test the resolver. The resolver carries its own round-trip and confluence fixtures, in the same way that Q6 already demands fidelity tests for the RDF projection.
- **Scope enforcement.** A leak silently corrupts every cache key (stated above). That makes the enforcer the correctness root for all caching and for change-scoped CI. A leak reproduces deterministically, so it looks like correct behavior.
- **The census walker.** Every coverage guarantee (OB-COV-1..3) assumes that the walk enumerates the corpus root correctly. A glob or symlink bug quietly shrinks the denominator, which is the exact failure that the census exists to prevent. The walker ships with a fixture tree of the pathological cases.
- **The parser's spans, its sentence segmentation, and the author-owned span.** Section contracts, voice checks, and prose-link extraction all trust one parse. A mis-parsed heading lets a section contract pass with no finding anywhere. The two other properties are here on measured grounds. Over this repository's own specification, most errors of a lexical checker came from the decision about which text is a sentence. The rest of that structural share came from text which quotes another author ([evaluation](../evaluations/what-a-check-can-know.md)). So the parser owns both, and no voice rule declares an exemption for either. A parser conformance corpus is part of the engine's own test surface.
- **The scaffolder.** Edges marked `created_by: scaffold` are corpus facts that nobody reviews individually. A scaffolder bug manufactures wrong edges at exactly the scale that the assisted-fraction metric celebrates. Scaffolder output goes through the same validation pipeline as authored input. That the output is generated is never a reason to trust it.
- **An importer**, on the scaffolder's terms and for the identical reason. Edges marked `created_by: import` arrive in bulk from a system that this corpus does not govern, and nobody reads them one at a time. A wrong imported edge produces a *correct* check result over a *wrong* graph, so no check finds it and no advisory posture helps ([spec 4](04-assurance-model.md#promotion-measures-a-rule-and-not-a-producer-of-facts)). A fixture set over the importer is the instrument, and it is what lets an imported edge carry full weight ([Q19](09-decisions.md#q19--inbound-integration-an-external-system-of-record)).
- **External-anchor resolvers.** Write-time impact detection fires on anchor identity ([spec 2](02-taxonomy-model.md#behavior-at-the-limits)). A resolver that mis-normalizes makes `governs` edges silently miss.
- **The cache.** A cache that can change a verdict is a store under another name. `headwater check --no-cache` and `headwater check` produce byte-identical output, and that comparison is a fixture rather than an assumption ([spec 6](06-engine-architecture.md#nothing-stores-the-graph)).
- **Kind resolution and the graph projector**, the two components that [Q6](09-decisions.md#q6--where-the-corpus-graph-lives-at-rest) named. The projector now carries a check of its own rather than a warning. Every export run emits a census over the projection, and an omission that no declared loss reason covers fails the run ([spec 6](06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped)).
- **The probe grader.** It re-derives every verdict from a committed transcript and a declared expectation, and nobody reads a probe result one instance at a time. A grader that evaluates a predicate wrongly returns systematically green efficacy, which is the silent pass one level up, and no probe finds it. The grader is a pure function of the transcript, the expectations and its own version. So `generate --check` over the result document is the standing test, and a fixture set of transcripts covers the predicate forms ([spec 5](05-ai-integration.md#a-probe-is-a-document-with-a-declared-expectation)).
- **The filter of an export profile**, which is the one root whose defect nobody can repair. Every other component on this list produces a wrong result that a later run corrects. A filter that carries a document it should have withheld has released bytes across a boundary, and no run takes them back ([Q17](09-decisions.md#q17--governed-access-and-the-solution-layer)). The census proves that the output matches the declared partition. A differential fixture set per profile proves that the partition matches the declared predicate. Both are needed, because the census cannot see a predicate that is wrong in the same way that the output is.

## Testing: a check without a failing fixture does not ship

Every check ships with at least one fixture that it fails and one that it passes. This is the floor, not the goal.

This rule also connects to promotion ([spec 4](04-assurance-model.md#promotion-advisory-to-blocking)). The false-positive rate is measured against real corpora. A check that fails on no constructed example is a check whose author does not know what it detects. That is exactly the wallpaper that the manifesto principle exists to remove.

## What this leaves open

- The `Neighbourhood(depth)` scope is speculative. If no real check needs depth > 1, the correct move is to cut it and to keep `Edge` as the only relational scope.
- Whether `Shelf` is a separate scope or only `Corpus` with a filter. This matters only if sibling-comparison checks become common.
- Whether plugins are in-process (fast, but a foreign-code trust question) or subprocess (safe, but the per-instance overhead can dominate for `Document`-scoped checks). [Q1](09-decisions.md#q1--implementation-language) supplies a third option that answers both horns. A WebAssembly component runs in-process, and it receives no filesystem, no network, and no clock unless the host grants them. That is the plugin contract above, restated as a capability model. The plugin design makes the call, and it is no longer a choice between two bad options. The [Q1 spike](../evaluations/language-spike-results.md) makes this plausible and does not test it. It compiled the engine *to* WebAssembly, which is not the same as hosting a component.
