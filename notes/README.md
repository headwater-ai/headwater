# notes

Working notes. Not corpus, not site, and nothing here is a governed document.

A file lives here when all three of these hold. It is a point-in-time working input that a record already cites, so deleting it would leave a `traces_to` edge pointing at nothing. It is not for a reader outside this repository, so publishing it would be a leak rather than a projection. And it carries no `id`, no front matter and no kind, so the corpus cannot type it.

**Why not `docs/`.** `.headwater/corpus.json` sets the corpus root to `docs` and to nothing else, so a file placed under it is walked by every run. A file the taxonomy cannot type is reported as untyped on every run, forever. Two such files stand under `docs/` by choice — `docs/doctrine/maturity-model.md` and `docs/w3id/README.md` — and each one is named in prose that states the count. A third would move a hand-typed figure in `docs/spec/13-open-obligations.md` and in `docs/evaluations/specifying-the-engine.md` for no gain.

**Why not `site/`.** `tools/assemble-site.sh` copies `site/` whole into the directory Cloudflare serves, so every byte under it is on the air at `https://headwater.tools/`. That is the rule HW-DR-0037 states, and a file under `site/` that is not a page is published anyway.

**What cites this directory.** A `traces_to` edge from a decision record resolves a repository path through the source tree, so a path here is a target the engine reads and reports on. Moving a file out of this directory without moving the edge raises `relation.target.unresolved`.
