---
id: HW-PROBE-a-session-that-changes-a-verb-s-default-reaches-the-verb-s-interface-contract
status: current
status_since: 2026-10-22
summary: "The task asks a session to change the default of one verb, and this asks whether the session opens the interface contract that states that default before it reports the change."
last_verified: 2026-10-22
probe_category: navigability
expectation: opened
oracle: "none"
title: "A session that changes a verb's default reaches the verb's interface contract"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  examines:
    - HW-IFACE-headwater-route
---

# A session that changes a verb's default reaches the verb's interface contract

## Task

Make `headwater route` offer seven ranked pointers by default instead of five, and leave the rest of its report as it is. Make the change, and say what else in this repository has to change with it.

## Target

The session must open [the interface contract of `headwater route`](../interfaces/headwater-route.md). It governs the routing code and the entry point of the verb, and it states the default budget of five pointers in two places. A change to the default that leaves the contract unchanged makes the contract stale.

## Key

`opened` over `HW-IFACE-headwater-route`.

**This probe was one of three that replaced two probes of the tuning set of the fourteen-scenario campaign (#1833).** [HW-DR-0111](../decisions/0111-the-additive-campaign-adds-six-layers-to-plain-code-in-fourteen-scenarios-and-registers-one-comparison-each.md) recorded that set and why the two probes left it. [HW-DR-0112](../decisions/0112-the-additive-campaign-measures-eleven-actions-by-five-layers-in-seven-scenarios.md) replaced that set, and #1874 builds the 44-probe set. Proposal 1, the first comment on #1833, is the design.

**What this probe expects, stated before any paid run.** About 55 percent in scenario 3, with a range of 40 to 75 percent. It is the nearest of the three to the ceiling, and the set holds it for that reason: the set needs one probe that a layer can move down as well as up. A search for the name of the verb over `docs/` finds the contract at once, and the routing code points to it. But a session that changes one constant and reports is done without opening anything under `docs/`.

**The task section is the prompt, and nothing else of this document reaches the session.** [Spec 15](../spec/15-the-recorder-contract.md#the-prompt-is-the-task-section-and-the-answer-is-the-final-line) fixes that.
