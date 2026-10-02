---
id: HW-PROBE-a-session-says-how-a-governs-line-reaches-a-document
status: current
status_since: 2026-10-03
summary: Many documents share the terms of this task and several still state the answer of a replaced ruling, so a session must find the one record in force among near copies.
last_verified: 2026-10-03
probe_category: navigability
expectation: answered
oracle: "none"
title: "A session says how a governs line reaches a document"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-SPEC-ai-integration
---

# A session says how a governs line reaches a document

## Task

An agent in this repository writes a new document. The document must declare a `governs` edge onto a source file, in its front matter. Can a command of the `headwater` tool write that line into the new document? Or must the line be typed by hand?

Answer on one line with one word, `command` or `hand`, and with nothing else. Your whole final message is that word.

## Expectation

The terminal answer is `command`, the one expected value of the closed set below.

```yaml
answers: [command, hand]
expected: [command]
```

**This probe has the distractors-at-scale shape (#1472).** The record in force is [HW-DR-0104](../decisions/0104-an-agent-writes-governs-traces-to-and-cited-in-through-the-verb-and-the-review-of-its-pull-request-is-the-acceptance.md), at `status: current`. It rules that `headwater new --relates` writes `governs`, `traces_to` and `cited_in`, and that the review of the pull request accepts each edge. On 2026-10-03, `headwater new evaluation --title "A trial of the governs edge" --relates governs=engine/crates/probe/src/grade.rs` exited 0 in a copy of this repository and wrote the edge.

**Near copies state the other answer.** [HW-DR-0083](../decisions/0083-governs-and-traces-to-are-created-by-an-agent-because-a-session-proposes-the-line-and-a-person-types-it.md) is `superseded` by HW-DR-0104. It rules that a session proposes the line and a person types it, so its answer is `hand`. Several documents that are not superseded still state its answer, because they cite it and were not updated. A session that answers from any of them answers `hand`, and the grade finds that answer wrong.

**The count, and how it was measured.** A near copy here is a document that holds the two key terms of the task, `governs` and `created_by`. The tree is a present tree sealed for the five probes of #1472 with `tools/probe/seal.sh`, on 2026-10-03. In that tree, `grep -r -l 'created_by' docs | wc -l` printed 52, and 39 of those files also hold `governs`. A search of the 39 for a sentence that says no verb writes the edge, or that a person types it, found these documents, which state the answer `hand` about `governs`:

- HW-DR-0083 itself, which is `superseded`.
- [HW-DR-0074](../decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md), at `current`: "a session proposes the line and a person types it".
- [The orchestration architecture](../process/specs/orchestration-architecture.md), at `current`: "That relation is `created_by: agent`, and no verb writes it".
- [HW-OBL-0001](../obligations/0001-the-promotion-fix-has-no-reading-of-the-assisted-fraction.md), at `current`: "No verb writes either, so every half of both is hand entry".
- [HW-OBL-0105](../obligations/0105-nothing-plays-the-hook-role-that-two-relations-name.md), at `discharged`: "No verb of this engine writes either edge".
- The generated index `docs/decisions/README.md` and the tombstone `docs/spec/09-open-questions.md`, which each print the summary of HW-DR-0083 beside the mark `superseded`.

So 39 near copies share the key terms, and 7 of them state the other answer. HW-DR-0104 and [the contract of `headwater new`](../interfaces/headwater-new.md) state the expected one. The contract says that `--relates` accepts a relation that declares `created_by: agent`.

**The distractors are not built for this probe.** Each one is a document of this corpus that went stale when HW-DR-0104 landed. A campaign over this probe therefore measures the corpus as it is. If a later change repairs these documents, the count above falls, and this probe must count again before it runs.
