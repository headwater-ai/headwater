---
id: HW-HOW-keep-derived-files-from-conflicting-in-parallel-pull-requests
status: current
status_since: 2026-09-27
summary: "Commit the derived files a person reads on the forge, compute the graph export where a build step reads it, and union the capture-cost store."
last_verified: 2026-09-27
title: "Keep derived files from conflicting in parallel pull requests"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0049
    - engine/crates/cli/tests/merge_driver.rs
---

# Keep derived files from conflicting in parallel pull requests

**Audience:** an adopter of Headwater whose repository merges more than one pull request in a day.

`headwater generate` writes derived files, and each pull request that edits the corpus writes them again. A forge such as GitHub merges a pull request with no merge driver and reads no merge attribute. So when two open pull requests both change one derived file at the same lines, the second one shows a conflict. A pull request with a conflict runs no CI. This guide sorts your derived files into the ones you commit and the ones you compute.

## Before you start

- You have adopted Headwater, and `headwater generate --check` passes on your default branch.
- You know which build step or deploy reads each file that `headwater generate` writes. `headwater derived` lists the files.
- To add a document on a second shelf with `headwater new`, that shelf's kind needs an identifier. In the standard package, `decision` has one after `headwater init`. `specification` does not, and `headwater new specification` refuses until your overlay declares `kinds.specification.identifier` and its scheme. The refusal prints the two lines to add under `add:` in `.headwater/overlay.yml`. Paste them, run `headwater taxonomy resolve`, and run `headwater new specification` again ([#1264](https://github.com/headwater-ai/headwater/issues/1264)).

## Steps

1. **Commit a file that a person reads on the forge.** A shelf index, a register, `.headwater/taxonomy.lock` and `.headwater/corpus.json` are in this group. The `verified_revision` stamps in your documents are in it too, because a person writes them. A shelf index holds one row for each document, so two branches that add documents at different places in it merge as text.
2. **Compute a file that only a build step reads.** The graph export is the usual case. It holds the whole graph, so it moves on nearly every edit, and a committed copy conflicts on most pairs of pull requests. Remove each `graph_export` entry from the `add_to: projections:` block of `.headwater/overlay.yml`, and run `headwater taxonomy resolve`. An entry that states a `filter`, a `tombstone` or a profile other than `default` exists only on a declaration. Keep that entry, add `committed: false` to it, and run `headwater taxonomy resolve`. Then `headwater generate` does not write the file, and neither `--check` requires it.
3. **Add the path of the export to `.gitignore`, and delete the committed copy.** For example, add `.headwater/export.json` and run `git rm --cached .headwater/export.json`.
4. **Compute the export in the step that reads it.** Run `headwater export --format json > <path>` in that step, before the reader starts. With no `graph_export` declared, the one profile is `default`, so the command needs no `--profile`. For an entry that you kept with `committed: false`, run `headwater export --profile <name>` in that step instead. It writes the file to the output path that the entry declares.
5. **Run `headwater generate`, and commit the result.** The descriptor and the lock move once, because they record the projections that you declare.
6. **Make sure that `.gitattributes` has the line `.headwater/capture-cost.jsonl merge=union`.** Each run of `headwater new` adds one reading at the end of that file. So two branches that each add a document both change its last line, and git reports a conflict. `union` keeps the lines of both sides, and that result is correct for this file because no reading depends on another. `headwater init --git` writes this line, and the same line for `.headwater/adoption.jsonl`. It writes neither where a line of yours already names the file. If you ran `headwater init --git` with an earlier release, run it again, or add the line by hand.

## How to know it worked

- `headwater generate --check`, `headwater taxonomy resolve --check` and `git ls-files .headwater/export.json` all agree: the first two exit 0 and the last prints nothing.
- In a local merge or rebase, two branches that add documents to two different shelves with `headwater new` merge with no conflict. This includes a branch that edits a governed file and records a new `verified_revision`.
- A forge merge of the same two pull requests can still conflict on `.headwater/capture-cost.jsonl`. Nobody has measured whether GitHub applies `merge=union`. It did not apply `-merge` when this repository measured that attribute on 2026-09-24.
- One conflict stays, on a shelf whose identifiers are numbers. Two branches that each run `headwater new decision` both take the next number. The merge conflicts on the shelf index and on the claim file of that number under `.headwater/ids/`, which is by design ([HW-DR-0054](../decisions/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md)). `headwater generate` alone does not resolve it, because two documents then hold one identifier and `headwater check --strict` exits 1.

## If two branches took the same number

Do these steps on the branch that lands second. They are the same whether you merge the other branch into yours or rebase yours onto it.

1. Take the claim file from the branch that landed first, by its name: `git checkout origin/main -- .headwater/ids/<scheme>/<identifier>`. Do not use `--ours` or `--theirs`. A rebase swaps what those two names mean, and `--theirs` in a rebase writes your own claim over the landed one. That claim then names a file that is not on the tree, and neither `headwater check --strict` nor `headwater generate --check` reports it.
2. Give your document the next free number. Rename its file to the new number, and change its `id` to match.
3. Run `headwater check --fix`. It writes the claim file for the new number.
4. Run `headwater generate`, and commit the result, or run `git rebase --continue` in a rebase. `headwater check --strict` then exits 0.
5. Read the claim file of the landed number. It names the landed document, not yours.
