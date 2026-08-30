---
id: HW-DR-0033
status: current
status_since: 2026-08-30
summary: "`clap` derives the command line for 17 lock entries, and a flag belongs to the verb that reads it rather than to the binary."
last_verified: 2026-08-30
title: "Q33 — Whether the command line is derived, and who a flag belongs to"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  governs:
    - engine/crates/cli/Cargo.toml
    - engine/crates/cli/src/lib.rs
---

# Q33 — Whether the command line is derived, and who a flag belongs to

## Context

**The command line was parsed by hand, in one loop, and the loop knew no verbs.** `main` walked `std::env::args()` and matched each word against thirty-three arms. Every arm read a flag into a variable, and the verb was decided after the loop ended. So a flag reached the binary rather than a verb, and a flag that no verb read was accepted and dropped.

**That flat namespace was written down as a promise.** [The contract for `headwater check`](../interfaces/headwater-check.md) stated that every flag of the binary is parsed before the verb is decided. It stated that `headwater check --level L0` exits 0 and writes the report `headwater check` writes. `--level` is read by `headwater conformance` and by nothing else. [The contract for `headwater sweep`](../interfaces/headwater-sweep.md) stated the same shape over `--strict`, and [spec 12](../spec/12-check-layer.md) stated it as a structural fact of `main`. A test asserted it: `--strict` moved neither the status nor a byte of a sweep report.

