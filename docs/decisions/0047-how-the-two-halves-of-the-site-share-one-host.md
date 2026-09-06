---
id: HW-DR-0047
status: draft
status_since: 2026-09-06
summary: "One directory holds both halves of the site, composed by `tools/assemble-site.sh` with the hand-built half last. The Cloudflare Workers Builds build command runs that script, which puts a build on the deploy path for the first time."
last_verified: 2026-09-06
title: "How the two halves of the site share one host"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0037
    - HW-DR-0038
    - HW-DR-0039
  governs:
    - tools/assemble-site.sh
    - tools/cloudflare-build.sh
    - wrangler.jsonc
    - site/_headers
---

# How the two halves of the site share one host

## Context

Two halves make up `https://headwater.tools/`. `site/` holds the hand-built pages, committed byte for byte, and [HW-DR-0037](0037-q37-which-parts-of-the-site-are-hand-built-and-which-are-a-projection-of-this-corpus.md) governs them by name. `mkdocs build --strict` renders the corpus into `.headwater/site-build`, which git ignores.

**The generated half reached no reader, which is the problem this record answers.** `/spec/`, `/decisions/`, `/obligations/`, `/tutorials/your-first-governed-corpus/` and `/corpus.json` all answered 404 on 2026-09-06. The build ran as a blocking step on every push, and nothing read the directory after the job ended.

**The owner ruled on 2026-09-04 that the generated half deploys before the repository becomes public.** That ruling settles whether, and this record settles how.

**Two mechanisms could serve it.** A GitHub Actions job could assemble the directory and call `wrangler deploy`, which needs two repository secrets and the retirement of the existing git integration. The Cloudflare Workers Builds build command could run the same assembly, which needs one field in a dashboard.

## Decision

**One directory holds both halves, and `tools/assemble-site.sh` composes it.** The script copies the generated half into `.headwater/site-deploy` first and the hand-built half second. A path that both halves carry is served with the committed bytes, and the script names every such path on each run.

**Every Cloudflare Workers Builds configuration runs that script.** A build command names one file, `tools/cloudflare-build.sh`, which installs the pinned site toolchain, runs the build, and runs the assembly. This project holds two such configurations, one for the default branch and one for every other branch. Each carries a build command of its own. A dashboard field carries no commit and goes stale in silence, so every version this deploy installs moves in a reviewed change instead. `wrangler.jsonc` names `.headwater/site-deploy` as the asset directory and states what writes it.

**The repository keeps one deploy path.** A second path in GitHub Actions would race the first, and the last writer would decide what a reader sees.

**The landing page is the root of both halves.** The generated half writes no root page, because `docs/` holds no index document. So `site/index.html` stands at the root with no collision to resolve.

**`site/` is never written by any of this.** The assembly reads that half. Two CI steps hold the rule, and `.githooks/pre-commit` refuses a commit that deletes a file there.

## Consequences

**Something now runs between the commit and the served bytes.** HW-DR-0037 states that nothing does, and that sentence measured a tree with no build command. [HW-DR-0039](0039-q39-how-a-figure-reaches-a-hand-built-page-now-that-a-build-interpolates-one.md) names a build command on the deploy path as one of the two events that retire `tools/refresh-figures.sh` as the only answer. Both sentences answer to this record.

**The build image carries the risk this repository cannot test.** The Cloudflare image must hold `python3` and `pip`. CI runs the same assembly on every push, so a break in the script is found by the runner that already builds the site. A break in the image is found by the deploy, which fails visibly and leaves the previous deploy live.

**A build configuration with no build command has no directory to serve.** The second configuration deploys a preview of each pull request, with `wrangler versions upload` in place of `wrangler deploy`. It reaches the same `wrangler.jsonc` and therefore the same asset directory, which only a build writes. A preview build that fails on every change reports nothing about any change, and a check that is always red is a check nobody reads.

**The order of the two acts is not free.** `wrangler.jsonc` names a directory that git ignores, so the build command has to exist before that name reaches the default branch. The reverse order serves a directory that is not there.

**A wide content policy reaches every generated page.** A generated page links its stylesheet, its script, its web fonts and its search index as files beside it. `default-src 'none'` serves such a page with none of them. `site/_headers` carries the wide policy as the default and re-asserts the narrow one on each hand-built path.

**A third-party origin serves script to most of the generated half.** [HW-OBL-0160](../obligations/0160-the-generated-half-of-the-site-loads-highlight-js-from-a-third-party-with-no-subresource-integrity.md) records that debt and names what discharges it.

**Every shelf root that holds a document serves an index page.** This record stated on 2026-09-06 that seven of them answered 404, because `.headwater/overlay.yml` declared an index projection for `spec`, `decisions` and `interfaces` alone. That sentence measured a tree with three declarations and the overlay now names ten shelves between its own entry and the base. The CI step that composes the two halves asserts a file at each root. It derives the shelf list from `.headwater/corpus.json` and `.headwater/taxonomy.lock`, so a shelf added later is covered by the push that adds it.

**A shelf root is reachable by address and absent from the navigation.** A generated index carries no front matter, so it is no node of the graph, and the `site_nav` emitter lists the nodes. MkDocs reports each of the ten as a page outside the navigation, at `info`, and `--strict` passes on it. [#536](https://github.com/headwater-ai/headwater/issues/536) carries the question of whether a shelf index belongs in the sidebar.
