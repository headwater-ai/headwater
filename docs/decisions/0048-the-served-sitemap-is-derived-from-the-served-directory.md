---
id: HW-DR-0048
status: draft
status_since: 2026-09-06
summary: "The sitemap a reader gets is derived from the directory that is served, by one walk that both halves call. Nobody types a page list, and the assembled directory is the only place the union exists."
last_verified: 2026-09-06
title: "The served sitemap is derived from the served directory"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  constrains:
    - HW-DR-0047
  traces_to:
    - HW-DR-0037
    - HW-DR-0047
  governs:
    - tools/sitemap.py
---

# The served sitemap is derived from the served directory

## Context

**Two sitemaps were about to be served as one, and the wrong one won.** [HW-DR-0047](0047-how-the-two-halves-of-the-site-share-one-host.md) rules that `tools/assemble-site.sh` copies the generated half first and the hand-built half second. A path that both halves carry is therefore served with the committed bytes. On 2026-09-06 that rule met its first collision. [#535](https://github.com/headwater-ai/headwater/issues/535) added `site/sitemap.xml`, and `mkdocs build` writes a `sitemap.xml` of its own on every run. [#554](https://github.com/headwater-ai/headwater/issues/554) is the report.

**Each of the two was wrong on its own, and for a different reason.** The hand-built one listed seven URLs that a person typed. `site/changelog/index.html` was already committed on the day that list landed. The list did not carry it, so the file was stale before it was pushed. The generated one carries zero URLs. The MkDocs template emits a `<loc>` element from a canonical URL alone. `mkdocs.yml` declares no `site_url`, so no page holds one, and no page contributes a line. So the rule HW-DR-0047 states would have told a search engine that this site has eight pages. The reverse rule would have told it that this site has none.

**A typed page list is what [HW-DR-0037](0037-q37-which-parts-of-the-site-are-hand-built-and-which-are-a-projection-of-this-corpus.md) forbids.** That record admits two forms where a number belongs on a hand-built page. The page links to the artifact that produced the number, or a build interpolates the number from a run. A list of URLs is a figure of the same kind, and it goes stale in the same silence. The changelog page carries no date for this reason.

**Neither half knows the answer.** The hand-built half knows its own eight pages and nothing of the 285 pages the corpus renders. The generated half knows the reverse. The assembled directory is the only place where both exist together.

## Decision

**A sitemap is derived from a directory of pages, and never typed.** `tools/sitemap.py` walks a directory and writes one `<loc>` element for each `index.html` under it. The URL comes from the path. The module carries no list, and a page that is added, renamed or removed moves the output on the next run.

**Two callers pass two directories, and there is one walk.** `tools/refresh-crawler-files.sh` passes `site/` and commits the result as `site/sitemap.xml`, beside the `llms.txt` and `robots.txt` it already derives from the same pages. `tools/assemble-site.sh` passes `.headwater/site-deploy` after it has copied both halves there, and it writes the union over whichever copy it finds. A reader gets the first while `wrangler.jsonc` names `./site`, and the second once that file names the assembled directory.

**`sitemap.xml` is the one path the assembly composes rather than copies.** HW-DR-0047 rules that the hand-built half wins a collision. This record names one exception, and it is the only file whose correct contents neither half holds. The assembly reports the exception on a line of its own, and it still names every other collision as a shadowing.

**`robots.txt` carries a `Sitemap:` line.** The target resolves on both sides of the switch HW-DR-0047 orders, because a sitemap stands at the root of each directory that is served. The line is derived like the rest of the file.

**`sitemap.xml.gz` is removed from the assembled directory.** `mkdocs build` writes it beside its own sitemap. Nothing links it, and a compressed second copy of a list is one more thing that can disagree with the list.

## Consequences

**The commit gate holds the committed sitemap, and it held nothing before.** `.githooks/pre-commit` already refuses a commit whose `site/llms.txt` or `site/robots.txt` disagrees with a fresh run. `site/sitemap.xml` joins them as a third entry in the same table. A CI step of the same name reads the committed tree. `.githooks/fixtures.sh` drives both, and it carries a case that adds a page under `site/` and reads the refusal. A retitle does not move a sitemap, because a title is not a URL, so that case adds a page rather than retitling one.

**The union is 293 URLs, and no step asserts the count.** 285 pages come from the generated half and 8 from the hand-built half, measured on 2026-09-06. The assembly prints the total on every run. A count in a CI step moves whenever a shelf gains a document. A step that a person re-blesses on ordinary work stops being read.

**The sitemap carries `<loc>` and nothing else.** A `<lastmod>` element needs a date. Two dates are available. One is a file modification time, which a fresh clone resets. The other is a date a person types, which is the failure this record answers. A search engine treats `<lastmod>` as a hint and never as a requirement.

**`mkdocs.yml` still declares no `site_url`, and that absence costs more than a sitemap.** The same setting is what puts a `<link rel="canonical">` element on a generated page, and 0 of 286 rendered pages carry one. This record needs no canonical URL, because it derives the sitemap from a directory instead. Whether the generated half should declare one is a question about duplicate content, and [#556](https://github.com/headwater-ai/headwater/issues/556) carries it.

**This record governs the script and not the three files it writes.** `site/llms.txt`, `site/robots.txt` and `site/sitemap.xml` stand outside every `governs` edge in this corpus. One case in `.githooks/fixtures.sh` needs a file under `site/` in exactly that position. It tests the escape hatch of the clause that refuses a deletion under `site/`. A governed path is refused a second time there, by a rule the engine already runs. So the three stay ungoverned on purpose. The edge here reaches `tools/sitemap.py`, which is where the rule lives.
