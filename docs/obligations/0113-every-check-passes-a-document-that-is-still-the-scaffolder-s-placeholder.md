---
id: OBL-repo-0113
title: "Every check passes a document that is still the scaffolder's placeholder"
status: current
status_since: 2026-08-14
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
    - SPEC-HW-check-layer
    - DR-repo-0007
---

# Every check passes a document that is still the scaffolder's placeholder

## Context

The section contract reads a heading and never what is under it. `section.required.missing` reports a heading the kind requires and the document does not carry. Nothing reports a heading with the scaffolder's own prompt beneath it. Nothing reads the `summary` facet for content either, so the prompt the verb writes into that field is a value every rule accepts.

The [MCP working-tree write class](../spec/05-ai-integration.md#what-the-working-tree-write-class-registers-and-what-a-session-looks-like-after-a-write) is why this record is filed now. A person who runs the verb at a terminal is looking at the file. A tool call puts the same document into a checkout with nobody looking. [Q7](../spec/09-decisions.md#q7--scope-of-the-mcp-surface) rests the whole write path on the human who reviews the diff.

## Obligation

The corpus owes a rule that separates an authored document from a scaffolded one, or a statement that no such rule is wanted. The second answer is defensible: the remedy for an empty section is a rewrite, and [spec 4](../spec/04-assurance-model.md#where-promotion-cannot-finish) makes such a category advisory whatever its rate. An advisory finding still names the file, and this run names nothing.

The narrower half is mechanical and total. The scaffolder writes a known string into every section it opens and into the summary. A rule that reported that exact string would have one correct outcome, which is the [fixability](../spec/12-check-layer.md#fixability) bar for an error. It would also read a string that this engine writes, rather than one that an author might mean.

## Discharge

Measured on 2026-08-14 over a copy of this tree. The tree carried 27 findings and `headwater check --strict` exited 0. The `new` tool of an MCP server started with `--write` then authored a decision record titled "Every check may be disabled by an operator". Its three sections each read `TODO write this section.`, and its summary read the scaffolder's prompt. The tree then carried 27 findings, and the strict run still exited 0. No finding named the document.

The sweep now names the class, and the narrower half is still unbuilt. `unwritten_section` is a class of the [assisted sweep](../spec/04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep), so a proposal about a document like this one can reach a reader. That closes nothing on its own. A sweep is opt-in, its finding is a model's judgment, and no run of the checks produces one. The mechanical half that this record's second paragraph describes has no writer. `engine/crates/sweep/fixtures/corpus/0003-every-check-may-be-disabled-by-an-operator.md` is the document, and the test `no_check_names_the_placeholder_document_and_the_sweep_does` holds both halves of that sentence.

Two things did record it, and neither is a check. The capture-cost store took a line naming `protocol` as the surface of the run, beside the document, in one working tree. `headwater generate --check` then failed, because the `decisions` shelf carries a generated index and the new document is not in it. That second one is a weaker guard than it looks. It reports a stale projection rather than a bad document, a run of `headwater generate` clears it, and the `obligations` shelf carries no index at all.
