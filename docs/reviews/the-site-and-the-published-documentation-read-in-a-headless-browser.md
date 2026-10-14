---
id: HW-REV-the-site-and-the-published-documentation-read-in-a-headless-browser
status: current
status_since: 2026-10-14
summary: "A headless browser read 623 served pages at phone and desktop width. 81 scroll sideways, each for a wide table. Six pages are flagged for the owner to look at."
last_verified: 2026-10-14
doc_type: review_record
title: "The site and the published documentation, read in a headless browser"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - tools/site/review-site.py
    - tools/site/site-review-fixtures.sh
    - tools/site/site-figures-fixtures.sh
---

# The site and the published documentation, read in a headless browser

## What this is

[Issue #1489](https://github.com/headwater-ai/headwater/issues/1489) asked for the site to be read the way a person reads it: at phone width and at desktop width, in the light and the dark theme, with the keyboard, and with the navigation a reader sees. This record is the first half of that review, the agent pass. The owner ruled on 2026-10-03 that an agent reads every page first and that the owner then looks at the pages it flags. The section *Pages the owner looked at* is empty until the owner does.

**The review ran headless.** The browser was Chrome for Testing 153.0.8010.12, the `chrome-headless-shell` that Playwright installs, on 2026-10-15. No person had looked at a page when this record was written.

One existing machine reading came before this one. `tools/site/check-site-console.py` loads every served page in headless Chrome on each CI run and fails on an uncaught error. It reads one window size, one color scheme and the console. A page that is wider than a phone has an empty console, so that check passes it.

## How the pages were read

`tools/site/review-site.py` is the reader, and `tools/site/site-review-fixtures.sh` holds it. Each reading is one browser load at one size and one color scheme. The page loads inside a frame of the reading's size. Full headless Chrome makes no window narrower than 500 pixels, and a frame is 390 pixels wide in every browser. A reading whose page saw another width is flagged as not taken, and it is not judged. The first runs below loaded the page as the window in the headless shell, which gives 390 pixels. A second live run through the frame on the same day flagged the same 81 of 623 pages. An injected probe records four things on each load:

- **Horizontal overflow.** The page's `scrollWidth` against its `clientWidth`, and the outermost elements past the right edge.
- **The color scheme the page saw.** A dark reading that the page did not see as dark is flagged as a broken reading. Blink's `preferredColorScheme` setting is `0` for dark and `1` for light, measured on this host. The flag `--force-dark-mode` did nothing in the headless shell.
- **Keyboard focus.** The probe focuses the first focusable element and reads its computed outline, the color of that outline and its box shadow. On this host the focus took on every reading, and `:focus-visible` matched, so the reading is reliable headless.
- **The navigation.** On a generated page, every link in the shelves navigation must answer 200, and every top-level and second-level section of `.headwater/nav.yml` must appear in it by name.

Two runs make the record:

| run | pages | readings | flagged | time |
|---|---|---|---|---|
| the nine hand-built pages of `site/`, from the tree at `486bd3d6` | 9 | 36 | 0 | 8 s |
| every URL of `https://headwater.tools/sitemap.xml`, and the pages its navigation reaches that the sitemap does not list | 623 | 1286 | 81 | 60 s |

Every page was read at 1280x800 and at 390x844 in the light scheme. Each of 20 pages was also read in the dark scheme at both sizes, with a screenshot of the first screen of each reading. That is the nine hand-built pages and one landing page for each of the 11 second-level sections of `.headwater/nav.yml`. So the live run took 623 pages times 2 readings, and 20 pages times 2 more, which is 1286. The screenshots are outside the repository, under `/home/james/.claude/jobs/9e020361/tmp/issue-1489/screens/`. The ones of the tree are in that directory and the ones of the live site are in its `live/` subdirectory. No screenshot is committed, so each row quotes a reading instead.

The sitemap lists 617 URLs. The navigation reached six more, the landing pages of the six process sections. The sitemap lists no page under `process/`, as HW-DR-0107 rules, and each of the six answered 200.

## The hand-built pages

Each page was read at 1280 and 390 pixels wide, in light and in dark. On every reading the page did not scroll sideways, the dark reading saw the dark scheme, and the brand link took focus with the browser's own ring (`outline auto 1px`).

| page | what was read | defects |
|---|---|---|
| `site/index.html` | desktop and phone, light and dark | The brand glyph renders as an empty box (see *Defects*). |
| `site/changelog/` | desktop and phone, light and dark | The brand glyph. |
| `site/compare/` | desktop and phone, light and dark | The brand glyph. |
| `site/glossary/` | desktop and phone, light and dark | The brand glyph. |
| `site/how-it-works/` | desktop and phone, light and dark | The brand glyph. |
| `site/install/` | desktop and phone, light and dark | The brand glyph. Four system versions that nothing holds (see *Every figure on a hand-built page*). |
| `site/ns/` | desktop and phone, light and dark | The brand glyph. |
| `site/proof/` | desktop and phone, light and dark | The brand glyph. A release typed outside a `data-figure` element, on a page that says nothing on it was typed by a person, now held. Flagged for the owner for its dark theme. |
| `site/tutorial/` | desktop and phone, light and dark | The brand glyph. A count in prose that nothing holds. |

## The sections of the documentation site

Each row is one second-level section of `.headwater/nav.yml`, under its top-level section. A page belongs to the section whose navigation names it. Every page of a section was read at desktop and phone width in the light scheme. The landing page named in the second column was also read in the dark scheme at both widths. The shelves navigation showed both top-level sections and all 11 second-level sections on each of the 614 generated pages, at both widths. Every link in it answered 200.

| section | landing page read four ways | pages read | flagged | defects |
|---|---|---|---|---|
| Reference: Guides | `tutorials/` | 8 | 1 | `explanations/how-a-headwater-release-reaches-an-adopter/` scrolls sideways at 390 px, a table to 558 px. |
| Reference: Specification | `spec/` | 73 | 33 | 33 pages scroll sideways at 390 px, each for a table. `spec/16-harness-support/` also does at 1280 px, a table to 1332 px. Flagged for the owner: the navigation fills the first screen at 390 px. |
| Reference: Taxonomy library | `taxonomies/` | 8 | 7 | 7 pages scroll sideways at 390 px. `taxonomies/` does so in light and dark, a table to 503 px. |
| Reference: Decisions and evidence | `decisions/` | 368 | 40 | 40 pages scroll sideways at 390 px. `reviews/the-sixty-four-restored-help-strings-checked-against-the-binary/` also does at 1280 px, a table to 1672 px. |
| Reference: Probes | `probes/` | 151 | 0 | None. |
| Project process: Process specifications | `process/specs/` | 1 | 0 | None. The sitemap leaves the page out, as HW-DR-0107 rules. |
| Project process: Process decisions | `process/decisions/` | 1 | 0 | None. The sitemap leaves the page out, as HW-DR-0107 rules. |
| Project process: Process evaluations | `process/evaluations/` | 1 | 0 | None. The sitemap leaves the page out, as HW-DR-0107 rules. |
| Project process: Process obligation records | `process/obligations/` | 1 | 0 | One title in its list read `The CI job named \"(advisory)\" carries…`, with the escape characters shown. |
| Project process: Process how-to guides | `process/how-to/` | 1 | 0 | None. The sitemap leaves the page out, as HW-DR-0107 rules. |
| Project process: Process explanations | `process/explanations/` | 1 | 0 | None. The sitemap leaves the page out, as HW-DR-0107 rules. |

The nine pages outside these rows are the hand-built pages, and none of them was flagged on the live host either. The process sections were read at their landing pages only, because the sitemap names none of their documents. That is by design: [HW-DR-0107](../decisions/0107-the-process-shelves-stay-published-in-a-labeled-section-of-the-site-that-a-search-engine-does-not-index.md) keeps the process shelves out of search indexes. A full review of the process half reads `.headwater/site-deploy` from a local build, which this host could not make on 2026-10-15.

## Defects, and where each one went

| defect | where | outcome |
|---|---|---|
| A title of HW-OBL-0145 carried `\"` into the served navigation and the shelf index. | `docs/process/obligations/0145-the-ci-job-named-advisory-carries-every-blocking-shell-suite-in-the-repository.md` | Fixed in this change. The title and the heading now read `"(advisory)"`. |
| `site/proof/` named the release `v0.5.0` outside a `data-figure` element, and nothing held it. | `site/proof/index.html` | Fixed in this change. `tools/site/site-figures-fixtures.sh` holds it against `README.md`. |
| 81 of 623 pages scroll sideways at 390 px, 2 also at 1280 px. Every flag names a wide table that nothing contains. | the theme under `mkdocs/overrides/` | Intake, run 20261014-1040. |
| The brand glyph U+1F704 renders as an empty box on a host with no font for the Alchemical Symbols block. | `.brand .glyph`, on both halves | Intake, run 20261014-1040. Flagged for the owner, who can say whether their own devices show it. |
| The four system versions on the install page and a count in the tutorial's prose are held by nothing. | `site/install/index.html`, `docs/tutorials/your-first-governed-corpus.md` | Intake, run 20261014-1040, one line each. |

Each intake line goes to the product owner, who files the issue. The second half of this review writes each issue number into this table.

## Every figure on a hand-built page

On 2026-10-15 the nine pages carried 106 `data-figure` elements over 34 distinct keys, on 3 pages: 57 on `site/index.html`, 44 on `site/proof/index.html` and 5 on `site/how-it-works/index.html`. `tools/site/refresh-figures.sh` fills each one when the site is published, and it refuses a key it cannot measure and a key that reaches no page. Each other number on the nine pages is in the table below. A section number, a step number and a numbered example such as `ACME-DR-0001` are labels and not figures, so the table leaves them out.

| figure | page | what holds it |
|---|---|---|
| every `data-figure` element (106) | `index`, `proof`, `how-it-works` | generated by `tools/site/refresh-figures.sh` at publish |
| the engine release, `v0.5.0` and `version 0.5.0` | `index`, `install`, `proof` | `tools/site/site-figures-fixtures.sh`, against `git checkout v…` in `README.md` |
| the engine release, `version 0.5.0` | `tutorial` | `check_version_pin` in `tools/site/render-tutorial.py`, and step 9 of the release guide, which `tools/repo/release-guide-fixtures.sh` holds |
| the package release, `4.13.0` | `tutorial` | `.claude/tutorial/fixtures.sh` runs the tutorial document, and `tools/site/refresh-figures.sh --check` holds the page's verbatim blocks against it |
| `21 check instances`, and every other number in an output block | `tutorial` | the same two |
| "went from 7 to 21", and "fourteen more" | `tutorial` | nothing. Intake, run 20261014-1040 |
| the tutorial's date, `2026-09-09` | `tutorial` | the `data-tutorial-date` drift check in `tools/site/refresh-figures.sh`, against the date the tutorial document states for its run |
| Rust `1.91` | `install` | `tools/site/site-figures-fixtures.sh`, against `rust-version` in `engine/Cargo.toml` |
| Linux x86_64, and macOS on Apple silicon | `index`, `install` | `tools/site/site-figures-fixtures.sh`, against the target matrix in `.github/workflows/release.yml` |
| glibc `2.39`, Debian 12, Ubuntu 20.04, Debian 11 | `install` | nothing. Intake, run 20261014-1040 |
| the date of the measurement campaign, `2026-09-30` | `index`, `proof` | a past event. It does not move when the corpus moves |
| each release, date and taxonomy version in an entry | `changelog` | a past release. An entry states what that release was, and it does not move when the corpus moves |

`compare`, `glossary` and `ns` state no figure.

## Which readings a machine repeats

| reading | what runs it | cost per run |
|---|---|---|
| HTTP 200, a probe that ran, and an empty console, on every served page | `tools/site/check-site-console.py`, in CI | 151 s on `486bd3d6`, and 14 s for its fixtures |
| Overflow and focus on the nine hand-built pages, at both widths | `tools/site/review-site.py site`, in CI from this change | 2.4 s, and 13.5 s for the 38 cases of its fixtures, measured on 2026-10-15 |
| Overflow, focus and navigation on every generated page, and dark readings and screenshots on a chosen set | `tools/site/review-site.py`, by hand | 60 s over the live host with 8 browsers, measured on 2026-10-15 |
| The release, the Rust floor and the platforms that a hand-built page states | `tools/site/site-figures-fixtures.sh`, in CI from this change | 0.2 s |

HW-DR-0110 records why the generated half stays a run by hand.

A person judges what no reading here measures: whether a page reads well, whether a color pair has enough contrast, whether the first screen shows what a reader came for, and whether a screenshot looks right.

## Flagged for the owner

Each flagged page names the screenshots to open, under `/home/james/.claude/jobs/9e020361/tmp/issue-1489/screens/`.

1. **The brand glyph, on every page.** It is an empty box in every screenshot of this host. Open `home--390x844-dark.png` and `live/taxonomies--390x844-dark.png`. Say whether a device you use shows the glyph.
2. **`spec/` at phone width.** The two rows of shelves navigation and the open panel fill the whole first screen, and no content shows above the fold. Open `live/spec--390x844-light.png` and `live/tutorials--390x844-light.png`.
3. **`taxonomies/` at phone width.** It scrolls sideways in both schemes, and the screenshot shows the horizontal scrollbar. It stands for the 81 pages that scroll sideways. Open `live/taxonomies--390x844-dark.png` and `live/taxonomies--390x844-light.png`.
4. **`proof/` in the dark scheme.** The band below the first section is light on a dark page. In the light scheme the same band is dark. Say whether the inversion is the design. Open `live/proof--390x844-dark.png` and `live/proof--390x844-light.png`.
5. **`process/obligations/` before the fix.** The list showed `\"(advisory)\"`. Open `live/process_obligations--390x844-dark.png`. The fix reaches the live site at the next deploy.
6. **`decisions/` at desktop width in the dark scheme.** The shelf index lists 1, 2, 3, 101 and 86 in that order, under "the reading order this corpus derives". Say whether a reader expects that order. Open `live/decisions--1280x800-dark.png`.

## Pages the owner looked at

The owner has not looked yet. The second half of this review writes here which of the flagged pages the owner opened, and what the owner found on each.
