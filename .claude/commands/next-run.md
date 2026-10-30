---
description: Run N iterations of the Headwater build order as a resumable run over six agent definitions, merging through an integrator slot
argument-hint: "[iteration count, default 20] [--parallel N]"
---

Run `$ARGUMENTS` iterations (default 20) of the Headwater build order. You are the parent, and your job is judgment: what to merge, what a stale premise means, which surprise is a lesson. Every stage is an agent definition under `.claude/agents/`, dispatched by `subagent_type`, each carrying its own instructions; paste nothing a definition already says. Load the `hw-run-policy` skill at the start of every session, the first included.

**Width.** Without `--parallel N` one issue is in flight, merged before the next starts; with it N build at once and merges go through the slot below into the merge queue. Five to eight lost concurrency ([the evaluation](../../docs/process/evaluations/the-build-order-as-a-multi-agent-system.md)): raise it only on its numbers.

## The value rule

Canonical here; every other file cites it rather than restating it.

**Before any work starts, name the ground it stands on** (owner, 2026-09-30). Four equal grounds make work eligible. **Outside reader:** someone who is not this repository is better off. **Correctness:** the engine or corpus says something false, or a check misses its case. **Efficiency:** a run, a hook or plan usage costs less. **Documentation:** a document under `docs/`, not `docs/process/`, becomes true. Work on none becomes an obligation record on the register `hw-run-policy` names. `bug` sorts first and `adopter-blocking` next.

## The doctrine

<!-- doctrine -->
# The doctrine

Ten lines the parent of a build-order run obeys on every turn. A run copies them into its run directory so a compacted parent can act from them alone ([HW-PD-0006](../../docs/process/decisions/0006-the-entrypoint-keeps-its-name-and-becomes-a-resumable-run.md)), and `sh .claude/agents/fixtures.sh` holds `.claude/commands/next-run.md` to the same bytes.

1. **Name the ground the work stands on before it starts.** Work that stands on none of the four goes to the register as an obligation record, never to the tracker and never to a slot.
2. **Do not stop between iterations, and never poll.** On any completion, act in the same turn: dispatch before you narrate, then end the turn. With agents in flight an ended turn is the blocking wait and their reports wake you; a shell `true` is a poll. On `DRAIN`, start no stage, and exit at zero in flight.
3. **Wait by blocking, never by polling.** You never read a pull request's `mergeable`; the agent that owns the pull request does, and its report is your notification.
4. **The merge decision never leaves you, and the mechanics never stay with you.** Rule, then hand the merge to a fresh `hw-integrate`, one in flight at a time and never two.
5. **Read a verdict, never a build output.** Ask every agent for under 400 tokens back, and open the file it wrote only when you are ruling on it.
6. **Compose nothing long in your own turn.** Write a prompt or a note with `Write` and pass a path. A heredoc in a shell argument is context twice.
7. **A verdict of PASS is what the verifier ran, not proof that the branch is sound.** The veto is a resume, never a bin: send the branch back to the agent that owns its loop, with the finding. After a restart, a fresh `hw-iterate` takes its handover.
8. **Read the doctrine and the last five ledger lines on a turn you already pay for, never the whole ledger.** The ledger lives on disk; nothing of it lives in your context by default.
9. **Escalate to the human only when a redirect changes milestone order, or a decision needs an owner rather than an answer.** Everything else you rule, and you write the ruling down where the next reader meets it.
10. **Correct an environment claim the moment you disprove it, in the place it was written.** Left standing, it teaches every later agent the wrong lesson.
<!-- /doctrine -->

## The loop

`sh tools/run/run-dir.sh start` makes the run directory and prints its path, which every dispatch carries. That directory is the ledger ([HW-PD-0005](../../docs/process/decisions/0005-the-ledger-is-split-its-tabular-parts-are-jsonl-and-its-totals-are-derived.md)): its `log` takes one line per iteration, `tail` is what you read, `net` derives opened minus closed, and the prose ledger is yours to append through the verbs `hw-run-policy` names. Read `lessons.md` by heading, only the parent's sections, never whole. Ask an agent one line by `SendMessage`, never `ListAgents`. When the owner's prompt fixes the scope, record it first with `run-dir.sh decide <run> "SCOPE: <issues>"`. Every queue refill and resumed session reads it before it fills a slot.

1. **Top of the run.** Dispatch `headwater-product-owner` and `hw-queue` in one turn. Read the queue agent's report and nothing else.
2. **Fill.** While fewer than N issues are in flight and the queue holds one, dispatch `hw-adjudicate` for the issue `run-dir.sh next` prints, with the file `run-dir.sh dispatch <run> <issue> adjudicate` prints.
3. **On an adjudicate report.** `VERDICT: BUILD` runs `run-dir.sh accept <run> <issue> branch=<name> note=<path> footprint=<a,b>`, which stages, claims, and prints the `hw-iterate` dispatch file with any `WAITS-ON` the claim found; dispatch it. It owns build, verify and rework, and reports once. `VERDICT: REFUSE` is ruled by the three kinds in `hw-run-policy`, recorded with `run-dir.sh rule`, and the next issue is taken in the same turn.
4. **On an iterate report.** `VERDICT: PASS` is your cue to rule, from the verdict and the notes it names, never from the branch. If you merge, append the ruling with `run-dir.sh integrate <run> <PR> issue=<N> branch=<name> ruling=<words> footprint=<a,b> sha=<its HEAD>` and dispatch the next adjudicate in the same turn. `VERDICT: STOP` is ruled like a refusal.
5. **The integrator slot.** Depth one. When nothing is integrating and `run-dir.sh integrations` prints a ruling, dispatch a fresh `hw-integrate` with the file `run-dir.sh dispatch <run> none integrate` prints. A ruling you append while one is live goes to it by `SendMessage`, as the line `integrations` prints, never to a second one. An ejected one returns for a new ruling, recorded as `ruling=ejected` until you rule it again ([HW-PD-0020](../../docs/process/decisions/0020-merges-go-through-the-github-merge-queue-one-squash-commit-per-pull-request.md)). Never a long-lived one ([HW-PD-0003](../../docs/process/decisions/0003-a-dispatch-pays-when-it-retires-more-parent-turns-than-it-costs.md)).
6. **Every fifth merge, and at the end**, dispatch `headwater-product-owner`. Its `RELEASE READY` line is a question for the owner under the rule `hw-run-policy` states, and never a tag.
7. **Then**, if `run-dir.sh applies <run>` prints `APPLY` lines, dispatch one `hw-iterate` with `issue: none (run lessons <run>)`. A line its pull request could not apply goes to a process obligation record.

## The veto

Your veto of a `PASS` goes by `SendMessage` to that `hw-iterate` id, with your finding. It counts toward the three `FAIL`s at which that agent stops: something upstream is wrong, and a fourth will not find it. Name exact mutants, never a standard.
## Resume

`/next-run --resume <run-id>` is a fresh parent on a run `tools/run/supervise.sh` restarted. `hw-run-policy` says what a resumed parent reads and does.

## The dispatch

`run-dir.sh dispatch <run> <issue|none> <stage> [field=value]...` renders every dispatch into the issue's scratch directory and prints the path. Pass that path and nothing else; an environment fact belongs in `hw-run-policy`. The verb's header names the fields each stage takes, and `tools/run/run-dir-fixtures.sh` holds the rendered lines.

## Stop

Report at the end of the run, when a decision needs an owner, or when a human asks: a table of issue, pull request and result, the lessons one included; what verification caught that a report did not, and what was your own error; anything you would not merge again without a ruling; the branches left for the owner; what `run-dir.sh end` prints. `ls` each file a command for the owner names.
