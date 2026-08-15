---
id: HW-DR-0021
title: Q21 — Terminological succession, and validity under merge
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: A retired-term lexicon sits in the language regime, and a verdict reports the read set that a merge may void.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-EVAL-what-a-check-can-know
---

# Q21 — Terminological succession, and validity under merge

## Context

One [evaluation](../evaluations/what-a-check-can-know.md) settles this with [Q5](0005-voice-checking-depth.md). Both halves survive in substance, and the argument for each one changes.

### The observed case, with its commits

This project produced the case itself, which [principle 8](../spec/00-vision-and-scope.md#design-principles) makes the test that matters.

Commit `684a153` removed a framing from the specification and named the retired phrase, "reference system". Commit `a1aecc1` records the repair. A concurrent branch, written from an earlier commit, used the retired phrase in new prose. Git merged both without a conflict, and the linter passed because no rule knew the phrase was retired. A human found it while reading a diff.

The judgment "we no longer describe it that way" existed only as prose and a diff, so no mechanism could inherit it.

## Decision

### Part one: the retired-term lexicon, and where it lives

**A retired-term lexicon in the taxonomy, under the language regime** ([spec 2](../spec/02-taxonomy-model.md#the-language-regime-carries-the-terms-that-the-corpus-retired)). Terms do not become documents, because the glossary is a projection today and that would invert it. SKOS labels wait for the taxonomy export to have a consumer, which is the trigger that [Q13](0013-linkml-and-shacl-as-substrate.md) already set for emitter 4.

**It belongs to the language regime and never to a voice regime, and the reason is scope.** A voice regime binds per kind, and its `narrative` value exempts a kind entirely. A retired term is retired in a proposal as much as in a specification. The declaration count stays at thirteen.

### Part two: validity is not preserved under merge

The general statement is the larger of the two, and [spec 4](../spec/04-assurance-model.md#a-verdict-is-about-one-state-of-the-corpus) now makes it as a property of a verdict.

> Two changes that are each valid against the merge base can produce an invalid corpus, and no run against either branch tip reports it.

**We already have the read set, and that is the whole ruling.** [Spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) keys every result on a hash of exactly the in-scope inputs. A run therefore computes its read set as a byproduct of scope enforcement. It now reports that set, with the corpus tree and the lock hash. A merge is then an ordinary change. The engine derives the invalidated instances from it as from any diff, and a verdict survives only when nothing it read has moved.

## Consequences

**The open-question entry's argument against the lexicon is the one part of it that the measurement contradicts.** The entry said that lexical matching brings the false positives Q5 warns about. Q5 measured them, and they come from segmentation and from part of speech. The lexicon produced approximately none. A retired-term list is a closed, authored, small set of exact strings, which is the highest-precision shape a lexical rule takes. It inherits Q5's two parser obligations and not Q5's warning.

**The replacement decides fixability, and that reverses the leaning on posture.** With a replacement the fix is a substitution, which meets the [fixability bar](../spec/12-check-layer.md#fixability), so the check offers a patch and can finish the promotion path. The leaning said it ships advisory "because it is lexical". It ships advisory under the ordinary [principle 4](../spec/00-vision-and-scope.md#design-principles) rule, and being lexical is not what holds it there. The observed case is the other shape. "Reference system" had no replacement term, and the repair rewrote the clause.

**What happens when a lexicon lands on a live corpus needs no new mechanism.** A new entry makes checks that passed fail, which breaks the `consequence` [compatibility dimension](../spec/02-taxonomy-model.md#versioning-by-measured-compatibility) and forces a major version. A major ships a migration payload, which knows which rules it broke for which documents. Those are `migration-pending` findings at `(document, rule)` grain, with an owner and an expiry ([spec 7](../spec/07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states)). `.ste-lint-baseline.json` was a hand-rolled version of that with neither an owner nor an expiry.

**One consequence from the [warrant](../spec/01-conceptual-model.md#warrant).** A term is retired under every warrant and the check does not vary. The escape does. An `asserted` document has nobody who accepted anything, so it carries no `accepted_deviation`, and its remedy is the one that [spec 3](../spec/03-authoring-and-lifecycle.md#freshness-and-staleness) gives it already.

**The anomaly has a name outside version control, and the name comes with a solution.** Under snapshot isolation, two transactions that read overlapping data and write disjoint data each preserve an invariant that the pair violates. That is **write skew**, and git permits it for the identical reason: both detect a write-write overlap and neither holds a read set. Serializable snapshot isolation detects it by tracking what each transaction read, and it pays with a false abort ([spec 10 §F.6](../spec/10-theoretical-foundations.md#f6-write-skew-names-the-anomaly-and-read-sets-detect-it)).

**The correction to the entry's own text.** It said that corpus-scoped barriers are what catch a reintroduced term. They are not. A retired-term check reads one body and the lexicon, so it is `Document`-scoped. What makes a new lexicon entry retroactive is the lock. The lexicon sits in the taxonomy, the lock hash is in every cache key, and a lock change voids every cached result at once.

**Corpus scope is the honest cost of the performance bet.** A corpus-scoped instance reads everything, so any concurrent change voids it and no incremental test rescues one. Their count is now readable as the work that every merge repeats.

**The engine emits and never orders** ([spec 6](../spec/06-engine-architecture.md#ci-adapters)). A merge queue answers the question completely and pays with a serialized landing, and that trade belongs to the forge. This is the boundary that [Q7](0007-scope-of-the-mcp-surface.md) drew for the write path, met a second time. The invalidation test fails toward re-running, because a false invalidation costs one run and a false survival ships an invalid corpus with a green report.

**How exposed this repository is.** Across the 29 merge commits on `main`, the mainline had moved past the merge base in 16 of them. In 11 of the 29, both sides changed at least one file under `docs/spec/` from that base. The rate does not show that the failure is common. It shows that the window is the normal case, which is what the entry claimed and could not measure.
