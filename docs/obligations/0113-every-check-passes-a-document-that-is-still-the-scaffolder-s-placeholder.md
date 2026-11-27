---
id: HW-OBL-0113
title: "Every check passes a document that is still the scaffolder's placeholder"
status: discharged
status_since: 2026-11-27
waiting_on: build
last_verified: 2026-08-14
summary: "A decision record whose three sections each read TODO raised no finding at all, and the count over the corpus did not move."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-check-layer
    - HW-DR-0007
---

# Every check passes a document that is still the scaffolder's placeholder

## Context

The section contract reads a heading and never what is under it. `section.required.missing` reports a heading the kind requires and the document does not carry. Nothing reports a heading with the scaffolder's own prompt beneath it. Nothing reads the `summary` facet for content either, so the prompt the verb writes into that field is a value every rule accepts.

The [MCP working-tree write class](../spec/05-ai-integration.md#what-the-working-tree-write-class-registers-and-what-a-session-looks-like-after-a-write) is why this record is filed now. A person who runs the verb at a terminal is looking at the file. A tool call puts the same document into a checkout with nobody looking. [Q7](../spec/09-decisions.md#q7--scope-of-the-mcp-surface) rests the whole write path on the human who reviews the diff.

## Obligation

The corpus owes a rule that separates an authored document from a scaffolded one, or a statement that no such rule is wanted. The second answer is defensible: the remedy for an empty section is a rewrite, and [spec 4](../spec/04-assurance-model.md#where-promotion-cannot-finish) makes such a category advisory whatever its rate. An advisory finding still names the file, and this run names nothing.

The narrower half is mechanical and total. The scaffolder writes a known string into every section it opens and into the summary. A rule that reported that exact string would detect every case with no judgment. It would also read a string that this engine writes, rather than one that an author might mean. Total detection is not the [fixability](../spec/12-check-layer.md#fixability) bar for an error, which reads the remedy. The remedy here is to write the section, and that is a rewrite, so the finding is advisory.

## Discharge

Measured on 2026-08-14 over a copy of this tree. The tree carried 27 findings and `headwater check --strict` exited 0. The `new` tool of an MCP server started with `--write` then authored a decision record titled "Every check may be disabled by an operator". Its three sections each read `TODO write this section.`, and its summary read the scaffolder's prompt. The tree then carried 27 findings, and the strict run still exited 0. No finding named the document.

The sweep names the class. `unwritten_section` is a class of the [assisted sweep](../spec/04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep), so a proposal about a document like this one can reach a reader. That closed nothing on its own. A sweep is opt-in, its finding is a model's judgment, and no run of the checks produces one.

**`scaffold.placeholder.unwritten` discharges this record ([#1491](https://github.com/headwater-ai/headwater/issues/1491), slice 3, [PR #2139](https://github.com/headwater-ai/headwater/pull/2139)).** It is the narrower half that the second paragraph of the Obligation section describes. `headwater_check::placeholder` implements it, and the scaffolder reads its three prompts from that module, so the writer and the rule share one copy of each string. The rule reports two things. The first is a paragraph whose whole text is `TODO write this section.`, outside a code block, a code span and a block quotation. The second is a top-level facet whose value is the prompt that the scaffolder writes for that facet. Each finding names the file and the line. The rule reads a document at every lifecycle state, `draft` included, because the measured case above was a draft.

The finding is a warning, so a strict run still exits 0. The Obligation section gives the reason. An adopter who wants a placeholder to fail a strict run binds the rule with a control whose posture is blocking. `headwater/standard` 4.31.0 binds the rule to `OB-SECT-2` through the control `CT-SECT-2` ([#1492](https://github.com/headwater-ai/headwater/issues/1492)).

`engine/crates/sweep/fixtures/corpus/0003-every-check-may-be-disabled-by-an-operator.md` is the document, and the test `a_check_names_the_placeholder_document_and_the_sweep_still_does` holds both halves. The rule names it three times, once for each of its two sections and once for its summary, and the sweep still carries its finding. `engine/crates/cli/tests/placeholder_unwritten.rs` runs `headwater new` and then `headwater check`, which is the path of an adopter. `engine/crates/check/tests/placeholder_unwritten.rs` holds the cases that a looser rule would report.

Two things did record it, and neither is a check. The capture-cost store took a line naming `protocol` as the surface of the run, beside the document, in one working tree. `headwater generate --check` then failed, because the `decisions` shelf carries a generated index and the new document is not in it. That second one is a weaker guard than it looks. It reports a stale projection rather than a bad document, a run of `headwater generate` clears it, and the `obligations` shelf carries no index at all.
