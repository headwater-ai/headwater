---
id: DR-repo-0008
title: Q8 — Probe cost and cadence
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: A probe is a document with a declared expectation, and cadence follows the purpose of the run.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - EVAL-HW-the-measurement-layer
---

# Q8 — Probe cost and cadence

## Context

One [evaluation](../evaluations/the-measurement-layer.md) settles this with [Q20](0020-where-scent-lives.md), because Q20's open half is a grading problem and the grader is a probe.

**The open-question entry asks about cadence, and the instrument does not exist.** [Spec 5](../spec/05-ai-integration.md#measuring-whether-any-of-this-works) named six probe categories, two constraints and a cost envelope. It never said what a probe **is**. There was no declaration, no authored form, no owner, and no definition of the declared expectation that the grader compares against. So the rule that the grader is never the system under test had nothing to bind. That finding outranks the question that the entry asked.

**The audit found more.** Twenty claims across the specification name an instrument. Six are charged to the probe layer, and three of those six are misfiled. Two ask the grader to read the agent's prose, and one is `generate --check` under another name. Fourteen belong to `taxonomy audit`, the coverage report, fixture sets, or human adjudication, and every one of them is equally unbuilt. Two name instruments that cannot exist as written, and each stays with the record that made it.

## Decision

A probe is a document with a declared expectation. It has a kind, a shelf, an identifier and an acceptance. Its expectation is a predicate over the run record, from a closed set: `opened`, `not_opened`, `cited`, `answered`, `patched`. A question that needs a rubric is not a probe, and the [coherence sweep](../spec/04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep) owns those. The grader then holds no model, because a predicate over an event log needs none. That is the shape that [Q7](0007-scope-of-the-mcp-surface.md) used for the write path, where a guarantee is a code path that does not exist.

## Consequences

**A self-report and a produced output are different things**, and the specification never separated them. A self-report is the agent's account of its own process, and no probe accepts one. A produced output is the artifact that the task asked for, and an expectation may read it. Without the distinction, Sufficiency has no instrument, and two records wrote probes that cannot run.

**Two categories are gone.** Fidelity compares a projection with its source, which `generate --check` proves with no model. The counterfactual is an **arm** of every probe rather than a seventh category. To call it a category hid that it applies to the others, and hid that the pair doubles the cost.

**Cadence follows the purpose of the run, not the category.** A regression tier runs a fixed set, one arm, against a recorded baseline, on a schedule, and weekly is a sound default. A campaign runs both arms, powered, for one named claim, as one batch at one model version. Categories are orthogonal to both. The leaning's "scheduled weekly" is right for the first tier and meaningless for the second.

**No probe runs against a proposed change, and a change still does something.** The network is closed at check time, and no LLM sits in the validation path. A verdict that a rerun may reverse is also not what a gate needs. What a change does instead is free. A probe result is a verdict about one state of the corpus, so it carries a [read set](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict). The run reports which recorded results the change voided. That is [Q21](0021-terminological-succession-and-validity-under-merge.md)'s machinery, and it is what makes a trend over quarters readable at all.

**Storage: the leaning is right and it is one artifact short.** A run emits a transcript, which the corpus commits as a snapshot that a `probe_run` anchor resolver reads. The result is a document generated from that snapshot, so its [warrant](../spec/01-conceptual-model.md#warrant) is `regenerated`. Commit only the result and its warrant is `asserted`, and [spec 3](../spec/03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two) then refuses it as evidence for anything. A probe run is an external system of record, so [Q19](0019-inbound-integration-an-external-system-of-record.md) already built the machinery and the declaration count stays at thirteen.

**Drift and defect are separated, as [principle 3](../spec/00-vision-and-scope.md#design-principles) requires.** The grading is deterministic, so a result that disagrees with its own transcript is a defect, and the grader joins the [correctness roots](../spec/12-check-layer.md#the-correctness-roots). The behavior is not deterministic, so a rerun that returns a different rate is variance or drift, and only an interval separates them. A model name is not a pin. Two snapshots of one named model moved from 84% to 51% on a task in three months.

**The envelope is derived, and money is not the constraint.** A campaign's session count comes from statistical power, which puts it at one hundred to three hundred sessions for a detectable effect. Against a corpus of this size that is a low-hundreds-of-dollars line item, and a weekly regression tier is a small monthly bill. What binds is the authoring of scenarios and expectations, and the correctness of the grader. The envelope stays as a declared budget per tier, and the harness fails closed rather than overspends.
