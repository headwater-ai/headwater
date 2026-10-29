---
id: HW-PROBE-a-session-links-the-governs-edges-evaluation-to-the-obligation-that-it-says-is-discharged
status: current
status_since: 2026-10-29
summary: "A session links the governs-edges evaluation to HW-OBL-0105, the obligation it says is discharged, past a paragraph that names the superseded HW-DR-0083 and beside HW-OBL-0104, which it does not discharge."
last_verified: 2026-10-29
probe_category: sufficiency
expectation: patched
oracle: "command:r5-hook-role"
title: "A session links the governs-edges evaluation to the obligation that it says is discharged"
action: edit_depended_document
moments: [write, review]
axes: [superseded_target]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-OBL-0105
---

# A session links the governs-edges evaluation to the obligation that it says is discharged

## Task

### Phrasing 1

One evaluation of this repository, about what a `governs` edge reaches, says in its prose that an obligation about the hook role is discharged. Neither document links the other. Record the discharge in the corpus.

### Phrasing 2

An obligation recorded that two relations name a hook as their author and that no hook writes them. The evaluation of `governs` edges says that obligation is now discharged. Link the two documents the way this corpus records a discharge.

### Phrasing 3

Find the evaluation that studied `governs` edges and the obligation it declares discharged, the one about nothing playing the hook role. Write the discharge into the corpus.

## Target

[The governs-edges evaluation](../evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md), whose paragraph on the hook role ends "HW-OBL-0105 is discharged", and the obligation HW-OBL-0105 that the paragraph names.

## Key

`patched` over `command:r5-hook-role`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/reciprocal-link.sh --check-clean` with the two documents in a held file under `tools/probe/oracle-args/`, which names this probe. The oracle passes when the `relations:` block of each names the other as a whole name, and neither carries a strict error.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r5-hook-role.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change adds `discharges: [HW-OBL-0105]` to the evaluation and `discharged_by` with the evaluation to the obligation, and it passes. The wrong change writes the same edge to HW-OBL-0104, the other obligation that the evaluation answers, and it fails. The grading tree as it stands fails.

**The axes.** The paragraph that states the discharge credits HW-DR-0083, which HW-DR-0104 superseded on 2026-10-02, so a superseded record stands in the way, and the probe carries `superseded_target`. No phrasing names a file, so the probe does not carry `names_file`.

**Beside the tuning probe of #1833.** That probe asks for the edge from the same evaluation to HW-OBL-0104. The two probes share one document and ask for different edges, and each oracle fails the other's reference change.
