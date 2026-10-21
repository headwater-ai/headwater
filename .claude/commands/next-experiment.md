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
3. Run `sh tools/probe/try.sh plan <root>`. On exit 5, report the refusal it prints and stop. On exit 4, put each path it prints to the owner. On the owner's yes, run `sh tools/probe/try.sh accept <root> <ruling-url>` with the URL of that comment, and run `plan` again.
4. Run `sh tools/probe/try.sh launch <root>`. Exit 6 names a state of the checkout or a try root that exists already. Repair the checkout, or stop and ask.
5. Wait in the foreground with a Bash `timeout` of `300000`:

       sh tools/run/wait-for.sh 'sh tools/probe/try.sh wait <root>'

   On `RE-ISSUE`, run the same call again. Never wait in the background.
6. Read the line that `wait` printed. On `canary-needs-reader`, read the records that `<try>/canary-needs-reader` names, and put them to the owner. Write `canary-passed` only on the owner's word, then run `sh tools/probe/try.sh relaunch <root>` and wait again. On `disk-floor`, `capped`, `canary-failed`, `stop` or `driver-exited`, report it to the owner and stop.
7. On `done`, run `sh tools/probe/try.sh result <root>`. It writes the transcript and `result.md`, and it appends the `result` event to `<root>/ledger.jsonl`. Run `sh tools/probe/try.sh ledger <root>` to read the round so far.
8. Open one pull request of your own. It commits the transcript, the figures of `result.md` and the narrative paragraphs the transcript leaves to a person. When the change was not kept, the same pull request reverts it. Name the issue with `Refs`, because the round is not finished.
9. Report to the owner, and stop.

## The report

Report in plain terms first: the scores, what they mean and what the try cost. No interval and no statistical term comes first. For example:

> The tuned version solved 32 of 72 tasks against 39 of 72 before. That is worse, so it is not kept. The try cost $24.52.

Then give the `pooled:` line of `result.md` with its interval, for a reader who wants it. Then name the pull request, the round's spend against its approval from `sh tools/probe/try.sh spent <root>`, and the question the owner owes: a second try, another layer, or neither.

## What you never do

- **You never start a second try** or open another layer in the same session.
- **You never spend money** that the owner did not approve in `.headwater/probe.yml`.
- **You never apply the keep rule by hand**, and never edit a figure that `result` wrote.
- **You never merge.** The owner or the merge queue merges your pull request.
