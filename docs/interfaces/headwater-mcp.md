---
id: HW-IFACE-headwater-mcp
status: draft
status_since: 2026-08-25
summary: "How the stdio MCP server exposes read tools, optional working-tree writes, and a one-write session seal."
last_verified: 2026-08-25
title: "headwater mcp"
relations:
  governs:
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/src/main.rs
    - engine/crates/query/src/mcp.rs
---

# headwater mcp

## Synopsis

    headwater mcp [--now <date>] [--write] [--root <path>]

The server reads JSON-RPC messages from standard input and writes one response per request to standard output.

## Description

`headwater mcp` starts the agent-facing surface of the same library as the command line. It walks the corpus once, fixes the date once and then answers MCP requests from those values.

The default server registers six read tools: `route`, `explain`, `related`, `resolve_identifier`, `governing_docs_for_path` and `check`. The `check` tool takes one format from `text`, `json`, `sarif` or `markdown`. The other read tools take the target or task named by their command-line counterparts.

`--write` also registers `new` and `fix`. `new` takes a kind, title and optional array of relation strings. `fix` takes one output format. No tool commits, pushes or merges, and the default server registers no tool that writes.

A write tool returns the account and artifact as separate content blocks. A call that moves a byte spends the server. Later tool calls receive a JSON-RPC `-32000` error because their answers would describe a stale corpus. A refused write or a write that lands no byte does not spend the server.

The transport uses one JSON object per line. Notifications produce no response. Unknown methods and invalid tool arguments produce JSON-RPC errors. Tool descriptions state read-only and destructive hints, but registration is the boundary that controls which tools exist.

## Preconditions

The repository must carry a readable `.headwater/taxonomy.lock`, consumer declaration and corpus. The server loads these before it accepts a message.

The host must provide a date, or `--now` must provide one in `YYYY-MM-DD` form. The server uses that date for its whole session.

The process must have readable standard input and writable standard output. The server does not need a network endpoint or a model.

## Options

| Option | What it does |
|---|---|
| `--now <date>` | Fix the date for every check and write in this server session. |
| `--write` | Register the working-tree `new` and `fix` tools. It does not register commit, push or merge tools. |
| `--root <path>` | Select the repository to load and serve. |
| `--no-color` | Confirm the binary's color-free output. It changes no byte. |

`--wide` is refused because the server prints protocol responses rather than help. Global `--help` and `--version` are answered before the server runs.

## Exit status

**0** means that the server started and returned when standard input closed. Tool findings and JSON-RPC errors do not change this process status.

**1** means that the command line, date, taxonomy, consumer declaration or corpus could not load. A protocol error is carried in the response with its JSON-RPC code.

## Environment

No environment variable reaches this verb. The date, write consent and repository come from the command line, and all other inputs come from the loaded tree and protocol messages.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.lock` | Read at startup. |
| `.headwater/taxonomy.yml` | Read at startup through the consumer loader. |
| The corpus | Read once at startup for every registered tool. |
| Documents and `.headwater/capture-cost.jsonl` | Written by `new` when `--write` is enabled. |
| Documents | Written by `fix` only when a derived patch lands. |

The default server writes no file. A write-enabled server ends its usable session after the first call that moves a byte.

## See also

[`headwater route`](headwater-route.md) and [`headwater explain`](headwater-explain.md) document the terminal reads served by this protocol.

[Spec 5](../spec/05-ai-integration.md#what-the-server-may-do-and-the-axis-that-decides-it) defines the query, working-tree write and landed-write classes.

[Spec 6](../spec/06-engine-architecture.md#mcp-server) defines the MCP transport boundary.
