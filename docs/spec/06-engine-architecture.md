# 6 — Engine architecture

One parse, one graph, many consumers.

## Why one engine

The obvious decomposition — a separate linter per concern — is the wrong one. Each
tool re-walks the tree, re-parses front matter, re-implements path matching, and
re-derives what kind each document is. That is slow, and worse, it is *divergent*:
two tools disagreeing about what a document is produces contradictory findings and
an unfixable bug report.

docgov parses once, builds one typed graph, and runs every check against it. Checks
become small predicates over a shared model instead of programs.

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

**Resolve** merges the base taxonomy and overlays, validates against the
meta-schema, and writes a content-hashed lock. Everything downstream reads the lock,
never the sources — so a check result depends on a hash a reviewer can see in a
diff.

**Parse** reads each file once: front matter, headings, links, code fences. It does
not interpret. Classification assigns a kind by the declared resolution rules and
records the derivation for `explain`.

**Graph build** resolves relations into edges, indexes identifiers, binds external
anchors (code paths, work items, URLs), and reports what could not be resolved.

**Cache** is content-addressed per file plus taxonomy hash, so incremental runs are
proportional to the change, not the corpus. The change-scoped mode used by CI and
hooks is the same code path with a smaller working set.

## Checks

A check is a pure function `(graph, config) → findings`. Three tiers:

| Tier | Source | Example |
|---|---|---|
| **Schema-derived** | Generated from the taxonomy | Required facets present, enum values valid, sections present, relation cardinality, reciprocity, nuclearity and satellite inheritance, lifecycle transitions, sequence expectations, identifier format |
| **Built-in** | Shipped with the engine, configured by taxonomy | Voice regime, link resolution, staleness, size budgets, projection freshness, orphan detection, reference directionality, transition continuity, summary scent |
| **Plugin** | Adopter-supplied | Anything organisation-specific — an internal identifier format, a compliance mapping, a house rule |

Most of what an adopter wants is tier one, and tier one costs nothing to add: a new
relation type in the schema brings its cardinality, endpoint, and reciprocity checks
with it.

### Plugin interface

Deliberately narrow. A plugin receives the built graph and returns findings. It gets
no file-system walk, no parsing, and no ability to mutate the graph. That
constraint keeps plugins fast, cacheable, and incapable of introducing the
divergence the single-engine design exists to prevent.

## Projections

Generated artefacts, each declared in the taxonomy, each with the same contract:

```
docgov generate            # write
docgov generate --check    # fail if any committed output differs
```

Projection kinds the engine implements: shelf indexes, relation views (decision
lineage, traceability matrices), agent rule files, site navigation, graph export,
coverage reports, and templates. A projection carries a generated-file marker; the
engine refuses to overwrite a file that lacks one and did not come from a previous
run, so a projection can never silently eat an authored document.

## Interfaces

### CLI

```
docgov check      [--changed-only] [--strict] [--format text|json|sarif|markdown]
docgov generate   [--check]
docgov new        <kind> [--title ...]
docgov route      <task description>
docgov query      <expression>
docgov explain    <path|identifier>
docgov taxonomy   validate | resolve | diff | migrate | audit
docgov coverage   [--format ...]
docgov probe      [--category ...]
```

Advisory by default (exit 0 with findings on stdout); `--strict` for gates. The
default is deliberate: a tool that blocks on first contact gets removed, and a tool
that is removed catches nothing.

### `taxonomy validate` versus `taxonomy audit`

Two commands because two kinds of question — and the distinction is the standard
one between reasoning over a **TBox** (the terminology) and reasoning over an
**ABox** (the assertions) against it
([spec 1](01-conceptual-model.md#two-layers-terminology-and-assertions)).

**`validate`** decides the schema alone:
referential integrity, determinism, purpose completeness, kind rigidity, edge
provenance, overlay confluence, core satisfiability. It needs no documents, always
terminates in a verdict, and gates everything.

**`audit`** measures the schema *against a corpus*: facet differentiation and
orthogonality, edge counts and staleness by `created_by`, transition-continuity
distribution, scent quality. Its findings are advisory by construction — a young or
small corpus fails differentiation for reasons that are not defects — and they are
about the taxonomy, not the documents. A facet nothing distinguishes is a schema
problem that only documents can reveal.

Keeping them apart matters: `validate` must stay fast and total because it gates,
while `audit` is a periodic design review with a tool attached.

### Library

The CLI is a thin shell over a library API — load, graph, check, query, generate.
Editor integrations, the MCP server, and CI adapters all consume the library
directly rather than shelling out and parsing text.

### MCP server

The agent-facing surface of the same library ([AI integration](05-ai-integration.md)).

### CI adapters

The engine emits findings; adapters translate them to a platform's native
vocabulary — annotations, check runs, job summaries, review comments. Adapters are
thin and swappable so that no forge is privileged in the core. Portability is a
requirement, not an aspiration: the reference system's coupling to one CI platform
is a stated failure mode we are correcting.

## Performance targets

| Operation | Target |
|---|---|
| Full check, 1,000 documents, cold | < 5 s |
| Full check, warm cache | < 1 s |
| Change-scoped check (hook) | < 200 ms |
| Route query | < 100 ms |

Hooks and agent-facing queries must be fast enough to be invisible. A pre-commit
check that costs two seconds gets bypassed within a week, and a routing call that
costs a second gets skipped by whoever wrote the agent loop.

## Implementation constraints

- **Single binary or single runtime.** Installing the engine must not require a
  toolchain per check. The reference system's mix of Python, Node, and shell — each
  with its own Docker fallback — is a cost we do not repeat.
- **Offline.** No network at check time.
- **Deterministic.** Same corpus, same lock, same output, byte for byte. This is
  what makes `--check` on projections meaningful.
- **Embeddable.** Usable as a library from an editor plugin or an agent process
  without spawning subprocesses.

Language choice is deferred ([open questions](09-open-questions.md)), but these
constraints do most of the choosing.
