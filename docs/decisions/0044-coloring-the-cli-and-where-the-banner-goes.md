---
id: HW-DR-0044
status: draft
status_since: 2026-08-30
summary: "Color is sensed per stream and off by default in a pipe, and turned off everywhere by `--no-color` or `NO_COLOR`. A new masthead banner prints on the root help screen alone, off by `--no-banner` or `HEADWATER_NO_BANNER`."
last_verified: 2026-08-30
title: "Coloring the CLI, and where the banner goes"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-sonnet-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0033
    - HW-DR-0042
    - HW-DR-0043
  governs:
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/src/paint.rs
    - engine/crates/cli/src/main.rs
    - engine/crates/cli/tests/help.rs
    - engine/crates/cli/tests/width.rs
---

# Coloring the CLI, and where the banner goes

## Context

**[#473](https://github.com/headwater-ai/headwater/issues/473) asks for two things.** The human-readable reports should read easier at a glance. The tool should carry a masthead. Neither ask is disputed. What #473 leaves open is the mechanism. Three questions sit under it: a palette, a rule for when color renders, and where a banner may print without breaking an existing promise. A mockup session settled each one by trial.

**No source file under `engine/crates` names a color crate, and none writes an escape sequence.** `--no-color` is declared globally already (`engine/crates/cli/src/lib.rs:222`). Its own help text states the reason:

> "Nothing here emits an escape sequence on any stream, in any format, under any terminal or for any value of `NO_COLOR`, so this flag confirms the state rather than changing it." (`NO_COLOR_TEXT`, `lib.rs:157`)

That sentence becomes false the moment any run emits color. It is rewritten here, not patched.

**`paint.rs` states a standing rule this decision has to answer, not ignore.** For `--wide`/`COLUMNS` it argues that reading the terminal is the wrong move:

> "a width that depends on the terminal makes a piped run and a run under a terminal write different bytes" (`paint.rs:12-13`)

`width_of` is built so that a run in a narrow terminal and a run piped into a file write the same bytes (`paint.rs:79-80`). That holds unless `--wide` is stated outright. A color rule that senses the terminal reopens the same divergence that comment rejects.

**Two facts distinguish color from width, and both matter to the ruling below.** First, a leaked escape sequence actively corrupts a non-terminal consumer. A pipe into `grep`, a log file, or `less` without `-R` all break. A column-wrap choice never did that: wrapped text stays readable wherever it lands. Second, every fixture this corpus currently pins runs headless. `cargo test`, the git hooks and CI all capture output through a pipe, never a terminal. So a rule that renders color only when the destination is a real terminal changes no byte of anything currently recorded. The width rule's own worry, that a recorded fixture reads whoever's terminal ran it, does not apply here.

**`HW-DR-0042` already spent the first screen's budget on one line per entry.** That bears on where a banner may go. A screen with a stated line-count ceiling, or a stated "one line" promise, cannot silently gain a line. Doing so would relitigate that ruling. `HW-DR-0043` states the doctrine a refusal already answers to: one sentence on standard error. That bears on the same question for the bare-invocation path.

## Decision

**The palette is the ANSI 8-color set, not truecolor.** A hardcoded 24-bit hex reads wrong against a light-background terminal theme. The standard SGR colors (red, yellow, blue, green, magenta, plus bold and dim or faint) are remapped by whatever theme the caller's terminal already runs. `rustc` and `cargo` already behave this way. The roles:

| Role | Rendering |
|---|---|
| `error` | red, bold |
| `warn` | yellow, bold |
| `info` | blue |
| a file path or a flag name | cyan |
| a verb name or a `fix:` label | green, bold |
| an obligation or adoption-task identifier (`OB-…`, `AD-…`) | magenta |
| a section heading | default color, bold |
| structural or already-stated text (counts, dividers, repeated context) | default color, dim |

One set of roles, reused unchanged across `headwater check`, `headwater sweep report`, `headwater explain` and the help screen. The whole binary then reads as one system, not a report-specific scheme.

**Where color cannot render, the fallback is bold and dim weight plus three glyphs, and no escape sequence at all.** The glyphs are `✗`, `▲` and `·`, for error, warn and info. `NO_COLOR`'s own convention governs only color. A bold or dim SGR attribute is not color, so this fallback stays safe even under a strict reading of `NO_COLOR`. It keeps the severity distinction visible where hue cannot carry it.

**Color is sensed per stream, and overridden globally.** Standard output and standard error each decide independently whether they are a terminal (`std::io::IsTerminal`). Each renders the palette above when it is, and the no-color fallback when it is not. `--no-color`, or a set `NO_COLOR`, forces the fallback on both streams regardless of what either senses. That matches how `--no-color` is already accepted, and until now ignored, today. There is no third state and no `--color=always`. A caller that wants color forced into a pipe has no lever here. That keeps the flag surface at one flag, not a three-way enum.

**This is a deliberate, one-time departure from the `--wide` precedent, not a quiet reopening of it.** The reasoning is the two facts in Context. An escape sequence actively harms a non-terminal reader in a way wrapped text does not. Every byte this corpus currently pins is already produced headless, so nothing recorded moves. `--wide` keeps its rule unchanged. This decision does not touch it.

**The banner is one masthead line and one rule, and it prints on the root help screen alone.**

    headwater 0.9.0 — a documentation corpus, governed and checked like code
    ────────────────────────────────────────────────────────────────────────

The masthead word renders in the `verb`/`fix:` green, and the existing tagline follows it in dim, above `Usage:`. It answers only to `headwater help`, `headwater --help` and `headwater -h` printed with no verb. It never answers to a verb's own help page, to `--version`, to the bare-invocation refusal, or to any stream a `--format`/`--json` run writes. Three reasons, each already a standing promise elsewhere. `HW-DR-0042` fixes the first screen at one line per entry and a 60-line ceiling, a promise a verb page never made. `VERSION_TEXT` promises "one line... from anywhere", a promise a banner line would break outright. `HW-DR-0043`'s refusal doctrine holds a refusal to one sentence on standard error, a promise a banner above it would also break. Extending the banner to either would need its own decision, one that rewrites the promise it breaks, not a quiet side effect of this one.

**`--no-banner` and `HEADWATER_NO_BANNER` suppress it, and both are declared global.** `--no-banner` is accepted on every verb's own page, consistent with `--no-color`'s existing shape. There it is honestly described as inert outside the root screen. A caller who writes it out of habit is answered rather than refused, the same posture `NO_COLOR_TEXT` already states for color.

**Detection happens before `clap` builds the command tree, following the precedent `paint::width()` already set for `--wide`.** `--wide` and `COLUMNS` are read by a raw pre-scan of `std::env::args_os()`, before the `Command` exists, because the help template needs the answer before parsing runs. `--no-color`, `NO_COLOR`, `--no-banner` and `HEADWATER_NO_BANNER` follow the identical shape. The decision logic itself, over the flag, the env var and the stream, is a pure function mirroring `width_of`'s exact shape. It is unit-testable with a `Case` table and no real terminal, `width.rs`'s own pattern for `--wide`/`COLUMNS`.

**Bare invocation and `--version` are unchanged in every respect but color.** The one-line refusal (`main.rs:126`) keeps its wording and its doctrine. It renders in the `error` role when standard error is a terminal, and plainly otherwise. `--version` keeps its one line, uncolored. Nothing in #473 asked for that, and `HW-DR-0042`'s "one line" reasoning would need its own relitigation first.

## Consequences

**`NO_COLOR_TEXT` (`lib.rs:157`) is rewritten, not patched**, since the sentence it states today is the opposite of the new behavior. Eighteen interface contracts under `docs/interfaces/` hand-state today's sentence in their own Options table, and each needs the same rewrite. Each also gains a `--no-banner` row. `docs/interfaces/headwater-help.md` additionally documents the banner itself, its scope, and the two flags that turn it off.

**A new `Global` entry, `NO_BANNER`, joins `GLOBALS` in `lib.rs` beside `NO_COLOR`.** `Cli` gains a `no_banner: bool` field to match. `paint.rs` gains the pure `color_of`/`banner_of`-style decision functions, the per-role paint functions, and `paint::banner()`. `first_screen()` is the only call site that inserts the banner. `finding.rs`, `check::Run::render`, `explain.rs`, `sweep::plan` and `sweep::intake` each read the per-stream color decision at render time. Each already reads nothing from the terminal today.

**`engine/crates/cli/tests/width.rs`-style case tables gain the new flag and env cases.** `interface_contract.rs` gains cases for the eighteen rewritten Options tables. Both land before the Rust that satisfies them, per the build-order amendment this same session made to `.claude/commands/next-run.md` part 1. Every existing fixture that pins exact bytes for a headless run is expected to keep passing unchanged. That is itself the check that the "every current fixture already runs headless" claim in Context is true, not assumed.

**#473's Done-when items are the acceptance bar for the pull requests this decision licenses.** A split across more than one pull request is expected. The eighteen-document sweep is mechanical, and separable from the rendering code that gives it something true to say.
