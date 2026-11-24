---
id: HW-OBL-0249
status: current
status_since: 2026-11-24
summary: "When only the commit gate refuses the merge commit of pre-pr.sh, the script prints failed at merge and a conflict line that names no path."
last_verified: 2026-11-24
title: "pre-pr.sh reports a merge conflict with no path when only the commit gate refused the merge commit"
waiting_on: build
provenance:
  warrant: asserted
---

# pre-pr.sh reports a merge conflict with no path when only the commit gate refused the merge commit

## Context

`hw-run-policy` sends a finding about the build order, its tooling or the CI of this repository to this shelf. Run `20261122-0114` wrote intake line 39. The product owner ruled it RECORD into this record, which its end-of-run report calls P4. This record read the merge step of the script again on `0cdd96298`. The incident is as the builder of #1971 reported it.

## Obligation

The merge step of `tools/run/pre-pr.sh` writes `conflict on <files>` from the list of conflicted paths, and `see <log>` when that list is empty. In the tree of #1971 the merge had no text conflict. The commit gate refused the merge commit with a stale-projection message, because the hook ran the binary built before the merge. The script printed `failed at merge` and a conflict line that named no path. A hand merge, a rebuild and `headwater generate` then moved nothing, and the merge committed.

## Discharge

This record discharges when `pre-pr.sh` tells a refused merge commit apart from a text conflict and names the refusal. A change that records why the two stay one line also discharges it.
