---
id: HW-IFACE-headwater-probe
status: current
status_since: 2026-09-06
summary: "How plan, record, grade, compare and stale compose probe measurements without gating a build."
last_verified: 2026-10-03
title: "headwater probe"
relations:
  governs:
    - [engine/crates/cli/src/lib.rs, engine/crates/cli/src/main.rs]
    - engine/crates/probe/src/lib.rs
    - engine/crates/probe/src/plan.rs
    - engine/crates/probe/src/budget.rs
    - .headwater/probe.yml
---

# headwater probe

## Synopsis

    headwater probe plan [--tier regression|campaign|documentation|additive]
                          [--arm present|absent|no-hook|no-skills|no-claude-md|mcp|scenario-<n>]
                          [--category name | --set name] [--exclude probe]... [--repetitions n] [--seed n]
    headwater probe plan [--tier regression|campaign|documentation] --arm <arm> --delta
    headwater probe plan --scenario <n> --delta
    headwater probe plan --instrument
    headwater probe plan --folds
    headwater probe plan --oracles
    headwater probe plan --answer-keys <probe>
    headwater probe record <path>
    headwater probe grade <path>
    headwater probe compare <treated> <control>
    headwater probe stale

`plan` takes no path. `record` and `grade` take one transcript path. `compare` takes two transcript paths, the treated run first and the control run second. `stale` takes no operand. Each subcommand refuses an unknown word.

## Description

`headwater probe` is the five-part measurement harness. `plan` composes a selection from the declared probes, tier, arms, category and budget. `record` reads a transcript and confirms its structure without evaluating an expectation. `grade` evaluates the declared expectations. `compare` grades two transcripts and states the difference of their rates. `stale` compares committed transcript read sets with the current corpus.

Probe output never changes an exit status. A probe is a measurement, not a build gate. No subcommand opens a network connection or writes a file.

