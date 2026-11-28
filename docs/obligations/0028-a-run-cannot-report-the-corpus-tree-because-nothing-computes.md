---
id: HW-OBL-0028
title: "A run cannot report the corpus tree, because nothing computes one"
status: discharged
status_since: 2026-11-29
waiting_on: build
last_verified: 2026-11-29
summary: "Checks and cache asked every run for a corpus tree beside its findings, and the engine computed none. Every format of a check report now writes a tree over every census row."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-engine-architecture
---

# A run cannot report the corpus tree, because nothing computes one

## Context

[Checks and cache](../subsystems/checks-and-cache.md#what-a-run-states-beside-its-findings) asks every run to report three things beside its findings: the corpus tree, the taxonomy lock hash, and the read set. [The assurance model](../spec/assurance-model.md#a-verdict-is-about-one-state-of-the-corpus) states the same contract: a run reports the state that it evaluated. Until this record was discharged, the engine reported the lock hash and the read set, and no format wrote a tree.

The read set is not a tree, because it holds what the checks read and a tree holds what the census walked.

One value had the name before this record was discharged. `headwater_probe::plan::tree_digest` digests the path and the content digest of each **typed** row, and a probe plan, a shadow-log line and `headwater route --json` carry it. It does not move when an untyped, unreadable, excluded or non-document file arrives, and recorded probe results carry its values as provenance. So it is not the tree that this record asks for, and it stays as it is.

## Obligation

Without a tree, a run states nothing about a file that no check opened, which is the exact set a merge can add. The corpus owes a tree that covers every row of the census, and every format of the check report owes that tree.

## Discharge

**`Census::tree_digest` and the four emitters discharge this record ([#1491](https://github.com/headwater-ai/headwater/issues/1491), slice HW-OBL-0028).** The census computes one digest over every row, in path order. Each row gives its path, its class and the digest of its bytes, or `-` where the census read no bytes. `render_at` in the adapter takes the tree from the census that it renders, so no caller can hand a format a tree of some other census. The text report writes a `tree` heading after `clock`. The JSON report writes a top-level `tree` member, and its shape moved to `1.10`. The Markdown report names the tree in its first sentence, beside the lock. The SARIF report writes it in the run's property bag at `properties.headwater.tree`, and its loss entry for the corpus tree now names that place.

**The residual is stated where a reader meets it.** A row that read no bytes, such as a source file or an excluded file, gives its path and its class only. So an edit to the bytes of such a file does not move the tree, because the census did not read those bytes. The paths outside the corpus root that a language regime lists are not rows, and they stay out. The rustdoc of `Census::tree_digest` states both.

**SARIF does not carry the tree in `run.automationDetails.id`.** The schema defines that member as the unique identity of a run, and two runs over one tree would carry one value. GitHub code scanning also derives the category of an analysis from it, and the shipped action passes no category of its own. So a value that moves on every commit could split the alert history of an adopter. The module comment of `engine/crates/adapter/src/sarif.rs` names the condition that reopens this choice.

**The check layer asks for no tree, and this record stood on Checks and cache alone.** [The check layer](../spec/check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) fixes the gate as a comparison over listed inputs, which needs neither a tree nor a run. So the test that decides whether a verdict survives a merge did not wait on this record, and this discharge changes no gate rule. [HW-OBL-0102](0102-two-documents-state-what-voids-a-verdict-and-they-do-not.md) carries that ruling and the construction under it.

**The MCP server was the first consumer that could answer about a tree that is gone.** It walks the corpus once, before it accepts a message, so a session outlives the walk behind it. A `check` call over a server started before an edit was measured at 28 findings where a fresh run reported 30, and the two answers of that server were identical across the edit. [#1880](https://github.com/headwater-ai/headwater/issues/1880) changed the `check` tool before this record was discharged. It now runs `headwater check --no-cache` over the tree as it is now, and not over the walk of the server. Its `markdown` and `sarif` answers are the bytes of that run, so they carry the tree of that run. Its `text` and `json` answers are a summary of totals and of the findings on changed files, and they do not carry the tree.

**A server that writes cannot make that mistake, because it ends instead.** The working-tree write class arrived on 2026-08-14, and a call that moves a byte of the checkout spends the server ([AI integration](../spec/ai-integration.md#what-the-working-tree-write-class-registers-and-what-a-session-looks-like-after-a-write)). Every later call is refused rather than answered from the walk the write made stale. One British spelling was planted in a copy of this tree, which then carried 28 findings. One `fix` call over the protocol wrote the patch and answered 27. The next call was refused. Under the reading above the second answer would have been 28.
