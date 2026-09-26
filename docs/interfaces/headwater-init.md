---
id: HW-IFACE-headwater-init
status: current
status_since: 2026-09-06
summary: "How to start a corpus by writing a consumer declaration and an overlay from tree evidence and interview prompts."
last_verified: 2026-09-25
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

`--git` is the step that makes a merge safe for the derived files of the repository. It appends one line of the form `<path> -merge` to `.gitattributes` for each fold that a producer the tree holds writes. `headwater taxonomy resolve` writes `.headwater/taxonomy.lock`, which carries a digest over its whole text and is always a fold. `headwater generate` writes each file that carries the generated-file marker. Such a file is a fold only where its opening states a count or a digest. [`headwater derived`](headwater-derived.md) computes that shape. It keeps every other line of `.gitattributes` byte for byte, and it writes no line that the file already declares. So a second run writes nothing. The step asks `git check-attr` for the attributes. Where git does not answer, the step reads the root `.gitattributes` alone and reads no nested `.gitattributes`. A nested file that the step did not read can already declare a path below its directory. So the step writes no line for that path. It names each such path and the nested file on standard error. A `<path> merge=headwater-regenerate` line that an earlier release committed gets a `-merge` line after it, and git takes the later line.

**The committed line needs no configuration, and that is why it is `-merge`.** Under `-merge`, git keeps the current side, writes no conflict marker and records a conflict. Every clone does this. Git reads a driver that no configuration defines as an ordinary text merge ([#1058](https://github.com/headwater-ai/headwater/issues/1058)). So a committed `merge=headwater-regenerate` line gave a clone without the configuration a silent text merge of each fold.

**A forge does not read the attribute.** We measured this on GitHub on 2026-09-24. A pull request that moved a `-merge` path showed as mergeable, and its test merge held the edits of both branches. A check of the merged tree in CI is what covers the forge. [The adopter's CI guide](../how-to/wire-headwater-check-into-your-own-workflow.md#make-it-cover-a-merge) names the two branch settings that the cover needs.

**A generated file that is one record per entity takes no line.** Each row names one document, so two branches that each add a document write two rows. A text merge of them is what the producer writes over the merged tree. [#1058](https://github.com/headwater-ai/headwater/issues/1058) measured this over every generated file of the repository that maintains this engine. A `-merge` on such a file would stop every pair of branches that each add a document.

**The set is computed from the tree, so the step runs after the first `headwater generate`.** When the repository is first bound, no producer has written a file yet, and the set holds the lock alone. On a repository that is already bound, `--git` skips the declaration and the overlay and does the git step alone. Run it again when a producer starts to write a new file. [`headwater derived`](headwater-derived.md) names each producer output that carries no line.

**Only a producer that the tree holds gives a line.** The step reads the set that [`headwater derived`](headwater-derived.md) computes, and that set holds only the outputs of the producers the tree holds. Every tree holds the two verbs. The other two producers are a script and a test run of the repository that maintains this engine. A tree holds the script only where it carries that script, and the test run only where it carries the engine workspace. An adopter's tree usually holds neither, so the step writes a line for neither.

**The step prints the two `git config` lines and the override lines, and it writes them only under `--git-config`.** Git takes no merge driver from a repository without the consent of the clone. The lines are `git config merge.headwater-regenerate.name "regenerate a derived artifact"` and `git config merge.headwater-regenerate.driver "headwater merge-driver %O %A %B %P"`. The driver line names a verb of the binary and no file of the tree, so the step writes no shim. [`headwater merge-driver`](headwater-merge-driver.md) states what the driver does.

The override is one line of the form `<path> merge=headwater-regenerate` for each derived file, in the file that `git rev-parse --git-path info/attributes` names. That file wins over `.gitattributes` in this clone alone, so the conflict then names the command that rebuilds the file. The step writes the override only after both `git config` lines succeed. An override without the configuration names an undefined driver, and git then does a text merge again. In a linked worktree, git names the file in the common directory, so one override serves every worktree of the clone.

**A clone that already names the driver gets the override without `--git-config`.** An earlier release committed `merge=headwater-regenerate` and wrote no override. In a clone that ran that release with `--git-config`, the new `-merge` line would deselect the driver in silence. So where `git config merge.headwater-regenerate.driver` already has a value, the step writes the override too, and it says so. The migration from an earlier release is one command: run `headwater init --git` again in each configured clone.

**No step reaches a merge that comes before the override.** Git runs no hook before a merge. A clone can set the two `git config` lines by hand and merge before any run writes the override. That merge reads the committed `-merge` alone. Each fold conflicts with the current side in place, and no message names the producer. A repository whose hooks write the override on checkout, merge and commit has the same gap until the first of those hooks runs. Run `headwater init --git` after the `git config` lines and before the first merge.

## Preconditions

Without `--git`, the repository must not already contain `.headwater/taxonomy.yml`. Without `--corpus`, a subdirectory containing Markdown must exist. The selected package may be absent, but the command reports that the adopter must copy or vendor it before resolution.

## Options

| Option | What it does |
|---|---|
| `--corpus <dir>` | Names the corpus root instead of using tree evidence. |
| `--package <name>` | Names the package to consume. |
| `--git` | Appends the `-merge` lines to `.gitattributes`. Prints the two `git config` lines that name the driver and the override lines that select it. On a bound repository, it does this and nothing else. |
| `--git-config` | Also runs the two `git config` lines in this clone, and then appends the override lines to its `info/attributes`. It requires `--git`. |
| `--root <path>` | Selects the repository to initialize. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

## Exit status

**0** means that the declaration and overlay were written, and that the git step completed where `--git` asked for it.

**1** means one of five failures. The repository is already bound and `--git` is absent. No corpus root could be proposed. A file could not be written. A `git config` line failed, and then no override is written. Or `--git` ran inside a git repository and git did not give the merge attributes. The step then does its other work, prints the reason on standard error, and exits 1, as [`headwater derived`](headwater-derived.md) does.

**1**, and never 101, when standard output or standard error cannot be written, and one sentence on standard error names a failed standard output.

## Environment

The command reads no environment variable. Under `--git`, it runs `git` from the `PATH` for `git check-attr`, `git rev-parse` and `git config --get`. Under `--git-config`, it also runs `git config` to write the two lines.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.yml` | Written as the consumer declaration. |
| `.headwater/overlay.yml` | Written with interview prompts. |
| Repository directories | Read to propose the corpus root and package location. Under `--git`, read to find each file that carries the generated-file marker. |
| `.gitattributes` | Under `--git`, read, and appended to with one line for each derived file that it does not declare. |
| `.git/config` | Under `--git-config`, written by the two `git config` lines. |
| `info/attributes` in the git directory | Under `--git-config`, read, and appended to with one `merge=headwater-regenerate` line for each derived file that it does not select. Git names the path. Without `--git-config`, not written. |

## See also

[`headwater taxonomy resolve`](headwater-taxonomy.md) validates the declaration and writes the lock. [`headwater infer`](headwater-infer.md) reports the first adoption debt.
