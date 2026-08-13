---
id: OBL-repo-0095
title: "Whether the RDF emitter carries `owl:inverseOf`"
status: current
status_since: 2026-08-12
last_verified: 2026-08-13
summary: "An inverse manufactures 332 edges no author declared, which leaves `taxonomy audit` with no baseline for `created_by`."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - DR-repo-0013
---

# Whether the RDF emitter carries `owl:inverseOf`

## Context

[Q13](../spec/09-decisions.md#q13--linkml-and-shacl-as-substrate) rules that emitters never chain and stages six of them. Whether the RDF emitter carries `owl:inverseOf` is open. The inverse infers the reciprocal half of every edge that `reciprocal: required` exists to report as missing.

## Obligation

Over this corpus the inverse manufactures 332 edges that no author declared, which leaves `taxonomy audit` with no baseline for `created_by`. Emitter 4 owns the choice, and it owes a ruling.

## Discharge

The [worked example](../evaluations/owl-skos-worked-example.md) argues to omit the inverse. That emitter met a second obstacle here, because 22 of this repository's 36 documents carried no discriminator on a heterogeneous shelf. The [typing pass](../spec/13-open-obligations.md#what-the-first-typing-of-this-corpus-found) removed it, so an instance export now reads each kind, identifier and edge rather than derives it.
