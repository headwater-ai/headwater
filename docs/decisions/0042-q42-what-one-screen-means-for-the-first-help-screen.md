---
id: HW-DR-0042
status: current
status_since: 2026-08-30
summary: "\"One screen\" is retired as a claim about a terminal and replaced by a claim about content: the first screen carries one line per entry, and the ceiling that follows from it is 60 lines."
last_verified: 2026-08-30
title: "Q42 — What \"one screen\" means for the first help screen"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0033
  governs:
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/tests/help.rs
---

# Q42 — What "one screen" means for the first help screen

## Context

**[#342](https://github.com/headwater-ai/headwater/issues/342) reports that the first screen is three screens.** `headwater --help` printed 76 lines from an empty directory, at 80 columns, on standard output alone. 25 of them were the descriptions of five global flags, under one heading. `--wide` took nine lines of that and `--no-color` took six. The eighteen verb entries took one line each, which is what [#321](https://github.com/headwater-ai/headwater/issues/321) clause 4 asks of them.

**Clause 4 of #321 asks that the screen fit one screen, and no layout at 80 columns ever could.** One line per verb costs 24 lines for eighteen verbs. Six group headings and five blank separators cost eleven more. So 35 lines are spent before one flag or one example is printed. A terminal of 24 lines was never reachable, and #342 measures two layout mutations that confirm it.

**The lever is what the screen says rather than how it is set.** #342 accounts for the growth of the screen. The fold that [#321](https://github.com/headwater-ai/headwater/issues/321) clause 12 landed added twelve lines, and the two flags that clause 12 declares added fifteen. The fold bought a widest line of 80 columns where the widest was 229. The fifteen are reference material, and they sit in the position a table of contents should hold.

**#342 puts two answers and asks for a ruling before any code.** One answer shortens what the first screen says about a global flag. The other rules that "one screen" is not the bar and writes down what is.

## Decision

**"One screen" is retired as a claim about a terminal. It is replaced by a claim about content.**

> The first screen carries one line per entry. One line per verb, one line per global flag, a two-line worked example, a name line, a usage line and a closing pointer. Nothing on the first screen is a paragraph. A reader of the first screen is choosing a verb, and the prose that explains a flag belongs on the page of the verb that will use it.

**The bar is testable in two forms, and this engine holds both.** The bar itself is the entry: every global flag occupies exactly one line of the first screen. The consequence is the height: the first screen is at most 60 lines. `every_global_flag_is_one_line_on_the_first_screen` in `engine/crates/cli/tests/help.rs` holds the first, and `the_first_screen_holds_the_height_the_ruling_names` holds the second.

**The entry is the bar and the height is only its consequence, because the entry is what catches a flag added later.** A global flag with no entry in the table fails the case on the argument identifier that `clap` carries. An entry whose summary is too long to fit the column folds onto a second line and fails the case on the count. A height alone would pass both.

**The second answer of #342 is refused.** A ruling that stated no height bar would leave the screen where it stands and change nothing a reader meets. It is cheaper by one small table. The 25 lines it would leave in place are 33 percent of the first screen. An adopter reads that screen before any other output of this engine.

**The long form of every global flag stays where it already is.** All four declared global flags are `global = true`, so `clap` prints each description in full on all 32 verb pages. `headwater check --help`, `headwater check -h` and `headwater help check` print one text, and this ruling does not touch it.

## Consequences

**The measured screen is 56 lines and the ceiling is 60.** The block of five global flags went from 25 lines to five, so the screen went from 76 lines to 56. The widest line is 79 columns. `COLUMNS=120 headwater --wide --help` is 55 lines at a widest of 95. The four lines of headroom absorb a sixth verb group of two verbs, or four more verbs in the groups that exist. It is a ceiling and not a pin. It goes red when the screen stops being a list, and not when a verb arrives.

**The summary is declared beside the description, at the flag.** `GLOBALS` in `engine/crates/cli/src/lib.rs` carries one entry per global flag, and each entry holds the `clap` identifier, the name, the one-line summary and the whole description. [HW-DR-0033](0033-q33-whether-the-command-line-is-derived-and-who-a-flag-belongs-to.md) rules that a flag belongs to the verb that reads it. It rules that the description of a flag is written at the declaration of that flag. A global flag is declared in that file, so its summary is declared there too. It is not declared on `headwater_verbs::Verb`, which is where the same pair is declared for a verb.

**`Arg::long_help` is refused, and the reason is clause 4 of #342.** `clap` renders `long_help` for `--help` and the short help for `-h`. Introducing one would make the two spellings of a flag print different text, which is the byte identity that #342 asks to keep. It also splits along the wrong axis. The axis that matters is the first screen against a verb page, and not the long spelling against the short one.

**The first screen renders its global flags by hand, the way it already renders its verbs.** `first_screen` dropped the `{options}` tag, which is the only reason those descriptions were on the root screen at all. The block it prints instead comes off `GLOBALS` through `paint::row`, which is the helper the verb rows a few lines above already use. No `clap` argument changes, so the risk of the change is one template string. The column is derived from the longest name rather than written down. `paint::row` overruns the width when the column is narrower than the name it is given.

**This record refines one sentence of HW-DR-0033 and contradicts none of it.** That record describes the first screen as "worked examples, the verbs under group headings with one line each, the global flags, and a closing pointer". The phrase "the global flags" is the part that was unpriced, and this record prices it at one line each.

**Nothing here rules on the length of a verb page.** The page of `headwater check` is 94 lines. A reader reaches a verb page after choosing a verb, and not while choosing one. #342 states that boundary and this record keeps it.
