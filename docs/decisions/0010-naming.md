---
id: HW-DR-0010
title: Q10 — Naming
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: The name is Headwater, capitalized in prose and lower case as an identifier.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: unevidenced
---

# Q10 — Naming

## Context

The name was a working name through the design phase, and the namespace that every emitted artifact carries depends on it. No evaluation was run for this decision, and none was needed: what settled it was the availability of a domain and the separation of two assets that the entry had treated as one.

## Decision

The name is **Headwater**. It is no longer a working name, and the design phase does not reopen it.

The casing has two forms, and they do not mix. In prose, the name of the system is *Headwater*, capitalized. As an identifier, it is `headwater` in lower case. It names the CLI verb that people type dozens of times a day, and the package name. It also names the `.headwater/` directory, the `headwater:` annotation prefix, and the `https://w3id.org/headwater/` namespace.

## Consequences

**The namespace and the web address are separate assets, from 2026-08-11.** The namespace is `https://w3id.org/headwater/`. The W3C Permanent Identifier Community Group runs w3id.org, and it redirects a permanent identifier to whatever location the project hosts today. The web address is an ordinary domain. A change of domain therefore reaches no emitted artifact.

This decision first fixed the namespace at `https://headwater.dev/`, and an unrelated consultancy holds that domain. The separation is the better arrangement in any case. A namespace URI outlives every release, and a domain that the project rents by the year is a weak vehicle for that promise. [Q11](0011-license-and-distribution-posture.md) reads the namespace as the instrument that the trademark reservation protects, and the path `w3id.org/headwater` carries that reading unchanged.
