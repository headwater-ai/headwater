---
id: HW-OBL-0160
status: discharged
status_since: 2026-09-08
summary: "325 generated pages loaded highlight.js and two stylesheets from cdnjs with no integrity attribute. The theme option is off, a guarded color-mode script replaces the one that threw without those elements, and no served page names that origin."
last_verified: 2026-09-08
title: "The generated half of the site loads highlight.js from a third party with no subresource integrity"
waiting_on: build
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0047
---

# The generated half of the site loads highlight.js from a third party with no subresource integrity

## Context

**The built-in MkDocs theme loads a highlighter from another origin.** Every page it renders carries one script element and two stylesheet elements pointing at `https://cdnjs.cloudflare.com`. None of the three carries an `integrity` attribute, so nothing compares the bytes that arrive against bytes this project has read.

**277 of the 304 files the build writes carry those elements**, counted on 2026-09-06. No hand-built page under `site/` names any third-party origin.

**[HW-DR-0047](../decisions/0047-how-the-two-halves-of-the-site-share-one-host.md) admits the origin in the content policy**, because the alternative on that day was a page that reports three blocked requests to a visitor. The owner ruled on 2026-09-06 to admit it and record the debt.

**The subject of this corpus is where a claim comes from.** A site that argues for provenance, and runs script it cannot name the bytes of, states one thing and does another.

## Obligation

**The corpus owes the served pages a highlighter whose bytes it can name.** Either the three files sit under `site/` or beside the theme, or the elements carry an integrity attribute that a build computes.

**The obvious removal is not available.** `theme.highlightjs: false` deletes the two `hljs-light` and `hljs-dark` link elements, and `js/darkmode.js` reads both by identifier with no null guard. So the removal throws a `TypeError` on every generated page at load. Refusing the origin at the network keeps the elements and costs the coloring alone. That was measured, not reasoned.

**What the debt costs today is one origin and no coloring.** Refusing it in `site/_headers` is a one-line change, and it leaves every other capability of a generated page intact. The stylesheet, the web fonts and the search worker all resolve under `'self'`, measured against a served copy on 2026-09-06.

## Discharge

**No served page names an origin this repository does not control.** #566 took a third remedy rather than either of the two above. `mkdocs.yml` sets `theme.highlightjs: false`. `mkdocs/overrides/js/darkmode.js` shadows the theme's file, and it guards the two elements that the flag removes. `site/_headers` drops the origin from `script-src` and from `style-src`.

**Three files hold the string over the assembled directory on 2026-09-08.** They are the comment in `site/_headers`, this record's own rendered page, and the search index that reads it. No served page loads anything from that origin.

**The count in this record was 277, and it was 325 on the day it was paid.** The page population grew by 48 while the record stood. An absolute count of a moving corpus is stale from the day after it is written, which is HW-OBL-0142's subject.

**The coloring is gone, and that is the cost this remedy pays.** Vendoring the three files keeps it. That was refused: it puts a third party's minified bytes in this repository, to color a fenced block. `tools/site/check-site-console.py` carried one `ALLOWANCES` entry for that highlighter's unknown-language warning, and the population behind it is zero.

**Removal without the guard does not discharge this, and it was measured failing.** The theme emits `#hljs-light` and `#hljs-dark` only under `highlightjs`. Its own `js/darkmode.js` sets `.disabled` on both with no null check. So the configuration change on its own throws on every page: 325 pages with a finding, out of 333 served, at commit `0a7acf7`. `mkdocs build --strict`, `tools/site/assemble-site.sh` and `tools/site/check-site-fragments.py` each exit 0 over that same tree. `tools/site/site-console-fixtures.sh` holds the guard with a case that puts the unguarded line back.
