---
id: HW-OBL-0180
status: current
status_since: 2026-09-08
summary: "A caller that passes the plain color mode forever is a defect that no type and no piped test can report."
last_verified: 2026-09-11
title: "A renderer's color mode is wired at a call site that no type forbids from being wrong"
waiting_on: build
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - engine/crates/paint/src/lib.rs
---

# A renderer's color mode is wired at a call site that no type forbids from being wrong

## Context

[HW-DR-0045](../decisions/0045-coloring-the-cli-and-where-the-banner-goes.md) rules that a stream's own terminal state decides the color, that `--no-color` and `NO_COLOR` force plain text, and that there is no third state. A caller who wants color in a pipe has no lever. That rule is what makes every artifact of this repository safe to compare byte for byte.

It also makes the color unobservable from inside the repository. Every recorded fixture, every case under `engine/`, and every step of `.github/workflows/ci.yml` reaches the engine through a pipe, and `paint` returns its argument unchanged there. So a renderer that takes a `ColorMode` and a call site that passes `ColorMode::Plain` forever is green in all of them. That function was `headwater_check::paint::paint` until #479 and it is `headwater_paint::paint` after it, re-exported under the old path.

The shape is not hypothetical. From #475 and #478 until 2026-09-08, `check`, `sweep` and `explain` colored their reports and nothing asserted it. Over the same period `route`, `infer` and `generate --check` rendered zero escape sequences under a real terminal. Seventeen interface contracts said that the default senses the stream and colors only there. Three of those verbs kept a published promise, three broke it, and no run of this repository could tell the two groups apart.

`tools/engine/color-fixtures.sh` closes one half of the gap. It attaches a pseudo-terminal with util-linux `script` and counts the escape bytes a verb's own report writes. HW-DR-0045 forbids a flag rather than a test, so the suite needs no amendment and no new dependency. It covers `route`, `generate --check`, `check`, `explain` and `sweep plan` on 2026-09-08.

## Obligation

The suite reports a surface it was told to look at. The general shape stays open: nothing forbids a wrong argument at a call site the suite does not name.

The count is 29 `.render(` sites in `engine/crates/cli/src/main.rs` on 2026-09-08. Six of them pass a mode that reads the stream. The other 23 render text that no case of this repository has ever seen under a terminal. The check runner reads the stream too, and it does not render through that method. So the two counts do not add up to the surfaces the suite covers.

On 2026-09-11 the count is thirty occurrences of `.render(` in that one file, and the thirty is not thirty report surfaces. Twelve of them are not a report on standard output at all. They are two prints to standard error, two digest inputs, two file writes and two date fields. The last four are one JSON value, one refusal message, one appended store line and one cache statistic. Of the eighteen that remain, nine passed a mode that reads the stream at `d6ac7c93` and ten do after the `derived` build of #479. The other eight still print a report that no case of this repository has seen under a terminal.

The two figures this paragraph replaces said twenty-nine and seventeen. Both were one low, because `headwater_compat::payload::account(…).render()` prints to standard output and was counted as something else. Read thirty and eighteen. `tools/engine/color-fixtures.sh` names fifteen surfaces, from fourteen.

**The eight that remain are not eight equal pieces of work, and nothing recorded that until the dependency graph was read.** `headwater-check` depends on `headwater-census`, `headwater-resolve` and `headwater-lock`. While the palette was a module of `headwater-check`, a renderer inside any of those three could not name it: cargo refuses the cycle before a line compiles. Six of the eleven command lines that still broke the published promise render in exactly those crates — `derived` in `headwater-census`, and `taxonomy validate`, `resolve`, `publish`, `vendor` and `migrate` in `headwater-resolve` and `headwater-lock`.

So the surfaces were taken cheapest first, and each iteration left a remainder more expensive than the one before it. #791 took the last two that needed no argument, no file write and no dependency move. #479 moved the primitives into `headwater-paint`, a leaf crate that depends on nothing, which makes the other five reachable for the first time. What is left is now uniform in cost.

Each change of this shape still wires one more call site by hand. So this record is no nearer discharge for the wiring, and the decision it asks for is what closes it.

The corpus owes a decision on which of two remedies it wants, and the decision needs somebody to price them.

The first remedy is a type. A mode that only a stream reader can produce moves the defect from a call site to a compile error. A renderer could not then receive one by hand. It costs a change to every renderer and to every test that calls one.

The second remedy is coverage. The verb list in `tools/engine/color-fixtures.sh` becomes a set the engine derives, rather than five lines an author wrote. A verb that grows a human report is then covered on the day it lands. It costs a way to name every human report a verb writes, and no declaration of this repository holds one today.

Neither remedy is free and one of them has to be chosen. A third position is legitimate, and it has to be stated rather than reached by default. That position is that a person who runs the binary is what catches a color defect.

## Discharge

A decision record that names the remedy, and the build that carries it.

For the first remedy, the discharge is a type that a call site cannot supply by hand. Beside it goes the case that shows a hand-written mode failing to compile.

For the second, the discharge is a run of `tools/engine/color-fixtures.sh` whose verb list comes from a declaration. Beside it goes the case that shows a new human report entering the list with no edit to the suite.

For the third, the discharge is the decision record alone, and this obligation closes against it.
