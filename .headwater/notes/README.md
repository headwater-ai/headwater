# notes

Working notes. Not corpus, not site, and nothing here is a governed document.

A file lives here when all three of these hold. It is a point-in-time working input that a record already cites, so deleting it would leave a `traces_to` edge pointing at nothing. It is not for a reader outside this repository, so publishing it would be a leak rather than a projection. And it carries no `id`, no front matter and no kind, so the corpus cannot type it.

**Why under `.headwater/`.** This directory is already the one place a file sits outside the corpus root without any declaration saying so. [Spec 3](../../docs/spec/03-authoring-and-lifecycle.md) and [spec 7](../../docs/spec/07-distribution-and-federation.md) describe it in the same words: no census row covers it, no language regime binds it, and no rule reads it. A working note has exactly that status, so it needs no exclusion to get it. `coverage.unaccounted` counts a reading of a path the census never walked, and nothing reads these files, so no figure moves when one arrives or leaves.

**Why not `docs/`.** Two declarations would be needed to keep a file here private, and both would have to hold. `.headwater/taxonomy.yml` would need a `corpus.exclude` entry, or every run reports the file under the corpus root. `mkdocs.yml` would need an `exclude_docs` line, or the file is served at `https://headwater.tools/`. Each mechanism is in use and neither is exotic, and that is the objection rather than the answer: a file that belongs in neither the corpus nor the site should not be placed in the corpus and then declared out of it twice.

**Why not the repository root.** Nothing put it there except history. A reader scanning the root is looking for what the repository is, and every other entry there answers that question.

**Why not `site/`.** `tools/site/assemble-site.sh` copies `site/` whole into the directory Cloudflare serves, so every byte under it is on the air. That is the rule HW-DR-0037 states, and a file under `site/` that is not a page is published anyway. `website-design-brief.md` was served that way for two weeks, which is the measurement rather than the worry.

**What would retire this directory.** A kind and a shelf. The remedy for a file that nothing types is a declaration rather than a deviation, which is the rule `.headwater/taxonomy.yml` already states about the two untyped files under the corpus root. Give these documents a kind and they move into `docs/` and need none of the reasoning above.

**What cites this directory.** A `traces_to` edge from a decision record resolves a repository path through the source tree, so a path here is a target the engine reads and reports on. Moving a file out of this directory without moving the edge raises `relation.target.unresolved`.
