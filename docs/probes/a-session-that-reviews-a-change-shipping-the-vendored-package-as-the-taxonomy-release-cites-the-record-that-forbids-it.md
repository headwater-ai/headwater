---
id: HW-PROBE-a-session-that-reviews-a-change-shipping-the-vendored-package-as-the-taxonomy-release-cites-the-record-that-forbids-it
status: current
status_since: 2026-10-30
summary: "A review of a change that zips the vendored copy of the starter package as the taxonomy release asset is right only when it cites HW-DR-0089, which forbids it."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: answered
oracle: "none"
title: "A session that reviews a change shipping the vendored package as the taxonomy release cites the record that forbids it"
action: review_change
moments: [intent, read]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-DR-0089
---

# A session that reviews a change shipping the vendored package as the taxonomy release cites the record that forbids it

## Task

### Phrasing 1

Review this change to how the taxonomy release builds its asset. Before the change, the release ran `headwater taxonomy publish` on the taxonomy source at the tagged commit and zipped what it published. After the change, it zips the vendored copy of the starter package that this repository keeps for its own checks, and attaches that zip to the tag. The author says that this saves a build step.

Which decision record of this repository does this change break? Answer with the identifier of that record and nothing else.

### Phrasing 2

A pull request makes the release of the starter taxonomy package skip the publish step. It attaches a zip of the copy of the package that this repository already vendors for itself, in place of a fresh publish of the source at the tag.

Name the decision record that this pull request goes against. Reply with its identifier only.

## Target

[HW-DR-0089](../decisions/0089-a-taxonomy-release-publishes-the-source-at-the-tagged-commit-and-never-ships-the-vendored-copy.md), "A taxonomy release publishes the source at the tagged commit and never ships the vendored copy".

## Key

The terminal answer is one identifier from the closed set below, and only the expected value satisfies the probe.

```yaml
answers: [HW-DR-0089, HW-DR-0090, HW-DR-0067, HW-DR-0022]
expected: [HW-DR-0089]
```

**Why the wrong values are near.** HW-DR-0090 rules what the notes of a release state. HW-DR-0067 rules where the vendored copy lives, and not whether a release ships it. HW-DR-0022 rules what a package digest checks.

**The change is in the task.** The task describes the change in words, and the grading copy and the workspace hold no history. A session needs no `git` call.

**The axes.** No phrasing names a file, so the probe does not carry `names_file`. No superseded or contradicted record stands in the way, so the probe does not carry `superseded_target`.

**Leak.** `leaks:` declares HW-DR-0089.
