---
id: HW-OBL-0133
status: draft
status_since: 2026-08-26
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
waiting_on: adopter
---

# Two test helpers key a scratch directory on the pid alone, and the race springs on the next case added

## Context

The build-order run's #179 iteration found this while building its own test harness, and recorded the finding only in a pull request body. `cargo` runs the cases of one test target as threads of a single process, so `std::process::id()` does not name a test uniquely.

## Obligation

Two test helpers key a scratch directory on the process id alone. Each calls `remove_dir_all` on that directory before copying a fixture tree in. `engine/crates/scaffold/tests/pipeline.rs` names its directory `headwater-scaffold-{pid}`, and `engine/crates/hash/tests/oracle.rs` names its directory `subject-{pid}`. Both are safe today only because each file holds exactly one `#[test]`.

The moment a second case joins either file, one case's `remove_dir_all` deletes the other's tree mid-run. The symptom reads as `NotFound` out of `std::fs::copy`, which looks like a missing fixture rather than a race. So the first person to hit it looks in the wrong place.

Two other files, `engine/crates/import/tests/fixtures.rs` and `engine/crates/resolve/tests/publish.rs`, already key on `{pid}-{case}` and hold many cases each, so the correct shape exists in the tree twice already. The engine holds six test files that construct scratch directories by hand, across four different naming shapes. Choosing one convention is a small design decision, cheaper to make once than five times.

## Discharge

This closes when no test helper keys a shared path on the process id alone. Either the two outlier files adopt the existing `{pid}-{case}` convention, or a single shared helper replaces all six hand-built call sites. The choice itself needs stating, whichever way it goes. A second `#[test]` added to one of the two affected files, as a fixture that would have raced and does not, stands as the proof. The issue records the trap rather than patching it, so nothing has discharged this obligation yet.
