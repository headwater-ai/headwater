---
id: OBL-repo-0027
title: "What `optional` holds under the `package` root"
status: current
status_since: 2026-08-12
last_verified: 2026-08-13
summary: "The grammar of `$package.optional.forbids` is settled and the shape of what it reads is not."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - DR-repo-0002
---

# What `optional` holds under the `package` root

## Context

The grammar of `$package.optional.forbids` is settled and the shape of what it reads is not. [Spec 7](../spec/07-distribution-and-federation.md#publishing) declares no `optional` block in the package manifest.

## Obligation

So nothing says whether the library is one flat set of named declarations, or one set for each block. The flat reading needs a name to be unique across blocks, and it lets a reference resolve on its own. The other reading makes a reference resolvable only against the address that receives it, which costs `explain` the ability to report what a reference reads.

This is the fifth surface that has a required declaration and no stated form.

## Discharge

The resolver refuses a `$package` reference and names this gap as the reason, rather than report a node that does not exist. What closes this is a statement of the shape in spec 7, and nothing else reaches it.
