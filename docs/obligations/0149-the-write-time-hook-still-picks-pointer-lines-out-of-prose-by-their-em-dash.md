---
id: HW-OBL-0149
status: current
status_since: 2026-09-06
last_verified: 2026-08-26
title: "The write-time hook still picks pointer lines out of prose by their em dash"
summary: "The write-time hook still finds pointer lines by grepping for an em dash, so a document with no summary drops out in silence."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
waiting_on: adopter
---

# The write-time hook still picks pointer lines out of prose by their em dash

## Context

[#321](https://github.com/headwater-ai/headwater/issues/321)'s clause 10 stopped the intent-time hook from reading routing output as prose, and named only `.claude/hooks/intent.sh`. The write-time hook, `.claude/hooks/write.sh` at the `PostToolUse` position, kept its old reading, and [#344](https://github.com/headwater-ai/headwater/issues/344) carries it forward.

## Obligation

`.claude/hooks/write.sh` decides what to show an author by grepping the rendered text of `headwater route`, rather than by reading a structured answer. One grep, `' — '`, selects pointer lines by their em dash, a character `Pointer::render` writes only when the document declares a summary. A governed document with no `scent` facet renders no em dash and drops out of the advisory in silence. A second grep, `'names the anchor'`, decides the route was anchored rather than ranked by matching one sentence of one renderer. This couples the hook to prose written for a person rather than for a machine. `engine/crates/query/src/route.rs` carries a comment written for exactly this reader. It describes a withheld pointer line that carries no em dash, because the hook selects pointers by grepping for one. `headwater route --json` now answers both questions as data. `anchors` and `pointers` are written on every run, empty where there is nothing. `text` carries the same rendering byte for byte, already read by `intent.sh`. The write-time hook does not read it.

## Discharge

What closes this is `.claude/hooks/write.sh` reading `headwater route --json` at the `PostToolUse` position, and deciding from `anchors` and `pointers` instead. The file then holds no `grep ' — '` and no `grep 'names the anchor'`. A fixture case, built on the stand-in engine `stub_route` already provides, shows a document with no summary reaching the advisory. The position keeps failing open on the same four counts it fails open on today. Those are no engine, an engine that will not execute, a payload that will not parse, and a document that will not parse. [HW-DR-0055](../decisions/0055-a-hook-reads-a-wire-format-through-the-engine-and-not-through-an-interpreter.md) moved the third of the four off an interpreter and onto the engine, and the count is the same. The comment in `engine/crates/query/src/route.rs`, describing the withheld line's missing em dash, goes or is rewritten, because the constraint it records is gone. Nothing else pays this debt, and `waiting_on: adopter` records that the corpus waits on an adopter to write it.
