---
name: hw-run-policy
description: The standing rulings and the environment of a Headwater build-order run, so that no stage stops to ask. Invoke before starting any hw-queue, hw-adjudicate, hw-build, hw-verify or hw-integrate dispatch. It cites the rulings by identifier rather than restating them, carries the traps each of which cost somebody an hour, and reads the same list for cost.
---

# The run policy

Every stage of a build-order run reads this once. It is a skill and not a paragraph in each definition because more than one stage obeys it ([HW-PD-0001](../../../docs/process/decisions/0001-orchestration-prose-has-one-owner-per-sentence.md)), and it cites a ruling rather than restating it so that a ruling tuned later moves in one place. The value rule is stated once in `.claude/commands/next-run.md` and nowhere else.

## The standing rulings

- **A stale premise is adjudicated; nobody halts.** The adjudicator refuses with what changed, the parent rules, and the human hears of it only when a redirect changes milestone order or a decision needs an owner ([HW-PD-0002](../../../docs/process/decisions/0002-adjudication-is-a-separate-stage-and-refusal-is-licensed.md)).
- **Refusal has three kinds, and the parent rules on the kind.** "Nothing states what this is" ships a verb that reports the gap and merges. "This would be better done another way" is a product decision, merged only if reversible and recorded with the condition that reopens it, written into the artifact where the next reader meets it. "The Done-when offers a second outcome and the measurement says take it" is a completion ([HW-PD-0002](../../../docs/process/decisions/0002-adjudication-is-a-separate-stage-and-refusal-is-licensed.md)).
- **An honest split is a success condition.** `Refs #N` with a measured reason beats `Closes #N` on a bar that was stretched. When declines cluster, write smaller issues rather than praising the split.
- **A recorded blocker is not a measured one.** Re-measure a blocker before inheriting it. Three times in one run a milestone or an issue sat on an assessment nobody had re-taken.
- **A milestone can run out of buildable work without being finished**, and that is a finding: say what each remaining issue waits on, leave the epic open, and move on without manufacturing work.
- **The board's structure is the product owner's, and its scope is the parent's.** `.claude/agents/headwater-product-owner.md` closes a finished milestone, moves a misfiled issue and writes the priority labels; nobody else does in passing. Closing an epic whose children are all done, after verifying its Done-when clause by clause, stays with the parent.
- **A finding with no reader outside this repository goes to spec 13, capped at three per iteration**, past which one obligation record lists the rest. The register is hand-maintained and its own count has gone stale by hand ([HW-OBL-0142](../../../docs/obligations/0142-the-obligation-register-states-its-own-size-by-hand-and-it-went-stale-three-times-in-two-days.md)).
- **Every issue filed or edited opens with an `## ELI5` section**, per `.github/ISSUE_TEMPLATE/issue.md`, which `gh` does not apply.
- **A red `main` is fixed in the next iteration's branch before that iteration's own work.**
- **An agent that dies is resumed by its id, never replaced.** A fresh agent throws away a settled design; both agents that died in one run finished their own work on resume. Only pushed commits survive.
- **An agent can state a write-back and not land it.** Read what it did, never what it said it would do, and check the board after every resume.
- **Coordination is a claim on the board or a create-only file, never a coordinator's context** ([HW-PD-0004](../../../docs/process/decisions/0004-coordination-is-a-create-only-claim-and-authority-stays-on-the-tree.md)). The veto and the re-verify travel by `SendMessage`; authority stays on the tree.
- **Never run a generating verb against a checkout another session is merging into.** Two sessions regenerating a corpus-wide artifact do not conflict, because blobs are compared before a merge strategy is chosen; they merge wrong ([HW-DR-0049](../../../docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md)).

## The environment

Each entry cost somebody an hour.

