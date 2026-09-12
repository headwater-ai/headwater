---
id: HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on
status: current
status_since: 2026-09-11
summary: Every traceability edge in this corpus is hand entry under a creator value that names a hook, and this asks whether an agent supplies the identifier with nobody asking for one.
last_verified: 2026-09-11
probe_category: navigability
expectation: cited
oracle: "none"
title: "What a session writes points back at the ruling it rests on"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft
  evidence_basis: unevidenced
relations:
  examines:
    - HW-DR-0049
  traces_to:
    - HW-OBL-0105
---

# What a session writes points back at the ruling it rests on

## Task

Write up, for a later reader of this repository, why a count taken over the whole corpus is worked out when it is read rather than kept in a committed file. Make the write-up point at the ruling it rests on, so the reader can check it.

The task names no verb, no file and no kind. It asks for a piece of writing and for a pointer inside it.

## Expectation

`cited` over `HW-DR-0049`. An artifact the session produced carries that identifier.

**The task asks for a written artifact because the predicate has nothing else to read.** A `cited` verdict comes from the `cites` key of a `produced` entry, which the recorder derives from the artifact. A task that only asks a question produces no entry, and the grader then refuses the form on every run of it. So a `cited` probe whose task ends in an answer is a probe that can never return a verdict, and the refusal reads as a harness fault rather than as the authoring fault it is.

**The examined target is a document of this corpus and never an anchor.** A produced artifact cites an identifier, and nothing cites a path, so the grader refuses a `cited` expectation whose targets all carry paths alone. `HW-DR-0049` is a decision with an identifier. A `code_path` anchor here would make this probe permanently ungradable in the same silent way.

**What the two arms separate is discovery of the ruling and not agreement with it.** A session with the corpus absent can state the same reason and has no identifier to name. So a miss here is either a session that did not find the ruling or a session that found it and wrote nothing a reader can follow, and the produced artifact separates those two by hand rather than by grade.

[HW-OBL-0105](../obligations/0105-nothing-plays-the-hook-role-that-two-relations-name.md) records the claim this narrows. `traces_to` declares `created_by: hook`, no verb of this engine writes one, and every half of every such edge in this corpus is hand entry. That record says the honest creator may be `agent` rather than `hook`, and it asks for the actor or the correction. A rate from this probe is the first evidence about which of the two the corpus should declare.
