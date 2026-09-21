---
id: HW-EVAL-two-triggers-one-verdict-how-this-repository-s-ci-decides-where-a-job-runs-and-what-it-trusts
status: current
status_since: 2026-09-21
summary: "The event that started a run decides which of two machines it takes, and every other condition in the workflow can only take work away."
last_verified: 2026-09-21
title: "Two triggers, one verdict: how this repository's CI decides where a job runs and what it trusts"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - .github/workflows/ci.yml
---

# Two triggers, one verdict: how this repository's CI decides where a job runs and what it trusts

This repository is public, it has one maintainer, and since 2026-09-21 it builds itself on self-hosted runners. Those three facts together decide the shape of `.github/workflows/ci.yml`, and until this document nothing stated the shape as a whole. The rules lived in the header comment of the workflow. That is where a person editing the file meets them, and nowhere a person asking how the arrangement works would look. This evaluation is that statement. The workflow keeps the rules an editor must obey, and this document carries the reasoning and the measurements behind them.

The reader outside this repository is an adopter who runs `headwater check` over a corpus of their own in CI. They are considering a self-hosted runner because a hosted one is slow. The security argument below is the part that transfers, and it transfers with nothing changed. It is a property of how GitHub dispatches workflow runs, rather than a property of this corpus. The last section separates that from the part that belongs here.

Three subjects sit next door and are not here. [DEVELOPING.md](../../DEVELOPING.md) carries what CI runs, gate by gate. A fixture suite holds that list against the workflow in both directions, so this document does not repeat it. The runner containers are provisioned outside this repository, and their threat model and incident runbook live with that provisioning. [One clone, many agents](one-clone-many-agents-how-this-repository-isolates-the-sessions-that-build-it.md) carries how several sessions share one checkout, which is the question next to this one and not this one.

## The only boundary is the event, and it is the only one that can be

A fork can open a pull request against this repository. That pull request runs **the fork's own copy of the workflow file**, including any line the fork rewrote. So no condition written here defends against a hostile fork. A fork may delete a condition, invert it, or hardcode the self-hosted labels in place of the expression that chooses them.

One fact is not like the others. A `push` event fires only for a ref actually written to this repository, and writing one needs an access nobody outside the project holds. A fork pushes to the fork, and that fires a `push` in the fork. So `github.event_name == 'push'` is a claim about how the run was created, and a fork cannot make it true.

That is why self-hosted eligibility is written as a test on the event and never as a test on which repository opened the pull request. The second test reads `github.event.pull_request.head.repo.full_name`, which is webhook data that the pull request's author cannot forge. The distinction that matters is not whether the field is trustworthy. It is that the *condition reading it* lives in a file the author owns on their own fork. So the field is sound as data and unsound as a gate.

What actually stops a hostile fork's run is a repository setting rather than anything in the workflow. `fork_pr_contributor_approval` stands at `all_external_contributors`, which is the strictest value GitHub offers. Every run started by anyone outside this repository waits for a manual approval before a step executes. The workflow header states the rule that follows: never approve a run whose diff touches `.github/` without reading that diff first. Nothing enforces that sentence, and no expression substitutes for it.

A `push` is trusted in a narrow sense, and the narrowness matters. Trusted means written by someone with commit access, and not read by a person before it ran. The build-order agents push branches and open pull requests before anyone reviews them. A dependency that a `cargo build` pulls in can also run arbitrary code as its own build script. So the runner is one ephemeral container per job, and the container is what bounds a compromised dependency. It confines one to that job's cache volumes, and to a network that reaches nothing beside the runner.

## A condition that only takes work away is safe, and that is why one run per commit is safe

Until 2026-09-21 a pull request opened from a branch of this repository built the same commit twice. A `push` run built it on the self-hosted containers, and a `pull_request` run built it again on a GitHub-hosted runner. Both reported under the same two names onto the same head commit.

Both jobs now carry a condition that skips the `pull_request` run when the head repository is this repository. Deleting that condition, which a fork may do, gives the fork the run it would have had anyway: a GitHub-hosted runner, behind the same approval. Inverting it makes the fork's own run skip. The condition decides which of two runs does the work, and never which machine a run may use. So it grants a fork nothing it did not already hold.

The `pull_request` trigger stays, and the reason is a distinction GitHub documents and this repository depends on. A job skipped by its own condition reports success to a required status check. A **workflow** skipped by a trigger filter reports nothing at all, and the required check then waits forever. Removing the trigger would therefore make every fork pull request unmergeable, because a fork has no `push` run here to answer in its place.

## Two check-runs of one name, and what settles between them

