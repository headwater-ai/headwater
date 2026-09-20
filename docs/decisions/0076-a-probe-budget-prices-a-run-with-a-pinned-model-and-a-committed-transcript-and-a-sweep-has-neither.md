---
id: HW-DR-0076
status: current
status_since: 2026-09-20
summary: "Rules that a budget prices a run identity a probe has and a sweep does not, names the sweep outside every tier, and states where .headwater/probe.yml lives and why."
last_verified: 2026-09-20
title: "A probe budget prices a run with a pinned model and a committed transcript, and a sweep has neither"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: measure+draft
  evidence_basis: evidenced
---

# A probe budget prices a run with a pinned model and a committed transcript, and a sweep has neither

## Context

[#87](https://github.com/headwater-ai/headwater/issues/87) asks one question of [spec 5](../spec/05-ai-integration.md#the-envelope-is-declared-and-the-harness-fails-closed): does a declared budget price every mechanism that reaches a model, or only a run that carries a run identity, a pinned model, and a committed transcript? [#165](https://github.com/headwater-ai/headwater/issues/165) (closed) shipped the two tiers a budget prices today, regression and campaign, declared in `.headwater/probe.yml`. `headwater probe plan` fixes six members of a run identity before the run starts (the lock, the tree, the selection, the read set, the seed, and the harness version), projects the session count against the tier's ceiling, and refuses a run above it.

The [coherence sweep](../spec/04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep) also reaches a model, and it shipped after the budget mechanism did. `headwater sweep plan` writes a briefing, an agent reads the corpus and writes a return file, and `headwater sweep report` reads that file back. No step of that round trip fixes a run identity, pins a model, or commits a transcript. So the engine can neither project a sweep's cost before it runs nor account for one after, and nothing in spec 5 or spec 4 said whether that gap was a defect or the design.

[#515](https://github.com/headwater-ai/headwater/issues/515) (closed) ruled the retention window for a probe transcript and left the same question open for a sweep's return file: whichever way this record rules, that file's retention follows from it.

## Decision

**A budget prices a run that has a run identity, a pinned model, and a committed transcript. It does not price every mechanism that happens to reach a model.** "The engine invokes it" is not the basis, because a probe's own recorder is outside this repository too, and the budget still prices the probe run around it. The basis is what the run leaves behind: a fixed run identity a later comparison can cite, a model version the run pinned rather than asked for, and a transcript this corpus commits and can reread.

A probe run has all three. A sweep has none of them. `headwater sweep plan` and `headwater sweep report` name no run identity between the two calls, pin no model, and commit no transcript, because [spec 4](../spec/04-assurance-model.md#the-output-is-a-proposal-and-the-engine-confirms-the-half-of-it-that-is-checkable) already routes the sweep's model call through whatever harness the calling agent runs in. **A sweep is outside the budget mechanism by construction, and `.headwater/probe.yml` declares no tier for it.** `headwater sweep plan` takes no budget parameter and refuses no run on cost.

**`.headwater/probe.yml` stays where it is, outside the corpus root and outside the taxonomy, and this record is where a reader of spec 5 now finds out why.** A budget is a policy of the repository that runs the probes, and not a fact about any document. No taxonomy rule reads it, no language regime binds it, and no shelf classifies it, which is the same test [#74](https://github.com/headwater-ai/headwater/issues/74) applied to the capture-cost store and answered the same way. The file sits beside the lock and beside the cache for that reason, and it does not move.

## Consequences

[Spec 5](../spec/05-ai-integration.md#the-envelope-is-declared-and-the-harness-fails-closed) states the basis beside the paragraph that already describes `headwater probe plan`. That paragraph projects a session count against a tier's budget. The new text names the sweep as outside, and it carries the reasoning for where `.headwater/probe.yml` lives.

[Spec 4's sweep section](../spec/04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep) states plainly that no budget prices a sweep. It also states that a sweep's return file is retained by nothing. The retention question [#515](https://github.com/headwater-ai/headwater/issues/515) held open for this file closes the same way this record rules: a file that no budget prices is a file no evidence obligation reaches.

`engine/crates/probe/src/budget.rs`'s test module carries a regression case: a `sweep:` tier in the declared file is refused as `Unreadable::UnknownTier`, the same as any other undeclared tier name. The case passes today with no code change. It exists so that a later change cannot add a sweep tier without reopening this ruling.

The module comment of `engine/crates/probe/src/budget.rs` and the header of `.headwater/probe.yml` both cite `[#87]`. Each names it as the owner of "the two tiers and their cadence" and of "it may move this file." The first clause is already stale, since the two tiers landed in #165. Both comments now cite this record instead of the closed issue. The second clause is corrected too: this record is the one that rules the file stays.

[HW-OBL-0092](../obligations/0092-how-a-probe-reaches-a-harvesting-tier-and-what-a-transcript.md) named three open threads: the route from a probe to a tier, the retention policy for transcripts, and who pays for a campaign. Two are now answered by work that landed after that record's last verification. The route closed in #165. Retention closed in #515, documented at [spec 15](../spec/15-the-recorder-contract.md#how-long-a-result-stays-citable-and-why-that-period-is-not-a-number-this-schema-produces). The campaign question was already answered by the record's own Discharge section, which pointed to Q11. That record closes in the same change as this one.
