---
id: HW-PROBE-a-session-that-reviews-a-change-dropping-the-release-digest-from-the-notes-of-the-engine-release-cites-the-record-that-governs-the-workflow
status: current
status_since: 2026-10-30
summary: "A review of a change that drops the release digest from the notes that the engine release writes is right only when it cites HW-DR-0090, which governs that workflow."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: answered
oracle: "none"
title: "A session that reviews a change dropping the release digest from the notes of the engine release cites the record that governs the workflow"
action: review_change
moments: [intent, read]
axes: [names_file]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-DR-0090
---

# A session that reviews a change dropping the release digest from the notes of the engine release cites the record that governs the workflow

## Task

### Phrasing 1

Review this change to `.github/workflows/release.yml`. It shortens the notes that the engine release writes:

```diff
-          printf 'release.digest: %s\n\nThis tag carries `headwater/standard` with that `release.digest`. Pass the value to `headwater taxonomy vendor <dir> --expect %s`. It is what `headwater taxonomy publish` printed for the package in this tree, and it is not a checksum of any one file.\n' "$digest" "$digest" >release-notes.md
+          printf 'This tag carries `headwater/standard`.\n' >release-notes.md
```

Which decision record of this repository does this change break? Answer with the identifier of that record and nothing else.

### Phrasing 2

A pull request edits `.github/workflows/release.yml` so that the release notes say only "This tag carries `headwater/standard`." The line with `release.digest` and the `--expect` value is gone from the notes.

Name the decision record that this pull request goes against. Your whole reply is its identifier.

## Target

[HW-DR-0090](../decisions/0090-each-release-states-in-its-notes-the-release-digest-that-a-consumer-pins.md), "Each release states in its notes the release digest that a consumer pins", which governs the release workflow of the engine.

## Key

The terminal answer is one identifier from the closed set below, and only the expected value satisfies the probe.

```yaml
answers: [HW-DR-0090, HW-DR-0089, HW-DR-0022, HW-DR-0046]
expected: [HW-DR-0090]
```

**Why the wrong values are near.** HW-DR-0089 governs the other release workflow, `release-taxonomy.yml`, and rules what that release publishes, not what the notes state. HW-DR-0022 rules that a digest checks a fetched artifact against a pin, and it says nothing of the notes. HW-DR-0046 rules that a migration records a semver and a release digest as two fields.

**The change is in the task.** The grading copy and the workspace hold no history, so the diff is the whole of the change. A session needs no `git` call to read it.

**The axes.** Each phrasing names the file, so the probe carries `names_file`. No superseded or contradicted record stands in the way, so the probe does not carry `superseded_target`.

**Leak.** `leaks:` declares HW-DR-0090. The workflow names HW-DR-0090 in its header, and that is the governed file, not a channel that every session loads.
