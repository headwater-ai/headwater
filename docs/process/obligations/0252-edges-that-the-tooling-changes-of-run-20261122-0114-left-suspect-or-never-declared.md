---
id: HW-OBL-0252
status: current
status_since: 2026-11-24
summary: "Tooling edits left governs edges on ci.yml, hw-integrate.md and the supervise fixtures suspect. No document governs pre-pr.sh, its fixtures or fast-forward-main.sh."
last_verified: 2026-11-24
title: "Edges that the tooling changes of run 20261122-0114 left suspect or never declared"
waiting_on: build
provenance:
  warrant: asserted
relations:
  traces_to:
    - HW-PD-0013
    - HW-PD-0020
    - HW-PD-0023
    - HW-OBL-0242
---

# Edges that the tooling changes of run 20261122-0114 left suspect or never declared

## Context

`hw-run-policy` sends a finding about the build order, its tooling or the CI of this repository to this shelf. Run `20261122-0114` wrote intake lines 46, 47, 48 and 72. The product owner ruled each one RECORD into this record, which its end-of-run report calls P7. HW-OBL-0242 records the same kind of debt for #1561. This record checked each edge again on `0cdd96298`.

## Obligation

**Edges left suspect.** A person re-reads the document against the target and restamps the edge, and nobody owns that re-read.

- A comment-only edit to `.github/workflows/ci.yml` left the `governs` edges of HW-PD-0013 to HW-PD-0020, and of the evaluation of the two triggers, suspect by hash.
- `docs/process/specs/orchestration-architecture.md` governs `.claude/agents/hw-integrate.md`, and the edits of #1996 left that edge suspect. The run-lessons pull request of this run edits the same agent again.
- HW-PD-0023 governs `tools/run/supervise.sh` and `tools/run/supervise-fixtures.sh`. Its edge to `supervise.sh` was suspect before #1997, and the run-lessons pull request of this run edits the fixtures.

**Paths that no document governs.**

- No `governs` or `traces_to` edge names `tools/run/pre-pr.sh`, `tools/run/pre-pr-fixtures.sh` or `tools/engine/engine-readme-fixtures.sh`. Since #1995, `pre-pr.sh` mirrors the fixture steps of CI and a drift judge holds it.
- No edge names `tools/run/fast-forward-main.sh`, which #1997 added and which `run-dir.sh` and `supervise.sh` call.

## Discharge

Each item discharges alone. A suspect edge discharges when a person restamps it after a re-read, or the document changes to match its target. A path discharges when a document that rules it declares the edge. The record discharges when every item has.
