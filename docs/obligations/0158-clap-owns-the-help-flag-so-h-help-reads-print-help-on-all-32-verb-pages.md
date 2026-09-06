---
id: HW-OBL-0158
status: current
status_since: 2026-09-06
summary: "The fifth entry of the GLOBALS table carries clap's own two words rather than a description this repository wrote, because clap adds its help argument after the point where a caller can name it."
last_verified: 2026-08-28
title: "clap owns the help flag, so -h, --help reads Print help on all 32 verb pages"
waiting_on: build
---

# clap owns the help flag, so -h, --help reads Print help on all 32 verb pages

## Context

[HW-DR-0042](../decisions/0042-q42-what-one-screen-means-for-the-first-help-screen.md) put a `GLOBALS` table in `engine/crates/cli/src/lib.rs`. Each entry holds the one-line summary the first screen prints and the whole description a verb page prints. Four of the five entries carry a description this repository wrote. The entry for `-h, --help` carries the two words `Print help`, which is what `clap` supplies.

`Command::mut_arg` panics on an argument declared before `build()` runs. The build step is what adds the help argument, so a call before it finds nothing to mutate. Called after `build()`, on the root command and on each verb's own subcommand, `mut_arg("help", …)` finds the argument and sets its text. `Command::mut_args` documents that it does not touch the built-in help or version argument, but the single-argument call does, once the argument exists to find.

A standalone reproduction confirms the post-build call renders `HELP.description` on the root screen and on a verb page. Both land in the same position in the flag block, without disturbing argument parsing. Threading that call through `command_at` itself is unproven. `command_at` sets `help_template` and loops `mut_subcommand` over all eighteen top-level verbs before `paint::painted` runs, and the reproduction did not exercise that path.

`disable_help_flag` with a declared help argument is a second candidate route. It needs no post-build ordering, and it costs a fifth argument in the `Cli` struct beside the four already there. The change that found this gap made no ruling between the two routes.

## Obligation

A reader of a verb page meets five global flags, and one of them speaks in a different voice from the other four. The corpus owes that reader one of two things. Either a description of `-h, --help` written the way the other four are written, or a ruling that `clap` owns the string.

`engine/crates/cli/tests/help.rs` holds every argument of every command to carrying help text at all, so nothing here is undescribed. The gap is the voice and not the presence.

## Discharge

Either route discharges this record if `HELP.description` reaches every verb page and `the_long_description_of_a_verb_is_reachable_three_ways` stays green. The count in the title of this record is the count on the day it was written. `headwater json` and its three second words made it 36 on 2026-09-06, and no check reads either number. `headwater_verbs::VERBS` is what a reader counts. The flag block of a verb page must stay inside 80 columns under either route.

The alternative discharge is a ruling that `clap` owns this one string. In that case the entry names `clap` as the author, and this record closes on the ruling rather than on a build.
