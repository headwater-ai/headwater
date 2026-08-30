---
id: HW-IFACE-headwater-export
status: draft
status_since: 2026-08-25
summary: "How to emit a taxonomy export profile to a file or standard output with a declared loss census."
last_verified: 2026-08-25
title: "headwater export"
relations:
  governs:
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/src/main.rs
    - engine/crates/generate/src/export.rs
---

# headwater export

## Synopsis

    headwater export [--profile <name>] [--format <target>] [--at <date>] [--check] [--json] [--root <path>]

The command emits declared export profiles, or one selected emitter target to standard output.

## Description

Without `--format`, it writes every declared profile to its taxonomy-declared path. `--check` compares those files without writing. With `--format`, it writes one artifact to standard output and reports its projection census to standard error.

The command refuses an ambiguous profile selection or an uncovered emitter loss.

## Preconditions

The repository must load its taxonomy, corpus and projection declarations. A stream export must select one profile when several are declared. A declared export must have a writable or checkable target path.

## Options

| Option | What it does |
|---|---|
| `--profile <name>` | Selects one declared export profile. |
| `--format <target>` | Emits one artifact to standard output using the target. |
| `--at <date>` | Adds a generation date to a stream artifact. |
| `--check` | Checks declared output files without writing. |
| `--json` | Selects JSON output where the command supports a format choice. |
| `--root <path>` | Selects the repository to load. |
| `--no-color` | Confirms color-free output. |

## Exit status

**0** means that declared outputs match, or that the stream artifact was emitted without an uncovered loss.

**1** means that loading, profile selection, emission or writing failed, or that `--check` found drift.

**A refusal writes nothing to standard output, and a run that reported and then failed still wrote its report.** Drift found by `export --check` is the second case, where the regeneration report prints and the run then exits 1. A refusal of the command line, of a profile name or of an emitter target prints nothing there at all. The account of a refusal is one English sentence on standard error, under `--json` and `--format json` alike, which is what [HW-DR-0043](../decisions/0043-q43-whether-a-refusal-under-json-is-a-json-document.md) rules.

**A refusal of this verb names the spelling of the target that the caller typed.** `--json` and `--format json` reach one value, so the two write one artifact byte for byte. A message a person reads is not an artifact. A message that named the other flag would send a reader to a flag nobody typed.

## Environment

The command reads no environment variable.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.lock`, corpus and projections | Read to build the export. |
| Declared export paths | Written without `--format` and `--check`. |
| Standard output | Receives a stream artifact with `--format`. |
| Standard error | Receives the stream projection census. |

## See also

[`headwater generate`](headwater-generate.md) writes native projections. [`headwater taxonomy publish`](headwater-taxonomy.md) creates a package artifact.
