---
description: Run N iterations of the Headwater build order as a resumable run over five agent definitions, merging through an integrator slot
argument-hint: "[iteration count, default 20] [--parallel N]"
---

Run `$ARGUMENTS` iterations (default 20) of the Headwater build order. You are the parent, and your job is judgment: what to merge, what a stale premise means, which surprise is a lesson. Every stage of the work is an agent definition under `.claude/agents/`, dispatched by `subagent_type`, and each one carries its own instructions, so you paste nothing into a prompt that a definition already says.

**Width.** Without `--parallel N` the run is sequential: one issue in flight, merged before the next starts. With it, N issues build at once and the merge is still one at a time through the slot below. Wider is not better: a run that went from five to eight saw concurrency fall and an idle fleet appear ([the evaluation](../../docs/evaluations/the-build-order-as-a-multi-agent-system.md)). Raise it only on the numbers that record names.

## The value rule

This is the canonical statement. Every other file cites it rather than restating it, and so do `.claude/commands/next.md` and `.claude/agents/headwater-product-owner.md`.

**Before any work starts, name the reader who is not this repository.** If the only party better off is Headwater's own corpus, the work is not eligible for an iteration and not eligible for the tracker: it is scaffolded as an obligation record under [13 — Open obligations](../../docs/spec/13-open-obligations.md) and left there. `adopter-blocking` means work an outside adopter cannot proceed without, and it sorts above everything else. This command is an issue generator by construction, and the evaluation records the day it took 45 issues and closed 16.

## The doctrine

<!-- doctrine -->
# The doctrine

Ten lines the parent of a build-order run obeys on every turn. `.claude/commands/next-run.md` carries a byte-identical copy of this file, `sh .claude/agents/fixtures.sh` holds the two together, and a run copies it into its run directory so that a parent which has compacted can act from it alone ([HW-PD-0006](../../docs/process/decisions/0006-the-entrypoint-keeps-its-name-and-becomes-a-resumable-run.md)). Every line is a rule the parent itself must keep; a rule one stage keeps lives in that stage's agent definition, and a rule two stages keep lives in a skill ([HW-PD-0001](../../docs/process/decisions/0001-orchestration-prose-has-one-owner-per-sentence.md)).

1. **Name the reader who is not this repository before any work starts.** Work whose only beneficiary is this corpus goes to spec 13 as an obligation record, never to the tracker and never to a slot.
2. **Do not stop between iterations.** On any completion, act in the same turn: dispatch the next stage or the next issue before you write a word of narration. A merged pull request is the middle of the run.
3. **Wait by blocking, never by polling.** You never read a pull request's `mergeable`; the agent that owns the pull request does, and its report is your notification.
4. **The merge decision never leaves you, and the mechanics never stay with you.** Rule, then hand the merge to a fresh `hw-integrate`, one in flight at a time and never two.
5. **Read a verdict, never a build output.** Ask every agent for under 400 tokens back, and open the file it wrote only when you are ruling on it.
6. **Compose nothing long in your own turn.** Write a prompt or a note with `Write` and pass a path. A heredoc in a shell argument is context twice.
7. **A verdict of PASS is what the verifier ran, not proof that the branch is sound.** The veto is a resume, never a bin: send the branch back to the agent that built it, with the finding.
8. **Read the doctrine and the last five ledger lines on a turn you already pay for, never the whole ledger.** The ledger lives on disk; nothing of it lives in your context by default.
9. **Escalate to the human only when a redirect changes milestone order, or a decision needs an owner rather than an answer.** Everything else you rule, and you write the ruling down where the next reader meets it.
10. **Correct an environment claim the moment you disprove it, in the place it was written.** Left standing, it teaches every later agent the wrong lesson.
<!-- /doctrine -->

## The loop

`sh tools/run-dir.sh start` makes the run directory under the git common dir, reachable from every worktree, with the doctrine copied in and the last run's lessons and decisions seeded; it prints the path, and every dispatch carries it. That directory is the ledger ([HW-PD-0005](../../docs/process/decisions/0005-the-ledger-is-split-its-tabular-parts-are-jsonl-and-its-totals-are-derived.md)): `run-dir.sh log` takes one line per iteration, `run-dir.sh tail` is what you read, `run-dir.sh net` derives opened minus closed, and `lessons.md` and `decisions.md` are yours to append with `Edit`. Read `lessons.md` once at the top and never again.

1. **Top of the run.** Dispatch `headwater-product-owner` and `hw-queue` in one turn. The queue agent writes the ordered eligible issues into the run directory; read its report and nothing else.
2. **Fill.** While fewer than N issues are in flight and the queue holds one, dispatch `hw-adjudicate` for the next issue with the template below.
3. **On an adjudicate report.** `VERDICT: BUILD` claims the footprint with `sh tools/run-dir.sh claim <run> <issue> <branch> <artifacts>` and dispatches `hw-build` with the same template, the adjudication note's path, and any `WAITS-ON` the claim printed. `VERDICT: REFUSE` is ruled by the three kinds in the `hw-run-policy` skill, written into the run's decisions file, and the next issue is taken in the same turn.
4. **On a build report.** Dispatch `hw-verify` with the branch, the pull request number, the adjudication note, and the attacks you chose from the `hw-verification-bar` skill. Choosing the attacks is the judgment you keep; running them is not.
5. **On a verify report.** `VERDICT: PASS` is your cue to rule. If you merge, append the ruling to the integrator queue and dispatch the next adjudicate in the same turn. `VERDICT: FAIL` is the veto below.
6. **The integrator slot.** Depth one. When nothing is integrating and the queue holds a ruling, dispatch a fresh `hw-integrate` with the pull request, the ruling and the footprint the adjudicator declared. Never a second one while the first runs, and never one long-lived integrator ([HW-PD-0003](../../docs/process/decisions/0003-a-dispatch-pays-when-it-retires-more-parent-turns-than-it-costs.md)).
7. **Every fifth merge**, dispatch `headwater-product-owner` again.

## The veto

A `FAIL` goes back to the `hw-build` agent that wrote the branch, by `SendMessage` to its id, with the verifier's finding and nothing you added. A resumed agent keeps its settled design; a fresh one throws it away. When it returns, dispatch `hw-verify` again. Two consecutive iterations failing their own bar is the stop condition: something upstream is wrong and a third will not find it.

## The dispatch

Composed with `Write` into the issue's scratch directory and passed as a path. Nothing else goes in the prompt.

    issue:        #<N> <title>
    run:          <run directory>
    scratch:      $CLAUDE_JOB_DIR/tmp/issue-<N>/
    branch:       <name>            (build, verify, integrate)
    pull request: #<PR>             (verify, integrate)
    footprint:    <artifacts>       (from the adjudication; integrate)
    waits-on:     #<N> or none      (from the claim; build, integrate)
    ruling:       <your ruling>     (integrate)
    attacks:      <chosen headings from hw-verification-bar>  (verify)
    deadline:     <minutes>
    report:       under 400 tokens, ending in the fixed block your definition names

## Stop

Report only at the end of the run, when a decision needs an owner, or when a human asks. A table of issue, pull request and result; what your verification caught that a report did not, and what it caught that was your own error; anything you would not merge again without a ruling; the run directory's path.
