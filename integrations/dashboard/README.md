# Headwater corpus dashboard

This tool is for a person who wants to see the state of a documentation corpus without running a command line tool. Somebody on the team runs one command, and everybody else opens the page it writes in a browser.

The page has three views:

- **Staleness.** Every document, sorted by the date it was last verified, oldest first.
- **Warrant.** Every document, grouped by how far it is established: `asserted`, `proposed`, `accepted`, and so on. A document that states no warrant shows as "none stated" in its own group. The page never shows such a document as `asserted`.
- **Coverage.** Every code path that a document governs, with the documents that govern it. When you give the tool a checkout, the page also shows how many of that checkout's files the governed paths cover.

## Run it

It needs Python 3.7 or later and nothing else. 3.7 is the first version with `date.fromisoformat` and the `capture_output` argument of `subprocess.run`, which the tool uses. It does not need the Headwater engine.

    python3 integrations/dashboard/dashboard.py --export .headwater/export.json --tree . --out dashboard.html

- `--export` is the `json` target of a `graph_export` profile, at `export_version` 1.1 or later. This repository's own `.headwater/export.json` is the worked example, and the tests read it. The tool refuses an earlier version.
- `--tree` is optional. Give it a checkout of the commit that the export was generated from. In a git checkout the file list is `git ls-files`. In any other directory, it is every file except those under `.git`. Without `--tree`, the coverage view lists the governed paths and states no share.
- `--out` is the page to write. The default is `dashboard.html` in the current directory.
- `--corpus-identity` names the corpus that the export belongs to. The default is `local`.

If the export is not a valid `graph_export` file, the tool refuses it with exit status 2. It prints one line that names the document and the field, for example ``dashboard: graph.documents[0] (FX-DR-0002): `warrant` is list, not a string`` for the fixture below with a list in its `warrant`. It does not write a page in that case, so a lead never reads a partial page as a complete one. A `last_verified` value must be a real calendar date in the form `YYYY-MM-DD`. The tool refuses `2026-02-30`, `last tuesday`, `20260115` and `2026-W03-4` on every Python version, because either value would take a false place in the staleness order.

An export that holds no documents is valid. The tool writes the page, and the page says at the top that the export holds no documents.

## How the page reaches a person who runs no command

One person runs the command, and every other reader only opens a file. There are two ways to do this:

- **A person who already works in the repository** runs the command after `headwater generate` writes a new `.headwater/export.json`. That person then puts `dashboard.html` where the team can open it, for example a shared drive, a wiki attachment, or a static web server. The page is one HTML file that loads nothing from the network, so you can copy it anywhere. A browser opens it directly from disk.
- **A CI job** runs the same command on each push to the default branch and publishes `dashboard.html` as a build artifact or to a static site. The lead opens the link. Nobody runs a command by hand.

This repository does not publish the page. It gives you the tool only. Where a team puts the page is the team's decision.

## Where each value comes from

| view | field in the export |
|---|---|
| staleness | `graph.documents[].facets.last_verified`. The export has no top-level `last_verified`. |
| warrant | `graph.documents[].warrant`. When the key is absent, the page shows "none stated". |
| coverage | `graph.edges[]` whose `relation` is `governs` and whose target is a `code_path` anchor. The paths are `target.patterns` when the export writes it, and `target.id` when it does not. |

A document can govern a list of paths as one entry, such as `[src/a.rs, src/b.rs]`. The export writes that entry as one anchor. Its `id` is a key and not a path, and its `patterns` holds the paths (export version 1.2 or later). The page shows one row for the list, names every path in it, and counts the files that any of the paths match.

A code path is an exact file, a directory, or a glob, where `*` stays inside one path segment and `**` crosses segments. A path that matches no file in the tree shows 0 files matched. This is often a sign that the path in the document is wrong.

## It is read-only

The tool reads the export file and, with `--tree`, lists the files of the checkout. It writes one file, the page that `--out` names. It runs no Headwater verb: no `new`, no `fix`, no `taxonomy migrate --apply`, and no equivalent. It commits nothing. This is the same query-class boundary that [spec 5](../../docs/spec/05-ai-integration.md#what-the-server-may-do-and-the-axis-that-decides-it) draws for the MCP server.

## One corpus, keyed from the start

[HW-DR-0080](../../docs/decisions/0080-q66-the-corpus-dashboard-ships-free-and-self-hosted-with-tenant-isolation-stated-as-a-data-model-constraint-from-day-one.md) rules that this dashboard is a free, self-hosted tool. It also rules that every row is keyed by `(corpus_identity, id)` from the first commit. Every document row and every code path row in `dashboard.py` carries that pair. A page shows one corpus, so it never displays the identity. The tool does not decide who allocates a corpus identity. [HW-OBL-0026](../../docs/obligations/0026-the-operational-shape-of-a-hosted-server-is-unstated.md) still owes that.

## Tests

    python3 -m unittest discover -s integrations/dashboard -p 'test_*.py'

`fixtures/two-documents.json` is the decisive fixture. Both of its documents carry `last_verified` only under `facets`, and one of them has no `warrant` key. A view that reads the wrong field writes an empty page, and a view that fills in a default shows `asserted`. Both defects fail the tests. `tools/repo/integrations-fixtures.sh` runs these tests in CI.
