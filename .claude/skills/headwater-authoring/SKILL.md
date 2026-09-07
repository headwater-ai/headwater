---
name: headwater-authoring
description: Draft or revise a governed document in this repository — a decision, an obligation record, an evaluation, a review record, a numbered specification part. Use when asked to record a finding, file a decision, write up an evaluation, or add a document under docs/, and when `headwater new` refuses. It calls the engine for everything the taxonomy decides and carries only the judgment the engine reports as hand entry.
---

# Headwater authoring

Every document under `docs/` is typed by the taxonomy in `.headwater/taxonomy.lock`. The engine decides the placement, the identifier, the front matter and the section headings. What is left is judgment, and that is what this skill carries.

Nothing here restates a rule the engine holds. If a sentence below disagrees with the engine, the engine is right and this file is stale — `.claude/skills/fixtures.sh` is what catches that.

## Never write a new document by hand

Run the verb:

    headwater new <kind> --title "<the title>"

It writes the file, prints the origin of every value, and prints the relations the document may declare and this run did not. The concrete kinds are `acceptance_criterion`, `decision`, `decision_register`, `design_spec`, `evaluation`, `interface_contract`, `obligation_record`, `obligation_register`, `probe`, `probe_result`, `probe_transcript`, `process_decision`, `requirement`, `review_prompt`, `review_record`, `specification` and `tutorial`.

A `Write` of a new document under `docs/` is refused by the `PreToolUse` hook, which names this verb. An `Edit` of a document that already exists passes, and that is the repair path: scaffold first, then edit the file the verb wrote.

The verb never overwrites. To change a title after the fact, edit the `title` facet and rename the file yourself, because [HW-OBL-0106](../../../docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md) records that no rule reads a shelf layout after birth.

**Every run that writes a document also appends one capture-cost reading, and that is why the verb matters beyond convenience.** The line goes to `.headwater/capture-cost.jsonl`, it holds counts and no prose, and it names no person and no agent. Commit it with the document it names: the two arrive in one working tree, and a reading whose document is not on the tree is reported as resolving to nothing. A document you write by any other route carries no reading at all, and `headwater capture` then reports a lower reach rather than the same one. A run that exits non-zero because the reading did not land prints the line to append by hand.

## What the run leaves you, and how to fill each one

The verb marks a field `hand entry` when no declaration determines it. Read its report rather than this list — the report is derived from the lock and this paragraph is not.

**The summary is the scent facet, and it is the highest-value sentence in the document.** Routing serves it as a pointer with no other cue, so it competes against the other documents on its shelf. Grade it against the shelf siblings: a summary that shares no discriminating term with them cannot separate them, and a summary that rephrases the title carries nothing the path did not. Write the finding, not the topic. `A shelf layout is read by the scaffolder and by nothing else` separates. `About shelf layouts` does not.

**The body under each required heading.** The headings come from the kind's section contract and the prose does not. An `obligation_record` states the context, what the corpus owes, and what would discharge it. A `decision` states the context, the decision, and the consequences.

**The provenance block.** No taxonomy declares one, so the verb writes none while every hand-typed document carries one — [HW-OBL-0030](../../../docs/obligations/0030-the-provenance-block-belongs-to-the-engine-and-nothing-states.md) holds that gap. Copy the shape from a sibling on the same shelf, and know that no check reads a single field of what you wrote.

**The `accepted_by` line answers to the merge rather than to the byte you write.** [HW-DR-0034](../../../docs/decisions/0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md) rules that acceptance is the merge onto `main`, so a provenance block on a branch states a proposal and nothing more. You may write `accepted_by` and `warrant: accepted` on a document you draft, where that document goes into a pull request a named human reads before the merge. Say in the proposal that the stamp is part of what you are asking them to accept. Where nobody will read the document before it lands, and an autonomous run that merges its own request is that case, write `warrant: asserted` and no `accepted_by`. Both shapes are compliant and the ruling prefers neither. Never move a `warrant` to `accepted` on a document that is already on `main`, because that edits an accepted document rather than stamping a draft. [HW-OBL-0108](../../../docs/obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md) holds the count that raised the ruling and is discharged by it.

**The `status` line answers to the merge the same way, and the scaffolder does not write the value you want.** [HW-DR-0052](../../../docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md) rules that an author writes the state the document will hold once the branch lands, and that the merge activates it. `headwater new` writes `draft`, because a regime opens at its initial state. Move it to `current` before you propose a document as finished, and leave `draft` only where you are asking a reader to comment rather than to rely. A merged document standing at `draft` is the defect that ruling was written for, and copying the front matter of a sibling is how it spreads.

