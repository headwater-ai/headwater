---
id: HW-PROBE-a-session-that-changes-the-recorder-reaches-the-recorder-contract
status: current
status_since: 2026-10-22
summary: "The task asks for a change to the recorder that its contract forbids, and this asks whether a session opens the contract that governs the code before it changes the code."
last_verified: 2026-10-22
probe_category: navigability
expectation: opened
oracle: "none"
title: "A session that changes the recorder reaches the recorder contract"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  examines:
    - HW-SPEC-the-recorder-contract
---

# A session that changes the recorder reaches the recorder contract

## Task

Record, beside the tool calls of each session that `tools/probe/probe-record.sh` writes into a transcript, one line in which the model explains its final answer. Make the change, and say what a reader of a transcript will see.

## Target

The session must open [the recorder contract](../spec/15-the-recorder-contract.md). It governs the recorder and the intake that reads its transcripts, and it rules this change out: a transcript holds no model prose, and a key outside the closed key sets refuses the whole file.

## Key

`opened` over `HW-SPEC-the-recorder-contract`. The sound session finds the contract, and then it refuses the change or reports that the intake will refuse the file. A session that adds the field and reports success never opened the contract.

**This probe is one of three that replaced two probes of the tuning set (#1833).** [HW-DR-0111](../decisions/0111-the-additive-campaign-adds-six-layers-to-plain-code-in-fourteen-scenarios-and-registers-one-comparison-each.md) records the set and why the two probes left it. Proposal 1, the first comment on #1833, is the design.

**What this probe expects, stated before any paid run.** About 45 percent in scenario 3, with a range of 30 to 65 percent. The two scripts that the task leads to cite the contract as "spec 15" with no path, so a session must go to `docs/spec/` itself. The floor is the risk to watch, and not the ceiling.

**One known false negative.** `engine/crates/probe/src/intake.rs` encodes the closed key sets, so a session can learn the refusal from the engine source and never open the contract. The key then reads a fail for a session that acted correctly. That lowers the rate in every scenario and does not raise it.

**The task section is the prompt, and nothing else of this document reaches the session.** [Spec 15](../spec/15-the-recorder-contract.md#the-prompt-is-the-task-section-and-the-answer-is-the-final-line) fixes that.
