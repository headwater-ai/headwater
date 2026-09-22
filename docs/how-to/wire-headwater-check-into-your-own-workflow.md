---
id: HW-HOW-wire-headwater-check-into-your-own-workflow
status: current
status_since: 2026-09-23
summary: "A composite action runs headwater check --strict against your own corpus, with the released binary alone: no clone, no Rust toolchain, no cargo."
last_verified: 2026-09-23
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

## Before you start

You need a GitHub Actions runner on Linux x86_64, for example `ubuntu-24.04` or `ubuntu-latest`. `headwater-ai/headwater` publishes a release binary for that platform alone. A runner of a different kind refuses cleanly, and it names the platform it needed. It does not download a binary that cannot run there.

You need no Rust toolchain and no `cargo` on the runner. The action downloads a release archive, checks its `sha256` sum, and runs the binary inside it. It never builds the engine.

Your workflow needs `permissions: security-events: write` if you want the SARIF report on GitHub's code-scanning page. Without that permission, the action still runs the checks and still fails the job on an error, and it skips the upload step alone.

## Steps

Add one step to a workflow file, for example `.github/workflows/headwater-check.yml`:

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

Two different refs are in play, and they answer two different questions. `@main` names the commit of *this action's own YAML and scripts* your workflow runs. No tagged release of this repository yet carries `integrations/`. Pin it at a release tag once one does. `version: v0.1.2` names the *engine binary* the action downloads and runs against your corpus. That choice is entirely independent of the first ref. `root` is the corpus this action checks, relative to your checkout. `latest` also works for `version`. It resolves to the newest tag that carries the binary this action needs. It skips a tag of the `taxonomy/…` release stream, and it skips a tag whose release carries no asset. `strict` is `true` by default. The job fails on an error-severity finding from `headwater check`. It also fails when a committed projection, such as a shelf index or the graph export, disagrees with your corpus and your lock.

## How to know it worked

Two runs prove it, and the action's own CI holds both as one fixture.

A corpus with nothing wrong passes. The job succeeds. A SARIF file, named `headwater-check.sarif` by default or whatever `sarif-path` names, appears in the job's working directory, and it holds no `error`-level result.

A corpus with something wrong fails. Edit a document's `summary:` front matter, and leave its committed shelf index unregenerated. The job exits non-zero. Open the uploaded SARIF report on your repository's code-scanning page, or the local file when you set `upload-sarif: "false"`. One `error`-level result, under the rule `headwater/projection.stale`, names the path that disagrees with what your corpus and your lock produce. Its message states the remedy: run `headwater generate` and commit the result.
