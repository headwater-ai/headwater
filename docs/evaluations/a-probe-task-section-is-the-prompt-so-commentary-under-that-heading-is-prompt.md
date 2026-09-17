---
id: HW-EVAL-a-probe-task-section-is-the-prompt-so-commentary-under-that-heading-is-prompt
status: current
status_since: 2026-09-17
summary: "Two probes leaked commentary into the section a recorder sends as the prompt, and a third declared an answer set with no output contract."
last_verified: 2026-09-17
title: "A probe task section is the prompt, so commentary under that heading is prompt"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-RUN-regression-probe-transcript-for-2026-09-16
    - HW-RUN-regression-probe-transcript-for-2026-09-17
---

# A probe task section is the prompt, so commentary under that heading is prompt

## Context

[Issue #908](https://github.com/headwater-ai/headwater/issues/908) asks why two probes have never passed a graded run, and why a third failed in a different shape on two models. The three are [the pointer probe](../probes/the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens.md), [the tombstone probe](../probes/a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer.md) and [the adjudication probe](../probes/an-agent-reaches-the-adjudication-from-the-document-that-lost-it.md).

Each of the three is a defect in the probe document. None of the three is a reading of a model, and none is a routing-precision question. The evidence is the two committed transcripts of 2026-09-16 and 2026-09-17, and the raw harness logs behind them. It is also a replay of `headwater route` over the tree in front of this evaluation.

**A recorder sends the body of a probe's `## Task` section as the whole of the session's first turn, and it sends nothing else.** That was true of `tools/probe/probe-record.sh` before this evaluation and no document said so. [Spec 15](../spec/15-the-recorder-contract.md#the-prompt-is-the-task-section-and-the-answer-is-the-whole-final-message) now states it, with the rule by which the same recorder derives an answer. Both rules bound every probe already written, and two probes did not meet them.

## The pointer probe: the evaluation of #896 read the right router and the wrong text

[The evaluation of #896](the-three-discovery-misses-of-2026-09-16-have-three-different-mechanisms-and-only-one-is-routing-precision.md) replayed the router over the probe's intended task sentence and found `HW-OBL-0107` ranked first. That reading is correct, and it was taken over a text no session ever saw. The probe's `## Task` section carried a second paragraph of commentary below the instruction, and the recorder sent both.

Two replays over this branch's tree separate them. The first is the sentence the probe meant to ask:

    headwater route --root . "Decide which shelf a new document belongs on, and where its identifier comes from."

`docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md` (`HW-OBL-0107`) ranks first of five offered, and the budget withholds 214 more. The purpose scores are `obligation` 7, `behavior` 3, `evidence` 2, and 1 each for the other three.

The second is the text the session was actually given, which is the instruction and the commentary paragraph together:

    headwater route --root . "Decide which shelf a new document belongs on, and where its identifier comes from.

    The task names no verb and no file. It is the text the router below was given, word for word, because the predicate is over what that text resolves to."

`docs/obligations/0141-an-identifier-scheme-names-no-prefix-so-an-rdf-projection-derives-a-document-iri-from-a-file-path.md` (`HW-OBL-0141`) ranks first. `HW-OBL-0107` falls to second, and the budget withholds 348 more rather than 214. The purpose scores are `obligation` 12, `rationale` 10, `requirement` 8, `behavior` 6, `evidence` 6 and `procedure` 4.

The second set of six numbers is the set the session of 2026-09-17 quoted back in its own final message, after running the router itself. That is the confirmation that the commentary reached the model. The rank quoted here is a fact about this tree on the day this was written. A later reader re-runs the two commands rather than trusting the numbers, for the reason [HW-DR-0049](../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md) already gives.

The commentary did more than move a rank. It told the session that the text was a router input under measurement. The session of 2026-09-16 made no tool call and asked the user which document was meant. The session of 2026-09-17 read the commentary as the question. It ran `headwater route` on the task text, then wrote an accurate account of how the router resolves a purpose to a shelf. Neither session opened `HW-OBL-0107`, and the probe expects `opened` over it. The 2026-09-17 answer was a good answer to the paragraph that leaked, and the paragraph was addressed to a human reader.

The remedy is the paragraph's removal from `## Task` to `## Expectation`, where it now stands.

## The adjudication probe: the target was the index rather than the record it points at

This probe examined `HW-REG-decisions` alone, which is `docs/spec/09-decisions.md`, a generated index over the decisions shelf. Its Q8 row is one sentence that links [HW-DR-0008](../decisions/0008-probe-cost-and-cadence.md). That record is the adjudication for Q8. The index is a pointer at it and not a second destination.

Four sessions are recorded against this probe. Three of them, on 2026-09-09, 2026-09-16 and 2026-09-17, read `docs/spec/09-open-questions.md` and then read `docs/decisions/0008-probe-cost-and-cadence.md`. They followed the tombstone's one outgoing link to the answer, by the shortest route there is. The grader recorded `not satisfied`, because neither call named the index. The session of 2026-09-11 is the one that failed: it read the tombstone, made no second call, and answered from its heading text.

[HW-OBL-0014](../obligations/0014-no-probe-tests-whether-an-agent-reaches-the-adjudication-from.md) carried a wrong account of this until now. It read the 2026-09-16 session as one that "read a losing document that names a different decision than the one it went looking for". `docs/decisions/0008-probe-cost-and-cadence.md` is Q8's own record and not a different decision.

The remedy is `examines: [HW-REG-decisions, HW-DR-0008]`. `engine/crates/probe/src/grade.rs` satisfies `opened` where any recorded call names any examined document. So the widened edge grades the shorter route as a pass, and it still fails the session that opened neither.

This probe carried a leaked commentary paragraph too, with a hand-counted figure of 136 citations inside it. The reading today is 133. That paragraph is now under `## Expectation`, and it states the command rather than the number.

## The tombstone probe: two sessions reached the answer and the recorder discarded it

This probe declares a closed answer set of `present`, `withheld` and `absent`. Its `## Task` section carried no leaked commentary. It carried no output contract either.

`step_derive_answer` in `tools/probe/probe-record.sh` takes the final message of the session. It trims the space around it, strips one trailing period, and folds the case. It writes a value only where the whole of what remains equals one declared answer. The step is deliberate: a recorder that pulled the word out of surrounding prose would read the session rather than observe it.

Both graded sessions converged on `absent` and neither survived that comparison. The 2026-09-16 session searched `.headwater/overlay.yml`, found the `site` profile with no filter, and closed on the word in bold Markdown after three paragraphs of justification. The 2026-09-17 session searched `.headwater/export.json`, reached the same finding, and wrote the bare word after two sentences of justification. Each result reads "the session ended with no answer", which is true of the transcript and false of the session.

The remedy is one added instruction under `## Task`, which states that the whole final message is the word. The `withheld` arm stays unreachable for a separate reason that this evaluation does not touch. No export profile of this corpus declares a filtered grain, so no session can meet a counted tombstone. [HW-OBL-0013](../obligations/0013-no-probe-tests-whether-a-counted-tombstone-stops-a-confident.md) holds that, and it waits on a build.

## What the three share

A probe is a document that a session never reads, and a document that a person reads carefully. The two audiences met under one heading, and nothing separated them. The commentary that helps a reviewer understand why a task is worded as it is becomes prompt the moment the recorder copies the section. Spec 15 held every other line of the boundary between a recorder and a session, and it held neither of these two.

The misses that followed are not cheap. Each of the two recordings graded eight sessions, at a realized cost of $1.66 and $3.90. Three of the eight in each ran against a probe that could not be passed as written. Each result reported 3 of 8 satisfied before this change, a rate of 37.5%.

## Consequences

The read-set digest of each of these three probes moves with this change, so [the result of 2026-09-16](../probe-results/regression-probe-transcript-for-2026-09-16.md) and [the result of 2026-09-17](../probe-results/regression-probe-transcript-for-2026-09-17.md) go stale. `headwater probe stale` reports both. That is the correct outcome rather than a cost. The documents a session was pointed at changed, so a run over the old bytes says nothing about the new ones.

**One of the three remedies re-grades the two recordings, and two of them cannot.** A probe result is a function of the transcript, the expectations and the grader version. So a corrected `examines` edge changes what the committed transcripts already say. `headwater generate` rewrites both results. The adjudication session of each run moves from `not satisfied` to `satisfied`, on a witness that names the call it came from. The reported rate of each moves from 3 of 8 to 4 of 8. Read the new number beside the staleness rather than instead of it. The other two remedies reach nothing recorded. A task text that never went to a session is outside what a re-grade can recover. So is an answer the recorder never wrote down. Each needs a fresh recording.

No engine change follows from this evaluation. The grader already satisfies `opened` over any of several examined documents, and the recorder's answer rule is the one the design asks for. What was missing was a written contract, and spec 15 now carries it.

Three things a later reader checks against a fresh recording of this tier:

- **The pointer probe.** Check whether the session opened `HW-OBL-0107`, and re-run the first of the two router commands above. A miss with the bare task text is a reading of a model, which neither recorded miss was. [The evaluation of #910](the-pointer-probe-grades-a-session-against-the-router-s-own-first-pick-so-a-correct-session-that-declines-a-wrong-pointer-fails.md) shows that the first live miss over the bare text was not one. The first pointer does not answer the task.
- **The adjudication probe.** Check which of the two examined documents the session opened. A session that opens neither is the failure the probe is for, and it has happened once.
- **The tombstone probe.** Check whether the recorded `answer` is a value rather than `null`. A `null` after this change means the output contract did not bind. That is a reading worth having, and a different finding from the one above.

A fourth thing belongs to whoever next writes a probe. Read the `## Task` section aloud as the only text a session gets, because it is.
