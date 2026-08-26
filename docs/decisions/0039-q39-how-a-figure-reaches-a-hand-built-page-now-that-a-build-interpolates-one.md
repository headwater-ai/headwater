---
id: HW-DR-0039
status: draft
status_since: 2026-08-26
summary: "The interpolating form that HW-DR-0037 admits runs before the commit rather than after it, and a script writes every figure on a hand-built page from a run."
last_verified: 2026-08-26
title: "Q39 — How a figure reaches a hand-built page, now that a build interpolates one"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  constrains:
    - HW-DR-0037
  governs:
    - site/scripts/refresh-figures.sh
  traces_to:
    - site/DESIGN-BRIEF.md
---

# Q39 — How a figure reaches a hand-built page, now that a build interpolates one

## Context

**[HW-DR-0037](0037-q37-which-parts-of-the-site-are-hand-built-and-which-are-a-projection-of-this-corpus.md) admits two forms where a number belongs on a hand-built page, and it closed both.** The first form is a link to the generated artifact that produced the number. The second is a build that interpolates the number, so that the page carries no source of its own.

**The record measures the second form as unavailable, and the measurement is about the deploy path.** `wrangler.jsonc` declares an asset directory and no build command, so nothing runs between the commit and the served bytes. `site/_headers` sets `default-src 'none'`, so a served page reaches no data file at read time either. Both readings are still true on this tree, and this record disturbs neither.

**The first form is unavailable for a different reason, which [#435](https://github.com/headwater-ai/headwater/issues/435) states.** The repository is private, and no workflow publishes a built artifact anywhere a visitor can reach. So a link to a run resolves to nothing for the reader it was written for.

**Both admitted forms were therefore closed, and a page of figures had nowhere to stand.** The self-assessment page waited behind that, and the landing page carried figures from a run that nobody could name.

**The cost of the gap is measured rather than argued.** `site/DESIGN-BRIEF.md` section 3 states a table of run figures typed by hand on 2026-08-23. It reports 245 files under the corpus root, 523 findings, and taxonomy `headwater/standard 3.2.0`. A run of this engine on 2026-08-26 reports 332 files, 524 findings, and `headwater/standard 3.3.0`. Three days moved every figure in the one document that states why a hand-typed figure is a defect.

## Decision

**The interpolating form runs before the commit rather than after it.** HW-DR-0037 forecloses a build between the commit and the served bytes. It forecloses nothing on the machine of the person who writes the page. So the interpolation happens there, and the interpolated figure is in the bytes that the commit carries.

**`site/scripts/refresh-figures.sh` is that build.** It runs the engine over this repository, reads the result, and rewrites the text of every element under `site/` that carries a `data-figure` attribute. A key with no measurement fails the run, and a measurement that reaches no page is reported.

**Five sources supply every figure, and each one is a run.** `headwater check --json` gives the census, the findings, the rules wired, the taxonomy and the clock. The text output of the same verb gives the census lines that the JSON omits, and the obligation register. `docs/interfaces/README.md` gives the verb count and the group count, and `headwater generate --check` holds that file. `engine/crates/generate/src/profile.rs` gives the emitter split, read off the `Emitter` values and the arms of `is_built`. `headwater conformance` gives the level this repository reaches.

**Every figure states its denominator in the prose beside it.** A count with no stated population is a trap for the next reader, and this repository has paid for that more than once. The script also refuses to finish when the census partition does not add up to the file count it reports.

**`--check` writes nothing and exits non-zero when a page and a fresh run disagree.** That is the form to put in front of a reviewer, and it is the same shape as `headwater generate --check` over a projection.

**The same script holds the tutorial page against the tutorial document.** Every command block and every output block on `site/tutorial/index.html` carries a class that marks it verbatim. The script reads each one back and fails when it is no longer in `docs/tutorials/your-first-governed-corpus.md`. So the page cannot drift from the document that `.claude/tutorial/fixtures.sh` already runs in CI.

**The discipline is one line.** Run the script before any commit that touches a page carrying a figure, and read what it prints.

**A figure that is hand-run is not a figure that is hand-typed, and that distinction is the whole ruling.** A hand-typed figure has no source, no date, and no way to be shown stale. A hand-run figure names the command that produced it, carries the date of the run, and fails a check when the two disagree.

## Consequences

**HW-DR-0037 is amended in one paragraph and not reopened.** The paragraph that reads "Only the linking form is available on this tree" now covers the deploy path alone. Everything else in that record stands. The boundary at `site/` stands, and so does the rule that a hand-built page states no figure a person typed. The statement that nothing checks it stands too.

**Nothing gates this, and the record states that rather than imply a gate.** No hook runs the script, and no job runs it, for the same reason that no rule reads a file under `site/`. `.headwater/corpus.json` declares one corpus root, `docs`, and `census.rs` reports a path that does not end in `.md` as not a document. What holds this rule is the reader of a pull request, which is the answer HW-DR-0037 already gives for its own.

**This record governs the script and not the pages.** HW-DR-0037 already carries a `governs` edge for every page under `site/`. [HW-OBL-0104](../obligations/0104-a-governs-edge-reaches-the-path-it-names-and-nothing.md) prices a second edge onto one path at a second edge to maintain. The `constrains` edge above is what ties this ruling to the record it amends.

**A figure that no run produces stays off the page.** The benchmark row of the self-assessment page is the case, and it is present, empty and labeled. Principle 11 forbids a number that no run produced, and an empty row that says so is the honest form of that refusal.

**No date on a page under `site/` names a publication or a release.** A run date is a fact about a measurement that happened. The date on the terms page is a fact about a ratification, and it reads the same way. [HW-DR-0031](0031-q31-whether-this-repository-becomes-public-and-when.md) and [HW-OBL-0130](../obligations/0130-the-publication-date-of-this-repository-is-unset-and-the-owner-alone-sets-it.md) reserve the publication date to the owner, and nothing here reads a date off the board or off a milestone.

**What would retire this script.** A published artifact that a visitor can reach opens the linking form. A build command in the deploy path opens the interpolating form where HW-DR-0037 first looked for it. Either one makes this script the second-best answer rather than the only one, and neither is on this tree.
