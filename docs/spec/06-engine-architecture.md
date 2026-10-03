---
id: HW-SPEC-engine-architecture
status: current
status_since: 2026-08-01
last_verified: 2026-09-30
summary: One parse, one typed graph, and many consumers, with the pipeline, the library boundary, and the performance targets.
doc_type: design_spec
sequence: 6
title: "Engine architecture"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: [claude-fable-5, claude-opus-5]
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cites_evidence:
    - HW-EVAL-adjacent-work
    - HW-EVAL-first-contact
    - HW-EVAL-graph-export-and-federation
    - HW-EVAL-language-spike-results
    - HW-EVAL-the-measurement-layer
    - HW-EVAL-the-serving-boundary
    - HW-EVAL-warrant-and-adjudication
    - HW-EVAL-what-a-check-can-know
---

# 6 — Engine architecture

One parse, one graph, many consumers.

## Why one engine

The obvious decomposition — a separate linter per concern — is the wrong one. Each tool re-walks the tree, re-parses front matter, re-implements path matching, and re-derives what kind each document is. That is slow, and worse, it is *divergent*. When two tools disagree about what a document is, the result is contradictory findings and an unfixable bug report.

Headwater parses once, builds one typed graph, and runs every check against it. Checks become small predicates over a shared model instead of programs.

## Pipeline

```
┌──────────────┐   ┌───────────┐   ┌────────────┐
│ taxonomy pkg │──▶│  resolve  │──▶│   lock     │
│  + overlays  │   │  + verify │   │ (hashed)   │
└──────────────┘   └───────────┘   └─────┬──────┘
                                         ▼
┌──────────────┐   ┌───────────┐   ┌────────────┐   ┌──────────┐
│ corpus files │──▶│   parse   │──▶│   graph    │──▶│  cache   │
└──────────────┘   │ + classify│   │ build/link │   └──────────┘
                   └───────────┘   └─────┬──────┘
                                         │
        ┌────────────┬───────────────────┼──────────────┬────────────┐
        ▼            ▼                   ▼              ▼            ▼
     checks       queries          projections       export      explain
        │            │                   │              │            │
     findings    pointers          derived files    JSON / MCP   derivation
```

**Resolve** merges the base taxonomy and overlays, validates against the meta-schema, and writes a content-hashed lock. Everything downstream reads the lock, never the sources. Thus a check result depends on a hash that a reviewer can see in a diff.

**Parse** reads each file once: front matter, headings, links, code fences. It does not interpret. Classification assigns a kind by the declared resolution rules and records the derivation for `explain`. It also writes a **census**, which states an outcome for each file under the corpus root before any check runs. So a document that failed to classify shows as unchecked, and it is not silently absent.

**Graph build** reads the documents through the census and opens no document file itself. It resolves relations into edges, indexes identifiers, binds external anchors (code paths, work items, URLs), and reports what it could not resolve.

**Cache** is content-addressed per file plus taxonomy hash. That a run is then proportional to the change and not to the corpus is a promise that [HW-OBL-0072](../obligations/0072-a-cache-of-check-results-does-not-make-a-run-proportional.md) holds open. The change-scoped mode that CI and hooks use narrows nothing. It supplies the version of each named document from before the change. So a rule that reads one reaches a verdict rather than a skip. It therefore evaluates more instances than a full-corpus run and never fewer.

### Subsystems

Each stage is built by one subsystem, and each subsystem is a group of crates under `engine/crates/`. A subsystem spec on `docs/subsystems/` describes the inside of one subsystem and governs the source of its crates ([HW-DR-0098](../decisions/0098-an-engine-subsystem-is-described-by-a-technical-design-spec-on-a-shelf-of-its-own-and-its-behavior-stays-where-it-is-already-written.md)). Every crate is in exactly one row, and `engine/crates/cli/tests/subsystem_map.rs` holds this table against the directory.

