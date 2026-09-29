---
id: HW-IFACE-headwater-show
status: current
status_since: 2026-09-27
summary: "headwater show prints the bytes of one document as they are on disk, found by path or identifier through the resolver that explain uses."
last_verified: 2026-09-29
title: "headwater show"
relations:
  governs:
    - [engine/crates/cli/src/lib.rs, engine/crates/cli/src/main.rs]
    - to: engine/crates/query/src/explain.rs
      verified_revision: sha256:785942bd8dd0c45ace2cac7220cdd31ecccee1575770b356dd86de98ebf822ce
    - to: engine/crates/census/src/walk.rs
      verified_revision: sha256:41331986b4a8f1942af52154cea911c0b70e7ec63199e470e1ee0b6b5cae3241
---

# headwater show

## Synopsis

    headwater show <path|identifier> [--root <path>]

The target is a path under the corpus root or an identifier declared by a document. A path can have the form that a shell or an editor writes, relative to the repository root, as the Synopsis section of [`headwater explain`](headwater-explain.md#synopsis) states.

## Description

`headwater show` prints the content of one document. It writes the bytes of the file exactly as they are on disk, and it writes nothing else. It does not decode the text, render the Markdown or add a newline at the end. A reader who redirects standard output to a file gets a copy of the document.

The verb finds the document with the same function that [`headwater explain`](headwater-explain.md) uses. A target that `explain` resolves, `show` resolves to the same file. A target that `explain` refuses, `show` refuses with the same sentence on standard error.

Use `explain` to learn what a document is. Use `show` to read what it says, by its identifier, when you do not know its path.

## Preconditions

The repository must carry a readable `.headwater/taxonomy.lock`, consumer declaration and corpus. The target must resolve to a typed or untyped census row. `show` refuses every row that the walk could not read, as the paragraphs under Exit status say. Such a row is a symlink, a named pipe, a socket or a device, an unreadable directory, or a name that is not UTF-8 ([#1366](https://github.com/headwater-ai/headwater/issues/1366)).

An identifier resolves only to a typed document. An untyped document resolves only by its path.

## Options

| Option | What it does |
|---|---|
| `<path|identifier>` | Select the document to print. |
| `--root <path>` | Select the repository to read. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. Standard output is the document's own bytes, so this flag changes only a refusal on standard error. |
| `--no-banner` | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

`--json` is not an option of this verb, because the output is the document and not a report about it. Global `--help`, `--version` and `--no-banner` are answered before the verb runs.

## Exit status

**0** means that the target resolved and the bytes of its file were written to standard output. Standard error is empty.

**1** means that the target was missing or is a row that the walk could not read, or that the file could not be read. It also means that the command line was invalid or the repository could not load. A missing target writes its refusal to standard error and no byte to standard output. A bare `headwater show` writes "`show` takes a path or an identifier".

**A missing target states one of five things.** The five states and their sentences are the table in the Exit status section of [`headwater explain`](headwater-explain.md#exit-status). `show` writes the same sentence as `explain` for each target, because one function writes the refusal for both verbs.

**A row that the walk could not read is refused in the sentence of `explain`.** `explain` refuses such a row, and `show` writes the same sentence with its own name as the verb. The sentence names the path and the reason, and it ends "so `show` prints nothing". For a symlink, the reason names the target of the link. `show` writes no byte to standard output ([#1366](https://github.com/headwater-ai/headwater/issues/1366)).

**A symlink is refused.** The census walk does not follow a symlink, and `show` does not follow one either. A symlink can also be a directory above the row. When the path to the document passes through one, `show` writes no byte to standard output. Standard error names the link in the sentence "is a symlink, which the walk does not follow, so `show` prints nothing". A link can name any file on the host, and this refusal keeps every read inside the root. A path that holds no document and that passes through a symlink out of the root gets a different refusal. It is outside the repository, and `show` writes the sentence that [`headwater explain`](headwater-explain.md) writes for that state ([#1249](https://github.com/headwater-ai/headwater/issues/1249)).

**A named pipe, a socket or a device is refused.** The census walk never opens such an entry, because a read of a named pipe that has no writer does not end. `show` does not open one either, and it writes no byte to standard output. Standard error names the path, and the sentence ends "which the census never opens, so `show` prints nothing" ([#1333](https://github.com/headwater-ai/headwater/issues/1333)).

**1**, and never 101, when standard output or standard error cannot be written, and one sentence on standard error names a failed standard output.

## Environment

No environment variable reaches this verb. The target and repository come from the command line, and the census and graph come from the tree.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.lock` | Read for the resolved taxonomy. |
| `.headwater/taxonomy.yml` | Read through the consumer loader. |
| The corpus | Read for census rows and identifiers. The resolved document is read as bytes. |

The verb writes no file.

## See also

[`headwater explain`](headwater-explain.md) states the kind, derivation and relations of the same document.

[`headwater route`](headwater-route.md) selects pointers from a task description.
