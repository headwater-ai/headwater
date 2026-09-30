---
id: HW-DR-0049
status: current
status_since: 2026-09-23
summary: "A recorded artifact holds one record per entity and derives every total, because two branches that each add one document write the same new total and a merge takes it without a conflict."
last_verified: 2026-10-01
title: "A corpus-wide fold is derived and never stored"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  governs:
    - to: .gitattributes
      verified_revision: sha256:9bb7abf94b7983f37f6803014e77508f73bf194a61a2e7742024c656fac1af19
  traces_to:
    - HW-EVAL-what-a-check-can-know
    - engine/crates/census/src/census.rs
    - engine/crates/graph/src/lib.rs
    - .githooks/merge-regenerate
    - HW-IFACE-headwater-init
    - HW-DR-0097
    - HW-PD-0020
---

# A corpus-wide fold is derived and never stored

## Context

**A recorded artifact of this repository held totals over the whole corpus.** The census opened with the count of files under the corpus root. The graph opened with the count of nodes and of declared edge halves, and it gave each anchor the number of edges that reach it. Each of those numbers is a fold over the records below it.

**A fold does not survive a merge, and the failure has two forms.** Issue #494 opened on the loud form. Two branches wrote different totals, git reported a conflict, and a person resolved it by hand. The second form is quiet. Two branches that each add one document both rewrite 386 to 387. A three-way merge reads one change written twice rather than two changes. It takes 387 for a tree that holds 388, and it reports nothing.

**The quiet form is the one that matters, and a measurement settles how to answer it.** Four git attributes were run over two branches that wrote the same value: `-merge`, `merge=binary`, `-diff -merge`, and a custom merge driver. All four merged clean and left the wrong number. A driver is never called at all, because git compares blobs before it selects a merge strategy, and two identical blobs need no merge. The same four attributes do raise a conflict when the two branches write different values. So the whole family covers the form that git already reports. None of it covers the form that git reports as clean.

**The evaluation states the general fact and this record fixes the rule that follows.** [What a check can know](../evaluations/what-a-check-can-know.md) names the anomaly write skew, under *The shapes a record takes*. It records the shapes and the measurement. A shape is a property of an artifact, so a ruling has to say which shape an artifact takes and who decides.

## Decision

**A fold over the corpus is derived and never stored.** A recorded artifact holds one record for each entity, in a fixed order, and it holds no count of those records. A reader who wants a total reads a report, and a report computes one. Two branches then write two different lines, and no blob on either side is identical. The merge of them is the correct artifact of the merged tree.

**Nothing is asserted less by this, and that is the ground the rule stands on.** A total is a function of the records that it counts. Records compared exactly are totals compared exactly. A rule that dropped an assertion to gain a merge would be a trade, and this is not one.

**The order is part of the artifact and not a presentation choice.** Two records that a walk emits in discovery order move under an edit somewhere else in the corpus. A test holds the order for each artifact that this rule covers.

**Where decomposition costs more than it returns, the artifact keeps its fold and answers to a check on the merged state.** One record per check instance would be a file of five thousand lines that moves on every commit. Such an artifact declares `-merge`, which refuses a merge rather than reconciling one. A configured clone selects `merge=headwater-regenerate` in its own `info/attributes`, and the conflict then names the producer. The refusal is a fallback and never the rule, because the measurement above shows that it reaches only the form git already reports.

**No git attribute is ever treated as covering the quiet form.** A future artifact that stores a fold is not made safe by an attribute, and a reader who believes otherwise will store one.

**The trade against blessed-diff review is named, because issue #494 asks for it.** The concern was that a shape which hides or resolves a count makes a real regression easier to miss. This rule hides nothing and resolves nothing. It removes a summary line and adds the records that the summary counted. A shrinking denominator is the failure the census exists to catch. It now appears as a deleted record that names the file that left the tree. The old form reported the same event as a number that fell by one and named nothing. The diff grows, and a person still reads it before blessing it.

## Consequences

**Two artifacts changed shape.** The recorded census holds one record per file and no total. The recorded graph holds one line per citation rather than a count for each anchor. It drops the count of prose links that did not resolve, because the exceptions section already names each broken link.

