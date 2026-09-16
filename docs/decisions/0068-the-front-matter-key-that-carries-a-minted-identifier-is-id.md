---
id: HW-DR-0068
status: current
status_since: 2026-09-16
summary: "The front-matter key `id` is the fixed, global name for the value a kind's `identifier: {scheme: …}` facet mints."
last_verified: 2026-09-16
title: "The front-matter key that carries a minted identifier is id"
relations:
  governs:
    - engine/crates/graph/src/lib.rs
    - engine/crates/graph/src/index.rs
  traces_to:
    - HW-OBL-0053
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  evidence_basis: evidenced
---

# The front-matter key that carries a minted identifier is id

## Context

A kind declares `identifier: {scheme: …}`. The meta-schema's `kind_identifier` block closes that declaration to one member, `scheme`. Nothing in the schema language states which front-matter key holds the value a scheme mints.

The typing pass wrote `id` anyway, because every governed document in this corpus needs one. `headwater_graph::Config` carries the key as a parameter, `identifier_facet`, defaulted to `"id"`. That way the index and the generated identifier check read the same key. `.headwater/README.md`'s guess table carries the row, and [HW-OBL-0053](../obligations/0053-the-front-matter-key-that-carries-an-identifier-is-undeclared.md) carries the debt.

[Q2](../spec/09-decisions.md#q2--schema-format) already rules that Headwater owns its own schema language. So this record is not blocked on that ruling. It is blocked on someone stating the declaration inside the language Q2 settled.

[Q4](0004-relation-storage.md) met the same shape of gap for `relations:`. It closed the gap the same way: a fixed literal key, stated once for every kind, and never a per-kind parameter. A relation instance and a minted identifier are both values that every kind's front matter carries in a predictable place. An author, or a check, would otherwise look up the key per kind before it reads a document. That lookup is the cost the fixed key removes.

## Decision

The front-matter key `id` carries the value that a kind's `identifier: {scheme: …}` facet mints, as a global fixed rule. No kind overrides it. The meta-schema's `kind_identifier` block stays closed to `scheme` alone. It gains no second member for the key name, because every kind uses the one key name.

## Consequences

**`headwater_graph::Config`'s `identifier_facet` default stops describing a guess.** The doc comment on `Config`, and the matching comment in `index.rs`, cite this record instead of naming the key an unsettled parameter. The field itself stays a parameter, for the same reason `relations_facet` stayed a parameter after Q4. The module reads a name rather than a literal. A future change to the declaration then changes one default, and not every call site.

**`.headwater/README.md`'s guess table loses the row for this gap.** The table lists what the typing pass still had to guess, and this row named a settled question.

**[HW-OBL-0053](../obligations/0053-the-front-matter-key-that-carries-an-identifier-is-undeclared.md) discharges.** The obligation was to state the declaration, and this record is that statement. Its entry leaves [13 — Open obligations](../spec/13-open-obligations.md) to match.

**Every existing document keeps validating.** Every `kinds.*.identifier` declaration in the base package and the overlay already uses only `{scheme: …}`. Every governed document already writes `id`. The decision states what the corpus already does. It changes no file under `docs/` beyond the ones this record names.
