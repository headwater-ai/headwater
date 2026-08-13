---
description: Run N stacked iterations of the build order, one subagent each, merging between
argument-hint: "[iteration count, default 20]"
---

Run `$ARGUMENTS` iterations of `/next` (default 20), one Opus 5 subagent per iteration, **sequentially**. Merge between each so iteration N+1 branches from N's merge and reads a board N already changed. Parallel runs do not stack; they collide.

You are the parent. Your job is judgment: what to merge, what a stale premise means, which of the agent's surprises is a lesson and which is noise. The subagents do the volume. Almost everything you control sits in two places: what you put in the next prompt, and what you refuse to take on trust.

## The ledger

Keep a running ledger at `~/.claude/headwater-build-order-ledger.md`, **on disk, not in your context**. Twenty iterations will compact you at least once, and everything that makes this compound lives in the ledger. Read it at the top of every iteration and append to it at the bottom. It holds five sections:

- **Lessons.** What an iteration learned that the next one should not rediscover.
- **Open findings.** The [13 — Open obligations](docs/spec/13-open-obligations.md) entries and issue comments prior iterations filed, by number, plus any defect you found in verification that you are handing forward.
- **Decisions needing an owner.** What you merged but would not merge again without a human ruling. Write it when it happens, not at the end, because the reason is legible then and paraphrased later.
- **Milestone status.** Which milestones closed, whose bar you verified, and what the next one assumes.
- **Log.** One line per iteration: issue, pull request, merge commit, verdict, and what your own verification proved rather than what the agent claimed.

Seed **Lessons** with these, which cost earlier runs a cycle each:

- Fix your own wording rather than filing a finding about it. A finding is for someone else's text.
- Prefer deleting a stand-in to adding beside it. Two live paths is the defect the stand-in was meant to avoid.
- Choose a recorded fixture's grain so it survives ordinary prose edits. A fixture that changes on every commit is a fixture nobody reads.
- Never pick a milestone epic as the iteration's issue.
- Apply the `correctness-root` rule only among items whose dependencies already exist.
- When you close a finding, grep for the **sentence the change falsifies**, not for the entry you remember, and grep instruction files as well as prose. A grep for the feature name finds none of them; a grep for the claim finds all of them. One missed entry becomes false in three places, and the sweep that follows finds a dozen more.
- A finding recorded as a clause inside another finding's paragraph is a finding nobody can find again — invisible to a structural audit and to a grep for its feature name. One iteration nearly filed a duplicate of one.
- A derived state must be derived from what runs, not from what is declared.
- A green run is no evidence that a constraint is enforced. The strongest finding can come from a fixture that passes.
- An invariant duplicated into an anonymous `match` arm is invisible to a test suite, because a test is written against a name. One definition carried tests, an inlined second copy carried none, and a one-line behavior change passed all 377 existing tests while three defects hid behind it.
- A rule that stops a thing from existing cannot be scoped to that thing. Scope follows the surviving artifact, not the defect.
- A projection is a good place to put pressure on a schema, and the pressure arrives as "I would have to print something false here".
- Measure the artifact before arguing about the mechanism. Two issues and a milestone epic argued for three rounds about which emitter could write a file; counting the file's headings that stood over zero documents took ten minutes and settled it outright.
- A recorded gap is not a measured one. A register counted 1519 unchecked citations for several iterations as a scope problem, and nobody had ever resolved the 1519. A twenty-line script did, and found two broken — one inside the register itself.
- Replace an unmeetable bar with what is true, rather than leaving it standing. One milestone clause survived two milestones and three separate measurements proving nothing could satisfy it.
- Never pipe the gate. `<gate> | <filter> && <next>` reports the filter's exit status, so a failure passes silently.
- Redirect stderr away from an invariant test; never merge it with `2>&1`. A run statistic on stderr next to an exact artifact on stdout will look like a broken invariant to anyone who captures both.
- Commit and push in small steps. A transport error costs you everything unbanked.

## Each iteration's prompt

Build it fresh. It must contain all six:

1. **The body of `.claude/commands/next.md`, pasted in full.** Subagents do not get slash commands, so an agent told to "run /next" invents its own idea of what that means.

