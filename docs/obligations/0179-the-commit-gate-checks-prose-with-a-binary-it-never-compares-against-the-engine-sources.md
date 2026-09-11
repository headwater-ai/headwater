---
id: HW-OBL-0179
status: current
status_since: 2026-09-08
summary: "The figures clause of the commit gate refuses when its binary is behind the engine sources, and the prose clause above it asks nothing."
last_verified: 2026-09-08
title: "The commit gate checks prose with a binary it never compares against the engine sources"
waiting_on: build
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - .githooks/pre-commit
---

# The commit gate checks prose with a binary it never compares against the engine sources

## Context

[HW-DR-0039](../decisions/0039-q39-how-a-figure-reaches-a-hand-built-page.md) gives `tools/site/refresh-figures.sh` a third outcome. The script establishes that no file under `engine/` is newer than the binary it measures with. It exits `3` when one is. That guard closes one case. A stale engine calls a correct page stale, and the gate then names a command that writes the wrong number over the right one.

`.githooks/pre-commit` selects the same binary for its own prose clause, at lines 145 to 151. It runs `check --strict --change` with that binary at line 175. That clause runs on every commit, and the figures clause runs only after it passes. Nothing between the two asks whether the binary is behind the tree. So one clause of this gate holds the question and the clause above it does not.

The consequence is a wrong verdict rather than a wrong write. A binary that predates a rule cannot fire the rule. The gate then passes a commit that the current engine refuses, and the refusal arrives in CI instead. A binary built ahead of the tree fires a rule the tree does not declare. The gate then refuses a commit that nothing is wrong with. `.claude/hooks/lib.sh` resolves the same two paths for the harness hooks, and it asks nothing either. Those hooks bind nothing, so the gate is where this is worth paying for.

Measured on 2026-09-08 against this branch. Seventeen files name one of the two binary paths. Two of them gate or write with the binary, and one of those two compares it against `engine/`.

## Obligation

The corpus owes the same question at the top of the gate that HW-DR-0039 puts in the middle. Or it owes a stated reason why the prose clause does not need it. Both answers are a ruling and then a build. Neither is a copy of the figure guard. That guard exits `3` and writes nothing. A gate that refused every commit on a tree with an unbuilt engine change would be refused by its own authors.

Two facts price the ruling. `CLAUDE.md` states that the gate fails open with one printed line when there is no engine at all. Failing open there loses a finding and writes nothing. A gate that fails open on a stale engine loses the same finding, and it writes nothing either. So a printed line may be the whole remedy the prose clause needs. That is a different answer from the one the figures clause needed.

**A claim this record does not carry, because it did not survive a reading.** The adjudication of [#679](https://github.com/headwater-ai/headwater/issues/679) recorded that `HEADWATER_BLESS=1` reads whatever binary is in the tree. It recorded that a bless run then rewrites the committed fixtures with it.

Twenty-three test targets under `engine/crates/` read that variable. Every documented caller of it runs `cargo test`, which compiles the current sources first. A bless run therefore records with an engine built from the tree in front of it. The hazard named there is not live. The `headwater generate` writing path is a separate question, and [#704](https://github.com/headwater-ai/headwater/pull/704) answered it for `--check` alone.

## Discharge

A ruling states what the prose clause of the gate does when its binary is behind `engine/`. The three answers are refuse, warn and say nothing. Where it warns or refuses, `.githooks/pre-commit` carries the test. `.githooks/fixtures.sh` provokes it the way the figures clause is provoked. Touch a tracked engine source in the scratch tree, so the staged binary reads as behind. Then assert the sentence that the clause prints, and read no second build.

Where the ruling is that the clause says nothing, the reason goes into `.githooks/pre-commit`. It belongs in the comment above the engine selection, beside the paragraph that states why the newer profile answers. This record then closes on that comment.
