---
id: HW-OBL-0173
status: current
status_since: 2026-09-07
summary: "Four named steps of ci.yml carry a shell decision procedure that ends in exit 1, no fixture suite drives any of them over a scratch input, and no shell of this repository is parsed or linted at all."
last_verified: 2026-09-07
title: "Four inline gates in the workflow decide an exit status and no suite provokes one"
waiting_on: adopter
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - .github/workflows/ci.yml
    - HW-OBL-0145
---

# Four inline gates in the workflow decide an exit status and no suite provokes one

## Context

`.github/workflows/ci.yml` holds 40 `run:` steps over 1022 lines, and five of its lines carry `exit 1`. Four named steps own them.

- *The build wrote nothing into the deployed site*
- *Every rendered page points at the corpus descriptor*
- *The two halves of the site compose into one served directory*
- *The SARIF this corpus emits is SARIF*

Each one is a decision procedure written in the workflow, and each one reddens a required check.

Line 376 is the sharpest of them. It runs `grep -rL 'rel="describedby"'` over `.headwater/site-build`, filters `404.html` out, and exits 1 on a non-empty result. The comment above it explains that `|| true` is required. `grep -L` exits 1 when it finds nothing, so the success case would otherwise end the job. That reasoning is correct and nothing provokes either arm of it. A build that wrote no HTML at all produces an empty result and the same exit 0.

Thirteen fixture suites ship under `tools/`, `.githooks/` and `.claude/`. Four of them read `ci.yml`, measured by a `grep -c 'ci\.yml'` over each file: `tools/build-declaration-fixtures.sh`, `.claude/skills/fixtures.sh`, `tools/developing-fixtures.sh` and `tools/readme-fixtures.sh`. Nine read it not at all. Not one of the thirteen drives an inline gate of the workflow over a scratch input.

The two suites that do read `ci.yml` read it two different ways. `tools/build-declaration-fixtures.sh` parses every `run:` value, which is what lets it judge a cargo step written behind a `cd` or inside a block scalar. `.claude/skills/fixtures.sh` calls `grep -q 'headwater sweep\|sweep report\|sweep plan'` over the whole file at lines 674 and 728. A comment naming the verb is a false red there, and a step invoking it through a variable is a false green.

No mechanism in this repository parses or lints a shell script. `sh -n` appears nowhere, `shellcheck` runs nowhere, and the one occurrence of the string is a `# shellcheck disable=SC2086` directive at `tools/engine-readme-fixtures.sh:270` that no linter reads.

## Obligation

The gates that hold this repository are held by nothing themselves. A gate that reddens on correct input and a gate that stays green on broken input are one class of defect. The workflow carries four procedures where neither arm is provoked. `.claude/skills/fixtures.sh` reads the workflow at a grain that cannot separate a comment from a step. That is a second reader of the same file disagreeing with the first.

## Discharge

Each inline gate is discharged when a scratch-driven case runs its refusal and asserts the words of it. A second case runs its success arm over an input that must pass. The site gates need a scratch build directory rather than a real `mkdocs build`, which is what makes them affordable.

The reader half is discharged when every suite that judges `ci.yml` reads a parsed `run:` value, the way `tools/build-declaration-fixtures.sh` already does. A shell parse over every script of this repository, by `sh -n` or by a linter, is a separate and smaller remedy that no gate carries.
