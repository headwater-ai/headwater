---
id: HW-PROBE-a-session-records-the-walkthrough-that-the-federation-part-rests-a-rule-on-as-the-evidence-it-cites
status: current
status_since: 2026-10-29
summary: "A task that names spec 7 asks for the schema-format walkthrough to be declared as its evidence, and the oracle requires cites_evidence on the spec part and cited_by on the evaluation."
last_verified: 2026-10-29
probe_category: sufficiency
expectation: patched
oracle: "command:r5-federation-walkthrough"
title: "A session records the walkthrough that the federation part rests a rule on as the evidence it cites"
action: edit_depended_document
moments: [write, review]
axes: [names_file]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-SPEC-distribution-and-federation
---

# A session records the walkthrough that the federation part rests a rule on as the evidence it cites

## Task

### Phrasing 1

`docs/spec/07-distribution-and-federation.md` rests its rule that no key states whether a migration step is mechanical on a finding of the schema-format walkthrough, and it links that evaluation in its prose. Its front matter does not list the walkthrough among the evidence it cites. Record that link in the corpus.

### Phrasing 2

In `docs/spec/07-distribution-and-federation.md`, the paragraph on whether a step is mechanical quotes the schema-format walkthrough. Make the front matter of the corpus say that this spec part cites that walkthrough as evidence.

### Phrasing 3

Read the part of `docs/spec/07-distribution-and-federation.md` that says no key states whether a step is mechanical. Add the evaluation it relies on to the evidence that the spec part declares, in the way this corpus records evidence.

## Target

[Spec 7](../spec/07-distribution-and-federation.md), "No key states whether a step is mechanical", and the evaluation HW-EVAL-schema-format-walkthrough that it rests the rule on.

## Key

`patched` over `command:r5-federation-walkthrough`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/reciprocal-link.sh --check-clean` with the two documents in a held file under `tools/probe/oracle-args/`, which names this probe. The oracle passes when the `relations:` block of each names the other as a whole name, and neither carries a strict error. The taxonomy declares `cites_evidence` from a design spec to an evaluation with the inverse `cited_by`, and it requires both halves.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r5-federation-walkthrough.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change adds the walkthrough under `cites_evidence` of spec 7 and spec 7 under `cited_by` of the walkthrough, and it passes. The wrong change writes the spec half alone, and it fails. The grading tree as it stands fails.

**The axes.** Each phrasing names the file, so the probe carries `names_file`. No superseded record stands between the two documents, so the probe does not carry `superseded_target`.
