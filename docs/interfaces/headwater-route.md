---
id: HW-IFACE-headwater-route
status: current
status_since: 2026-09-06
summary: "How a task description becomes ranked document pointers, when routing stays silent, and what the budget limits."
last_verified: 2026-08-25
title: "headwater route"
relations:
  governs:
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/src/main.rs
    - engine/crates/query/src/lib.rs
    - engine/crates/query/src/route.rs
    - engine/crates/query/src/json.rs
---

# headwater route

## Synopsis

    headwater route <task description> [--budget <n>] [--json] [--root <path>]

Every word after `route` forms the task description. The verb takes no separate path operand.

## Description

`headwater route` matches a task description to declared purposes, then ranks document pointers under those purposes. It reads summaries, facet values, paths and governance anchors. It does not search document bodies.

A task that names a governed source path receives the documents that govern that path before lexical ranking. Other tasks first match terms against declared purposes, then require a separating term to reach a document. This confidence gate prevents a common term from producing a guessed pointer.

The default budget is five ranked pointers. Anchored pointers are never removed by the budget. The report states how many ranked pointers the budget withheld.

Text prints the task, matched purposes, pointers and any silence reason. JSON carries those values, a stable silence token where appropriate and the text report from the same run. Silence is a result, not a failure.

## Preconditions

The repository must carry a readable `.headwater/taxonomy.lock`, consumer declaration and corpus. The corpus is loaded through the resolved taxonomy.

The task must contain at least one word of two characters or more after parsing. An empty task is refused before the corpus is loaded.

## Options

| Option | What it does |
|---|---|
| `--budget <n>` | Set the maximum number of ranked pointers. The default is 5. |
| `--json` | Write the route as a machine-readable JSON document. |
| `--root <path>` | Select the repository to read. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

`--format` is not an option of this verb. `--wide` is refused because the verb prints no help layout. Global `--help`, `--version` and `--no-banner` are answered before the verb runs.

## Exit status

**0** means that the task was evaluated, whether pointers were offered or the route was silent.

**1** means that the task was missing, an option was invalid, or the repository could not load. A silent route writes its normal report and still exits 0.

**All three reasons for exit 1 are refusals, so standard output is empty on every one of them.** Each is decided before anything is written, and the account is one English sentence on standard error. That holds under `--json` as under the report a person reads, which is what [HW-DR-0043](../decisions/0043-q43-whether-a-refusal-under-json-is-a-json-document.md) rules.

## Environment

No environment variable reaches this verb. The task, budget and repository come from the command line, and the graph and corpus come from the tree.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.lock` | Read for the taxonomy and graph declarations. |
| `.headwater/taxonomy.yml` | Read through the consumer loader. |
| The corpus | Read for typed documents, summaries, facets, paths and edges. |

The verb writes no file.

## See also

[`headwater explain`](headwater-explain.md) reads the derivation and requirements of one pointer.

[Spec 5](../spec/05-ai-integration.md#intent-time-routing) defines purpose matching, confidence gating, anchors and the pointer budget.

[`headwater mcp`](headwater-mcp.md) serves the same route read through a JSON-RPC tool.
