---
id: HW-DR-0080
status: current
status_since: 2026-09-21
summary: "The corpus-staleness dashboard (#505) ships as a free, self-hosted companion tool, with no hosted form now. Its data model carries a corpus identity on every row from day one, so a later hosted form is an addition rather than a rewrite."
last_verified: 2026-09-21
title: "Q66 — the corpus dashboard ships free and self-hosted, with tenant isolation stated as a data-model constraint from day one"
relations:
  traces_to:
    - HW-DR-0011
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
---

# Q66 — the corpus dashboard ships free and self-hosted, with tenant isolation stated as a data-model constraint from day one

## Context

[#505](https://github.com/headwater-ai/headwater/issues/505) asks for a read-only web page over `headwater export`, `explain`, `coverage` and `conformance` output. It offers a staleness view, a warrant view and a coverage view, fed by the `json` target of a declared `graph_export` profile. It names this ruling as a precondition, and it asks for the ruling before design work starts. This way the choice happens once, and nobody discovers it midway through an implementation.

[Q11](0011-license-and-distribution-posture.md) settles Apache-2.0. It records that a hosted server and a hosted probe harness are the only two places in the specification where a commercial tier could sit. It does not decide whether either is built. [HW-OBL-0094](../obligations/0094-where-a-commercial-tier-could-sit-and-how-thin-the-trademark.md) carries that placement question at `waiting_on: ruling`, and it stays there. This record narrows one instance of the question, the dashboard, and it does not answer where a commercial tier sits in general. [HW-OBL-0026](../obligations/0026-the-operational-shape-of-a-hosted-server-is-unstated.md) records that the operational shape of any hosted server is unstated. This record does not answer that either. It constrains only the dashboard's own data model, for the day somebody proposes a hosted form of the dashboard.

The closest observed analog already draws a commercial line in roughly this place. [HW-EVAL-adjacent-work §S.3](../evaluations/adjacent-work.md#s3-where-the-closest-analog-draws-the-commercial-line) records that Vale ships its command-line checker free. Vale's author sells a hosted style-guide platform and a hosted rules layer beside it. The checker is not the product. The hosted layer that a person cannot easily self-host is the product. A dashboard with staleness, warrant and coverage views, and possibly team rollups later, is the same shape: a hosted analytics surface beside a free checker.

## Decision

The corpus-staleness dashboard ships as a free, self-hosted companion tool. No hosted form ships now, and #505 does not build one.

The dashboard's data model carries a corpus identity as a named column on every row from day one. A self-hosted, single-corpus deployment never needs to filter by that column, but the column is there anyway. Concretely:

- Every row the dashboard stores or renders is keyed by `(corpus_identity, document_or_anchor_identifier)`, never by `document_or_anchor_identifier` alone. This holds for a document's staleness entry, its warrant entry, and a `code_path` anchor's coverage entry. A self-hosted instance holds exactly one `corpus_identity` value and never displays the column. The column still exists in the schema and in every query, from the first commit.
- No store the dashboard writes is shared mutable state keyed by anything less than that pair. A cache, an index or a materialized view can read more than one `.headwater/export.json` at a time. Such a store partitions by `corpus_identity` before any other operation touches it. Two rows with different, correctly allocated `corpus_identity` values never fold together under one key.
- The dashboard's read path treats each input file as exactly one corpus's data, whether it loads `.headwater/export.json` or a later `graph_export` target. A self-hosted deployment assumes one dashboard instance, one corpus, one export file. A hosted deployment must not fall back on that assumption silently. It must plumb `corpus_identity` through the ingestion boundary on purpose, because nothing in the schema forced the question earlier.

This is a constraint on the dashboard's own data model. It is not a design for a hosted server. It does not say who would run one, how it would authenticate a customer, or how it would bill. Those questions stay open at [HW-OBL-0026](../obligations/0026-the-operational-shape-of-a-hosted-server-is-unstated.md) and at [HW-OBL-0094](../obligations/0094-where-a-commercial-tier-could-sit-and-how-thin-the-trademark.md).

**The keying scheme is necessary and not sufficient.** Collision-freedom holds only where every `corpus_identity` value is itself allocated distinctly per tenant. This record names the column and its place in every key. It does not name who allocates a `corpus_identity` value, or how a hosted server would keep two customers from colliding on one. That allocation mechanism belongs to the operational shape of a hosted server, which [HW-OBL-0026](../obligations/0026-the-operational-shape-of-a-hosted-server-is-unstated.md) still owes. A later hosted form must not treat this record's schema constraint as a substitute for that missing piece.

## Consequences

**#505 may proceed.** Its data model follows this ruling. It is a self-hosted, single-corpus tool. `corpus_identity` is present in the schema and runs through every row from the first commit, and the free tool's own interface never shows it.

**A later hosted form is additive.** Every row already carries a corpus identity, so adding a second corpus's data to the same store needs no migration that touches every existing row. The isolation column is already there, already populated, and already part of every key. A hosted form still has to add authentication, per-customer provisioning, and billing. The operational shape of the server itself is out of scope here, and it stays with [HW-OBL-0026](../obligations/0026-the-operational-shape-of-a-hosted-server-is-unstated.md).

**HW-OBL-0094 gains a second citation and stays open.** This record narrows one instance of "where a commercial tier could sit," the dashboard, and it does not settle the question in general. [HW-OBL-0094](../obligations/0094-where-a-commercial-tier-could-sit-and-how-thin-the-trademark.md)'s `waiting_on: ruling` is unchanged by this decision.
