---
id: HW-IFACE-headwater-new
status: current
status_since: 2026-09-06
summary: "How to scaffold one typed document from the resolved taxonomy without overwriting existing files."
last_verified: 2026-08-25
title: "headwater new"
relations:
  governs:
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/src/main.rs
    - engine/crates/scaffold/src/lib.rs
---

# headwater new

## Synopsis

    headwater new <kind> --title <text> [--relates <relation=identifier>] [--facet <facet=value>] [--now <date>] [--root <path>]

The command proposes and writes one document whose kind, shelf, facets, sections and identifier come from the resolved taxonomy.

## Description

The command decides the complete artifact before it writes any file. It derives engine-owned fields, accepts a title and declared facet values, and can add scaffold-created relation edges.

It never overwrites a document, and it never overwrites a claim. A kind whose scheme allocates `reconcile-first` gets one file of the identifier claim store, written before the document and holding the path of the document. That file records what the run minted, so an allocator on another branch reads a value this tree does not yet hold. The command writes the document and appends one capture-cost reading. If the document lands but the reading does not, the command reports the line to append and exits non-zero.

## Preconditions

The repository must have a readable consumer declaration, resolved taxonomy lock, corpus and configuration. The requested kind must exist in the resolved taxonomy.

The title is required. A supplied relation must be declared as scaffold-created, connect permitted kinds and resolve at its target. A supplied facet must be required by the kind, must not be one that a declaration decides, and must use a permitted value. The identifier the run mints must be claimed by no document and by no file of the claim store.

## Options

| Option | What it does |
|---|---|
| `<kind>` | Selects the document kind. |
| `--title <text>` | Supplies the document title and name facet. |
| `--relates <relation=identifier>` | Adds a repeatable scaffold-created relation. |
| `--facet <facet=value>` | Supplies a repeatable hand-entered facet value. |
| `--now <date>` | Sets the document date in `YYYY-MM-DD` form. |
| `--root <path>` | Selects the repository to load. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

## Exit status

**0** means that the document and its capture-cost reading were written.

**1** means that the command line, taxonomy, requested values or write failed. A failed run can leave a document when its reading failed to append. A run whose claim could not be made writes nothing at all, because the claim is made before the document.

## Environment

The command reads the system date when `--now` is absent. It reads no other environment variable.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.lock` and corpus configuration | Read to derive the artifact. |
| Documents and graph indexes | Read to validate identifiers and relations. |
| The selected document path | Written when it does not already exist. |
| `.headwater/ids/<scheme>/<identifier>` | Written before the document, for a scheme that allocates `reconcile-first`. It holds the path of the document, it is never written twice, and it is never modified. |
| `.headwater/capture-cost.jsonl` | Appended with one reading after the document write. |

## See also

[`headwater infer`](headwater-infer.md) reports adoption debt. [`headwater check`](headwater-check.md) checks the document after it lands.
