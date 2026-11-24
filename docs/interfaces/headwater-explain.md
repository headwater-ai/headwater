---
id: HW-IFACE-headwater-explain
status: current
status_since: 2026-09-06
summary: "How a path or identifier yields the taxonomy derivation, requirements, relations, and graph edges."
last_verified: 2026-11-23
title: "headwater explain"
relations:
  governs:
    - [engine/crates/cli/src/lib.rs, engine/crates/cli/src/main.rs]
    - engine/crates/query/src/lib.rs
    - engine/crates/query/src/explain.rs
    - engine/crates/query/src/json.rs
    - engine/crates/graph/src/edges.rs
    - engine/crates/census/src/walk.rs
    - engine/crates/cli/tests/show.rs
    - engine/crates/cli/tests/explain_code_path.rs
---

# headwater explain

## Synopsis

    headwater explain <path|identifier> [--json] [--root <path>]

The target is a path under the corpus root, an identifier declared by a document, or a code file that a document governs.

A path can have the form that a shell or an editor writes ([#1227](https://github.com/headwater-ai/headwater/issues/1227)). `./x`, `a/../x` and an absolute path under the repository root find the same document as `x`. A relative path is relative to the repository root, which is `--root`. It is not relative to the working directory of the process. With `--root elsewhere`, `./x` is `elsewhere/x`. `headwater_census::walk::typed` reads the path for this verb, for [`headwater show`](headwater-show.md) and for the `explain`, `related` and `governing_docs_for_path` tools of [`headwater mcp`](headwater-mcp.md).

## Description

`headwater explain` prints the census derivation for one document, or the documents that govern one code file. For a document, it states the path, identifier, kind, derivation steps, purpose, summary, warrant, required facets, required sections, permitted relations and related graph edges.

An untyped document can still be explained. Its derivation states the step that stopped classification, and its kind-dependent fields are empty because no kind requires them.

The text report follows the order above. JSON carries the same fields in a document with its own shape version. A related edge states its direction, target, cue, governing end and the far document pointer where one exists. The relation name tells how the explained document stands to the far end, from both ends of the edge ([#1901](https://github.com/headwater-ai/headwater/issues/1901)). An outbound edge carries the key that the explained document wrote, the relation or its declared inverse. An inbound edge carries the other name of the pair. Where the far document wrote the inverse, that name is the relation. Otherwise it is the declared inverse, or `<relation> of` where the relation declares no inverse. Thus a successor always reads `supersedes` and the document it replaced always reads `superseded_by`, whichever of the two documents wrote the edge. In JSON, `inbound` tells which end wrote the key. Version `1.2` of the shape is the first to name a relation this way. A `1.1` document named every outbound edge by the relation, also where the author wrote the inverse. A `governs` edge onto a `code_path` anchor also states how many tree entries the anchor reaches, in total and for each pattern it holds ([HW-DR-0074](../decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md)). JSON carries this as a `reach` member with `total` and `members`. Each element of `members` holds `pattern`, `matched` and `paths`. `paths` is the list of tree entries that one pattern matched, sorted and with no duplicate. `matched` is the number of entries that the pattern matched. By default no bound applies to `paths`: it holds every entry that the pattern matched, and `matched` is its length ([#1260](https://github.com/headwater-ai/headwater/issues/1260)). At 5,000 files under one glob, the JSON document is 176,502 bytes and costs 10 ms of CPU time. At 50,000 files it is 1,751,505 bytes and costs 73 ms. [Spec 5](../spec/05-ai-integration.md#review-time-checks) names the command that measured these values on 2026-09-29. `--paths-at-most <n>` is the one exception. With it, `paths` holds only the first `n` entries in the same sorted order, and `matched`, `total` and `governed_entries` still count every entry ([#1346](https://github.com/headwater-ai/headwater/issues/1346)). So a caller that reads only the counts reads a document that does not grow with the tree. With a bound of 20 over the same 5,000 files, the document is 2,202 bytes (2026-09-30). The edit hook passes the bound for this reason. A pattern with no wildcard has one path, the pattern itself. Two `governs` edges of one document can reach one entry. So JSON also writes `governed_entries` at the top level: the number of distinct entries that all the outbound `governs` edges of the document reach. JSON writes it only where at least one such edge carries `reach`. The text report writes it in parentheses after the edge. JSON also writes a `targets` array on every related edge. For an anchor that binds, it holds each pattern as the resolver normalized it, sorted, because the sorted patterns are the identity of the anchor. For an anchor that binds nothing, it holds each member as the author wrote it, in the order the author wrote them. For a document target it holds the identifier, and for an inbound edge it holds the path of the far document. `target` is the members of `targets` joined by `, `, for a reader. A pattern that holds a comma is one member of `targets`, and nothing can split it out of `target`. `headwater json field related 0 targets 1` prints the second member of the first edge. An outbound edge onto a `source-tree` anchor also states its digest when the resolver states one ([#2016](https://github.com/headwater-ai/headwater/issues/2016)). The digest is the value that `headwater check --fix` and `headwater check --fix --baseline` write as `verified_revision` on the edge now. [The `headwater check` contract](headwater-check.md) states how it is derived. The text report writes it in parentheses after the reach: `content now <digest>`, then the recorded `verified_revision` or `none recorded`. A recorded digest that carries `verified_as: baseline` gets the words `as a baseline, which records no reading`. JSON carries this as a `digest` member. `current` is the digest the edge reaches now. `recorded` is the `verified_revision` of the edge, and `baseline` tells whether that edge carries `verified_as: baseline`. An edge with no `verified_revision` writes neither `recorded` nor `baseline`. An inbound edge, a document target and an anchor with no digest write no `digest` member. A literal that names a directory is such an anchor. The member is an addition to an element, so the shape version stays `1.3`. To print the digest, the verb reads every file that each such edge reaches.

**A code file is answered with the documents that govern it** ([#1968](https://github.com/headwater-ai/headwater/issues/1968)). A code file is a regular file inside the repository that is not a census row. A governing edge must reach it, through the reach that `Surface::governing_docs_for_path` reads. The text report states the path of the file on its first line. Then each governing document gets one line, `governed by` and the pointer that [`headwater route`](headwater-route.md) writes, with the bracket for an unaccepted warrant. The identifier of the document is on the line below its pointer. The documents are sorted by path. A path can reach its file through a symlink inside the root. Then the answer also names the documents that govern the file at the end of the link. JSON is the object `{"pointers": [...]}`, and each element is the pointer that `route` writes. It is the `structuredContent` of the `governing_docs_for_path` tool of [`headwater mcp`](headwater-mcp.md) for the same path. It has no `related` member. `--paths-at-most` is accepted and changes nothing in this answer.

This answer has three limits. A path with no file behind it is refused, also where a pattern of a `governs` edge reaches it. A file that no governing edge reaches is refused. [`headwater show`](headwater-show.md) refuses a code file, because it prints the bytes of a document. The `explain` tool of `headwater mcp` does not answer for a code file, and its `governing_docs_for_path` tool answers the same question.

The `warrant` line prints the declared value as written, and that includes a value outside the four values of [spec 3](../spec/03-authoring-and-lifecycle.md#the-warrant-and-what-each-value-requires). A document that declares no warrant, or a `warrant` key with no value, gets no `warrant` line. A related edge does not state the warrant of its far document. The bracket that marks a pointer to a document nobody accepted belongs to a pointer, and [`headwater route`](headwater-route.md) states its two wordings.

**The text report renders the palette [HW-DR-0045](../decisions/0045-coloring-the-cli-and-where-the-banner-goes.md) rules on, when standard output is a terminal.** The path is cyan, and the repeated labels — `kind`, `purpose`, `summary`, `warrant`, `requires the facets`, `requires the sections`, `may declare` — are dim, the same way on every run. JSON never colors: `--json` selects a machine format regardless of the stream. `--no-color` forces the plain text this verb already wrote before this decision.

## Preconditions

The repository must carry a readable `.headwater/taxonomy.lock`, consumer declaration and corpus. The target must resolve to a typed or untyped census row, or to a code file that a governing edge reaches.

A census row that the walk could not read is not a document, and `explain` refuses it ([#1366](https://github.com/headwater-ai/headwater/issues/1366)). Such a row is a symlink, an unreadable directory, a name that is not UTF-8, or a named pipe, a socket or a device. The census never opened the entry, so it knows no kind and no requirement. The refusal names the path and the reason on standard error, in the shape "`<path>` is <reason>, so `explain` prints nothing". Standard output stays empty, with or without `--json`. The `explain` tool of [`headwater mcp`](headwater-mcp.md) answers such a row with the same sentence.

An identifier resolves through the graph index. A target that matches no corpus path, no identifier and no governed code file is refused. Exit status states which of five things it named instead.

## Options

| Option | What it does |
|---|---|
| `<path|identifier>` | Select the document or the code file to explain. |
| `--json` | Write the explanation as machine-readable JSON. |
| `--paths-at-most <n>` | Write at most the first `n` entries of each `paths` list, sorted. `n` is a whole number above zero. The counts do not change. The option requires `--json`. Without it, the verb refuses the option with exit status 1 and prints nothing on standard output, because the text report lists no path. |
| `--root <path>` | Select the repository to read. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

`--format` is not an option of this verb. `--wide` is refused because the verb prints no help layout. Global `--help`, `--version` and `--no-banner` are answered before the verb runs.

## Exit status

**0** means that the target resolved and its explanation was printed. For a code file, the list of the documents that govern it was printed.

**1** means that the target was missing or was a census row that the walk could not read. It also means that the command line was invalid or the repository could not load. A missing target writes its refusal to standard error and no explanation to standard output.

**A missing target still classifies.** The refusal names one of five states. An existing file that no governing edge reaches gets the refusal of its state, and so does a path with no file.

| The state | What the refusal says |
|---|---|
| Shaped like a declared identifier scheme, and no document declares it | "is shaped like an identifier of this corpus, and no document declares it" |
| Under the corpus root, with no document there | "is a path of this corpus, with no document written there yet" |
| Under the corpus root, and an exclusion claims it | "is excluded by `<pattern>`" |
| Outside every corpus root this repository declares | "is outside every corpus root this repository declares" |
| Outside the repository. The path is absolute, climbs above the root with `..`, goes through a symlink out of the root, or is not UTF-8 | "is outside this repository, or is not a path it can read" |

The first row is checked before the other four, and it never runs `Corpus::classify`. `Shape::identifier_shaped` tests the target against the fixed prefix each declared `identifier_schemes` entry opens on. A pattern of `{namespace}-DR-{seq:04d}` opens on `HW-DR-`, for example. A target that opens that way is refused as an identifier, whatever the rest of it says. The last four states classify a path: one matcher decides them, `headwater_census::walk::Corpus::classify`, which the walk also uses for an existing file. Before that matcher runs, `headwater_census::walk::within` finds a path that goes through a symlink out of the root, and that path takes the last row. The corpus root is the one symlink that the walk follows, so a path under a linked corpus root stays inside. In this repository, a harness hook reads the second row and refuses a raw write there.

**The identifier state does not say whether the identifier is a typo or an invention.** `headwater explain HW-DR-9999` and `headwater explain HW-DR-004` both write "is shaped like an identifier of this corpus, and no document declares it" (measured 2026-09-23, repairing [#845](https://github.com/headwater-ai/headwater/issues/845)). Before that repair, both fell through to the fourth row instead. Both read "is outside every corpus root this repository declares" — true of a path, and irrelevant to an identifier (measured 2026-09-12). The `resolve_identifier` tool of [`headwater mcp`](headwater-mcp.md) is the read that separates a typo from an invention. It reports an identifier that no document carries at all. It also reports the path of an untyped document that carries the identifier. That match is exact, never a fuzzy one over case or separator. A caller that has to tell the two apart asks that tool rather than this verb.

**1**, and never 101, when standard output or standard error cannot be written, and one sentence on standard error names a failed standard output.

## Environment

No environment variable reaches this verb. The target and repository come from the command line, and the census and graph come from the tree.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.lock` | Read for the resolved taxonomy. |
| `.headwater/taxonomy.yml` | Read through the consumer loader. |
| The corpus | Read for census rows, front matter and graph edges. |
| `.headwater/imports/` | Read where `.headwater/taxonomy.yml` declares an import, for the anchors that an imported snapshot supplies. An `imports` entry that does not read stops the verb with exit 1, and so does an `at` path outside the repository root. |
| the path each `harvests.<name>.at` names | Read where `.headwater/taxonomy.yml` declares a pinned corpus export, for the anchors that export supplies. A `harvests` entry that does not read stops the verb with exit 1, and so does an `at` path outside the repository root. A pin with no digest binds no anchor. An absent file binds no anchor, and neither does a file that fails the pinned digest or is not an export. |

The verb writes no file.

## See also

[`headwater route`](headwater-route.md) selects pointers from a task description.

[Spec 2](../spec/02-taxonomy-model.md#kind-resolution) defines the derivation and the requirements this verb prints.

[`headwater mcp`](headwater-mcp.md) serves the same explanation through a JSON-RPC tool.