**A caller cannot see the loss, which is what makes it worth a ruling.** The run succeeds. The report is the report a run with no flag writes. Nothing on either stream says that a word the caller typed reached nothing. [clig.dev](https://clig.dev/) puts this under errors, and the review that scoped [#321](https://github.com/headwater-ai/headwater/issues/321) named it there.

**One published kit answers the whole surface and is out of reach.** Charmbracelet ships Bubble Tea, Lip Gloss, Glamour, Huh, Gum and `fang`, and every one of them is written in Go. Reaching one means starting a subprocess. [Spec 6](../spec/06-engine-architecture.md#cli) states two things about this engine. It is a thin shell over a library, and no crate of it opens a socket. Both are properties of the argument rather than rules that somebody keeps, so the cost of reaching Go is the argument. The one idea worth taking from Lip Gloss is that a style is a named value asked for by role.

**The dependency graph of this engine is small enough that an addition is a decision.** Before this ruling the lock holds 35 packages: 23 crates of this workspace and 12 external ones, under 2 direct declarations. Those two are `saphyr-parser` in `crates/yaml` and `pulldown-cmark` in `crates/doc`. [HW-DR-0023](0023-the-engine-lint-floor.md) set the standard that a manifest line earns its place on the day it lands, and a third external declaration is measured against that.

## Decision

**`clap` is taken with the `derive` feature, and `clap_complete` arrives with the verb that calls it.** On the day this record landed, the manifest of `crates/cli` declared one external dependency. The section below states what `anstyle` and `clap_complete` each cost. `clap_complete` was declared on 2026-08-24 with `headwater completions`, and `anstyle` is still not declared.

**The surface is declared once, as types, in a library target of the same crate.** `engine/crates/cli/src/lib.rs` holds `Cli` and four enums, and `dispatch` in `main.rs` matches them. A second copy of the verb list is the defect [#257](https://github.com/headwater-ai/headwater/issues/257) measured, so the parser carries no verb summary and no group. Those are fields on `headwater_verbs::Verb`, and clause 5 of #321 owns them.

**The flat pre-dispatch flag namespace is withdrawn.** A flag belongs to the verb that reads it. `headwater check --level L0` exits 1 and writes no report, and `headwater sweep report <path> --strict` does the same. Two flags are global, and each for its own reason. `--root` is global because every verb reads it. `--version` is global for a different reason. It answers before any verb runs, so `headwater check --version` prints the same one line as `headwater --version`. A global flag is a declaration for one flag rather than a namespace for all of them.

**Exit 1 is preserved against `clap`'s 2.** `clap` maps every parse error except help and version onto exit 2, and `Command` exposes no setting for that number. So the parse goes through `try_parse`, and nothing lets `clap` call `exit` itself. The contract for `headwater check` lists eleven reasons for exit 1, under the sentence that there is no third status. A 2 makes that sentence false. `engine/crates/cli/tests/wiring.rs` holds the status, and it holds `--help` and `--version` at 0 with nothing on standard error.

**A refusal keeps the voice this binary already had.** `clap` renders a message, a usage block and a pointer that names no binary. [#306](https://github.com/headwater-ai/headwater/issues/306) already ruled that a refusal names where the grammar is rather than reprinting it. So the message alone is what reaches the caller. It carries the `headwater:` prefix that every other line of this stream carries, and the pointer to `headwater --help` goes under it.

## Consequences

**The measured lock, before and after.** The count is `[[package]]` entries in `engine/Cargo.lock`.

| reading | before | after |
|---|---|---|
| lock entries | 35 | 52 |
| crates of this workspace | 23 | 23 |
| external packages | 12 | 29 |
| direct external declarations | 2 | 3 |

`clap` with `derive` resolves 21 packages. Four of them are already here, because `thiserror` brings `proc-macro2`, `quote`, `syn` and `unicode-ident`, so the marginal cost is 17.

**`anstyle` is in this lock and it is not a declaration.** `clap_builder` and `anstream` both name it, so it resolves the moment `clap` does. A direct declaration of it in `crates/cli/Cargo.toml` moves the lock by zero packages. It is therefore a re-declaration of a resolved package rather than an adoption. The manifest states that, where a reader of the manifest meets it.

**`clap_complete` cost one package, and it arrived on 2026-08-24 with the verb that calls it.** Its only dependency is `clap`. A declaration with nothing calling it would have put a package in the lock that no code reaches. `unused_crate_dependencies` is not in the lint floor, so nothing would have reported it. The verb that calls it is `headwater completions`, which is clause 8 of #321. The lock goes from 52 entries to 53, and `clap_complete` is the only entry added. The estimate this paragraph carried was exact.

**What a caller reads on a mistyped command line comes from `clap`, and it is better than what it replaced.** A wrong value is named back with the flag it was written for, as in `invalid value '2026-13-45' for '--now <date>'`. A missing value names the flag and its value name. The 33 refusals the argument loop carried are gone, and one call site of `fail` carries all of them.

**What a caller reads on an unknown verb is this binary's and stays this binary's.** `headwater chekc` names every verb the dispatch table carries, because each level of the parse declares an external-subcommand form that reaches the message `headwater_verbs::listed` writes. `clap` on its own answers `unrecognized subcommand` and names at most one near miss.

**The help describes every verb, and no description is written in this parser.** `headwater --help` was 357 lines and 25,415 bytes of hand-written synopsis and prose. It is now a first screen: worked examples, the verbs under group headings with one line each, the global flags, and a closing pointer. The long form of one verb is behind `headwater help <verb>` and behind both spellings of the flag, and those three routes print one text. The group, the one-line summary and the long description are fields on `headwater_verbs::Verb`, so no copy of the verb list reaches this parser. `headwater check --help` is not a copy of the whole grammar, which is the one thing the shorter surface fixed on its own.

**Color is declared off.** The parse sets `ColorChoice::Never`, so no escape sequence reaches either stream. Clause 11 of #321 owns the question of when color is right, and `anstyle` is already resolved for whoever takes it.

**Two global flags arrived on 2026-08-24, and each one is global for a reason of its own.** This ruling says that a flag belongs to the verb that reads it, and `--root` was the one exception, because every verb reads it. `--wide` and `--no-color` are the second and the third, and neither is read by a verb at all. `--wide` is read before the parse, by the code that lays out the help the parse is about to print. `--no-color` is read by nothing and says so. A flag no verb reads has nowhere else to be declared.

**The test that read the source as text is gone, and one direction of it is now a build failure.** `engine/crates/cli/tests/verbs.rs` scraped the `["word", …]` match arms and the `USAGE` literal out of `main.rs`. It holds the command tree `clap` builds against `headwater_verbs::VERBS` instead, walked to its leaves, in both directions. The third direction needs no case at all. `dispatch` matches the parser's enum exhaustively, so a verb in the parser with no arm behind it does not compile.

**One assertion inverts, and it is the strongest evidence here.** `engine/crates/cli/tests/sweep.rs` asserted that `--strict` moves neither the status nor a byte of a sweep report. It asserts the refusal. A test that has to be reversed states a reversal more exactly than the prose that describes it.

**One paragraph above moved on 2026-08-24, and the reason it carried had ended.** The help paragraph read *The help is 25 lines and it describes nothing*, and that was true on the day this record landed. [#333](https://github.com/headwater-ai/headwater/issues/333) recovered the 23,038 bytes of description that this ruling deleted, out of the commit this record points at. Clause 5 of #321 put those words on `headwater_verbs::VERBS` rather than on the parser. Nothing else here moved: the lock, the exit statuses, and the verb that owns a flag are as this record states them.

**What this ruling does not decide.** Five things, and #321 carries all five. The layout of the help, the description of every flag, and the width the output wraps at. The machine-readable output of the verbs that lack one, and the completion scripts. Each one is a separate change over the surface this ruling settles.

**All five landed, and the dated paragraphs above and below say where.** The description of every flag and the layout of the help landed on 2026-08-24 under #333 and under clauses 4 to 6 of #321. The width landed the same day, for the help alone. The machine-readable output landed the same day, under clause 9. The completion scripts landed the same day, under clause 8.

**The width is decided in this engine and never by the terminal, and that cost the feature `clap` offers for it.** `StyledStr::wrap` is compiled out without the `wrap_help` feature. So `Command::term_width` set a number every renderer read and nothing acted on. Every help string reached a caller on one line, and the widest was 1,126 columns. Taking the feature would have pulled `terminal_size`. A width that follows the terminal makes a piped run and a run under a terminal write different bytes. So the strings are folded in `engine/crates/cli/src/paint.rs` before `clap` sees them, at 80 columns. `terminal_size` is absent from `engine/Cargo.lock`, and `clap` asks nothing about the stream it writes to. So the independence is a property of the dependency graph rather than a rule somebody keeps.

**`--wide` is the one reader of `COLUMNS`, and it is refused wherever it would do nothing.** It reads the raw command line before the tree is built. `clap` renders help inside the parse, out of strings that were folded before the parse started. The reading is held to the range 80 to 120. A run that lays nothing out refuses the flag rather than accepting it. A flag accepted and ignored is the defect [#337](https://github.com/headwater-ai/headwater/issues/337) and [#338](https://github.com/headwater-ai/headwater/issues/338) are filed about. [#340](https://github.com/headwater-ai/headwater/issues/340) landed on 2026-08-28 and carried the width of the report of `headwater check`. That report is laid out by `headwater_check::fill` at the width the flag states, so the flag is answered there. The refusal narrowed with it, and a machine format is now the case it names.

**`--json` is declared on eight verbs and on none of the others, and this ruling is what decides that.** A flag belongs to the verb that reads it. Eight command lines write a JSON document and the rest write none, so eight declarations are what the ruling permits. A global `--json` would reach `headwater new`, which writes no document a program reads. On the four verbs that already declare `--format`, the two names reach one value before the verb is entered. A command line that states both is refused with exit 1. A precedence rule would let a caller state a value and the engine substitute its own. That is the defect [#337](https://github.com/headwater-ai/headwater/issues/337) and [#338](https://github.com/headwater-ai/headwater/issues/338) are filed about.

**Four reads gained a machine form on 2026-08-24, and each one writes a document rather than a report.** The four are `route`, `explain`, `gate` and `conformance`. Each document names its own shape in a `version` member, so a consumer pins the document rather than this engine. Each is built by a function that destructures its source exhaustively. A field added to the type and not to the document therefore does not compile. The `route` document also carries the rendered report, because `.claude/hooks/intent.sh` reads the pointer set and hands an agent the text out of one run.

**The completion scripts landed on 2026-08-24, and they read the tree that parses.** `headwater completions bash|zsh|fish|powershell` writes a script on standard output and writes no file. `clap_complete` walks the command tree that `engine/crates/cli/src/lib.rs` builds. That is the tree `main` parses with, the tree `headwater help <verb>` prints out of, and the tree `engine/crates/cli/tests/verbs.rs` holds against `headwater_verbs::VERBS` in both directions. So no script carries a copy of the verb list, and a flag added to a verb reaches every shell with no edit. `clap_complete::Shell` carries a fifth name, `elvish`. The grammar block of [spec 6](../spec/06-engine-architecture.md#cli) declares four. A name in that block either runs or states its wait, so the value parser refuses a fifth with the four printed. The tree is built at the fixed width of 80 columns rather than at the width the command line asks for. A completion script is read by a shell, so no byte of it may follow a caller's terminal. `clap_complete` states a toolchain floor of 1.85, which is the floor `clap` already states, so this declaration moves no other number. [#334](https://github.com/headwater-ai/headwater/issues/334) carries the margin at that floor, and it is unchanged.

**`--no-color` is a flag that changes no byte, and its first sentence says so.** This binary writes no color on either stream, in every format, under any terminal and for any value of `NO_COLOR`. The flag is declared so that `headwater check --no-color` runs, where it exited 1 before. A caller who writes the near-universal spelling now meets an answer rather than a refusal. `engine/crates/cli/tests/width.rs` holds the absence of an escape byte in three places that do not read one another. Those are the help under four conditions, the three machine formats, and the two files the verb writes.
