---
id: HW-IFACE-headwater-change
status: draft
status_since: 2026-09-18
summary: "The one verb that shells out to git, and the manifest it writes for headwater check --change to read."
last_verified: 2026-09-18
title: "headwater change"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  evidence_basis: evidenced
relations:
  governs:
    - [engine/crates/cli/src/lib.rs, engine/crates/cli/src/main.rs]
    - engine/crates/vcs/src/lib.rs
---

# headwater change

## Synopsis

    headwater change <base-rev> <out-dir> [--root <path>] [--no-color] [--no-banner]

The verb takes two operands, in order. The first is the revision to compare the working tree against. The second is a directory to write into. Neither takes a flag spelling, because both are required.

## Description

`headwater change` writes the manifest that [`headwater check --change`](headwater-check.md) reads. It is the one verb of this binary that runs a version control command. The one crate it calls, `engine/crates/vcs`, is the one crate of the workspace that does. [Spec 12](../spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version) fixes the boundary this verb sits outside of. The check-evaluation path "runs no version control command, and it opens no file that the manifest does not name." A change manifest still has to come from somewhere. Until this verb existed, the only producer was `.githooks/change-manifest`, a shell script that belonged to this repository alone. [HW-DR-0072](../decisions/0072-the-binary-is-the-only-interface-an-adopter-must-run-and-every-integration-point-outside-it-is-declared.md) rules that gap a defect. An adopter with no shell script of their own could never reach a rule that reads a transition. <!-- headwater allow=surface.local_path.instructed scope=block until=2027-09-30 reason=accepted_deviation note=history of how this repository produced the manifest -->

The verb runs three git commands against `<base-rev>`. `git diff --name-status --find-renames` finds every document the working tree moved since that revision. `git show` reads the bytes each moved document held at that revision. `git ls-files --others` finds every document the working tree holds that the index does not, because a diff against a revision never reports one of those. The verb writes `<out-dir>/manifest`, in the grammar `headwater check --change` reads. It writes `<out-dir>/prior/<n>` as well, one file per document whose prior version the manifest names. The manifest's own path goes to standard output. The caller owns `<out-dir>` and removes it.

**`<base-rev>` is resolved against the git top level, not against `--root`.** A `--root` that names a subdirectory of a larger repository still anchors every path this verb writes at the repository root. That is the way `git diff` itself anchors its own paths. A corpus root that is not inside a git repository at all is refused.

**`<out-dir>` must sit outside the tree `<base-rev>` is compared against.** `git ls-files --others` finds every file the working tree holds that the index does not. A directory this verb is still writing into is exactly such a file. An `<out-dir>` under the corpus root is therefore named as a document the change itself adds. The commit gate and the CI job of this repository both avoid this with `mktemp -d`.

**No path is normalized, and no path is filtered.** A rename is named at the path the document arrived at. The path it left is the source of its prior bytes. A deleted document is named as a `prior` line rather than dropped, because the working tree holds no file there. That absence is the only evidence left of what stood there. A file that is no document of any corpus is still named. Filtering here would hide the one defect this verb can have. A path written in a form the census does not use reaches no row, and the report names it. Under a filter, that same path would be dropped before the check layer ever saw it. The run would then report a correct-looking nothing.

## Preconditions

**`<base-rev>` must resolve to a commit this clone holds.** A shallow clone that never fetched it is the usual cause. The verb refuses rather than write a manifest with a gap in it.

**The corpus named by `--root` must sit inside a git repository.** The verb resolves the repository's top level from that path. A path with no `.git` above it anywhere is refused.

**`git` must be on the host.** The verb runs a subprocess for each command named above. A host with none is refused rather than left to write an empty manifest.

**No path this verb would write may hold a tab.** A manifest line is tab separated, so a path holding one would produce a line the reader would mis-split. The verb refuses whole, before it writes anything.

## Options

| Option | What it does |
|---|---|
| `<base-rev>` | The revision to compare the working tree against. Spec 12 fixes two: the committed `HEAD` for a working-tree hook, and the merge base of a proposed change for a CI job. The option is required. |
| `<out-dir>` | The directory to write the manifest and the prior versions into. The caller owns it and removes it. This verb only ever creates inside it. The option is required. |
| `--root <path>` | The repository to read. It defaults to the working directory, and it is resolved to its git top level before any command runs. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

`--format` and `--json` are not options of this verb. Its one artifact is the manifest file. The one line this verb writes to standard output is that file's own path. `--wide` is refused, because the verb prints no help layout of its own beyond the two global screens. Global `--help`, `--version` and `--no-banner` are answered before the verb runs.

## Exit status

**0** means the manifest wrote, and the path printed on standard output names it.

**1** means one of four things. The command line named zero or one operand rather than two. Or `<base-rev>` does not resolve to a commit this clone holds. Or the corpus named by `--root` is not inside a git repository. Or a path this verb would write holds a tab. Each reason is one English sentence on standard error. Nothing is written to standard output, or into `<out-dir>`, on any of the four.

There is no third status.

**1**, and never 101, when standard output or standard error cannot be written, and one sentence on standard error names a failed standard output.

## Environment

No environment variable reaches this verb. The revision, the output directory and the repository all come from the command line. Every byte the manifest names comes from git.

## Files

| Path | How this verb treats it |
|---|---|
| `<out-dir>/manifest` | Written. The first line is `headwater change 1`, and every line after it is `added\t<path>` or `prior\t<path>\t<file>`. |
| `<out-dir>/prior/<n>` | Written, one file per `prior` line of the manifest, holding the bytes the named document held at `<base-rev>`. |
| The working tree under `--root` | Read, through `git diff`, `git show` and `git ls-files`. Never written. |

The verb writes no file of the corpus itself. It touches no cache and no claim store.

## See also

[`headwater check`](headwater-check.md) reads the manifest this verb writes, with `--change`.

[Spec 12](../spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version) fixes the two anchors `<base-rev>` may be. It also states the boundary this verb sits outside of. The check-evaluation path runs no version control command. A separate producer verb may, because it sits outside that loop.

[HW-DR-0072](../decisions/0072-the-binary-is-the-only-interface-an-adopter-must-run-and-every-integration-point-outside-it-is-declared.md) rules that git plumbing is one of two integration points that may sit outside the binary. It names this verb as what closes the gap it found.

`.githooks/change-manifest` is the caller this repository's own commit gate and CI job use. It resolves the built engine and hands its two arguments to this verb. It runs no git command of its own. <!-- headwater allow=surface.local_path.instructed scope=block until=2027-09-30 reason=accepted_deviation note=how this repository calls the verb -->
