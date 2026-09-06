---
id: HW-OBL-0129
status: current
status_since: 2026-09-06
waiting_on: build
summary: "Spec 12 lists an unparseable file and an unclassifiable path among the structural findings of Phase A, and the engine emits no finding for either. The census and `corpus.classified` report both, which is what the control on OB-COV-1 declares, and no document says so."
last_verified: 2026-08-16
title: "Spec 12 calls two Phase A outcomes structural findings and the engine emits none"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-check-layer
    - HW-DR-0029
---

# Spec 12 calls two Phase A outcomes structural findings and the engine emits none

## Context

[Spec 12](../spec/12-check-layer.md#two-phases-and-why-the-order-matters) states what Phase A produces. "Failures here are structural findings: an unparseable file, an unclassifiable path, a dangling edge, an ambiguous shelf match." A reader takes that to mean four kinds of failure that a run reports as a finding.

**Two of the four were measured on a scratch copy of this corpus, and neither produces one.** A Markdown file with no front matter was put on `docs/probe-runs/`. The census counts it, moves the untyped row from two to three, and names the file. The finding count holds at 523 and `headwater check --strict` exits 0. A file with unterminated front matter was then put under `docs/spec/`. The same thing happened, at the same count and the same exit status.

**The third is a finding, and it is an error.** A `governs` edge pointed at a path that no longer exists reports `relation.target.unresolved` and takes a strict run to exit 1. So the sentence is right about a dangling edge and wrong about the two above. The fourth was not measured here.

**Something does report both, and it is not a check.** `headwater conformance` moves `corpus.classified` from a two-file gap to a three-file gap and names the new file, with a remedy beside it. `.headwater/taxonomy.yml` states in a comment that this gap carries no waiver, because the remedy is a declaration rather than a deviation.

## Obligation

The corpus owes one sentence that says which Phase A outcomes are findings and which are facts. What exists is a specification sentence that promises four findings. Beside it is a control declaration that promises facts for one of them, and no document reconciles the two.

**The reconciliation is already decided, and only the writing down is missing.** `OB-COV-1` reads "Every file under the corpus root is classified, or reported as unclassifiable". The control that discharges it declares `produces_facts` rather than a rule, and the comment above that block says what that is for. It names a way to discharge an obligation by doing the thing rather than by reporting a finding about it. So a census row is the discharge, by declaration, and it always was.

**What this costs a reader.** A person reads spec 12, and then reads a green strict run over a corpus holding three untyped files. They have to decide which of the two is wrong. Three such files stand in this corpus today. The right answer is that neither is wrong and the word "findings" is carrying two meanings, and no artifact in this repository says so.

## Discharge

**What would discharge this.** One correction in [spec 12](../spec/12-check-layer.md#two-phases-and-why-the-order-matters), separating the Phase A outcomes that a rule reports from the ones the census and the conformance ladder report. The evidence for the separation is the control declaration on `OB-COV-1`, which already draws it.

**What would not discharge this.** A new rule that turns an untyped file into a finding. That would move a decision the taxonomy has already taken, and it would take it in a diff about wording. It would also break the exemption this corpus relies on. `docs/doctrine/maturity-model.md` and `docs/w3id/README.md` stand untyped by choice, and the conformance gap is where that choice is reported.

**Why this record exists rather than a correction.** The sentence is a normative claim of a specification part, and a change to it is a change to what an adopter is promised. [Q29](../spec/09-decisions.md#q29--whether-a-corpus-root-may-contain-code-and-what-an-interface-contract-may-reach) leaned on the same sentence for a gloss and had to withdraw it, which is how this was found. The finding is filed here so that the correction is argued in its own change rather than inside a ruling about something else.
