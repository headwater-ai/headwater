---
id: HW-DR-0113
status: draft
status_since: 2026-10-30
summary: "Proposed, not ruled: while main requires no approving review, an agent writes no acceptance stamp, against HW-DR-0034. It is also the draft fixture of the superseded-target axis of HW-DR-0112."
last_verified: 2026-10-30
title: "113 — Whether an agent may write the acceptance stamp while the default branch requires no approving review"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-OBL-0167
  conflicts_with:
    - HW-DR-0034
---

# 113 — Whether an agent may write the acceptance stamp while the default branch requires no approving review

**This record is a proposal. The owner has not ruled on it.** It stays at `draft` until the owner rules, and nothing in it binds a build before then. [HW-DR-0034](0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md) is the ruling in force, and this record disagrees with it.

## Context

HW-DR-0034 rules that acceptance is the merge onto `main`. It lets an agent write `warrant: accepted` and `accepted_by` on a document that it drafts, where a named human reviews the pull request before the merge. Its last paragraph names what reopens it: a branch-protection posture that lets a document reach `main` with no review.

[HW-OBL-0167](../obligations/0167-the-default-branch-requires-no-approving-review-which-is-the-condition-hw-dr-0034-named-as-reopening-it.md) measured that posture. The ruleset `Protect main` requires a pull request and zero approving reviews. `gh api repos/headwater-ai/headwater/rulesets/20627391` read the same count of 0 on 2026-10-30. So the condition that HW-DR-0034 named is met, and the obligation asks the owner for a ruling. It names three answers and proposes none. This record takes one of the three answers. It rewrites what an agent may write, because a setting of zero does not hold that line.

The question is real for every record that an agent drafts in this repository. The build order merges its own pull requests through the merge queue, and no rule of this engine reads a field of the provenance block. So nothing on the path to `main` checks the person that `accepted_by` names.

## Decision

**Proposed: while the default branch requires no approving review, an agent writes no acceptance stamp.** An agent writes `warrant: asserted` on each document that it drafts, and it writes no `accepted_by`. This holds when the agent expects a named human to read the pull request. A review that the branch does not require leaves no record that it occurred, so the stamp states a fact that nothing can show.

**Proposed: a person writes the stamp after the merge.** A person reads the merged record, sets `warrant: accepted` and names themselves. That is the promotion that [spec 3](../spec/03-authoring-and-lifecycle.md#promotion-is-one-human-one-document-one-diff) already states, and HW-DR-0034 keeps it.

**This disagrees with HW-DR-0034, and the front matter says so.** HW-DR-0034 lets an agent write the stamp at draft time where a named human reviews the request. This record removes that permission for as long as the required count is zero. The two records cannot both be in force, so this record declares `conflicts_with: HW-DR-0034`.

## Consequences

**Until the owner rules, HW-DR-0034 is in force.** An agent that drafts a document follows HW-DR-0034 and not this record.

**If the owner accepts this record, it supersedes HW-DR-0034 in part.** The acceptance rule stays: the merge is still the act that makes a document governed. Only the permission for an agent to write the stamp at draft time goes. HW-OBL-0167 then discharges. The documents that already carry an agent's stamp stay as written, as HW-DR-0034 ruled for the earlier habit.

**The owner may rule the other way.** The owner may raise the required count to one, which keeps HW-DR-0034 whole. The owner may also keep the count at zero and state why a solo repository draws the line at the merge. Either ruling retires this record.

**This record is also a fixture of the additive campaign.** HW-DR-0112 requires one real `draft` that disagrees with a `current` record. A session that reads this record as the rule in force cites the wrong document. A ruling on this record therefore also moves that fixture, and the campaign then needs another.
