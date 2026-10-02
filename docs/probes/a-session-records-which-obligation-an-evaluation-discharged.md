---
id: HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged
status: current
status_since: 2026-10-03
summary: The task names one file to change, and the check reports a finding on that file until the session also changes a second document that the task does not name.
last_verified: 2026-10-03
probe_category: sufficiency
expectation: patched
oracle: "relation.reciprocity.missing"
title: "A session records which obligation an evaluation discharged"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-OBL-0004
---

# A session records which obligation an evaluation discharged

## Task

The evaluation at `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md` is the measurement that discharged the obligation HW-OBL-0104. Record that fact in the front matter of the evaluation, with the relation that this repository declares for it.

## Expectation

`patched` over the oracle `relation.reciprocity.missing`. The session produced an artifact, something ran the rule set over it, and that rule reported nothing over it.

**This probe has the change-task shape (#1472).** The task names one file. The relation it asks for is `discharges`, and the decision-record bundle declares it with `inverse: discharged_by` and `reciprocal: required`. So the obligation at the other end, `docs/obligations/0104-a-governs-edge-reaches-the-path-it-names-and-nothing.md`, owes the inverse half. Until it carries that half, the oracle reports a finding over the evaluation. A session that changes only the file the task names leaves that finding.

**The two candidate patches, checked.** On 2026-10-03, two copies of this repository at the commit this probe was written on were patched and checked with `headwater check --json`. The findings over the two files are below.

The first patch adds `discharges: [HW-OBL-0104]` under `relations:` in the evaluation, and changes nothing else:

    relation.reciprocity.missing error  the evaluation  `HW-EVAL-governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it` declares `discharges: HW-OBL-0104`, and `discharges` requires both ends, so docs/obligations/0104-a-governs-edge-reaches-the-path-it-names-and-nothing.md owes `discharged_by`
    warrant.evidence.unsupported warn   the evaluation  `HW-OBL-0104` claims `evidenced`, and `HW-EVAL-governs-edges-…`, which `discharges` makes its evidence, has …
    warrant.evidence.unsupported warn   the evaluation  `HW-EVAL-governs-edges-…` claims `evidenced` and declares `traces_to` to `HW-OBL-0117`, whose warrant is …

The second patch is the first patch, and it also adds `discharged_by: [HW-EVAL-governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it]` under `relations:` in the obligation:

    warrant.evidence.unsupported warn   the evaluation  `HW-OBL-0104` claims `evidenced`, and `HW-EVAL-governs-edges-…`, which `discharges` makes its evidence, has …
    warrant.evidence.unsupported warn   the evaluation  `HW-EVAL-governs-edges-…` claims `evidenced` and declares `traces_to` to `HW-OBL-0117`, whose warrant is …

The oracle reports over the first patch and not over the second. The last finding is on the tree before either patch, and the other `warrant.evidence.unsupported` finding is in both patches, so neither one separates them. `headwater check --fix` writes the missing inverse half, so a session that runs the check and its fix also passes.

**Why a session meets the second document at all.** `headwater new --relates` refuses `discharges`, because the bundle declares it `created_by: author`, so the session types the line. No verb writes the far half for it. Only the check, or a reading of the bundle, tells the session that the obligation owes a line.

**In the `documentation` tier, the asked file is absent.** That tier removes `docs/`, so neither the evaluation nor the obligation is in the tree. The probe stays on that line, as the unmeasured-claim probe does, so that the two absent arms are compared over one set of probes. A session there can only write a new file, and the oracle then reads it.

**The fact the task states is true.** HW-OBL-0104 is `discharged`, and the evaluation `traces_to` it. The evaluation does not yet declare `discharges`, because no evaluation of this corpus declares it.

[HW-OBL-0004](../obligations/0004-working-tree-write-tools-have-no-measured-effect.md) records the claim this probe narrows: that the write tools of the layer change what a session writes. An oracle grades the outcome. This probe supplies a second instrument for that claim and does not supply an observed result.
