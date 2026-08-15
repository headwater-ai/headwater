---
id: HW-SPEC-check-layer
status: current
status_since: 2026-08-02
last_verified: 2026-08-14
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
    - HW-EVAL-graph-export-and-federation
    - HW-EVAL-language-spike-results
    - HW-EVAL-relation-storage
    - HW-EVAL-shacl-worked-example
    - HW-EVAL-the-measurement-layer
    - HW-EVAL-the-serving-boundary
    - HW-EVAL-warrant-and-adjudication
    - HW-EVAL-what-a-check-can-know
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
| **Shape** | the TBox, generated | required facet, enum membership, identifier pattern, cardinality, unknown-facet detection | JSON Schema, and LinkML or SHACL when either emitter arrives |
| **Graph** | relation declarations, generated | reciprocity, endpoint kinds, lifecycle-sensitivity, satellite inheritance, live conflicts, windowed participation expectations | SHACL (via SPARQL), when that emitter arrives |
| **Corpus** | declarations that need many documents | facet orthogonality, continuity distribution, scent distinctiveness | — |
| **Document** | regimes, applied to the body | voice, section contract, normative language, size budgets, prose-link resolution | — |
| **Plugin** | adopter code | anything organization-specific | — |

This table settles three things.

**Shape and Graph checks are generated, not written.** A new facet or relation in the taxonomy produces its checks with no code. That is the full point of taxonomy-as-data. It is also where most of the check count lives.

