---
id: HW-OBL-0151
status: current
status_since: 2026-09-06
title: "The glossary's Package entry omits conformance and still lists plugins, which #367 dropped"
summary: "The glossary's Package entry is missing `conformance` and still names `plugins`, a key spec 7 no longer carries."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
last_verified: 2026-08-26
waiting_on: adopter
---

# The glossary's Package entry omits conformance and still lists plugins, which #367 dropped

## Context

This surfaced while verifying #367, the correction to spec 7's manifest example. The glossary defines `Package` in one line, at `docs/spec/glossary.md:690`, listing the parts a published package carries. No rule reads that line the way `reachable` or `required_kind` read the manifest itself, so the entry can drift without failing a check.

## Obligation

The entry omits `conformance`, a `contents` key that has named the rule set and levels a package ships since well before #367. It still names `plugins`, which #367 removed from the canonical manifest example in spec 7. No citation in the corpus commits to a manifest key by that name. The only other appearance of `plugins` is spec 12's unrelated check-plugin architecture, `DocumentCheck` and WASM hosting. That architecture never ties itself to spec 7 or to `contents`.

A correct entry lists exactly six `contents` keys: `taxonomy`, `conformance`, `bundles`, `migrations`, `doctrine`, and `templates`. It also covers the top-level `profiles` and `interview` keys, if the entry means to name those too.

## Discharge

This closes when the `Package` entry in `docs/spec/glossary.md` names the six `contents` keys spec 7's corrected example carries. The wording should stay consistent with spec 7's own account of a package. The fix touches only that one glossary line, not spec 7 itself, spec 12's check-plugin architecture, or the manifest schema.
