---
id: HW-EVAL-what-the-shadow-mode-routing-log-showed-at-its-bound
status: current
status_since: 2026-10-03
summary: "At 587 joined person prompts the embedding path was never silent. The main thread read a document after only 11 prompts, so the log cannot grade recall."
last_verified: 2026-10-03
title: "What the shadow-mode routing log showed at its bound"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0064
---

# What the shadow-mode routing log showed at its bound

[HW-DR-0064](../decisions/0064-q64-whether-intent-time-routing-gains-an-offline-embedding-path-in-shadow-mode.md) collects one shadow-log line per prompt. Each line holds what the deterministic route offered, and what the embedding path would have offered. This evaluation is the reading of that log at the end of its collection period. It states each figure with the command that printed it. It recommends nothing. [HW-DR-0105](../decisions/0105-whether-the-embedding-path-becomes-the-silence-only-fallback-of-routing-and-what-happens-to-the-shadow-collection.md) is the proposal that reads it.

## What was read, and when

One command printed every figure below:

    sh tools/run/shadow-mine.sh

[The how-to](../how-to/mine-the-shadow-mode-routing-log.md) states the procedure that the tool runs, and the header of `tools/run/shadow-mine.sh` states each rule it applies. The tool ran at commit `18cb6c6e2b181ae8ae11900face91f9d6fe6ede2`, at 2026-10-03T04:54:44Z. It read a copy of the log and of the main-thread transcripts, taken at 2026-10-03T04:47:21Z. A second run over the live files at 2026-10-03T04:54:50Z printed the same figures.

The log held 96 files and 1,022 lines. Its first line is at 2026-09-20T04:48:19Z, and its last line is at 2026-10-03T04:42:21Z. The transcripts came from 52 project directories.

## The collector, and the period

The collector is #917's. [Its comment of 2026-10-01](https://github.com/headwater-ai/headwater/issues/917) holds the day 2026-09-30. On that day 45 of 45 person prompts joined, with a Wilson 95% interval of [0.9213, 1.0000]. [PR #1536](https://github.com/headwater-ai/headwater/pull/1536) closed it. PR #1390 made each line one `write()` under a lock. PR #1420 added the fixture with concurrent writers.

The tool excluded these lines, and listed each one under its file:

| exclusion | lines |
|---|---|
| torn, does not parse | 1 |
| empty | 0 |
| submitted by a recorder (`probe_session` not empty) | 0 |
| empty `prompt_id` | 7 |

Three files hold every excluded line:

| file | torn | empty `prompt_id` |
|---|---|---|
| `de9c8ddd-41e4-45d1-90b3-8617e676498e.jsonl` | 1, line 72 | 0 |
| `17deb7c5-5e16-4739-b66b-f67aad932602.jsonl` | 0 | 6, lines 1 to 6 |
| `verify971-real-session-def456.jsonl` | 0 | 1, line 1 |

The last file is a fixture session that a verifier wrote into the live log on 2026-09-22.

The bound of HW-DR-0064 is 500 person prompts. The 500th joined prompt, in the time order of its first line, was logged at 2026-10-02T10:28:15Z. That line closed the period. The floor is 58 silent person prompts, and the log holds 123.

## The three buckets of the join

The join is on distinct prompt ids, and never on lines.

| bucket | prompts |
|---|---|
| typed person prompts in the period | 671 |
| joined to a log line | 587 |
| typed, with no line (gaps) | 84 |
| logged ids that no person prompt claims | 350 |

Both sums hold. Typed = joined + gaps: 671 = 587 + 84. Logged = joined + unclaimed: 937 = 587 + 350. The capture over the period is 587 of 671, or 0.8748.

The 587 joined ids carry 633 lines, and 39 ids carry more than one line. The tool reads each prompt from its earliest line. A prompt id with two lines is not always one prompt logged twice. On 2026-10-01 one id carried "If you can change the local GitHub runner do so…" and then "/compact", 27 seconds later. So the harness gives one id to more than one submission, and the later line can route a different text. Of the 39 ids, 7 have a silent line and a line that is not silent. A count of silent lines over any line of an id gives 130, and a count over the earliest line gives 123.

