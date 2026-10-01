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

`sh tools/run/run-dir.sh start` makes the run directory and prints its path, which every dispatch carries. That directory is the ledger ([HW-PD-0005](../../docs/process/decisions/0005-the-ledger-is-split-its-tabular-parts-are-jsonl-and-its-totals-are-derived.md)): its `log` takes one line per iteration, `tail` is what you read, `net` derives opened minus closed, and the prose ledger is yours to append through the verbs `hw-run-policy` names. Read `lessons.md` by heading, only the parent's sections, never whole. Append to the integrator queue; never rewrite it. Ask an agent one line by `SendMessage`, never `ListAgents`.

1. **Top of the run.** Dispatch `headwater-product-owner` and `hw-queue` in one turn. Read the queue agent's report and nothing else.
2. **Fill.** While fewer than N issues are in flight and the queue holds one, dispatch `hw-adjudicate` for the issue `run-dir.sh next` prints, with the template below.
3. **On an adjudicate report.** `VERDICT: BUILD` runs `run-dir.sh stage <run> <issue> adjudicated note=<path> footprint=<a,b>`, claims the footprint with `run-dir.sh claim <run> <issue> <branch> <artifacts>`, and dispatches `hw-iterate` with the same template, the adjudication note's path, and any `WAITS-ON` the claim printed. It owns build, verify and rework, and reports once. `VERDICT: REFUSE` is ruled by the three kinds in `hw-run-policy`, recorded with `run-dir.sh rule`, and the next issue is taken in the same turn.
4. **On an iterate report.** `VERDICT: PASS` is your cue to rule, from the verdict and the notes it names, never from `git show` or `git diff` of the branch ([HW-PD-0022](../../docs/process/decisions/0022-the-verify-and-rework-loop-for-one-issue-runs-below-the-parent.md)). If you merge, append the ruling to the integrator queue and dispatch the next adjudicate in the same turn. `VERDICT: STOP` is ruled like a refusal.
5. **The integrator slot.** Depth one. When nothing is integrating and the queue holds a ruling, dispatch a fresh `hw-integrate` with every pull request ruled MERGE, each with its ruling and declared footprint. It enqueues all, and an ejected one returns for a new ruling ([HW-PD-0020](../../docs/process/decisions/0020-merges-go-through-the-github-merge-queue-one-squash-commit-per-pull-request.md)). Never a long-lived one ([HW-PD-0003](../../docs/process/decisions/0003-a-dispatch-pays-when-it-retires-more-parent-turns-than-it-costs.md)).
6. **Every fifth merge, and at the end**, dispatch `headwater-product-owner`.

## The veto

Your veto of a `PASS` goes by `SendMessage` to that `hw-iterate` id, with your finding. It counts toward the three `FAIL`s at which that agent stops: something upstream is wrong, and a fourth will not find it. Name exact mutants, never a standard.
## Resume

`/next-run --resume <run-id>` is a fresh parent on a run `tools/run/supervise.sh` restarted. Drain to zero before any exit, because a subagent dies with its parent. `hw-run-policy` says what a resumed parent reads and does.

## The dispatch

Composed with `Write` into the issue's scratch directory and passed as a path. Nothing else goes in the prompt; an environment fact belongs in `hw-run-policy`.

    issue:        #<N> <title>
    run:          <run directory>
    scratch:      $CLAUDE_JOB_DIR/tmp/issue-<N>/
    branch:       <name>            (iterate, build, verify, integrate)
    pull request: #<PR>             (verify, integrate)
    footprint:    <artifacts>       (from the adjudication; integrate)
    waits-on:     #<N> or none      (from the claim; iterate, build, integrate)
    ruling:       <your ruling>     (integrate)
    attacks:      <headings from hw-verification-bar, nothing more>  (build, verify; iterate picks)
    deadline:     <minutes>
    report:       the fixed block your definition names and nothing before it

## Stop

Report at the end of the run, when a decision needs an owner, or when a human asks: a table of issue, pull request and result; what verification caught that a report did not, and what was your own error; anything you would not merge again without a ruling; the branches left for the owner; what `run-dir.sh end` prints. `ls` each file a command for the owner names.
