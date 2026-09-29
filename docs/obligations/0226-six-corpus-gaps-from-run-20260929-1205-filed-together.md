---
id: HW-OBL-0226
status: current
status_since: 2026-09-29
summary: "Run 20260929-1205 surfaced six findings about this corpus's own edges, stamps and wording, none with a reader outside this repository. This record files them together under the intake cap."
last_verified: 2026-09-29
title: "Six corpus gaps from run 20260929-1205, filed together"
waiting_on: build
---

# Six corpus gaps from run 20260929-1205, filed together

## Context

`hw-run-policy` caps what one pass sends to this register as full records. Past the cap, one record lists the rest. Run `20260929-1205` wrote six intake lines about this corpus's own edges, stamps and wording. The product owner ruled each one RECORD, on the tenth-merge pass and the end-of-run pass of that run. None names a reader outside this repository. Each item below gives the stage that found it and the issue in hand at the time.

## Obligation

**Edges that a person types.** Each relation below is `created_by: agent`, so an agent may not write it, and HW-DR-0083 leaves it to a person.

- HW-DR-0094 rests its Context on HW-DR-0097 in prose and declares no `traces_to: HW-DR-0097` edge. (build, #1339)
- HW-DR-0099 names line 188 of `docs/evaluations/graph-export-and-federation.md` as superseded in prose only. It declares no `traces_to` edge to HW-EVAL-graph-export-and-federation. (build, #1368)

**Stamps.**

- HW-OBL-0062 gained a Discharge sentence on 2026-09-30 and keeps `last_verified: 2026-08-13`. No rule says whether a content edit to an open obligation owes a new re-verification stamp. (build, #1410)
- Row 20 of `docs/reviews/the-sixty-four-restored-help-strings-checked-against-the-binary.md` quotes the help text of `export --at` from before #1343. The review is a point-in-time record and stays as written. Nothing re-checks a help string that a review recorded, so the next reader cannot tell that the row is out of date. (build, #1343)

**Wording narrower or looser than the behavior.**

- One resolved `traces_to` edge raised seven population tallies of the `adoption` block by one each, with no new finding. Among them are `relation.target.unresolved` (886 to 887), `relation.target.is_source`, `relation.target.suspect`, `relation.endpoint.not_permitted`, `lifecycle.dependency.on_initial` and `warrant.evidence.unsupported`. A tally under the name of a rule reads as a count of failures, and this one counts every edge the rule reads. No document says which it is. (build, #1386)
- Line 40 of HW-OBL-0200 says the seal removes "every document under docs/ that names a selected probe". Since PR #1413 (#1384 PR D) the seal also removes files outside `docs/` that are not a fold, so the sentence is narrower than the behavior. (build, #1384)

## Discharge

Each item discharges alone, when the file it names says or does what the item asks, or when a change records why it stays. The record discharges when every item has. An item that grows a reader outside this repository leaves this record for an issue, and the record says where it went.
