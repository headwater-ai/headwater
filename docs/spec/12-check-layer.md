# 12 — The check layer

[Spec 6](06-engine-architecture.md) says that checks are pure functions over the corpus graph. That was sufficient at that level of detail. The LinkML and SHACL evaluations then found where the standards stop. Everything past that line lands here. Thus the check layer needs a design, not only a description.

The signatures below are pseudocode, given as examples. The implementation language is [Q1](09-open-questions.md#q1--implementation-language). Nothing here depends on it.

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

**`exportable_as` is machine-checkable.** The emitted LinkML and SHACL are generated from exactly the checks that are marked exportable. The export metadata declares which obligations it does not cover. Thus the "declared subset" promised in the SHACL evaluation cannot drift from the truth, because both sides are generated from the same list.

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

**2. Sound cache keys.** A result is keyed on a hash of exactly the inputs in scope, plus the taxonomy lock hash, the check's version, and any injected values. Nothing outside the scope can affect the result, so nothing outside it needs to be in the key.

**3. Parallelism, with visible serialization points.** `Document` and `Edge` checks are embarrassingly parallel. `Corpus` checks are the barriers. Because scope is declared, their count is a number that you can read, not a property that you discover under load.

**4. Enforcement, which is what makes the rest honest.** The view exposes *only* what the scope declared. A `Document`-scoped check physically cannot read a sibling. So a scope declaration cannot quietly rot into a lie. A scope that is declared but not enforced would silently corrupt every cache key derived from it. The enforcement is the feature, and the declaration alone would be a comment.

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

## The plugin interface

The interface is deliberately narrow:

```
Check {
  id()          -> CheckId
  obligation()  -> ObligationId      // required — spec 4 admits no orphan checks
  scope()       -> Scope
  severity()    -> Severity
  evaluate(view: &ScopedView) -> [Finding]
}
```

No filesystem, no network, no clock, no graph mutation. A plugin receives the same scoped view that a built-in check receives, and the same scope enforcement binds it. Thus a third-party check cannot break caching, cannot introduce non-determinism, and cannot see more of the corpus than it declared.

`obligation()` is required so that the plugin surface does not become the place where rules escape the "every rule earns its place" discipline.

## Where the LLM coherence sweep fits

The sweep is **not a check**, and every guarantee in this document depends on that distinction.

The sweep ([spec 4](04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep)) is non-deterministic. If it lived in the cached reproducible path, it would destroy every guarantee above. It runs as a separate **sampler**, with the same finding shape and the same reporting pipeline. Its provenance is marked `agent`, it never gates, and it is never cached as if it is reproducible.

That keeps "no LLM in the validation path" literally true, because the validation path is the one that produces verdicts. Coherence findings still flow through the same tooling that a human already reads.

## The correctness roots

Fixture discipline (below) covers checks. It does not cover the components that every check silently trusts. A defect in any of these produces systematically green or misdirected results, which is the silent-pass failure one level up. Each component therefore owes its own conformance fixtures, in the same spirit as "a check without a failing fixture does not ship":

- **The overlay resolver and the lock.** Every downstream verdict reads the lock. A resolver bug corrupts every check, projection, and conformance claim at once. A committed and diffable lock decreases the risk but does not test the resolver. The resolver carries its own round-trip and confluence fixtures, in the same way that Q6 already demands fidelity tests for the RDF projection.
- **Scope enforcement.** A leak silently corrupts every cache key (stated above). That makes the enforcer the correctness root for all caching and for change-scoped CI. A leak reproduces deterministically, so it looks like correct behavior.
- **The census walker.** Every coverage guarantee (OB-COV-1..3) assumes that the walk enumerates the corpus root correctly. A glob or symlink bug quietly shrinks the denominator, which is the exact failure that the census exists to prevent. The walker ships with a fixture tree of the pathological cases.
- **The parser's span retention.** Section contracts, voice checks, and prose-link extraction all trust one parse. A mis-parsed heading lets a section contract pass with no finding anywhere. A parser conformance corpus is part of the engine's own test surface.
- **The scaffolder.** Edges marked `created_by: scaffold` are corpus facts that nobody reviews individually. A scaffolder bug manufactures wrong edges at exactly the scale that the assisted-fraction metric celebrates. Scaffolder output goes through the same validation pipeline as authored input. That the output is generated is never a reason to trust it.
- **External-anchor resolvers.** Write-time impact detection fires on anchor identity ([spec 2](02-taxonomy-model.md#behavior-at-the-limits)). A resolver that mis-normalizes makes `governs` edges silently miss.
- **Kind resolution and the graph projector**, the two components already named in [Q6](09-open-questions.md#q6--where-the-corpus-graph-lives-at-rest).

## Testing: a check without a failing fixture does not ship

Every check ships with at least one fixture that it fails and one that it passes. This is the floor, not the goal.

This rule also connects to promotion ([spec 4](04-assurance-model.md#promotion-advisory-to-blocking)). The false-positive rate is measured against real corpora. A check that fails on no constructed example is a check whose author does not know what it detects. That is exactly the wallpaper that the manifesto principle exists to remove.

## What this leaves open

- The `Neighbourhood(depth)` scope is speculative. If no real check needs depth > 1, the correct move is to cut it and to keep `Edge` as the only relational scope.
- Whether `Shelf` is a separate scope or only `Corpus` with a filter. This matters only if sibling-comparison checks become common.
- Whether plugins are in-process (fast, but a foreign-code trust question) or subprocess (safe, but the per-instance overhead can dominate for `Document`-scoped checks). This interacts with Q1.
