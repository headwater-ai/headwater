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

**The corpus MCP server.** The server exposes the graph as tools that an agent calls directly: `route`, `governing_docs_for_path`, `resolve_identifier`, `related`, `explain`, `check`. This is strictly better than to make an agent grep a corpus that it does not understand. The graph already knows the answers, and a tool call returns them at no context cost for exploration. The server is read-only by default. Writes, where enabled, go through the same validation as a human edit.

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

## The stop rules

These are explicit behaviors that an assistant who works in the corpus must show:

1. **No decision record without external evidence.** If no work item, commit, discussion, or measurement supports it, halt and ask. If the human confirms that none exists, record the document as `unevidenced` ([spec 3](03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two)). Never invent rationale — a fabricated *why* is worse than an admitted absence, because someone will cite it.
2. **No hand edits to a generated file.** Change the source and regenerate.
3. **No new shelf, kind, or facet invented in place.** Structural change is a taxonomy change: propose it, version it, migrate it.
4. **No duplication of a fact that exists elsewhere.** Link. If the target is hard to find, fix the routing — do not copy.

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

Probes run on a schedule, with a pinned model, deterministic probe selection, and a cost envelope. Results feed the adaptive layer of the [assurance model](04-assurance-model.md): a rule that measurably changes nothing is a candidate for deletion, and deletion is a success.

## Anti-overfitting

Probes are written against **behavior**, not against phrasings. A probe that passes because a rule file contains a magic sentence tests the sentence. The harness rotates and paraphrases probes deterministically per run. Probes are also reviewed for the failure mode where the corpus is tuned to the probe suite instead of to its readers.

## What we do not do

- No LLM in the validation path. Verdicts are deterministic and reproducible.
- No retrieval-augmented generation as the primary mechanism. The precision argument matters: the graph is exact, cheap, and explainable, and embeddings are none of those. But the deeper objection is that **RAG accumulates nothing**. Every query re-derives its answer from fragments chosen by similarity, and the synthesis is discarded. Ask again next week, and the work happens again. A governed corpus compiles knowledge *once* — validated, related, projected — and keeps it current, so retrieval reads a standing answer and does not reconstruct one. Chunking strategies exist to manage the structural loss that embedding introduces — we decline the loss rather than manage it. Embeddings remain permitted as a fallback for genuinely fuzzy lookup, and never as the authority. An external RAG system that consumes the corpus is a separate integration question, not a change to this.
- No agent-authored documents merged without review. The agent drafts and links. A human accepts.
