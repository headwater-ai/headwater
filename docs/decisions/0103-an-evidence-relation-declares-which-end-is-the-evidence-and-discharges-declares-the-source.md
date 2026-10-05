---
id: HW-DR-0103
status: current
status_since: 2026-10-01
summary: "A relation of the evidence family declares evidence_at, from or to, because the family cannot say which end substantiates the other. Absent is to. discharges declares from, so an evaluation is the evidence for its obligation."
last_verified: 2026-10-01
title: "103 — An evidence relation declares which end is the evidence, and discharges declares the source"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-taxonomy-model
    - HW-SPEC-conceptual-model
---

# 103 — An evidence relation declares which end is the evidence, and discharges declares the source

## Context

The rule `warrant.evidence.unsupported` holds spec 3's sentence: "A pointer to a document with the `asserted` warrant does not support `evidenced`." It reads every relation of the `evidence` family in one direction. The source makes the claim, and the target is the evidence. So it reads the `evidence_basis` of the source and the warrant of the target.

Most evidence relations point that way. `traces_to`, `cites_evidence`, `verified_by` and `proven_by` all point from a claim to what it rests on. `discharges` points the other way. The `decision-record` bundle declares it from an `evaluation` to an `obligation_record`, and the evaluation substantiates the obligation. [Spec 1](../spec/01-conceptual-model.md) states the direction: "An asserted document does not discharge an evidence obligation." So the evaluation is the evidence.

The rule therefore read a `discharges` edge backwards. An `evidenced` evaluation that discharged an `asserted` obligation was reported, because the rule read the obligation's warrant as support for the evaluation's own claim. An `asserted` evaluation that discharged an `evidenced` obligation passed, and that is the failure spec 1 names. #1384 met the first case and left its `discharges` edges out of an evaluation for that reason. #1511 is the report.

The rule cannot find the direction by reading the family. [Spec 2](../spec/02-taxonomy-model.md#nuclearity) gives the reason for nuclearity. It says: "One thing that the family cannot supply is *which end* is the nucleus, because relation direction is the choice of the taxonomy author." The same is true of which end substantiates. The rule also cannot name `discharges` in a list of exceptions, because it reads the family and never one relation name. A taxonomy that renames its relations must read the same.

## Decision

1. A relation can declare `evidence_at`, with the value `from` or `to`. The value names the end that is the evidence. The other end makes the claim. The member is in meta-schema 0.9.0 beside `nucleus`, and it is optional.
2. Absent is `to`. That is the reading every evidence relation had before the member existed. So a source that validated under 0.8.0 validates and reads the same under 0.9.0.
3. `warrant.evidence.unsupported` reads the `evidence_basis` of the claimant and the warrant of the evidence, at the ends the relation declares. The finding still anchors at the half an author declared, whichever end is the claimant.
4. `relations.discharges` in the `decision-record` bundle declares `evidence_at: from`. headwater/standard 4.15.0 publishes it.

The member means something only on a relation of the `evidence` family. No rule refuses it on another family yet, and no reader reads it there.

## Consequences

An evaluation that declares `discharges` onto an obligation is the evidence for that obligation. An `asserted` evaluation that discharges an `evidenced` obligation is reported, and the message names the obligation as the claimant. An `evidenced` evaluation that discharges an `asserted` obligation is not reported. `engine/crates/check/tests/evidence_basis.rs` holds both cases.

The count of findings on this corpus does not move, because no document in it declares `discharges` today. The rule's version moves from 3 to 4, so a warm cache does not serve the old reading of a `discharges` pair.

Three other relations of this repository's lock also point from the evidence to the subject: `assesses`, `examines` and `records`. None of them raises a finding on this corpus today. This decision does not declare `evidence_at` on them. For `examines`, it is not clear that either end makes a claim that the other end supports. Each one is a separate question, and the run's intake carries it.
