---
id: HW-IFACE-headwater-probe
status: current
status_since: 2026-09-06
summary: "How plan, record, grade and stale compose probe measurements without gating a build."
last_verified: 2026-08-25
title: "headwater probe"
relations:
  governs:
    - engine/crates/cli/src/main.rs
    - engine/crates/probe/src/lib.rs
---

# headwater probe

## Synopsis

    headwater probe plan [--tier regression|campaign] [--arm present|absent]
                          [--category name] [--seed n]
    headwater probe record <path>
    headwater probe grade <path>
    headwater probe stale

`plan` takes no path. `record` and `grade` take one transcript path. `stale` takes no operand. Each subcommand refuses an unknown word.

## Description

`headwater probe` is the four-part measurement harness. `plan` composes a selection from the declared probes, tier, arms, category and budget. `record` reads a transcript and confirms its structure without evaluating an expectation. `grade` evaluates the declared expectations. `stale` compares committed transcript read sets with the current corpus.

Probe output never changes an exit status. A probe is a measurement, not a build gate. No subcommand opens a network connection or writes a file.

The harness reads the selection again when it grades a transcript. This prevents a result from grading against probes that the current corpus no longer declares.

## Preconditions

All subcommands require a readable `.headwater/probe.yml`, a readable taxonomy lock and a loadable corpus. `record` and `grade` also require a readable transcript path. `stale` requires committed probe transcripts to report, but it succeeds when none exist.

## Options

| Subcommand | Options | What it does |
|---|---|---|
| `plan` | `--tier <regression\|campaign>` | Select the tier. The default is `regression`. |
| `plan` | `--arm <present\|absent>` | Narrow the declared arms. An arm the tier does not declare refuses the run and names the arms it does declare. |
| `plan` | `--category <name>` | Narrow the selection to one declared category. |
| `plan` | `--seed <n>` | Record the caller's rotation seed. It does not select a subset. The default is `0`. |
| `record` | `<path>` | Read and report the transcript at the path. |
| `grade` | `<path>` | Grade the transcript at the path against the current regression selection. |
| `stale` | none | Report which committed transcript read sets changed. |
| every | `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. `plan` is the only subcommand whose own report carries color. `record`, `grade` and `stale` write plain text under either setting, and a refusal on standard error colors under a terminal for all four. |

Global `--root` selects the repository. `--help`, `--version`, `--wide` and `--no-banner` are handled by the binary before or around the subcommand.

## Exit status

**0** when a subcommand completes, including when a probe refuses a run or a transcript contains findings. An arm, tier or category that a declaration or the corpus does not carry refuses the run at this status. Probe results never gate.

**1** when the command line is invalid, or a required file cannot be read. A tier, arm or category name outside this engine's closed set makes the command line invalid. It is also 1 when the budget declaration is malformed or the corpus cannot load. The message names the refusal.

## Environment

No environment variable reaches a probe subcommand. The tier, repository and seed come from the command line, and the budget comes from `.headwater/probe.yml`. No model, key or endpoint is read.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/probe.yml` | read by `plan`, `grade` and `stale` to load the tier budgets |
| `.headwater/taxonomy.lock` | read by every subcommand through the corpus loader |
| `.headwater/taxonomy.yml` | read by every subcommand through the corpus loader |
| the corpus | read to select probes and to rebuild transcript read sets |
| the transcript path | read by `record` and `grade` |
| committed probe transcripts | read by `stale` |

No subcommand writes a file. Output goes to standard output.

## See also

[Spec 5](../spec/05-ai-integration.md#two-tiers-and-the-cadence-follows-the-purpose) defines the two tiers, probe categories and transcript contract.

[Spec 15](../spec/15-the-recorder-contract.md) defines the recorder and the six members of a probe run identity.

[`headwater sweep`](headwater-sweep.md) reports coherence findings and never evaluates a probe expectation.

[The command surface](README.md) lists the verbs and contracts that `headwater generate` derives.
