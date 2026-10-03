---
name: headwater-authoring
description: Write or revise a governed document in this repository, of any kind its taxonomy declares. Use when asked to record a finding, file a decision, or add a document to the corpus, and when `headwater new` refuses. It calls the engine for what the taxonomy decides and carries the judgment it leaves as hand entry.
---

# Headwater authoring

Every document in the corpus is typed by the taxonomy in `.headwater/taxonomy.lock`. The engine decides the placement, the identifier, the front matter and the section headings. What is left is judgment, and that is what this skill carries.

Nothing here restates a rule the engine holds. If a sentence below disagrees with the engine, the engine is right and this file is stale.

## Never write a new document by hand

Run the verb:

    headwater new <kind> --title "<the title>"

It writes the file, prints the origin of every value, and prints the relations the document may declare and this run did not. The kinds are the ones the lock declares. `headwater new` refuses a kind the lock does not declare, and its refusal names every kind the lock does declare.

The verb never overwrites. To change a title after the fact, edit the `title` facet and rename the file yourself, because no rule reads a shelf layout after a document is born.

**Every run that writes a document also appends one capture-cost reading, and that is why the verb matters beyond convenience.** The line goes to `.headwater/capture-cost.jsonl`, it holds counts and no prose, and it names no person and no agent. Commit it with the document it names: the two arrive in one working tree, and a reading whose document is not on the tree is reported as resolving to nothing. A document you write by any other route carries no reading at all, and `headwater capture` then reports a lower reach rather than the same one. A run that exits non-zero because the reading did not land prints the line to append by hand.

## What the run leaves you, and how to fill each one

The verb marks a field `hand entry` when no declaration determines it. Read its report rather than this list — the report is derived from the lock and this paragraph is not.

**The summary is the scent facet, and it is the highest-value sentence in the document.** Routing serves it as a pointer with no other cue, so it competes against the other documents on its shelf. Grade it against the shelf siblings: a summary that shares no discriminating term with them cannot separate them, and a summary that rephrases the title carries nothing the path did not. Write the finding, not the topic. `A shelf layout is read by the scaffolder and by nothing else` separates. `About shelf layouts` does not.

**The body under each required heading.** The headings come from the kind's section contract and the prose does not. Open a sibling of the same kind to see what one says under each heading, and write what this document adds.

**The provenance block.** The verb writes `warrant: asserted` and nothing else in it, because nobody has accepted the new document. Add any other member by hand, in the shape a sibling on the same shelf uses. An `accepted_by` that you add pairs only with `warrant: accepted`. Two checks read the block: `warrant.value.not_permitted` refuses a warrant outside the four values the engine permits, and `warrant.acceptance.unpaired` refuses an `accepted_by` that the warrant does not pair with.

**Never move a `warrant` to `accepted` yourself, on any document, at any time.** That move belongs to a person who has read the document. Propose it and let them make it.

**The `status` line opens at the initial state of the kind's regime.** `headwater new` writes that state, because a regime opens there. Which state a document holds when you propose it is a rule of this repository and not of the engine. Where the repository states no rule, ask.

## Relations

**Don't reconstruct the resolved relation set by hand.** A package, its bundles and `.headwater/overlay.yml` are unresolved layers, and a search scoped to one misses what the engine actually resolves: a relation declared once in the package is never repeated in a bundle. `headwater new <kind> --title "…"` already prints the resolved relations the document may declare, and `headwater explain <path>` reports the same for a document that already exists. Read one of those before you search the taxonomy sources.

The verb writes an edge only where the taxonomy declares `created_by: scaffold` or `created_by: agent` on the relation. Pass it as `--relates <relation>=<target>`, where the target is an identifier, or a path or pattern for a relation that admits an anchor. Where reciprocity is required and the new document opens at its initial state, the far half is owed, so the verb leaves the target alone and names the half in its report. After you promote the document, `headwater check --fix` writes it. Where the new document opens at a later state, the verb writes the far half at once. The verb writes no far half of a symmetric relation, and it sets no state on a target such as the one `supersedes` names. `headwater check --fix` writes that state once the new document leaves its initial state.

A relation that declares `created_by: author` is yours to type into the front matter. Two facts are worth knowing before you write an `agent` edge.

**A code-path anchor reaches every entry a pattern admits, and a bare path still reaches only itself.** Its raw value is a pattern over the tree, or a YAML sequence of patterns for one anchor over several files. So a document that governs a directory of forty files can propose one pattern instead of forty edges. Nothing forces an existing hand-typed edge to change: propose a pattern where it earns its keep, and say how many entries it reaches. The verb binds a target through the resolvers `headwater check` uses and prints how many entries it reaches. A pattern that matches no entry is refused, and nothing is written.

**An `agent` edge is a proposal, and the review of the pull request that carries it is the acceptance.** So an edge of yours reaches the main branch only through a pull request a person reviews. An anchor that binds only where the target cites the new document, such as one that the `comment-scan` resolver reads, binds once a Rust `//` or `/* */` comment in the file cites the new identifier. The verb names that comment in its report.

## The stop rules

These are the behaviors that pressure to be helpful breaks first, so they are stop conditions rather than preferences.

1. **No decision record with no external evidence.** If no work item, commit, discussion or measurement supports it, halt and ask. Where the human confirms that none exists, record the document as unevidenced. A fabricated rationale is worse than an admitted absence, because somebody will cite it.
2. **No hand edit to a generated file.** Change the source and run `headwater generate`.
3. **No new shelf, kind or facet invented in place.** That is a taxonomy change, and the `headwater-taxonomy` skill owns it.
4. **No second copy of a fact that exists elsewhere.** Link to it. Where the target is hard to find, repair its summary.
5. **No self-acceptance.** Never write the acceptance of your own work as a fact. Propose it to a person who reads it.

## A refusal is a declaration to add, and never a retry

`headwater new` decides everything before it writes anything, and it carries twenty-three refusals: nineteen from the decision, and four from the write, which puts every file on the tree or none of them. Two of the nineteen are the taxonomy telling you that a declaration is missing.

`Unnameable` means the kind names no identifier scheme and a relation may name a document of it, so the document would be neither end of any edge. Its message prints the two overlay lines that declare a scheme for the kind. `FacetUndeterminable` means a required facet declares a closed value set, an integer or a date that no role determines, and a prompt in that field is a value the checks refuse.

Neither is worked around. Hand both to the `headwater-taxonomy` skill, which owns the declaration. The one exception is a closed facet whose value is the author's to choose: pass `--facet <name>=<value>`, after you read what each value means.

## Finish with the engine

    headwater check

Errors must reach zero before a commit. `headwater check --fix` writes the corrections the engine derives without judgment — a British spelling, a contraction whose expansion is one word, a retired term that names a replacement, and a missing reciprocal half — and it leaves every finding whose remedy is a rewrite. Read the diff.

**An edit to a document that a probe reads moves no committed result.** A result pins only what its transcript read, so it keeps its bytes and every verdict. The next `headwater generate` names each result whose read set moved, and `headwater probe stale` names the results the edit reached. Neither one fails for it.
<!-- installed by headwater init --harness, digest sha256:5e62a09aa050ef94d7146f26e9d312f6200449370d04d99370d3131352b54e10 -->
