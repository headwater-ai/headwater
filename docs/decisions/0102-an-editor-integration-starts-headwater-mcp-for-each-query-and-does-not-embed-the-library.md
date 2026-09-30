---
id: HW-DR-0102
status: current
status_since: 2026-09-30
summary: "An editor plugin is a client of headwater mcp: it starts the server for each query, sends one read request and stops it. The library stays embeddable, but no editor host embeds it."
last_verified: 2026-09-30
title: "An editor integration starts headwater mcp for each query and does not embed the library"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-engine-architecture
    - HW-DR-0001
  governs:
    - to: integrations/vscode/client.js
      verified_revision: sha256:3bcd9b14898a4b07170e088bbad9e0e85d3bbbc69712e6d2527b9d382fe93be3
    - to: integrations/jetbrains/src/main/java/ai/headwater/jetbrains/Client.java
      verified_revision: sha256:dacff788b092c097bbf9c07c1e601fd3bc91ec3afb8e2abee2806ad9c7872f6a
---

# An editor integration starts headwater mcp for each query and does not embed the library

## Context

[Spec 6](../spec/06-engine-architecture.md#library) said that editor integrations consume the library directly, and that they do not start a subprocess. Both editor plugins that this repository ships start a subprocess. `integrations/vscode/client.js` spawns `headwater mcp --root <root>` for each query. `integrations/jetbrains/src/main/java/ai/headwater/jetbrains/Client.java` does the same with a `ProcessBuilder`. No decision recorded that choice, so the specification and the code disagreed ([#1318](https://github.com/headwater-ai/headwater/issues/1318)).

The owner had two ways to remove the disagreement. One was to move the plugins to a library binding. The other was to amend the design and record the subprocess approach. On 2026-09-29 the owner answered "Amend the design" on the issue.

## Decision

**An editor integration is a client of `headwater mcp`.** For each query, it starts `headwater mcp --root <root>`, sends one request and stops the process. It never passes `--write`, so the server that it starts registers no tool that writes. It reads each answer from `structuredContent`, which [the `headwater mcp` interface](../interfaces/headwater-mcp.md) states as the machine contract.

The reasons are in the header comments of the two clients and in `integrations/vscode/README.md`:

- `headwater mcp` reads standard input and output only. So there is no running server for a plugin to find.
- The server walks the corpus once when it starts. A server that stays alive across edits answers from an old walk. A new process for each query answers from the tree as it is.
- One session costs about 0.16 s on this repository. The plugins stop a query after 5 s.
- One engine binary serves two hosts, Node and the JVM. No binding for each host is necessary, and nobody must build or ship one. On 2026-09-30, `engine/crates/` held no library binding for either host and no `wasm` crate.

**The owner rejected the alternative.** A library binding for each editor host is not the design, and the plugins do not move to one.

**This decision does not change three things.** The library stays embeddable, and spec 6 still states that as an implementation constraint. The MCP server consumes the library in the same process. [HW-DR-0001](0001-implementation-language.md) stands: the language is Rust so that a host can embed the library. It was not chosen because every host must embed it.

## Consequences

Spec 6 §Library states the position. The CLI and the MCP server use the library in their own process. An editor integration starts `headwater mcp` for each query, and the section links here. The CI adapter under `integrations/headwater-check/` runs the released binary, and spec 6 states that too. The owner ruled on editor integrations alone, so spec 6 states no rule for CI adapters beyond what the action does.

The two `governs` edges name the two client files. An agent that edits one of them sees this record, and the edge goes suspect when either file changes.

HW-DR-0001 still says that spec 6 requires editor integrations to consume the library in the same process. That sentence records the reasoning of 2026-08-11, and this record does not edit an accepted decision. This record traces to HW-DR-0001, and the paragraph above states what of it still holds.

**Two conditions open this decision again.** The first is a measured cost for each query that a reader in an editor notices, near the 5 s limit of the plugins. The second is an editor host where a plugin cannot start a subprocess.
