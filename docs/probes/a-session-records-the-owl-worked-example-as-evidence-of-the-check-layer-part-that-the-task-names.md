---
id: HW-PROBE-a-session-records-the-owl-worked-example-as-evidence-of-the-check-layer-part-that-the-task-names
status: current
status_since: 2026-10-29
summary: "A task that names spec 12 asks for the OWL and SKOS worked example to be declared as its evidence, and the oracle fails a change that writes only the evaluation's half."
last_verified: 2026-10-29
probe_category: sufficiency
expectation: patched
oracle: "command:r5-check-layer-owl"
title: "A session records the OWL worked example as evidence of the check layer part that the task names"
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
    - HW-SPEC-check-layer
---

# A session records the OWL worked example as evidence of the check layer part that the task names

## Task

### Phrasing 1

`docs/spec/12-check-layer.md` rules that a construct which entails rather than checks is out, and it links the OWL and SKOS worked example for that rule. The worked example is not among the evidence its front matter declares. Record that link in the corpus.

### Phrasing 2

The paragraph of `docs/spec/12-check-layer.md` about `owl:minCardinality` and `rdfs:range` draws on an evaluation that it does not list as cited evidence. Fix the front matter of the corpus so that the link is recorded.

### Phrasing 3

Read `docs/spec/12-check-layer.md` where it explains why a construct that entails is out. Declare the evaluation behind that paragraph as evidence the spec part cites.

## Target

[Spec 12](../spec/12-check-layer.md), "A construct that entails rather than checks is out", and the evaluation HW-EVAL-owl-skos-worked-example that it links for the rule.

## Key

`patched` over `command:r5-check-layer-owl`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/reciprocal-link.sh --check-clean` with the two documents in a held file under `tools/probe/oracle-args/`, which names this probe. The oracle passes when the `relations:` block of each names the other as a whole name, and neither carries a strict error.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r5-check-layer-owl.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change adds the worked example under `cites_evidence` of spec 12 and spec 12 under `cited_by` of the worked example, and it passes. The wrong change writes the evaluation's half alone, which a session that edits the document it was told about last can do, and it fails. The grading tree as it stands fails.

**The axes.** Each phrasing names the file, so the probe carries `names_file`. No superseded record stands between the two documents, so the probe does not carry `superseded_target`.
