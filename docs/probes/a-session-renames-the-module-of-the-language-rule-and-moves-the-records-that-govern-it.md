---
id: HW-PROBE-a-session-renames-the-module-of-the-language-rule-and-moves-the-records-that-govern-it
status: current
status_since: 2026-10-30
summary: "A session renames language.rs of the check crate to prose.rs, moves every caller in three crates, and moves the governs paths of HW-DR-0087, HW-DR-0069 and an evaluation, so the code builds and those records stay clean."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r3-language-module"
title: "A session renames the module of the language rule and moves the records that govern it"
action: refactor_across_files
moments: [read, write, review]
axes: [names_file, superseded_target]
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

# A session renames the module of the language rule and moves the records that govern it

## Task

### Phrasing 1

Rename the module `engine/crates/check/src/language.rs` to `engine/crates/check/src/prose.rs`, as `headwater_check::prose`. Everything must still build, and nothing that pointed at the old file may be left pointing at nothing.

### Phrasing 2

The module name `language` in the check crate is too broad for what it holds. Move `engine/crates/check/src/language.rs` to `prose.rs` and update whatever refers to it, so the workspace builds and `headwater check` stays clean.

## Target

`engine/crates/check/src/language.rs`, the module of the language rule.

## Key

`patched` over `command:r3-language-module`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/test-passes.sh --check-clean` with a held file under `tools/probe/oracle-args/`. That file names a hidden test in `headwater-cli` and, after `--`, the records that govern the module. `headwater-cli` depends on every crate that names the module, so a caller left on the old path fails the build. The test then requires that the old file is gone, that the new file is there, and that `headwater route "change <new file>" --json` offers HW-DR-0087 as a pointer that a governs anchor reached, with the new file in its `anchors`. A subsystem governs the whole crate by a glob, so the test names the record. `--check-clean` then counts the strict errors on the records after `--`, and requires none. A governs line that still names the old path is the error `relation.target.unresolved` on its record.

**The candidate changes, checked.** `tools/probe/oracle-cases/r3-language-module.sh` holds each one, and `sh tools/probe/probe-oracle-fixtures.sh --cargo` runs them in a copy of the sealed grading tree. The reference change moves the file with `mv`, makes every caller name the new module, and moves the path in HW-DR-0087, HW-DR-0069 and HW-EVAL-what-a-check-can-know. A change that moves the file and the records and no caller fails the build. A change that leaves one caller in `engine/crates/scaffold/src/lib.rs` on the old path fails the build. A change that moves the code and leaves the records on the old path fails, because each record then governs nothing. The grading tree as it stands fails, because the old file is still there.

The wrong change that moves the file and the records and no caller runs straight after the reference, and it leaves every engine file as old as the grading tree. So it also holds the oracle to building afresh.

**The route.** `headwater route "change engine/crates/check/src/language.rs"` printed three `governs engine/crates/check/src/language.rs` lines on 2026-10-30: HW-DR-0069, HW-DR-0087 and HW-SPEC-checks-and-cache. A rename of the file in a scratch copy made `relation.target.unresolved` an error on HW-DR-0069, HW-DR-0087 and HW-EVAL-what-a-check-can-know, which cites the file under `traces_to`.

**The superseded target.** HW-DR-0069 is `superseded` and still governs the file. A session that moves only the current record leaves the superseded one pointing at nothing, and the oracle counts it.

**The pilot's cause.** In the pilot of #1835, `refresh-figures` passed 15 of 15 because plain code answered it, and `future-date` passed 0 of 13 because no document stated its key where a search lands. The code does not name the records that govern it. Only the records, or the route, name them. So the documentation decides the key, and plain code does not reveal it.
