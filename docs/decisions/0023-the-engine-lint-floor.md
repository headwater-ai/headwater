---
id: HW-DR-0023
status: current
status_since: 2026-08-15
summary: A published Rust guideline set is not installed as a skill, because a skill carries no rule of its own. Twenty-two lints are declared once in the workspace manifest, and each was chosen by measuring the corpus rather than by adopting a list.
last_verified: 2026-08-15
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  governs:
    - engine/Cargo.toml
title: "Q23 — The engine lint floor"
---

# Q23 — The engine lint floor

## Context

Two published sets of Rust guidance were offered for installation as agent-facing skills: [actionbook/rust-skills](https://github.com/actionbook/rust-skills), which is thirty-one skills over three layers with hooks and background agents, and [the Microsoft pragmatic Rust guidelines](https://microsoft.github.io/rust-guidelines/agents/all.txt), which is about fifteen thousand words of identified rules with a rationale under each one. Both name an agent as the reader, and the second states that the rules make an API easier for a model to use.

**This repository already answers where a rule lives, and the answer is not a skill.** [Spec 5](../spec/05-ai-integration.md#how-a-skill-reaches-an-agent-and-what-nothing-does) states that a skill carries no rule of its own. Every mechanical statement in one is a call to a verb that ships. It also states that nothing makes a skill load, so the reach of a description is a measurement that no probe has taken. A guideline set installed as a skill would therefore be a rule with two defects at once. Nothing enforces it, and nothing says whether the agent read it.

**The holding mechanism refuses it for a second reason.** `.claude/skills/fixtures.sh` drives every claim a skill makes against the engine, as a blocking job. A skill that describes an engine that moved under it produces a confident wrong answer. A skill of Rust guidance names no verb, so nothing could hold it. It would be the only skill here that decays without a signal.

**The scale is wrong as well.** The seven skills of this repository are 430 lines together. The smaller of the two candidates is thirty times that, and the larger half of its content is aimed at a published library. The engine sets `publish = false`. That leaves the guidance on semantic versioning, on foreign function interfaces, on macros, and on documentation for a downstream consumer. It answers a question this workspace does not have.

**The gap the two sets point at is real, and it was in the manifest.** `engine/Cargo.toml` declared no lints and no crate declared any, so `cargo clippy --all-targets -- -D warnings` in the continuous integration job ran the default set alone. The universal-practices section of the Microsoft guidelines names exactly this, and it is the one part of either set that this repository can hold.

## Decision

**A lint is declared once in `[workspace.lints]`, and every crate inherits it with `[lints] workspace = true`.** Twenty-two entries are declared for twenty-two crates. The alternative is the same rule copied into each manifest. That is the second-copy failure that [stop rule 4](../spec/05-ai-integration.md#the-stop-rules) names for prose and that a manifest does not escape.

**The set is chosen by measurement, and never by adoption.** Each entry was at zero hits on the run that added it, or its hits were corrected in the same change. The reason is mechanical rather than stylistic: the continuous integration job reads `-D warnings`, so a lint named at any level in this file fails the build, and a level of `warn` here is a level of `deny` there.

**A lint that reports correct code is left out rather than allowed at the site.** An `#[allow]` beside a correct line records that the rule is wrong in a place where nobody compares it against the rule. A named absence in the manifest, with the reason next to it, is read by whoever considers the same lint next.

**No skill of this repository carries Rust guidance.** The judgment that no lint reads stays in `engine/README.md` and in the `headwater-engine` skill, which carry the invocation, the toolchain floor and the mistakes that cost a session. Neither restates a rule that the manifest holds.

## Consequences

**What the measurement found, on 2026-08-15.** A probe that enabled `clippy::pedantic`, the panic family and `missing_docs` reported 6449 warnings over 49 lint names, of which `missing_docs` was 2486, `clippy::must_use_candidate` 1064 and `clippy::expect_used` 972. A second probe of twenty-nine candidates reported 400. The twenty-two that shipped report zero.

**The engine has no `unsafe` block, and `forbid` is what records it.** Zero occurrences were counted under `engine/crates/*/src`. `forbid` rather than `deny` because the property worth keeping is that no module can re-admit the construct with a local attribute.

**The corrections the floor required.** `cargo clippy --fix` wrote about forty mechanical changes across seventeen files, for `explicit_iter_loop`, `stable_sort_primitive`, `semicolon_if_nothing_returned`, `uninlined_format_args` and their neighbors. Nine `match` expressions became `let ... else` by hand, in `census`, `probe`, `conformance` and the CLI. The workspace test suite passes on the result.

**Five lints were measured and left out, and each names its reason in the manifest.** `unwrap_in_result` reports four `read` helpers inside `#[cfg(test)]` modules, where an `expect` on a fixture is the intent. The job reads tests because it passes `--all-targets`. `verbose_file_reads` reports one site in `crates/scaffold/src/tree.rs` that keeps the handle it read from in order to write through it. `float_cmp` reports one `assert_eq!` against `0.0` that is exact because the numerator is zero. `trivial_casts` reports one cast in `crates/check/src/register.rs` that an array of closures needs in order to reach a single type. `assigning_clones` reports ten sites whose correction saves an allocation that nothing here counts.

**A lint set at zero does no work today, which is the point.** Every entry is a regression that the build refuses rather than a defect the change repaired. The value is readable only when somebody writes the twenty-third crate.

**What no lint reads is unchanged and unwritten.** Panic semantics at a boundary, `#[non_exhaustive]` on an error type and the documentation of a fallible function are guidance that both candidate sets carry and that no rule here holds. `missing_docs` at 2486 and `clippy::missing_errors_doc` at 166 measure the distance, and no task claims it.
