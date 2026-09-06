---
id: HW-OBL-0147
status: current
status_since: 2026-09-06
summary: "The skills fixture suite passes on zero verbs counted, and fails in every clone that has not yet built the engine."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
last_verified: 2026-08-26
title: "The skills fixture suite passes on an empty input set, and always fails in a clone that has not built the engine"
waiting_on: adopter
---

# The skills fixture suite passes on an empty input set, and always fails in a clone that has not built the engine

## Context

Both defects surfaced while verifying pull request #328, which closed issue #309, and both predate that pull request rather than being caused by it. `.claude/skills/fixtures.sh` is the suite that checks the repository's own instruction files against the verbs they name, and it is the mechanism at issue here. Neither defect has a reader outside this repository, which is why it is self-audit rather than adopter-blocking.

## Obligation

Three cases in `.claude/skills/fixtures.sh` count the verbs an instruction file names into a variable called `counted`. Each case then reports a pass message that interpolates `counted` without comparing it against anything. On a scratch tree with every `SKILL.md` and every `.claude/agents/*.md` file removed, one case reports `ok 0 verbs named, and every one of them ships`. The suite exits 0, with all four cases passed. A count that is never checked against a floor cannot distinguish a corpus with verbs from a corpus with none. The Rust suite already carries the fix as a pattern. `engine/crates/cli/tests/verbs.rs` asserts `forms.len() > 20` before it compares. Its verb-index case uses `BTreeSet` equality, which an empty left side cannot satisfy unless the right side is empty too.

A second, unrelated defect fails the same suite in the opposite direction. `.claude/agents/headwater-maintainer.md:26` cites `engine/target/release/headwater` as a repository path, inside a sentence that tells the reader to build it if absent. A link-resolution case in the suite reads that citation as a path that must exist. In any clone that has not compiled the engine, `sh .claude/skills/fixtures.sh` exits 1, with 21 cases passed, 1 failed, and 2 skipped. This is why the benefit of #328, that the verb case now runs without a built engine, stops at that one case. It never reaches the suite's own exit status.

## Discharge

Closing this needs a guard before the comparison, in all three `counted` cases, matching the floor `verbs.rs` already asserts. It also needs one case that pins the guard. That case is an instruction-free scratch tree that exits non-zero and names the empty set in its message, rather than claiming everything shipped. The unbuilt-engine path needs the same treatment, as a skip rather than a failure. With that fix, `sh .claude/skills/fixtures.sh` exits 0 in a worktree that has never compiled the engine, and names its engine-dependent cases as skipped. A pull request that states the case count under both conditions, built and unbuilt, lets a later reader tell a skip from a silent pass. Eighteen further `2>&1` stream merges in the four fixture suites remain unaudited, and are out of scope for this record.
