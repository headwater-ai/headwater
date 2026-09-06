---
id: HW-IFACE-headwater-explain
status: current
status_since: 2026-09-06
summary: "How a path or identifier yields the taxonomy derivation, requirements, relations, and graph edges."
last_verified: 2026-08-25
title: "headwater explain"
relations:
  governs:
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/src/main.rs
    - engine/crates/query/src/lib.rs
    - engine/crates/query/src/explain.rs
    - engine/crates/query/src/json.rs
    - engine/crates/census/src/walk.rs
---

# headwater explain

## Synopsis

    headwater explain <path|identifier> [--json] [--root <path>]

The target is a path under the corpus root or an identifier declared by a document.

## Description

`headwater explain` prints the census derivation for one document. It states the path, identifier, kind, derivation steps, purpose, summary, warrant, required facets, required sections, permitted relations and related graph edges.

An untyped document can still be explained. Its derivation states the step that stopped classification, and its kind-dependent fields are empty because no kind requires them.

The text report follows the order above. JSON carries the same fields in a document with its own shape version. A related edge states its direction, target, cue, governing end and the far document pointer where one exists.

**The text report renders the palette [HW-DR-0045](../decisions/0045-coloring-the-cli-and-where-the-banner-goes.md) rules on, when standard output is a terminal.** The path is cyan, and the repeated labels — `kind`, `purpose`, `summary`, `warrant`, `requires the facets`, `requires the sections`, `may declare` — are dim, the same way on every run. JSON never colors: `--json` selects a machine format regardless of the stream. `--no-color` forces the plain text this verb already wrote before this decision.

## Preconditions

The repository must carry a readable `.headwater/taxonomy.lock`, consumer declaration and corpus. The target must resolve to a typed or untyped census row.

An identifier resolves through the graph index. A target that matches neither a corpus path nor an identifier is refused. Exit status states which of four things it named instead.

## Options

| Option | What it does |
|---|---|
| `<path|identifier>` | Select the document to explain. |
| `--json` | Write the explanation as machine-readable JSON. |
| `--root <path>` | Select the repository to read. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

`--format` is not an option of this verb. `--wide` is refused because the verb prints no help layout. Global `--help`, `--version` and `--no-banner` are answered before the verb runs.

## Exit status

**0** means that the target resolved and its explanation was printed.

**1** means that the target was missing, the command line was invalid, or the repository could not load. A missing target writes its refusal to standard error and no explanation to standard output.

**A missing target still classifies.** The refusal names one of four states.

| The state | What the refusal says |
|---|---|
| Under the corpus root, with no document there | "is a path of this corpus, with no document written there yet" |
| Under the corpus root, and an exclusion claims it | "is excluded by `<pattern>`" |
| Outside every corpus root this repository declares | "is outside every corpus root this repository declares" |
| Not a path this repository can classify | "is not a path this repository can classify" |

One matcher decides all four: `headwater_census::walk::Corpus::classify`, which the walk also uses for an existing file. `.claude/hooks/write.sh` reads the first row and refuses a raw write there.

## Environment

No environment variable reaches this verb. The target and repository come from the command line, and the census and graph come from the tree.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.lock` | Read for the resolved taxonomy. |
| `.headwater/taxonomy.yml` | Read through the consumer loader. |
| The corpus | Read for census rows, front matter and graph edges. |

The verb writes no file.

## See also

[`headwater route`](headwater-route.md) selects pointers from a task description.

[Spec 2](../spec/02-taxonomy-model.md#kind-resolution) defines the derivation and the requirements this verb prints.

[`headwater mcp`](headwater-mcp.md) serves the same explanation through a JSON-RPC tool.
