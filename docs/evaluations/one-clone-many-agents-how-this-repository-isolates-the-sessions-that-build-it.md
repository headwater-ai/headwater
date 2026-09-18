---
id: HW-EVAL-one-clone-many-agents-how-this-repository-isolates-the-sessions-that-build-it
status: current
status_since: 2026-09-18
summary: "Fifteen agent sessions share one clone through four rules, and each rule was written after a failure that read as a pass."
last_verified: 2026-09-18
title: "One clone, many agents: how this repository isolates the sessions that build it"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-PD-0004
---

# One clone, many agents: how this repository isolates the sessions that build it

Fifteen agent sessions built this repository at the same time on 2026-09-11, and fourteen worktrees were live on the same host on 2026-09-18. Each rule that makes this work was written after a failure, and each one lives in the header of the file that implements it. No document states the arrangement as a whole, so no document states which lines hold up other files and which lines are free choices. This evaluation is that statement. The decision records named below carry the rulings, and this document carries the evidence and the failures.

The reader outside this repository is an adopter who runs several agent sessions against one corpus of their own. Spec 16 states the hook contract that raises most of the questions below, and an adopter who binds a harness writes the hooks themselves. The last section separates the part that is portable from the part that belongs to this repository.

Two subjects sit next door and are not here. [The build order as a multi-agent system](the-build-order-as-a-multi-agent-system.md) carries what a dispatch costs and how a run is shaped. [HW-DR-0054](../decisions/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md) carries the committed identifier claim store, which is a different claim with a different lifetime.

## What is shared, and what is not

One clone holds every session. Three things are shared across every session, and two are private to each one.

| what | where | shared or private |
|---|---|---|
| the working tree of a change | `.claude/worktrees/<name>` | private to one session |
| git objects, refs and config | the git common dir | shared by every worktree |
| run state, session marks, the routing log, the embedding model | four directories under the git common dir | shared by every worktree |
| compiler output | three pooled target directories under the user cache | shared by whichever sessions hold a slot |
| the built engine | `engine/target/<profile>/headwater` inside each worktree | private, and copied in after a pooled build |

The private column is short on purpose. A worktree costs a checkout of the tree and nothing else, because git keeps one copy of the objects. Everything expensive is shared, and everything that a retirement sweep may delete is private.

## The four rules

### A worktree holds a change, and the git common dir holds everything else

`git rev-parse --git-common-dir` answers one absolute path from every worktree of a clone, and git commits nothing under it. State that must outlive a worktree goes there. The reason is a deletion. `tools/repo/retire-worktree.sh` removes a finished tree and every file inside it, and [HW-DR-0064](../decisions/0064-q64-whether-intent-time-routing-gains-an-offline-embedding-path-in-shadow-mode.md) puts the routing log outside the tree for that reason. Four directories use it, and the table in [DEVELOPING.md](../../DEVELOPING.md) names each one with its writer.

Coordination on that shared state is a file, and the file is created once. `tools/run/run-dir.sh` opens a claim with a `set -C` redirect, which dash implements as the `O_EXCL` flag. The kernel serializes two claimants, and the second one reads the first claim and reports what it waits on. A claim is never empty, because it holds the issue and the branch that made it. [HW-PD-0004](../process/decisions/0004-coordination-is-a-create-only-claim-and-authority-stays-on-the-tree.md) is the ruling, and its second half matters as much as the first: authority stays on the tree. A claim orders the merges and refuses nobody, because a message between peers carries no authority that either one may act on.

### A hook reads its binary through one root and its corpus through another

A harness gives a hook two candidate roots, and they disagree for every session that works in a worktree. `CLAUDE_PROJECT_DIR` names the checkout the session started in and never moves. The payload's own `cwd` member follows the session. A hook that resolves a corpus from the first one checks the main checkout while the session edits a worktree. The report it prints is about somebody else's tree.

`.claude/hooks/lib.sh` separates the two questions rather than picking a winner. `hw_root` locates a binary, which is not a question about a corpus, so any checkout's binary answers it. `hw_resolve_root` reads `cwd` for everything that is corpus-relative. The order is part of the rule. A hook takes its plain reads of the payload first, through the uncorrected root. A worktree with no engine of its own then loses only the corpus-relative answers.

