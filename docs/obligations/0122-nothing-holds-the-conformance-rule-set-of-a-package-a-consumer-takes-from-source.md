---
id: HW-OBL-0122
status: discharged
status_since: 2026-08-24
waiting_on: ruling
summary: "A conformance rule set is held by the release digest and never by the lock, so a repository that takes its package from source can edit the rules its own gate reads."
last_verified: 2026-08-24
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

**This record is discharged.** [#336](https://github.com/headwater-ai/headwater/issues/336) moved the authored files of `headwater/standard` to `taxonomy-source/headwater-standard/`, outside the vendored package root. That root then held a real vendored artifact, published from that source and installed by `headwater taxonomy vendor`. It was `packages/` on the day of #336 and it is `.headwater/packages/` now ([HW-DR-0064](../decisions/0064-the-vendored-package-root-moves-under-headwater-and-the-old-root-is-named-in-a-refusal.md)).

**The first of the two closing conditions this record named is the one that closed it.** A digest over the rule set, recorded where a run can compare it, was the first. `release::compute` takes a digest over every member of the vendored artifact, and `conformance.yml` is one of those members. `.headwater/taxonomy.yml` now pins that digest as `taxonomy.digest`, and `headwater conformance` reports `pin.current` as met against it.

**The premise named in the context above does not hold, and that is why the second condition does not apply.** This repository takes no package from source. It takes a vendored artifact, on the same terms as any other consumer of `headwater/standard`. A report line naming the rule set as unheld would misstate the corpus this record now describes.

**What changed is the directory, and not the rule.** The base package directory was once both the maintained source and the name a consumer resolves against. No digest could stand over a directory that a person kept editing. Nobody edits it now. Only `taxonomy publish --from` and `taxonomy vendor` write there, so a release record is always the reason anything sits under that name.
