---
id: EVAL-HW-what-a-check-can-know
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: The Q5 and Q21 evidence, which measures what a lexical checker gets wrong on the checker that this repository already runs.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cited_by:
    - REG-HW-decisions
    - SPEC-HW-taxonomy-model
    - SPEC-HW-authoring-and-lifecycle
    - SPEC-HW-assurance-model
    - SPEC-HW-engine-architecture
    - SPEC-HW-theoretical-foundations
    - SPEC-HW-adjacent-work
    - SPEC-HW-check-layer
  traces_to:
    - tools/ste-lint.py
---

# What a check can know — the Q5 and Q21 evaluation

This evaluation closes [Q5](../spec/09-open-questions.md#q5--voice-checking-depth) and [Q21](../spec/09-open-questions.md#q21--terminological-succession-and-validity-under-merge). The two entries share one subject, and neither states it. A checker that matches strings does not know what a text means. A checker scoped to a change does not know what the rest of the corpus became. Both entries ask where the resulting error goes.

Q21 half-states the connection. Its retired-term lexicon is a lexical rule, and its own table says that the rule "brings the false positives that Q5 warns about". That is the smaller half of the connection, and the measurement below shows that it is also wrong.

## The unusual asset, and what it is evidence for

This repository is the observed case for both entries, which is what [principle 8](../spec/00-vision-and-scope.md#design-principles) asks for.

`tools/ste-lint.py` is a working lexical checker over `docs/spec/`. It carries a retired-phrase list, a per-line escape hatch with a reason, and a baseline of what predates each rule. It is interim scaffolding rather than a design, and this evaluation does not change one line of it. What it supplies is data. No prior evaluation has used it, and [principle 11](../spec/00-vision-and-scope.md#design-principles) makes an unmeasured claim about efficacy a claim that we may not publish.

Q21's semantic conflict is also this repository's own. Commit `684a153` removed a framing from the specification. Commit `a1aecc1` records the repair. A concurrent branch reintroduced the retired phrase in new prose, and git merged both without a conflict. The linter passed, because no rule knew that the phrase was retired. A human found it while reading a diff.

## What the specification already fixed

Six rulings bind both halves, and neither entry may revisit them.

**Severity is the check's and posture is the control's.** [Spec 12](../spec/12-check-layer.md#severity-is-the-checks-posture-is-the-controls) separates them, so a change of posture is a configuration change with an audit trail. Q5's "per-category severities" is thus a statement about posture, not about severity.

**A fix is offered only when it is mechanical and total.** [Spec 12](../spec/12-check-layer.md#fixability) sets that bar, and it names the rewrite of a section to satisfy a contract as the case that fails it.

**Promotion runs on two instruments, and one of them has a stated blind spot.** [Spec 4](../spec/04-assurance-model.md#promotion-advisory-to-blocking) records that an advisory finding forces nobody to suppress it, so suppression labels measure a biased subset. It requires an adjudicated sample for promotion because of that.

**Escape hatches carry a reason from a closed set.** `false_positive` means that the finding is wrong. `accepted_deviation` means that it is right and tolerated. Only the first counts toward the promotion statistics.

**A check reads exactly what its scope declared, and the cache key is the hash of those inputs.** [Spec 12](../spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on) enforces the scope through the type of the view, and calls a key that omits an input a correctness bug.

**The engine emits what a gate consumes, and never becomes the gate.** [Q7](../spec/09-open-questions.md#q7--scope-of-the-mcp-surface) closed on that boundary, and [spec 6](../spec/06-engine-architecture.md#ci-adapters) says that no forge is privileged in the core.

One further ruling is close enough to check and to set aside. [Principle 4](../spec/00-vision-and-scope.md#design-principles) gained an exception in the [serving-boundary evaluation](the-serving-boundary.md): where one error class is unrecoverable, a control ships at its final posture. That exception does not reach voice checking. A voice finding that fires wrongly is visible and cheap. A voice finding that fails to fire costs a sentence that a later reader or a later run still catches. Both error classes recover, so the ordinary promotion path applies.

## The measurement

### Method

The corpus is the 14 files of `docs/spec/`: 81,953 words of Markdown source, which the linter reduces to 1,572 prose units and 5,432 sentences. A run today reports **0 errors and 613 warnings**, with 13 further errors held by the baseline.

Warnings by rule: passive 449, paragraph length 88, auxiliary 54, progressive 12, heading length 10.

Three rules are the lexical voice proxies, and the samples below come from them. The sample is drawn with a fixed seed (`20260811`): 60 of 449 passive findings, 25 of 54 auxiliary findings, and every one of the 12 progressive findings. Each finding was read in its whole sentence and classified by hand, under the two labels that [spec 4](../spec/04-assurance-model.md#suppression) already defines.

- **False positive**: the matched text is not an instance of the construction that the rule names. A copula with a predicate adjective is not the passive voice. A noun phrase after *is* is not a progressive. An adjective built with *un-* is not a past participle. A modal with a copula has no active form to rewrite into.
- **Accepted deviation**: the match is a real instance of the construction, and the author keeps it.

### Result

| Rule | Population | Sampled | False positives | Rate |
|---|---|---|---|---|
| `passive` | 449 | 60 | 5 | 8% |
| `auxiliary` | 54 | 25 | 14 | 56% |
| `progressive` | 12 | 12 | 10 | 83% |

The rates differ by a factor of ten, and the reason is not the size of the pattern set. It is what each pattern is a proxy for. `passive` matches a form of *be* with a past participle, and that form is nearly always the passive voice. `progressive` matches a form of *be* with a word that ends in *-ing*, and in this corpus that word is usually a noun ("is nothing", "is reasoning power", "is authoring syntax"). `auxiliary` matches a modal with *be*, and in more than half of its findings the *be* is a copula that no rewrite removes.

### Where the errors came from at first contact

The blocking rules supply a second measurement, recorded in the history rather than made for this evaluation. Commit `3665182` landed the linter with 65 grandfathered violations. Commit `290a84d` then found four defects in the sentence splitter, each of which merged two adjacent sentences and reported the pair as one over-long sentence. Sentence-length errors fell from 58 to 26. **32 of the original 58, or 55%, were never real.** Of the 13 that survived at 28 words or more, 11 were rewritten and 2 carry an inline exception because they quote other authors.

Nothing in that repair touched a pattern. Every one of the 32 came from the layer that decides which text is a sentence.

The lexicon itself contributed almost nothing. The British-spelling rule found six violations on its first run and all six were real. The retired-phrase rule found one, and that one is correct: it echoes a phrase inside a quoted definition. The contraction rule found none.

### What the three sources add up to

A lexical checker on this corpus makes three kinds of error, in this order of size.

1. **Segmentation and span.** Which text is a sentence, and which text belongs to the author. 32 of 58 blocking errors at first contact, plus the two survivors that quote other authors, plus the passive findings that sit inside a quotation.
2. **Part of speech.** Whether the matched word is the verb that the rule assumes. 10 of 12 progressive findings, 14 of 25 auxiliary findings, 5 of 60 passive findings.
3. **The lexicon.** Whether the listed string is the thing we meant to list. Approximately zero, across four rules and one landing.

That order decides Q5, and it is the opposite of the order that the entry assumes.

### The instrument that Q5 names does not exist

613 advisory findings stand in this corpus. Four inline exceptions stand against them, and one of those four is for a quotation. Nobody suppressed a passive finding, because an advisory finding forces nobody to do anything.

So the false-positive rate that Q5 asks to revisit on is a fraction with a denominator of one. [Spec 4](../spec/04-assurance-model.md#promotion-advisory-to-blocking) already names this blind spot in general terms. This corpus is the extreme form of it, and it means that the suppression instrument can never close Q5. The adjudicated sample is the only instrument that reaches an advisory rule, and the table above is one.

### The finding that changes the ruling

Read the passive column again. The rule measures 92% precise on its sample, 449 findings stand, and not one of them should block.

Precision is not what decides the posture of this rule. Spec 4 already lists the criterion that does: promotion needs "an unambiguous, mechanical remediation path". The remediation for a passive sentence is a rewrite, and the rewrite needs judgment about who the actor is and whether naming the actor helps the reader. ASD-STE100 itself permits the passive in descriptive writing where the agent is unknown, which is most of a specification.

So a rule can hold an excellent false-positive rate and still be one that must never block. The promotion criteria already carry the fact, in a list where it reads as one requirement among four. This evaluation makes it decisive for a class of rules rather than a checkbox.

## Prior art and observed applications

[Principle 10](../spec/00-vision-and-scope.md#design-principles) asks for the research literature and at least one observed industry application. Sources below were checked against memory and against this repository. The web research budget for this session was exhausted before the numeric claims about static-analysis tolerance could be re-verified, and they are marked where they appear.

**ASD-STE100 is the standard that the house profile applies, and it argues against blocking on voice.** Its verb rules are written for procedures, where the actor is the reader and the imperative is available. Rule 3.2 asks for the active voice in descriptive writing "as much as possible", which is a different instruction from the procedural rule. The standard therefore agrees with the measurement: the passive in a specification is a preference and not a defect.

**The controlled-language checkers all ship the same shape, and none of them blocks on voice.** Vale, `textlint`, `proselint`, `write-good` and `alex` are rule sets over a text with per-rule severity and a per-line disable comment. Vale ships `error`, `warning` and `suggestion`, and its published style packages put readability and voice rules at the lower two. `alex` is the closest analog to a retired-term lexicon: a list of terms with replacements and reasons. It ships advisory by default, it is famous for its false positives on quoted and technical text, and its documented remedy is exactly a scoped ignore comment. Every one of these tools reaches the design that Q5 leans toward, which confirms the leaning and says nothing about its posture.

**The static-analysis adoption literature supplies the tolerance numbers, and they are lower than intuition.** Bessey and colleagues report a decade of Coverity deployments. Developers abandon a checker whose false-positive rate rises past roughly a third, and they ignore a correct checker that offers no clear remediation. Google's Tricorder reports a stricter operating point. An analyzer stays enabled only while its "not useful" rate stays under about a tenth, measured from the reviewers who press the button. Both numbers are from memory and were not re-verified in this session, and both point the same way. Tricorder's discipline is the sharper one for us, because it measures the reaction of a reader rather than the correctness of a finding. A reader who ignores a finding leaves no label at all.

**Database serializability is the frame for Q21's second half, and it names our anomaly exactly.** Berenson and colleagues, in "A Critique of ANSI SQL Isolation Levels" (1995), named **write skew**. Two transactions read overlapping data and write disjoint data, and each one preserves an invariant that the pair violates. Snapshot isolation permits it, because it detects write-write conflicts and nothing else. That is git. A branch is a transaction, the merge base is its snapshot, and a textual conflict is a write-write conflict at line grain. Git has no read set, so it cannot see the dependency that a semantic conflict runs through.

Fekete and colleagues (2005) showed that snapshot isolation is serializable when the dependency graph has no cycle with two consecutive read-write antidependencies, which makes the anomaly statically analyzable. Cahill, Röhm and Fekete (2008) turned that into Serializable Snapshot Isolation, which PostgreSQL ships as its `SERIALIZABLE` level. SSI tracks read sets at run time and aborts a transaction that closes a dangerous structure. Its documented cost is the false abort: a transaction that was serializable is rolled back anyway, and the system fails toward the recoverable error.

**Merge queues are the observed application, and they are the expensive answer.** The "not rocket science rule", as Graydon Hoare stated it for Rust's `bors`, has one clause. Never merge a commit that has not passed its tests **in the merged state**. GitHub merge queue, Zuul and `bors` all implement it, and all three pay for it with serialized landing. Zuul's speculative gating exists to recover the throughput. It tests a speculative future state so that several changes land in one pass, and it discards the speculation when an earlier change fails. That is optimistic concurrency, and it belongs to the forge that controls the landing order.

**Incremental build and incremental typecheck already solved the cheaper half.** Bazel keys an action on a hash of its declared inputs, and reuse is correct exactly when that declaration is complete. Salsa and the red-green algorithm inside `rustc` memoize a query with the set of queries it read, and invalidate a memoized result when a recorded dependency changes. Both are the same rule: **a result carries the set of inputs it depended on, and it survives a change only when that set is untouched.** Headwater already computes that set, per check instance, because [spec 12](../spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on) enforces the scope and hashes exactly the in-scope inputs into the cache key. The ingredient that a database has to add at run time, and that git does not have at all, is a byproduct of a decision we already made.

**Terminology deprecation at scale is the observed application for the vocabulary half.** Git's own default-branch rename in 2020 is the clearest case. The judgment was easy and its propagation was not, and what carried it was configuration (`init.defaultBranch`) plus tooling, never memory. The inclusive-language migrations across the Linux kernel, the major clouds and the language ecosystems ship as lexicons with replacements and reasons. The durable pattern in all of them is the split that this evaluation adopts. A retirement with a replacement becomes a mechanical rewrite that tooling performs. A retirement of a framing, with no replacement term, produces argument rather than change, and it needs prose and a human.

## The decision — Q5

**Lexical, confirmed. The escape hatch and the per-category posture survive. Two things change.**

**The revisit condition is replaced, because it names an instrument that cannot reach the question.** "Measured false-positive data" would have promoted a rule that must never block, and it cannot be collected for an advisory rule by the suppression instrument at all. What decides a voice category's posture is [spec 12](../spec/12-check-layer.md#fixability)'s fixability bar. A category may become blocking only when its remediation is mechanical and total. A category whose remediation is a rewrite is **permanently advisory**, on the same terms that [spec 4](../spec/04-assurance-model.md#measuring-coherence-where-we-can-continuity-across-links) already gives the shift ratio.

That is a general rule and it is stated as one. Where the promotion path stops early, spec 4 records it beside the case where the path is skipped entirely.

**The classifier is refused on aim rather than on accuracy.** A small local classifier attacks the third of the three error sources measured above, which is the smallest. It does not touch segmentation, and segmentation produced 32 of 58 blocking errors at first contact. It does not touch the lexicon, which produced approximately none. The residue it could fix lives inside the categories that may never block, where a better finding buys nothing that an advisory finding does not already buy. The trigger to reopen this is named. It is a voice category with a mechanical remediation, whose measured errors come mostly from part of speech. A classifier would then make that category eligible to block.

**Two obligations follow, and both land on the parser rather than on the rules.** A voice check runs over author-owned text, so a quotation, a code span, a citation line and a generated block are outside every voice rule by construction. And sentence segmentation joins the correctness roots of [spec 12](../spec/12-check-layer.md#the-correctness-roots), beside span retention, because this corpus shows that it is where a lexical checker's errors actually come from.

**The escape hatch carries a reason from spec 4's closed set.** A free-text reason cannot feed the promotion statistics, and the interim linter accepts only free text. That divergence is recorded here rather than repaired, because the scaffolding retires into the check layer and does not grow.

## The decision — Q21, part one: the vocabulary

**A retired-term lexicon, in the taxonomy, in the language regime.** Terms do not become documents. SKOS labels wait for the taxonomy export to have a consumer, which is the trigger that [Q13](../spec/09-open-questions.md#q13--linkml-and-shacl-as-substrate) already set for emitter 4.

**The entry's argument against the lexicon is the one part of it that the measurement contradicts.** A retired-term list is a closed, authored, small set of exact strings with a stated replacement. That is the highest-precision shape a lexical rule takes, and the lexicon is the component of this corpus's checker with no measured false positives. The lexicon does not inherit Q5's warning. It inherits Q5's two obligations, which are about the parser.

**The lexicon lives in the language regime and not in the voice regime, and the reason is scope.** A voice regime is bound per kind, and one of its values (`narrative`) exempts a kind from voice rules entirely. A retired term is retired in a proposal as much as in a specification. [Spec 2](../spec/02-taxonomy-model.md#language-is-declared-not-assumed) already puts the spelling lexicon and the controlled-vocabulary check in the language regime, and calls a controlled profile "a promise about the text". This is one more such promise. The declaration count stays at thirteen.

**An entry carries a term, a required reason, and an optional replacement, and the replacement decides fixability.** With a replacement, the fix is a substitution, which is mechanical and total, so the check offers a patch. Without one, the finding carries remediation prose and no patch. The observed case is the second shape: the retired phrase was "reference system", and the repair rewrote the clause around it. So the one case we have is the case that no fix repairs, and the lexicon would still have caught it, because the reintroduction used the retired string exactly. That is n=1 and it is stated as n=1.

**The reason field is required, and Q21 is right about why.** A retired term with no recorded reason is the authority rank that [Q18](../spec/09-open-questions.md#q18--recording-adjudicated-disagreements) rejected, with extra steps.

**Posture is advisory at first, under the ordinary [principle 4](../spec/00-vision-and-scope.md#design-principles) rule and not because the rule is lexical.** Unlike a voice category, a retired-term entry with a replacement can finish the promotion path, because its remediation is mechanical. That is the opposite of the entry's leaning, and it follows from the criterion that Q5 just established.

**What happens when a lexicon lands on a live corpus needs no new mechanism, and the interim tool's answer is the wrong one.** A new retired term makes checks that passed fail, which is the `consequence` dimension of [spec 2](../spec/02-taxonomy-model.md#versioning-by-measured-compatibility), and a broken dimension forces a major version. A major ships a migration payload, which declares which rules it broke for which documents, which is exactly the `(document, rule)` pair grain of `migration-pending` ([spec 7](../spec/07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states)). Those findings are counted, visible in coverage, never blocking, and never individually suppressed, and the state carries an owner and an expiry. `.ste-lint-baseline.json` is a hand-rolled version of that with no owner and no expiry, and it is one more reason the scaffolding retires.

**One limit of the observed case is worth recording.** The reintroduction landed in `docs/evaluations/language-choice.md`, and the linter's scope is `docs/spec/` alone. A check reaches the corpus that its taxonomy governs, and a judgment that reaches further than the corpus is outside every mechanism here.

**One consequence from the warrant ruling.** A retired term is retired under every [warrant](../spec/01-conceptual-model.md#warrant), and the check does not vary by warrant. The escape does. An `asserted` document has no human who accepted anything, so it cannot carry an `accepted_deviation`. Its remedy is the one that [spec 3](../spec/03-authoring-and-lifecycle.md#freshness-and-staleness) already gives it: produce the content again, or delete it.

## The decision — Q21, part two: validity under merge

**The specification states the general fact, and it states it as a property of a verdict.**

> A verdict is about one state of the corpus. Two changes that are each valid against the merge base can produce an invalid corpus, and no run against either branch tip reports it.

**A run therefore reports what it evaluated and what it read.** The corpus tree, the lock hash, and the **read set**: the union of the in-scope inputs that produced the run's results. Spec 12 already computes that set per check instance, because the cache key is a hash of exactly those inputs and a key that omits one is a correctness bug. Publishing the union costs a report and no new mechanism.

**The merge is then an ordinary change, and change-scoped evaluation decides whether a verdict survives it.** Given the merge result and the tree that a run evaluated, the engine computes the invalidated instances the way it computes them from any other diff. An empty result means the verdict carries. A non-empty result voids the verdict, and the recomputation is change-scoped over the union of the two changes rather than a full run.

That is the read-set half of Serializable Snapshot Isolation, arrived at from the other end. A database has to add read tracking to detect write skew. We have it already, and for a reason that has nothing to do with merges: scope enforcement is what makes cache keys sound.

**Corpus-scoped checks are the honest cost of the performance bet.** Their read set is the whole corpus, so any concurrent change voids them, and no incremental test can rescue one. [Spec 12](../spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on) already calls them the barriers and says their count is a number a reader can see. This ruling gives that number a second meaning: it is the work that every merge repeats.

**The correction to Q21's own text.** The entry says that corpus-scoped checks are what catch a reintroduced term. They are not. A retired-term check reads one document body and the lexicon, so it is `Document`-scoped. What makes a new lexicon entry retroactive is the lock. The lexicon sits in the taxonomy, the lock hash is in every cache key, and a lock change voids every cached result at once. The mechanism the entry reached for is the wrong one, and the right one is stronger, because it needs no barrier.

**The engine emits and never orders.** Headwater does not hold a queue, decide a landing order, speculate on future states, or block a merge. Those belong to the forge, and [Q7](../spec/09-open-questions.md#q7--scope-of-the-mcp-surface) already drew that line for the write path. What the engine owes a gate is the read set and the tree that it applies to. A merge queue can then skip a full re-run when nothing in that set moved. That is the same relationship as "emit what a change proposal needs, and let an adapter open it".

**The error asymmetry is stated, because it decides which way the test fails.** A false invalidation costs a re-run. A false validation ships an invalid corpus and reports it green. So the test fails toward re-running, which is [principle 7](../spec/00-vision-and-scope.md#design-principles) read the way that spec 6 reads it for an exporter. SSI makes the same trade and calls its cost the false abort.

### How exposed this repository is

The question sweep supplies the exposure measurement. Across the 29 merge commits on `main`, the first-parent side had moved past the merge base in **16 of them (55%)**. In **11 of 29 (38%)**, both sides changed at least one file under `docs/spec/` from that base. Every one of the 16 is a window in which a branch-tip verdict describes a tree that no longer exists. One of them produced the recorded incident.

The rate is not evidence that the failure is common. It is evidence that the window is the normal case rather than an edge case, which is what Q21 claims and could not show.

## What stays open

**Q5's real measurement has not been made, and the run that makes it is now specified.** This evaluation measured three ASD-STE100 structural rules as proxies. It did not measure `future_intent`, `change_narration` or `phased_rollout`, which are the categories that the declarative voice regime actually forbids, because no implementation of them exists. The run has three steps. Implement the three categories. Evaluate them over `docs/spec/`, which declares the declarative regime for every kind that it contains. Then adjudicate a fixed random sample of at least 50 findings per category, under the two labels above. Until that run exists, the claim that declarative voice is "mechanically detectable at useful precision" ([spec 8](../spec/08-design-departures.md)) is unmeasured, and it is published as unmeasured.

**Whether any voice category is ever fixable.** If none is, the whole voice layer is permanently advisory, and that is a smaller feature than spec 8 currently claims. Phased-rollout language is the most likely candidate for a mechanical fix, and nobody has tried.

**Whether a corpus ever needs a retired-term lexicon at all.** This repository needed one, at a scale of one entry. An adopter with no vocabulary judgment to record declares none, and the check has no instances.

**Whether the read set is small enough to be worth publishing.** A run over a thousand documents has a read set of about a thousand entries plus the lock, which is cheap. A corpus-scoped check makes it the whole corpus, and at that point the report says so and the invalidation test is trivially true. Nobody has measured the size of the report on a real corpus, and no such corpus exists.

**One claim is unmeasured**, as [principle 11](../spec/00-vision-and-scope.md#design-principles) requires. Publishing the read set should let a gate skip a full re-run on most merges. The instrument is the fraction of merges whose read set the other side never touched. The 55% and the 38% above are the closest thing to a prior that we have.

## Consequences for the specification

Twelve changes follow, and all are applied.

| Where | Change |
|---|---|
| [Spec 2](../spec/02-taxonomy-model.md#language-is-declared-not-assumed) | The language regime carries a retired-term lexicon: term, required reason, optional replacement |
| [Spec 2](../spec/02-taxonomy-model.md#language-is-declared-not-assumed) | A new lexicon entry breaks the `consequence` dimension, so it forces a major and rides the migration state |
| [Spec 3](../spec/03-authoring-and-lifecycle.md#voice) | Posture per voice category is decided by fixability, not by precision. A rewrite-remediated category is permanently advisory |
| [Spec 3](../spec/03-authoring-and-lifecycle.md#voice) | A voice check runs over author-owned text. Quotations, code, citations and generated blocks are outside it |
| [Spec 3](../spec/03-authoring-and-lifecycle.md#voice) | A voice escape hatch carries a reason from spec 4's closed set, and free prose beside it |
| [Spec 4](../spec/04-assurance-model.md#where-promotion-cannot-finish) | New rule: a control whose remediation needs judgment stays advisory permanently, whatever its false-positive rate |
| [Spec 4](../spec/04-assurance-model.md#a-verdict-is-about-one-state-of-the-corpus) | New section: validity is not preserved under merge, stated as a property of a verdict |
| [Spec 6](../spec/06-engine-architecture.md#ci-adapters) | A run reports its tree, its lock hash and its read set. The engine emits and never orders the landing |
| [Spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) | New section: the read set, the merge as an ordinary change, and the barrier cost of corpus scope |
| [Spec 12](../spec/12-check-layer.md#the-correctness-roots) | Sentence segmentation and the author-owned span join the parser's entry in the correctness roots |
| [Spec 10 §F.6](../spec/10-theoretical-foundations.md#f6-write-skew-names-the-anomaly-and-read-sets-detect-it) | New subsection and references: snapshot isolation, write skew, and serializable snapshot isolation |
| [Spec 11 §Q](../spec/11-adjacent-work.md#q--what-a-lexical-rule-gets-wrong-and-what-a-merge-queue-buys) | New section: controlled-language checkers, static-analysis tolerance, merge queues, incremental reuse, terminology migrations |

Two entries in [spec 9](../spec/09-open-questions.md) are rewritten as closed entries, and the glossary gains **read set**, **retired term** and **semantic conflict**.
