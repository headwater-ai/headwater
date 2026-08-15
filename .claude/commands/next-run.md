---
description: Run N stacked iterations of the Headwater build order, one subagent each, merging between
argument-hint: "[iteration count, default 20]"
---

Run `$ARGUMENTS` iterations (default 20) of the Headwater build order, one Opus 5 subagent per iteration, **sequentially**. Merge between each, so iteration N+1 branches from N's merge and reads a board N already changed. Parallel runs do not stack; they collide.

You are the parent. The subagents do the volume. Your job is judgment: what to merge, what a stale premise means, which of the agent's surprises is a lesson and which is noise. Almost everything you control sits in two places — **what you put in the next prompt, and what you refuse to take on trust.**

## Keep going

**Do not stop between iterations.** The most common failure of this command is a parent that merges, writes a paragraph about what happened, and waits. Nobody asked for a status report. When an iteration merges, pick the next issue and launch the next agent **in the same turn**. Report when the run ends, when something needs a human ruling, or when the user asks.

Long stretches with no user message are normal and expected. The run is the task; a merged pull request is not a stopping point, it is the middle.

## The ledger

Keep a running ledger at `~/.claude/headwater-build-order-ledger.md`, **on disk, not in your context**. Twenty iterations will compact you at least once, and everything that makes this compound lives in the ledger. Read it at the top of every iteration and append at the bottom. Five sections:

- **Lessons.** What an iteration learned that the next should not rediscover.
- **Open findings.** Entries filed in [13 — Open obligations](docs/spec/13-open-obligations.md), issues you opened, and any defect you are handing forward.
- **Decisions needing an owner.** What you merged but would not merge again without a human ruling. Write it when it happens — the reason is legible then and paraphrased later.
- **Milestone status.** Which milestones closed, whose bar you verified, what the next one assumes.
- **Log.** One line per iteration: issue, pull request, merge commit, verdict, and **what your own verification proved** rather than what the agent claimed.

## Each iteration's prompt

Build it fresh. It must contain all seven parts.

### 1. The procedure the agent follows

Subagents do not get slash commands, so paste this in full. An agent told to "run /next" invents its own idea of what that means.

> **Work exactly one iteration of the Headwater build order** (org project "Headwater build order", `headwater-ai/headwater`), then stop.
>
> **Read the board first**, because it moves under you: `gh project item-list 1 --owner headwater-ai --format json` for status, `gh issue list --milestone "<lowest M with open issues>" --state open`, the milestone epic, and any issue the candidate says it depends on.
>
> **The issue is already picked for you** (see below). For the record, the selection rules are: anything already In Progress and unfinished; then anything labeled `correctness-root`, because [spec 12](docs/spec/12-check-layer.md) says every check trusts it silently; then the item that unblocks the most others; otherwise the lowest issue number in the milestone.
>
> **Check the issue against reality before building.** The body was written at a point in time. Confirm its premise still holds against `docs/spec/`, [9 — The decision register](docs/spec/09-decisions.md) and [13 — Open obligations](docs/spec/13-open-obligations.md). If it does not, say what changed, adjudicate it yourself, and say plainly in the pull request what you decided — do not halt, and do not implement a stale ask.
>
> **Do the work.** Follow `CLAUDE.md`. Branch, small commits, a pull request. The bar is the issue's "Done when", not a reading of the title.
>
> **Write back. This is the part that compounds and it is not optional.** Comment on the issue wherever it was wrong — interfaces that came out different, assumptions that failed, cost that surprised you. Edit the downstream issue bodies this work invalidated, saying in the edit what changed and why. Route any finding that contradicts or sharpens a closed decision to spec 13; spec 9 is a register of settled decisions and accepts no new questions. If the work supplied or killed an instrument for an unmeasured claim, move the `discharges:Qn` label and the matching spec 13 entry. If the milestone order is now wrong, say so with the reason — do not re-plan in silence.
>
> **Report four lines**: what shipped, the pull request, what the next iteration should pick and why, and **what you learned that is written down nowhere yet**. If that last line is empty, say so — it rarely is, and an empty answer usually means the write-back was rushed.
>
> Do not start a second issue.

