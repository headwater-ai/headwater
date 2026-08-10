# 5 — AI integration

Agents now read documentation more often than people do. The failure mode of an agent is not that it skims. When context is missing, an agent invents, and it invents with confidence. The corpus is designed for that reader.

## The four moments

An assistant touches documentation at four different moments. Each moment needs a different mechanism. Most "AI-ready docs" efforts do not separate the moments, and that is why they give less than they promise.

| Moment | Question | Mechanism |
|---|---|---|
| **Intent** | "What governs the thing I am about to do?" | Routing — resolve a task description to governing documents *before* the agent reads any file |
| **Read** | "What rules apply to this file?" | Path-scoped rule loading, started when the file opens |
| **Write** | "Did what I just changed invalidate a document?" | Impact detection on edit, and metadata backfill on creation |
| **Review** | "Is this change consistent with the corpus?" | Change-scoped checks, and agent review against the governing set |

### Intent-time routing

This is the highest-value moment, and it is the one most often missed. An agent that reads the governing standard *before* it writes code produces different code. An agent that finds the standard afterwards produces an apology.

Routing matches the **declared purpose** of each kind against the intent of the task before it matches any text. "why is it like this?" resolves to kinds that serve `rationale`, and "what does it do?" resolves to kinds that serve `behavior`. That is a search over the corpus's intentional structure, not over its prose. It is cheaper and more precise than lexical ranking, and it degrades gracefully because purposes are a small closed set. Lexical ranking then orders results *within* the matched purpose, and derived reading precedence breaks ties ([spec 2](02-taxonomy-model.md#reading-precedence-is-derived)). Where two linked documents both match, routing offers a nucleus before its satellite. It offers a successor before the document that it superseded, and a governing document before the one that it governs.

In all other respects, routing is a deterministic projection over the graph — summaries, facets, relations, and code-path anchors — not a semantic search. A task description resolves to a ranked, budget-capped set of **pointers**: paths and one-line summaries, never content. Pointers keep the corpus as the single source of truth and keep the context cost near zero.

Routing is **confidence-gated and fails open**: below the threshold it says nothing. A wrong pointer costs more than a missing one, because an agent will follow it.

