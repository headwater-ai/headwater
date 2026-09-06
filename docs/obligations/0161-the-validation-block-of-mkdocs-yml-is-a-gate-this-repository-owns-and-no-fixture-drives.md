---
id: HW-OBL-0161
status: draft
status_since: 2026-09-06
summary: "Two raised severities in mkdocs.yml are the whole of two gates, and no case in this repository fails when either one is lowered"
last_verified: 2026-09-06
title: "The validation block of mkdocs.yml is a gate this repository owns and no fixture drives"
waiting_on: build
---

# The validation block of mkdocs.yml is a gate this repository owns and no fixture drives

## Context

`mkdocs.yml` carries a `validation:` block with three severities. Two of them are raised above the MkDocs 1.6 default, and each raise is a gate this repository chose rather than a default it inherited.

`links.anchors` rose to `warn` for [#431](https://github.com/headwater-ai/headwater/issues/431), after 335 links pointed at an anchor no built page carried for three weeks. `nav.omitted_files` rose to `warn` for [#528](https://github.com/headwater-ai/headwater/issues/528), after ten served index pages sat outside the navigation for the same reason. Each raise is stated in a comment beside the key and in nothing else.

## Obligation

**A change that lowers either severity back to `info` passes every gate this repository has.** CI runs `mkdocs build --strict` over the tree of the moment. That build is green whether the severity is raised or lowered, because each raise landed in the change that emptied its own population. So the two gates are asserted by prose and held by no case that fails without them.

**Every other gate here has a suite that drives it over a scratch corpus, including every refusal.** `.githooks/fixtures.sh`, `.claude/hooks/fixtures.sh`, `.claude/skills/fixtures.sh`, `.claude/tutorial/fixtures.sh` and `tools/site-fragments-fixtures.sh` are the five. `mkdocs.yml` is the one configuration file of this repository that none of them reads.

**The cost is the shape both raises were made against.** A severity of `info` prints the defect to a log nobody reads. The defect is then invisible for as long as it takes a person to notice the page. That cost was measured twice, at three weeks and at two days.

## Discharge

**This record discharges when a case fails on a lowered severity and passes on a raised one.** The case builds a scratch copy of this corpus and adds a page that no navigation entry names. `mkdocs build --strict` over that tree has to exit 1. It then lowers `nav.omitted_files` to `info` over the same tree and asserts exit 0. `tools/site-fragments-fixtures.sh` is the shape to copy, because it already drives the second reader of the same class over a scratch tree.

**A weaker discharge names each raised severity in one list that a case reads.** That catches a deletion and not a downgrade, so it is worth less than the pair above.

**What does not discharge this.** CI running `mkdocs build --strict` on a tree whose population is empty. A comment stating why the severity is raised, which is what both keys carry today.
