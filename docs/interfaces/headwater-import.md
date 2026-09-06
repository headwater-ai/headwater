---
id: HW-IFACE-headwater-import
status: current
status_since: 2026-09-06
summary: "How to validate a pinned fetched snapshot and import its declared relation edges into documents."
last_verified: 2026-08-25
title: "headwater import"
relations:
  governs:
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/src/main.rs
    - engine/crates/import/src/lib.rs
---

# headwater import

## Synopsis

    headwater import [<name>] [--expect <digest>] [--write] [--root <path>]

The command reads a declared fetched snapshot, checks its pin, and reports the relation halves it would add.

## Description

Without `--write`, the command reports pending edge halves and changes nothing. With `--write`, it writes the validated halves into documents at their near ends.

It requires a digest from the declaration or from `--expect`. It refuses the whole import when a snapshot, edge or write cannot be validated.

## Preconditions

The consumer declaration must contain one or more import declarations. A bare command is valid only when exactly one import exists. The selected snapshot directory must exist and match the expected digest.

## Options

| Option | What it does |
|---|---|
| `<name>` | Selects a declared import when more than one exists. |
| `--expect <digest>` | Supplies the digest to check against the snapshot. |
| `--write` | Writes validated edge halves into documents. |
| `--root <path>` | Selects the repository to load. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

## Exit status

**0** means that the snapshot was read and the plan was reported or written.

**1** means that the import was undeclared, unpinned, invalid or not writable. A refusal writes no edge.

## Environment

The command reads no environment variable.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.yml` | Read for import declarations and digest. |
| The declared snapshot directory | Read and checked against its digest. |
| Documents | Written only with `--write`, after the whole plan validates. |

## See also

[`headwater taxonomy vendor`](headwater-taxonomy.md) installs a fetched taxonomy package. [`headwater check`](headwater-check.md) reads the imported graph.
