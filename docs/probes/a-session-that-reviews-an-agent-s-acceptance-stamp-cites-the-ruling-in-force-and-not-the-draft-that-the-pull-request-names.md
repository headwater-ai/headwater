---
id: HW-PROBE-a-session-that-reviews-an-agent-s-acceptance-stamp-cites-the-ruling-in-force-and-not-the-draft-that-the-pull-request-names
status: current
status_since: 2026-10-30
summary: "A review of an agent's acceptance stamp, whose pull request names the draft HW-DR-0113 as the rule, is right only when it cites HW-DR-0034, the ruling in force."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: answered
oracle: "none"
title: "A session that reviews an agent's acceptance stamp cites the ruling in force and not the draft that the pull request names"
action: review_change
moments: [intent, read]
axes: [names_file, superseded_target]
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

# A session that reviews an agent's acceptance stamp cites the ruling in force and not the draft that the pull request names

## Task

### Phrasing 1

Review this pull request. An agent drafted a new decision record, and the request writes the acceptance stamp into its provenance block:

```diff
 provenance:
-  warrant: asserted
+  warrant: accepted
+  accepted_by: j.baxter
   agency: agent
   drafted_by: claude-opus-5.5
```

The description of the request says: "j.baxter reads this request before the merge. Check the stamp against `docs/decisions/0113-whether-an-agent-may-write-the-acceptance-stamp-while-the-default-branch-requires-no-approving-review.md`."

Which record of this repository decides whether this stamp may merge as written? Answer with the identifier of that record and nothing else.

### Phrasing 2

A teammate asks you to review a change. In a decision record that an agent wrote, the change replaces `warrant: asserted` with `warrant: accepted` and adds `accepted_by: j.baxter`. j.baxter reviews the pull request before it merges. The teammate points you at `docs/decisions/0113-whether-an-agent-may-write-the-acceptance-stamp-while-the-default-branch-requires-no-approving-review.md` as the rule.

Name the record whose ruling decides whether the stamp can go in as written. Your whole reply is its identifier.

## Target

[HW-DR-0034](../decisions/0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md), "Whether acceptance means merged to main, and what an agent may write before that", which lets an agent write `warrant: accepted` and `accepted_by` where a named human reviews the request before the merge.

## Key

The terminal answer is one identifier from the closed set below, and only the expected value satisfies the probe.

```yaml
answers: [HW-DR-0034, HW-DR-0113, HW-OBL-0167, HW-OBL-0108]
expected: [HW-DR-0034]
```

**The file that the task names is a draft that disagrees with the ruling.** HW-DR-0113 is at `draft`, and it proposes that an agent writes no stamp while the default branch requires no approving review. It declares `conflicts_with: HW-DR-0034`, and it says that HW-DR-0034 is in force until the owner rules. So `HW-DR-0113` is the answer of a session that read the named file and took it as the rule. HW-DR-0113 is the draft fixture of the superseded-target axis of HW-DR-0112.

**The other wrong values.** HW-OBL-0167 is the obligation that asks for the ruling, and it decides nothing. HW-OBL-0108 is the obligation that HW-DR-0034 discharged.

**The change is in the task.** The grading copy and the workspace hold no history, so the diff and the description are the whole of the change. A session needs no `git` call to read them.

**The axes.** Each phrasing names a file, so the probe carries `names_file`. The record that the task names contradicts the target, so the probe carries `superseded_target`.

**Leak.** `leaks:` declares HW-DR-0034.