Two corrections sit inside that function and both came from a live failure. A session that moves into `engine/` hands the hook a directory with no `.headwater/taxonomy.lock` under it. The function climbs to the top of the work tree. A fixture that points a hook at another corpus sets `HEADWATER_HOOK_ROOT`, and that value wins over everything. A session's own location is not grounds to override what a test asked for.

### Every consumer of the engine reads one path

The commit gate, all four harness hooks and several scripts under `tools/` resolve the binary from `engine/target/<profile>/headwater`, relative to the checkout they run in. Each takes the newer of `release` and `dev-release`, rather than the shipped profile by name. A stale `release` binary beside a fresh `dev-release` one would check a change through an engine older than the change.

That single path is what makes a pooled build safe. `tools/hw-cargo` points cargo at a target directory outside every checkout, and the last thing it does is copy the binary back to the path above. It resolves the destination from `--manifest-path` and not from the current directory. The current directory named the wrong worktree whenever a session's own location had drifted, and the binary built for one tree was written into another.

### A tree is retired on merged content and on emptiness

This repository squash-merges. No commit of a finished branch is an ancestor of `origin/main`, so ancestry calls every finished branch unmerged forever. A sweep built on ancestry retires nothing. `tools/repo/retire-worktree.sh` asks the pull request instead: a branch is retired when a merged pull request names a head that contains the branch tip.

Ancestry fails in the other direction as well, and that failure destroys work. `git worktree add <path> -b <branch> origin/main` puts a new tree exactly at the tip of `origin/main`, so ancestry cleared such a tree from the first second of its life. The sweep deleted the directory, and then the branch it had just freed, while the agent that made it was still building. So a tree that holds no commit `origin/main` lacks is kept. The cost is a tree that is empty and also abandoned, which is reported on every sweep and collected by nobody. The trade runs this way round because the other direction destroys work in progress.

Five guards hold the sweep, and each one cost somebody work before it was written. A lock is a deliberate hold. A process with its working directory under the tree pins the directory on the disk after the removal. The next cargo run there exits 101 with a `NotFound` that names the binary under test, which reads as a defect in the code. An uncommitted change is work that no branch holds. The other two are the two above.

## The measurements that set the shape

Every number here was taken on this repository, on an eight core host with 15GB of memory.

**Oversubscription, 2026-09-11, with fifteen worktrees live.** Nine cargo invocations ran at once, the run queue sat at 34 against 8 cores, and CPU pressure `some avg300` read 75%. The throttle in use at the time wrapped `cargo build` alone, so `cargo test`, `cargo check` and `cargo clippy` each took as many jobs as they liked. `tools/hw-cargo` wraps every cargo subcommand, which is what makes the cap real.

**Duplicated compilation, the same day.** One target directory per worktree means every worktree compiles the same dependency graph again. There were 1441 dependency rlibs on disk under 621 distinct names. So 57 percent of that output repeated work already finished elsewhere on the same disk, including 34 copies of `syn`.

**The link, and why two profiles exist.** A one crate relink costs 153 seconds of CPU under `release` and 25 under `dev-release`. The `release` figure is one core held for two and a half minutes, because `lto = true` and `codegen-units = 1` make that link single threaded. `lto = "thin"` was measured before it was ruled out, at 287 seconds of CPU spread over more cores. Every hook runs the binary for about a fifth of a second, so the cheap profile is what a checked commit needs.

**A gate that woke the wrong session.** The review hook runs the commit gate at the end of a turn. The parent of a build-order run owns no checkout and edits nothing. Run against that parent, the hook read the integrator's half-finished rebuild as red nine times in one run. That cost 20 million input tokens over nine turns in run 20260911-1331. The parent's session id is written into the run directory, and a session named there ends its turn without the gate.

**A gate that stopped a session for somebody else's work.** A session with zero mutating tool calls was stopped by a stale site figure and a stale taxonomy lock digest. An unrelated branch had left both on the checkout it ran against. `.claude/hooks/touch.sh` marks a session that writes a file, and the review hook passes a session that carries no mark.

## What was rejected, and what the rejection cost to find

**`mkdir` as the claim primitive.** It is the textbook answer and it is wrong on this host. On coreutils supplied by the uutils rewrite, two racing `mkdir` calls on one path both succeeded in 17 of 20 races, measured 2026-09-07. A sequential second call is refused, so every test that did not race passed. A noclobber redirect, `ln`, `ln -s` and Python's `os.mkdir` each held at 0 of 20. The race case stays in the fixture suite, so that a primitive that stops being atomic is reported rather than trusted.