**No engine code performs the part between `plan` and `record`, so no build waits for a model.** That part drives a session and observes it. `plan` fixes the six members of the run identity that a run inherits. It projects the cost of the run against the declared budget of the tier, and it refuses a run above that budget. `stale` holds the read set of every committed transcript against the tree in front of it, and it reports which recorded results a change voided. The harness is a crate that `headwater-check` cannot name. An exit status that carried a rate would be a build that a model moves. These are the four facts that [spec 12](../spec/12-check-layer.md#four-things-stop-a-sweep-from-gating-and-none-of-them-is-a-rule-that-somebody-keeps) fixes for the sampler path.

The harness reads the selection again when it grades a transcript. This prevents a result from grading against probes that the current corpus no longer declares.

`grade` states, for each graded session, how many of its recorded calls went to a tool of an MCP server. A call to an MCP server is a call whose tool name starts with `mcp__`. The grader reads the tool name and never the argument. A session with no `calls` key gets the words "not counted" and never the number 0. `grade` also states how many of the sessions that recorded their calls made no call to an MCP server. It states how many sessions recorded no calls. Thus a reader gets the share of sessions that never called the server from the committed result, and not from the transcript.

`compare` grades each of its two transcripts in the same way that `grade` grades one. It prints one line for each probe that both transcripts ran. Then it prints one line with the label `pooled`, over all the graded sessions of each transcript. Each line has this form:

    <probe>: <x1>/<n1> against <x0>/<n0>, <d> points [<low>, <high>]

`<x1>/<n1>` is the satisfied sessions of the treated transcript over its graded sessions, and `<x0>/<n0>` is the same count for the control transcript. A refused session is in no denominator. `<d>` is the treated rate minus the control rate, in percentage points with its sign. `[<low>, <high>]` is the Newcombe hybrid score interval of that difference, built from the two Wilson intervals that `grade` reports. A result page computes the same interval, and [HW-DR-0112](../decisions/0112-the-additive-campaign-measures-eleven-actions-by-five-layers-in-seven-scenarios.md) registers it. The line states no verdict. Three cases get a stated line, not a silent drop:

- When one transcript graded no session of a probe, the line ends `no difference: the treated transcript graded no session`, or names the control transcript.
- When only one transcript ran a probe, the line is `<probe>: only the treated transcript ran this probe, so it has no difference`, or names the control transcript.
- When the intake refuses either transcript, `compare` prints `The treated transcript reached no grader, so nothing was compared: ` and the refusal, or names the control transcript. It prints no comparison line.

## Preconditions

All subcommands require a readable `.headwater/probe.yml`, a readable taxonomy lock and a loadable corpus. `record` and `grade` also require a readable transcript path, and `compare` requires two. `stale` requires committed probe transcripts to report, but it succeeds when none exist.

## Options

| Subcommand | Options | What it does |
|---|---|---|
| `plan` | `--tier <regression\|campaign\|documentation\|additive>` | Select the tier. The default is `regression`. A paired tier refuses a probe whose predicate names a document that its own ablation removes. Every tier refuses one whose document sits under the `instrument`, which every arm removes. The `additive` tier runs the scenarios of the additive campaign, one `scenario-<n>` arm at a time, and it does not pair arms. It does not apply the ablation rule to a scenario arm, because scenario 0 removes `docs/` and is the floor of the campaign. |
| `plan` | `--arm <present\|absent\|no-hook\|no-skills\|no-claude-md\|mcp\|scenario-<n>>` | Narrow the declared arms to one arm. An arm the tier does not declare refuses the run and names the arms it does declare. A paired tier refuses a narrowing to one arm that it declares. A `scenario-<n>` arm names scenario `<n>` of the `scenarios` mapping. Only the `additive` tier declares one, and the `additive` tier declares no other arm. |
| `plan` | `--set <name>` | Narrow the selection to the probes of one set of the `sets` mapping of `.headwater/probe.yml`, across categories. A set that the file does not declare refuses the run and names the sets it does declare. It cannot be used with `--category`. The file is refused whole when a set is empty or when two sets hold one probe. |
| `plan` | `--delta` | Plan nothing, and print the delta of the `--arm` arm against the present tree, as the tier declares it. Each line is `- <path>` for a path that the arm removes, or `+ <path>` for a path that it adds. The option reads `.headwater/probe.yml` and does not load the corpus. It needs `--arm` or `--scenario`. A script that builds the tree of an arm reads this output, so that the script does not parse the declaration again. |
| `plan` | `--scenario <n>` | With `--delta`, plan nothing, and print how the tree of scenario `<n>` differs from the present tree. The `layers` and `scenarios` mappings of `.headwater/probe.yml` declare the scenario. A line `- <layer> <path>` names a path that the scenario removes. A line `= <layer> <path>` names a path that it keeps from the present tree. A line `+ <layer> <path>` names a path that it writes from a copy that the repository keeps for the layer. A line `- tooling <path>` names a path of the `tooling` list, which every scenario removes. Scenario 0 is plain code, so it removes the path of every layer. The option reads `.headwater/probe.yml` and does not load the corpus. A scenario that the file does not declare is refused with status 1. A scenario that adds more than one layer to its base is also refused with status 1. The one exception is the scenario that declares `exception: full-stack` and holds every layer. It needs `--delta`, and it cannot be used with `--arm`, `--instrument`, `--folds`, `--oracles` or `--answer-keys`. A script that builds the tree of a scenario reads this output, so that the script does not parse the declaration. |
| `plan` | `--instrument` | Plan nothing, and print each path of the `instrument` sequence of `.headwater/probe.yml` on its own line. Every arm of every tier removes these paths. The option reads `.headwater/probe.yml` and does not load the corpus. It cannot be used with `--delta`, `--folds`, `--oracles` or `--answer-keys`. A script that removes the instrument from a tree reads this output, so that the script does not parse the declaration. A declaration with no `instrument` prints nothing. |
| `plan` | `--folds` | Plan nothing, and print each path of the `folds` sequence of `.headwater/probe.yml` on its own line. A fold is a file outside `docs/` that names a probe and does not state its answer. The seal keeps these files. The option reads `.headwater/probe.yml` and does not load the corpus. It cannot be used with `--delta`, `--instrument`, `--oracles` or `--answer-keys`. A script that seals a workspace reads this output, so that the script does not parse the declaration. A declaration with no `folds` prints nothing. |
| `plan` | `--oracles` | Plan nothing, and print each command oracle that the `oracles` mapping of `.headwater/probe.yml` declares. Each line has three fields that a tab separates. The lines are `<name> parser <parser>` and `<name> command <command>`, then one `<name> path <pattern>` line for each pattern. Last is one `<name> hold <path>` line for each file that the grading copy keeps. The option reads `.headwater/probe.yml` and does not load the corpus. It cannot be used with `--delta`, `--instrument`, `--folds` or `--answer-keys`. The recorder and the campaign read this output, so that no script parses the declaration. A declaration with no `oracles` prints nothing. |
| `plan` | `--answer-keys <probe>` | Plan nothing, and print each path that the `answer_keys` mapping of `.headwater/probe.yml` declares for the probe, on its own line. An answer key is a document that an earlier session wrote in answer to the task of the probe. The seal removes these documents. The option reads `.headwater/probe.yml` and does not load the corpus. It cannot be used with `--delta`, `--instrument`, `--folds` or `--oracles`. A script that seals a workspace reads this output. A probe with no answer key prints nothing. |
| `plan` | `--category <name>` | Narrow the selection to one declared category. |
| `plan` | `--exclude <probe>` | Remove one probe from the selection, by its identifier. Repeat the option for more than one probe. The selection digest excludes the probe. An identifier that is not in the selection refuses the run. |
| `plan` | `--repetitions <n>` | Plan fewer repetitions than the tier declares, for a pilot. A number above the declared count refuses the run, and so does `0`. |
| `plan` | `--seed <n>` | Record the caller's rotation seed. It does not select a subset. The default is `0`. |
| `record` | `<path>` | Read and report the transcript at the path. |
| `grade` | `<path>` | Grade the transcript at the path against the current regression selection. When the transcript names part of the selection, the grade uses that part. |
| `compare` | `<treated> <control>` | Grade the two transcripts against the current regression selection, as `grade` does. Print the difference of the treated rate minus the control rate for each probe and for the `pooled` line, each with its Newcombe interval. |
| `stale` | none | Report which committed transcript read sets changed. |
| every | `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. Four subcommands, `plan`, `record`, `grade` and `stale`, color their own report under a terminal. `compare` prints its report in plain text. A refusal on standard error colors under a terminal for all five subcommands. |

Global `--root` selects the repository. `--help`, `--version`, `--wide` and `--no-banner` are handled by the binary before or around the subcommand.

## Exit status

**0** when a subcommand completes, including when a probe refuses a run or a transcript contains findings. An arm, tier or category that a declaration or the corpus does not carry refuses the run at this status. Probe results never gate.

`grade` also exits 0 when the intake refuses a transcript whole, for example a transcript with no `Run identity` block. That refusal is a probe result, and [spec 12](../spec/12-check-layer.md#four-things-stop-a-sweep-from-gating-and-none-of-them-is-a-rule-that-somebody-keeps) says why a probe result never gates. `grade` prints the refusal on standard output, not on standard error. A script tells the three results of `grade` apart by the start of the first line of standard output:

- `Graded by grader ` when the grader read the transcript.
- `This transcript reached no grader: ` when the intake refused the transcript, for any of its reasons.
- `Nothing was graded. ` when the plan refuses the corpus, so that `grade` did not read the transcript.

The first line carries no color escape, also under a terminal.

`compare` exits 0 on every comparison it prints, whatever the rates are. It also exits 0 when the intake refuses either transcript, and when the plan refuses the corpus. In that case it prints `Nothing was compared. ` and the refusal. It exits 1 when an operand is missing or a transcript cannot be read, as `grade` does.

**1** when the command line is invalid, or a required file cannot be read. A tier, arm or category name outside this engine's closed set makes the command line invalid. It is also 1 when the budget declaration is malformed or the corpus cannot load. Under `--delta`, an arm that the tier does not run is also 1, because a script builds a tree from the output. Under `--instrument`, `--folds` or `--answer-keys`, a malformed or unsafe declaration is 1, and nothing is printed on standard output. The message names the refusal.

**1**, and never 101, when standard output or standard error cannot be written, and one sentence on standard error names a failed standard output.

## Environment

No environment variable reaches a probe subcommand. The tier, repository and seed come from the command line, and the budget comes from `.headwater/probe.yml`. No model, key or endpoint is read.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/probe.yml` | read by `plan`, `grade`, `compare` and `stale` to load the tier budgets |
| `.headwater/taxonomy.lock` | read by every subcommand through the corpus loader |
| `.headwater/taxonomy.yml` | read by every subcommand through the corpus loader |
| the corpus | read to select probes and to rebuild transcript read sets |
| `.headwater/imports/` | read by every subcommand where `.headwater/taxonomy.yml` declares an import, for the anchors that an imported snapshot supplies. An `imports` entry that does not read stops the verb with exit 1, and so does an `at` path outside the repository root. |
| the path each `harvests.<name>.at` names | read by every subcommand where `.headwater/taxonomy.yml` declares a pinned corpus export, for the anchors that export supplies. A `harvests` entry that does not read stops the verb with exit 1, and so does an `at` path outside the repository root. A pin with no digest binds no anchor. An absent file binds no anchor, and neither does a file that fails the pinned digest or is not an export. |
| the transcript path | read by `record` and `grade`, and both transcript paths are read by `compare` |
| committed probe transcripts | read by `stale` |

No subcommand writes a file. Output goes to standard output.

## See also

[Spec 5](../spec/05-ai-integration.md#three-tiers-and-the-cadence-follows-the-purpose) defines the three tiers, probe categories and transcript contract.

[Spec 15](../spec/15-the-recorder-contract.md) defines the recorder and the six members of a probe run identity.

[`headwater sweep`](headwater-sweep.md) reports coherence findings and never evaluates a probe expectation.

[The command surface](README.md) lists the verbs and contracts that `headwater generate` derives.
