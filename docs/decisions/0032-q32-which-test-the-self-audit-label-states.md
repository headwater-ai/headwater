---
id: HW-DR-0032
status: current
status_since: 2026-08-30
summary: The `self-audit` label states the reader test and not the provenance test. How a finding was found is not the test, and the only question is whether a reader outside this repository is better off.
last_verified: 2026-08-30
title: "Q32 — Which test the self-audit label states"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
---

# Q32 — Which test the self-audit label states

## Context

The build order of this repository states one value rule, in `.claude/commands/next-run.md`. Before any work starts, somebody names the reader who is not this repository. Where the only party better off is this corpus, the work is not eligible for an iteration.

**Two labels on the issue tracker carry that rule.** `adopter-blocking` marks work an outside adopter cannot proceed without. `self-audit` marks the other side of it, and the label sorts work out of every selection rule the build order reads.

**The description of `self-audit` named two tests at once.** It read as work found by running the engine over this repository with no reader outside it. The first clause is a test of provenance, which is how the finding arrived. The second is a test of the reader, which is who is better off when the work is done. The two partition the board differently, and nothing said which one decides.

**The conflation costs work in both directions.** A defect found by running the engine over this corpus can block an adopter, and the provenance test hides it. A defect found by reading an issue from outside can serve nobody but this corpus, and the provenance test admits it.

## Decision

**`self-audit` states the reader test.** The label description reads *No reader outside this repository. How it was found is not the test.*

**The reader test is the one the value rule already states.** The rule asks for the reader who is not this repository, and it asks for nothing about the instrument that produced the finding. A label that carried the provenance test would sort the board by a fact the rule does not read.

**Provenance is a poor test on its own terms.** Almost every finding in this repository arrives from running the engine over this repository, because that is the only corpus there is. A test that nearly every candidate passes separates nothing.

**The label marks a wait and never a fault.** `self-audit` says that the work has no reader outside this repository today, so it stands open and something else runs first. It says nothing about whether the work is right.

## Consequences

**A finding found by the engine over this corpus is not `self-audit` when an adopter needs it.** The label comes off, and the work competes on the same footing as anything else. `adopter-blocking` is the label that then applies, and it sorts above everything in the selection rules.

**A finding found anywhere else is `self-audit` when no outside reader exists.** The origin of the report does not exempt it.

**The two labels stay the product owner's.** `.claude/agents/headwater-product-owner.md` writes both, and no other agent applies either one. This record settles which test one of them states, and it settles nothing about who applies it.

**The value rule keeps its single statement.** `.claude/commands/next-run.md` holds the canonical wording, and every other file cites it. This record adds no second copy of the rule and reads the label description against it.
