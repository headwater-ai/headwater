---
id: HW-OBL-0148
status: current
status_since: 2026-09-06
last_verified: 2026-08-26
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

No crate in `engine/` declares `rust-version`. The floor is stated in `engine/README.md` and pinned in the container recipe's `rust:1.85-slim` image, but CI runs `rustup default stable` and never reads either. `engine/crates/cli/Cargo.toml` pins clap 4.6.6. Both `clap` and `clap_builder` at that version declare `rust-version = "1.85"`, exactly the stated floor. The margin for a routine `cargo update` to raise that floor further, unnoticed, is now zero.

## Discharge

Closing the first gap means deriving the help test's cases from `headwater_verbs::VERBS`, not writing them by hand. A hand-written list beside `VERBS` is the same defect issues #257 and #309 both named. The new case must ask every verb for `--help` and `-h`, not the bare binary alone. It must assert exit 0 and empty standard error for each verb. The case should be watched failing first, so a broken per-verb help path names the verb that broke it.

Closing the second gap means declaring `rust-version = "1.85"` under `[workspace.package]` in `engine/Cargo.toml`. The comment should name it as the floor `engine/README.md` already states. It should also note that clap sits exactly on that floor today. Both `cargo build` and `cargo test` must still succeed unchanged on current stable and inside `rust:1.85-slim`.

Neither fix is written yet. This document closes issue #334, recorded rather than planned, and stands as the debt until the corpus carries both fixes.
