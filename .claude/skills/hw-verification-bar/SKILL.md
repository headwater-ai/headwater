---
name: hw-verification-bar
description: The adversarial checks a branch of the Headwater build order must survive before it merges, and the review questions behind them, each with the ruling it rests on. Invoke before verifying any branch as hw-verify, and when a parent chooses which attacks to dispatch. It says what finds defects that a green suite cannot, and it decides nothing itself.
---

# The verification bar

Resetting a worktree to the branch and running the suite, the linter and the CLI is necessary and nowhere near sufficient: it catches almost nothing the build agent has not already caught. What follows is what does. The parent chooses which of these to dispatch, by heading, and the verifier runs them and reports which fired, which held and what it could not check. The bar lives here rather than in the parent's command because two stages read it and the parent only chooses from it ([HW-PD-0001](../../../docs/process/decisions/0001-orchestration-prose-has-one-owner-per-sentence.md)).

Three of these run on every branch without being chosen: *Make the new thing fail*, *Re-derive one number by hand*, and *Suspect your own check first*.

## Make the new thing fail

Break the corpus by hand and confirm the new rule fires with a message naming the offender. A check that fires on no input is indistinguishable from one that does not work. When a change reports fewer findings than before, attack that hardest: a lexical rule can saturate and report zero forever.

## Test a gate in three directions

A real drift must fail it; an ordinary edit elsewhere must not; and a change to a source must make the derived artifact go stale. Only the third separates a derived artifact from a snapshot somebody typed. A gate tested in two directions has been tested in the two that pass.

## Choose the edits yourself

Run the build agent's own decisive scenario with your documents, not the ones its fixtures use. Twice this has found the test narrower than its name.

## Attack a number that did not move

Inject defects into the new material. It is the only way to tell "the new documents are clean" from "the new documents are on a shelf no rule reaches". An injection into a document held under the adoption block proves nothing, because a pending finding does not move the count.

## A new rule gained instances

Check that a new rule gained instances and not only existence, and make the build note predict the number before you run it. A predicted number that matches is evidence; a number read after the fact is a description.

## Run the next component

A correctness root's own suite cannot fail on a defect that lives one component downstream. Run the producer of any file that reads a source the change touched, not only the component that changed.

## Regress with a different regression

Revert the fix and watch a named test go red, with a regression the build agent did not use. This is how you learn that a fix is unheld: revert it and the whole suite stays green. Note when a regression will not compile, which is the strongest result available, because the wrong state is unrepresentable rather than tested for.

## Report cache-hit counts

A byte-identity differential compares verdicts and cannot see a rule that has become permanently unkeyed. Read keyed and not-keyed before and after, and check that the delta is the arithmetic of what was added. A widened rule needs a cache `VERSION` bump, so run `headwater check --strict` warm and cold and compare.

## Re-derive one number by hand

One claimed number, with your own script, naming the denominator. Two agents once reported 32 findings where the parent believed 27, and both were right about a different denominator. A number without its denominator is not a claim.

## Validate with something that is not this system

An external parser, a published schema, a stock validator. A corpus-wide artifact that two branches derive the same wrong way merges clean at exit 0 ([HW-DR-0049](../../../docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md)), so the thing that checks it must not be the thing that wrote it.

## Reproduce a performance claim in the right build

Debug and release differ by an order of magnitude. A claim made in one is reproduced in that one, and a claim about the shipped artifact is reproduced with `--release`.

## The review questions

Is the named value injective over the states that change the verdict: not "is the input named?" but "can two states that must differ produce one key?" Is there an `Option` at a parse boundary destroying the difference between "observed nothing" and "did not observe", where the wrong reading is always the one that returns green? Does an empty arm have a location, a row nobody wrote or a column nobody declared? Does the treatment arm prove the instrument or only the subject? And what would the check print if the thing it protects were absent, because a check that cannot run reads as a check that passed.

## Suspect your own check first

When your check contradicts the build note, suspect your check. Across four runs the verifier was wrong more often than the builder, and the most expensive error was a denominator. The recorded shapes: a GitHub anchor slug derived by the wrong rule, a citation counted from a fixture that is not corpus content, a token that survived `--fix` read as a miss when the rule's table never held it, a stale lock the engine deliberately never reads from source, a comparison whose two sides were resolved together, and a git-dependent suite run over a `git archive` export. Establish the baseline on `main` before reading a branch's number as a regression, and check that a path resolves before you check its fragment.

## A second opinion from a different model

`Agent` takes `model: fable`, which is a different model rather than another instance of the one that wrote and the one that checks. It has found a live defect in three of eight branches. Spend it on a ruling about to merge, a claim you cannot cheaply re-derive where the cost of being wrong is a milestone, and the largest change of a run; never on a routine merge. Give it the repository and not only the artifact, ask it to refute rather than review, tell it that "I could not break this" is a respected answer, and verify its checkable claims before acting on any, because it is confidently wrong in different places and that asymmetry is the whole value.
