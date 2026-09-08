---
id: HW-OBL-0180
status: current
status_since: 2026-09-08
summary: "A renderer takes a color mode as an ordinary argument, so the caller that hands it the plain value forever is the shape no type and no piped test can report."
last_verified: 2026-09-08
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
    - engine/crates/check/src/paint.rs
---

# A renderer's color mode is wired at a call site that no type forbids from being wrong

## Context

[HW-DR-0045](../decisions/0045-coloring-the-cli-and-where-the-banner-goes.md) rules that a stream's own terminal state decides the color, that `--no-color` and `NO_COLOR` force plain text, and that there is no third state. A caller who wants color in a pipe has no lever. That rule is what makes every artifact of this repository safe to compare byte for byte.

It also makes the color unobservable from inside the repository. Every recorded fixture, every case under `engine/`, and every step of `.github/workflows/ci.yml` reaches the engine through a pipe, and `headwater_check::paint::paint` returns its argument unchanged there. So a renderer that takes a `ColorMode` and a call site that passes `ColorMode::Plain` forever is green in all of them.

The shape is not hypothetical. From #475 and #478 until 2026-09-08, `check`, `sweep` and `explain` colored their reports and nothing asserted it. Over the same period `route`, `infer` and `generate --check` rendered zero escape sequences under a real terminal. Seventeen interface contracts said that the default senses the stream and colors only there. Three of those verbs kept a published promise, three broke it, and no run of this repository could tell the two groups apart.

`tools/color-fixtures.sh` closes one half of the gap. It attaches a pseudo-terminal with util-linux `script` and counts the escape bytes a verb's own report writes. HW-DR-0045 forbids a flag rather than a test, so the suite needs no amendment and no new dependency. It covers `route`, `generate --check`, `check`, `explain` and `sweep plan` on 2026-09-08.

## Obligation

The suite reports a surface it was told to look at. The general shape stays open: nothing forbids a wrong argument at a call site the suite does not name.

The count is 29 `.render(` sites in `engine/crates/cli/src/main.rs` on 2026-09-08. Six of them pass a mode that reads the stream. The other 23 render text that no case of this repository has ever seen under a terminal. The check runner reads the stream too, and it does not render through that method. So the two counts do not add up to the surfaces the suite covers.

The corpus owes a decision on which of two remedies it wants, and the decision needs somebody to price them.

The first remedy is a type. A mode that only a stream reader can produce moves the defect from a call site to a compile error. A renderer could not then receive one by hand. It costs a change to every renderer and to every test that calls one.

The second remedy is coverage. The verb list in `tools/color-fixtures.sh` becomes a set the engine derives, rather than five lines an author wrote. A verb that grows a human report is then covered on the day it lands. It costs a way to name every human report a verb writes, and no declaration of this repository holds one today.

Neither remedy is free and one of them has to be chosen. A third position is legitimate, and it has to be stated rather than reached by default. That position is that a person who runs the binary is what catches a color defect.

## Discharge

A decision record that names the remedy, and the build that carries it.

For the first remedy, the discharge is a type that a call site cannot supply by hand. Beside it goes the case that shows a hand-written mode failing to compile.

For the second, the discharge is a run of `tools/color-fixtures.sh` whose verb list comes from a declaration. Beside it goes the case that shows a new human report entering the list with no edit to the suite.

For the third, the discharge is the decision record alone, and this obligation closes against it.
