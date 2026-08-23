---
id: HW-OBL-0110
title: "The loss census carries a finding by substring, so an index of rules and paths passes it"
status: current
status_since: 2026-08-14
waiting_on: build
last_verified: 2026-08-14
summary: "The audit that holds a format to its loss set asks whether the bytes contain a rule name and a path, and a text report with no findings block at all carried 39 of 45."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-engine-architecture
    - HW-SPEC-assurance-model
---

# The loss census carries a finding by substring, so an index of rules and paths passes it

## Context

[Spec 6](../spec/06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped) requires an emitter to declare what its target cannot carry. It also requires a census over the output that holds the declaration to the bytes. `headwater_adapter::census` is that audit for the four output formats of a run. It reads the rendered artifact rather than the emitter. That is the right posture, because an emitter that audited itself would be the untrusted projector one layer out.

The instrument it reads with is a substring test. A finding counts as carried when the artifact contains its rule name anywhere, and contains its path anywhere. Those are two independent tests over the whole document. Neither asks whether the two appear together, and neither asks whether either appears in a findings block.

## Obligation

**A format that prints an index of rule names and an index of paths passes this audit whatever its findings block holds.** The text report prints both. It names every rule it ran in a list, whether or not the rule found anything. Its read set names every input the checks opened.

The measurement is over the check crate's fixture tree, which carries all three escape classes. Cut a text report down to the two index blocks, with no findings at all. The audit reports **39 of 45 findings carried and 6 unaccounted**. So the audit reports a defect, and it reports one for six findings out of forty-five. The artifact it read carries none of them. That artifact is 6,700 bytes of a whole one of 24,381.

The six that escaped did so because their paths are not inputs of the read set. That is a property of edge-scoped and corpus-scoped findings, rather than a property the audit reaches for. A corpus whose findings are all document-scoped would pass this audit with an empty findings block.

The corpus owes an audit whose power does not depend on which other blocks a format happens to print. The claim it is asked to hold is that a reader of the artifact can find each finding. A substring test over the whole artifact tests something weaker than that.

## Discharge

A finding is carried when its rule and its path appear **in one place a reader can find**. That is the grain the audit's own comment states, and the code does not implement it. The cheapest instrument that matches the statement is a test per line or per record. SARIF and JSON have a record per finding, Markdown has a table row, and the text report has a finding block. None of the four needs a parser to cut into units.

The bar for closing this record is a probe rather than an argument. Take an artifact of each format, remove its findings, and leave every other block intact. The audit must then report **every** finding of the run as unaccounted. That probe is what measured the number above, and it belongs beside the audit rather than in a scratch directory.

**Nothing here says the four formats drop a finding today.** Each of them carries every finding of the fixture run, and the recorded artifacts show it. What this record holds is that the audit would not say so if one of them stopped.
