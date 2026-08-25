---
id: HW-IFACE-headwater-capture
status: draft
status_since: 2026-08-25
summary: "How the capture-cost store reports assisted fields, sections, identifiers, edges, and document reach."
last_verified: 2026-08-25
title: "headwater capture"
relations:
  governs:
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/src/main.rs
    - engine/crates/scaffold/src/reading.rs
---

# headwater capture

## Synopsis

    headwater capture [--format text|json | --json] [--root <path>]

The verb takes no operand. It reads the capture-cost store and reports its population and reach.

## Description

`headwater capture` reads one JSON object per line from `.headwater/capture-cost.jsonl`. Each valid reading records the lock, date, entry surface, kind, document, identifier and four assisted-count pairs.

The text report states the store, reading dates and unreadable lines. It then states the aggregate assisted fraction, groups by kind and surface, and document reach. A store with no readings reports an empty population rather than a measured zero.

The JSON report carries the same counts, lock list, date range, groups, classified count and documents whose readings resolve to nothing. It does not add a reading to the store. A reading under an old lock remains in the store and the report names every lock it spans.

## Preconditions

The repository must carry a readable `.headwater/taxonomy.lock`, consumer declaration and corpus. The store may be absent, which represents a corpus with no readings.

Unreadable store lines are reported with their line numbers. An unreadable store file, rather than an unreadable line, is a refusal.

## Options

| Option | What it does |
|---|---|
| `--format text` | Write the person-readable report. This is the default. |
| `--format json` | Write the machine-readable count report. |
| `--json` | The same target as `--format json`. |
| `--root <path>` | Select the repository and store to read. |
| `--no-color` | Confirm the binary's color-free output. It changes no byte. |

`--format` accepts only `text` and `json`. Stating `--json` beside `--format` is refused. `--wide` is refused because the verb prints no help layout. Global `--help` and `--version` are answered before the verb runs.

## Exit status

**0** means that the store and corpus were read and the report was printed. This includes unreadable lines that the store loader could identify.

**1** means that the format, command line, store file or repository was refused. The verb does not use the report as a gate.

## Environment

No environment variable reaches this verb. The format and repository come from the command line, and the store and corpus come from the tree.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/capture-cost.jsonl` | Read one line per valid or unreadable store entry. |
| `.headwater/taxonomy.lock` | Read through the corpus loader. |
| `.headwater/taxonomy.yml` | Read through the consumer loader. |
| The corpus | Read to classify documents and resolve identifiers. |

The verb writes no file. `headwater new` appends readings to the store before this verb reads them.

## See also

[`headwater new`](../spec/03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) appends a reading when it scaffolds a document.

[`headwater mcp`](headwater-mcp.md) records protocol scaffolding as a separate capture surface.

[Spec 3](../spec/03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) defines the assisted fraction and the store boundary.
