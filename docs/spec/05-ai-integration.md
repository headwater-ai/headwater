# 5 — AI integration

Documentation is now read more often by agents than by people, and an agent's
failure mode is not skimming — it is confident invention when context is missing.
The corpus is designed for that reader.

## The four moments

An assistant touches documentation at four distinct moments. Each needs a different
mechanism, and conflating them is why most "AI-ready docs" efforts underdeliver.

| Moment | Question | Mechanism |
|---|---|---|
| **Intent** | "What governs the thing I am about to do?" | Routing — resolve a task description to governing documents *before* any file is read |
| **Read** | "What rules apply to this file?" | Path-scoped rule loading, triggered by the file being opened |
| **Write** | "Did what I just changed invalidate a document?" | Impact detection on edit, and metadata backfill on creation |
| **Review** | "Is this change consistent with the corpus?" | Change-scoped checks, and agent review against the governing set |

### Intent-time routing

The highest-value moment, and the one usually missed. An agent that reads the
governing standard *before* writing code produces different code; an agent that
finds it afterwards produces an apology.

Routing matches the **declared purpose** of each kind against the intent of the task
before it matches any text: "why is it like this?" resolves to kinds serving
`rationale`, "what does it do?" to kinds serving `behaviour`. That is a search over
the corpus's intentional structure rather than over its prose — cheaper and more
precise than lexical ranking, and it degrades gracefully because purposes are a
small closed set. Lexical ranking then orders results *within* the matched purpose,
and derived reading precedence breaks ties
([spec 2](02-taxonomy-model.md#reading-precedence-is-derived)): where two linked
documents both match, a nucleus is offered before its satellite and a successor
before what it superseded.

Routing is otherwise a deterministic projection over the graph — summaries, facets,
relations, and code-path anchors — not a semantic search. A task description resolves to a
ranked, budget-capped set of **pointers**: paths and one-line summaries, never
content. Pointers keep the corpus as the single source of truth and keep the
context cost near zero.

It is **confidence-gated and fails open**: below the threshold it says nothing. A
wrong pointer costs more than a missing one, because an agent will follow it.

#### Scent is the thing being engineered

Information-foraging theory names what routing actually trades in: **scent** — the
proximal cue that predicts distal value. A reader or agent follows scent and
abandons a patch when it weakens, so scent quality, not corpus quality, decides
whether anything is found. A perfect document with a vague summary is invisible.

That makes the `summary` facet the corpus's entire scent surface, and it is
measured rather than assumed:

| Measure | What it catches |
|---|---|
| **Distinctiveness** | A summary sharing no discriminating term with its siblings cannot separate them; the shelf reads as undifferentiated |
| **Non-restatement** | A summary that only rephrases the title carries no information the path did not |
| **Length band** | Too short to discriminate, or too long to scan in a pointer list |
| **Routing precision** | Of the pointers offered, how often the top one was opened and sufficed |
| **Abandonment** | Pointers offered and never opened — scent that promised and did not pay |

The first three are static and run as advisory checks. The last two come from probe
transcripts, which is the only place the corpus can observe foraging behaviour
rather than infer it.

The confidence gate is a scent threshold: the engine stays silent when the strongest
available cue is weak, because a misleading cue is worse than an absent one. That is
the same asymmetry stated in foraging terms, and it is why the gate errs toward
silence.

```
docgov route "add rate limiting to the ingest API"
  docs/standards/api-design.md      — API surface conventions, versioning, error shapes
  docs/specifications/ingest/...    — ingest service behaviour and SLOs
  docs/decisions/dr-0031.md         — why throttling is applied at the edge, not per-service
```

### Read-time rule loading

Rules load additively when an agent opens a matching file. They are **generated
projections of canonical documents**, not a parallel copy: a rule file is a short
pointer to the standard it derives from, carrying only enough content to make the
agent stop and read the source. Regeneration is checked in CI, so a standard and
its rule cannot disagree.

Rules obey a **size budget** declared on the kind or projection that produces
them. Always-loaded context is metered on every request, so budgets are enforced
per file and in aggregate, and an over-budget rule is a finding that names the
layer responsible for trimming it. The budget is not optional: an agent-facing
projection with no applicable budget fails `taxonomy validate`
([spec 2](02-taxonomy-model.md#the-meta-schema)).

When a budget binds, **satellites are dropped before nuclei**. A generated rule is a
satellite of the standard it derives from; dropping the standard and keeping its
teaser is exactly backwards, and without declared nuclearity the engine has no basis
for choosing. Relation nuclearity makes context pruning a structural operation
rather than a heuristic one.

### Write-time hooks

- **Backfill** — a newly created document gets its front matter, identifier, and
  required sections completed against its kind, at creation time, when it is cheap.
- **Impact detection** — an edit to code that a document declares it governs raises
  an advisory prompt naming the specific documents at risk. Advisory on purpose: a
  blocking gate here trains people to write "no doc impact" reflexively, which
  destroys the signal.

### Review-time checks

The same engine, scoped to the change: findings limited to touched paths and to the
documents related to them through the graph, rendered as review comments carrying
their remediation.

## Agent surfaces

Beyond files, three richer surfaces:

**The corpus MCP server.** The graph exposed as tools an agent calls directly:
`route`, `governing_docs_for_path`, `resolve_identifier`, `related`, `explain`,
`check`. This is strictly better than making an agent grep a corpus it does not
understand — the graph already knows the answers, and a tool call returns them
without burning context on exploration. Read-only by default; writes, where enabled,
go through the same validation as a human edit.

**Authoring skills.** Packaged procedures for the judgment-bearing work: drafting a
decision record, running a corpus-wide sweep, proposing a taxonomy change. Skills
carry the doctrine an agent needs and call the deterministic engine for everything
mechanical, so the LLM does the reasoning and never the arithmetic.

This is also the system's answer to the capture-cost problem that killed every prior
design-rationale tool ([spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric)).
An agent that drafts a decision record from evidence already sitting in the commit,
the ticket, and the conversation moves the cost off the author — who was never the
beneficiary. The claim is falsifiable and is tracked as the assisted fraction: if
enabling agent authoring does not raise it, the argument fails and we should expect
gIBIS's adoption curve.

**A maintainer subagent.** A context-isolated agent that owns documentation upkeep
across a change: which documents this touched, what is now stale, what decision
lacks evidence. It runs with a scoped instruction subset — its own bounded context —
rather than inflating every session's always-on prompt.

## What structured knowledge buys

The reason to build any of this is more predictable machine behaviour, and the claim
has to be stated accurately or it will shape the work wrongly.

**A governed corpus does not make a language model deterministic.** Sampling is
stochastic, identical prompts produce different outputs, and no amount of schema
changes that. Anyone claiming otherwise is selling something.

What it does buy:

| Mechanism | Effect |
|---|---|
| **Ambiguity removal** | Fewer legitimate readings of the input, so fewer defensible-but-divergent outputs. Variance narrows; it does not vanish |
| **Oracles** | Output can be checked against a declared expectation ([spec 2](02-taxonomy-model.md#contract-sidecars-the-specification-as-oracle)) rather than judged by eye |
| **Attribution** | When output deviates, the artefact that licensed it is identifiable, so the fix lands on the corpus or the prompt rather than on a hunch |
| **Reproducible comparison** | A pinned corpus and a pinned model give a baseline later runs can be diffed against |

The achievable target is **bounded, auditable non-determinism**: output varying
within a space the corpus defines, deviations visible, causes attributable.

That is not a lesser goal, and it sets the investment priority. Effort belongs in
oracles and traceability — checkable expectations, and citation of what licensed each
decision — not in prompt engineering aimed at coaxing a model into repeating itself.
The first compounds and is measurable; the second is a treadmill.

### Generated artefacts cite what licensed them

Any artefact an agent produces under the corpus's direction — a document, a
generated test, an implementation written against a specification — cites the
identifiers of the artefacts that governed it, inline, at the point of the decision.

```python
# per REQ-INGEST-014 (specifications/ingest/parser/functional.md)
# version 5 and 6 swap these two fields
```

This is cheap, it survives refactoring better than a link in a commit message, and
it converts "why does the code do this?" from an investigation into a lookup. It is
also what makes attribution work in practice: when generated output is wrong, the
citation says whether the corpus misled the agent or the agent ignored the corpus.
Those have opposite fixes, and without the citation they are indistinguishable.

Where sources conflict, the agent cites **both** and flags the conflict
([spec 2](02-taxonomy-model.md#disagreement-is-adjudicated-not-ranked)).
An agent silently resolving a contradiction destroys the evidence that one existed.

## The stop rules

Explicit behaviours an assistant working in the corpus must exhibit:

1. **No decision record without external evidence.** If no work item, commit,
   discussion, or measurement supports it, halt and ask. If the human confirms none
   exists, log a registered gap. Never manufacture rationale — a fabricated *why*
   is worse than an admitted absence, because it will be cited.
2. **No hand-editing a generated file.** Change the source and regenerate.
3. **No new shelf, kind, or facet invented in place.** Structural change is a
   taxonomy change: propose it, version it, migrate it.
4. **No duplication of a fact that exists elsewhere.** Link. If the target is hard
   to find, fix the routing — do not copy.

These are the rules a capable model breaks most readily under pressure to be
helpful, which is exactly why they are stated as stop conditions rather than
preferences.

## Measuring whether any of this works

Instruction files are written on the assumption that the assistant reads and follows
them. That assumption is testable, and untested it is usually optimistic.

A **probe suite** runs scenarios against the corpus in a controlled session and
grades behaviour from the tool-call transcript, not from the model's self-report —
the grader re-derives every verdict from what the agent actually opened and did.
Probe categories:

| Category | Asks |
|---|---|
| Discovery | Does the agent find the governing document at all? |
| Sufficiency | Having found it, does it have enough to act correctly? |
| Fidelity | Does the derived rule teach the same thing as its canonical source? |
| Navigability | Can it get from a code path to the governing document, and back? |
| Consistency | Same question, different phrasings — same answer? |
| Counterfactual | Does removing the context actually change behaviour? |

The counterfactual category is the one that matters most and is most often skipped.
An A/B run — corpus present versus absent — is the only evidence that the
instruction surface earns its context cost. Without it, "the AI reads our docs" is a
belief.

Probes run scheduled, with a pinned model, deterministic probe selection, and a
cost envelope. Results feed the adaptive layer of the [assurance
model](04-assurance-model.md): a rule that measurably changes nothing is a
candidate for deletion, and deletion is a success.

## Anti-overfitting

Probes are written against **behaviour**, not against phrasings. A probe that passes
because a rule file contains a magic sentence tests the sentence. Probes are
rotated, paraphrased deterministically per run, and reviewed for the failure mode
where the corpus is tuned to the probe suite instead of to its readers.

## What we do not do

- No LLM in the validation path. Verdicts are deterministic and reproducible.
- No retrieval-augmented generation as the primary mechanism. The precision argument
  matters — the graph is exact, cheap, and explainable where embeddings are none of
  those — but the deeper objection is that **RAG accumulates nothing**. Every query
  re-derives its answer from fragments chosen by similarity, and the synthesis is
  discarded; ask again next week and the work happens again. A governed corpus
  compiles knowledge *once* — validated, related, projected — and keeps it current,
  so retrieval reads a standing answer rather than reconstructing one. Chunking
  strategies exist to manage the structural loss that embedding introduces; we
  decline the loss instead of managing it. Embeddings remain acceptable as a
  fallback for genuinely fuzzy lookup, and never as the authority. An external RAG
  system consuming the corpus is a separate integration question, not a change to
  this.
- No agent-authored documents merged without review. The agent drafts and links;
  a human accepts.
