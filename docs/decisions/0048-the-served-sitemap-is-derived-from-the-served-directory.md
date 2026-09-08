---
id: HW-DR-0048
status: current
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
    - tools/refresh-crawler-files.sh
---

# The served sitemap is derived from the served directory

## Context

**Two sitemaps were about to be served as one, and the wrong one won.** [HW-DR-0047](0047-how-the-two-halves-of-the-site-share-one-host.md) rules that `tools/assemble-site.sh` copies the generated half first and the hand-built half second. A path that both halves carry is therefore served with the committed bytes. On 2026-09-06 that rule met its first collision. [#535](https://github.com/headwater-ai/headwater/issues/535) added `site/sitemap.xml`, and `mkdocs build` writes a `sitemap.xml` of its own on every run. [#554](https://github.com/headwater-ai/headwater/issues/554) is the report.

**Each of the two was wrong on its own, and for a different reason.** The hand-built one listed seven URLs that a person typed. `site/changelog/index.html` was already committed on the day that list landed. The list did not carry it, so the file was stale before it was pushed. The generated one carries zero URLs. The MkDocs template emits a `<loc>` element from a canonical URL alone. `mkdocs.yml` declares no `site_url`, so no page holds one, and no page contributes a line. So the rule HW-DR-0047 states would have told a search engine that this site has eight pages. The reverse rule would have told it that this site has none.

**A typed page list is what [HW-DR-0037](0037-q37-which-parts-of-the-site-are-hand-built-and-which-are-a-projection-of-this-corpus.md) forbids.** That record admits two forms where a number belongs on a hand-built page. The page links to the artifact that produced the number, or a build interpolates the number from a run. A list of URLs is a figure of the same kind, and it goes stale in the same silence. The changelog page carries no date for this reason.

**Neither half knows the answer.** The hand-built half knows its own eight pages and nothing of the 286 pages the corpus renders. The generated half knows the reverse. The assembled directory is the only place where both exist together.

## Decision

**A sitemap is derived from a directory of pages, and never typed.** `tools/sitemap.py` walks a directory and writes one `<loc>` element for each `index.html` under it. The URL comes from the path. The module carries no list, and a page that is added, renamed or removed moves the output on the next run.

**Two callers pass two directories, and there is one walk.** `tools/refresh-crawler-files.sh` passes `site/` and commits the result as `site/sitemap.xml`, beside the `llms.txt` and `robots.txt` it already derives from the same pages. `tools/assemble-site.sh` passes `.headwater/site-deploy` after it has copied both halves there, and it writes the union over whichever copy it finds. A reader gets the second, because `wrangler.jsonc` names the assembled directory. It named `./site` when this record was drafted, and [#558](https://github.com/headwater-ai/headwater/pull/558) made the switch HW-DR-0047 ordered. The first copy stays derived and stays committed, because it is what the `Sitemap:` line resolves to if that name ever moves back.

**`sitemap.xml` is the one path the assembly composes rather than copies.** HW-DR-0047 rules that the hand-built half wins a collision. This record names one exception, and it is the only file whose correct contents neither half holds. The assembly reports the exception on a line of its own, and it still names every other collision as a shadowing.

**`robots.txt` carries a `Sitemap:` line.** The target resolves whichever directory is served, because a sitemap stands at the root of each of them. The line is derived like the rest of the file.

**`sitemap.xml.gz` is removed from the assembled directory.** `mkdocs build` writes it beside its own sitemap. Nothing links it, and a compressed second copy of a list is one more thing that can disagree with the list.

## Consequences

**The commit gate holds the committed sitemap, and it held nothing before.** `.githooks/pre-commit` already refuses a commit whose `site/llms.txt` or `site/robots.txt` disagrees with a fresh run. `site/sitemap.xml` joins them as a third entry in the same table. A CI step of the same name reads the committed tree. `.githooks/fixtures.sh` drives both, and it carries a case that adds a page under `site/` and reads the refusal. A retitle does not move a sitemap, because a title is not a URL, so that case adds a page rather than retitling one.

**No step asserts the assembled URL count.** `tools/assemble-site.sh` prints the total on every run, and a reader who needs the number runs it. A count in a CI step moves whenever a shelf gains a document. A step that a person re-blesses on ordinary work stops being read. This paragraph carried a total until 2026-09-08. The assembly reported a larger number two days after that total was measured.

**The sitemap carries `<loc>` and nothing else.** A `<lastmod>` element needs a date. Two dates are available. One is a file modification time, which a fresh clone resets. The other is a date a person types, which is the failure this record answers. A search engine treats `<lastmod>` as a hint and never as a requirement.

**`mkdocs.yml` declares a `site_url`, and that setting does more than fill a sitemap.** The same setting puts a canonical link element on a generated page, and every generated page except `404.html` carries one. This paragraph names that element in words and not in its markup. A page of this corpus that spelled it out would answer a search for it. This record needs no canonical URL, because it derives the sitemap from a directory instead. [#556](https://github.com/headwater-ai/headwater/issues/556) is the report, and `tools/check-site-canonical.py` holds the element against the URL each page is served at.

**This record governs the two scripts and not the three files they write.** `site/llms.txt`, `site/robots.txt` and `site/sitemap.xml` stand outside every `governs` edge in this corpus. One case in `.githooks/fixtures.sh` needs a file under `site/` in exactly that position. It tests the escape hatch of the clause that refuses a deletion under `site/`. A governed path is refused a second time there, by a rule the engine already runs. So the three stay ungoverned on purpose. The edges here reach `tools/sitemap.py`, which holds the rule, and `tools/refresh-crawler-files.sh`, which no record governed while it wrote two of the files. HW-DR-0047 already governs `tools/assemble-site.sh`, which is the other caller.
