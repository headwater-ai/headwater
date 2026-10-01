---
id: HW-OBL-0230
status: current
status_since: 2026-10-01
summary: "Eight comments in cli, paint, generate, probe, sweep and the command-surface subsystem spec state facts the code no longer holds, found by the builds and verifies of run 20261001-1107."
last_verified: 2026-10-01
title: "Comments in the command-surface crates and their subsystem spec state eight stale facts"
waiting_on: build
---

# Comments in the command-surface crates and their subsystem spec state eight stale facts

## Context

The builds and verifies of #1288 slices (d) and (e) in run `20261001-1107` found eight comments that state facts the code no longer holds. Each was checked on `d309e914`. None has a reader outside this repository, because each is a comment that a contributor reads, and #1563 already carries six other stale crate comments at its clause cap. This is the third spec 13 record of the run, so it lists the rest together, as `hw-run-policy` asks.

## Obligation

- The module doc of `engine/crates/generate/src/verb_index.rs` says that `headwater_verbs::VERBS` is the dispatch table of the binary. The doc heading of `dispatch` in `engine/crates/cli/src/main.rs` says the same, and the body under it says that the `clap` tree now decides. `docs/subsystems/command-surface.md` states that the array does not decide which verb runs.
- The module doc of `engine/crates/cli/src/lib.rs` says "Color is declared off. Nothing here emits an escape sequence". `command_in` now sets `paint::color_choice(mode)` and `paint::help_styles(mode)`.
- `banner` in `engine/crates/cli/src/paint.rs` writes the tagline as a literal and does not read `headwater_verbs::TAGLINE`. The two strings match today, and nothing holds them together.
- The `Verb::Other` comment in `engine/crates/cli/src/lib.rs` and the doc of `dispatch` in `main.rs` say that `clap` names at most one near miss for an unknown subcommand. clap 4.6.6 with the `suggestions` feature lists every candidate above a Jaro score of 0.7.
- The module doc of `engine/crates/paint/src/lib.rs` and the module doc of `engine/crates/check/src/paint.rs` say that `headwater-check` depends on `headwater-lock`. `engine/crates/check/Cargo.toml` names `headwater-lock` only under `[dev-dependencies]`, so the cycle argument holds for census and resolve and not for lock.
- The comments in `engine/crates/probe/Cargo.toml` and `engine/crates/sweep/Cargo.toml` give the reasons each crate names `headwater-check`. They omit `headwater_check::paint` in both, and reciprocity and `Shape::descends_from` in sweep.
- `docs/subsystems/command-surface.md`, section *The parse, and the words a caller reads*, says that the comments on the derive types are `//` comments. The `Shell` enum in `engine/crates/cli/src/lib.rs` carries `///` comments, which `clap` ignores on a value enum.
- `docs/subsystems/command-surface.md` calls `VERBS` an array. It is a slice.

## Discharge

Each item discharges alone, when the comment or the sentence states what the code does, or when the code changes to match it. For the tagline, a test or a shared constant holds the two together. The record discharges when every item has.
