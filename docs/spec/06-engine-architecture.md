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

**Parse** reads each file once: front matter, headings, links, code fences. It does not interpret. Classification assigns a kind by the declared resolution rules and records the derivation for `explain`.

**Graph build** resolves relations into edges, indexes identifiers, binds external anchors (code paths, work items, URLs), and reports what it could not resolve. It also emits a **census**: every file under the corpus root and the outcome for each. The census fixes the denominator for coverage before any check runs. Thus a document that failed to classify is visibly unchecked, not silently absent.

**Cache** is content-addressed per file plus taxonomy hash, so incremental runs are proportional to the change, not the corpus. The change-scoped mode that CI and hooks use is the same code path with a smaller working set.

## Checks

A check is a pure function from a **scoped view** of the graph to findings. Checks come from five origins:

| Origin | Comes from | Exportable as |
|---|---|---|
| **Shape** | the taxonomy, generated | LinkML + SHACL |
| **Graph** | relation declarations, generated | SHACL |
| **Corpus** | declarations that need many documents at once | — |
| **Document** | regimes applied to the body, which is not in the graph | — |
| **Plugin** | adopter code | — |

The first two are *generated*: a new facet or relation brings its checks with no code. That is the point of taxonomy-as-data, and most of the check count is there. The last three are why a native engine exists at all. They are exactly what LinkML and SHACL cannot express.

Every check declares its **scope** (document, edge, neighborhood, shelf, corpus), and the engine enforces it. A check sees only what it declared. Scope is what makes change-scoped evaluation exact, cache keys sound, and parallelism safe.

The design — scope semantics, instances and coverage, the two-phase census, fixability, determinism, and the plugin contract — is [spec 12](12-check-layer.md).

## Projections

Projections are generated artifacts. The taxonomy declares each one, and each has the same contract:

```
headwater generate            # write
headwater generate --check    # fail if any committed output differs
```

The engine implements these projection kinds: shelf indexes, relation views (decision lineage, traceability matrices), agent rule files, site navigation, graph export, coverage reports, and templates. A projection carries a generated-file marker. The engine refuses to overwrite a file that lacks the marker and did not come from a previous run. Thus a projection can never silently destroy an authored document.

## Interfaces

### CLI

```
headwater check      [--changed-only] [--strict] [--format text|json|sarif|markdown]
headwater generate   [--check]
headwater new        <kind> [--title ...]
headwater route      <task description>
headwater query      <expression>
headwater explain    <path|identifier>
headwater taxonomy   validate | resolve | diff | migrate | audit
headwater coverage   [--format ...]
headwater probe      [--category ...]
```

The CLI is advisory by default (exit 0 with findings on stdout). Use `--strict` for gates. The default is deliberate: a tool that blocks on first contact is removed, and a removed tool catches nothing.

### `taxonomy validate` versus `taxonomy audit`

There are two commands because there are two kinds of question. The distinction is the standard one between reasoning over a **TBox** (the terminology) and reasoning over an **ABox** (the assertions) against it ([spec 1](01-conceptual-model.md#two-layers-terminology-and-assertions)).

**`validate`** decides the schema alone: referential integrity, determinism, purpose completeness, kind rigidity, edge provenance, overlay confluence, core satisfiability. It needs no documents, always terminates in a verdict, and gates everything.

**`audit`** measures the schema *against a corpus*. It measures facet differentiation and orthogonality, edge counts and staleness by `created_by`, and relation-choice drift by family. It also measures discriminator distribution on heterogeneous shelves, state-dwell distribution, transition-continuity distribution, and scent quality. Its findings are advisory by construction, because a young or small corpus fails differentiation for reasons that are not defects. The findings are about the taxonomy, not the documents. A facet that nothing distinguishes is a schema problem that only documents can show.

The separation matters. `validate` must stay fast and total because it gates, while `audit` is a periodic design review with a tool attached.

### Library

The CLI is a thin shell over a library API — load, graph, check, query, generate. Editor integrations, the MCP server, and CI adapters all consume the library directly. They do not start a subprocess and parse text.

### MCP server

The MCP server is the agent-facing surface of the same library ([AI integration](05-ai-integration.md)).

### CI adapters

The engine emits findings. Adapters translate them to the native vocabulary of a platform: annotations, check runs, job summaries, review comments. Adapters are thin and swappable so that no forge is privileged in the core. Portability is a requirement, not an aspiration. Coupling of the core to one CI platform is a stated failure mode that we correct ([spec 8](08-design-departures.md)).

## Performance targets

| Operation | Target |
|---|---|
| Full check, 1,000 documents, cold | < 5 s |
| Full check, warm cache | < 1 s |
| Change-scoped check (hook) | < 200 ms |
| Route query | < 100 ms |

Hooks and agent-facing queries must be fast enough to be invisible. A pre-commit check that costs two seconds is bypassed within a week. The developer who wrote the agent loop will skip a routing call that costs a second.

## Implementation constraints

- **Single binary or single runtime.** Installation of the engine must not require a toolchain per check. A mix of Python, Node, and shell — each with its own container fallback — is a cost that we do not accept.
- **Offline.** No network at check time.
- **Deterministic.** Same corpus, same lock, same output, byte for byte. This is what makes `--check` on projections meaningful.
- **Embeddable.** Usable as a library from an editor plugin or an agent process, with no need to spawn subprocesses.

The language choice is deferred ([open questions](09-open-questions.md)), but these constraints make most of the choice.