Add these amendments every time:

- **An honest split is a success condition, not a failure to hide.** If the issue is more than lands in one pull request, split it on the board, take the first sound piece, file the remainder, and return `Refs #N`. In the last run five of nine iterations did this and every one was right.
- **State each write-back as you land it and then verify it took.** An agent once wrote "removing the label from this issue" in an otherwise flawless adjudication and the label was still attached.
- **Every "not made here" in your report needs a tracker.** Scoping something out is often right; leaving it untracked never is.

### 2. The environment traps

Carry the whole list forward — it is cheap to paste and each entry cost somebody an hour.

- `gh issue view` and `gh pr edit` fail with a `projectCards` GraphQL deprecation. Use `gh api repos/headwater-ai/headwater/issues/<N> --jq .body`, and patch bodies with `gh api -X PATCH repos/headwater-ai/headwater/{pulls,issues}/<N> -F body=@file.md`. Read the comments too — corrections live there.
- Worktree isolation refuses heredocs and compound shell. Write scripts to files and run them.
- **Give the agent its own scratch directory**, `$CLAUDE_JOB_DIR/tmp/issue-<N>/`, and tell it your own instruments live in `$CLAUDE_JOB_DIR/tmp/parent-only/` and are off limits. An agent once wrote its checking script onto the exact path holding yours, and your "independent" re-derivation then ran the agent's code against the agent's work with nothing to tell you.
- `Write` is blocked in the shared checkout and `EnterWorktree` is refused from a subagent. Tell it to create one by hand: `git worktree add .claude/worktrees/<name> -b <branch> origin/main`, staging files through its own tmp subdirectory, leaving the shared checkout untouched on `main`, and removing the worktree at the end or saying it is still there.
- `git push -u origin <branch>`, never a bare `git push` — `push.default` is `matching` on this machine, so a bare push also tries to push a stale local `main`. Branch from `origin/main` after a fetch.
- The pinned container is Rust 1.85 while CI runs current stable, so clippy differs and `-D warnings` promotes new lints. Run it with `--user` and a container-internal `CARGO_TARGET_DIR`. `--user` and `rustup component add` cannot both work there, because rustup needs a writable `RUSTUP_HOME`, so a formatter fix is applied by hand. **Capture each step's exit status separately** and check CI rather than trusting a local green run.
- **Never pipe the gate.** `<gate> | <filter> && <next>` reports the filter's exit status, so a failure passes silently.
- **Never merge stderr into stdout on an invariant test.** `headwater check` prints a run statistic to stderr and the artifact to stdout; `2>&1` makes a correct invariant look broken. An agent once wrote "the cache invariant is literally false" into durable memory on the strength of one.
- **A test of the engine must re-resolve the lock.** `headwater check` reads `.headwater/taxonomy.lock` and never the package sources. Run `headwater taxonomy resolve` between editing `packages/` and checking — **and mind the order**: resolving *before* publishing moves the lock with the source, so both sides match again and a comparison you meant to fail passes. That one is mine, from testing `taxonomy diff`.
- Build with `cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml`. **The full suite now outlasts a ten-minute timeout after a widely-depended-on crate changes** — redirect to a file and run it in the background rather than inline.
- **`cargo` runs a target's cases as threads of one process, so `std::process::id()` is not a unique key per test.** A temp-dir helper keyed on the pid alone breaks when a second `#[test]` joins its file, and the symptom is `NotFound` out of `std::fs::copy`, which reads as a missing fixture rather than a race.
- **Commit and push in small steps.** A transport error or an API limit costs everything unbanked, and only pushed commits survive an agent death.
- `git config core.hooksPath .githooks` in the worktree. A `Stop` hook runs `.githooks/pre-commit` and exits 2 on failure; dormant while `--strict` exits 0.
- A rust-analyzer LSP is available. **Never navigate from its `documentSymbol` line numbers** — they point at an item's first doc-comment line, not its signature, and the gap runs to sixteen lines in this codebase. `findReferences` at the wrong line returns "No references found", which is indistinguishable from a true negative. `grep -n 'fn <name>'` first.

