---
id: HW-OBL-0223
status: current
status_since: 2026-09-27
summary: "The campaign's absent arm keeps docs/, so no rate says whether the documents change what an agent builds. The documentation tier removes them, and nobody has paid for a run of it."
last_verified: 2026-09-27
title: "No run measures whether the documents under docs change what an agent builds"
waiting_on: measurement
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-ai-integration
    - HW-OBL-0004
    - HW-OBL-0112
---

# No run measures whether the documents under docs change what an agent builds

## Context

This repository claims that its documents change what an agent builds. [Spec 5](../spec/05-ai-integration.md#probe-categories) measures a claim with two arms of one probe set, and the absent arm names a declared ablation.

Until [#1010](https://github.com/headwater-ai/headwater/issues/1010), one ablation existed. It removes `CLAUDE.md`, `.claude/`, `.githooks/` and `.headwater/`, and it keeps `docs/`. So the `campaign` tier measures whether the governance changes what an agent builds. The documents are present in both of its arms, and no rate of it can say what they do.

#1010 added a second paired tier, `documentation`, to `.headwater/probe.yml`. Its ablation is the four governance paths, `docs`, and two files outside `docs/` that copy what a document states. The probe shelves are the instrument, and every arm of every tier removes them. `tools/probe/ablate.sh documentation <workspace>` produces its absent tree. `headwater probe plan --tier documentation` refuses a probe whose predicate names a document that the ablation removes. A session cannot open or cite a document that it never had.

## Obligation

No run of the `documentation` tier exists, so the claim is unmeasured.

Two probes on `docs/probes/` are its instrument. [HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request](../probes/a-session-names-the-status-a-settled-decision-carries-in-its-pull-request.md) grades the ruling of [HW-DR-0052](../decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md). [HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted](../probes/a-session-names-the-event-that-makes-a-document-accepted.md) grades the ruling of [HW-DR-0034](../decisions/0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md). Each declares one expected answer from a closed set. The status probe also asks for the identifier of its ruling, because sessions that read nothing gave its expected value anyway (#1294).

With the two existing admissible probes, `--category sufficiency` selects four probes. On 2026-09-27 the plan projects 4 x 2 x 58 = 464 sessions. That is $116.00 at the declared rate of 25 cents a session, and $226.20 to $236.06 at the realized rates of 48.75 and 50.875 cents. The ceiling is $50.00, so the plan refuses on budget. [#980](https://github.com/headwater-ai/headwater/issues/980) is where a person agrees to spend more.

The claim has a precise shape, and a published rate has to keep it. Present against absent at the `documentation` tier measures documents and governance together. The documents alone are the `documentation` tier's absent arm against the `campaign` tier's absent arm. That difference holds only when both ran in one batch on one model version. A `campaign` rate is never quoted for this claim.

[HW-OBL-0004](0004-working-tree-write-tools-have-no-measured-effect.md) and [HW-OBL-0112](0112-a-surface-cannot-move-the-assisted-fraction-of-a-run.md) record the other unmeasured effects of this corpus on a session, and neither one isolates the documents.

## Discharge

Two runs discharge this record: one of the `documentation` tier and one of the `campaign` tier. They run over the same selection, in one batch, on one model version, and both transcripts are committed under `docs/probe-runs/`. The result names the difference between the two absent arms as the effect of the documents, with its interval.

A ruling that the documents are not a claim this repository makes also discharges it. That ruling removes the `documentation` tier.