## Relations

The verb writes an edge only where the taxonomy declares `created_by: scaffold` on the relation. Pass it as `--relates <relation>=<identifier>`, and it writes the far half into the target document where reciprocity is required.

Every other relation is yours to type into the front matter, and two of them carry a cost worth knowing before you propose one.

**`governs` reaches the path it names and no path under it.** The match is string equality against the value on the edge, so a `governs` edge onto a directory reaches nothing at all, and a document that governs a directory of forty files owes forty edges. [HW-OBL-0104](../../../docs/obligations/0104-a-governs-edge-reaches-the-path-it-names-and-nothing.md) prices it. Propose the edges for the files the change actually touched, and say how many the full set would be, rather than proposing a set nobody will maintain.

**`governs` and `traces_to` both declare `created_by: hook`, and no verb writes either.** [HW-OBL-0105](../../../docs/obligations/0105-nothing-plays-the-hook-role-that-two-relations-name.md) records that every such edge in this corpus is hand entry. So a proposal of yours is the only mechanism there is, and it goes to a human rather than into a file you wrote alone.

## The stop rules

These are the behaviors that pressure to be helpful breaks first, and spec 5 states them as stop conditions rather than preferences.

1. **No decision record with no external evidence.** If no work item, commit, discussion or measurement supports it, halt and ask. Where the human confirms that none exists, record the document as unevidenced. A fabricated rationale is worse than an admitted absence, because somebody will cite it.
2. **No hand edit to a generated file.** Change the source and run `headwater generate`.
3. **No new shelf, kind or facet invented in place.** That is a taxonomy change, and the `headwater-taxonomy` skill owns it.
4. **No second copy of a fact that exists elsewhere.** Link to it. Where the target is hard to find, repair its summary.
5. **No self-acceptance.** Self-acceptance is a document that reaches `main` with no human review between the draft and the merge, and it is not a value an agent typed on a branch ([HW-DR-0034](../../../docs/decisions/0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md)). Write a stamp for a pull request a named human reads. Mark the document `asserted` where nothing stands between your draft and `main`.

## A refusal is a declaration to add, and never a retry

`headwater new` decides everything before it writes anything, and it carries twenty-three refusals: nineteen from the decision, and four from the write, which puts every file on the tree or none of them. Two of the nineteen are the taxonomy telling you that a declaration is missing.

`Unnameable` means the kind names no identifier scheme and a relation may name a document of it, so the document would be neither end of any edge. `FacetUndeterminable` means a required facet declares a closed value set, an integer or a date that no role determines, and a prompt in that field is a value the checks refuse.

Neither is worked around. Hand both to the `headwater-taxonomy` skill, which owns the declaration.

**One kind meets `FacetUndeterminable` by design, and it takes a flag rather than a declaration.** `obligation_record` requires `waiting_on`, which is closed to `ruling`, `build`, `measurement` and `adopter`, and no role derives a value for it. Read the Discharge section you are about to write, and pass `--facet waiting_on=<value>`. [HW-DR-0030](../../../docs/decisions/0030-q30-whether-what-an-obligation-waits-on-is-a-state-or-a-property-and-how-many-values-it-takes.md) states what each value means. The refusal is the point: a record that does not say what it waits on is not written.

## Finish with the engine

    headwater check

Errors must reach zero, which is what the commit gate holds. `headwater check --fix` writes the corrections the engine derives without judgment — a British spelling, a contraction whose expansion is one word, a retired term that names a replacement, and a missing reciprocal half — and it leaves every finding whose remedy is a rewrite. Read the diff.

Prose under `docs/spec/`, `docs/decisions/`, `docs/evaluations/`, `docs/obligations/`, `docs/interfaces/`, `docs/requirements/`, `docs/acceptance-criteria/`, `docs/tutorials/` and `docs/process/decisions/` answers to the `ste_house` language regime. Invoke the `ste-editor` skill for the rules that no check reads.

Adding or removing a document under `docs/` moves three recorded fixtures. Run `cargo test --workspace --manifest-path engine/Cargo.toml`, re-record with `HEADWATER_BLESS=1`, and read the diff: `check --strict` and `generate --check` both pass while all three are stale.
