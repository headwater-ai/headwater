---
id: HW-PROBE-a-session-renames-the-module-of-the-check-crate-that-reads-the-state-a-document-stands-in
status: current
status_since: 2026-10-30
summary: "A session renames lifecycle_state.rs of the check crate to standing.rs, moves its callers in the check, generate, lock and scaffold crates, and moves the governs path of HW-DR-0026, so the code builds and that record stays clean."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r3-lifecycle-module"
title: "A session renames the module of the check crate that reads the state a document stands in"
action: refactor_across_files
moments: [read, write, review]
axes: [names_file]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-DR-0112
---

# A session renames the module of the check crate that reads the state a document stands in

## Task

### Phrasing 1

Rename `engine/crates/check/src/lifecycle_state.rs` to `engine/crates/check/src/standing.rs`, so the module is `headwater_check::standing`. Update every caller, and keep the workspace building and the check clean.

### Phrasing 2

The module that reads which state a document stands in is called `lifecycle_state`, which reads like the vocabulary of the same name. Move `engine/crates/check/src/lifecycle_state.rs` to `standing.rs` and update whatever refers to it.

## Target

`engine/crates/check/src/lifecycle_state.rs`, the module of `lifecycle.state.not_admitted`.

## Key

`patched` over `command:r3-lifecycle-module`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/test-passes.sh --check-clean` with a held file under `tools/probe/oracle-args/`. That file names a hidden test in `headwater-cli` and, after `--`, the records that govern the module. `headwater-cli` depends on every crate that names the module, so a caller left on the old path fails the build. The test then requires that the old file is gone, that the new file is there, and that `headwater route "change <new file>" --json` offers HW-DR-0026 as a pointer that a governs anchor reached, with the new file in its `anchors`. A subsystem governs the whole crate by a glob, so the test names the record. `--check-clean` then counts the strict errors on the records after `--`, and requires none. A governs line that still names the old path is the error `relation.target.unresolved` on its record.

**The candidate changes, checked.** `tools/probe/oracle-cases/r3-lifecycle-module.sh` holds each one, and `sh tools/probe/probe-oracle-fixtures.sh --cargo` runs them in a copy of the sealed grading tree. The reference change moves the file with `mv`, makes every caller name the new module, and moves the path in HW-DR-0026. A change that moves the file and the records and no caller fails the build. A change that leaves one caller in `engine/crates/generate/src/probe_result.rs` on the old path fails the build. A change that moves the code and leaves the records on the old path fails, because each record then governs nothing. The grading tree as it stands fails, because the old file is still there.

**The route.** `headwater route "change engine/crates/check/src/lifecycle_state.rs"` printed two `governs engine/crates/check/src/lifecycle_state.rs` lines on 2026-10-30: HW-DR-0026 and HW-SPEC-checks-and-cache, each by a governs anchor. A rename of the file in a scratch copy made `relation.target.unresolved` an error on HW-DR-0026.

**No superseded target.** No superseded or contradicted record governs the file, so the probe takes `names_file` alone. The word `lifecycle_state` also names a vocabulary of the taxonomy, and a session that renames that too breaks the lock.

**The pilot's cause.** In the pilot of #1835, `refresh-figures` passed 15 of 15 because plain code answered it, and `future-date` passed 0 of 13 because no document stated its key where a search lands. The code does not name the record that governs it. So the documentation decides the key, and plain code does not reveal it.
