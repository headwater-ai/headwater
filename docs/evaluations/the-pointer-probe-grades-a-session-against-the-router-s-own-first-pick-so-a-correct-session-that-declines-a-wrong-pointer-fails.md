---
id: HW-EVAL-the-pointer-probe-grades-a-session-against-the-router-s-own-first-pick-so-a-correct-session-that-declines-a-wrong-pointer-fails
status: current
status_since: 2026-09-17
summary: "A session that declined a wrong first pointer answered correctly from two spec parts that the router ranks past 100th."
last_verified: 2026-09-17
title: "The pointer probe grades a session against the router's own first pick, so a correct session that declines a wrong pointer fails"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-RUN-regression-probe-transcript-for-2026-09-17-after-the-probe-corrections
---

# The pointer probe grades a session against the router's own first pick, so a correct session that declines a wrong pointer fails

## Context

[Issue #910](https://github.com/headwater-ai/headwater/issues/910) asks for a judgment on one live miss. The session `regression-20260917b-pointer` in [the recording after the probe corrections](../probe-runs/regression-probe-transcript-for-2026-09-17-after-the-probe-corrections.md) ran `headwater route` over the corrected task text. The router ranked `HW-OBL-0107` first. The session then read `docs/spec/02-taxonomy-model.md` and `docs/spec/03-authoring-and-lifecycle.md`, and it never opened `HW-OBL-0107`.

The issue offers two judgments. The first is a model-behavior gap outside the reach of the corpus, as [the evaluation of #896](the-three-discovery-misses-of-2026-09-16-have-three-different-mechanisms-and-only-one-is-routing-precision.md) said for an earlier case. The second is a navigability gap that a corpus change can close. The evidence below supports neither. The session was right, the pointer was wrong, and the probe cannot tell the two apart.

The evidence has two sources. The first is the raw harness log of the session, which holds every tool result and the final message. The committed transcript holds neither, because [spec 15](../spec/15-the-recorder-contract.md) keeps a digest or an empty string in place of a result. The second is a replay of the router over this tree, quoted below so that a later reader can repeat it.

## What the session saw, and what it answered

The raw log confirms that the session saw the ranking. The result of its `headwater route` call lists `HW-OBL-0107` first, then `HW-OBL-0171`, `HW-OBL-0141` and `HW-OBL-0192`. It then lists `HW-OBL-0044`, the purpose scores `obligation` 7, `behavior` 3 and `evidence` 2, and 214 withheld pointers. So the issue is correct that the session saw the pointer and declined it.

The final message answers both halves of the task, and each half is correct. For the shelf, it cites the kind-resolution section of spec 2. The most specific shelf pattern wins, a homogeneous shelf declares the kind, and a heterogeneous shelf reads a discriminator facet. For the identifier, it cites the identifiers section of spec 3 and [HW-DR-0054](../decisions/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md). It states that `headwater new` mints from the corpus and the claim store together, and it states why the claim store exists.

`HW-OBL-0107` does not answer that task. Its summary says that `kinds.specification` names no identifier scheme, so `headwater new specification` refuses on the stock package. That is one defect of one kind. A reader who asks where a new document goes, and where its identifier comes from, learns neither from it. The session read five summaries and went to the documents that answer. That is the behavior spec 5 wants from an agent that meets a wrong pointer.

## Where the router puts the documents that answer

    headwater route --root . --budget 500 --json "Decide which shelf a new document belongs on, and where its identifier comes from."

The route offers 220 pointers on this tree. The first twelve are all obligation records. `docs/spec/03-authoring-and-lifecycle.md` ranks 120th, `docs/spec/02-taxonomy-model.md` ranks 126th, and `docs/decisions/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md` ranks 172nd. A rank is a fact about this tree on this day, and a later reader runs the command again rather than trusting the number.

The mechanism is in `engine/crates/query/src/route.rs`. Spec 5 has the router match the purpose of each kind before it compares any text. Lexical rank orders documents only inside a matched purpose. The task scores `obligation` 7 and `behavior` 3, so every obligation record that shares a term with the task outranks every design spec. Many obligation summaries name a shelf, a scheme or an identifier, because this corpus records its gaps in those areas.

The #896 evaluation found the cold-agent probe's target at 119th, and a replay on this tree shows the same mechanism. That task scores `behavior` fifth of six purposes, so `docs/spec/12-check-layer.md` ranks 127th. It is still the first design spec in that route.

## Why the two earlier evaluations called the router right

[The probe](../probes/the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens.md) sets its `examines` edge from the output of the router. Its expectation says so: the probe names the document that `headwater route` ranks first, and a later commit moves the edge when the rank moves. No sentence in the probe states that a person read `HW-OBL-0107` and judged that it answers the task.

So "the router ranks the examined document first" is true by construction. The #896 evaluation read that fact as "the router was right". [The #908 evaluation](a-probe-task-section-is-the-prompt-so-commentary-under-that-heading-is-prompt.md) inherited it, and it said that a miss with the bare task text is a reading of a model. Issue #910 inherited it a third time, as "a routing tool that gives the right answer". Each step checked the rank against the edge, and the edge came from the rank.

The #908 finding itself still holds. The commentary paragraph reached the session and changed the route. Its removal was necessary, and this live run is the first reading over the text the probe meant.

## Judgment

**This miss is not a model-behavior gap, and it is not a navigability gap in `HW-OBL-0107`.** It is a routing-precision miss that the probe grades as a session failure. A session that follows a wrong pointer passes this probe, and a session that declines it fails. The probe rewards the behavior that spec 5 calls the expensive one. Spec 5 says: "A wrong pointer costs more than a missing one, because an agent will follow it."

No wording in `HW-OBL-0107` would make a session open it for this task, and no wording should. Its summary is accurate for what it records.

## The corpus change this names, and the verdict it needs

**The probe needs a target that a person judged to answer its task, with that judgment written in its expectation.** A rank from the router is not a judgment of relevance. Two remedies meet that bar, and they measure different things.

- **Keep the task and change the target.** The probe then examines `HW-SPEC-taxonomy-model` and `HW-SPEC-authoring-and-lifecycle`, and it measures whether a session reaches the answer despite the pointer. That is a discovery reading and not a pointer reading, so the title of the probe no longer describes it. The committed transcript would re-grade to `satisfied`. That re-grade is not independent evidence, because this evaluation chose the target after it read the session.
- **Keep the purpose and change the task.** A person writes a task that names no verb and no file, and reads the first pointer the router offers for it. The probe is valid only where that person judges that the pointer answers the task. This keeps the title true, and it needs a fresh recording, because no recorded session saw the new text.

The second remedy is the one this evaluation recommends. The probe exists to take the first reading of routing precision for [#404](https://github.com/headwater-ai/headwater/issues/404), and the first remedy would remove that reading. This evaluation does not make the change, because the choice of task text is a judgment that a person owns.

A future live run shows that the change worked when all of these hold:

- the expectation of the probe names the person who judged that the examined document answers the task,
- the router still ranks that document first over the task text on the tree of the run, and
- the session opens that document, and its final message answers the task.

A session that declines a pointer that a person judged correct is then a reading of a model. It is the reading that the #908 evaluation expected from this run and did not get.

## Consequences

The #896 and #908 evaluations each state that the router was right for this probe. Each now carries one sentence that points here. This change edits no probe, so no read-set digest moves and no probe result goes stale.

[#915](https://github.com/headwater-ai/headwater/issues/915) owns the router change, with this task as its failing case. It asks for a ruling on two changes. The first stops one kind from filling the budget and shows why each pointer was offered. The second lets a route offer a section inside a document that covers many topics. [#819](https://github.com/headwater-ai/headwater/issues/819) does not reach either mechanism. Its shadow-mode log records what an alternative ranking would offer, and it changes nothing that a session is served.

A reader who meets a failed `opened` verdict on any probe whose target came from the router checks one thing first. Does the examined document answer the task, by the judgment of a person who read it?
