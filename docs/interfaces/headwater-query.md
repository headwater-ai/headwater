---
id: HW-IFACE-headwater-query
status: draft
status_since: 2026-08-25
summary: "Why `headwater query` always refuses an expression, and which graph reads replace it."
last_verified: 2026-08-25
title: "headwater query"
relations:
  governs:
    - engine/crates/cli/src/main.rs
    - engine/crates/query/src/lib.rs
---

# headwater query

## Synopsis

    headwater query <expression>

The verb takes one or more words as the expression. It refuses every expression because this repository defines no expression grammar.

## Description

`headwater query` is listed in [spec 6](../spec/06-engine-architecture.md#cli), but no document defines an expression or its selected data. The engine therefore implements no query language.

The verb reports this gap and names the reads that exist. Use `headwater route` to match a task to documents. Use `headwater explain` to read the graph behind one document.

## Preconditions

None. The command line reaches the refusal without loading a corpus or a taxonomy lock.

## Options

The verb has no verb-specific options. Global `--root`, `--wide`, `--no-color`, `--no-banner`, `--version` and `--help` are parsed by the binary, but `query` does not read them for this refusal.

## Exit status

**1** for every expression. The refusal states that the grammar is not defined and names `headwater route` and `headwater explain` as the available reads.

**0** for `--help`, `--version` and the global help paths that the binary answers before it enters this verb.

## Environment

No environment variable reaches this verb. The refusal depends only on the command line.

## Files

None. The verb reads and writes no file.

## See also

[HW-OBL-0029](../obligations/0029-what-headwater-query-takes.md) records the missing expression grammar and waits on a build.

[`headwater route`](../spec/06-engine-architecture.md#the-verbs) matches a task description to the graph.

[`headwater explain`](../spec/06-engine-architecture.md#the-verbs) reports the kind, purpose and edges of one document.
