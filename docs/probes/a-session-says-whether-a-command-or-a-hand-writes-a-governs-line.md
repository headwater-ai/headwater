---
id: HW-PROBE-a-session-says-whether-a-command-or-a-hand-writes-a-governs-line
status: current
status_since: 2026-10-10
summary: "Many documents share the terms of this task, and a superseded record, a discharged one and two generated pages still say a person types a governs line, so a session must find the one record in force, over the tree that #1709 repaired."
last_verified: 2026-10-10
probe_category: navigability
expectation: answered
oracle: "none"
title: "A session says whether a command or a hand writes a governs line"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  supersedes:
    - HW-PROBE-a-session-says-how-a-governs-line-reaches-a-document
  traces_to:
    - HW-SPEC-ai-integration
---

# A session says whether a command or a hand writes a governs line

## Task

An agent in this repository writes a new document. The document must declare a `governs` edge onto a source file, in its front matter. Can a command of the `headwater` tool write that line into the new document? Or must the line be typed by hand?

Answer on one line with one word, `command` or `hand`, and with nothing else. Your whole final message is that word.

## Expectation

The terminal answer is `command`, the one expected value of the closed set below.

```yaml
answers: [command, hand]
expected: [command]
```

**This probe replaces [the probe that the campaign of 2026-10-03 ran](a-session-says-how-a-governs-line-reaches-a-document.md), and only the tree is different (#1709).** The task and the key are word for word the same. The key held. The recount of that probe missed one current near copy, and #1709 repaired it, so the tree that a session searches changed. A result of this probe does not compare with the results of that campaign, which ran the old identifier.

**This probe has the distractors-at-scale shape (#1472).** The record in force is [HW-DR-0104](../decisions/0104-an-agent-writes-governs-traces-to-and-cited-in-through-the-verb-and-the-review-of-its-pull-request-is-the-acceptance.md), at `status: current`. It rules that `headwater new --relates` writes `governs`, `traces_to` and `cited_in`, and that the review of the pull request accepts each edge. On 2026-10-10, `headwater new probe --relates traces_to=HW-SPEC-ai-integration` wrote this document's own `traces_to` edge, which declares `created_by: agent` as `governs` does.

**Near copies state the other answer.** [HW-DR-0083](../decisions/0083-governs-and-traces-to-are-created-by-an-agent-because-a-session-proposes-the-line-and-a-person-types-it.md) is `superseded` by HW-DR-0104. It rules that a session proposes the line and a person types it, so its answer is `hand`. Three other documents still state its answer. None of them is a current record: one is a discharged obligation, and two are generated pages that print the summary of HW-DR-0083. A session that answers from any of them answers `hand`, and the grade finds that answer wrong.

**The count, and how it was measured.** A near copy here is a document that holds the two key terms of the task, `governs` and `created_by`. The tree is the tree of #1709, extracted with `git archive` and sealed with `tools/probe/seal.sh` for the five probes of the `HARDER` table in `engine/crates/probe/tests/corpus.rs`, this probe in place of its predecessor. In that tree, `grep -r -l 'created_by' docs | wc -l` prints the documents that hold the first term, and `xargs grep -l governs` over that list prints the near copies. Then `grep -n -i -E "no verb (of this engine )?writes|person types|is hand entry|types the line"` over the near copies prints each candidate sentence. A person reads each candidate and keeps the documents that state the answer `hand` about `governs`.

**Why the search has one more term.** The search of the predecessor did not have `types the line`. So it did not find [the governs-edges evaluation](../evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md), a current near copy that said "A person or an agent that has read the file types the line." On the tree of `origin/main` at `f4a7c908`, the wider search prints that sentence. So the true count on the tree of the campaign was 5 of 39, not 4. #1709 repaired the sentence, and the evaluation now cites HW-DR-0104.

**The recount of 2026-10-10.** The first search printed 52 documents, and 39 of them are near copies. The wider search prints no sentence of the governs-edges evaluation. These 4 of the 39 state the answer `hand` about `governs`:

- HW-DR-0083 itself, which is `superseded`.
- [HW-OBL-0105](../obligations/0105-nothing-plays-the-hook-role-that-two-relations-name.md), at `discharged`: "No verb of this engine writes either edge".
- The generated index `docs/decisions/README.md` and the tombstone `docs/spec/09-open-questions.md`, which each print the summary of HW-DR-0083 beside the mark `superseded`.

The other candidate sentences are about another relation or another artifact, or they state the expected answer. `docs/taxonomies/brd-prd/doctrine.md` says that a person types the line of a relation between a business need and a product document, and it cites HW-DR-0083. It does not name `governs`, so the count leaves it out. Spec 13 says that a person typed each line at the time of HW-DR-0083, and then that HW-DR-0104 supersedes it. HW-DR-0104 and [the contract of `headwater new`](../interfaces/headwater-new.md) state the expected answer.

**The distractors are not built for this probe.** Each one is a document of this corpus that went stale when HW-DR-0104 landed. A campaign over this probe therefore measures the corpus as it is. If a later change repairs more of these documents, the count above falls, and this probe must count again before it runs.
