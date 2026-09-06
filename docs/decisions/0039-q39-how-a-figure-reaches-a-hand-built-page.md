---
id: HW-DR-0039
status: current
status_since: 2026-08-30
summary: "The interpolating form that HW-DR-0037 admits runs before the commit rather than after it, and a script writes every figure on a hand-built page from a run."
last_verified: 2026-09-06
title: "Q39 — How a figure reaches a hand-built page"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  constrains:
    - HW-DR-0037
  governs:
    - tools/refresh-figures.sh
  traces_to:
    - notes/website-design-brief.md
---

# Q39 — How a figure reaches a hand-built page

## Context

**[HW-DR-0037](0037-q37-which-parts-of-the-site-are-hand-built-and-which-are-a-projection-of-this-corpus.md) admits two forms where a number belongs on a hand-built page, and it closed both.** The first form is a link to the generated artifact that produced the number. The second is a build that interpolates the number, so that the page carries no source of its own.

**The record measures the second form as unavailable, and the measurement is about the deploy path.** `wrangler.jsonc` declares an asset directory and no build command, so nothing runs between the commit and the served bytes. `site/_headers` sets `default-src 'none'`, so a served page reaches no data file at read time either. Both readings are still true on this tree, and this record disturbs neither.

**The first form is unavailable for a different reason, which [#435](https://github.com/headwater-ai/headwater/issues/435) states.** No workflow of this repository publishes a built artifact anywhere a visitor can reach. A run page of this repository answers a reader with no credentials since 2026-09-06, and it carries no artifact for a link to name.

**Both admitted forms were therefore closed, and a page of figures had nowhere to stand.** The self-assessment page waited behind that, and the landing page carried figures from a run that nobody could name.

**The cost of the gap is measured rather than argued.** `notes/website-design-brief.md` section 3 states a table of run figures typed by hand on 2026-08-23. It reports 245 files under the corpus root, 523 findings, and taxonomy `headwater/standard 3.2.0`. A run of this engine on 2026-08-26 reports 332 files, 524 findings, and `headwater/standard 3.3.0`. Three days moved every figure in the one document that states why a hand-typed figure is a defect.

## Decision

**The interpolating form runs before the commit rather than after it.** HW-DR-0037 forecloses a build between the commit and the served bytes. It forecloses nothing on the machine of the person who writes the page. So the interpolation happens there, and the interpolated figure is in the bytes that the commit carries.

**`tools/refresh-figures.sh` is that build.** It runs the engine over this repository, reads the result, and rewrites the text of every element under `site/` that carries a `data-figure` attribute. A key with no measurement fails the run, and so does a measurement that reaches no page.

**Five sources supply every figure, and each one is a run.** `headwater check --json` gives the census, the findings, the rules wired, the taxonomy and the clock. The text output of the same verb gives the census lines that the JSON omits, and the obligation register. `docs/interfaces/README.md` gives the verb count and the group count, and `headwater generate --check` holds that file. `engine/crates/generate/src/profile.rs` gives the emitter split, read off the `Emitter` values and the arms of `is_built`. `headwater conformance` gives the level this repository reaches.

**Every figure states its denominator in the prose beside it.** A count with no stated population is a trap for the next reader, and this repository has paid for that more than once. The script also refuses to finish when the census partition does not add up to the file count it reports.

**`--check` writes nothing and exits non-zero when a page and a fresh run disagree.** That is the form to put in front of a reviewer, and it is the same shape as `headwater generate --check` over a projection. It is also the form a gate runs, and the Consequences below name the two callers.

**The same script holds the tutorial page against the tutorial document.** Every command block and every output block on `site/tutorial/index.html` carries a class that marks it verbatim. The script reads each one back and fails when it is no longer in `docs/tutorials/your-first-governed-corpus.md`. So the page cannot drift from the document that `.claude/tutorial/fixtures.sh` already runs in CI.

**The discipline is one line.** Run the script before any commit that touches a page carrying a figure, and read what it prints.

**A figure that is hand-run is not a figure that is hand-typed, and that distinction is the whole ruling.** A hand-typed figure has no source, no date, and no way to be shown stale. A hand-run figure names the command that produced it, carries the date of the run, and fails a check when the two disagree.

## Consequences

**HW-DR-0037 is amended in one paragraph and not reopened.** The paragraph that reads "Only the linking form is available on this tree" now covers the deploy path alone. Everything else in that record stands. The boundary at `site/` stands, and so does the rule that a hand-built page states no figure a person typed. The statement that nothing checks it stands too.

**The check runs at two places, and a person is neither of them.** `.githooks/pre-commit` refuses a commit whose page disagrees with a fresh run, under `HEADWATER_SKIP_FIGURE_CHECK`. That clause announces the skip, so a bypass reaches the terminal of the author who takes it. The CI step named "The figures on the hand-built pages came from a run" reads the committed tree, and it carries no escape hatch. That step is the half that holds. Fourteen assertions of `.githooks/fixtures.sh`, across eight scenarios, hold it. They drive the three directions a derived figure moves in, the escape hatch, the denominator, the two governed pages and the partition itself.

**This record stated on 2026-08-30 that no hook and no job ran the script.** It named the reader of a pull request as the thing that held the rule. The pages carried a run of 2026-08-26, and nothing ran the script again for eleven days. 59 figure occurrences on three pages and 11 of the 43 verbatim blocks were adrift when it ran again. The reason that sentence gave stays true of the engine, which reads no file under `site/`. `.headwater/corpus.json` declares one corpus root, and `census.rs` reports a path that does not end in `.md` as not a document.

**`--check` compares the 26 figures that are a function of the corpus and the lock.** Eight are a function of the clock as well, and a difference in one of those is reported rather than failed. `run.date` is the clock itself. The seven others read the findings list, which two dated mechanisms move with no file touched. An adoption task lapses at `task.until < now`, and an escape directive expires at `suppression.until < now`. The page claims that these numbers came from a run on the date beside them, and that claim stays true.

**`headwater generate` already states this rule of its own `coverage_report` projection.** It prints the reason on every run: a coverage report is a function of the clock as well as of the corpus and the lock. A migration task lapses and a suppression expires on a date, so a committed copy fails a check on a morning when nothing changed. A figure derived from that report inherits the property. So this reading follows what the engine prints rather than adding to it.

**Measured on 2026-09-06, each arm on a tree that nothing else touched.** The clock at 2027-07-01 is past the `until` of the one adoption task, and it moves `findings.errors`, `findings.raised`, `findings.reported` and `rules.fired`. One escape directive, live on the day it was written and lapsed the day after its `until`, moves `findings.reported`, `findings.suppressed`, `findings.advisory` and `findings.directives`. The second arm is the one that matters, because `headwater check --strict` exits 0 on both sides of it. So a gate over these figures is the only thing that goes red on the morning a directive lapses. `until` is required on every directive, so every directive reaches that morning.

**A figure that reaches no page fails the run.** This record stated on 2026-08-30 that such a figure is reported. A renamed marker attribute, a moved page and an empty `site/` each left every figure unused, and the run still exited 0. [HW-DR-0050](0050-q50-where-the-visual-register-of-the-hand-built-pages-lives-now-that-eight-pages-each-carried-a-copy.md) refuses a page that opts out of the shared register rather than skipping it, and this is that discipline for a figure. All 34 measured figures reach a page on 2026-09-06, so the guard refuses nothing that stands today. It reads a union across pages, so a figure lost on one page alone stays invisible while another page carries it. The remedy is a declaration of what each page owes, and this script has nowhere to read one.

**This record governs the script and not the pages.** HW-DR-0037 already carries a `governs` edge for every page under `site/`. [HW-OBL-0104](../obligations/0104-a-governs-edge-reaches-the-path-it-names-and-nothing.md) prices a second edge onto one path at a second edge to maintain. The `constrains` edge above is what ties this ruling to the record it amends.

**A figure that no run produces stays off the page.** The benchmark row of the self-assessment page is the case, and it is present, empty and labeled. Principle 11 forbids a number that no run produced, and an empty row that says so is the honest form of that refusal.

**No date on a page under `site/` names a publication or a release.** A run date is a fact about a measurement that happened. The date on the terms page is a fact about a ratification, and it reads the same way. [HW-DR-0031](0031-q31-whether-this-repository-becomes-public-and-when.md) and [HW-OBL-0130](../obligations/0130-the-publication-date-of-this-repository-is-unset-and-the-owner-alone-sets-it.md) reserve the publication date to the owner, and nothing here reads a date off the board or off a milestone.

**What would retire this script.** A published artifact that a visitor can reach opens the linking form. A build command in the deploy path opens the interpolating form where HW-DR-0037 first looked for it. Either one makes this script the second-best answer rather than the only one, and neither is on this tree.
