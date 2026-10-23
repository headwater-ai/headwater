---
description: Run one paid try of the tuning round of the additive campaign in this session, report it to the owner in plain terms, then stop
argument-hint: "<round root>"
---

Run one paid try of the tuning round, then stop. `$ARGUMENTS` names the round root. The header of `tools/probe/try.sh` says what the round root holds, and what its two files `env` and `base-pin` say. This file does not copy that header.

**This command runs in its own session and never inside `/next-run`.** The build-order agents do not run a paid try, and `hw-queue` never queues an issue labeled `paid-experiment` ([HW-PD-0025](../../docs/process/decisions/0025-a-paid-try-runs-under-its-own-command-and-never-under-the-build-order.md)). The decision records why: the first paid try went through the build order, and its agents cost more turns than the try decided anything.

**The value rule binds here too.** `.claude/commands/next-run.md` states it once, under *The value rule*, and this file does not restate it.

## The rulings this command obeys

- **One layer at a time, plan first.** The owner reads a layer's tuning plan and accepts it on the issue before any money is spent on that layer (OWNER #1777 and #1778, 2026-10-19). A pull request then writes the comment URL to `rounds.tuning.open.plan` in `.headwater/probe.yml`.
- **Stop after one try** and show the owner the result, before a second try or any other layer (OWNER #1777, 2026-10-19).
- **The keep rule** is the owner's (OWNER #1777, 2026-10-20), and `sh tools/probe/try.sh result` applies it. Do not apply it by hand.
- **The base invalidators** are the owner's (OWNER #1819 and #1821, 2026-10-20), and `sh tools/probe/try.sh plan` holds them. Accept a stale base only on the owner's comment URL.

## One try

1. Run `sh tools/probe/try.sh status <root>`. Read the spend, the approval and the open layer.
2. **When no layer is open,** write the tuning plan of the next layer for the owner, as a comment on its issue, and stop. The owner accepts it on the issue, and a pull request writes `open`. `try.sh plan` refuses with 5 until then.
3. Run `sh tools/probe/try.sh rank <root>` and `sh tools/probe/try.sh plan <root>` to read where the round of the open layer stands. Choose the next stage as the header of `try.sh` defines them: `screen <k>` for the next variant, `best` where no best so far is measured at the confirmation's count, `confirm <variant>` for the top of the rank, or `base`, which measures the base scenario again at the confirmation's count. Run `sh tools/probe/try.sh plan <root> <stage>`. On exit 5, report the refusal it prints and stop. On exit 6, report the rule of rounds, order or stages that it names, and stop. On exit 4 of a screen or a best so far, put each path it prints to the owner. On the owner's yes, run `sh tools/probe/try.sh accept <root> <ruling-url>` with the URL of that comment, and run `plan` again. A confirmation exits 4 when no base is measured at 145 sessions at the pin of `<root>/base-pin`, or when the engine's source changed after that pin, whatever the owner accepted. Then the stage is `base`: plan and launch it at the variant's commit, record it with `result`, and plan the confirmation again.
4. Run `sh tools/probe/try.sh launch <root> <stage>`. Its try root `<try>` is the one that `plan` printed. Exit 6 names a state of the checkout or a try root that exists already. Repair the checkout, or stop and ask.
5. Wait in the foreground with a Bash `timeout` of `300000`:

       sh tools/run/wait-for.sh 'sh tools/probe/try.sh wait <root> <try>'

   On `RE-ISSUE`, run the same call again. Never wait in the background.
6. Read the line that `wait` printed. On `canary-needs-reader`, read the records that `<try>/canary-needs-reader` names, and put them to the owner. Write `canary-passed` only on the owner's word, then run `sh tools/probe/try.sh relaunch <root> <try>` and wait again. On `disk-floor`, `capped`, `canary-failed`, `stop` or `driver-exited`, report it to the owner and stop.
7. On `done`, run `sh tools/probe/try.sh result <root> <try>`. It writes the transcript and `result.md`, and it appends the event of the stage to `<root>/ledger.jsonl`. A screen and a best so far are never kept, and a confirmation is kept or not under the keep rule. Run `sh tools/probe/try.sh ledger <root>` to read the round so far.
8. Open one pull request of your own. It commits the transcript, the figures of `result.md` and the narrative paragraphs the transcript leaves to a person. When the change was not kept, the same pull request reverts it. Name the issue with `Refs`, because the round is not finished.
9. Report to the owner, and stop.

## The report

Report in plain terms first: the scores, what they mean and what the try cost. No interval and no statistical term comes first. For example:

> The confirmed version solved 82 of 145 tasks against 80 of 145 before. That is too close to call, so it is not kept. The try cost $72.50.

Then give the `pooled:` line of `result.md` with its interval, for a reader who wants it. Then name the pull request, the round's spend against its approval from `sh tools/probe/try.sh spent <root>`, and the question the owner owes: the next stage of the round, another layer, or neither.

## What you never do

- **You never start a second try** or open another layer in the same session.
- **You never spend money** that the owner did not approve in `.headwater/probe.yml`.
- **You never apply the keep rule by hand**, and never edit a figure that `result` wrote.
- **You never merge.** The owner or the merge queue merges your pull request.
