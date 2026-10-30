---
id: HW-PROBE-a-session-renames-the-identifier-index-of-the-graph-crate-and-its-callers
status: current
status_since: 2026-10-30
summary: "A session renames the index module of the graph crate to identifiers, moves its callers in the check, import, scaffold and cli crates, and moves the governs path of HW-DR-0068, so the code builds and that record stays clean."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r3-index-module"
title: "A session renames the identifier index of the graph crate and its callers"
action: refactor_across_files
moments: [read, write, review]
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

# A session renames the identifier index of the graph crate and its callers

## Task

### Phrasing 1

Rename the `index` module of the graph crate to `identifiers`, so callers write `headwater_graph::identifiers::Index`. Update every caller in the workspace, keep it building, and keep `headwater check` clean.

### Phrasing 2

The graph crate's `index` module holds the index of minted identifiers, and the name `index` says too little. Call the module `identifiers` and fix everything that refers to it.

## Target

`engine/crates/graph/src/index.rs`, the index of minted identifiers.

## Key

`patched` over `command:r3-index-module`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/test-passes.sh --check-clean` with a held file under `tools/probe/oracle-args/`. That file names a hidden test in `headwater-cli` and, after `--`, the records that govern the module. `headwater-cli` depends on every crate that names the module, so a caller left on the old path fails the build. The test then requires that the old file is gone, that the new file is there, and that `headwater route "change <new file>" --json` offers HW-DR-0068 as a pointer that a governs anchor reached, with the new file in its `anchors`. A subsystem governs the whole crate by a glob, so the test names the record. `--check-clean` then counts the strict errors on the records after `--`, and requires none. A governs line that still names the old path is the error `relation.target.unresolved` on its record.

**The candidate changes, checked.** `tools/probe/oracle-cases/r3-index-module.sh` holds each one, and `sh tools/probe/probe-oracle-fixtures.sh --cargo` runs them in a copy of the sealed grading tree. The reference change moves the file with `mv`, makes every caller name the new module, and moves the path in HW-DR-0068. A change that moves the file and the records and no caller fails the build. A change that leaves one caller in `engine/crates/cli/src/main.rs` on the old path fails the build. A change that moves the code and leaves the records on the old path fails, because each record then governs nothing. The grading tree as it stands fails, because the old file is still there.

**The route.** `headwater route "change engine/crates/graph/src/index.rs"` printed two `governs engine/crates/graph/src/index.rs` lines on 2026-10-30: HW-DR-0068 and HW-SPEC-graph-build, each by a governs anchor.

**No superseded target, and no file named.** No superseded or contradicted record governs the file. The task names the module and not its path, so a session finds the file and the record.

**The pilot's cause.** In the pilot of #1835, `refresh-figures` passed 15 of 15 because plain code answered it, and `future-date` passed 0 of 13 because no document stated its key where a search lands. The code does not name the record that governs it. So the documentation decides the key, and plain code does not reveal it.
