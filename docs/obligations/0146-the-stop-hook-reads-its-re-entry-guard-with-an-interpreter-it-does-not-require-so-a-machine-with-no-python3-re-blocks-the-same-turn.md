---
id: HW-OBL-0146
status: current
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

This is discharged when `review.sh` decides the re-entry guard with no `python3` on `PATH`. A fixture must drive `review.sh` under the sabotaged `PATH` with `stop_hook_active` true and assert exit 0. A paired case drives the same setup with `stop_hook_active` false, over a tree the gate refuses, and asserts exit 2. The guard has to disappear only for the second stop, and stay in place for the first. Discharge also waits on `.claude/hooks/lib.sh` and the `CLAUDE.md` paragraph on the four hook positions, both brought into agreement with whatever the fix lands on. Each should name either the removed dependency or the residue it leaves behind. One decision stays open regardless of that fixture: which existing verb absorbs the read, and in what input form. Issue #319 asks the same question from the other side, and this document leaves it unsettled too.
