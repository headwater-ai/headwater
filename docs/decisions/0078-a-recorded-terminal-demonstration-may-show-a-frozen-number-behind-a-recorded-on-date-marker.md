---
id: HW-DR-0078
status: current
status_since: 2026-09-20
summary: "The owner relaxed principle 11 for one asset. A front-page recording may show a number a live run does not produce, if a recorded-on-<date> marker stands beside it. Nothing compares that number against a later run."
last_verified: 2026-09-20
title: "A recorded terminal demonstration may show a frozen number behind a recorded-on-date marker"
relations:
  supersedes:
    - HW-DR-0061
  traces_to:
    - HW-OBL-0183
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: measure+draft
  evidence_basis: evidenced
---

# A recorded terminal demonstration may show a frozen number behind a recorded-on-date marker

## Context

[#601](https://github.com/headwater-ai/headwater/issues/601) asks what holds a recorded terminal demonstration true against a real run. [HW-DR-0061](0061-q61-how-a-recorded-terminal-demonstration-is-held-against-a-run.md) answered that question by confining a recording to the tutorial's scratch corpus. Nothing in this repository refreshes a number frozen inside a binary asset. That record left one question with the product owner: whether principle 11 of [spec 0](../spec/00-vision-and-scope.md#design-principles) may relax for a marketing asset. HW-DR-0061 named the question and left the rule in force.

A later build-order run put the question to the owner directly, as `RULING #601`. May a recording show output counts a live run does not reproduce? The condition offered was a visible marker of the recording's own age. The owner answered on the issue's own timeline: "Relax the rule with a `recorded-on-<date>` marker." A recording may show output counts a live run does not reproduce, if it carries a visible `recorded-on-<date>` marker ([comment](https://github.com/headwater-ai/headwater/issues/601#issuecomment-5752910106)).

This is a ruling on the one asset HW-DR-0061 confined. It answers the owner's question with disclosure, and not with the enforcement HW-DR-0061 named as the reopening condition. This record is the successor HW-DR-0061 needs: no decision record in this corpus had yet revised a `current` decision in place.

## Decision

**A front-page recording may show a number a live run does not produce, if a visible marker stands beside it.** The marker takes the exact form `recorded on YYYY-MM-DD`, as plain text next to the embed. A reader who sees the recording also sees the date it stopped matching a live run. The marker is what tells them so.

**The marker discloses the freeze. It does not hold the frozen number against anything.** Nothing regenerates the recording. Nothing compares its numbers against a later run. HW-DR-0061 named that comparison, with a recorder on the runner, as the condition to reopen this question. The owner chose the marker instead. The comparison HW-DR-0061 described is an accepted deviation for this one asset now, not a gap this corpus still owes.

**HW-DR-0061's still-standing findings carry forward unchanged.** The tool is `vhs`. The embed format is an animated GIF. GitHub renders no asciinema player in a README, and it blocks an animated third-party SVG in its image proxy. This record narrows the rule that confined a recording's subject matter. It leaves the choice of tool and format exactly as HW-DR-0061 stated them.

**What a blocking check may hold is the marker's presence, and not the frozen number's accuracy.** A GIF is a binary no fixture reads. The check that enforces this ruling reads the README's text instead. It fails when a recording embed carries no `recorded on YYYY-MM-DD` marker beside it, and it never opens the GIF.

## Consequences

**The recording clause of [HW-OBL-0183](../obligations/0183-no-mechanism-holds-a-committed-binary-asset-against-a-run.md) is narrowed by this record.** That record's discharge condition, a CI step that regenerates a committed binary asset and compares it against the tree, was written against HW-DR-0061's confinement rule. For a front-page recording, that condition does not describe what closes the #601-shaped half of the obligation. The owner chose a marker over a compare-and-regenerate step, and the marker's presence is what the blocking check holds now.

**A recording of a corpus number, once committed, ages by construction and stays true by disclosure rather than by refresh.** A reader who runs the command named in the recording, and sees different numbers, has not been shown a lie. The marker beside the recording already told them the count was recorded on a stated date, and not read from their own run.

**Nothing here mints a recording.** No recorder is installed on the host that measured this record. The committed recording and its embed stay open on #601 until a recorder exists. This record answers only what a committed recording may show, and how it discloses its own age, once one is built.
