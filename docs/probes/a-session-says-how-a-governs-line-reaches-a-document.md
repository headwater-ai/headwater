---
id: HW-PROBE-a-session-says-how-a-governs-line-reaches-a-document
status: superseded
status_since: 2026-10-10
summary: Many documents share the terms of this task, and a superseded record, a discharged one and two generated pages still state the answer of a replaced ruling, so a session must find the one record in force among near copies.
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
  superseded_by:
    - HW-PROBE-a-session-says-whether-a-command-or-a-hand-writes-a-governs-line
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

**Near copies state the other answer.** [HW-DR-0083](../decisions/0083-governs-and-traces-to-are-created-by-an-agent-because-a-session-proposes-the-line-and-a-person-types-it.md) is `superseded` by HW-DR-0104. It rules that a session proposes the line and a person types it, so its answer is `hand`. Three other documents still state its answer. None of them is a current record: one is a discharged obligation, and two are generated pages that print the summary of HW-DR-0083. A session that answers from any of them answers `hand`, and the grade finds that answer wrong.

**The count, and how it was measured.** A near copy here is a document that holds the two key terms of the task, `governs` and `created_by`. The tree is a present tree sealed for the five harder probes of #1472 with `tools/probe/seal.sh`. In that tree, `grep -r -l 'created_by' docs | wc -l` prints the documents that hold the first term, and `xargs grep -l governs` over that list prints the near copies. Then `grep -n -i -E "no verb (of this engine )?writes|person types|is hand entry"` over the near copies prints each candidate sentence. A person reads each candidate and keeps the documents that state the answer `hand` about `governs`.

**The recount of 2026-10-03, after #1642.** #1642 repaired three current documents that stated the answer `hand`: [HW-DR-0074](../decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md), [the orchestration architecture](../process/specs/orchestration-architecture.md) and [HW-OBL-0001](../obligations/0001-the-promotion-fix-has-no-reading-of-the-assisted-fraction.md). Each one now cites HW-DR-0104. On the tree of that change, sealed as above, the first search printed 52 documents, and 39 of them are near copies. These 4 of the 39 state the answer `hand` about `governs`:

- HW-DR-0083 itself, which is `superseded`.
- [HW-OBL-0105](../obligations/0105-nothing-plays-the-hook-role-that-two-relations-name.md), at `discharged`: "No verb of this engine writes either edge".
- The generated index `docs/decisions/README.md` and the tombstone `docs/spec/09-open-questions.md`, which each print the summary of HW-DR-0083 beside the mark `superseded`.

The other candidate sentences are about another relation or another artifact. `docs/taxonomies/brd-prd/doctrine.md` says that a person types the line of a relation between a business need and a product document, and it cites HW-DR-0083. It does not name `governs`, so the count leaves it out. HW-DR-0104 and [the contract of `headwater new`](../interfaces/headwater-new.md) state the expected answer. The contract says that `--relates` accepts a relation that declares `created_by: agent`.

**The first count, before #1642.** On a tree sealed the same way earlier on 2026-10-03, the same method found 52 documents, 39 near copies, and 7 of the 39 that state the answer `hand`. The three documents that #1642 repaired were the difference.

**The distractors are not built for this probe.** Each one is a document of this corpus that went stale when HW-DR-0104 landed. A campaign over this probe therefore measures the corpus as it is. If a later change repairs more of these documents, the count above falls, and this probe must count again before it runs.

**This is the probe that the campaign of 2026-10-03 ran, and it is superseded (#1709).** Its key held against HW-DR-0104. The recount above missed one current near copy: [the governs-edges evaluation](../evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md) said that a person or an agent types the line. #1709 repaired that sentence, and the repair changed the tree that this probe measures. So [its successor](a-session-says-whether-a-command-or-a-hand-writes-a-governs-line.md) asks the same task with the same key over the repaired tree. The recorded results under `docs/probe-results/` and `docs/probe-runs/` stand, and nobody rewrites them.
