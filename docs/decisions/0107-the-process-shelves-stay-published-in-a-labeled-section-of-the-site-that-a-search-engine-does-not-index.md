---
id: HW-DR-0107
status: current
status_since: 2026-10-04
summary: "The five process shelves stay on the published site, in their own labeled section that carries a robots noindex. One `sections` member of `site_nav` declares the section, and dropping the pages was refused because public pages cite them"
last_verified: 2026-10-04
title: "The process shelves stay published, in a labeled section of the site that a search engine does not index"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: evidenced
relations:
  constrains:
    - HW-DR-0036
    - HW-DR-0037
    - HW-DR-0048
  governs:
    - engine/crates/generate/src/site_nav.rs
    - tools/site/check-site-noindex.py
    - tools/site/site-noindex-fixtures.sh
    - mkdocs/overrides/js/shelf-tabs.js
---

# The process shelves stay published, in a labeled section of the site that a search engine does not index

## Context

This repository keeps five process shelves under `docs/process/`: specifications, decisions, evaluations, obligation records and explanations. Their 56 documents record how Headwater itself is built and run. An adopter does not need them to use Headwater.

Before [#1681](https://github.com/headwater-ai/headwater/issues/1681), the published site showed these shelves among the product shelves. The shelf switcher listed "Process decisions" beside "Decision records", with nothing to tell a newcomer which was which. The site served each process page to search engines as a product page.

The issue said that `site/sitemap.xml` did not list the generated pages. That was true of the file, but no crawler reads it. `wrangler.jsonc` serves `.headwater/site-deploy`, and `tools/site/assemble-site.sh` writes the sitemap of that directory from every `index.html` in it. So the served sitemap listed all 56 process pages.

The issue offered two ways out: drop the process pages from the site, or keep them in a separate section. It also asked whether two `site_nav` projections could express the section. They cannot, for three reasons:

- Each `site_nav` writes its own file, and each file opens with `nav:`.
- `mkdocs.yml` reads the nav through `INHERIT`, which takes one parent file. MkDocs replaces a list whole when it merges, so a second `nav:` replaces the first.
- A labeled section is one more level of nesting, and the emitter wrote only a shelf and its pages.

The owner approved a design on 2026-10-04 from a mockup, as "option A2". The top links stay as they are. The shelf switcher becomes two labeled rows. "Reference" holds five group tabs, and "Project process" is a tinted row of the five process shelves. The owner then set the order of the groups and shelves: learn, then look up, then adopt, then audit.

## Decision

**The process pages stay published, in their own labeled section of the site, and every one of them carries `<meta name="robots" content="noindex">`.** A search engine then does not send an adopter to them. A reader who follows a link from a product page still reaches the page.

**Dropping the process pages from the site was refused, for three reasons:**

- 17 product documents outside `docs/process/` link to a process document. A dropped page makes each of those links a dead link on the site.
- `mkdocs.yml` raises `validation.nav.omitted_files` to `warn`, so `mkdocs build --strict` fails on a file under `docs/` that no nav entry names. To drop the shelves, the site must also add `process/` to `exclude_docs`. That list then mirrors the overlay by hand.
- Under `--strict`, `nav.not_found` fails on a nav entry whose page is not there. So the nav and `exclude_docs` must change together, or the build fails. Two hand edits that must agree are the defect a generated nav exists to remove.

**One declaration chooses the section: a `sections` member on the `site_nav` projection.** Meta-schema 0.11.0 adds it. A section has a `title`, and either `for` (its shelves) or `groups` (labeled lists of shelves). `noindex: true` asks for the robots meta on every page of its shelves. The order of sections, groups and shelves is the order the declaration writes. A shelf that no section names stays at the top level of the nav. The engine refuses `sections` on any other kind, a shelf named twice, and a section with both `for` and `groups`. The resolve refuses a shelf name that the taxonomy does not declare.

**The emitter writes the `noindex` list beside `nav:`, as `extra.headwater_noindex`.** It holds one path prefix for each shelf of a `noindex` section, relative to the corpus root. The prefix is the text of the shelf pattern before its first glob character, so every page of the shelf starts with it. For example, `docs/process/decisions/**` gives `process/decisions/`, and `docs/internal-*.md` gives `internal-`. MkDocs merges an `extra` mapping of the inherited file into the configuration, and the template reads it as `config.extra.headwater_noindex`. The other route was a template that finds the section by its title. That keys a crawler rule on a label that a person can rename, so it was not taken.

**This amends one clause of [HW-DR-0036](0036-q36-which-of-mkdocs-docusaurus-or-astro-this-corpus-emits-navigation-for-and-why.md#consequences): "The generated file carries `nav:` alone".** The file now carries `nav:`, and `extra.headwater_noindex` where a section asks for it. The reason the clause gives still holds. `plan()` derives every byte of the file from the taxonomy, and nothing that a person would add to a working `mkdocs.yml` is in it. A declaration with no `sections` writes a file with no `extra` key and no section level.

**This also amends the shape that [HW-DR-0036](0036-q36-which-of-mkdocs-docusaurus-or-astro-this-corpus-emits-navigation-for-and-why.md#context) states: "the nav a generator needs is two levels", and "one output file over flat groups".** Each shelf is still flat. A declaration with `sections` puts a section level above the shelves. A section with `groups` puts a group level between the section and its shelves. So the nav of this repository is four levels deep: section, group, shelf, document. A declaration with no `sections` writes the two levels that HW-DR-0036 describes.

**This amends one sentence of [HW-DR-0048](0048-the-served-sitemap-is-derived-from-the-served-directory.md): "`tools/site/sitemap.py` walks a directory and writes one `<loc>` element for each `index.html` under it".** The module now writes one `<loc>` element for each `index.html` that does not carry the robots `noindex` meta. The rule of that record still holds: the module reads each page, it carries no list, and the output follows the served directory.

**This constrains [HW-DR-0037](0037-q37-which-parts-of-the-site-are-hand-built-and-which-are-a-projection-of-this-corpus.md).** The generated half of the site is still a projection of the corpus. The section, its label and its `noindex` come from the overlay, and no hand-built page states them.

## Consequences

**The served sitemap lists no process page.** `tools/site/sitemap.py` leaves out each page whose `index.html` carries the robots `noindex` meta. It reads that from the page, so the sitemap is still derived from the served directory. On this branch, the assembled site has 614 pages, and its sitemap lists 558 of them. The 56 that it leaves out are the process pages.

**A check holds the crawler files.** `tools/site/check-site-noindex.py` reads the assembled directory. It fails when `sitemap.xml` or `llms.txt` lists a `noindex` page, or when a page under a `noindex` prefix has no meta. It also fails for each prefix that the nav declares and that no served page is under. `tools/site/site-noindex-fixtures.sh` causes each of those failures on a synthetic tree. CI runs both scripts after the assembly step.

**The template shows the sections as rows.** `mkdocs/overrides/main.html` finds the shelf of a page at any depth. It identifies a shelf as the item whose children are pages, and never by its label. A section of groups is a row of tabs, and each tab is a link to the first shelf of its group. `js/shelf-tabs.js` turns each tab into a toggle for its panel. With no script, every shelf is two clicks away. A shelf that no section names appears in an "Other" tab, so a new shelf cannot vanish from the switcher. Each page of a `noindex` section opens with a notice that links back to the specification.

**An adopter gets the nav section from the engine, and the `noindex` meta only from a template.** The adopter template at `integrations/site-generator/mkdocs.yml` names no `custom_dir`. So an adopter that declares a `noindex` section gets the nested nav and the `extra.headwater_noindex` list. Their own theme must read the list to emit the meta.

**The nav names the same files.** Before this change, `.headwater/nav.yml` names 604 files. After it, the nav names the same 604 files and this record, 605 in all. Only their nesting changed, so `validation.nav.omitted_files` reads the same set. [HW-OBL-0161](../obligations/0161-the-validation-block-of-mkdocs-yml-is-a-gate-this-repository-owns-and-no-fixture-drives.md) is still open: no fixture drives that block.

**Two things are out of scope.** The MkDocs search index still holds the process pages. A `robots.txt` rule would stop a crawler from fetching a page, and the crawler would then never see its `noindex`, so none was added.

**This opens again** if the site moves to a generator that does not read MkDocs's `extra` mapping, or if public pages stop citing process documents. In the second case, dropping the pages costs no dead link.
