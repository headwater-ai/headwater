---
id: HW-IFACE-headwater-infer
status: current
status_since: 2026-09-06
summary: "How to report uncovered findings as adoption tasks and optionally append them to the taxonomy lock."
last_verified: 2026-08-25
title: "headwater infer"
relations:
  governs:
    - [engine/crates/cli/src/lib.rs, engine/crates/cli/src/main.rs]
    - .headwater/taxonomy.lock
---

# headwater infer

## Synopsis

    headwater infer [--owner <name>] [--until <date>] [--write] [--now <date>] [--root <path>]

The command reports findings that no existing adoption task accounts for, grouped into document and rule pairs. It also proposes a shelf and a kind for each directory that the tree already writes and that no shelf claims, and it names each file that the proposal leaves untyped.

## Description

Without `--write`, the command prints the proposed payload and changes nothing. With `--write`, it appends new tasks to the authored adoption block in `.headwater/taxonomy.lock`.

It mints identifiers above every declared one, deduplicates pairs by document and rule, and prints what it carried through. A second run with the same inputs proposes no new task.

### The overlay fragment

The section `what this taxonomy does not explain` prints a candidate overlay fragment from the same read of the tree. Each line of the fragment goes under the `add:` key of `.headwater/overlay.yml` without an edit, and `headwater taxonomy resolve` then accepts it. The verb never writes the fragment. An author reads it, edits it if necessary, and puts it in the overlay.

The grouping rule:

- The fragment reads only the files that no declared shelf claims.
- It groups those files by their parent directory.
- It proposes one shelf and one kind for each directory that holds two or more documents with front matter.

For each such directory, the fragment declares four things:

- A shelf whose path is `<directory>/*.md`, that is homogeneous and that names the kind. The path claims the Markdown files of that directory and no subdirectory.
- A kind that is a `governed_document`. It requires each front-matter key that every document of the group writes, and it permits each key that only some documents write. It requires each heading that every document of the group writes, at any level.
- A purpose for the kind. Its intent and its phrase are a first draft, because a tree does not state what a document is for.
- A facet for each front-matter key that the bound taxonomy does not already declare, at `volatility: mutable`.

Each block opens with a `#` comment that names its evidence: the directory, the number of documents, the keys and the headings that it read.

The slug rule: the shelf, the kind and the purpose of one directory take one name. The name is the last segment of the directory path, in lowercase, with each character that is not a letter or a digit changed to `_`. Where that name is already declared, or another directory gives the same name, the name is the whole directory path, changed in the same way. Where that name is also taken, the verb adds `_2`, `_3` and so on.

### What the proposal does not explain

After the fragment, the section names each file that stays untyped with the fragment in the overlay, and one reason for each file:

- `no front matter`: the file opens with no front-matter block.
- `alone in its directory`: the file is the only document with front matter in its directory, so it is no evidence of a kind.
- `unreadable`: the engine could not read the file.
- `ambiguous shelf`: two shelves of equal specificity claim the path.
- `discriminator`: a heterogeneous shelf claims the path and its discriminator does not name a kind.
- `abstract kind`: the shelf or the discriminator names an abstract kind.

Where no file is outside every shelf and every file classified, the section prints one line that says that there is nothing to propose.

## Preconditions

The repository must load through its committed taxonomy lock and corpus. On a tree with no `.headwater/` directory, run `headwater init` and `headwater taxonomy resolve` first. The host date must exist unless `--now` supplies one. `--until` cannot be before the evaluation date.

`--write` requires `--owner`. The lock's adoption block must be readable if it exists.

## Options

| Option | What it does |
|---|---|
| `--owner <name>` | Names the person or team that owns proposed debt. Required with `--write`. |
| `--until <date>` | Sets the task expiry date. It defaults to ninety days after the evaluation date. |
| `--write` | Appends proposed tasks to the lock. |
| `--now <date>` | Sets the evaluation date in `YYYY-MM-DD` form. |
| `--root <path>` | Selects the repository to load. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

## Exit status

**0** means that the report completed. Findings in the report do not change this status.

**1** means that the repository, date, options or adoption block could not be read or written.

**1** also means that the payload this run built did not load. That is a defect in this engine, because the verb escapes every value it writes.

**1**, and never 101, when standard output or standard error cannot be written, and one sentence on standard error names a failed standard output.

## Environment

The command reads the system date when `--now` is absent. It reads no other environment variable.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.lock` and corpus | Read to run checks and inspect existing adoption tasks. |
| `.headwater/imports/` | Read where `.headwater/taxonomy.yml` declares an import, for the anchors that an imported snapshot supplies. An `imports` entry that does not read stops the verb with exit 1, and so does an `at` path outside the repository root. |
| the path each `harvests.<name>.at` names | Read where `.headwater/taxonomy.yml` declares a pinned corpus export, for the anchors that export supplies. A `harvests` entry that does not read stops the verb with exit 1, and so does an `at` path outside the repository root. A pin with no digest binds no anchor. An absent file binds no anchor, and neither does a file that fails the pinned digest or is not an export. |
| `.headwater/taxonomy.lock` | Written only with `--write`, by appending proposed tasks. |

## See also

[`headwater check`](headwater-check.md) produces the findings. [`headwater taxonomy resolve`](headwater-taxonomy.md) refreshes the lock after source changes.
