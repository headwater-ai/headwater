---
id: HW-HOW-wire-headwater-check-into-your-own-workflow
status: current
status_since: 2026-09-23
summary: "A composite action runs headwater check --strict against your own corpus, with the released binary alone: no clone, no Rust toolchain, no cargo."
last_verified: 2026-09-25
title: "Wire headwater check into your own workflow"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-harness-support
  governs:
    - integrations/headwater-check/**
---

# Wire headwater check into your own workflow

**Audience:** an adopter of Headwater, in a repository of their own. The consumer surface in `.headwater/overlay.yml` lists this guide, and `headwater check` holds it to that list.

## Before you start

You need a GitHub Actions runner on Linux x86_64, for example `ubuntu-24.04` or `ubuntu-latest`. `headwater-ai/headwater` publishes a release binary for that platform alone. A runner of a different kind refuses cleanly, and it names the platform it needed. It does not download a binary that cannot run there.

You need no Rust toolchain and no `cargo` on the runner. The action downloads a release archive, checks its `sha256` sum, and runs the binary inside it. It never builds the engine. The checksum proves that the bytes the action downloaded are the bytes `headwater-ai/headwater` published under that tag. It does not sign or attest the release itself. The `.sha256` file and the archive it names both come from the same release. A compromise of the release process would carry a matching checksum too. Treat the download the way you already treat any other third-party action or binary in your pipeline.

Your workflow needs `permissions: security-events: write` if you want the SARIF report on GitHub's code-scanning page. Without that permission, the upload step fails and says so in the job's log. It does not fail your job by itself. The action still runs the checks. The job's own pass or fail still answers to `strict` and to what your corpus reports, not to the upload.

## Steps

Add one step to a workflow file, for example `.github/workflows/headwater-check.yml`: <!-- headwater allow=surface.local_path.instructed scope=block until=2027-09-30 reason=false_positive note=the workflow file is in the repository of the adopter, which is the CI forge integration point -->

```yaml
name: headwater check
on: [push, pull_request]
permissions:
  contents: read
  security-events: write
jobs:
  check:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
      - uses: headwater-ai/headwater/integrations/headwater-check@main
        with:
          root: .
          version: v0.1.2
          strict: "true"
```

Two different refs are in play, and they answer two different questions. `@main` names the commit of *this action's own YAML and scripts* your workflow runs. No tagged release of this repository yet carries `integrations/`. Pin it at a release tag once one does. `version: v0.1.2` names the *engine binary* the action downloads and runs against your corpus. That choice is entirely independent of the first ref. `root` is the corpus this action checks, relative to your checkout. `latest` also works for `version`. It resolves to the newest tag that carries the binary this action needs. It skips a tag of the `taxonomy/…` release stream, and it skips a tag whose release carries no asset. `strict` is `true` by default. The job fails on an error-severity finding from `headwater check`. It also fails when a committed projection, such as a shelf index or the graph export, disagrees with your corpus and your lock. It also fails when your taxonomy lock is not what your overlay and your vendored packages resolve to.

The action runs three checks, and you add no step of your own for the lock. `headwater check --strict` reads your documents against your lock. `headwater generate --check` compares each committed projection with what your corpus and your lock produce. `headwater taxonomy resolve --check` compares your lock with what your overlay and your vendored packages resolve to. Neither of the first two checks reads the overlay or the packages, so only the third check finds a lock that is out of date. When the lock fails, the job log and the job summary show the message of the engine, which names the source that changed.

## Make it cover a merge

`headwater init --git` commits a `-merge` line in `.gitattributes` for each generated file that states a count or a digest. In an adopter's tree, that file is the taxonomy lock. On your machine, git stops a merge that moves such a file. GitHub does not read that attribute. We measured this on the throwaway pull request [#1073](https://github.com/headwater-ai/headwater/pull/1073). GitHub showed it as mergeable, and its test merge held the edits of both branches in one `-merge` file.

So when you merge with the button on GitHub, this workflow is the only cover. Its `pull_request` run checks the tree of the test merge, and two checks inside the action read the result:

- `headwater taxonomy resolve --check`, inside the action, fails when the merged lock is not what the merged overlay and packages resolve to. That includes a lock conflict that somebody resolved by taking one side.
- `headwater generate --check`, inside the action, fails when a committed projection is not what the merged corpus and lock produce.

`headwater check --strict` does not compare the lock with its sources, so it does not cover the lock by itself.

The run covers the merge only when you set two things in the branch protection or the ruleset of your default branch:

1. Make the `check` job a **required status check**. Without this, GitHub lets you merge while the check is red or has not run.
2. Turn on **require branches to be up to date before merging**. Without this, the result can come from a test merge onto an older base. A change that landed on the base after that merge is not in the tree that the check read.

## How to know it worked

Two runs prove it, and the action's own CI holds both as one fixture.

A corpus with nothing wrong passes. The job succeeds. A SARIF file, named `headwater-check.sarif` by default or whatever `sarif-path` names, appears in the job's working directory, and it holds no `error`-level result.

A corpus with something wrong fails. Edit a document's `summary:` front matter, and leave its committed shelf index unregenerated. The job exits non-zero. Open the uploaded SARIF report on your repository's code-scanning page, or the local file when you set `upload-sarif: "false"`. One `error`-level result, under the rule `headwater/projection.stale`, names the path that disagrees with what your corpus and your lock produce. Its message states the remedy: run `headwater generate` and commit the result.
