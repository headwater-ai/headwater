---
id: HW-IFACE-headwater-init
status: current
status_since: 2026-09-06
summary: "How to start a corpus by writing a consumer declaration and an overlay from tree evidence and interview prompts."
last_verified: 2026-09-23
title: "headwater init"
relations:
  governs:
    - [engine/crates/cli/src/lib.rs, engine/crates/cli/src/main.rs]
---

# headwater init

## Synopsis

    headwater init [--corpus <dir>] [--package <name>] [--git [--git-config]] [--root <path>]

The command writes `.headwater/taxonomy.yml` and `.headwater/overlay.yml` for a repository with no consumer declaration. With `--git`, it also writes the `.gitattributes` lines that the merge driver needs.

## Description

It proposes the corpus root from the directory with the most Markdown files unless `--corpus` supplies one. It uses `headwater/standard` unless `--package` supplies a package name.

It writes questions that the tree cannot answer into the overlay. It does not resolve the taxonomy, fetch a package or write a lock.

### The git step

`--git` is the step that makes a merge safe for the derived files of the repository. It appends one line of the form `<path> merge=headwater-regenerate` to `.gitattributes` for each file that a producer the tree holds writes. `headwater taxonomy resolve` writes `.headwater/taxonomy.lock`, and `headwater generate` writes each file that carries the generated-file marker. It keeps every other line of `.gitattributes` byte for byte, and it writes no line that the file already declares. So a second run writes nothing.

**The set is computed from the tree, so the step runs after the first `headwater generate`.** When the repository is first bound, no producer has written a file yet, and the set holds the lock alone. On a repository that is already bound, `--git` skips the declaration and the overlay and does the git step alone. Run it again when a producer starts to write a new file. [`headwater derived`](headwater-derived.md) names each producer output that carries no line.

**Only a producer that the tree holds gives a line.** The step reads the set that [`headwater derived`](headwater-derived.md) computes, and that set holds only the outputs of the producers the tree holds. Every tree holds the two verbs. The other two producers are a script and a test run of the repository that maintains this engine. A tree holds the script only where it carries that script, and the test run only where it carries the engine workspace. An adopter's tree usually holds neither, so the step writes a line for neither.

**The step prints the two `git config` lines, and it runs them only under `--git-config`.** Git takes no merge driver from a repository without the consent of the clone. The lines are `git config merge.headwater-regenerate.name "regenerate a derived artifact"` and `git config merge.headwater-regenerate.driver "headwater merge-driver %O %A %B %P"`. The driver line names a verb of the binary and no file of the tree, so the step writes no shim. [`headwater merge-driver`](headwater-merge-driver.md) states what the driver does.

## Preconditions

Without `--git`, the repository must not already contain `.headwater/taxonomy.yml`. Without `--corpus`, a subdirectory containing Markdown must exist. The selected package may be absent, but the command reports that the adopter must copy or vendor it before resolution.

## Options

| Option | What it does |
|---|---|
| `--corpus <dir>` | Names the corpus root instead of using tree evidence. |
| `--package <name>` | Names the package to consume. |
| `--git` | Appends the `merge=headwater-regenerate` lines to `.gitattributes`, and prints the two `git config` lines that name the driver. On a bound repository, it does this and nothing else. |
| `--git-config` | Also runs the two `git config` lines in this clone. It requires `--git`. |
| `--root <path>` | Selects the repository to initialize. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

## Exit status

**0** means that the declaration and overlay were written, and that the git step completed where `--git` asked for it.

**1** means one of four failures. The repository is already bound and `--git` is absent. No corpus root could be proposed. A file could not be written. Or a `git config` line failed.

## Environment

The command reads no environment variable. Under `--git-config`, it runs `git` from the `PATH`.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.yml` | Written as the consumer declaration. |
| `.headwater/overlay.yml` | Written with interview prompts. |
| Repository directories | Read to propose the corpus root and package location. Under `--git`, read to find each file that carries the generated-file marker. |
| `.gitattributes` | Under `--git`, read, and appended to with one line for each derived file that it does not declare. |
| `.git/config` | Under `--git-config`, written by the two `git config` lines. |

## See also

[`headwater taxonomy resolve`](headwater-taxonomy.md) validates the declaration and writes the lock. [`headwater infer`](headwater-infer.md) reports the first adoption debt.
