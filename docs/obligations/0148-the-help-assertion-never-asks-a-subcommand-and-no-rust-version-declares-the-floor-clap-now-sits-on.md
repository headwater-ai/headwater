---
id: HW-OBL-0148
status: current
status_since: 2026-09-06
last_verified: 2026-09-07
title: "The help assertion never asks a subcommand, and no rust-version declares the floor clap now sits on"
summary: "The help test pins only the root command, and no machine-readable rust-version stops clap's own margin from reaching zero."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
waiting_on: adopter
---

# The help assertion never asks a subcommand, and no rust-version declares the floor clap now sits on

## Context

Verification of PR #332 surfaced two gaps in the engine. PR #332 adopted clap for the command-line parser, and neither gap traces to that change or blocks it. The first sits in `engine/crates/cli/tests/wiring.rs`, in the case that asserts `--help` output. The second sits in `engine/Cargo.toml`, where no crate declares the Rust version the engine now needs.

## Obligation

The case `the_help_flag_answers_on_standard_output_outside_a_corpus` loops over `--help` and `-h` against the bare binary alone. It asserts exit code, empty standard error, the `--root <path>` marker, and every verb name from `headwater_verbs::VERBS`. It never asks a subcommand. `headwater check --help` follows a different code path, introduced by the clap migration, and nothing in the suite pins it. Measured on the migration head, `headwater --help` prints 406 bytes and `headwater check --help` prints 537, both on standard output with exit 0. A regression that broke only the per-verb help path would leave the whole suite green.

The second gap is closed, and the margin this document called zero was already spent on the day it was written. `ordered-float 5.5.0` entered `engine/Cargo.lock` transitively under `saphyr` on 2026-08-26. It declares a floor of 1.90. The two container commands in `engine/README.md` pinned an image at 1.85, so both exited 101 before they compiled a line. They stayed dead for 12 days and 111 commits. This repository went public on 2026-09-06 with both of them dead, and a change on that day edited both without running either.

`[workspace.package]` in `engine/Cargo.toml` now declares `rust-version = "1.90"`, and each of the 23 crates under `engine/crates/` declares `rust-version.workspace = true`. `engine/clippy.toml` states the same number as its `msrv`. `tools/engine/engine-readme-fixtures.sh` compares the highest `rust-version` across `cargo metadata --locked` against the image tag those two commands pin. CI runs that suite, and it needs no container and no build.

The `last_verified` field of this document was not evidence about the margin the paragraph above called zero. That field moves when the register is regenerated, not when a person or a check confirms the claim. It read 2026-08-26 on the day the margin was spent, which is the same date as the commit that spent it. So this entry looked freshly verified on the day it became wrong.

No check reads this document for the floor it states. `tools/engine/engine-readme-fixtures.sh` enumerates floor statements by four shapes. It reads an image tag, a sentence that names a Rust version, an `msrv` key, and a `rust-version` key. The prose here matches none of the four, so the entry that predicted this failure sits outside that population. Widening the shapes to reach it would catch every correct historical mention of a version. That is why the population stops where it does.

## Discharge

Closing the first gap means deriving the help test's cases from `headwater_verbs::VERBS`, not writing them by hand. A hand-written list beside `VERBS` is the same defect issues #257 and #309 both named. The new case must ask every verb for `--help` and `-h`, not the bare binary alone. It must assert exit 0 and empty standard error for each verb. The case should be watched failing first, so a broken per-verb help path names the verb that broke it.

The second gap needs nothing further. `cargo build` and `cargo test` succeed on current stable and inside the pinned image, and `cargo clippy --all-targets --locked -- -D warnings` reports nothing at the raised `msrv`.

This document closes issue #334, recorded rather than planned. The first fix is not written, and this document stands as the debt until the corpus carries it.
