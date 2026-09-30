---
id: HW-IFACE-headwater-site
status: current
status_since: 2026-09-29
summary: "A built site held against its corpus. Four findings: a missing page, a stale page under a shelf, a dead link and a dead fragment."
last_verified: 2026-09-29
title: "headwater site"
relations:
  governs:
    - engine/crates/generate/src/site.rs
    - [engine/crates/cli/src/lib.rs, engine/crates/cli/src/main.rs]
---

# headwater site

## Synopsis

    headwater site <site-dir> [--root <path>]

The verb takes one argument, the directory that a site generator wrote. It holds that directory against the corpus at `--root`, and it prints one line for each finding.

## Description

This verb answers one question: does the built site agree with the corpus it was built from? A generator reads the corpus one time and writes HTML. After that, nothing reads the HTML again. Three defects then reach a reader. The navigation names a page that the build did not write. A page stays after its document left the corpus. A link points to a page that is not there. This repository had each defect. It had dead fragment links ([#431](https://github.com/headwater-ai/headwater/issues/431)). It had shelf indexes that the navigation did not name ([#528](https://github.com/headwater-ai/headwater/issues/528)). It had 17 links with no fragment that answered 404 ([#350](https://github.com/headwater-ai/headwater/issues/350)).

The verb reports four classes of finding. Each line starts with the class, then names the page or the source, then gives one sentence.

| Class | What it means |
|---|---|
| `site.page.missing` | The navigation names a source, and the site holds no page for it. The line names the source. |
| `site.page.stale` | An HTML page is under the directory of a declared shelf. No document of the corpus and no generated page is served at that path. The line names the page. |
| `site.link.dead` | An in-site `href` on a page resolves to no file of the site. A link with no fragment is read too. The line names the page and the `href`. |
| `site.fragment.dead` | An in-site `href` resolves to a page, and its fragment matches no `id` attribute on that page. The line names the page and the `href`. |

After the findings, one line counts the findings, the navigation entries, the pages and the in-site links that the run read.

### Where the navigation comes from

The verb builds the projection plan in memory, as `headwater generate` does, and it reads the paths that each `site_nav` projection names. It does not read the committed navigation file. A committed file that is stale is a finding of `headwater generate --check`, and this verb does not report it a second time.

### The layout, which is all the verb knows about a generator

For a source `a/b.md` under the corpus root, the page is `a/b/index.html` or `a/b.html`. For `a/README.md` and `a/index.md`, the page is `a/index.html`. A page is present if either form is present. MkDocs writes the first form with `use_directory_urls` and the second form without it. The verb names no generator, and it reads no configuration file of a generator.

A file that is not Markdown has no page, because a generator copies it and does not render it.

### What is a stale page

A page is stale when it is under the directory of a declared shelf and no source is served at its path. A source is a document of the corpus or a page that the plan writes, such as a shelf index. So a page is stale when its document was deleted, moved, or is not typed.

**No lifecycle state removes a page.** Every terminal state of the standard package is `terminal-retained`, so a superseded, deprecated or discharged document stays published. So to the verb, a page that is not published is a page whose source is not a document of the corpus.

A page outside the directory of every shelf is not in scope. Examples are the home page, `404.html`, a search page and the assets of a theme. A shelf whose directory is the corpus root itself gives no scope, because every page of the site is under it.

### What is an in-site link

The verb reads the `href` attribute of every tag on every page. An `href` is in-site when it has no scheme, does not start with `/`, and is not a bare `#`. A link that starts with `/` or `//` depends on the host and the path that serve the site. The directory does not state them. So the verb does not read such a link. It does not read a link that leaves the site.

An in-site link resolves against the URL of its page. The query part is removed. A path that ends in `/` resolves to `index.html` in that directory. A path to a directory without a final `/` also resolves to its `index.html`, because a server answers it that way after a redirect. A link that is only a fragment, such as `#usage`, points to its own page.

### How the verb reads HTML

The verb does not use an HTML parser. A small scanner reads the attributes of each tag: in double quotes, in single quotes, or bare. It skips comments and the bodies of `script` and `style`, because a string with the shape of an attribute there is not an attribute. It decodes `&amp;` in a URL and no other entity. It decodes `%XX` escapes in a path and in a fragment, because a file name and an `id` are written decoded.

### It opens no socket and writes nothing

The verb lives in the generate crate, which is in the checking loop. `engine/crates/cli/tests/network_boundary.rs` holds that no crate of that loop except `headwater-fetch` opens a socket. The verb writes no file, changes no artifact and changes no cache.

## Preconditions

The corpus at `--root` must resolve, as for every verb that reads a corpus. `headwater taxonomy resolve` writes the lock that it reads.

The taxonomy must declare a `site_nav` projection. Without one, no navigation names a page to look for, and the verb refuses. The base package declares none. Add the projection to the overlay:

    add_to:
      projections:
        - {kind: site_nav, output: .headwater/nav.yml}

The site directory must exist and hold at least one `.html` file. A generator must run first. The verb reads what the generator last wrote, and it does not run a generator.

## Options

| Option | What it does |
|---|---|
| `<site-dir>` | The directory that a site generator wrote, such as the output directory of MkDocs. A relative path is read from the current directory, and not from `--root`. |
| `--root <path>` | Select the repository whose corpus the site is held against. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. This report paints nothing. |
| `--no-banner` | Suppress the masthead. It is accepted here and does nothing, because only the root help screen prints one. |

The verb takes no option of its own. `--format` and `--json` are not options of this verb.

## Exit status

**0** means that the run found no finding.

**1** means one of two things. At least one finding is on standard output, or the verb refused and one sentence on standard error says why. The verb refuses in four cases. The `<site-dir>` argument is absent, the directory is not there, the directory holds no `.html` file, or the taxonomy declares no `site_nav` projection. A corpus that does not resolve is refused as it is for every verb that reads a corpus.

## Environment

The verb reads no environment variable itself. `NO_COLOR` and `HEADWATER_NO_BANNER` have the effect they have for every verb.

## Files

| Path | How this verb treats it |
|---|---|
| `<site-dir>/**` | Walked. Every `.html` file is a page. Every file is a target that a link can resolve to. |
| `.headwater/taxonomy.lock` | Read, for the shelves and the projections. |
| The corpus under the corpus root | Read, to find the documents and to build the plan. |
| `.headwater/imports/` | Read where `.headwater/taxonomy.yml` declares an import, for the anchors that an imported snapshot supplies. An `imports` entry that does not read stops the verb with exit 1, and so does an `at` path outside the repository root. |
| the path each `harvests.<name>.at` names | Read where `.headwater/taxonomy.yml` declares a pinned corpus export, for the anchors that export supplies. A `harvests` entry that does not read stops the verb with exit 1, and so does an `at` path outside the repository root. A pin with no digest binds no anchor. An absent file binds no anchor, and neither does a file that fails the pinned digest or is not an export. |
| The committed `site_nav` output, such as `.headwater/nav.yml` | Not read. The plan gives the same list. |

The verb writes no file.

## See also

[`headwater generate`](headwater-generate.md) writes the `site_nav` output, and `--check` holds the committed file against the plan.

[HW-DR-0047](../decisions/0047-how-the-two-halves-of-the-site-share-one-host.md) rules how the two halves of this repository's own site share one host.
