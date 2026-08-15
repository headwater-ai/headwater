---
id: OBL-repo-0104
title: "A governs edge reaches the path it names and nothing under it"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "Impact detection compares the edited path against the anchor by equality, so an edge onto a directory answers for no file inside it."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - SPEC-HW-ai-integration
    - SPEC-HW-taxonomy-model
    - SPEC-HW-conceptual-model
---

# A governs edge reaches the path it names and nothing under it

## Context

[Spec 5](../spec/05-ai-integration.md#write-time-hooks) states that a document can declare that it governs code, and that an edit to that code raises an advisory prompt. The [glossary](../spec/glossary.md) repeats it. [Spec 2](../spec/02-taxonomy-model.md#behavior-at-the-limits) makes anchor resolution a correctness root for the same reason.

`Surface::governing_docs_for_path` answers the query, and it compares the path it is asked about against the string each edge reached. The comparison is equality. Its own doc comment states the rule: the path is matched against what an edge reached rather than against a pattern. A resolver normalizes two spellings of one target into one node, and that is the whole of what the comparison tolerates.

The write-time hook of [#72](https://github.com/headwater-ai/headwater/issues/72) measured the consequence. This repository first declared one `governs` edge from spec 5 onto `.claude/hooks`, which is the directory that holds the three hooks. The harness then reported an edit to `.claude/hooks/write.sh`. The engine reached no governing document and the hook wrote nothing. That answer reads exactly like the answer over a path that nothing governs at all.

An anchor also carries no pattern syntax. `normalize` is lexical and `source-tree` resolves the result against the tree. So `.claude/hooks/**` is a path that no file answers to, and the edge reports as unresolved.

This repository now declares four edges, one for each file under that directory, and the hook fires. Four edges for one fact is the measurement rather than the fix.

## Obligation

The corpus owes a ruling on what a `code_path` anchor denotes. One file, or a subtree, and spec 2 states neither.

Under equality, an author pays one edge for every file that a document governs. [Spec 3](../spec/03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) names that cost as the thing that killed every prior design-rationale tool. A directory of forty files takes forty hand-typed edges, and [OBL-repo-0105](0105-nothing-plays-the-hook-role-that-two-relations-name.md) records that nothing mechanical writes even one of them.

The failure mode is silence rather than error. A hook that finds no governing document exits 0 and reports nothing. A rename inside a governed directory produces an unresolved anchor, which a check does report, but a *new* file in that directory produces nothing anywhere. Impact detection is then green over code that a document governs in every sense except the one the engine reads.

**The same silence covers a set that names no directory at all.** [DR-repo-0024](../decisions/0024-q24-readability-and-what-a-sweep-can-be-asked-about.md) rules over three files, and the agent that drafted it named one. Nothing reported the two it left out. The anchor resolved, every check passed, and a document that governs one of three reads exactly like a document that governs one. So the gap is wider than the directory case above. A `governs` set has no denominator, and the author is the only reader who knows the full set.

The [first-run walkthrough](../evaluations/default-taxonomy-first-run.md) calls impact detection the most valuable thing the corpus does for a coding agent. Spec 5 makes the same claim in the same words. A mechanism that reaches one file for each declared edge is a smaller claim, and no document states which one holds.

## Discharge

A ruling on the denotation, and the fixtures that hold it.

Where the ruling keeps equality, spec 5 and the glossary owe one sentence. It says that an author declares one edge for each governed file. The [maturity model](../doctrine/maturity-model.md) owes it too. That document asks an adopter to install the write-time hooks, and says nothing about what an adopter then declares.

Where the ruling admits a subtree, `governing_docs_for_path` owes a containment test and three failing fixtures. They are a path equal to the anchor, a path under it, and a path that shares a prefix with the anchor and leaves it. `.claude/hooks` against `.claude/hooks-disabled/write.sh` is the third, and a naive prefix test passes it wrongly.

Either ruling closes this record. The corpus states no answer today, and the four edges this repository declares are what the absent answer costs.
