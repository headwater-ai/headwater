---
id: HW-PROBE-a-session-that-reviews-an-agent-s-acceptance-stamp-on-a-branch-that-needs-no-approval-cites-the-ruling-in-force-and-not-the-draft-that-disagrees-with-it
status: current
status_since: 2026-10-30
summary: "A review of an agent's acceptance stamp on a branch that needs no approval is right only when it cites HW-DR-0034, and not the draft HW-DR-0113 that matches the task's words."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: answered
oracle: "none"
title: "A session that reviews an agent's acceptance stamp on a branch that needs no approval cites the ruling in force and not the draft that disagrees with it"
action: review_change
moments: [intent, read]
axes: [superseded_target]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-DR-0034
---

# A session that reviews an agent's acceptance stamp on a branch that needs no approval cites the ruling in force and not the draft that disagrees with it

## Task

### Phrasing 1

Review this change. The default branch of this repository requires no approving review. An agent drafted a decision record, and its pull request writes the acceptance stamp into the provenance block:

```diff
 provenance:
-  warrant: asserted
+  warrant: accepted
+  accepted_by: j.baxter
   agency: agent
```

j.baxter says that they read the request before the merge. Which record of this repository decides whether an agent may write this stamp? Answer with the identifier of that record and nothing else.

### Phrasing 2

The ruleset on the main branch here asks for zero approvals. A pull request from an agent sets `warrant: accepted` and `accepted_by: j.baxter` on a decision record that the agent wrote, in place of `warrant: asserted`. j.baxter reviews it before it merges.

Which record holds the ruling in force on whether an agent can write that stamp? Reply with its identifier only.

## Target

[HW-DR-0034](../decisions/0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md), "Whether acceptance means merged to main, and what an agent may write before that", which lets an agent write `warrant: accepted` and `accepted_by` where a named human reviews the request before the merge.

## Key

The terminal answer is one identifier from the closed set below, and only the expected value satisfies the probe.

```yaml
answers: [HW-DR-0034, HW-DR-0113, HW-OBL-0167, HW-OBL-0108]
expected: [HW-DR-0034]
```

**The words of the task lead to a draft that disagrees with the ruling.** HW-DR-0113 is at `draft`, and its title asks whether an agent may write the stamp while the default branch requires no approving review. It proposes that an agent writes none, and it declares `conflicts_with: HW-DR-0034`. It also says that HW-DR-0034 is in force until the owner rules. So `HW-DR-0113` is the answer of a session that found the record whose words match the task and took a proposal as the rule. HW-DR-0113 is the draft fixture of the superseded-target axis of HW-DR-0112.

**The other wrong values.** HW-OBL-0167 records that the branch asks for no approval and that a ruling is owed, and it decides nothing. HW-OBL-0108 is the obligation that HW-DR-0034 discharged.

**The change is in the task.** The grading copy and the workspace hold no history, so the diff is the whole of the change. A session needs no `git` call to read it.

**The axes.** No phrasing names a file, so the probe does not carry `names_file`. A draft contradicts the target, so the probe carries `superseded_target`.

**Leak.** `leaks:` declares HW-DR-0034.
