---
id: HW-OBL-0158
status: draft
status_since: 2026-08-28
summary: "The fifth entry of the GLOBALS table carries clap's own two words rather than a description this repository wrote, because clap adds its help argument after the point where a caller can name it."
last_verified: 2026-08-28
title: "clap owns the help flag, so -h, --help reads Print help on all 32 verb pages"
waiting_on: build
---

# clap owns the help flag, so -h, --help reads Print help on all 32 verb pages

## Context

[HW-DR-0042](../decisions/0042-q42-what-one-screen-means-for-the-first-help-screen.md) put a `GLOBALS` table in `engine/crates/cli/src/lib.rs`. Each entry holds the one-line summary the first screen prints and the whole description a verb page prints. Four of the five entries carry a description this repository wrote. The entry for `-h, --help` carries the two words `Print help`, which is what `clap` supplies.

Two routes fail in `clap` 4.6.6. `Command::mut_arg` panics on an argument it cannot find. The build adds the help argument, rather than the declaration, so a call before the build finds nothing. `Command::mut_args` documents that it does not affect the built-in help or version argument at all.

The route that remains is `disable_help_flag`, with a help argument declared in the `Cli` struct beside the four flags that are already there. That argument takes `ArgAction::Help` and `global = true`. The change that found this gap left the route alone for one reason. It moves the ordering of the flag block on all 32 verb pages. Clause 4 of #342 asks that the byte identity of the three routes hold.

## Obligation

A reader of a verb page meets five global flags, and one of them speaks in a different voice from the other four. The corpus owes that reader one of two things. Either a description of `-h, --help` written the way the other four are written, or a ruling that `clap` owns the string.

`engine/crates/cli/tests/help.rs` holds every argument of every command to carrying help text at all, so nothing here is undescribed. The gap is the voice and not the presence.

## Discharge

The `Cli` struct declares the help argument, `disable_help_flag` turns off the one `clap` adds, and `HELP.description` reaches all 32 verb pages. `the_long_description_of_a_verb_is_reachable_three_ways` must stay green, and the flag block of a verb page must stay inside 80 columns.

The alternative discharge is a ruling that `clap` owns this one string. In that case the entry names `clap` as the author, and this record closes on the ruling rather than on a build.
