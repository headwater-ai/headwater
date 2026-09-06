---
id: HW-OBL-0160
status: draft
status_since: 2026-09-06
summary: "277 generated pages load highlight.js and two stylesheets from cdnjs with no integrity attribute, so a third party supplies script to a site whose own subject is provenance."
last_verified: 2026-09-06
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

**This record discharges when no generated page names an origin this repository does not control.** Vendoring the three files into the theme's custom directory is the remedy that keeps the coloring. It needs a template override, because the elements come from the theme's own `base.html`.

**A weaker discharge is an integrity attribute on each of the three elements.** That names the bytes without moving them, and it still leaves the availability of the site tied to another host.

**What does not discharge this.** Admitting the origin in the content policy, which is what the tree does today. Removing the highlighter without a guard in `js/darkmode.js`, which trades a supply-chain debt for an exception on every page.
