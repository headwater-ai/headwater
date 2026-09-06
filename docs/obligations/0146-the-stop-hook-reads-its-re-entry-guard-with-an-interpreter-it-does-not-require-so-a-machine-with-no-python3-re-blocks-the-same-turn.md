---
id: HW-OBL-0146
status: discharged
status_since: 2026-09-06
last_verified: 2026-08-26
title: "The Stop hook reads its re-entry guard with an interpreter it does not require, so a machine with no python3 re-blocks the same turn"
summary: "The Stop hook reads its re-entry guard through python3 with no fallback, so a machine without python3 re-blocks the same turn."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
waiting_on: adopter
---

# The Stop hook reads its re-entry guard with an interpreter it does not require, so a machine with no python3 re-blocks the same turn

## Context

`.claude/hooks/lib.sh` reads every field of the harness JSON payload by shelling out to `python3`, through the helper `hw_field`. Three hook positions call it: `intent.sh`, `write.sh`, and `review.sh`. `intent.sh` and `write.sh` guard that read with `|| exit 0`. Without `python3` the read fails there, and each hook exits silently rather than deciding. That silence matches [the hook contract](../spec/05-ai-integration.md#the-hook-contract-and-what-a-hook-cannot-bind): a hook unable to decide lets the action proceed. `review.sh:29` reads `stop_hook_active`, the flag marking a re-entry after the hook already stopped the turn once, and it carries no such guard. On a machine with no `python3` that read resolves to an empty string instead of an exit. The hook then proceeds into the commit gate rather than out of it.

## Obligation

The corpus owes a `review.sh` that decides its re-entry guard without requiring `python3`. As written, a machine with no `python3` runs the gate on every stop, including the second one. A failing tree exits 2, the harness re-invokes the hook with `stop_hook_active` true, and the hook still cannot read the flag. It exits 2 again, with no way out of the loop. `.claude/hooks/fixtures.sh` does not catch this gap. Its `fails_open` helper at line 211 drives only `write.sh` under a sabotaged `PATH`. Its three `stop_hook_active` cases all run with a working interpreter present, so no fixture combines the two conditions.

The interpreter is not required for the read itself. `engine/crates/yaml/src/json.rs` already parses JSON as a subset of the YAML 1.2 core schema. `engine/crates/query/src/mcp.rs` parses every JSON-RPC request through that same loader, over stdin. `hw_quote`, the one line of Python that escapes a value for the wire format, already has a matching writer in that same crate. Whatever reads `stop_hook_active` should reach an existing verb of the engine rather than shell out. The hook contract forbids a hook from introducing its own verb. The fix has to reach an existing verb as an input form, not a new `headwater hook <moment>` entry point.

## Discharge

**This record is discharged, and no position of these hooks runs an interpreter.** [HW-DR-0055](../decisions/0055-a-hook-reads-a-wire-format-through-the-engine-and-not-through-an-interpreter.md) settled the decision this record left open. `headwater json` reads the harness payload. It answers nothing about a corpus, and it names no moment. The hook contract's third term therefore reaches it in neither of the two ways it forbids a verb.

**The guard is decided by the engine, and a host without one ends the turn rather than stopping it twice.** `review.sh` reads `stop_hook_active` through `hw_field`, and it exits 0 before that read when there is no engine to make it. The loop this record was filed about therefore has no state it can start from.

**The pair this record asked for is in `.claude/hooks/fixtures.sh`, and one case more than it asked for.** A `python3` that records every call stands first on `PATH` over the tree the gate refuses. A second stop exits 0, a first stop exits 2 and names the planted document, and the recording file is empty for both. A third case moves the engine away and asserts that the same first stop ends the turn. The same recorder covers the other three positions at full strength, so the claim is about all four rather than about one.

The two files this record named are in agreement with what landed. `.claude/hooks/lib.sh` states that `sh` and the built engine are the whole of what a session needs. The `CLAUDE.md` paragraph on the four positions states the same and names the ruling.