| stage | subsystem | crates | why the crates are here |
|---|---|---|---|
| resolve | [Taxonomy resolution](../subsystems/taxonomy-resolution.md) | `yaml`, `ref`, `meta`, `resolve`, `lock`, `hash` | Eleven other crates use `hash`. The lock is its first consumer, and one implementation stops two digests from disagreeing. |
| resolve | [Taxonomy distribution and audit](../subsystems/taxonomy-distribution-and-audit.md) | `fetch`, `compat`, `audit` | These crates move a taxonomy between repositories and measure it against a corpus. None of them builds the lock. |
| parse | [Parse and census](../subsystems/parse-and-census.md) | `doc`, `census`, `vcs` | `census` and `graph` use `vcs` for the change manifest. The only other crate that uses it is `cli`. |
| graph build | [Graph build](../subsystems/graph-build.md) | `graph` | |
| cache, checks | [Checks and cache](../subsystems/checks-and-cache.md) | `check`, `adapter` | `adapter` renders one run of the check layer for a CI platform. |
| queries, explain | [Queries and explain](../subsystems/queries-and-explain.md) | `query`, `embed` | `embed` is the offline embedding path that routing reads ([HW-DR-0064](../decisions/0064-q64-whether-intent-time-routing-gains-an-offline-embedding-path-in-shadow-mode.md)). |
| projections, export | [Projections and export](../subsystems/projections-and-export.md) | `generate`, `mark` | `generate` writes the marker, and the census reads it. |
| authoring | [Authoring](../subsystems/authoring.md) | `scaffold`, `import` | Each writer of an authored document puts its files on the tree through `scaffold`, so `scaffold` also holds the writers of `check --fix` and `taxonomy migrate --apply`. `import` uses the reciprocal splice of `scaffold` and keeps no second one. |
| measurement | [Measurement](../subsystems/measurement.md) | `probe`, `conformance`, `sweep` | No code in these crates reaches a model, so no build waits on one. Each crate names `headwater-check`, so `headwater-check` can name none of them. The projections stage runs the intake and the grader of `probe`, because a probe result is a projection. |
| every stage | [Command surface](../subsystems/command-surface.md) | `cli`, `verbs`, `paint` | These crates run no stage. `verbs` is the list of verbs that the help, the refusals and the verb index read, and the parser in `cli` dispatches. `paint` is the terminal palette ([HW-DR-0045](../decisions/0045-coloring-the-cli-and-where-the-banner-goes.md)). |

The diagram above does not draw two subsystems. Authoring (`new`, `capture`, `import`) writes documents, and measurement (`probe`, `conformance`, `sweep`) measures a corpus or a consumer.

## Nothing stores the graph