Silence for weak scent and silence for a withheld document are different facts, and routing keeps them apart. Where a route runs over a filtered [export profile](06-engine-architecture.md#an-export-profile-carries-a-filter), a document that the filter removed is reported at the profile's declared tombstone grain. It never falls under the confidence gate, because nothing about it is uncertain.

A third case joins those two. A pointer to a document with the `asserted` [warrant](01-conceptual-model.md#warrant) states that warrant beside the summary. Nobody accepted the document, and an agent that follows the pointer has to know that before it reads. To offer such a pointer silently is the failure that [Q15](09-open-questions.md#q15--a-synthesized-content-tier) exists to prevent, reached through our own routing surface.

#### Scent is the thing being engineered

Information-foraging theory names what routing actually trades in: **scent** — the proximal cue that predicts distal value. A reader or an agent follows scent, and abandons a patch when the scent weakens. Thus scent quality, not corpus quality, decides whether anything is found. A perfect document with a vague summary is invisible.

That makes the `summary` facet the corpus's entire scent surface, and the surface is measured rather than assumed:

| Measure | What it catches |
|---|---|
| **Distinctiveness** | A summary that shares no discriminating term with its siblings cannot separate them. The shelf reads as undifferentiated |
| **Non-restatement** | A summary that only rephrases the title carries no information that the path did not |
| **Length band** | Too short to discriminate, or too long to scan in a pointer list |
| **Routing precision** | Of the pointers offered, how often the agent opened the top one, and it sufficed |
| **Abandonment** | Pointers offered and never opened — scent that promised and did not pay |

The first three are static and run as advisory checks. The last two come from probe transcripts. Probe transcripts are the only place where the corpus can observe foraging behavior rather than infer it.

The confidence gate is a scent threshold. The engine stays silent when the strongest available cue is weak, because a cue that misleads is worse than a missing cue. That is the same asymmetry stated in foraging terms, and it is why the gate errs toward silence.

One caution on the word *entire* above. The summary is the whole scent surface for routing, where a pointer list answers a query and no referring edge exists. A reader who follows a relation meets a different proximal cue first, which is the referring text. [Q20](09-open-questions.md#q20--where-scent-lives) asks whether a relation should carry a cue of its own, and until it is settled this section describes one of the two moments.

```
headwater route "add rate limiting to the ingest API"
  docs/standards/api-design.md      — API surface conventions, versioning, error shapes
  docs/specifications/ingest/...    — ingest service behavior and SLOs
  docs/decisions/dr-0031.md         — why throttling is applied at the edge, not per-service
```

### Read-time rule loading

Rules load additively when an agent opens a file that matches. They are **generated projections of canonical documents**, not a parallel copy. A rule file is a short pointer to the standard that it derives from. It carries only enough content to make the agent stop and read the source. CI checks the regeneration, so a standard and its rule cannot disagree.

Rules obey a **size budget** declared on the kind or projection that produces them. The engine meters always-loaded context on every request, so it enforces budgets per file and in aggregate. An over-budget rule is a finding that names the layer that must trim it. The budget is not optional: an agent-facing projection with no applicable budget fails `taxonomy validate` ([spec 2](02-taxonomy-model.md#the-meta-schema)).

When a budget binds, the engine **drops satellites before nuclei**. A generated rule is a satellite of the standard that it derives from. To drop the standard and keep its teaser is exactly backwards, and without declared nuclearity the engine has no basis for the choice. Relation nuclearity makes context pruning a structural operation rather than a heuristic one.

### Write-time hooks

- **Backfill** — at creation time, when it is cheap, the engine completes the front matter, identifier, and required sections of a new document against its kind.
- **Impact detection** — a document can declare that it governs code. An edit to that code raises an advisory prompt that names the specific documents at risk. The prompt is advisory on purpose: a blocking gate here trains people to write "no doc impact" reflexively, and that destroys the signal.

### Review-time checks

The same engine runs, scoped to the change. It limits findings to the touched paths, and to the documents that relate to them through the graph. It renders the findings as review comments that carry their remediation.

## Agent surfaces

Beyond files, there are three richer surfaces:

**The corpus MCP server.** The server exposes the graph as tools that an agent calls directly: `route`, `governing_docs_for_path`, `resolve_identifier`, `related`, `explain`, `check`. This is strictly better than to make an agent grep a corpus that it does not understand. The graph already knows the answers, and a tool call returns them at no context cost for exploration.

#### What the server may do, and the axis that decides it

"Read-only or not" is the wrong question, and [Q7](09-open-questions.md#q7--scope-of-the-mcp-surface) asked it for a while. `headwater check --fix` writes files today, in a human's working tree, and the result lands in a diff that the human commits. A hosted server that commits to a branch produces the same bytes with no review at any point. The axis is **whose review the result passes through**, not whether bytes move.

| Class | Tools | Ships | Why |
|---|---|---|---|
| **Query** | `route`, `governing_docs_for_path`, `resolve_identifier`, `related`, `explain`, `check` | first release | It changes nothing |
| **Working-tree write** | `new`, `fix` | first release, and off by default per server | The human reviews at commit, and the [fixability bar](12-check-layer.md#fixability) forbids a judgment-bearing patch |
| **Landed write** | a commit, a push, a merge, a server that writes to a repository | never | Acceptance is a human act ([spec 3](03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed)), and no forge is privileged in the core |

The third row is a refusal and not a deferral. A server-side commit produces a document with no `accepted_by`, or with an invented one. The provenance model forbids it before any judgment about trust arrives. Headwater instead emits what a change proposal needs: findings, patches, and a task list. An adapter opens the proposal, with the credential that its operator granted it. That is the same boundary that keeps the engine out of the merge-queue business ([Q21](09-open-questions.md#q21--terminological-succession-and-validity-under-merge)).

Working-tree writes stay off by default, because a client may connect to a checkout that the user did not intend to change. The opt-in is per server.

**The annotation is not the enforcement.** The protocol lets a server declare that a tool only reads. It also states that a client must not treat that declaration from an untrusted server as a guarantee. Headwater annotates its tools correctly and relies on something else. Where a class of tool is off, the server does not register it, so no handler exists to call. A property that a caller reads off a tool list is a hint. A property with no code path behind it is a guarantee.

**A write tool is a disclosure channel, and that is why the third row is a refusal rather than a preference.** The published attacks on this protocol put attacker text into a model's context at discovery time, before any tool runs. A confirmation prompt at each call therefore never sees them. What the attacks then need is an actuator. The observed case against a widely deployed server used a write tool as the exit. The agent read private content, then published it by opening a proposal on a public repository. A server that cannot land a write lends an injected instruction nothing. That argument is about [Q17](09-open-questions.md#q17--governed-access-and-the-solution-layer) as much as about this entry.

**The server applies no filter to a corpus that its reader already holds.** It runs in-process against a checkout, so the reader has every byte. A filter there would control one reading path while the bytes stay readable along another. [Spec 11 §L.6](11-adjacent-work.md#l6-a-filter-in-the-tool-layer-is-advisory-and-the-documentation-says-so) records that failure in a shipped tool. A server that serves a reader who holds no checkout serves exactly one declared [export profile](06-engine-architecture.md#an-export-profile-carries-a-filter) and never mixes the two sources.

**Authoring skills.** Packaged procedures for the work that bears judgment: to draft a decision record, to run a corpus-wide sweep, to propose a taxonomy change. Skills carry the doctrine that an agent needs, and they call the deterministic engine for everything mechanical. Thus the LLM does the reasoning and never the arithmetic.

This is also the system's answer to the capture-cost problem that killed every prior design-rationale tool ([spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric)). An agent can draft a decision record from evidence that already sits in the commit, the ticket, and the conversation. That moves the cost off the author, who was never the beneficiary. The claim is falsifiable, and it is tracked as the assisted fraction. If we enable agent authoring and the fraction does not rise, the argument fails, and we must then expect the adoption curve of gIBIS.

**A maintainer subagent.** A context-isolated agent that owns documentation upkeep across a change: which documents this touched, what is now stale, what decision lacks evidence. It runs with a scoped instruction subset — its own bounded context — and does not inflate the always-on prompt of every session.

### The machinery is the adoption model

These surfaces read as conveniences, and they are not: they are the supply chain for the graph. Everything distinctive — routing, lifecycle propagation, impact detection, absence findings — degrades together when edge density stays low. The traceability literature that the design cites ([spec 10](10-theoretical-foundations.md#b5-traceability-information-models--our-idea-has-a-name-and-a-literature)) says that author-maintained links decay because the payer is not the beneficiary. A release can omit the authoring skills, the hooks that invoke them at intent, write, and review time, and the telemetry that watches them. Such a release leaves its central bet untested. That is why [spec 0](00-vision-and-scope.md#what-we-build) puts them in the first release rather than after it.

Three commitments make the machinery governable rather than merely present:

- **Judgment stays human.** Skills propose kinds, relations, summaries, and lifecycle changes from the artifacts already in the change. A human accepts. A skill that cannot justify a classification refuses it and does not guess ([the stop rules](#the-stop-rules)).
- **Every required datum is attributed.** `created_by` and the assisted fraction say whether the machinery actually maintains the graph. An assisted fraction that *falls* is an assurance finding — evidence that the adoption model fails — not a dashboard curiosity.
- **Generated context is checked before it is trusted.** Agent-facing projections carry freshness policy and size budgets like anything else, and the engine verifies them before they load. Stale governance instructions delivered efficiently are an efficient source of wrong behavior.

## What structured knowledge buys

The reason to build any of this is machine behavior that is more predictable. We must state the claim accurately, or it will shape the work wrongly.

**A governed corpus does not make a language model deterministic.** Sampling is stochastic. Identical prompts produce different outputs, and no amount of schema changes that. Anyone who claims otherwise has something to sell.

What it does buy:

| Mechanism | Effect |
|---|---|
| **Ambiguity removal** | Fewer legitimate readings of the input, so fewer defensible-but-divergent outputs. Variance narrows. It does not vanish |
| **Oracles** | We can check output against a declared expectation ([spec 2](02-taxonomy-model.md#contract-sidecars-the-specification-as-oracle)) rather than judge it by eye |
| **Attribution** | When output deviates, the artifact that licensed it is identifiable, so the fix lands on the corpus or the prompt rather than on a hunch |
| **Reproducible comparison** | A pinned corpus and a pinned model give a baseline, and we can diff later runs against it |

The achievable target is **bounded, auditable non-determinism**: output that varies within a space that the corpus defines, deviations visible, causes attributable.

That is not a lesser goal, and it sets the investment priority. Effort belongs in oracles and traceability — checkable expectations, and citation of what licensed each decision. Effort does not belong in prompt engineering that tries to coax a model to repeat itself. The first compounds and is measurable. The second is a treadmill.

### Generated artifacts cite what licensed them

Any artifact that an agent produces under the corpus's direction cites the identifiers of the artifacts that governed it. Examples are a document, a generated test, and an implementation written against a specification. The citation is inline, at the point of the decision.

```python
# per REQ-INGEST-014 (specifications/ingest/parser/functional.md)
# version 5 and 6 swap these two fields
```

This is cheap, and it survives refactoring better than a link in a commit message. It changes "why does the code do this?" from an investigation into a lookup. It is also what makes attribution work in practice. When generated output is wrong, the citation says whether the corpus misled the agent or the agent ignored the corpus. Those have opposite fixes, and without the citation no one can tell them apart.

Where sources conflict, the agent cites **both** and flags the conflict ([spec 2](02-taxonomy-model.md#disagreement-is-adjudicated-not-ranked)). If an agent resolves a contradiction silently, it destroys the evidence that one existed.

Where a human already settled the conflict, the agent cites the **adjudication**. A settled disagreement is a decision that `overrides` the document whose effect it displaces, and derived reading precedence puts the successor first. So the agent follows a ruling that a named person made, rather than making the same ruling again with no record ([Q18](09-open-questions.md#q18--recording-adjudicated-disagreements)).

## The stop rules

These are explicit behaviors that an assistant who works in the corpus must show:

1. **No decision record without external evidence.** If no work item, commit, discussion, or measurement supports it, halt and ask. If the human confirms that none exists, record the document as `unevidenced` ([spec 3](03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two)). Never invent rationale — a fabricated *why* is worse than an admitted absence, because someone will cite it.
2. **No hand edits to a generated file.** Change the source and regenerate.
3. **No new shelf, kind, or facet invented in place.** Structural change is a taxonomy change: propose it, version it, migrate it.
4. **No duplication of a fact that exists elsewhere.** Link. If the target is hard to find, fix the routing — do not copy.
5. **No self-acceptance.** An agent never writes `accepted_by`, and never changes a [warrant](01-conceptual-model.md#warrant) to `accepted`. It drafts, and it marks what it drafted as `asserted` where no human read the result. A stamp that an agent applies to its own output is the mark that Serena's onboarding pass never wrote, with a false name attached ([spec 11 §L.5](11-adjacent-work.md#l5-onboarding-ships-the-synthesized-tier-and-marks-nothing)).

These are the rules that a capable model breaks most readily under pressure to be helpful. That is exactly why we state them as stop conditions rather than preferences.

## Measuring whether any of this works

Instruction files are written on the assumption that the assistant reads and follows them. That assumption is testable. When it is not tested, it is usually optimistic.

A **probe suite** runs scenarios against the corpus in a controlled session. It grades behavior from the tool-call transcript, not from the model's self-report. The grader re-derives every verdict from what the agent actually opened and did. Probe categories:

| Category | Asks |
|---|---|
| Discovery | Does the agent find the governing document at all? |
| Sufficiency | After it finds the document, does it have enough to act correctly? |
| Fidelity | Does the derived rule teach the same thing as its canonical source? |
| Navigability | Can it get from a code path to the governing document, and back? |
| Consistency | Same question, different phrasings — same answer? |
| Counterfactual | Does removal of the context actually change behavior? |

The counterfactual category is the one that matters most, and it is the one most often skipped. An A/B run — corpus present versus absent — is the only evidence that the instruction surface earns its context cost. Without it, "the AI reads our docs" is a belief.

Two constraints protect the instrument, and [spec 11 §M](11-adjacent-work.md#m--what-the-survey-shows-as-a-whole-convergence-is-not-evidence) records why both are needed. Every adjacent project that claims this benefit either graded itself or skipped the counterfactual. The literature shows what a weak grader does to a result. A systematic comparison of RAG and graph-based RAG reached the opposite conclusion to the original study. The cause was the grading method rather than the systems. The same authors found that an LLM judge reverses its verdict when the order of two candidates is reversed.

- **The grader is never the system under test.** A verdict comes from the tool-call transcript and a declared expectation. No model judges whether the corpus helped, and no probe accepts an agent's account of its own behavior.
- **A published claim carries its counterfactual.** Corpus present against corpus absent, with a pinned model and a recorded probe selection. A claim with no such pair is reported as unmeasured rather than as supported, and the [evidence register](04-assurance-model.md) carries that mark.

Probes run on a schedule, with a pinned model, deterministic probe selection, and a cost envelope. Results feed the adaptive layer of the [assurance model](04-assurance-model.md): a rule that measurably changes nothing is a candidate for deletion, and deletion is a success.

## Anti-overfitting

Probes are written against **behavior**, not against phrasings. A probe that passes because a rule file contains a magic sentence tests the sentence. The harness rotates and paraphrases probes deterministically per run. Probes are also reviewed for the failure mode where the corpus is tuned to the probe suite instead of to its readers.

## What we do not do

- No LLM in the validation path. Verdicts are deterministic and reproducible.
- No retrieval-augmented generation as the primary mechanism. The precision argument matters: the graph is exact, cheap, and explainable, and embeddings are none of those. But the deeper objection is that **RAG accumulates nothing**. Every query re-derives its answer from fragments chosen by similarity, and the synthesis is discarded. Ask again next week, and the work happens again. A governed corpus compiles knowledge *once* — validated, related, projected — and keeps it current, so retrieval reads a standing answer and does not reconstruct one. Chunking strategies exist to manage the structural loss that embedding introduces — we decline the loss rather than manage it. A third objection, which [spec 11 §L.2](11-adjacent-work.md#l2-prefer-references-to-search--a-third-argument-against-retrieval) takes from Serena's memory design, is that every retrieval method carries both error modes at once. Lexical and semantic search each return false positives and false negatives, and a named edge returns neither. Retrieval is therefore a source of variance, and a declared edge removes it. Embeddings remain permitted as a fallback for genuinely fuzzy lookup, and never as the authority. An external RAG system that consumes the corpus is a separate integration question, not a change to this.
- No agent-authored documents merged without review. The agent drafts and links. A human accepts.
