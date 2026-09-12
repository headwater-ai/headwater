---
id: HW-OBL-0018
title: "The bundle set is a guess about how adopters cluster"
status: current
status_since: 2026-08-11
waiting_on: adopter
last_verified: 2026-08-13
summary: "Q3 ships optional content as bundles, and the set of them is a guess that no adopter has revised."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0003
---

# The bundle set is a guess about how adopters cluster

## Context

[Q3](../spec/09-decisions.md#q3--how-much-of-the-default-taxonomy-ships-in-the-box) rules that the base package is minimal and that optional content ships as add-only bundles. The set of bundles is a guess about how adopters cluster, and no adopter exists yet.

## Obligation

The corpus owes a bundle set that a real adopter has met. It is data in a package, so the first real adopter revises it at the cost of a release.

## Discharge

The [design-spec taxonomy](../taxonomies/design-spec/doctrine.md#findings) supplies a first data point. It adds the `narrative` voice regime at the address that the sketched `proposals` bundle also adds, and an adopter that selects both meets the collision.

The [`standards-spec` entry](../taxonomies/standards-spec/doctrine.md#findings) supplies a second data point, at [line 164](https://github.com/headwater-ai/headwater/blob/5fb9518/docs/taxonomies/standards-spec/doctrine.md#L164) of its doctrine. The sketch names `standards`, and the entry is `standards-spec`, which is named for a whole ladder rather than for one shelf. The library index says that every admitted entry is a revision of this guess, so the sketch and the library must not drift.

The [`brd-prd` entry](../taxonomies/brd-prd/doctrine.md#findings) supplies a third data point, at [line 173](https://github.com/headwater-ai/headwater/blob/5fb9518/docs/taxonomies/brd-prd/doctrine.md#L173). The sketch lists `procedure`, `standards`, `evidence`, `proposals`, `operations` and `compliance`. None of them is a requirements cluster. `proposals` is the nearest, and a requirement is accepted work rather than a proposal.

This item is data or a deferred component, so the first real adopter is the evidence rather than a further argument.
