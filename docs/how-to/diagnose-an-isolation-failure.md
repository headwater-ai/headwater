---
id: HW-HOW-diagnose-an-isolation-failure
status: current
status_since: 2026-09-18
summary: "Five questions separate a failure of your change from a failure of the checkout it ran in, and each one is one command."
last_verified: 2026-09-18
title: "Diagnose an isolation failure"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-harness-support
  governs:
    - .githooks/pre-commit
    - .claude/hooks/lib.sh
    - tools/hw-cargo
    - tools/repo/retire-worktree.sh
---

# Diagnose an isolation failure

**Audience:** a contributor to this repository. An adopter of Headwater needs no step of this guide, and the consumer surface in `.headwater/overlay.yml` does not list it.

## Before you start

An isolation failure belongs to the checkout rather than to the change inside it. Several agent sessions build this repository at the same time. Each one works in a worktree under `.claude/worktrees/`. All of them share one clone, one set of git hooks, one pool of compiler output and one directory of shared state. A defect in that sharing arrives as a defect in your code. Two of them arrive as a pass.

You need a shell in the worktree where the failure happened. Step 1 tells you whether you also need a built engine.

[DEVELOPING.md](../../DEVELOPING.md), under the heading "Where state lives when several sessions run at once", states the four rules these steps test. [The isolation evaluation](../evaluations/one-clone-many-agents-how-this-repository-isolates-the-sessions-that-build-it.md) states why each rule is the shape it is. This guide asks the questions in the order that finds the most failures first.

Find your symptom, and go to the step beside it.

| what you saw | step |
|---|---|
| the commit gate passed and printed almost nothing | 1 |
| a check refused something your branch does not contain, or accepted something it does | 2 |
| a cargo failure quotes a path under `.claude/worktrees/` that is absent from the disk | 3 |
| `cargo test` exits 101 with `NotFound` that names the binary under test | 3 |
| `cannot find function` in a crate you did not edit, on a `dev-release` build alone | 3 |
| a verb reports no taxonomy, or reports a corpus you are not editing | 4 |
| your commits arrive on another branch, or inside another session's worktree | 5 |
| the retirement sweep reports `unmerged, holding ? commit(s)` | 5 |

## Steps

### 1. Ask whether this worktree has an engine

    ls -l engine/target/release/headwater engine/target/dev-release/headwater

A worktree that nobody has built in has neither. `.githooks/pre-commit` and all four hooks under `.claude/hooks/` fail open when they find no engine. That is deliberate, because the commit gate and the CI job sit downstream of them. The gate therefore exits 0 and checks nothing, and it prints one line that reads the same as the ordinary line a fresh clone prints. Do not read that silence as a pass.

Build the cheap profile, which every one of those positions accepts:

    sh tools/hw-cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked

When both profiles are present, the newer binary answers rather than the shipped one. A stale `release` binary beside a fresh `dev-release` one would otherwise check your change through an engine older than the change.

### 2. Ask which hook body ran

    git config --get core.hooksPath

