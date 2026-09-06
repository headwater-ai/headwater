<!-- headwater:generated verb_index. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file. -->

# The command surface

`headwater` dispatches 20 verbs. 20 of them have a contract on this shelf, and 0 have none.

A verb with no contract carries a mark in the last column, and that cell is what this file is for. An index of the contracts that exist would say nothing about the verbs nobody has described yet.

The headings and the second column are the groups and the summaries that `headwater --help` prints, read off the same table. The long description of a verb is what `headwater help <verb>` prints, and it is not repeated here.

## Checking a corpus

| Verb | What it does | What a caller types | Contract |
|---|---|---|---|
| `check` | run the pipeline over the corpus, against the committed lock | `headwater check` | [headwater check](headwater-check.md) |
| `gate` | hold an earlier run's read set against the tree in front of it | `headwater gate` | [headwater gate](headwater-gate.md) |
| `conformance` | evaluate this repository against a package's conformance rules | `headwater conformance` | [headwater conformance](headwater-conformance.md) |

## Reading a corpus

| Verb | What it does | What a caller types | Contract |
|---|---|---|---|
| `route` | resolve a task description to the documents that govern it | `headwater route` | [headwater route](headwater-route.md) |
| `explain` | why a document is the kind it is, and what it serves | `headwater explain` | [headwater explain](headwater-explain.md) |
| `query` | listed in spec 6, and no document says what an expression is | `headwater query` | [headwater query](headwater-query.md) |
| `capture` | read the capture-cost store back | `headwater capture` | [headwater capture](headwater-capture.md) |
| `mcp` | serve the reads and one run of the checks to an agent | `headwater mcp` | [headwater mcp](headwater-mcp.md) |

## Writing a corpus

| Verb | What it does | What a caller types | Contract |
|---|---|---|---|
| `new` | scaffold a document of a kind | `headwater new` | [headwater new](headwater-new.md) |
| `infer` | report the debt this taxonomy raises over this corpus | `headwater infer` | [headwater infer](headwater-infer.md) |
| `generate` | write every projection the taxonomy declares | `headwater generate` | [headwater generate](headwater-generate.md) |
| `import` | write the edges a committed snapshot declares | `headwater import` | [headwater import](headwater-import.md) |
| `export` | emit a declared export profile through an emitter target | `headwater export` | [headwater export](headwater-export.md) |

## Sampling, which never gates

| Verb | What it does | What a caller types | Contract |
|---|---|---|---|
| `sweep` | the two halves of the coherence sweep | `headwater sweep plan`, `headwater sweep report` | [headwater sweep](headwater-sweep.md) |
| `probe` | the four parts of the probe harness | `headwater probe plan`, `headwater probe record`, `headwater probe grade`, `headwater probe stale` | [headwater probe](headwater-probe.md) |

## The taxonomy

| Verb | What it does | What a caller types | Contract |
|---|---|---|---|
| `init` | scaffold the consumer declaration and the overlay | `headwater init` | [headwater init](headwater-init.md) |
| `taxonomy` | read, write, publish and compare a taxonomy package | `headwater taxonomy validate`, `headwater taxonomy resolve`, `headwater taxonomy audit`, `headwater taxonomy publish`, `headwater taxonomy vendor`, `headwater taxonomy diff`, `headwater taxonomy migrate` | [headwater taxonomy](headwater-taxonomy.md) |

## Reading a wire format, and no corpus

| Verb | What it does | What a caller types | Contract |
|---|---|---|---|
| `json` | read one member of the JSON object on standard input | `headwater json field`, `headwater json count`, `headwater json quote` | [headwater json](headwater-json.md) |

## Getting help

| Verb | What it does | What a caller types | Contract |
|---|---|---|---|
| `help` | the long description of one verb, or this screen | `headwater help` | [headwater help](headwater-help.md) |
| `completions` | write the completion script of one shell | `headwater completions` | [headwater completions](headwater-completions.md) |
