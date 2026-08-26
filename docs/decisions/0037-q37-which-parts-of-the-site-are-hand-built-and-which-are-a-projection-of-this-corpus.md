---
id: HW-DR-0037
status: draft
status_since: 2026-08-26
summary: "Everything under `site/` is hand-built and the corpus is the generated projection. A hand-built page states no figure a person typed, and nothing checks that."
last_verified: 2026-08-26
title: "Q37 — Which parts of the site are hand-built and which are a projection of this corpus"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  constrains:
    - HW-DR-0016
  governs:
    - site/index.html
    - site/ns/index.html
    - site/how-it-works/index.html
    - site/compare/index.html
    - site/proof/index.html
    - site/tutorial/index.html
  traces_to:
    - site/DESIGN-BRIEF.md
---

# Q37 — Which parts of the site are hand-built and which are a projection of this corpus

## Context

**A site is live, and a merge to the default branch publishes it.** `site/` holds `index.html`, `ns/index.html`, `_headers` and `DESIGN-BRIEF.md`. `wrangler.jsonc` names `./site` as the asset directory of `https://headwater.tools/`, and its own comment states that Cloudflare runs `npx wrangler deploy` from the repository root on each push. [The w3id registration](../w3id/README.md) records that `https://w3id.org/headwater/` answers 302 and sends the client to `https://headwater.tools/ns/`. No staging step and no workflow stands between a merge and a reader.

**[Q16](0016-public-presence.md) ruled that the site is a projection of this corpus, and it stated the mechanism.** "Every number on the site comes from the evidence register, and a claim with no instrument is generated as unmeasured." The same entry adds that "a hand-written number on the site is then a finding, in the way that a hand-edited shelf index is."

**The owner amended that ruling on 2026-08-23, and no record held the amendment.** `site/DESIGN-BRIEF.md` opens with a table of decisions already taken. One row reads "Site architecture — Marketing hand-built, docs generated from the corpus". The source column marks that row "this amends Q16 and needs a decision record". Section 8 of the brief states the debt in its own words. It asks for two things. The first is which paths are hand-built. The second is the rule that a figure on such a page is "machine-inserted or linked to the run that produced it". This record is the record the brief asks for.

**The brief also demonstrates the cost of the rule it asks for.** Its section 3 types a table of run figures by hand, and reports 523 findings and 4 suppressions. A run of this engine over this corpus on 2026-08-26 reports 524 findings and 5 suppressions. Three days moved two of the numbers, in the one document that states why a hand-typed number is a defect.

## Decision

**The site has two halves, and a path is the boundary.** Everything under `site/` is hand-built. The generated projection is this corpus, rendered from the navigation that `.headwater/nav.yml` carries and the graph that `.headwater/export.json` carries. [Q36](0036-q36-which-of-mkdocs-docusaurus-or-astro-this-corpus-emits-navigation-for-and-why.md) picked the generator that reads the first of those two files.

**Q16 is amended and not reopened.** Its ruling that Headwater emits the navigation and that a third-party generator renders it stands whole. What narrows is the population that ruling covers. The projection is the corpus, and not the marketing pages that describe the corpus.

**A hand-built page states no figure a person typed.** Two forms are admitted where a number belongs on such a page. The page links to the generated artifact that produced the number, and a reader follows the link to the run. Or a build interpolates the number from a run, and the page carries no source of its own.

**Only the linking form is available on the deploy path, and two measurements say why.** `wrangler.jsonc` declares an asset directory and no build command, so nothing runs between the commit and the served bytes. `site/_headers` sets `Content-Security-Policy: default-src 'none'`, so a served page reaches no data file at read time either. [Q39](0039-q39-how-a-figure-reaches-a-hand-built-page-now-that-a-build-interpolates-one.md) narrows this sentence to the deploy path: the interpolating form runs on the author's side of the commit instead, where neither measurement applies.

**Nothing checks any of this, and the record states that rather than imply a gate.** `.headwater/corpus.json` declares one corpus root, `docs`, so `headwater check` reads no file under `site/`. A wider root would not reach these pages either. `census.rs` returns "not a document" for a path that does not end in `.md`, and every page under `site/` is HTML. What holds the rule is the reader of a pull request. This repository already carries a rule of that shape and states the same absence. `CLAUDE.md` requires an `## ELI5` section on every issue. It then states that no hook, no rule and no CI job reads an issue body.

**Three conditions would have to hold before a check could reach this rule, and none holds today.** A corpus root that admits `site/`, which [Q29](0029-q29-whether-a-corpus-root-may-contain-code-and-what-an-interface-contract-may-reach.md) already admits for code. A census that classifies an HTML file rather than reporting it as not a document. A rule that reads a digit in body text and asks for the link beside it. Each is an engine change or a taxonomy change, and this record proposes neither.

## Consequences

**This record declares no kind, no shelf and no facet.** A page under `site/` is on no shelf because it sits outside the corpus root, and not because a kind was chosen for it. Nothing under `packages/`, `taxonomy-source/`, `.headwater/overlay.yml` or `docs/taxonomies/` moves for this ruling.

**[HW-OBL-0097](../obligations/0097-whether-the-four-modes-of-di-taxis-are-the-kind-set.md) is untouched.** It asks whether the four modes of Diátaxis are the kind set for a documentation-site bundle, and it waits on an adopter. That question is about what ships to an adopter in a bundle. This record is about two halves of one repository's own site, so it neither answers that question nor prejudges it.

**No language rule of this repository reads one word a visitor sees.** The `ste_house` regime binds kinds, and `site/` carries no document of any kind. So the sentence limit, the retired terms, the spelling table and the voice categories reach none of the marketing prose. Section 7 of the brief states the same rules as a direction to a writer, and the writer is what holds them.

**A status band answers to the rule, because a status band is a claim about this repository.** The band states what a run or a query reports on the day it is written, and it carries no figure of its own. What the band may not carry is what two other records reserve.

**Nothing on a hand-built page names a publication date or a price.** [Q31](0031-q31-whether-this-repository-becomes-public-and-when.md) rules that this repository becomes public and leaves the date to the owner. [HW-OBL-0130](../obligations/0130-the-publication-date-of-this-repository-is-unset-and-the-owner-alone-sets-it.md) refuses a date that an agent proposes, that a run reads off the board, or that a milestone implies. [HW-OBL-0094](../obligations/0094-where-a-commercial-tier-could-sit-and-how-thin-the-trademark.md) holds where a commercial tier could sit at `waiting_on: ruling`. Section 8 of the brief records that the price of the audit is stated nowhere.

**This record governs each page by name, and the rule covers every file under `site/`.** A `governs` edge reaches the path it names and no path under it, which [HW-OBL-0104](../obligations/0104-a-governs-edge-reaches-the-path-it-names-and-nothing.md) prices. So an edge above names each page on this tree, and no edge names a directory. A page added under `site/` owes an edge of its own, in the commit that adds the page. A target with no file is `relation.target.unresolved`, which is an error, so the edge also refuses a later deletion of the page.
