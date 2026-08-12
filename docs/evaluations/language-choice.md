---
id: EVAL-HW-language-choice
status: current
status_since: 2026-08-10
last_verified: 2026-08-10
summary: The Q1 evidence, which decides the implementation language on embedding, scope enforcement and sum types rather than on speed.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: evaluate+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cited_by:
    - REG-HW-decisions
---

# The implementation language — the evidence for Q1

The evidence for [Q1](../spec/09-open-questions.md#q1--implementation-language). The question listed four candidates. Two of them fall to a constraint that [spec 6](../spec/06-engine-architecture.md#implementation-constraints) already states, so this document argues the remaining two.

**Python and Node are out on distribution, not on speed.** Spec 6 requires a single binary and no toolchain per check. It also gives a 200 ms budget for the change-scoped run that a commit hook performs. An interpreter start costs a large fraction of that budget before any work begins. And a mix of Python, Node, and shell, each with its own container fallback, is the named cost that spec 6 refuses. That leaves Rust and Go.

**The conclusion is Rust, and the reasons are not the ones that Q1's table gives.** Speed does not decide between these two. The three arguments that do decide it come from constraints that the specification already committed to, in spec 6 and [spec 12](../spec/12-check-layer.md).

## Speed is spent by the time the question reaches Rust and Go

Q1's table awards "fast" to Rust and "fast enough" to Go. Neither entry does any work, because the performance targets are not close for either language.

| Operation | Target | Rust | Go |
|---|---|---|---|
| Full check, 1,000 documents, cold | < 5 s | comfortable | comfortable |
| Change-scoped check (hook) | < 200 ms | comfortable | comfortable |
| Process start | inside the above | ~1 ms | ~1 ms |

A thousand Markdown documents is a small corpus. Hugo builds sites an order of magnitude larger on a Go Markdown parser, and it does so in the same time class. Garbage-collection pauses at this heap size are below a millisecond, and the run is short enough that the collector may never do generational work at all.

So the performance argument is real, and it is completely spent on the exclusion of Python and Node. To carry it forward as a reason to prefer Rust over Go is to double-count it. The decision has to rest on something else.

## What decides it

### 1. Embeddable means embeddable, and spec 6 means it literally

Spec 6 states the constraint twice. Under implementation constraints: "Usable as a library from an editor plugin or an agent process, with no need to spawn subprocesses." Under interfaces: "Editor integrations, the MCP server, and CI adapters all consume the library directly. They do not start a subprocess and parse text."

The hosts are known. An editor extension is Node or the JVM. An agent process is Python or Node. [Q16](../spec/09-open-questions.md#q16--public-presence) wants a site generated from the corpus, and [Q14](../spec/09-open-questions.md#q14--discovery-surface) wants a reader that encounters a corpus cold. Those two point at a browser.

Rust reaches every one of those hosts as an in-process library. A `cdylib` with a C ABI adds no runtime to the host. `napi-rs` produces a native Node addon, `pyo3` produces a Python extension, and `wasm32` produces a browser build of the same crate. Ruff and uv ship this way through pip, and Biome ships this way through npm. The route is ordinary.

Go reaches those hosts differently, and the difference is exactly what spec 6 rules out. `-buildmode=c-shared` puts the whole Go runtime inside the host process, with its own scheduler, collector, and signal handlers. That is a poor guest in a JVM or a CPython process, and two such libraries in one process is worse. Go compiled to WebAssembly carries the collector into the browser and produces binaries in the multi-megabyte class, and TinyGo avoids that by compiling a subset of the language.

**esbuild is the precise counter-example, and it proves the point rather than refuting it.** esbuild is Go, it is fast, and it reached the Node ecosystem completely. It reached it by shipping a binary that its JavaScript API starts as a child process and talks to over a pipe. That is the architecture that spec 6 forbids in the sentence quoted above. The most successful Go developer tool in the JavaScript ecosystem arrived by the one route that this design has already declined.

### 2. Scope enforcement is a type-system problem, and spec 12 already says so

Spec 12 names scope enforcement as a correctness root, in terms that are unusually specific:

> The view exposes *only* what the scope declared. A `Document`-scoped check physically cannot read a sibling. So a scope declaration cannot quietly rot into a lie. A scope that is declared but not enforced would silently corrupt every cache key derived from it. The enforcement is the feature, and the declaration alone would be a comment.

And the failure mode, from the same document: "A leak reproduces deterministically, so it looks like correct behavior."

In Rust, the view is a borrow with a lifetime, and it holds no path back to the graph:

```rust
pub struct DocumentView<'g> {
    doc: &'g Document,     // the only handle a Document-scoped check receives
}

pub trait Check: Send + Sync {
    fn scope(&self) -> Scope;
    fn evaluate(&self, view: &DocumentView<'_>) -> Vec<Finding>;
}
```

A check that wants a sibling has no field to follow and no graph to ask. It must change its declared scope, which changes the type it receives, which changes the inputs in its cache key. The leak is a compile error rather than a review finding.

Two further properties of spec 12 come with the same signature. `evaluate` takes `&self` and not `&mut self`, so a check cannot accumulate state between instances. `Check: Send + Sync` is what lets the runner evaluate document and edge checks in parallel, and it rejects a check that holds shared mutable state. Purity and parallel safety stop being rules that a reviewer enforces and become properties that the build enforces.

Go expresses the same design and enforces none of it. A view is a struct of pointers into a shared graph, and nothing prevents a method from reaching further than its scope declared. A check with a struct receiver can mutate itself and race under the parallel runner. `go test -race` finds the race after somebody writes a test that triggers it, which is later than a build error and less certain.

This argument matters more than it would in most systems. Spec 12 lists scope enforcement beside the census walker and the overlay resolver, as a component whose defect produces systematically green results. A wrong answer here is invisible.

### 3. The graph and its derivations are sum types

Spec 12 declares `Scope` as five alternatives, one of which carries a depth. `Severity`, `Finding` provenance, the taxonomy's abstract syntax, and the derivation record that `explain` prints are all the same shape. So is the classification outcome in the census, where every file resolves to one of a closed set of results.

Rust has enums and exhaustive `match`. Adding a scope variant produces a compile error at every site that must handle it, and the list of those sites is the compiler's output rather than a search. That property is worth most at exactly the moments this design expects: spec 12 already asks whether `Neighbourhood(depth)` should be cut and whether `Shelf` should collapse into `Corpus`. Both are edits to a closed set.

Go models a closed set as an interface plus a type switch, with no exhaustiveness check. A new variant compiles everywhere and fails at run time in whichever branch nobody updated. That is what Q1's table meant by "weaker ergonomics for schema and graph work". The cost is persistent rather than one-time, because taxonomy-as-data makes the engine mostly a set of interpreters over closed sets.

## What does not decide it

Three arguments look decisive and are not. Each one is recorded here so that nobody re-runs it.

**The RDF stack is a tiebreaker at most.** Rust has Oxigraph for SPARQL and rudof for SHACL and ShEx, both embeddable, and rudof comes out of a research group with published work behind it. Go's equivalent is thinner and younger. But [Q13](../spec/09-open-questions.md#q13--linkml-and-shacl-as-substrate) already settled that the engine never runs on SHACL's validation machinery, and that SHACL emission waits for an external consumer who asks for it. The only near-term use is the differential-testing oracle. A cross-check that runs in continuous integration may call an external validator as a subprocess, because nothing about it is on the check path. So this is a convenience and not a constraint.

**WebAssembly plugin hosting is a wash.** Spec 12 leaves plugins open between in-process and subprocess. WebAssembly answers both horns, because a component runs in-process and receives no filesystem, no network, and no clock unless the host grants them. That is spec 12's plugin contract restated as a capability model. Rust hosts it with wasmtime and Go hosts it with wazero, which is pure Go and needs no cgo. Neither language wins the host side. Rust wins only the other direction, where the engine itself compiles to WebAssembly, and that is argument 1 rather than a separate point.

**Determinism is a wash, and Go may be slightly ahead.** Both languages need the discipline that spec 12 states: sorted output, no unordered iteration reaching a result, an injected clock. Rust makes ordered iteration a type choice through `BTreeMap`. Go randomizes map iteration on purpose, which surfaces an accidental order dependency in the first test run rather than the hundredth. Call it even.

## What Rust costs, stated plainly

**The YAML ecosystem is in poor repair.** `serde_yaml` was archived in 2024. `serde_yml`, the most visible fork, was archived after soundness advisories in 2025. The maintained options are `yaml-rust2` and `yaml-spanned`. Go's `goccy/go-yaml` is healthier and reports source positions well.

That cost is smaller than it looks, for one reason. Spec 12 requires spans: "the parser retains spans for front-matter keys, headings, and links". No convenient `Deserialize` implementation gives that. Headwater builds the front-matter layer over a streaming event parser in either language. So the ecosystem gap lands on a thin layer that is custom work regardless. It is a real cost and it is the first thing the spike must retire.

**Iteration speed and the contributor pool are real, and the architecture shrinks both.** Spec 6 and spec 12 agree that most of the check count is generated from the taxonomy and that adopters extend the system through plugins. So the surface where anyone writes engine code is small, and the surface where an adopter writes code is a WebAssembly component in whichever language they prefer. Design principle 1 says that an adopter who needs a change to the engine indicates a design fault. If that principle holds, the contributor-pool argument mostly dissolves. If it fails, Headwater has a worse problem than its language.

**Compile times are real.** A workspace split along the pipeline stages of spec 6 keeps the inner loop small. This is an ordinary cost with an ordinary mitigation.

## The confrontation, as principle 10 requires

**What the sources confirm.** The Rust route for a single-binary developer tool that must live inside Node, Python, and browser hosts is well traveled. Ruff, uv, Biome, oxc, rust-analyzer, and Deno all take it, and the packaging is understood well enough to be boring.

**What the sources sharpen.** The reason is not speed. Every one of those projects could have met its performance targets in Go. They needed to live inside a host process that they did not own, which is the constraint that spec 6 states and argument 1 develops.

**What the sources contradict.** The closest observed analog to Headwater's check engine is Vale, which spec 0 names as a complementary tool. Vale is Go. It is a single binary, it walks a tree of Markdown, it applies configured rules, and it emits findings. Hugo, on the same parser stack, handles Markdown corpora far larger than any target here. So the claim "this shape of tool needs Rust" is false, and Q1 should not pretend otherwise. Go builds this engine. What Go does not build well is the *library* underneath the engine. Spec 6 describes the CLI as a thin shell over that library. The bet is on the library, and that is where the languages differ.

## The spike that Q1 asked for is the wrong spike

Q1 says: "Decide with a spike that implements the parse-classify-graph path in two candidates."

That spike answers nothing now. Both candidates produce a working parse-classify-graph path, and neither prototype exercises any of the three arguments above. It would measure the axis on which the two languages are equal and skip all three axes on which they differ.

Replace it with a risk-retirement spike in Rust alone. Each item below can falsify the decision, and Go remains the fallback until all four pass.

**All four passed.** The code is in [`spike/`](../../spike/) and the measurements are in the [results](language-spike-results.md).

| Retires | Acceptance | Result |
|---|---|---|
| The YAML gap | A front-matter parse that reports line and column for every key, and a `Finding` that carries the position to output | pass — 10 assertions |
| Scope enforcement (argument 2) | A test that fails to compile when a `Document`-scoped check reaches a sibling. A test that compiles proves the claim false | pass — 6 leaks rejected, 3 controls compile |
| Embedding (argument 1) | The same crate built as a native Node addon and as `wasm32`, with both binary sizes recorded | pass — 600 KiB addon, 113 KiB wasm |
| The budget | A change-scoped run over a synthetic 1,000-document corpus, warm, under 200 ms | pass — about 2 ms |

Item two is the one to run first. It is the argument that carries the most weight and the only one where Rust could fail to deliver what this document claims for it.

Two things the results add to this document. The YAML cost is real and takes a shape this document did not predict: the parser's own documentation is wrong about column indexing and about how a blank value arrives, and only an empirical check finds either. And argument 2 turned out to be stronger than stated, because scope is better carried as a *type* than as a value a check returns. That correction belongs to [spec 12](../spec/12-check-layer.md) and is not a language argument, although only the type system makes it available.

## What stays reversible

The CLI is a thin shell over a library, so almost all of the investment is in one crate rather than in a command-line program. Nothing in the specification depends on the language. Spec 12 states this in its own opening: the signatures are pseudocode and nothing there depends on Q1.

The point of no return is the first release, because that is when adopters acquire installed binaries and editor integrations. Before it, a switch to Go costs the engine and keeps the taxonomy, the specification, the fixtures, and the corpus. That is the window the spike above runs inside.