- `gh issue view` and `gh pr edit` fail with a `projectCards` GraphQL deprecation. Use `gh api repos/headwater-ai/headwater/issues/<N> --jq .body`, and patch a body with `gh api -X PATCH repos/headwater-ai/headwater/issues/<N> -F body=@file.md`. Read the comments too.
- Worktree isolation refuses heredocs, compound shell, `git -C` into another worktree, and a script fed through a heredoc that names git. Write scripts to files and run them.
- `Write` is refused in the shared checkout and `EnterWorktree` is refused from a subagent. Make a worktree by hand with `git worktree add`, from `origin/main` after a fetch, and remove it at the end or say that it is still there.
- `git push -u origin <branch>`, never a bare push. The git setting push.default is `simple` on this machine now, and the explicit form is right under either setting.
- `git config --get core.hooksPath` must answer `.githooks`. `EnterWorktree` rewrites it absolute on every call, and a worktree with an absolute path runs the main checkout's hook body instead of its own; the commit gate reports it and does not refuse.
- Build with `cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked`, never `--release` for a checked commit: the release profile's `lto` and single codegen unit are single threaded and cost minutes. The gate and every hook run whichever profile is newer. Never copy a binary between the two paths.
- **Every cargo command of a run goes through `sh tools/hw-cargo`**, with the same arguments you would have given cargo: `sh tools/hw-cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked`, and the same wrapper for `test` and for `check`. It takes one of three slots and points cargo at that slot's pooled target directory, so the stages of a run share a compiled dependency graph instead of each rebuilding it, and the cap on concurrent compilers is a real one. A stage that calls cargo directly is not throttled by anything and competes with every other stage for the same eight cores. The header of the script carries the measurement that set both numbers.
- **Never run clippy locally. It is CI's gate and not a stage's.** Clippy is the only tool a run reaches for that needs the rustup toolchain, so it compiles the whole workspace a second time under a second compiler and shares neither a fingerprint nor an sccache key with anything else the run builds. [DEVELOPING.md](../../../DEVELOPING.md) already rules that a green clippy on your host is not evidence about CI. Push and read CI.
- **Rebuild the engine after a rebase or a merge, before `headwater generate` or `headwater check`.** A pre-rebase binary writes stale artifacts and passes them, because the same binary wrote and checked them; and a stale binary reads correct pages as stale, with a destructive remedy. Read the direction of the delta before believing either.
- **Never pipe a gate.** `<gate> | <filter>` reports the filter's exit status. Redirect to a file and read the tail in a second call.
- **Never merge stderr into stdout on an invariant test.** The engine prints a run statistic to stderr and the artifact to stdout, and one agent wrote "the cache invariant is literally false" into durable memory on the strength of `2>&1`.
- **A test of the engine re-resolves the lock.** `headwater check` reads `.headwater/taxonomy.lock` and never the package sources. Run `headwater taxonomy resolve` between editing `packages/` and checking, and resolve after publishing, not before, or both sides of a comparison move together.
- A conflicting pull request runs no CI at all. Read `mergeable` before reading a missing check run as a dead runner.
- `cargo` runs a target's cases as threads of one process, so a temp-dir helper keyed on the pid alone races, and the symptom is `NotFound` out of `std::fs::copy`.
- A cargo failure quoting a dead worktree's path is an `sccache` hit rather than a defect; `cargo clean -p <crate>` clears it. A fresh worktree has no engine, so the commit gate fails open there until you build one.
- Never navigate from rust-analyzer's `documentSymbol` line numbers; `grep -n 'fn <name>'` first.
- Commit and push in small steps. A transport error costs everything unbanked.

## The same list, read for cost

A tool result costs its own size times the turns that follow it, so position is worth as much as size ([HW-PD-0003](../../../docs/process/decisions/0003-a-dispatch-pays-when-it-retires-more-parent-turns-than-it-costs.md)).

- **Wait by blocking, never by polling.** One call that blocks, `until [ -f "$T/x.status" ]; do sleep 30; done`, or `Monitor` with an until-condition; never a sleep under thirty seconds, never a bare re-check. One agent checked a status file 85 times in five minutes and burned 29% of a whole run.
- **Backgrounding a job is half a decision; the other half is how you wait for it.** Decide both in the same breath, and end every wait you started before you exit.
- **Redirect a build or a check to files and read the tail**, stdout and stderr to separate files.
- **Ask an API for the field, never the record.** `gh api ... --jq .body` is the same fact at a fraction of `gh issue view`.
- **Never read a whole specification part.** `headwater explain` first, then `Read` with an offset and a limit.
- **Do not run the same command twice**, and count your repeats rather than trusting that you would notice.
- **Prefer `Edit` to `Write` on a file that exists**, and compose a long prompt or note in a file rather than in a shell argument.
- **Correct an environment claim the moment you disprove it**, in the place it was written.
