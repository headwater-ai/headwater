---
id: HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks
status: current
status_since: 2026-09-11
summary: The declared instrument for the assist claim cannot separate its arms, so this grades the artifact a session produced against a rule instead of counting what a verb supplied.
last_verified: 2026-09-11
probe_category: sufficiency
expectation: patched
oracle: "section.required.missing"
title: "A session records an unmeasured claim in the shape this corpus checks"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-OBL-0004
---

# A session records an unmeasured claim in the shape this corpus checks

## Task

Nothing here states how much of a session's budget the standing instructions of this repository consume before any work starts. Write that gap down where this repository keeps what it still owes, so a later reader meets it in the same shape as every other entry there.

The task names no verb, no file and no kind. It states an intent and a place in words, and a session decides what the shape is.

## Expectation

`patched` over the oracle `section.required.missing`. The session produced an artifact, something ran the rule set over it, and that rule reported nothing.

**An artifact this rule never read is not an artifact that passed it.** `findings` is optional in the recorder contract, and the two readings of it are not the same fact. An absent key says that nothing checked the artifact. An empty list says that the oracle ran and found nothing. A grader that treated the first as the second would return a satisfied verdict for every run that never ran a check at all, which is the failure this expectation form exists to avoid. The grader refuses such an artifact, and the refusal names it.

**A `patched` probe is the one form whose oracle is a rule this engine carries**, and this is the first document of this corpus to write a rule name in the `oracle` facet rather than the `none` sentinel. Every other form writes the sentinel, because the taxonomy language cannot require a facet on one value of another facet. [HW-OBL-0123](../obligations/0123-a-facet-that-applies-to-one-value-of-another-facet-has-nowhere-to-say-so.md) holds that gap, and the pairing is enforced in both directions by the planner instead.

**The rule has to be able to fire on what the session writes.** `section.required.missing` reads the section contract that a kind declares, so it reports over a document the taxonomy classified and over nothing else. A session that writes a stray file outside every shelf produces an artifact that no rule reads, which the paragraph above refuses rather than passes.

[HW-OBL-0004](../obligations/0004-working-tree-write-tools-have-no-measured-effect.md) records the claim this narrows. Its declared instrument is the assisted fraction, and [HW-OBL-0112](../obligations/0112-a-surface-cannot-move-the-assisted-fraction-of-a-run.md) measured that the fraction is derived from a plan and cannot separate the arms of that claim at all. An oracle grades the outcome rather than the cost: whether the artifact conformed, and not how many units a verb supplied. That is a second instrument for one claim, and it is the half that is missing.