**Correct an environment claim the moment you disprove it.** An agent reported an invariant "literally false" and wrote it to memory; it had captured `2>&1` and read a stderr statistic as part of a stdout artifact. Left standing, that would have taught every later iteration to weaken the strongest test in the repository. If an agent's environment claim contradicts a recorded lesson, measure it yourself before it propagates — and if the agent wrote it somewhere durable, go and fix that too.

### 3. The Lessons, verbatim — and give the list a sink

This list is append-only by default and it reached 76 entries before somebody pruned it. Line for line that is the register-with-an-inflow-and-no-sink this command calls a finding when it meets one anywhere else. **Before each run, retire and consolidate**: merge entries that are one lesson in three wordings, and **move the entries that are review questions rather than construction rules into part 7**, where you verify. The injective-key test, the parse-boundary `Option`, the location of an empty arm and the treatment-arm test are things *you* ask of a branch, not things an agent applies while building. A seed too long to hold is a seed that gets skimmed.

### 4. The Open findings, with the ones this issue touches called out by name

An agent handed the question its predecessor filed settles the question; an agent without it builds around the question and files a second copy. **Say plainly which entries are the agent's to close and which are to record against.** Name the ones it must not "fix" by choosing a value — an escalated finding resolved by an agent is the same act it records.

### 5. The state of the system as of this merge

The invariants that must stay true, the blocking CI steps, the crates and verbs the work will touch, the structural idioms worth copying, and **the current numbers** — instance counts, finding counts, coverage, test suites, cache arithmetic.

Then: **state exactly why each number moves, and re-bless recorded fixtures deliberately.** Numbers are the cheapest tripwire you have. An agent that must explain a delta notices the delta it did not intend — that is how one iteration found a runner-decided skip bypassing the cache, because `hits + misses + unkeyed` silently stopped equalling the instance count.

**Give every count its denominator.** "27 findings" and "32 findings" were both true of this corpus on the same commit — one post-suppression, one including suppressed and migration-pending. Two agents reported 32, I wrote both up as wrong, and both were right. An agent handed a bare number reproduces it against a different denominator and reports a delta that is not there.

### 6. Pre-work, if your verification of the last branch found a defect

Hand it over as its own commit, before the issue's own work. **This is the highest-yield mechanism in this command.** One pre-work item early in a run triggered a sweep that found fifteen more stale claims; another was found one component further on by the agent that received it, in the verb whose whole job was the thing being guarded.

### 7. How you verify, and the one test you care most about

Tell the agent what you actually do before merging — that you break the corpus by hand, test a gate in all three directions, attack the numbers that did not move, run the next component and not just the one that changed, re-derive claims with your own scripts, regress implementations, and validate artifacts with tools that are not this system. **Quality rises when the agent knows the report is not the deliverable.**

**Name the one test you care most about and say why.** Several iterations built the thing that test would catch, first. When an issue names its own decisive fixture, quote it back and add *build it and make it fail before you make it pass*. When the bar is one whose failure destroys a user's data rather than reporting something wrong, say that too — it changes what gets built.

**Tell the agent to copy an existing mechanism rather than reinvent one, and name the mechanism.** This has twice turned into an audit of the mechanism itself: once when a downstream verb was found carrying the identical defect its upstream had just fixed, and once when the precedent held up as the model for "all or nothing" turned out to be a bare write loop that left half-written trees. **Pointing an agent at a precedent is how the precedent gets read.**

## Between iterations, before merging

**Verify independently and adversarially.** Reset a scratch worktree to the branch and run the suite, the linter and the CLI. That is necessary and nowhere near sufficient — it catches almost nothing the agent has not already caught. What actually finds things:

