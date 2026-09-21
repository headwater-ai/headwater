---
id: HW-DR-0081
status: current
status_since: 2026-09-21
summary: "The hand register keeps a curated selection of headings and drops its `supersedes` edge over the generated register, which stays the complete list."
last_verified: 2026-09-21
title: "The hand-authored decision register indexes a subset of the shelf, and stops asserting it supersedes the complete one"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  evidence_basis: evidenced
---

# The hand-authored decision register indexes a subset of the shelf, and stops asserting it supersedes the complete one

## Context

[#827](https://github.com/headwater-ai/headwater/issues/827) found that [9 — The decision register](../spec/09-decisions.md) had stopped tracking the decisions shelf. Measured on 2026-09-21, the shelf held 80 records. The generated register at [9 — Open questions](../spec/09-open-questions.md) named a heading for each of the 80. `headwater generate` writes that file, and `generate --check` holds it whole or not at all. The hand-authored register named 37. Its front matter still declared `supersedes: HW-REG-open-questions`, an edge that claims a completeness the hand register does not have.

Two readers relied on the hand register's last heading to pick the next decision number on 2026-09-11. Both minted `HW-DR-0044`, a number the shelf had already used. The register's own gap caused the collision: a complete register would have shown the shelf already past that number.

Backfilling the missing 43 headings would restore the claim the `supersedes` edge makes. That work argues each heading the way the existing 37 are argued. It costs over four times the outstanding count, and it is not this issue's Done-when to spend.

## Decision

The hand-authored register keeps its existing headings as a curated narrative index, and its prose says so. It drops the `supersedes: HW-REG-open-questions` edge, because a partial register does not supersede a complete one. A new paragraph names the generated register as the complete list. A reader who finds no heading here should look there, rather than assume the decision does not exist.

`tools/repo/decision-register-fixtures.sh` holds the corrected relation. It checks that every record the hand register cites by identifier still exists on the shelf. It also checks that the register's front matter carries no `supersedes` edge toward the generated register. It runs in CI beside the other register fixtures.

## Consequences

Every citation of a Q-numbered heading in this corpus keeps resolving, because no heading moves or leaves the file. A reader who wants the complete list reads the generated register, which cannot fall behind by construction. The hand register can still fall behind on which headings it carries. That does not mislead a reader, because the register claims no completeness. Re-adding the `supersedes` edge without also completing the register is a regression the fixture now catches. The Q-number-to-record correspondence stays a separate, already lapsed convention that this decision does not rule on.
