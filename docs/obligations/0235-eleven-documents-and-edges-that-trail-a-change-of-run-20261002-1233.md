---
id: HW-OBL-0235
status: current
status_since: 2026-10-03
summary: "Changes of run 20261002-1233 left eleven stale sentences and missing edges. They include n8n figures that no job holds, ungoverned tools/probe files and stale counts of kinds."
last_verified: 2026-10-03
title: "Eleven documents and edges that trail a change of run 20261002-1233"
waiting_on: build
provenance:
  warrant: asserted
---

# Eleven documents and edges that trail a change of run 20261002-1233

## Context

The builders and the maintainer of run `20261002-1233` found sentences that a change made stale, and code that no document governs or cites. None stops an adopter, and most name no reader outside this repository. So the product owner ruled each one RECORD and not an issue. Intake lines 8, 34, 46, 58, 60, 61, 62, 63, 64, 66, 77, 93 and 123 were ruled during the run. Lines 88, 100, 106, 128, 129 and 130 were ruled at the top of run `20261003-1026`. This is one of three spec 13 records for that run. Each item was checked on `a46bd95e` by the file and the heading or symbol it names.

## Obligation

**Evaluations and their figures.**

- The subsection *Aging: an edge records a digest of what it reached* of `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md` says that a change earns a stamp when a person re-read the document that declares the edge. Since #1520 a change states which edge was re-read. The evaluation is a record of a moment, and `headwater check` does not flag it.
- In `docs/evaluations/n8n-worked-example.md`, the standards-spec count table has the row `| Findings raised | 21 | 143 |`. `tools/taxonomy/drive_n8n.py` holds `Findings raised` only inside `## The count`, so no job holds this row or its column of 143. The prose "raised 12 findings" in `## The count` and the README shelf rows 1 and 2 are held by no run. Four mutants of the n8n count guard survive: W1 drops the row-count check, W3 narrows the tool name, W4 sums the nonzero rows only, and W5 changes the choice of the raised row.
- `docs/evaluations/n8n-worked-example.md` declares no `traces_to` onto `tools/taxonomy/drive_n8n.py` or onto the READMEs it is held against. HW-OBL-0231 records this gap and does not yet name these edges.

**The decision-record package.**

- The Discharge of HW-OBL-0039 says that the evidence-cited expectation "still names a `to_kind`, and it may now drop it". That expectation no longer exists. The design-spec doctrine states what HW-DR-0044 moved out. The first finding under `docs/taxonomies/decision-record/doctrine.md#findings` came through `relation.participation.overdue`, which no longer reports.
- `tools/repo/decision-record-fixtures.sh` is a code path that no document governs, and the decision-record doctrine may declare `governs` onto a code path. The description of that script in `docs/taxonomies/README.md` names the digest and byte-identity cases and not the Beacon root it also runs.

**Edges between specs, code and tests.**

- `docs/spec/05-ai-integration.md` describes what spec 15 does with the mark of a probe result. It declares no `traces_to` or `cites_evidence` edge onto `docs/spec/15-the-recorder-contract.md`, so nothing points a reader of the next change to spec 15 at it.
- `tools/probe/probe-record-fixtures.sh`, `tools/probe/ci-confine.sh`, `tools/probe/ci-confine-fixtures.sh`, `tools/probe/campaign.sh`, `tools/probe/campaign-dry-run.sh` and `tools/probe/layer-campaign.spec` are in the governed scope of `tools/**`. Spec 15 declares `governs` onto `tools/probe/probe-record.sh` and `tools/probe/egress-proxy.py` only. Spec 15 says that `probe-record-fixtures.sh` holds its environment and bind lists against the driver. The header of that script names the tools it holds and not spec 15.
- Slice 4d of #1572 repointed engine comments to documents outside the `traces_to` list of their subsystem spec. `docs/subsystems/parse-and-census.md`, `taxonomy-distribution-and-audit.md`, `graph-build.md` and `authoring.md` now have code that cites the `headwater generate` contract. `projections-and-export.md` has code that cites the checks-and-cache page. `command-surface.md` has code that cites the `headwater taxonomy` contract and spec 4. No `traces_to` lists them.
- `engine/crates/generate/tests/spec_seven_export.rs` holds six invariants of spec 7 and the non-claims count of HW-DR-0016 and HW-DR-0017. No document cites it, and no record under `docs/verifications/` names it.

**Counts and statements no change has caught up with.**

- HW-DR-0036 counts ten projection kinds that a taxonomy may declare, in the paragraph on `site_nav`. HW-OBL-0043 says "Eight kinds are declarable". `Kind::DECLARABLE` in `engine/crates/generate/src/lib.rs` holds 11, and the projections-and-export page states 11. Spec 9 says "129 records", and `docs/taxonomies/decision-record/doctrine.md` says "38 of the 39" open records wait on a ruling. The obligation shelves held 210 records when the line was written.
- The harness gives one `promptId` to more than one submission, such as a request and a `/compact` 27 seconds later. 39 of 587 joined identifiers carry more than one line of the shadow log, and 7 of them disagree on silence. `tools/run/shadow-capture.sh` and `tools/run/shadow-mine.sh` read the first record and the earliest line, and no document states that an identifier is not one prompt. The builder of #1670 meets this first.

## Discharge

Each item discharges alone, when the document states what the tree holds or declares the edge it names, or when a change records why it stays as written. The record discharges when every item has. An item that gains a reader outside this repository leaves this record for an issue, and the record says where it went.
