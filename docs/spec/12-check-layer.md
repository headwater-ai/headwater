# 12 — The check layer

[Spec 6](06-engine-architecture.md) says checks are pure functions over the corpus
graph. That was enough at altitude. The LinkML and SHACL evaluations then established
where the standards stop, and everything past that line lands here — so the check layer
needs designing rather than describing.

Signatures below are illustrative pseudocode. The implementation language is
[Q1](09-open-questions.md#q1--implementation-language) and nothing here depends on it.

## What a check is

```
check(view: ScopedView, ctx: Context) -> [Finding]
```

**Pure.** No file I/O, no network, no clock, no mutation of the graph. Everything it may
read arrives through the view; everything time-dependent arrives through `ctx` as an
injected value. That purity is not fastidiousness — it is what makes results cacheable,
reproducible, and safe to run in parallel, and it is what the determinism requirement in
spec 6 actually reduces to.

## The five origins of a check

Spec 6 sketched three tiers. The right decomposition is five, and it falls out of the
boundary the two standards evaluations found rather than from guesswork.

| Origin | Comes from | Examples | Exportable as |
|---|---|---|---|
| **Shape** | the TBox, generated | required facet, enum membership, identifier pattern, cardinality, unknown-facet detection | LinkML + SHACL |
| **Graph** | relation declarations, generated | reciprocity, endpoint kinds, lifecycle-sensitivity, satellite inheritance, live conflicts | SHACL (via SPARQL) |
| **Corpus** | declarations needing many documents | sequence expectations, facet orthogonality, continuity distribution, scent distinctiveness | — |
| **Document** | regimes, applied to the body | voice, section contract, normative language, size budgets, prose-link resolution | — |
| **Plugin** | adopter code | anything organisation-specific | — |

Three things this table settles.

**Shape and Graph checks are generated, not written.** Adding a facet or a relation to
the taxonomy produces its checks with no code. That is the whole point of taxonomy-as-
data, and it is where most of the check count lives.

**Document checks are the ones no graph standard can reach**, because the body is not in
the graph — the finding from the
[SHACL instance-data evaluation](../evaluations/shacl-worked-example.md#does-this-help-with-the-actual-documents).
They are also, not coincidentally, the checks that need source positions.

**`exportable_as` is machine-checkable.** The emitted LinkML and SHACL are generated from
exactly the checks marked exportable, and the export metadata declares which obligations
it does not cover. The "declared subset" promised in the SHACL evaluation therefore
cannot drift from the truth, because both sides are generated from the same list.

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
```

Scope buys four things, and the fourth is what makes the other three trustworthy.

**1. Change-scoped evaluation.** Given a diff, the engine computes exactly which check
instances are invalidated. `Document` checks re-run for changed files. `Edge` checks
re-run for every edge *incident to* a changed file — in both directions.

That last clause is the subgraph problem the SHACL evaluation surfaced, and declaring
scope is what solves it. Editing document A to add `supersedes B` invalidates the
reciprocity instance on that edge regardless of which endpoint the check reports
against, because the edge is the unit, not the file. No guessing from the diff.

**2. Sound cache keys.** A result is keyed on a hash of exactly the inputs in scope, plus
the taxonomy lock hash, the check's version, and any injected values. Nothing outside the
scope can affect the result, so nothing outside it needs to be in the key.

**3. Parallelism, with visible serialisation points.** `Document` and `Edge` checks are
embarrassingly parallel. `Corpus` checks are the barriers — and because scope is
declared, the count of them is a number you can look at rather than a property you
discover under load.

**4. Enforcement, which is what makes the rest honest.** The view exposes *only* what the
scope declared. A `Document`-scoped check physically cannot read a sibling. So a scope
declaration cannot quietly rot into a lie — and a declared-but-unenforced scope would
silently corrupt every cache key derived from it. The enforcement is the feature; the
declaration alone would be a comment.

## Instances, and why coverage needs them

A check is a template. The engine instantiates it per target: `facet_required` is not one
check, it is 412 instances. Findings, cache entries, and timings all attach to instances.

Coverage accounting ([spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for))
then falls out rather than being bolted on. Each run records, per document, which
instances were created, which ran, which were served from cache, and which were skipped
with a reason. **A document with zero instances is a finding** — it means a shelf pattern
is wrong or a file is misplaced, and both are worth knowing.

## Two phases, and why the order is load-bearing

**Phase A — classify and build.** Parse every file, resolve its kind, build edges, index
identifiers. Failures here are structural findings: unparseable file, unclassifiable
path, dangling edge, ambiguous shelf match.

**Phase B — check.** Everything above, over the graph Phase A produced.

Phase A emits a **census**: every file under the corpus root and what became of it. The
coverage report in Phase B is computed against that census — not against the set of
documents that happened to classify successfully.

That ordering is the direct answer to the silent-pass failure mode. The denominator is
fixed before any checking starts, so a document that fails to parse is counted, reported,
and visibly unchecked, rather than dropping out of the run and leaving a clean result
behind it.

## Findings

The shape is [spec 4](04-assurance-model.md#findings)'s. Two obligations the check layer
carries to satisfy it:

- **Source positions.** Findings anchor to a line, which means the parser retains spans
  for front-matter keys, headings, and links. This is a parser requirement *driven by*
  the check layer, and it is the concrete reason an RDF projection cannot be the internal
  representation — spans do not survive the round trip.
- **Remediation and fixability.** Every finding states what to do. Checks that can fix
  mechanically say so.

## Fixability

A check may return a patch alongside a finding. The rule for whether it may:

> A fix is offered only when it is **mechanical and total** — one correct outcome,
> derivable without judgement.

Regenerating a stale projection, adding a missing reciprocal link, normalising
front-matter key order, correcting an identifier's format: mechanical. Rewriting a
section to satisfy a contract, choosing a summary, resolving a conflict between two live
decisions: not. Those carry remediation prose instead, and offering a plausible automatic
fix for them would be worse than offering none, because it would be applied unread.

## Severity is the check's; posture is the control's

A check reports severity. Whether that severity blocks is the **control's** business
([spec 4](04-assurance-model.md)), not the check's.

Keeping them apart means the same check serves an advisory deployment and a blocking one
unchanged, and promoting a check from advisory to blocking is a configuration change with
an audit trail rather than a code change. A check that knew whether it was blocking would
have to be edited to be promoted, and the promotion criteria would lose their teeth.

## Suppression is the runner's

Checks know nothing about suppressions. The runner filters findings, records exactly what
it filtered, and feeds the suppression inventory into the coverage report.

A check that handled its own suppressions could hide them, and a suppression nobody can
count is indistinguishable from a rule that never fires.

## Determinism, concretely

- **The clock is injected.** `ctx.now` is a bound value, never a syscall — the finding
  the SHACL sequence-expectation example forced, and it applies to our own engine
  identically.
- **Stable ordering.** Findings sort by (path, line, check id, message). No iteration
  over an unordered map reaches output unsorted.
- **Complete cache keys.** Content hashes of in-scope inputs, taxonomy lock hash, check
  version, injected values. A key that omits an input is a correctness bug, not a
  performance one.

Same corpus, same lock, same injected clock, byte-identical output. That is what makes
`generate --check` and projection freshness meaningful at all.

## The plugin interface

Deliberately narrow:

```
Check {
  id()          -> CheckId
  obligation()  -> ObligationId      // required — spec 4 admits no orphan checks
  scope()       -> Scope
  severity()    -> Severity
  evaluate(view: &ScopedView) -> [Finding]
}
```

No filesystem, no network, no clock, no graph mutation. A plugin receives the same scoped
view a built-in does and is bound by the same scope enforcement — so a third-party check
cannot break caching, cannot introduce non-determinism, and cannot see more of the corpus
than it declared.

Requiring `obligation()` is what stops the plugin surface becoming the place rules go to
escape the "every rule earns its place" discipline.

## Where the LLM coherence sweep fits

It is **not a check**, and the distinction is load-bearing.

The sweep ([spec 4](04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep))
is non-deterministic, so it cannot live in the cached reproducible path without
destroying every guarantee above. It runs as a separate **sampler**: same finding shape,
same reporting pipeline, provenance marked `agent`, never gating, never cached as though
reproducible.

That keeps "no LLM in the validation path" literally true — the validation path is the
one that produces verdicts — while letting coherence findings flow through the same
tooling a human already reads.

## Testing: a check without a failing fixture does not ship

Every check ships with at least one fixture it fails and one it passes. This is the floor,
not the goal.

It also connects to promotion ([spec 4](04-assurance-model.md#promotion-advisory-to-blocking)):
false-positive rate is measured against real corpora, and a check that cannot be made to
fail on a constructed example is a check whose author does not know what it detects. That
is exactly the wallpaper the manifesto principle exists to remove.

## What this leaves open

- The `Neighbourhood(depth)` scope is speculative. If no real check needs depth > 1, it
  should be cut and `Edge` kept as the only relational scope.
- Whether `Shelf` is a distinct scope or just `Corpus` with a filter, which matters only
  if sibling-comparison checks turn out to be common.
- Whether plugins are in-process (fast, but a foreign-code trust question) or subprocess
  (safe, but the per-instance overhead may dominate for `Document`-scoped checks). This
  interacts with Q1.
