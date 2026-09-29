---
id: HW-HOW-publish-your-corpus-as-a-site
status: current
status_since: 2026-09-28
summary: "MkDocs builds a site from the navigation that headwater generate writes, and headwater site then holds the built site against the corpus."
last_verified: 2026-09-29
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

4. Add a home page, if your corpus root has no `index.md`. MkDocs makes `index.md` in `docs_dir` the home page, and each page of the site links to the home page. Without the home page, each of these links goes to a page that the build did not write, and step 8 shows each one. The command below writes a home page in `docs`. Use your corpus root in place of `docs`, and replace the text with your own.

```sh
printf '# My corpus\n' > docs/index.md
```

5. Install MkDocs. When your Python refuses to install a package outside a virtual environment, create and activate one first.

```sh
python3 -m pip install mkdocs==1.6.1
```

6. Build the site. Always use `--strict`. Without it, MkDocs reports a navigation entry for a missing page as a warning and exits 0.

```sh
mkdocs build --strict
```

7. Commit `mkdocs.yml`, the home page, `.headwater/overlay.yml`, `.headwater/taxonomy.lock` and `.headwater/nav.yml`. Do not commit the directory that MkDocs writes the pages to. Tell Git to ignore it.

<!-- headwater allow=surface.local_path.instructed scope=block until=2027-09-30 reason=false_positive note=site/ here is the output directory MkDocs writes in the repository of the adopter, which is the site generator integration point -->

```sh
printf 'site/\n' >> .gitignore
```

Commit `.headwater/nav.yml` with the other generated files, and do not generate it only in the build step. `headwater generate --check` fails when the file is not on the tree, and the conformance rule `projections.current` is then not met, so your corpus falls below L2. `headwater check --strict` still passes without it, so the check alone does not tell you. When two branches each add a document, their copies of `.headwater/nav.yml` can conflict. Run `headwater generate` on the merged tree to write the file again, and do not merge it by hand.

8. Compare the built site with your corpus. Do this step after each build, and do it last. The command reads the directory that MkDocs writes the pages to. When you set `site_dir` in `mkdocs.yml`, give that directory as the last word of the command.

<!-- headwater allow=surface.local_path.instructed scope=block until=2027-09-30 reason=false_positive note=site here is the output directory MkDocs writes in the repository of the adopter, which is the site generator integration point -->

```sh
headwater site site
```

The command exits 0 when the site agrees with the corpus. Otherwise it writes one line for each problem and exits 1. A line can show a page that the navigation names and the build did not write. A line can also show a page that has no document in the corpus now. A third type of line shows a link or a fragment that points to nothing in the built site. [The contract of the command](../interfaces/headwater-site.md) gives each type of line and what the command reads. Correct the corpus or the build. Then do step 2 again if you added, moved or removed a document, and do steps 6 and 8 again.

## How to know it worked

`mkdocs build --strict` exits 0, and each document on a shelf has a page in the output directory.

`headwater generate --check` exits 0. When it does not, the navigation is older than your corpus. Run `headwater generate` and commit the result.

The command of step 8 exits 0.

When a document leaves the corpus and `.headwater/nav.yml` still names it, `mkdocs build --strict` exits with a non-zero status and names the missing path. Run `headwater generate` to remove the entry.