2. **The environment traps.** `gh issue view` and `gh pr edit` fail with a `projectCards` GraphQL deprecation, so pass explicit `--json` or `--template`, and patch bodies with `gh api -X PATCH repos/headwater-ai/headwater/{pulls,issues}/<N> -F body=@file.md`. Worktree isolation refuses heredocs and compound shell, so write scripts under `$CLAUDE_JOB_DIR/tmp` and run them — **and tell the agent to use a subdirectory of its own, `$CLAUDE_JOB_DIR/tmp/issue-<N>/`, because the agent shares that directory with you** (see *Keep your instruments out of the agent's reach*). Use `git push -u origin <branch>`, never bare `git push`, which also tries to push a stale local `main`. Branch from `origin/main` after a fetch. The container pins Rust 1.85 while CI runs current stable, so clippy differs and `-D warnings` promotes new lints: run the pinned container with `--user` and a container-internal `CARGO_TARGET_DIR`, capture each step's status separately, and check CI rather than trusting a local green run. Note that `--user` and `rustup component add` cannot both work there, because rustup needs a writable `RUSTUP_HOME`, so a formatter fix has to be applied by hand. Carry the traps a prior iteration hit forward — that list grows and it is cheap to paste.

   **Correct an environment claim the moment you disprove it.** An agent reported an invariant "literally false" and wrote that to its memory; it had captured `2>&1` and read a stderr statistic as part of a stdout artifact. Left standing, that would have taught every later iteration to weaken the strongest test in the repository into a filtered comparison. If an agent's environment claim contradicts a recorded lesson, measure it yourself before it propagates — and if the agent wrote it somewhere durable, go and fix that too.

3. **The Lessons section**, verbatim.

4. **The Open findings section**, with the ones this issue touches called out by name. An agent handed the question its predecessor filed settles the question; an agent without it builds around the question and files a second copy. Say plainly which entries are the agent's *to close* rather than to record.

5. **The state of the system as of this merge.** The invariants that must stay true, the blocking CI steps, the crates or modules the work will touch, the verbs that exist, and **the current numbers** — instance counts, finding counts, coverage, test suites. Then: *state exactly why each number moves, and re-bless recorded fixtures deliberately.* Numbers are the cheapest tripwire you have. An agent that must explain a delta notices the delta it did not intend.

6. **How you verify, and any pre-work.** Tell the agent what you actually do before merging — that you break the corpus by hand, test a gate in all three directions, attack the numbers that did not move, re-derive claims with your own scripts, regress implementations, and validate artifacts with tools that are not this system. Quality rises when the agent knows the report is not the deliverable. If your verification of the previous branch found a defect, hand it over as **pre-work**: its own commit on this branch, before the issue's own work. That mechanism is the highest-yield thing in this command — one pre-work item early in a run triggered a sweep that found fifteen more stale claims.

Norms transfer too, and they hold better than instructions do. "The last four iterations each landed prose with zero new advisory findings — match that" is worth a line. So is the same sentence about the findings register itself: told that the last N iterations opened zero new obligations, five consecutive iterations opened zero, across a change that doubled the corpus. State the streak and the agent protects it.

## Between iterations, before merging

**Verify independently, and adversarially.** Reset a scratch worktree to the branch and run the suite, the linter and the CLI. That is necessary and it is nowhere near sufficient — it catches almost nothing an agent has not already caught. What actually finds things:

- **Make the new thing fail.** Break the corpus by hand and confirm the new rule fires with a message that names the offender. A check that fires on no input is indistinguishable from a check that does not work. When a change reports *fewer* findings, that is the case to attack hardest.
- **Test a gate in three directions, not two.** A real drift must fail it; an ordinary edit elsewhere must not; and **a change to a source must make the derived artifact go stale**. The first two are necessary and neither proves the artifact is derived rather than a snapshot. Renaming a source record's title and watching the gate exit 1 is the test that does.
- **Attack a number that did *not* move.** When a change doubles the corpus and the finding count is unchanged, that is a claim, not a reassurance. Inject defects into the *new* material and confirm they fire: it is the only way to tell "the new documents are clean" from "the new documents are on a shelf no rule reaches".
- **Check that a new rule gained instances, not just existence.** A rule added without instances cannot fire, and the totals will look healthy either way. Confirm the instance count moved by the amount the rule's scope implies, and make the agent predict that number before running it.
- **Re-derive one claimed number by hand.** Count the words yourself. Parse the artifact with a different tool. Diff the digest against the parent commit.
- **Regress the implementation to prove a new test can fail.** A differential nobody has seen fail is a differential that proves nothing.
- **Validate artifacts with something that is not this system** — an external parser, a published schema, a stock validator.
- **Re-run the experiment that found a prior defect**, against the fix.
- **Reproduce a performance claim in the build configuration the claim implies.** Debug and release differ by an order of magnitude, and an argument resting on "30 ms" needs the build where that is true.

**Keep your instruments out of the agent's reach.** A subagent shares `$CLAUDE_JOB_DIR/tmp` with you, and one of them wrote its own checking script onto the exact path holding yours. Your "independent" re-derivation then runs the agent's code against the agent's work, and nothing tells you. Keep parent verification scripts in `$CLAUDE_JOB_DIR/tmp/parent-only/`, never reuse a filename an agent might reach for, and tell each agent to work in a subdirectory of its own.

**When your own check contradicts the agent, suspect your check first.** Four times across two runs I was the one who was wrong: reading a stale lock the engine deliberately never reads from source; deriving an anchor slug by the wrong rule twice over — GitHub maps each space to one hyphen rather than collapsing runs, and it **keeps `_`**, so stripping `*` and `_` together as emphasis markers invents broken anchors; counting citations from a test fixture that is not corpus content; and double-counting linked identifiers so a partition looked twice its true size. Confirm your method before you write up a defect, and if the agent was right, say so plainly and move on. Two adjacent traps worth knowing: **a broken path and a broken fragment are different defects, and a fragment checker sees only one** — check the path resolves before you check the anchor; and a deliberate fixture that is *meant* to dangle will show up in every sweep you run, so establish the baseline on `main` before you read the branch's number as a regression.

**Use Fable as a second opinion, judiciously.** The `Agent` tool takes `model: fable`, which gets you a genuinely different model rather than another instance of the one that wrote the work and the one checking it. That matters: Opus verifying Opus shares blind spots, and the failure mode this whole command is built against — a confident, well-argued, plausible claim nobody re-derives — is exactly the failure a same-model reviewer is worst at catching. The evidence is concrete. An independent Fable 5 review of the three decisions a twenty-iteration run had escalated for a human ruling **agreed with the outcome in all three cases and found the recorded reasoning wrong in two** — the outcomes were safe, the stated reasons were not, and nothing in the run would have surfaced that.

Spend it where a second reading changes what you do:

- **A ruling you are about to merge** — an agent choosing not to build the thing the issue names. That is the case where the argument *is* the deliverable.
- **Anything heading for *Decisions needing an owner***, before you write the entry. A second opinion often turns "needs a human" into "needs a better reason", which is cheaper for everyone.
- **A claim you cannot cheaply re-derive yourself**, where the cost of being wrong is a milestone.
- **The escalation list at the end of the run**, as a pass over what you are handing back.

Do not spend it on routine merges. It does not know the codebase, it costs a round trip, and a second opinion on a green test suite tells you nothing you did not already measure. Give it the artifact and the argument rather than the repository, and **ask it to refute rather than to review** — a reviewer finds something to say, a refuter has to land a hit. Then treat what comes back as a second opinion and not a verdict: verify its checkable claims before you act on any of them, because it will be confidently wrong too, just in different places. That asymmetry is the entire value.

**Check the board the next iteration will read.** Close any epic whose children are all done, after confirming its Done-when bar yourself, clause by clause, against the merged tree. An open epic with nothing under it captures the selection rule and misdirects the next pick. **If the bar names work that no open issue carries, file that issue yourself** — twice in one run this was the only thing standing between the run and a closed milestone.

**Harvest the report.** The fourth line of step 6 is "what you learned that is written down nowhere yet". Promote anything actionable into Lessons. Add any new finding to Open findings. Watch the net: a findings register with an inflow and no sink is itself a finding, and once you see it, tell every later prompt to settle where it honestly can rather than file.

**At a milestone boundary**, say so loudly in the log and name what the next milestone assumes. Do not stop, but do not roll past a boundary silently either: milestone order is a planning decision and the run is allowed to discover it is wrong.

## Policy, so nobody stops to ask

- Step 3 of `/next` tells the agent to halt on a stale premise. You adjudicate and redirect instead. Escalate to the human only when the redirect would change milestone order, or when a decision needs an owner rather than an answer.
- If an issue turns out to be three pieces of work, say so in the report and pick it anyway, unless it will not land in one pull request. Then split it on the board and take the first piece. A sound first piece plus honestly-filed follow-ups beats a rushed whole, and late in a run it is usually the right call.
- **When an agent refuses to build something the issue names, adjudicate the kind of refusal.** "Nothing states what this is, so I shipped a verb that reports the gap" is sound engineering and merges. "This would be better done another way" is a product decision: merge it only if it is reversible and recorded with the condition that reopens it, and put it under *Decisions needing an owner* either way. A third kind is the best outcome of all: **"the issue's own Done-when offers a second outcome and the measurement says take it"** — that is a completion, not a refusal. Merge it, but require the reopening condition to be written **into the artifact itself**, where the next reader meets it, and not only into the pull request nobody re-opens.
- **An agent that returns `Refs #N` with a measured reason has done better work than one that returns `Closes #N` on a bar it stretched.** Three of five iterations in one run declined to close, each correctly, and each filed the real blocker as its own issue — which is how a milestone that looked one issue deep turned out to be four and still closed. Say so in the prompt, so the agent knows an honest split is a success condition rather than a failure to hide.
- If CI goes red after a merge, fix it in the next iteration's branch before that iteration's own work. Do not leave a red `main` behind you.
- **If an agent dies mid-iteration**, check its worktree before restarting. A transport error can cost twenty-five minutes of settled design and leave nothing on disk. `SendMessage` to its agent id resumes it with its context intact; a fresh agent throws that away.
- You may open your own pull request when a merge leaves something stale that no issue owns — a claim in an instruction file, a count nobody re-derives. Keep it small and separate from the iteration's work.
- Never force-push. Never push to `main`. Squash-merge when a branch's history carries a garbled commit message.
- Stop early and report if two consecutive iterations fail their own bar. Something upstream is wrong and iteration 3 will not find it.
- Verify `main` itself once at the end. Every merge was verified on its branch; nothing yet has verified their composition.

## Report

A table of issue, pull request, and result. Then the adjustments you made between runs and why — including what your verification caught that a report did not, **and what it caught that turned out to be your own error**, because that is the part a reader can calibrate against. If you took a second opinion from Fable, say what it changed. Then anything you would not merge again without a decision from the human. Then the ledger's path.