**The artifacts that keep their folds declare `-merge`, and `headwater derived` is what says which they are and of what shape.** Every count of them written by hand has been wrong, including the count this clause stated until [#676](https://github.com/headwater-ai/headwater/issues/676) removed it. The verb asks each producer for its own output set and takes the union, so the answer moves when the tree moves. `.gitattributes` names each path and says why it is on the list, and the verb holds that file against the computed set in both directions. The verb also computes the shape of each path, from the structure of the artifact and the rule of its producer. It reports a path whose merge attribute is not the treatment that shape takes.

**The driver needs one command for each clone, and the commit gate reports a clone that has not run it.** Git takes no merge driver from a repository, because a driver is an executable. The committed `-merge` needs no command, and it conflicts in every clone. The driver line goes in the `info/attributes` of a clone that set the driver config. Git reads a driver that no configuration defines as a text merge.

**Amended on 2026-09-24 by [#1058](https://github.com/headwater-ai/headwater/issues/1058).** Until then `.gitattributes` committed `merge=headwater-regenerate`, and every clone without the driver config merged each fold as text in silence. The same change measured every generated file. None of them states a count after #1058, and each one merged as text to the bytes its producer writes over the merged tree. So a generated file is one record per entity and carries no attribute. The lock, the recorded folds and the pages that carry figures keep `-merge`. (The amendment of 2026-09-29 below changes this sentence: no page under `site/` carries a figure or a merge attribute.) GitHub reads neither attribute, so a check of the merged tree in CI still covers the forge.

**Amended on 2026-09-27 by [#1251](https://github.com/headwater-ai/headwater/issues/1251): an artifact that only a build step reads is not committed.** This is a third shape beside "decompose" and "keep the fold under `-merge`". A forge reads no merge attribute, and a pull request that conflicts as text runs no CI and cannot join a merge queue. So for a committed file that conflicts, the only remedy is to stop the conflict. Two artifacts conflicted on most pairs of pull requests. Of 159 commits on `main` from 2026-09-20, `.headwater/export.json` moved in 72 and `engine/crates/check/fixtures/corpus.checks` moved in 48. `headwater export --format json` computes the export in one command, and no page, workflow or deploy read the committed copy. So this repository declares no `graph_export`, and `.gitignore` names the path. `corpus.checks` opened with a total over the corpus, and it is deleted. `check.report` over the pinned fixture tree stays the per-rule record, and the step summary of CI prints the totals of this repository. The Decision above names `corpus.checks` as the fold that sets the price, and the lock now sets it. The case that reopens a recorded total is an engine regression that `check.report` missed and a diff of the corpus totals would have caught. One conflict stays. Two branches that each mint the next number on one shelf conflict on the index of that shelf and on the claim file of the number, and the second branch renumbers its document. `engine/crates/cli/tests/merge_driver.rs` holds that case, and [Keep derived files from conflicting in parallel pull requests](../how-to/keep-derived-files-from-conflicting-in-parallel-pull-requests.md) states the rule for an adopter.

**Amended on 2026-09-29 by [#1273](https://github.com/headwater-ai/headwater/issues/1273): a page under `site/` carries no measured figure and no merge attribute.** The amendment of #1058 says that the pages that carry figures keep `-merge`, and since #1273 that sentence is false. [HW-DR-0097](0097-a-figure-on-a-hand-built-page-is-measured-when-the-site-is-published-and-the-committed-page-carries-none.md) moved each figure on those pages to the step that publishes the site. So a committed page carries a blank figure marker and no measured value. A page that states no total is not a fold, and `.gitattributes` does not list it. The lock and the recorded lock fixture keep `-merge`, and `.gitattributes` gives the reason for each. [HW-PD-0020](../process/decisions/0020-merges-go-through-the-github-merge-queue-one-squash-commit-per-pull-request.md) records what this change did to the merge queue.

**The merge queue closes the exposure of a stale branch.** An artifact that keeps its fold still merges quietly when the branch that carries it is behind the default branch. A check on the merged state before landing covers that case. A pull-request run builds the merge, but that run is conclusive only while the branch is current. The repository is public, and every merge goes through the GitHub merge queue ([HW-PD-0020](../process/decisions/0020-merges-go-through-the-github-merge-queue-one-squash-commit-per-pull-request.md)). The queue builds `main` and each queued pull request into one branch, and CI runs on the tip of that branch. So the queue enforces currency, and no request to rebase is the guarantee.

**One gap stays, and the queue catches it rather than preventing it.** GitHub merges with no custom merge driver, so a forge merge can write a derived artifact wrong. The projection step of the `headwater` job on the group tip fails on that artifact, and the queue ejects the pull request. HW-PD-0020 records that rule. The `headwater-engine` skill still tells an agent to rebase before it blesses. A branch that an agent blesses against a stale `main` is ejected from the queue.

**The identifier case is the same anomaly with a different mechanism, and it was met while this record was open.** Two branches minted `HW-DR-0048` under two slugs. The file names differ, so a merge takes both with no conflict of any kind, and neither branch is wrong on its own. `identity.duplicate` is an error at corpus grain, and it reads both claimants, so nothing is missing from what the rule checks.

**What it cannot do is run early enough.** No branch carries both claimants, so no run over either branch can see the pair. The rule is structurally unable to report before the second merge. That is a gap in when a check runs rather than in what it reads. It is the harder of the two to close. Every rule of this engine reads one tree, and this anomaly lives between two.

**So the two cases have one remedy.** A count and an identifier both need a run over the merged state, and neither needs a new rule. The fold reaches the same conclusion from an unrelated defect. That agreement is the evidence that the conclusion is about merges rather than about counts.

**An adopter inherits the rule, and `headwater init --git` gives the adopter the verbs and the configuration that hold it.** Any corpus that two people edit in parallel meets the same anomaly in any artifact that stores a count over the whole corpus. Since [#1058](https://github.com/headwater-ai/headwater/issues/1058), [`headwater init --git`](../interfaces/headwater-init.md) appends a `<path> -merge` line to `.gitattributes` for each fold that `headwater taxonomy resolve` and `headwater generate` write. It also prints the two `git config` lines that name `headwater merge-driver`, and the `info/attributes` lines that select that driver. The `--git-config` option runs the two `git config` lines in the adopter's own clone, and then appends the override lines to its `info/attributes`. Git takes no driver from a repository, so that explicit command is the consent of the clone ([HW-DR-0077](0077-the-consumer-surface-is-what-an-adopter-receives-runs-and-must-have-installed-and-it-is-a-closed-and-declared-list.md)).

**No attribute covers the quiet form in an adopter's repository either, so a check on the merged tree covers it there too.** That check is `headwater generate --check` and `headwater taxonomy resolve --check` in CI. The adopter receives no script of this repository, and no part of the gate that protects the published pages of this repository. The two artifacts above are this repository's application of the rule.
