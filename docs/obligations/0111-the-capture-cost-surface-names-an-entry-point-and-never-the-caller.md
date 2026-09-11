---
id: HW-OBL-0111
title: "The capture-cost surface names an entry point and never the caller"
status: current
status_since: 2026-08-14
waiting_on: measurement
last_verified: 2026-08-14
summary: "The store records which wire a run of the authoring verb arrived on, and an agent drove both wires on the day the term arrived."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0007
    - engine/crates/scaffold/src/reading.rs
---

# The capture-cost surface names an entry point and never the caller

## Context

The capture-cost store took a `surface` term on 2026-08-14, and [HW-OBL-0004](0004-working-tree-write-tools-have-no-measured-effect.md) is the record that asked for it. The term separates a run of `headwater new` at a terminal from a run of the `new` tool of the MCP server. Two arms are what make [Q7](../spec/09-decisions.md#q7--scope-of-the-mcp-surface)'s comparison possible at all.

## Obligation

The term records the entry point that a run arrived at. It records nothing about who drove that entry point. A person who types a tool call and an agent that emits one put the same message on one wire. The store holds one reading for each. So a difference between two rows of `headwater capture` is a difference between two entry points. It is never a difference between an assisted run and an unassisted one.

The corpus owes a reader of the two arms that sentence, or an instrument that separates the caller from the wire. This engine holds no such instrument. [HW-OBL-0108](0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md) is the same question one level up: an agent typed the acceptance stamp of every document here, and no field records that it did.

## Discharge

The store holds three readings that name an arm on the day the term arrived: one under `protocol` and two under `terminal`. One agent took all three, in one session, over one change. `protocol` names the run that wrote this record, over a server started with `headwater mcp --write`. The two terminal readings name the runs that wrote [HW-OBL-0112](0112-a-surface-cannot-move-the-assisted-fraction-of-a-run.md) and [HW-OBL-0113](0113-every-check-passes-a-document-that-is-still-the-scaffolder-s-placeholder.md). A reader of `headwater capture` sees two populations, and one caller produced every reading in both.

A term that named the caller is refused for the reason the store refuses a person. A per-author number is a performance measure, and a performance measure changes the behavior that it measures. So the remedy is a measurement rather than a field. [Spec 5](../spec/05-ai-integration.md#a-run-produces-a-snapshot-and-a-document) makes a transcript the one place where the corpus observes agent behavior rather than infers it. [The run of 2026-09-11](../probe-results/regression-probe-transcript-for-2026-09-11.md) is the first transcript that graded, and a later change moved the lock under it, so it grades to nothing now. It observes tool calls alone. A transcript carries no capture-cost reading, so this record still waits on a measurement that no probe of this corpus takes.