The graph is a function of the corpus and the lock, and every run builds it again. [Q6](09-decisions.md#q6--where-the-corpus-graph-lives-at-rest) names the three artifacts that derive from it, how long each lives, and why none is canonical. [Spec 12](12-check-layer.md#the-correctness-roots) holds the cache to that with a test.

**There is no embedded database, and the trigger to add one is named.** The [language spike](../evaluations/language-spike-results.md) measured a warm change-scoped pass at about 2 ms for 1,000 documents, and the budget is 200 ms. Nothing in the design asks a question that the in-memory graph cannot answer inside the targets below. Only a named query workload that misses a target reopens this.

The industry does not agree, and the disagreement belongs in view. CodeQL ships a derived database as its query surface, at very large scale. The refusal here follows from spec 0's non-negotiables: offline, deterministic, and reviewable in a diff. It does not follow from a claim that a derived store cannot work ([evaluation](../evaluations/graph-export-and-federation.md)).

## Checks

A check is a pure function from a **scoped view** of the graph to findings, and it sees only what its scope declares. [Spec 12](12-check-layer.md) is the design of the check layer, from the origin of a check to its export.

## Projections

Projections are generated artifacts. Each one has the same contract:

```
headwater generate            # write
headwater generate --check    # fail if any committed output differs
```

**Thirteen projection kinds exist, and this engine emits eight of them.** The block below names all thirteen, as a taxonomy writes each name. A kind under `runs` has an emitter here. A kind under `waits` has a slot that a declaration opens and no emitter fills.

```
runs
  shelf_index
  shelf_sections
  site_nav
  graph_export
  probe_result
  verb_index
  consumer_surface
  corpus_descriptor

waits
  relation_view
  agent_rules
  template
  transcription
  coverage_report
```

[Projections and export](../subsystems/projections-and-export.md#thirteen-kinds-two-value-sets-and-the-unbuilt-set) says what each kind carries, why a waiting kind writes nothing, and why one run can take several passes. `engine/crates/generate/tests/spec_six_projections.rs` holds the block above against the engine. The coverage report is the one waiting kind that no taxonomy can declare, and [spec 13](13-open-obligations.md) carries it:

```
coverage_report the register
  its content is a function of the clock as well as of the corpus and the lock, because a migration task lapses and a suppression expires on a date. A committed copy would fail this check on a morning when nothing changed. Spec 13 carries it
```

**Eleven of the thirteen are declarable, and two are not.** For a declarable kind, a taxonomy names the output path, and [principle 1](00-vision-and-scope.md#design-principles) makes that path a schema decision. [Spec 4](04-assurance-model.md#every-obligation-has-exactly-one-disposition) fixes the coverage report, and [Q20](09-decisions.md#q20--where-scent-lives) fixes the corpus descriptor at `.headwater/corpus.json`.

**The rules that each projection obeys are part of the `headwater generate` contract**, which [states them](../interfaces/headwater-generate.md#what-a-projection-writes-and-why).

### A verb index reads the command surface of the engine

A verb index carries one row for every verb that the binary dispatches, and it marks a verb that no document describes. [The `headwater generate` contract](../interfaces/headwater-generate.md#a-verb-index) states where the rows come from, how a row joins a document, and when the index declines.

### An export is a projection, and it declares what it dropped

A graph export is one more projection, and every emitter declares what its target cannot carry. [Spec 7](07-distribution-and-federation.md#an-export-is-a-projection-and-it-declares-what-it-dropped) states the export rules: whether an export is committed, the loss set, the projection census, and what an emitter withholds.

### An export profile carries a filter

An export profile selects what one audience receives. [Spec 7](07-distribution-and-federation.md#an-export-profile-carries-a-filter) states what a profile names, the rules that make the filter honest, the tombstone grains, and why an exporter fails closed.

### What a filtered export claims, and what it does not

A filtered export claims one boundary and states beside it what it does not claim. [Spec 7](07-distribution-and-federation.md#what-a-filtered-export-claims-and-what-it-does-not) states the claim and each non-claim.

## Interfaces

### CLI

```
headwater check       [--strict] [--fix] [--no-cache] [--now <date>] [--change <manifest>]
                      [--read-set <path>] [--register <path>]
                      [--format text|json|sarif|markdown | --json]
headwater change      <base-rev> <out-dir>
headwater gate        --read-set <path> [--now <date>] [--json]
headwater derived
headwater site        <site-dir>
headwater merge-driver <ancestor> <current> <other> <path>
headwater generate    [--check]
headwater new         <kind> --title <text> [--summary <text>]
                      [--relates <relation>=<identifier>] [--facet <facet>=<value>]
                      [--directory <path>] [--now <date>]
headwater capture     [--format text|json | --json]
headwater sweep       plan [--under <path>]
                    | report <path> [--format text|json | --json]
headwater route       <task description> [--json]
headwater neighbors   <task description> [--model <dir>] [--top <n>] [--json]
headwater query       <expression>
headwater explain     <path|identifier> [--json]
headwater show        <path|identifier>
headwater mcp         [--now <date>] [--write]
headwater import      [<name>] [--expect <digest>] [--write]
headwater export      [--profile ...]
                      [--format json|jsonschema|shacl|rdf|skos|okf|linkml | --json]
                      [--at <date>] [--check]
headwater init        [--corpus <dir>] [--package <name>] [--git [--git-config]]
headwater infer       [--owner <name>] [--until <date>] [--write]
headwater conformance [--level <name>] [--now <date>] [--json]
headwater taxonomy    validate | resolve [--check]
                    | migrate <dir> [--to <version>] [--apply] [--now <date>]
                    | diff <dir> [--to <version>] [--now <date>]
                    | audit [--now <date>] [--record]
                    | publish [--package <name> | --from <dir>] [--assembly <name>] --out <dir>
                              [--clear-killed] [--json]
                    | vendor <dir-or-location> [--expect <digest>]
                    | graph [--view concrete|abstract] [--legend]
                    | kinds [--json]
headwater coverage    [--format ...]
headwater probe       plan [--tier regression|campaign|documentation] [--arm <arm>] [--delta]
                           [--category <name>] [--seed <n>]
                    | plan --instrument
                    | plan --folds
                    | plan --answer-keys <probe>
                    | record <path>
                    | grade <path>
                    | stale
headwater json        field <key>...
                    | count [<key>...]
                    | quote
headwater help        [<verb> [<word>]]
headwater completions bash|zsh|fish|powershell
```

**This grammar is a statement of fact about the engine, and a name it declares either runs or waits.** Every verb the engine ships is above. `engine/crates/cli/tests/verbs.rs` holds this claim as a containment against `headwater_verbs::VERBS`. It names a verb the table carries when this block does not carry it. It also names a block entry when the table does not carry it and this paragraph does not declare the entry waiting. A name that the engine has not built stays here when something nameable would make it real. The engine then says what the name waits on when a caller types it. `query <expression>` is such a name, because no document states what an expression is. The verb ships the day one does. The same reading covers `coverage` and the five export targets that no consumer has asked for. Each of those states its wait when a caller types it. `taxonomy migrate` was such a name and it now runs. `--apply` rewrites the facet value of every document that a mechanical step covers, and the address of every overlay entry it covers. It emits the judgment steps as a task list. One of the three writes that the name waited on stays open. Nothing writes the open task set into the lock, and [spec 7](07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states) states why that omission is a decision. A name that nothing could make real has no place here, and `--changed-only` is the one such name this grammar carried. The test between the two is not how far away the work is. It is whether any document or any consumer could turn the name into a verb that runs.

**Each verb states its own rules in its contract.** The [interface contracts](../interfaces/README.md) hold one page for each verb above, with its flags, its two streams, its exit status and the files it touches. A refusal is never a document in the target that a caller named. [Q43](09-decisions.md#q43--whether-a-refusal-under---json-is-a-json-document) rules it, and each contract states it for its own verb.

The CLI is advisory by default (exit 0 with findings on stdout). Use `--strict` for gates. The default is deliberate: a tool that blocks on first contact is removed, and a removed tool catches nothing.

**No flag decides which findings count, and there is no `--changed-only`.** A flag that took a caller's list of changed documents would put a second input into the verdict that no reviewer sees. It would also report the rest as neither checked nor skipped. A scope derived from content rather than from a list is the cache, and the cache ships. That hidden input is the argument against the flag, and cost is not. The commit hook runs a full warm `check --strict`, and the cache keeps it inside its 200 ms target. It cost 193 ms of CPU time at 501 typed documents on 2026-10-01. A flag would buy time with a verdict that no reviewer can see, so the flag stays refused. [HW-OBL-0080](../obligations/0080-changed-only-is-the-content-addressed-cache-under-another-name.md) holds the measurement and the condition that reopens it. An adopter who wants patient debt gets it from the [adoption payload](07-distribution-and-federation.md#first-contact-adoption-is-a-migration-from-no-taxonomy). That is a fact about the corpus, rather than a property of an invocation ([Q12](09-decisions.md#q12--migration-path-for-an-existing-corpus)).

### `taxonomy validate` versus `taxonomy audit`

There are two commands because there are two kinds of question. The distinction is the standard one between reasoning over a **TBox** (the terminology) and reasoning over an **ABox** (the assertions) against it ([spec 1](01-conceptual-model.md#two-layers-terminology-and-assertions)).

**`validate`** decides the schema alone: referential integrity, determinism, purpose completeness, kind rigidity, edge provenance, overlay confluence, core satisfiability. It needs no documents, always terminates in a verdict, and gates everything.

**`audit`** measures the schema *against a corpus*. [The `headwater taxonomy` contract](../interfaces/headwater-taxonomy.md#description) states each reading and what it reports.

The separation matters. `validate` must stay fast and total because it gates, while `audit` is a periodic design review with a tool attached.

### Library

The CLI is a thin shell over a library API — load, graph, check, query, generate. The MCP server consumes the library in the same process. An editor integration is a client of the MCP server: it starts `headwater mcp` for each query and reads the structured answer ([HW-DR-0102](../decisions/0102-an-editor-integration-starts-headwater-mcp-for-each-query-and-does-not-embed-the-library.md)). The CI adapter under `integrations/headwater-check/` runs the released `headwater` binary.

### MCP server

The MCP server is the surface of the library that an agent reads ([AI integration](05-ai-integration.md)). [The `headwater mcp` contract](../interfaces/headwater-mcp.md#description) states its tools and its session, and [Queries and explain](../subsystems/queries-and-explain.md#mcp) states how it answers.

### CI adapters

The engine emits findings. Adapters translate them to the native vocabulary of a platform: annotations, check runs, job summaries, review comments. Adapters are thin and swappable so that no forge is privileged in the core. Portability is a requirement, not an aspiration. Coupling of the core to one CI platform is a stated failure mode that we correct ([spec 8](08-design-departures.md)). [Checks and cache](../subsystems/checks-and-cache.md#a-renderer-is-what-the-engine-ships-and-an-adapter-is-a-renderer-with-a-credential) states the four formats and the loss set of each. It also states the severity map, the marking of a suppressed finding, and what a run states beside its findings.

## Performance targets

| Operation | Target | Measured |
|---|---|---|
| Full check with no cache, 1,000 documents: `check --strict --no-cache` | < 5 s | 587 ms at 501 typed documents, 2026-10-01. Nothing has measured 1,000 documents. |
| Full check, warm cache: `check --strict` | < 1 s | 193 ms at 501 typed documents, 2026-10-01. |
| Commit hook: the full warm `check --strict --change`, because no mode narrows a check to a change ([HW-OBL-0080](../obligations/0080-changed-only-is-the-content-addressed-cache-under-another-name.md)) | < 200 ms | 193 ms at 501 typed documents, 2026-10-01. |
| Edit hook: `explain --json --paths-at-most 20` and the `headwater json` reads of its document | < 200 ms | 90 ms for a document that governs one glob over 50,000 files, 2026-09-30. |
| Route query: `route` | < 100 ms | 60 ms at 444 checked documents, 2026-09-28. |

Every value in this table is user and system CPU time, because load on a shared host moves CPU time less than wall time. Each value is a median over warm runs of the `dev-release` engine on an 18-core host. `sh tools/measure/mcp-check.sh` measured the three full-check rows, and `sh tools/measure/large-governs.sh` measured the edit hook. [Spec 5](05-ai-integration.md#review-time-checks) states the route measurement. [HW-OBL-0080](../obligations/0080-changed-only-is-the-content-addressed-cache-under-another-name.md) records where the time of a warm check goes and what would change the position.

Hooks and agent-facing queries must be fast enough to be invisible. A pre-commit check that costs two seconds is bypassed within a week. The developer who wrote the agent loop will skip a routing call that costs a second.

## Implementation constraints

- **Single binary or single runtime.** Installation of the engine must not require a toolchain per check. A mix of Python, Node, and shell — each with its own container fallback — is a cost that we do not accept.
- **Offline.** No network at check time.
- **Deterministic.** Same corpus, same lock, same output, byte for byte. This is what makes `--check` on projections meaningful.
- **Embeddable.** The library runs in process for the MCP server and any Rust caller, and an editor is a client of `headwater mcp` ([HW-DR-0102](../decisions/0102-an-editor-integration-starts-headwater-mcp-for-each-query-and-does-not-embed-the-library.md)).

These constraints made most of the choice. The language is Rust, and the embeddable requirement above is the argument that decided it ([Q1](09-decisions.md#q1--implementation-language)).
