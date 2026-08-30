---
id: HW-IFACE-headwater-gate
status: draft
status_since: 2026-08-25
summary: "How a published read set carries a verdict to another tree, and why new documents stay outside its reach."
last_verified: 2026-08-25
title: "headwater gate"
relations:
  governs:
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/src/main.rs
    - engine/crates/check/src/gate.rs
---

# headwater gate

## Synopsis

    headwater gate --read-set <path> [--now <date>] [--json] [--root <path>]

The verb takes the read-set path. It holds that artifact against the current tree and prints one verdict.

## Description

`headwater gate` reads the artifact that `headwater check --read-set` writes. It compares the recorded lock, clock rules, barriers, change-scoped rules and listed file hashes with the tree in front of it.

A verdict carries when the lock and every listed input still agree. No barrier or change-scoped rule may block it. Every windowed rule uses the recorded date. A moved lock, file, date, barrier, change-scoped rule, missing file or unhashed input voids the verdict.

The read set lists what the earlier run read. It never states that those were all documents in the earlier corpus. A new document can therefore escape this comparison, and every report states that limit.

The verb does not run the checks, walk the census or resolve the taxonomy. It hashes only the paths in the artifact. Text goes to standard output, and JSON names the verdict, reasons, listed input count and the coverage limit.

## Preconditions

The read-set path must be readable and contain `lock` and `clock` lines. Every other line must use a keyword that this engine reads.

The current repository must carry a readable `.headwater/taxonomy.lock`. The lock digest is compared with the digest in the artifact.

The host must provide a date, or `--now` must provide one in `YYYY-MM-DD` form. A different date voids every recorded windowed rule.

## Options

| Option | What it does |
|---|---|
| `--read-set <path>` | Read the artifact from this path. The option is required. |
| `--now <date>` | Ask about this date instead of the system date. |
| `--json` | Write the JSON verdict instead of the text report. |
| `--root <path>` | Select the repository whose lock and listed files are read. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

`--format` is not an option of this verb. `--wide` is refused because the verb prints no help layout. Global `--help`, `--version` and `--no-banner` are answered before the verb runs.

## Exit status

**0** means that the artifact was read and its verdict carries. The status does not claim that the current corpus is complete.

**1** means that the command line or an input was refused, or that one of the recorded reasons voided the verdict. The report still prints for a parsed artifact whose verdict does not carry.

**Those two halves put different bytes on different streams.** A voided verdict writes the whole report to standard output and nothing to standard error, under `--json` as under the report a person reads. A refusal writes nothing to standard output, and its account is one English sentence on standard error. So the property is that a refusal writes no document, and not that a non-zero exit writes none, which is what [HW-DR-0043](../decisions/0043-q43-whether-a-refusal-under-json-is-a-json-document.md) rules.

## Environment

No environment variable reaches this verb. The repository, date and read-set path come from the command line, and the lock and file bytes come from the tree.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.lock` | Read for the current lock digest. |
| The `--read-set` path | Read as the recorded artifact. |
| Each `input` path in the artifact | Read and hashed under `--root`. |

The verb writes no file and does not change the corpus or cache.

## See also

[`headwater check`](headwater-check.md) writes a read set with `--read-set`.

[Spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) defines the read set, its barriers and the limit of a carried verdict.

[`headwater conformance`](headwater-conformance.md) evaluates adoption of a package rather than carrying a prior check result.
