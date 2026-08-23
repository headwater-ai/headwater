---
id: HW-DR-0033
status: draft
status_since: 2026-08-23
summary: "`clap` derives the command line for 17 lock entries, and a flag belongs to the verb that reads it rather than to the binary."
last_verified: 2026-08-24
title: "Q33 — Whether the command line is derived, and who a flag belongs to"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
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

**`clap` is taken with the `derive` feature, and nothing else is taken with it.** The manifest of `crates/cli` declares one external dependency. `anstyle` and `clap_complete` are not declared, and the section below states what each one costs.

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

**`clap_complete` costs one package and is absent from this lock.** Its only dependency is `clap`. A declaration with nothing calling it would put a package in the lock that no code reaches. `unused_crate_dependencies` is not in the lint floor, so nothing would report it. It arrives with the verb that calls it, which is clause 8 of #321.

**What a caller reads on a mistyped command line comes from `clap`, and it is better than what it replaced.** A wrong value is named back with the flag it was written for, as in `invalid value '2026-13-45' for '--now <date>'`. A missing value names the flag and its value name. The 33 refusals the argument loop carried are gone, and one call site of `fail` carries all of them.

**What a caller reads on an unknown verb is this binary's and stays this binary's.** `headwater chekc` names every verb the dispatch table carries, because each level of the parse declares an external-subcommand form that reaches the message `headwater_verbs::listed` writes. `clap` on its own answers `unrecognized subcommand` and names at most one near miss.

**The help describes every verb, and no description is written in this parser.** `headwater --help` was 357 lines and 25,415 bytes of hand-written synopsis and prose. It is now a first screen: worked examples, the verbs under group headings with one line each, the global flags, and a closing pointer. The long form of one verb is behind `headwater help <verb>` and behind both spellings of the flag, and those three routes print one text. The group, the one-line summary and the long description are fields on `headwater_verbs::Verb`, so no copy of the verb list reaches this parser. `headwater check --help` is not a copy of the whole grammar, which is the one thing the shorter surface fixed on its own.

**Color is declared off.** The parse sets `ColorChoice::Never`, so no escape sequence reaches either stream. Clause 11 of #321 owns the question of when color is right, and `anstyle` is already resolved for whoever takes it.

**The test that read the source as text is gone, and one direction of it is now a build failure.** `engine/crates/cli/tests/verbs.rs` scraped the `["word", …]` match arms and the `USAGE` literal out of `main.rs`. It holds the command tree `clap` builds against `headwater_verbs::VERBS` instead, walked to its leaves, in both directions. The third direction needs no case at all. `dispatch` matches the parser's enum exhaustively, so a verb in the parser with no arm behind it does not compile.

**One assertion inverts, and it is the strongest evidence here.** `engine/crates/cli/tests/sweep.rs` asserted that `--strict` moves neither the status nor a byte of a sweep report. It asserts the refusal. A test that has to be reversed states a reversal more exactly than the prose that describes it.

**One paragraph above moved on 2026-08-24, and the reason it carried had ended.** The help paragraph read *The help is 25 lines and it describes nothing*, and that was true on the day this record landed. [#333](https://github.com/headwater-ai/headwater/issues/333) recovered the 23,038 bytes of description that this ruling deleted, out of the commit this record points at. Clause 5 of #321 put those words on `headwater_verbs::VERBS` rather than on the parser. Nothing else here moved: the lock, the exit statuses, and the verb that owns a flag are as this record states them.

**What this ruling does not decide.** Five things, and #321 carries all five. The layout of the help, the description of every flag, and the width the output wraps at. The machine-readable output of the verbs that lack one, and the completion scripts. Each one is a separate change over the surface this ruling settles.
