---
id: HW-IFACE-headwater-init
status: current
status_since: 2026-09-06
summary: "How to start a corpus by writing a consumer declaration and an overlay from tree evidence and interview prompts."
last_verified: 2026-08-25
title: "headwater init"
relations:
  governs:
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/src/main.rs
---

# headwater init

## Synopsis

    headwater init [--corpus <dir>] [--package <name>] [--root <path>]

The command writes `.headwater/taxonomy.yml` and `.headwater/overlay.yml` for a repository with no consumer declaration.

## Description

It proposes the corpus root from the directory with the most Markdown files unless `--corpus` supplies one. It uses `headwater/standard` unless `--package` supplies a package name.

It writes questions that the tree cannot answer into the overlay. It does not resolve the taxonomy, fetch a package or write a lock.

## Preconditions

The repository must not already contain `.headwater/taxonomy.yml`. Without `--corpus`, a subdirectory containing Markdown must exist. The selected package may be absent, but the command reports that the adopter must copy or vendor it before resolution.

## Options

| Option | What it does |
|---|---|
| `--corpus <dir>` | Names the corpus root instead of using tree evidence. |
| `--package <name>` | Names the package to consume. |
| `--root <path>` | Selects the repository to initialize. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

## Exit status

**0** means that the declaration and overlay were written.

**1** means that the repository is already bound, no corpus root could be proposed, or a file could not be written.

## Environment

The command reads no environment variable.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.yml` | Written as the consumer declaration. |
| `.headwater/overlay.yml` | Written with interview prompts. |
| Repository directories | Read to propose the corpus root and package location. |

## See also

[`headwater taxonomy resolve`](headwater-taxonomy.md) validates the declaration and writes the lock. [`headwater infer`](headwater-infer.md) reports the first adoption debt.
