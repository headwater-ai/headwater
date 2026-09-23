---
id: HW-IFACE-headwater-merge-driver
status: current
status_since: 2026-09-23
summary: "How git refuses the merge of a derived fold through a verb that keeps the current side and names the producer that rebuilds it."
last_verified: 2026-09-23
title: "headwater merge-driver"
relations:
  governs:
    - [engine/crates/cli/src/lib.rs, engine/crates/cli/src/main.rs]
---

# headwater merge-driver

## Synopsis

    headwater merge-driver <ancestor> <current> <other> <path>

Git runs the verb, and a person does not. The configuration names it as `headwater merge-driver %O %A %B %P`, and [`headwater init --git`](headwater-init.md) prints that configuration.

## Description

A derived artifact that holds a fold states one value over the whole corpus. Two branches that each move the fold write two values. A three-way merge of the two gives a value that is true of neither tree. [HW-DR-0049](../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md) rules that such a file is rebuilt after a merge and never reconciled.

`.gitattributes` gives each such path `merge=headwater-regenerate`, and git then calls this verb for the path. The verb does three things:

1. It leaves the current side (`%A`) byte for byte. Git reads the merge result from that file.
2. It writes to standard error the path and the command that rebuilds it. The command is `headwater taxonomy resolve` for `.headwater/taxonomy.lock` and `headwater generate` for every other path.
3. It exits 1, so git records the path as conflicted.

Git writes no conflict marker for a custom driver. So the file stays readable, and the resolution is one deterministic step: finish the merge, run the command, and stage the result. Every generated file records the digest of the lock. So when the lock is conflicted too, run `headwater taxonomy resolve` before `headwater generate`.

**The verb does not rebuild the file, although the driver name says "regenerate".** Git runs a driver during the merge, one path at a time. At that moment the other paths of the tree are not all merged. A fold that is rebuilt then describes a tree that never existed, and it looks correct. So the verb names the producer and does not run it.

**The verb does not break the rule of [spec 5](../spec/05-ai-integration.md) that no hook introduces a verb.** [HW-DR-0077](../decisions/0077-the-consumer-surface-is-what-an-adopter-receives-runs-and-must-have-installed-and-it-is-a-closed-and-declared-list.md) gives the reason. That rule forbids a second entry point to `route` or to `check`. This verb answers a question that no other verb answers, which is what a merge does with a derived fold.

### What the driver cannot reach

**Git calls no driver when the two branches wrote the same bytes.** A driver is a content merge. Git compares the two blobs first, and it resolves a path that has one blob at the tree level. Two branches that each add one document of one kind can write one identical fold. That merge gives a value true of neither tree, and git does not call this verb. `engine/crates/census/tests/merge_driver.rs` measures this case.

The check on the merged tree is what reaches that case, and the driver is its fallback. `headwater generate --check` and `headwater taxonomy resolve --check` are that check. An adopter runs them after a merge and in CI. This repository also runs them from a commit hook while a merge is in progress. No verb carries that hook, because HW-DR-0077 gives its exception to the driver alone, and a second verb for a hook needs its own ruling.

## Preconditions

Git must find `headwater` on the `PATH` that it runs with. The verb reads no corpus and no lock, so it runs on a tree in the middle of a merge.

Git calls the verb only for a path whose attribute is `merge=headwater-regenerate`, in a clone whose configuration names the driver. `headwater init --git` writes the attribute lines and prints the two configuration lines. `headwater init --git --git-config` also runs them.

## Options

| Option | What it does |
|---|---|
| `<ancestor>` | The file that holds the version of the common ancestor, `%O`. The verb does not read it. |
| `<current>` | The file that holds the version of the current side, `%A`. The verb does not change it. |
| `<other>` | The file that holds the version of the other side, `%B`. The verb does not read it. |
| `<path>` | The path of the artifact in the tree, `%P`. It selects the command that the message names. |
| `--root <path>` | Accepted and not read. The verb reads no corpus. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

## Exit status

**1** means that the path is left conflicted with the current side in place. Git reads it as a conflict. It is the only status that the verb gives. A call with fewer than four operands also exits 1, and it names the form that git uses. Git always supplies all four.

## Environment

No environment variable reaches this verb. Git finds the binary through `PATH`, and the verb reads nothing from its environment.

## Files

| Path | How this verb treats it |
|---|---|
| The `<current>` file | Left byte for byte. Git reads the merge result from it. |
| The `<ancestor>` and `<other>` files | Not read. |
| `.headwater/taxonomy.lock` | Named in the message, with `headwater taxonomy resolve`, when it is the `<path>`. |
| Any other `<path>` | Named in the message, with `headwater generate`. |

The verb writes no file, and nothing on standard output, because standard output during a merge belongs to git.

## See also

[`headwater init`](headwater-init.md) writes the `.gitattributes` lines and prints the configuration that names this verb.

[`headwater derived`](headwater-derived.md) computes which files a producer writes, and it reports a producer output that carries no `merge=headwater-regenerate`.

[HW-DR-0049](../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md) rules that a fold is derived and never stored. [HW-DR-0077](../decisions/0077-the-consumer-surface-is-what-an-adopter-receives-runs-and-must-have-installed-and-it-is-a-closed-and-declared-list.md) rules that merge safety ships as this verb.

[Spec 6](../spec/06-engine-architecture.md#cli) lists the command line.
