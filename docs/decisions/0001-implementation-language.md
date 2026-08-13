---
id: DR-repo-0001
title: Q1 — Implementation language
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: The language is Rust, with a WebAssembly build of the same crate for editor and browser embedding.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - EVAL-HW-language-choice
    - EVAL-HW-language-spike-results
---

# Q1 — Implementation language

## Context

The design phase deferred the implementation language, and four candidates were in view: Rust, Go, Python and TypeScript. The full argument is a separate [evaluation](../evaluations/language-choice.md), and the spike that tested it [passed on all four items](../evaluations/language-spike-results.md).

Python and TypeScript are out on distribution. Spec 6 requires a single binary and no toolchain per check, and it gives 200 ms to the change-scoped run that a commit hook performs. An interpreter start spends a large part of that budget before any work begins.

Rust and Go were the real question, and the table that the open-question entry used to carry decided it on the wrong axis. **Performance does not separate them.** A thousand Markdown documents is a small corpus, and both languages clear every target in [spec 6](../spec/06-engine-architecture.md#performance-targets) comfortably. The performance argument is completely spent on the exclusion of the two interpreted candidates, and to carry it forward is to count it twice.

Three arguments decide it, and each one comes from a constraint that the specification already made.

**Embedding.** [Spec 6](../spec/06-engine-architecture.md#library) requires that editor integrations, the MCP server, and CI adapters consume the library in-process and do not start a subprocess. Rust reaches a Node, Python, or JVM host as a shared library that adds no runtime to it, and reaches a browser through WebAssembly. Go reaches those hosts by putting its scheduler and collector inside them. The esbuild project is the proof of this rather than the counter-example. It is the most successful Go tool in the JavaScript ecosystem, and it arrived by the subprocess route that spec 6 declines.

**Scope enforcement.** [Spec 12](../spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on) says that a `Document`-scoped check must be unable to read a sibling. It adds that the enforcement is the feature, and that a leak silently corrupts every cache key derived from it. In Rust, a view is a borrow that holds no path back to the graph, so the leak is a compile error. The same signature makes check purity and parallel safety properties of the build. In Go, the same design compiles and the enforcement returns to code review. Spec 12 lists this component beside the census walker as one whose defect produces systematically green results.

**Sum types.** `Scope`, `Severity`, provenance, the taxonomy syntax, and the classification outcome in the census are all closed sets. Exhaustive matching turns a change to one of them into a compiler-generated list of the sites to fix. Spec 12 already expects two such changes. Taxonomy-as-data means that the engine is mostly interpreters over closed sets, so this is a recurring cost rather than a one-time one.

Three arguments that look decisive are not, and are recorded so that nobody re-runs them. The **RDF stack** is thicker in Rust, but [Q13](0013-linkml-and-shacl-as-substrate.md) already puts SHACL off the check path, so an external validator may run as a subprocess. **WebAssembly plugin hosting** is available in both. **Determinism** needs the same discipline in both.

**The counter-evidence, kept in view:** [Vale](../spec/00-vision-and-scope.md#what-we-do-not-build) is the closest observed analog to this engine, and it is Go. Go builds this command-line tool. What Go builds poorly is the embeddable library underneath it, which is where the whole investment goes.

## Decision

The language is **Rust**, with a WebAssembly build of the same crate for editor and browser embedding. Go is no longer the fallback.

## Consequences

The costs are real and stated in the evaluation. The YAML crate ecosystem is in poor repair, which matters less than it looks because spec 12 requires source spans that no convenient deserializer supplies. Iteration speed and the contributor pool are genuine, and the architecture shrinks the surface where anyone writes engine code at all.

**The spike changed, and then it ran.** A parse-classify-graph path in two candidates would have measured the one axis on which the languages are equal. The replacement was a risk-retirement spike in Rust alone, in [`spike/`](../../spike/), where any item could reopen this question. All four passed ([results](../evaluations/language-spike-results.md)).

| Item | Result |
|---|---|
| A front-matter parse that reports line and column for every key | 10 assertions pass, front matter and body |
| A scope leak that fails to compile | 6 leaks rejected, 3 positive controls compile |
| One crate built as a native Node addon and as `wasm32` | 600 KiB addon, 113 KiB wasm, identical output |
| A warm change-scoped run over 1,000 documents, under 200 ms | about 2 ms |

The scope-leak item ran first, because it carried the most weight and was the only claim that Rust might not have delivered. It delivered, and it also caught the spike's own author. The first Node wrapper tried to construct a view from outside the crate, and it did not compile.

**One result belongs in [spec 12](../spec/12-check-layer.md) rather than here.** Scope is better carried as a *type* than as a value that `scope()` returns. One trait per scope makes the declared scope and the argument type one fact, so they cannot disagree. Spec 12 records this in its own terms.
