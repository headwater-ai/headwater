---
id: HW-DR-0110
status: current
status_since: 2026-10-14
summary: "CI reads the nine hand-built pages at 390 and 1280 pixels wide for 2.4 seconds a run. The same reading over 623 generated pages flags 81 for one table defect, so it stays a run by hand until that defect is fixed."
last_verified: 2026-10-14
title: "110 — A headless review of the nine hand-built pages joins CI, and the review of the generated half stays a run by hand"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - tools/site/review-site.py
    - tools/site/site-review-fixtures.sh
    - tools/site/site-figures-fixtures.sh
---

# 110 — A headless review of the nine hand-built pages joins CI, and the review of the generated half stays a run by hand

## Context

[Issue #1489](https://github.com/headwater-ai/headwater/issues/1489) asked whether a headless-browser job joins CI, with its cost in seconds per run. One browser step already runs in CI. `tools/site/check-site-console.py` loads every served page and fails on an uncaught error. It took 151 seconds on `486bd3d6`, and its fixtures took 14 seconds more.

That step reads one window size and one color scheme. A page that is wider than a phone has an empty console, so the step passes it. `tools/site/review-site.py` reads what the step does not: overflow at 1280 and 390 pixels wide, the dark scheme, keyboard focus, and the navigation against `.headwater/nav.yml`. The review record [the site and the published documentation, read in a headless browser](../reviews/the-site-and-the-published-documentation-read-in-a-headless-browser.md) holds the results.

These are the costs, measured on 2026-10-15 on an 18-core host with Chrome for Testing 153:

| reading | pages | readings | time |
|---|---|---|---|
| overflow and focus on the nine pages of `site/`, at both widths | 9 | 18 | 2.4 s |
| the 29 cases of the fixtures of `review-site.py` | pages the suite writes | not counted | 7.6 s |
| the figures a hand-built page states, against their sources | 3 | none | 0.2 s |
| overflow, focus and navigation on every page of the live host, with 8 browsers | 623 | 1286 | 60 s |

The last run flagged 81 of the 623 pages. Each flag names a wide table that the theme does not put in a container that scrolls. None of the nine hand-built pages was flagged.

## Decision

Three steps join the CI job that runs the console check, beside it:

1. `python3 tools/site/review-site.py site`. It fails when a hand-built page scrolls sideways at either width, or when its first focusable element shows no focus indicator.
2. `sh tools/site/site-review-fixtures.sh`. Spec 12 refuses a check that has no failing fixture. The decisive case is a page 1000 pixels wide, which the review must flag at 390 pixels and not at 1280.
3. `sh tools/site/site-figures-fixtures.sh`. It fails when a hand-built page names a release, a Rust floor or a platform that disagrees with `README.md`, `engine/Cargo.toml` or `.github/workflows/release.yml`.

The first two exit 3 when the runner has no browser. The step then writes a warning annotation and passes, as the console step does. The three add about 10 seconds to a run.

The review of the generated half stays a run by hand, for two reasons. First, today it would fail every run on 81 pages for one defect that has its own fix. A gate that is red before anybody changes a page tells nobody anything. Second, the dark readings and the screenshots are for a person to read, and no exit status carries them.

## Consequences

A pull request that makes a hand-built page scroll sideways on a phone now fails CI. Before this decision every gate passed such a page.

The generated half joins CI when the wide tables are contained. The condition that reopens this decision is a run of `review-site.py` over `.headwater/site-deploy` that flags no page for overflow. The step then reads that directory instead of `site/`, and adds the time of one reading for each page at each width. The live run took 60 seconds for 1286 readings over the network. Nobody has measured a run over a local directory, and the step owes that measurement before it joins.

The dark readings, the screenshots and the owner's look stay by hand. A review that repeats them runs `review-site.py` with `--shoot` and `--shots`, and its record names the pages it flagged.