An absolute answer names one checkout, and git keeps this key in `.git/config`, which every worktree of the clone shares. Every hook under `.githooks/` hands off to the copy under the worktree actually committing, whatever this value names ([#925](https://github.com/headwater-ai/headwater/issues/925), fixed by [#946](https://github.com/headwater-ai/headwater/pull/946)). `git rev-parse --show-toplevel`, evaluated inside the hook itself, finds that copy. The one case that still runs the wrong body is a branch with no copy of that hook at all, cut before the guard existed. The hook reports that fallback when it happens. Set the value relative anyway, once per clone, because `EnterWorktree` writes it absolute again on every call and the fallback case still needs it:

    git config core.hooksPath .githooks

### 3. Ask whether a compiler reused another checkout's output

Three failures share one cause, which is compiler output that belongs to a different tree.

**A failure that quotes a path under `.claude/worktrees/` that is absent from the disk.** A test that resolves a directory from `env!("CARGO_MANIFEST_DIR")` compiles that absolute path into the object. `sccache` shares one object cache across every checkout on the machine. The remedy names the crate and nothing wider:

    cargo clean -p <crate>

**`cannot find function` in a crate your change never touched, or a binary that does not do what your source says, under `--profile dev-release` alone.** Two worktrees built into one target directory, and cargo linked the other tree's copy of a workspace crate. The build can also exit 0 and link the other tree's code with no error. `tools/hw-cargo` pools one target directory for each slot, and it now recovers from this without your help. It records the worktree that built last in each slot. When a different worktree builds there, it makes every workspace crate of that tree compile again. `sh tools/hw-cargo-link-fixtures.sh` compiles two worktrees into one slot and holds that recovery. So you see this failure only from a build that did not go through `tools/hw-cargo`, such as a plain `cargo` with a shared `CARGO_TARGET_DIR`. A slot with no `.root` file is safe, because the tool reads a missing file as a different worktree. [#849](https://github.com/headwater-ai/headwater/issues/849) carries it, and the remedy is the same command with the profile named:

    cargo clean --manifest-path engine/Cargo.toml --profile dev-release -p <crate>

**`cargo test` that exits 101 with `NotFound` that names the binary under test.** Somebody removed the worktree while this shell held its working directory inside it. A removed directory that a process holds stays pinned on the disk with nothing in it. Confirm with `git worktree list`, whose output omits your tree, and start a new shell in a directory that exists.

The bare `cargo clean` is not a stronger form of any of these. It removes the output of every crate in the workspace, and the next build pays for all of it. Name the crate.

### 4. Ask which corpus answered

Every verb takes `--root`, and a verb without one reads the current directory. The current directory after a build is `engine/`, which holds no `.headwater/taxonomy.lock`, so the verb reports no taxonomy rather than reporting your corpus. Pass the root:

    engine/target/dev-release/headwater check --root .

Inside a hook the question has a second half. `CLAUDE_PROJECT_DIR` names the checkout the session started in and never moves, so it names the main checkout for every session that works in a worktree. The payload's own `cwd` member names the worktree. `.claude/hooks/lib.sh` separates the two: `hw_root` locates a binary, which is not a question about a corpus, and `hw_resolve_root` answers everything that is. A hook of your own that reports the wrong corpus is a hook that used the first where it needed the second.

### 5. Ask which tree you are actually in

    git rev-parse --show-toplevel
    git worktree list

An agent that another agent dispatched inherits the working directory of whoever dispatched it. A `git worktree add` that resolves its destination against that inherited directory builds inside a peer's tree. The commits of two issues then interleave on one branch. [#848](https://github.com/headwater-ai/headwater/issues/848) measures four occurrences in one run, all in the same directory. Resolve the destination from the main checkout, which is the parent of the directory `git rev-parse --git-common-dir` prints.

The retirement sweep answers this question about every tree at once:

    sh tools/repo/retire-worktree.sh

It reports and retires nothing without `--retire`. A `KEPT` line that reads `unmerged, holding ? commit(s)` says that the sweep lost a field while it parsed a worktree with no branch, which is [#847](https://github.com/headwater-ai/headwater/issues/847). It does not say that the tree holds work. Read the tip by hand before you act on that line.

## How to know it worked

Five answers, one for each step. The worktree holds an engine under `engine/target/`. Your worktree's own hook body ran, whatever `core.hooksPath` reads, and the pre-commit report on an absolute value says which case it was. A repeated cargo command fails the same way twice or passes, rather than failing on a path that no other command mentions. A verb names the corpus you are editing, in the count of documents it reports. `git rev-parse --show-toplevel` prints the tree whose branch `git status` names.

One thing these five cannot prove. A gate that passes in your worktree answers for your tree alone. CI runs the same checks against the merge of your branch with `main`. Two branches that write one derived value merge with no conflict, and leave a value that is true of neither. Rebase onto `main` before you bless a recorded artifact, and read the result of the merge rather than the result of the branch.
