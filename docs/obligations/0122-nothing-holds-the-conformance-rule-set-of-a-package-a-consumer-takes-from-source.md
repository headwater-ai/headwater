---
id: HW-OBL-0122
status: current
status_since: 2026-08-14
waiting_on: ruling
summary: "A conformance rule set is held by the release digest and never by the lock, so a repository that takes its package from source can edit the rules its own gate reads."
last_verified: 2026-08-14
title: "Nothing holds the conformance rule set of a package a consumer takes from source"
provenance:
  warrant: proposed
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-distribution-and-federation
---

# Nothing holds the conformance rule set of a package a consumer takes from source

## Context

Every other verdict this engine reaches is held by a committed artifact. `headwater check` reads `.headwater/taxonomy.lock` and never the taxonomy sources. So an unresolved edit changes nothing that any check sees, and `taxonomy resolve --check` reports the stale lock in continuous integration.

A conformance rule set is not a taxonomy source. It declares no kind, facet, shelf or check, so it never reaches the lock and `taxonomy resolve` never reads it. What holds it instead is the release digest, which covers every file of a published artifact including this one. `headwater taxonomy vendor` refuses an artifact whose bytes moved, so a vendored rule set is held as tightly as a vendored taxonomy.

**The hold is the pin, and this repository has no pin.** It consumes the package it publishes, from source under `packages/`, and the directory carries no release record. That is exactly the gap `pin.current` reports here:

```
pin.current gap
  the version is 1.0.0 and there is no digest on either side. The package
  directory carries no release record, so no published artifact stands
  behind it
```

So the argument that makes the rule set reproducible and the gap the verb reports are one fact, read twice. An adopter in this state can edit the rules that their own `--level` gate reads. The edit lands in the commit that the gate runs on, and nothing reports that the rule set moved. A reviewer sees a diff to a YAML file and no verdict beside it.

This is narrower than it looks and it is not nothing. The edit is visible in a diff, which is more than the lock gives on its own. What is absent is the mechanical statement that the rules behind a verdict are the rules the publisher shipped.

## Obligation

The corpus owes a statement of what holds a conformance rule set. The consumer at issue takes a package from source rather than from an artifact. The alternative is a ruling that such a consumer sits outside the guarantee, and that the report has to say so.

The second is defensible and it is not free. A publisher is the ordinary occupant of that state, and this repository is one. A report that named the rule set as unheld would be honest, and no line of the report says it today.

## Discharge

Either of two things closes it. A digest over the rule set, recorded where a run can compare it. Or a line in the report of `headwater conformance` that names the rule set as unheld, whenever the package carries no release record.

The record closes when a run over a repository that takes its package from source states the standing of the rules it evaluated against.