The restriction to a checkout that held an engine moves nothing. Of the 671 typed prompts, 670 came from sessions that started in a checkout that holds an engine now. One came from a checkout that holds none, and it is a gap. So every restricted figure below equals its unrestricted figure.

## The two paths, per model digest

The log holds one embedding model, digest `sha256:f342437e6e5f16aa1b759f8a329adcfc9328236d2caed6c8f289f7364b1e8e44`. Lines with no digest are a second row, where only the deterministic path ran. No rate below pools the two rows.

| digest | prompts | deterministic silent | embedding silent |
|---|---|---|---|
| `sha256:f342…8e44`, all | 515 | 102 of 515 (0.1981) | 0 of 515 (0.0000) |
| `sha256:f342…8e44`, injected | 413 | 0 of 413 | 0 of 413 |
| `sha256:f342…8e44`, not injected | 102 | 102 of 102 | 0 of 102 |
| none, all | 72 | 21 of 72 (0.2917) | no model |
| none, injected | 51 | 0 of 51 | no model |
| none, not injected | 21 | 21 of 21 | no model |

**The embedding path is never silent.** It returns the nearest summaries with no floor on the score, so it offers documents on every prompt. On the 102 prompts where the deterministic route was silent, the embedding path offered documents on all 102.

**The two paths agree on some document in 183 of 413 prompts** where both offered one. That figure is the line `both offered: 413, share a document: 183`.

`injected` is true exactly where the deterministic route was not silent, on both rows. So the split by `injected` repeats the split by silence, and it adds no second population.

## Recall against what the session read

A read counts when the main thread of the session opened a Markdown document under the line's `corpus_root`. The read must come after the prompt and before the next person prompt. A read under `.git` or `.claude` does not count, because no pointer names either.

| digest | prompts with a read | deterministic recall | embedding recall |
|---|---|---|---|
| `sha256:f342…8e44`, all | 7 | 0 of 7 | 0 of 7 |
| `sha256:f342…8e44`, injected | 5 | 0 of 5 | 0 of 5 |
| `sha256:f342…8e44`, not injected | 2 | 0 of 2 | 0 of 2 |
| none, all | 4 | 2 of 5 (0.4000) | no model |

On the 7 prompts with a read under digest `f342…`, neither path offered a document that the session read at any rank. On the no-digest row, the deterministic path found one read document at rank 1 and one at rank 4 or lower.

**So the log cannot grade recall.** Only 11 of 587 joined prompts were followed by a read of a corpus document on the main thread. In this repository the person types to a session that dispatches agents, and the agents read the documents on their own threads. A recall over 7 prompts and 7 documents cannot separate the two paths.

## A figure outside the tool

One figure here is not a line of `shadow-mine.sh`. It is the score of the top neighbor on each line that carries neighbors, over the whole log and not over joined prompts alone. On 115 lines where the route was silent, the median top score is 0.3252, with quartiles 0.2664 and 0.3997. On 719 lines where the route offered pointers, the median is 0.4127, with quartiles 0.3429 and 0.4568. The two ranges overlap from 0.1585 to 0.5498. So no single floor on the score keeps the embedding path silent where the route is silent and lets it speak where the route speaks.

## What the population cannot say

**The transcripts are a floor.** Retiring a worktree deletes its transcripts, and the population shrinks after the fact. On 2026-09-27 a reading fell from 314 person prompts to 61. The earliest transcript that remains is at 2026-09-20T04:48Z, and the log began on 2026-09-17.

**Three limits stand that the intake of run 20260930-1333 recorded.** A session in a worktree with no engine logs through the engine of the checkout where it started, so the restriction reads the start directory. A prompt that the harness submits, such as an agent notification, also writes a line. The unclaimed bucket holds that line, because no person prompt claims it. One prompt id can carry more than one line after #1420, as stated above.

**The third set is not a control.** A prompt whose route was injected had the pointers in context. With 11 prompts that read anything, the split cannot show an effect either way.
