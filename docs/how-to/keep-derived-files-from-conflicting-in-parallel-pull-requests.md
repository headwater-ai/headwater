---
id: HW-HOW-keep-derived-files-from-conflicting-in-parallel-pull-requests
status: current
status_since: 2026-09-27
summary: "Commit the derived files a person reads on the forge, and compute the graph export in the build step that reads it. One same-shelf conflict stays."
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

## Steps

1. **Commit a file that a person reads on the forge.** A shelf index, a register, `.headwater/taxonomy.lock` and `.headwater/corpus.json` are in this group. The `verified_revision` stamps in your documents are in it too, because a person writes them. A shelf index holds one row for each document, so two branches that add documents at different places in it merge as text.
2. **Compute a file that only a build step reads.** The graph export is the usual case. It holds the whole graph, so it moves on nearly every edit, and a committed copy conflicts on most pairs of pull requests. Remove each `graph_export` entry from the `add_to: projections:` block of `.headwater/overlay.yml`, and run `headwater taxonomy resolve`.
3. **Add the path of the export to `.gitignore`, and delete the committed copy.** For example, add `.headwater/export.json` and run `git rm --cached .headwater/export.json`.
4. **Compute the export in the step that reads it.** Run `headwater export --format json > <path>` in that step, before the reader starts. With no `graph_export` declared, the one profile is `default`, so the command needs no `--profile`.
5. **Run `headwater generate`, and commit the result.** The descriptor and the lock move once, because they record the projections that you declare.

## How to know it worked

- `headwater generate --check`, `headwater taxonomy resolve --check` and `git ls-files .headwater/export.json` all agree: the first two exit 0 and the last prints nothing.
- Two pull requests that add documents to two different shelves merge with no conflict. This includes a pull request that edits a governed file and records a new `verified_revision`.
- One conflict stays. Two branches that each add a document at the end of one shelf both insert a row at one position of the shelf index. Git reports a conflict on that file. Run `headwater generate` on the merged tree, and commit the index that it writes. No attribute or setting on the forge removes this conflict.