**Document checks are the ones that no graph standard can reach**, because the body is not in the graph. That is the finding from the [SHACL instance-data evaluation](../evaluations/shacl-worked-example.md#does-this-help-with-the-actual-documents). Not by coincidence, they are also the checks that need source positions.

One example in that row asks for more than the row supplies. Prose-link resolution reads the destination of a link, and a fragment on that destination names a heading of another document. No scope below carries a second document's body. So the half that a document decides alone is a Document check. The other half waits for a grain that this list does not hold ([13 — Open obligations](13-open-obligations.md#design-work-that-nothing-blocks)).

**An origin is not a scope.** The origin says which part of the taxonomy a rule comes from, and the scope says what one instance of it covers. `identifier.claimed_twice` is Graph-origin, because the identifier index reports the collision, and it is corpus-scoped, because nothing smaller holds both claimants.

**`exportable_as` is machine-checkable.** The emitted shapes are generated from exactly the checks that declare a target, and the next section states the rules that keep the claim honest. The last column above states what an origin can reach, and the declaration is per rule. Two rules declare a target today, and both name `jsonschema`.

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

  + needs_body:    bool
  + needs_phase_a: bool      // what phase A could not make of this document
  + needs_clock:   bool
  + needs_prior:   bool      // change-scoped only: the prior committed version
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

**A gate reads the published set and the tree in front of it, and nothing else.** For each input the set lists, the gate hashes the file at that path in that tree. A listed hash that stands carries the verdict about that input. A hash that moved voids the verdict, and so does a listed path that the tree does not hold. The taxonomy lock is the one component that is not a document, and a lock that moved voids every result at once. `headwater gate --read-set <path>` is the verb, and it exits non-zero on a verdict that does not carry.

That statement is the whole test, and it decides three things.

**1. The test is a comparison over listed inputs rather than a diff of two trees.** A diff reads two trees, so it sees the file that a merge added. A diff also needs a tree, and no run of this engine computes one ([HW-OBL-0028](../obligations/0028-a-run-cannot-report-the-corpus-tree-because-nothing-computes.md)). The comparison needs neither a tree nor a run, so it costs one hash for each listed input. What that economy buys and what it cannot buy is decision 3.

**2. The clock voids a verdict, and a tree is not the whole subject of one.** A verdict is about one state of the corpus **and one day**. A rule that reads the injected clock is named on a `windowed` line of the artifact. A gate asked about another day voids those rules, and no tree change is needed for that. A run that reads no clock is about a tree alone, so the day decides nothing about it.

**A change voids a verdict too, and no line of the artifact names one.** A rule that reads the version a document stood at before a change decides about a tree, a day and a change. The artifact lists corpus paths, and a change is not one. So each such rule is named on a `change-scoped` line, and a gate refuses its verdict the way it refuses a barrier. A full-corpus run skips every instance of such a rule, so it publishes no line of that kind at all.

**3. Membership of the census is an input, and a list of members carries no membership.** A read set is a list of paths and hashes, and a hash is a fact about one member. The list records what a run read. It never records that those were all the documents there were. A document that a merge adds is therefore on no list, it moves no hash, and the comparison cannot see it. Two consequences follow and they are different sizes.

The small one is that a verdict about a listed document is worth what it looks worth. The bytes are there to compare, and the gate decides it.

The large one is that a document a merge adds generates instances that no earlier run held. So a gate reports the reach of its own answer, on a carrying verdict as well as on a voided one. It never reports that a corpus is green. That sentence is in the output of the verb rather than in this file alone. A reader of a green line does not open a specification first.

This is where the design gets something free that a database has to add. A serializable database tracks read sets at run time to detect write skew. Git detects nothing of the kind, because it holds no read set at all. Scope enforcement built ours for a different purpose ([spec 10 §F.6](10-theoretical-foundations.md#f6-write-skew-names-the-anomaly-and-read-sets-detect-it)).

**Corpus-scoped checks are the barriers, and this is where decision 3 consumes the answer.** A corpus-scoped instance decides its verdict from the extent of the census rather than from the contents of any member of it. `identifier.claimed_twice` fires on the presence of a second claimant, so its verdict rests on the absence of a document. A list of members states no extent, so no comparison over one rescues such a verdict. The artifact names each barrier on a `barrier` line, and a gate voids every one of them whatever the listed hashes did. Their count is the work that every merge repeats, and it is readable from the declarations rather than discovered under load.

The construction that this costs is small enough to state. One branch adds a document that mints an identifier, and a second branch adds a different document that mints the same one. Each branch holds one claimant, so each branch is valid. The merge holds two claimants and is invalid. Neither read set lists the document that the other branch added, so every listed hash stands. A gate that read the listed hashes alone would report that both verdicts survive, and the `barrier` line is what stops it.

**An external anchor is in no read set, so a key over one names what a resolver said.** A read set is a list of corpus paths, and an anchor names a target that is not one. The identity of an edge does not move when the target of its anchor does. A path anchor normalizes to the text that its author wrote, and an unbound target falls back to the same text. A key that held the identity alone would serve a verdict across the deletion of the file that the anchor names ([HW-OBL-0117](../obligations/0117-a-cached-verdict-about-an-anchor-survives-the-change-that-falsifies-it.md)). So the key of an edge instance names the binding that the resolver returned, beside the identity of the edge.

**The published artifact names no anchor, so a gate decides nothing about one.** It holds one line for each document that a run read, and the answer of a resolver is not a document. A comparison over listed hashes therefore cannot see the target of an anchor leave the tree. Every run over this corpus publishes a barrier line, and a barrier voids the whole answer before it reaches that limit. A gate that answered for one rule at a time would reach it, and [HW-OBL-0118](../obligations/0118-the-published-read-set-names-no-anchor-so-a-gate-decides-nothing-about-one.md) records the gap.

**The invalidation test fails toward re-running.** A false invalidation costs one run. A false survival ships an invalid corpus with a green report. That asymmetry decides every doubtful case, and it is the same rule that [principle 7](00-vision-and-scope.md#design-principles) gives an exporter.

## Temporal inputs: the clock and the prior version

Two inputs are about time. Both are injected, never fetched.

**The clock** (`needs_clock`) is a bound value. Windowed participation expectations read a declared origin date from the document ([spec 2](02-taxonomy-model.md#participation-expectations)) and compare it against `ctx.now`. There is no history walk and no search of the repository history.

**The prior version** (`needs_prior`) is the input that transition legality requires and that the earlier draft silently lacked. An illegal lifecycle transition is invisible in the current graph, because one `status` value cannot say how it was reached. A check that declares `needs_prior` receives the previously committed version of the changed document. The content hash of that version joins the cache key like any other input. The prior version is available **only in change-scoped evaluation**, because the diff is what supplies it. The caller supplies it as a manifest. That manifest names each document the change carries, and for each one a file that holds the bytes which stood before it. `headwater check --change` reads that manifest. This engine runs no version control command, and it opens no file that the manifest does not name. In a full-corpus run, instances of such a check are counted and reported as skipped, with the reason `change-scoped-only` — visible, never silent ([spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)).

**"Prior" is anchored, not assumed.** The prior version is the document as it stands on the branch where the change lands. That is the merge-base version for a proposed change, and the committed `HEAD` version for a working-tree hook. It is never an intermediate commit inside the incoming branch. A branch lands on the mainline as one state movement, whatever route it took internally, so intermediate flips are invisible by construction, not by accident. Two consequences are stated here so that nobody must discover them:

- **Legality is path-reachability, not edge membership.** A compound movement (`draft` at the merge-base, `superseded` in the result, via `current` inside the branch) is legal if and only if a path between the two states exists in the declared machine. A check of single-edge membership against the merge-base would reject movements that the machine permits.
- **Hook and CI can disagree only when the mainline moved** between the run of the hook and the merge. That is the ordinary race that every merge check has. The evaluation that counts is the one against the final merge-base, and that evaluation is deterministic: same merge-base, same incoming tree, same verdict.

That is a deliberately reduced guarantee, stated rather than implied. Transitions are verified when they land, and they are not re-derived from history later. Git history is not a check input. Vendoring and squash merges destroy it, and a guarantee that depends on a search of repository history is not a guarantee.

**A change reaches this engine as a named set of inputs, and never as a second tree.** The prior version above is the one exception, and the change that supplies it also bounds its scope. Three components enact the general rule. `headwater gate --read-set` decides a verdict by [a comparison over listed inputs](#the-read-set-and-what-a-merge-does-to-a-verdict) rather than by a diff of two trees. `headwater probe stale` decides whether a change voided a recorded result. It recomposes a digest over a named set of documents to do it. `taxonomy audit` reports the standing `asserted` population rather than the promotions per change that [spec 3](03-authoring-and-lifecycle.md#promotion-is-one-human-one-document-one-diff) once assigned to it. That verb reads one working tree, so it reaches no prior version. A count of promotions in one change is `warrant.promoted`, which declares `needs_prior` and reads the manifest a caller named. The caller names the set in each case, so the injection has an author. A verb that walked history would be the first component here to read a tree that no caller named.

## Instances, and why coverage needs them

A check is a template. The engine instantiates it for each target. For example, `facet_required` is not one check but 412 instances. Findings, cache entries, and timings all attach to instances.

Coverage accounting ([spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)) then comes directly from this, with no added mechanism. Each run records, per document, which instances were created, which ran, which were served from cache, and which were skipped with a reason. **A document with zero instances is a finding.** It means that a shelf pattern is wrong or that a file is misplaced. Both facts are good to know.

**Coverage counts routing rather than reading.** The engine generates an instance over its targets: one document, or both endpoints of one edge. A corpus-scoped instance has one target and that target is the corpus, so it counts against no document, whatever it read. The alternative deletes the rule above. Such an instance reads every document. A report that counted reading would call every document checked, and the first corpus-scoped rule would make the finding unreachable. The instance count would rise and the finding count would fall, which reads in every report as an improvement.

## Two phases, and why the order matters

**Phase A — classify and build.** The engine parses every file, resolves its kind, builds edges, and indexes identifiers. Failures here are structural findings: an unparseable file, an unclassifiable path, a dangling edge, an ambiguous shelf match.

**Phase B — check.** All the checks above run over the graph that Phase A produced.

Phase A emits a **census**: every file under the corpus root and what became of it. The coverage report in Phase B is computed against that census, not against the set of documents that classified successfully.

That order is the direct answer to the silent-pass failure mode. The denominator is fixed before any checks start. Thus a document that fails to parse is counted, reported, and visibly unchecked. It does not drop out of the run and leave a clean result behind it.

**A structural finding needs a rule before a gate can act on it.** Three of the four above are a census row. The coverage rule reads the census, so a document with no instance is a finding already. A dangling edge has no row, because a row is a file and an edge is not one. `relation.target.unresolved` is the rule that carries it. Its unit is one entry of one `relations:` block, because a target that binds to nothing has no second endpoint to pair with.

A defect that stops an edge from existing needs another grain. Five of them do that, and each leaves nothing to instantiate an edge-scoped rule over.

- a `relations:` block that is not a mapping
- a relation name that no declaration holds
- an entry that is neither a target reference nor a mapping
- an entry that names no target
- a triple that one document writes twice

The unit that survives is the document that wrote the block. `relation.declaration.unusable` is document-scoped, and its view carries what phase A made of that one document. A second reading of the same front matter would be a second definition of each defect. One of the five would also disagree with the build. A repeated triple is a repeated **normalized** target, and only a resolver decides that.

`identifier.unusable` carries the sixth. A document with no identifier is neither end of any edge. The census counts it as checked, and every relation it declares is lost. The identifier index says the same thing from the other side. It says it again for a document that writes a sequence where one identifier belongs.

Two documents that claim one identifier need a wider grain than either of them. Neither file is defective on its own. Each one declares a well-formed identifier that its scheme admits, and what is wrong is the pair. No document holds the pair. Nothing connects the two either, so `Edge` and `Neighbourhood` reach neither of them.

`identifier.claimed_twice` is corpus-scoped for that reason, and it is the first barrier this engine carries. The read set of its one instance is every row of the census that carries a document, which is the set the identifier index reads. A document-scoped instance would read one file, so its key would name one file. The verdict would then survive every edit to the other claimant, which is the edit that settles the collision. A cache that serves a verdict across that edit is the correctness bug a complete key exists to prevent.

The rule reports twice, once in each file, with one sentence that names both documents. The index reports the claimant that comes second in path order. Path order is a fact about the corpus rather than about either author. A single finding against the second file would report path order as the defect.

## Findings

The shape of a finding comes from [spec 4](04-assurance-model.md#findings). The check layer carries two obligations to satisfy it:

- **Source positions.** Findings anchor to a line, which means that the parser retains spans for front-matter keys, headings, and links. This is a parser requirement that the check layer *drives*. It is also the concrete reason that an RDF projection cannot be the internal representation: spans do not survive the round trip.
- **Remediation and fixability.** Every finding states what to do. Checks that can fix mechanically say so.

## Fixability

A check may return a patch alongside a finding. The rule for whether it may return one:

> A fix is offered only when it is **mechanical and total** — one correct outcome, derivable without judgment.

To regenerate a stale projection, to add a missing reciprocal link, to normalize front-matter key order, to correct the format of an identifier: these are mechanical. To rewrite a section to satisfy a contract, to choose a summary, to resolve a conflict between two live decisions: these are not. Those carry remediation prose instead. A plausible automatic fix for them would be worse than none, because it would be applied unread.

**Fixability is the patch, and never a second field beside it.** The bar above is a test over a defect. `fixable` in a report answers a narrower question: whether `headwater check --fix` writes this correction. A finding is fixable when a patch rides with it, and two values cannot disagree where there is one. [HW-OBL-0087](../obligations/0087-fixable-has-two-readings-inside-one-engine.md) records the reading that the flag lost. The severity carries that reading: a defect whose remedy is mechanical is an error, and the engine may still write nothing.

**The bar is per finding rather than per rule.** One rule states two defects whose remedies differ. `doesn't` expands to `does not` and nothing else, and `it's` is `it is` or `it has`, so a reader of the sentence settles the second. Both are the same rule and the same severity, and only the first carries a patch.

### What a patch may say

Two shapes, and each one is a promise the engine can keep.

- **A byte range of one file, and the text to put there.** The offsets come from the parser's spans, and the patch also states what the check believes lies at them.
- **One edge half, as a relation and a target.** No offset, because the shape of the far document decides where the half goes.

### A patch is written, read back, and refused

A rewrite at a wrong offset corrupts a document rather than reporting a wrong line. The offsets of a prose patch come from a [correctness root](#the-correctness-roots). So the write path carries the promise the scaffolder makes for a reciprocal half. It assumes, it tests the assumption against the file, and a wrong assumption is a refusal with nothing written.

Four guards, and every one of them holds one file at a time.

1. The range opens after the front-matter block. A facet is a different writer with a different read-back.
2. The bytes at the range are the bytes the check read.
3. The range lies inside one run of this author's prose that reads every byte it spans. A code span, a link and a block quotation each reach a rule with the markup removed. So an offset taken from the text a reader sees can land early.
4. The result parses again as the same document with the substitution in it. Every block keeps its kind, its quote depth and its runs, every run keeps its owner, and every link keeps its destination.

The fourth guard is what makes the promise, and the first three are insurance. A patch that turned a word into markup changes the parse, and a patch inside a link's destination does the same. The parse is what the guard compares.

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

### Four things stop a sweep from gating, and none of them is a rule that somebody keeps

A promise that a mechanism never blocks is worth what enforces it. Four things enforce this one, in the order of how hard each is to undo.

**The compiler.** The sampler is its own crate, and that crate names `headwater-check` for the finding shape. So `headwater-check` can never name the sampler: the dependency would be a cycle and the build would refuse it. No rule can read a sweep finding, no run can carry one, and `check --strict` cannot see one. This is the argument that the adapter boundary already makes one level up, applied where the cost of an error is highest.

**The exit status.** `headwater sweep report` exits zero with findings, exits zero with every finding refused, and exits zero when it refuses the whole file. There is no `--strict`. A refusal is a verdict about the sampler rather than about the corpus, and it is tempting to exit non-zero on one. That would put a model's output on an exit status, which is the property this section exists to deny. The two non-zero exits are the caller's: a path that the process cannot read, and a `--format` that names no target. Both are decided before any file is parsed.

**The absence of a caller.** [The commit gate](../../CLAUDE.md) runs `headwater check --strict` and nothing else, and the CI job runs the verbs that spec 6 lists. Neither names this verb. A sweep runs when a person or a schedule asks for one.

The probe half of this path is where the third enforcement reads differently, and the difference is worth stating. `generate --check` runs in the CI job and it reaches the grader, because a probe result is a projection over a committed transcript. What that gate compares is bytes. A run in which the model answered every question wrongly writes a result that reports a low rate, and the gate passes over it. The gate fails on a derivation that no longer agrees with its source. That is a defect in the grader, the parser or the committed inputs. So no exit status carries a model's behavior, and that is the property the four enforcements exist for.

**No socket.** No crate of this engine depends on an HTTP client, and the sweep verbs open no connection. A model that nobody can reach means that nobody wrote a return file, and a verb with no file to read says so and stops. So an unreachable model can never fail a build, because no build ever waits for one.

The engine also writes nothing. `sweep report` prints the front matter that would declare a proposed edge, and it has no `--write`. A proposal that an agent applies to itself is the same act as an agent that accepts its own draft. [HW-OBL-0108](../obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md) records that act at the scale of a whole corpus.

### What a suite over a sampler can hold

A check without a failing fixture does not ship, and a sampler has no such fixture, because the output under test is a model's. The suite therefore covers the two deterministic halves and states that it covers nothing else.

The plan is a function of the tree. The report is a function of a return file and the tree. Both are recorded whole and both are asserted to write one set of bytes over two runs. A return file that stands for what a model returns is written by hand, and it carries one refusal per test that the intake runs. A regression in any test then shows up as a finding that reached a reader and should not have.

One test in that suite asserts a silence. The fixture corpus holds the document of [HW-OBL-0113](../obligations/0113-every-check-passes-a-document-that-is-still-the-scaffolder-s-placeholder.md), the whole check layer runs over it, and no finding names it. The sweep intake then runs and one finding does. A green run is no evidence that a constraint is enforced, and this pair of assertions is the standing statement of that.

## The correctness roots

Fixture discipline (below) covers checks. It does not cover the components that every check silently trusts. A defect in any of these produces systematically green or misdirected results, which is the silent-pass failure one level up. Each component therefore owes its own conformance fixtures, in the same spirit as "a check without a failing fixture does not ship":

- **The overlay resolver and the lock.** Every downstream verdict reads the lock. A resolver bug corrupts every check, projection, and conformance claim at once. A committed and diffable lock decreases the risk but does not test the resolver. The resolver carries its own round-trip and confluence fixtures, in the same way that Q6 already demands fidelity tests for the RDF projection.
- **Scope enforcement.** A leak silently corrupts every cache key (stated above). That makes the enforcer the correctness root for all caching and for change-scoped CI. A leak reproduces deterministically, so it looks like correct behavior.
- **The census walker.** Every coverage guarantee (OB-COV-1..3) assumes that the walk enumerates the corpus root correctly. A glob or symlink bug quietly shrinks the denominator, which is the exact failure that the census exists to prevent. The walker ships with a fixture tree of the pathological cases.
- **The parser's spans, its sentence segmentation, and the author-owned span.** Section contracts, voice checks, and prose-link extraction all trust one parse. A mis-parsed heading lets a section contract pass with no finding anywhere. The two other properties are here on measured grounds. Over this repository's own specification, most errors of a lexical checker came from the decision about which text is a sentence. The rest of that structural share came from text which quotes another author ([evaluation](../evaluations/what-a-check-can-know.md)). So the parser owns both, and no voice rule declares an exemption for either. A parser conformance corpus is part of the engine's own test surface.
- **The scaffolder.** Edges marked `created_by: scaffold` are corpus facts that nobody reviews individually. A scaffolder bug manufactures wrong edges at exactly the scale that the assisted-fraction metric celebrates. Scaffolder output goes through the same validation pipeline as authored input. That the output is generated is never a reason to trust it. **The instrument is three tests and a fixture corpus.** The first records every case whole, so a decision that changes reaches a diff. The second fails when a refusal ships with no case, which is the rule below applied to a refusal rather than to a finding. The third tests the sentence above. It scaffolds into a copy of the fixture corpus and runs this whole check layer over the result. No finding may name a document that the run wrote or edited. It records the instance count beside the finding count, because a rule that stopped reading those documents would make a green run out of silence.
- **An importer**, on the scaffolder's terms and for the identical reason. Edges marked `created_by: import` arrive in bulk from a system that this corpus does not govern, and nobody reads them one at a time. A wrong imported edge produces a *correct* check result over a *wrong* graph, so no check finds it and no advisory posture helps ([spec 4](04-assurance-model.md#promotion-measures-a-rule-and-not-a-producer-of-facts)). A fixture set over the importer is the instrument, and it is what lets an imported edge carry full weight ([Q19](09-decisions.md#q19--inbound-integration-an-external-system-of-record)).
- **External-anchor resolvers.** Write-time impact detection fires on anchor identity ([spec 2](02-taxonomy-model.md#behavior-at-the-limits)). A resolver that mis-normalizes makes `governs` edges silently miss.
- **The cache.** A cache that can change a verdict is a store under another name. `headwater check --no-cache` and `headwater check` produce byte-identical output, and that comparison is a fixture rather than an assumption ([spec 6](06-engine-architecture.md#nothing-stores-the-graph)).
- **Kind resolution and the graph projector**, the two components that [Q6](09-decisions.md#q6--where-the-corpus-graph-lives-at-rest) named. The projector now carries a check of its own rather than a warning. Every export run emits a census over the projection, and an omission that no declared loss reason covers fails the run ([spec 6](06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped)).
- **The probe grader.** It re-derives every verdict from a committed transcript and a declared expectation, and nobody reads a probe result one instance at a time. A grader that evaluates a predicate wrongly returns systematically green efficacy, which is the silent pass one level up, and no probe finds it. The grader is a pure function of the transcript, the expectations and its own version. **The instrument is a fixture set of three transcripts and one property of the type.** One transcript satisfies every predicate form, one refutes every form, and one refuses every form. A grader that returns a single verdict for everything passes exactly one of the three. The property is that a satisfied verdict holds the event that satisfied it. That makes a verdict with no evidence behind it unwritable rather than untested. `generate --check` over the result document is the second half of the standing test. The projection that writes a result ships, and this repository still runs no such check. `docs/probe-runs/` holds no file, a recorder writes a transcript, and the generator prints that reason on every run ([spec 5](05-ai-integration.md#what-earns-the-grader-the-right-that-every-other-agent-facing-mechanism-is-denied)).
- **The filter of an export profile**, which is the one root whose defect nobody can repair. Every other component on this list produces a wrong result that a later run corrects. A filter that carries a document it should have withheld has released bytes across a boundary, and no run takes them back ([Q17](09-decisions.md#q17--governed-access-and-the-solution-layer)). The census proves that the output matches the declared partition. A differential fixture set per profile proves that the partition matches the declared predicate. Both are needed, because the census cannot see a predicate that is wrong in the same way that the output is.

## Testing: a check without a failing fixture does not ship

Every check ships with at least one fixture that it fails and one that it passes. This is the floor, not the goal.

This rule also connects to promotion ([spec 4](04-assurance-model.md#promotion-advisory-to-blocking)). The false-positive rate is measured against real corpora. A check that fails on no constructed example is a check whose author does not know what it detects. That is exactly the wallpaper that the manifesto principle exists to remove.

## What this leaves open

- The `Neighbourhood(depth)` scope is speculative. If no real check needs depth > 1, the correct move is to cut it and to keep `Edge` as the only relational scope.
- Whether `Shelf` is a separate scope or only `Corpus` with a filter. This matters only if sibling-comparison checks become common. The first sibling comparison did not settle it, and it could not. A taxonomy declares an identifier scheme per kind, several kinds sit on one shelf, and one kind sits on several shelves. So two claimants of one identifier need not share a shelf, and a shelf-scoped instance would pass over every collision that crosses one.
- Whether plugins are in-process (fast, but a foreign-code trust question) or subprocess (safe, but the per-instance overhead can dominate for `Document`-scoped checks). [Q1](09-decisions.md#q1--implementation-language) supplies a third option that answers both horns. A WebAssembly component runs in-process, and it receives no filesystem, no network, and no clock unless the host grants them. That is the plugin contract above, restated as a capability model. The plugin design makes the call, and it is no longer a choice between two bad options. The [Q1 spike](../evaluations/language-spike-results.md) makes this plausible and does not test it. It compiled the engine *to* WebAssembly, which is not the same as hosting a component.
