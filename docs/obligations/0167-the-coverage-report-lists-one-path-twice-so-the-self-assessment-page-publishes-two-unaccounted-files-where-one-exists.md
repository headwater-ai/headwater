---
id: HW-OBL-0167
status: current
status_since: 2026-09-06
summary: "coverage.unaccounted reads ['.headwater/ids', '.headwater/ids'], so the self-assessment page publishes 2 under a label that names files nobody accounted for."
last_verified: 2026-09-06
title: "The coverage report lists one path twice, so the self-assessment page publishes two unaccounted files where one exists"
waiting_on: adopter
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# The coverage report lists one path twice, so the self-assessment page publishes two unaccounted files where one exists

## Context

`headwater check --json` carries `coverage.unaccounted`, which is the list of paths the census put in no row. On 2026-09-06 that list reads `['.headwater/ids', '.headwater/ids']`. It holds one path, and it holds that path twice.

`.headwater/ids` is the directory of the identifier claim store, which reached `main` in `0efe2ba` on the same day. It is a directory and not a file, and 219 claim files sit under it.

    ./engine/target/release/headwater check --root . --json \
      | python3 -c "import json,sys; print(json.load(sys.stdin)['coverage']['unaccounted'])"

`tools/refresh-figures.sh` writes the length of that list into the `census.unaccounted` figure, and three pages carry it. So the self-assessment page publishes `2` under the label "of those silently unaccounted for". The landing page publishes the same `2` in a tile and in a ticker. The number a visitor reads is the length of a list rather than a count of files.

**The census partition is unaffected.** `typed`, `generated`, `untyped`, `excluded` and `not a document` still add to `seen`, and `tools/refresh-figures.sh` exits when they do not. `unaccounted` is a separate list, so a repeated entry in it moves no other figure.

**One sentence outside every figure goes false with it.** `site/glossary/index.html` defines a census as a reading where "nothing is silently unaccounted for". That sentence carries no `data-figure` element, so neither a script nor a rule reads it.

## Obligation

A census accounts for every path it saw, and this repository publishes that property as the thing its self-assessment page is for. Two readings of one path defeat the property in two separate ways.

**The count is wrong.** One path is unaccounted for and the page states two. A reader who takes the tile at face value looks for a second file that no run can name.

**The entry is a directory.** Every other census row is about a file, and a reader who follows `.headwater/ids` finds 219 files rather than one thing left over. So the label reads as a file count over an entry that is not a file.

The cost outside this repository is nothing today, because no adopter runs a census over this tree. The cost inside it is a page that a visitor reads on the day this repository goes public.

## Discharge

A reading of `census.rs` that says why one path arrives twice, and one of three answers discharges this.

**The duplicate is the defect.** The discharge is the deduplication, plus a fixture that puts one path in the list twice and reads one entry back.

**The directory is the defect.** The discharge is a census that reports the files under an unaccounted directory. The other form of it reports that directory as excluded, under the rule that already covers `.headwater/`.

**Both are the defect.** The discharge is both of the above, and the page then states a count of files that a reader can enumerate.

Nothing here proposes one. This record is the measurement, and the reading of `census.rs` that chooses between the three is the work.
