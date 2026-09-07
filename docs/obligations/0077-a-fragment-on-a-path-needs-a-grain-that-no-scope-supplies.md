---
id: HW-OBL-0077
title: "A fragment on a path needs a grain that no scope supplies"
status: discharged
status_since: 2026-09-08
waiting_on: build
last_verified: 2026-09-08
summary: "`link.fragment.unresolved` read a bare fragment and never one on a path. It reads both at corpus grain now, and the grain it was waiting for was one this engine already had."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0004
---

# A fragment on a path needs a grain that no scope supplies

## Context

A fragment with no path names a heading of the document that wrote it, and `link.fragment.unresolved` reads it at document grain. A fragment on a path names a heading of another document, and no [scope](../spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on) carries one. `Document` carries one document. `Edge` carries a relation instance, and [Q4](../spec/09-decisions.md#q4--relation-storage) rules that a prose link is not a relation. `Neighbourhood` carries the documents one relation away, and it carries the identity of each rather than the body.

## Obligation

The grain that reaches this is one document and the documents that its prose links reach, or so this record said. **The premise was wrong, and the error is worth keeping.** A fifth grain was never needed. `Corpus` reaches the question. `link.path.unresolved` was already there, for the same reference class, before this record was written.

The unreached half was the larger one by an order of magnitude. Every count of it goes stale with the next prose edit, so the ratio is what to carry. About nine in ten of this corpus's fragment-bearing prose links point at another document, and that was the share that reached no rule. This record has stated three different absolute pairs, and each was true on the day it was measured and false a week later.

## Discharge

**This record is discharged.** `link.fragment.unresolved` is a `CorpusCheck` at version 3. It reads the links the graph build bound. A same-document fragment resolves against the citing document's anchors, and a fragment on a resolving path against the target's. A finding names the citing document and the line either way. The anchors come from `fragment::anchors`, which is the one slug rule this engine has.

**The grain question answered itself, and the reason is in the code rather than in an argument.** Handing a document-scoped rule the target documents as a declared input is unsound. `Instance::paths()` is the read set, and the coverage account routes an instance to every path in it. They are one list. Soundness demands that every target document is in the read set, or a rename in the target leaves a cached pass standing. Each one there would then be counted as a document this rule checked.

**What this cost the coverage account, measured rather than assumed.** The 312 document-routed instances of this rule became 1. A classified document whose only routed check was this one would become a `coverage.document_unchecked` finding. That does not happen on this corpus, because `facet.required.missing` reports one instance against every classified document and skips none of them. **That is a fact about this corpus and not a property of the engine.** A thin shelf in an adopting corpus, whose kind requires no facet, could lose its only routed check to this move. [HW-OBL-0071](0071-a-corpus-scoped-check-makes-the-coverage-rule-unreachable.md) holds that shape, and this change makes it one instance larger.

**The gap was priced before it was closed.** A script that resolved every fragment on a path by hand found two that reached no heading, and both had stood on `main` unreported. A third was caught before it landed, by running that script rather than by a check. The MCP write tools added a section to [spec 5](../spec/05-ai-integration.md#what-the-working-tree-write-class-registers-and-what-a-session-looks-like-after-a-write). A record then cited it under a shorter slug than the heading makes. `headwater check` reported nothing, and the run was green with the link dead. That script is what the rule replaces, and no adopter ever had it.