A pull request from this repository now puts two check-runs of each required name on its head commit. One is the real conclusion from the `push` run, and one is a skipped result from the `pull_request` run. A merge gate that read only the most recent of those would be a gate a skip could satisfy.

GitHub reads them together rather than taking the latest. Its troubleshooting page states the rule for a name shared by two sources, which is that both must pass. The failure it documents for duplicate job names is a blocked merge rather than a masked one. This repository has measured the case. On 2026-09-21 commit `6751719d` failed both jobs on its `push` run while its `pull_request` run skipped, and the pull request read `BLOCKED`. Three superseded commits on pull requests #999 and #1001 carry the same disagreement in both completion orders, and each reports a combined state of failure.

One attempt to avoid the duplicate is recorded here because it looks correct and is not. A job name accepts an expression, so a skipped job can be given a name the ruleset does not require. The `push` run evaluates such an expression correctly. A job that GitHub **skips** does not: commit `0bf99400` reported both skipped check-runs with the raw expression text as their name. The names are literals, and they have to stay literals.

## What a pull request is tested at, and what nobody tests before a merge

A `push` run checks out the branch. So a pull request is tested at its head and never at its merge with `main`.

The run this replaced did check out the merge commit, and it is worth being exact about how much that was worth. The `pull_request` event fires on `opened`, `synchronize` and `reopened`, and all three are about the head branch. A base branch that moves afterwards causes GitHub to recompute the merge ref and starts no run, so the green check is never recomputed. That test therefore covered the merge with `main` as `main` stood at the last push to the head. Of the eight pull requests merged before the change, four went in with a head one commit behind `main`. For those four the check had certified a composition that never landed.

The run on `main` is the composition check, and it always was. It is the one run the workflow never cancels, for that reason. Spec 12 already rules the gap ordinary. Hook and CI can disagree only when the mainline moved, and the evaluation that counts is the one against the final merge base.

Closing the gap exactly costs one ruleset field. Turning on `strict_required_status_checks_policy` would require every head to contain the tip of `main` before merging. Under squash-only merges with linear history, a head that contains `main` squashes into the tree it already has. The field is off, on a measurement rather than a preference. Of the last 40 runs on `main`, 39 were green. The one failure was `actions/checkout` erroring on a runner rather than a composition breaking. Zero composition failures in 40 merges does not pay for the re-runs that strict would add to every branch a merge leaves behind. The trigger for revisiting it is a `main` run going red from composition, more than once in a fortnight. Red from composition means both parents green alone and red together.

## The two runners cache different things, because they are different shapes

A GitHub-hosted runner starts empty. It restores a cargo cache and a pip cache through `actions/cache`, because without them every dependency rebuilds on every run.

The self-hosted container takes neither. Its cargo registry and its sccache directory are bind mounts that outlive each job's container. Restoring an archive over them would be an upload and a download, in place of a directory that is already correct.

Its `engine/target` is deliberately **not** carried between jobs. Two attempts to carry it broke the checkout and then the fixture suite in turn. A persistent directory inside the checked-out tree is a directory `actions/checkout` cannot remove. A symlink to one is a file the fixture scratch copies collect. So a self-hosted job always builds into a cold target directory, and sccache is what makes that cheap.

A run's cancellation is keyed on the ref rather than on the event. Keying it on the event, which is what the workflow did until 2026-09-21, exempted from cancellation exactly the runs that occupy the runner. The queue that made cancellation worth having is made of `push` runs. Every ref but `main` cancels a superseded run, and `main` never does.

## What transfers, and what belongs to this repository

The event argument transfers whole. Any adopter with a public repository and a self-hosted runner faces the same asymmetry, and the same three settings answer it. Route the runner on the event. Hold fork pull requests for approval. Read the diff of anything that touches the workflow directory before approving it. The reasoning about skipped jobs against skipped workflows transfers too, because it is a property of required status checks rather than of any corpus.

What does not transfer is every number above. The cadence that makes strict status checks too expensive here is this repository's cadence. An adopter who merges twice a week should turn strict on rather than off. The split between a hosted cache and a bind mount is a property of owning the host. The decision to build into a cold target directory is the consequence of two specific failures rather than a general rule.

Two records name defects in the file this document governs, and neither is closed by it. [HW-OBL-0145](../obligations/0145-the-ci-job-named-advisory-carries-every-blocking-shell-suite-in-the-repository.md) records that the job named advisory blocks on 48 of its 55 steps. [HW-OBL-0173](../obligations/0173-four-inline-gates-in-the-workflow-decide-an-exit-status-and-no-suite-provokes-one.md) records that four inline gates in the workflow decide an exit status and no suite provokes one.
