---
id: HW-IFACE-headwater-explain
status: draft
status_since: 2026-08-25
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
---

# headwater explain

## Synopsis

    headwater explain <path|identifier> [--json] [--root <path>]

The target is a path under the corpus root or an identifier declared by a document.

## Description

`headwater explain` prints the census derivation for one document. It states the path, identifier, kind, derivation steps, purpose, summary, warrant, required facets, required sections, permitted relations and related graph edges.

An untyped document can still be explained. Its derivation states the step that stopped classification, and its kind-dependent fields are empty because no kind requires them.

The text report follows the order above. JSON carries the same fields in a document with its own shape version. A related edge states its direction, target, cue, governing end and the far document pointer where one exists.

## Preconditions

The repository must carry a readable `.headwater/taxonomy.lock`, consumer declaration and corpus. The target must resolve to a typed or untyped census row.

An identifier resolves through the graph index. A target that matches neither a corpus path nor an identifier is refused.

## Options

| Option | What it does |
|---|---|
| `<path|identifier>` | Select the document to explain. |
| `--json` | Write the explanation as machine-readable JSON. |
| `--root <path>` | Select the repository to read. |
| `--no-color` | Confirm the binary's color-free output. It changes no byte. |

`--format` is not an option of this verb. `--wide` is refused because the verb prints no help layout. Global `--help` and `--version` are answered before the verb runs.

## Exit status

**0** means that the target resolved and its explanation was printed.

**1** means that the target was missing, the command line was invalid, or the repository could not load. A missing target writes its refusal to standard error and no explanation to standard output.

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
