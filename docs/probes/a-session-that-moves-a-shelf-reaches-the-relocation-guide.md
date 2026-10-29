---
id: HW-PROBE-a-session-that-moves-a-shelf-reaches-the-relocation-guide
status: current
status_since: 2026-10-22
summary: "The task asks a session to move a shelf of records to a new directory, and this asks whether the session opens the guide that states the order of a shelf move."
last_verified: 2026-10-22
probe_category: discovery
expectation: opened
oracle: "none"
title: "A session that moves a shelf reaches the relocation guide"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  examines:
    - HW-HOW-relocate-a-shelf
---

# A session that moves a shelf reaches the relocation guide

## Task

Move the shelf of verification records from `docs/verifications/` to `docs/proofs/`, so that each record is still typed afterwards and `headwater check` still passes. Repair whatever the move breaks, and list what you changed.

## Target

The session must open [the relocation guide](../how-to/relocate-a-shelf.md). It is the one document that states the order of a shelf move: the taxonomy moves first, the directory moves second, the lock is resolved again, and the relative links into the old path are repaired.

## Key

`opened` over `HW-HOW-relocate-a-shelf`.

**This probe was one of three that replaced two probes of the tuning set of the fourteen-scenario campaign (#1833).** [HW-DR-0111](../decisions/0111-the-additive-campaign-adds-six-layers-to-plain-code-in-fourteen-scenarios-and-registers-one-comparison-each.md) recorded that set and why the two probes left it. [HW-DR-0112](../decisions/0112-the-additive-campaign-measures-eleven-actions-by-five-layers-in-seven-scenarios.md) replaced that set, and #1874 builds the 44-probe set. Proposal 1, the first comment on #1833, is the design.

**What this probe expects, stated before any paid run.** About 35 percent in scenario 3, with a range of 20 to 55 percent. The task says "move", and the title, the path and the summary of the guide say "relocate", so a search by name misses it. A search for "shelf" returns many matches before the guide. The recordings of 2026-10-03 show that a session with a task about the taxonomy and no harness goes to spec 2 and to `.headwater/`, and not to a how-to.

**This repository's own overlay declares the shelf of verification records.** No package declares it, so the guide tells a session to edit the `path:` of that shelf in place and to add no `override` key. The rest of the procedure still applies. The key reads only whether the session opened the guide.

**The task section is the prompt, and nothing else of this document reaches the session.** [Spec 15](../spec/15-the-recorder-contract.md#the-prompt-is-the-task-section-and-the-answer-is-the-final-line) fixes that.
