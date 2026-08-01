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

Routing is a deterministic projection over the graph — summaries, facets, relations,
and code-path anchors — not a semantic search. A task description resolves to a
ranked, budget-capped set of **pointers**: paths and one-line summaries, never
content. Pointers keep the corpus as the single source of truth and keep the
context cost near zero.

It is **confidence-gated and fails open**: below the threshold it says nothing. A
wrong pointer costs more than a missing one, because an agent will follow it.

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

Rules obey a **size regime**. Always-loaded context is metered on every request, so
budgets are enforced per file and in aggregate, and an over-budget rule is a finding
that names the layer responsible for trimming it.

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

**A maintainer subagent.** A context-isolated agent that owns documentation upkeep
across a change: which documents this touched, what is now stale, what decision
lacks evidence. It runs with a scoped instruction subset — its own bounded context —
rather than inflating every session's always-on prompt.

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
- No vector index as the primary retrieval mechanism. The graph is precise, cheap,
  and explainable; embeddings are at most a fallback for genuinely fuzzy lookup,
  and never the authority.
- No agent-authored documents merged without review. The agent drafts and links;
  a human accepts.