- **Make the new thing fail.** Break the corpus by hand and confirm the new rule fires with a message naming the offender. A check that fires on no input is indistinguishable from one that does not work. When a change reports *fewer* findings, attack that hardest.
- **Test a gate in three directions, not two.** A real drift must fail it; an ordinary edit elsewhere must not; and **a change to a source must make the derived artifact go stale**. Only the third separates a derived artifact from a snapshot somebody typed.
- **Choose the edits yourself.** Run the agent's own decisive scenario with *your* documents, not the ones its fixtures use. Twice this has found the test narrower than its name.
- **Attack a number that did not move.** Inject defects into the *new* material. It is the only way to tell "the new documents are clean" from "the new documents are on a shelf no rule reaches".
- **Check a new rule gained instances, not just existence**, and make the agent predict the number before running it.
- **Run the next component, not just the one that changed.** A correctness root's own suite cannot fail on a defect that lives one component downstream.
- **Regress the implementation to prove a new test can fail, with a different regression from the agent's.** This is how you learn that a fix is unheld: revert it and watch the whole suite stay green. **Note when a regression will not compile** — that is the strongest result available, because it means the wrong state is unrepresentable rather than merely tested for.
- **Report cache-hit counts, not just verdicts.** A byte-identity differential compares verdicts and cannot see a rule that has become permanently unkeyed. Read keyed and not-keyed before and after and check the delta is the arithmetic of what was added.
- **Re-derive one claimed number by hand**, with your own script, naming the denominator.
- **Validate artifacts with something that is not this system** — an external parser, a published schema, a stock validator.
- **Reproduce a performance claim in the build configuration the claim implies.** Debug and release differ by an order of magnitude.

**Ask the review questions.** Is the named value **injective** over the states that change the verdict — not "is the input named?" but "can two states that must differ produce one key?" Is there an **`Option` at a parse boundary** destroying the difference between "observed nothing" and "did not observe", where the wrong reading is always the one that returns green? Does an **empty arm have a location** — a row nobody wrote, or a column nobody declared? Does the **treatment arm prove the instrument** or only the subject?

**When your own check contradicts the agent, suspect your check first.** Across four runs I have been the one who was wrong more often than not. The most expensive was a denominator — see part 5. The rest, briefly, because each is live: deriving a GitHub anchor slug by the wrong rule (each space maps to one hyphen rather than collapsing runs, and `_` is kept); counting citations from a test fixture that is not corpus content; reading a token that survived `--fix` as a miss when the rule's table is a closed set that never contained it; reading a stale lock the engine deliberately never reads from source; resolving before publishing so both sides of a comparison moved together; and running a git-dependent fixture suite over a `git archive` export, where the commit-gate cases fail for want of a `.git` — a real `git worktree` of the same commit passes clean.

**If the agent was right, say so plainly, correct anything you already wrote down, and move on.** Two adjacent traps: a broken path and a broken fragment are different defects and a fragment checker sees only one, so check the path resolves before the anchor; and a deliberate fixture that is *meant* to dangle shows up in every sweep, so establish the baseline on `main` before reading the branch's number as a regression.

**Use Fable as a second opinion, judiciously.** `Agent` takes `model: fable`, which gets you a genuinely different model rather than another instance of the one that wrote the work and the one checking it. Opus verifying Opus shares blind spots, and the failure this whole command is built against — a confident, well-argued, plausible claim nobody re-derives — is exactly what a same-model reviewer is worst at catching. It has found a live defect in three of the last eight branches.

Spend it where a second reading changes what you do: a ruling you are about to merge; anything heading for *Decisions needing an owner*; a claim you cannot cheaply re-derive where the cost of being wrong is a milestone; the largest change of a run; and the escalation list at the end. Not on routine merges.

**Give it the repository, not only the artifact and the argument** — handed only an artifact a refuter can test coherence, and coherence-testing yields the cheapest hit available (*outcome right, reasoning wrong*), which is real and also the class to discount hardest. **Ask it to refute rather than review**, and tell it "I could not break this" is a respected answer, because a refuter who must produce something produces something. Then treat what comes back as a second opinion: **verify its checkable claims before acting on any of them** — it will be confidently wrong too, just in different places, and that asymmetry is the entire value.

**Know the limit of your own verification.** For forty-five iterations across three runs, merge-refusal never once fired, and the honest description was not "your job is what to merge" but "verification shapes what merges, and merging has been unconditional". **In the fourth run it fired twice** — both times as *send it back to the agent that wrote it*, not as a rejection, and both times the work came back better than the correction asked for. So the veto is real, but it is a resume rather than a bin. Two consequences still hold:

- **A run can rewrite a bar and then close against it.** This command licenses replacing an unmeetable bar with what is true, and licenses closing an epic whose bar you verified. Together they let a run set its own finish line. When you rewrite a Done-when and later close against the rewritten one, **say so in the closing comment and flag it for ratification**.
- **Your verification is iteration-grained and some defects only accumulate.** The worst finding of one run — that every agent-drafted document carried an acceptance stamp the drafting agent typed — was found because one issue happened to measure it. **Once a run, ask what would only be visible across all of it**, and measure it rather than recalling it.

## Policy, so nobody stops to ask

- **You adjudicate a stale premise; the agent does not halt.** Escalate to the human only when the redirect would change milestone order, or when a decision needs an owner rather than an answer.
- **If an issue is three pieces, say so and pick it anyway** — unless it will not land in one pull request, in which case split it on the board and take the first piece. A sound first piece plus honestly-filed follow-ups beats a rushed whole.
- **When an agent refuses to build something the issue names, adjudicate the kind of refusal.** "Nothing states what this is, so I shipped a verb that reports the gap" is sound engineering and merges. "This would be better done another way" is a product decision: merge only if it is reversible and recorded with the condition that reopens it, **written into the artifact where the next reader meets it** and not only into a pull request nobody re-opens. The third kind is the best outcome of all — "the issue's own Done-when offers a second outcome and the measurement says take it" — which is a completion, not a refusal.
- **`Refs #N` with a measured reason beats `Closes #N` on a bar that was stretched.** But read the decline rate in both directions: a persistent one is also a fact about **the grain the issues were written at**, and a growing share of them are written by this process. If declines cluster, write smaller issues rather than praising the honesty of the split.
- **A recorded blocker is not a measured one.** Three times in one run a milestone or an issue was parked on an assessment nobody had re-taken — a register that counted unresolved citations for several iterations, a denominator recorded as zero that was six, and a whole milestone closed as "externally blocked" six iterations before the verbs that dissolved the blocker shipped. **Re-measure a blocker before inheriting it.**
- **A milestone can run out of buildable work without being finished, and that is a finding rather than a scheduling problem.** Say so plainly, name what each remaining issue waits on, leave the epic open, and move to the next milestone rather than manufacturing work the dependency cannot support.
- **Check the board the next iteration will read.** Close any epic whose children are all done, after verifying its Done-when clause by clause against the merged tree. **Close an issue that has become an empty shell** — every deliverable dispersed elsewhere — with a table saying where each piece went; an open issue holding nothing captures the selection rule and misdirects the next pick. **If a bar names work no open issue carries, file that issue yourself.**
- **Harvest the report.** Promote anything actionable into Lessons, add findings to Open findings, and watch the net. A register with an inflow and no sink is itself a finding.
- If CI goes red after a merge, fix it in the next iteration's branch before that iteration's own work. Do not leave a red `main` behind you.
- **If an agent dies mid-iteration**, check its worktree, then **resume it rather than replacing it** — `SendMessage` to its agent id replays its transcript with context intact, where a fresh agent throws away a settled design. Both agents that died in one run resumed and finished their own work correctly, one mid-sentence. Only pushed commits survive.
- **An agent can state a write-back and not land it.** Read what it did, not what it said it would do. Check the board after every iteration, and after every resume especially.
- You may open your own pull request when a merge leaves something stale that no issue owns. Keep it small and separate.
- Never force-push. Never push to `main`. Never merge without verifying. Squash-merge when a branch's history carries a garbled commit message.
- Stop early and report if two consecutive iterations fail their own bar. Something upstream is wrong and iteration three will not find it.
- **Verify `main` itself once at the end.** Every merge was verified on its branch; nothing yet has verified their composition.

## Report

Only at the end of the run, or when a human asks.

A table of issue, pull request and result. Then the adjustments you made between iterations and why — including **what your verification caught that a report did not**, and **what it caught that turned out to be your own error**, because that is the part a reader calibrates against. If you took a second opinion from Fable, say what it changed. Then anything you would not merge again without a decision from the human. Then the ledger's path.
