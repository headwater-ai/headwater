---
id: HW-DR-0049
status: draft
status_since: 2026-09-06
summary: "A recorded artifact holds one record per entity and derives every total, because two branches that each add one document write the same new total and a merge takes it without a conflict."
last_verified: 2026-09-06
title: "A corpus-wide fold is derived and never stored"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-EVAL-what-a-check-can-know
    - engine/crates/census/src/census.rs
    - engine/crates/graph/src/lib.rs
    - .githooks/merge-regenerate
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

**Where decomposition costs more than it returns, the artifact keeps its fold and answers to a check on the merged state.** One record per check instance would be a file of five thousand lines that moves on every commit. Such an artifact declares `merge=headwater-regenerate`, which refuses a merge rather than reconciling one. The refusal is a fallback and never the rule, because the measurement above shows that it reaches only the form git already reports.

**No git attribute is ever treated as covering the quiet form.** A future artifact that stores a fold is not made safe by an attribute, and a reader who believes otherwise will store one.

**The trade against blessed-diff review is named, because issue #494 asks for it.** The concern was that a shape which hides or resolves a count makes a real regression easier to miss. This rule hides nothing and resolves nothing. It removes a summary line and adds the records that the summary counted. A shrinking denominator is the failure the census exists to catch. It now appears as a deleted record that names the file that left the tree. The old form reported the same event as a number that fell by one and named nothing. The diff grows, and a person still reads it before blessing it.

## Consequences

**Two artifacts changed shape.** The recorded census holds one record per file and no total. The recorded graph holds one line per citation rather than a count for each anchor. It drops the count of prose links that did not resolve, because the exceptions section already names each broken link.

**Six artifacts keep their folds and declare the driver.** They are the recorded checks, the two locks, and the three artifacts that `headwater generate` writes. `.gitattributes` names each one and says why it is on the list.

**The driver needs one command for each clone, and the commit gate reports a clone that has not run it.** Git takes no merge driver from a repository, because a driver is an executable. So the attribute alone leaves an ordinary merge in place, and that failure is silent by construction.

**One exposure stays open and no mechanism here closes it.** An artifact that keeps its fold still merges quietly when the branch that carries it is behind the default branch. What covers that is a check on the merged state before landing, and a pull-request run already builds the merge rather than the branch tip. That run is conclusive exactly while the branch is current. A merge queue is the usual guarantee for currency, and a private repository under a free plan may not have one.

**So currency is asked for and not enforced, until the repository is public.** Nothing in a free private repository refuses a merge of a stale branch. `.claude/commands/next-run.md` states the request where a run reads it, and the `headwater-engine` skill states it where an agent that blesses an artifact reads it. Both cite this record rather than restating the rule. When branch protection becomes available, the request becomes a required check and this paragraph is what it replaces.

**An adopter inherits the rule rather than the artifacts.** Any corpus that two people edit in parallel meets the same anomaly in any artifact that stores a count over the whole corpus. The rule is the transferable part, and the two artifacts above are this repository's application of it.
