---
id: HW-OBL-0133
status: discharged
status_since: 2026-10-15
last_verified: 2026-08-26
title: "Two test helpers key a scratch directory on the pid alone, and the race springs on the next case added"
summary: "Two scratch-directory test helpers key on the process id alone, safe only while their files hold one test each."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
waiting_on: build
---

# Two test helpers key a scratch directory on the pid alone, and the race springs on the next case added

## Context

The build-order run's #179 iteration found this while building its own test harness, and recorded the finding only in a pull request body. `cargo` runs the cases of one test target as threads of a single process, so `std::process::id()` does not name a test uniquely.

## Obligation

Two test helpers key a scratch directory on the process id alone. Each calls `remove_dir_all` on that directory before copying a fixture tree in. `engine/crates/scaffold/tests/pipeline.rs` names its directory `headwater-scaffold-{pid}`, and `engine/crates/hash/tests/oracle.rs` names its directory `subject-{pid}`. Both are safe today only because each file holds exactly one `#[test]`.

The moment a second case joins either file, one case's `remove_dir_all` deletes the other's tree mid-run. The symptom reads as `NotFound` out of `std::fs::copy`, which looks like a missing fixture rather than a race. So the first person to hit it looks in the wrong place.

Two other files, `engine/crates/import/tests/fixtures.rs` and `engine/crates/resolve/tests/publish.rs`, already key on `{pid}-{case}` and hold many cases each, so the correct shape exists in the tree twice already. The engine holds six test files that construct scratch directories by hand, across four different naming shapes. Choosing one convention is a small design decision, cheaper to make once than five times.

## Discharge

This closes when no test helper keys a shared path on the process id alone. Either the two outlier files adopt the existing `{pid}-{case}` convention, or a single shared helper replaces all six hand-built call sites. The choice itself needs stating, whichever way it goes. A second `#[test]` added to one of the two affected files, as a fixture that would have raced and does not, stands as the proof.

The first slice of [#1494](https://github.com/headwater-ai/headwater/issues/1494) discharged this record on 2026-10-15. `engine/crates/scaffold/tests/pipeline.rs` already keyed its directory on `{pid}-{test}` and held two cases, so an earlier change paid that half. `engine/crates/hash/tests/oracle.rs` now keys its subject file on the process and on a counter of calls, and it holds a second case, `a_second_implementation_agrees_on_subjects_of_many_blocks`. A scratch mutation put the key back to the process alone. The first case then failed in 3 of 3 runs, because the second case wrote over its subject file. The convention is the `{pid}-{case}` shape that the tree already used, and `DEVELOPING.md` states it under "The loop". Three other keys on the process identifier alone remain. They are in `engine/crates/cli/tests/width.rs`, `engine/crates/conformance/tests/rules.rs` and `engine/crates/check/tests/fixtures.rs`. Each one is inside one `#[test]` body and carries a label that no other case uses, so none of them is a shared helper.