**A compile cache as the way to share work.** `sccache` declines a proc-macro on `crate-type`, and it declines an incremental build. Those two reasons covered 1621 of 2961 compile requests. It also shares one object cache across every checkout on the machine. A test that resolves a directory from `env!("CARGO_MANIFEST_DIR")` bakes that absolute path into the object. A cache hit can then fail on a path that belongs to a deleted worktree. Pooling the target directory shares the work that the cache cannot.

**A coordinator that relays a collision.** The orchestrator of one measured run sent 67 messages. A few of them warned one worker about a derived artifact that another worker was about to regenerate. A coordinator that holds the state of N workers re-reads its whole context for every message it routes. The claim file replaced those messages, and the veto stayed a message, because the veto is a decision only the parent may make.

**An interpreter inside a hook.** The review hook read its own re-entry guard through `python3`, and on a host without one the read returned an empty string. The guard was then absent rather than unreadable, so the hook stopped the same turn forever with no way out. [HW-DR-0055](../decisions/0055-a-hook-reads-a-wire-format-through-the-engine-and-not-through-an-interpreter.md) moved every field read to the engine, which parses JSON already. The engine is a dependency every position already had.

**A hook that rewrites a shell command.** A `PreToolUse` hook on the `Bash` matcher bounded the output of every command. It handed the harness a rewritten string that carried a `trap` and a brace group. A worktree-isolated session refuses that shape outright, whether or not the command underneath it is safe. Measured 2026-09-16: `trap 'echo x' EXIT` is refused by name, and a function definition combined with a redirected brace group is refused even though each passes alone. A worktree is the ordinary way to run a session here. So that hook refused ordinary work rather than the rare case it meant to bound. `tools/cap-run` is the replacement, and it is a plain script that a caller names.

**Failing closed.** Every hook and the commit gate fail open when they cannot find an engine or cannot read their input. The cost is a gate that exits 0 in a fresh worktree and checks nothing. The alternative cost is a repository that nobody can commit to, and the CI job holds the same tree afterward in either case.

## What this arrangement still gets wrong

Six of these are open, and two are accepted.

- `core.hooksPath` is one key in a config that every worktree shares, and a harness tool writes it as an absolute path on every call. All fourteen worktrees on this host ran the main checkout's hook bodies on 2026-09-18. [#925](https://github.com/headwater-ai/headwater/issues/925).
- Nothing collects the per-run and per-session state under the git common dir, and HW-PD-0004 states that a run directory is deleted when its run closes. Eight run directories from 2026-09-07 onward were still present on 2026-09-18. [#926](https://github.com/headwater-ai/headwater/issues/926).
- A pooled target directory can link an rlib that belongs to a different worktree, under `dev-release` alone. [#849](https://github.com/headwater-ai/headwater/issues/849).
- An agent that inherits the working directory of the agent that dispatched it builds inside a peer's tree. Four occurrences in one run. [#848](https://github.com/headwater-ai/headwater/issues/848).
- The retirement sweep loses a field while it parses a worktree with no branch, so one arm of it has never run. [#847](https://github.com/headwater-ai/headwater/issues/847).
- The documented fast build writes a binary that several consumers of the engine do not look for. [#647](https://github.com/headwater-ai/headwater/issues/647).

Two gaps are accepted rather than open. `touch.sh` marks a write through an edit tool and not through a shell command. A session whose only mutation ran through `Bash` is therefore still gated, which is the safe direction. The sweep keeps a tree that is empty and abandoned, and reports it forever.

## What an adopter takes from this

Five of the rules above are portable, and two are local.

Portable: one location for state that must outlive a working tree, and a rule that nothing else holds shared state. A create-only file as the coordination primitive, with the atomicity of the primitive measured on the host rather than assumed. A corpus root read from what the harness says about this session, and not from the variable that names where the session began. One path for a built tool that every consumer agrees on, whatever built it. A retirement test based on content, because a squash merge defeats ancestry in every repository that uses it.

Local: the three pooled compiler slots, which answer to the core count of one host. Local as well: the two build profiles, which answer to the link cost of one workspace. An adopter takes the measurement rather than the number.

The deeper pattern under all five is one sentence. Every one of these mechanisms fails in the direction of doing nothing rather than refusing. So each one has to be read for what it would print if the thing it protects were absent. A check that cannot run reads exactly like a check that passed.
