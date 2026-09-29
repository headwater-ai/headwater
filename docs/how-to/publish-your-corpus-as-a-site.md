---
id: HW-HOW-publish-your-corpus-as-a-site
status: current
status_since: 2026-09-28
summary: "MkDocs builds a site from a governed corpus through the navigation that headwater generate writes, and a strict build fails on a missing page."
last_verified: 2026-09-28
title: "Publish your corpus as a site"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-vision-and-scope
    - HW-DR-0036
    - HW-DR-0077
  governs:
    - integrations/site-generator/**
---

# Publish your corpus as a site

**Audience:** an adopter of Headwater, in a repository of their own. The consumer surface in `.headwater/overlay.yml` lists this guide, and `headwater check` holds it to that list.

Headwater does not render a site. It writes the navigation of your corpus in the shape that MkDocs reads, and MkDocs builds the pages. [HW-DR-0077](../decisions/0077-the-consumer-surface-is-what-an-adopter-receives-runs-and-must-have-installed-and-it-is-a-closed-and-declared-list.md) makes the site generator a declared integration point that is not part of the mandatory surface.

## Before you start

You need a corpus that `headwater check --strict` passes. The tutorial [Your first governed corpus](../tutorials/your-first-governed-corpus.md) ends with one.

You need Python 3 and MkDocs 1.6.1. MkDocs installs PyYAML, Jinja2 and Markdown as its own dependencies. These programs belong to the site generator and not to Headwater. The `headwater` binary does not need them, and no conformance level asks for them.

**Other generators.** Headwater writes navigation for MkDocs alone. It writes nothing for Docusaurus or Astro. Docusaurus reads its sidebar from a code module, and Astro has no native navigation format. [HW-DR-0036](../decisions/0036-q36-which-of-mkdocs-docusaurus-or-astro-this-corpus-emits-navigation-for-and-why.md) records why that decides the choice. For another generator, read the reading order from `.headwater/nav.yml` and convert it yourself.

## Steps

1. Declare the navigation in your overlay. The base package declares no `site_nav` projection, so `headwater generate` writes no navigation until your overlay adds one. Add these lines at the end of `.headwater/overlay.yml`. If your overlay already has an `add_to:` key, do not add a second one, because the resolve refuses a duplicate key. Put `projections:` under the key you have, or add the list item to its `projections:` list.

```yaml
add_to:
  projections:
    - kind: site_nav
      output: .headwater/nav.yml
```

2. Resolve the taxonomy and generate the projections. The first command writes the lock, and the second command writes `.headwater/nav.yml`.

```sh
headwater taxonomy resolve
headwater generate
```

3. Copy the configuration [`integrations/site-generator/mkdocs.yml`](https://github.com/headwater-ai/headwater/blob/main/integrations/site-generator/mkdocs.yml) from `headwater-ai/headwater` to the root of your repository, next to `.headwater/`. Set `site_name` to the name of your site. Set `docs_dir` to the `corpus.root` value in `.headwater/taxonomy.yml`. MkDocs refuses a `docs_dir` that is the directory of `mkdocs.yml`, so this recipe needs a corpus root below the repository root, for example `docs`. The file has no `nav:` of its own, because `INHERIT` reads it from `.headwater/nav.yml`.

4. Install MkDocs. When your Python refuses to install a package outside a virtual environment, create and activate one first.

```sh
python3 -m pip install mkdocs==1.6.1
```

5. Build the site. Always use `--strict`. Without it, MkDocs reports a navigation entry for a missing page as a warning and exits 0.

```sh
mkdocs build --strict
```

6. Commit `mkdocs.yml`, `.headwater/overlay.yml`, `.headwater/taxonomy.lock` and `.headwater/nav.yml`. Do not commit the directory that MkDocs writes the pages to. Tell Git to ignore it.

<!-- headwater allow=surface.local_path.instructed scope=block until=2027-09-30 reason=false_positive note=site/ here is the output directory MkDocs writes in the repository of the adopter, which is the site generator integration point -->

```sh
printf 'site/\n' >> .gitignore
```

Commit `.headwater/nav.yml` with the other generated files, and do not generate it only in the build step. `headwater generate --check` fails when the file is not on the tree, and the conformance rule `projections.current` is then not met, so your corpus falls below L2. `headwater check --strict` still passes without it, so the check alone does not tell you. When two branches each add a document, their copies of `.headwater/nav.yml` can conflict. Run `headwater generate` on the merged tree to write the file again, and do not merge it by hand.

## How to know it worked

`mkdocs build --strict` exits 0, and each document on a shelf has a page in the output directory.

`headwater generate --check` exits 0. When it does not, the navigation is older than your corpus. Run `headwater generate` and commit the result.

When a document leaves the corpus and `.headwater/nav.yml` still names it, `mkdocs build --strict` exits with a non-zero status and names the missing path. Run `headwater generate` to remove the entry.
