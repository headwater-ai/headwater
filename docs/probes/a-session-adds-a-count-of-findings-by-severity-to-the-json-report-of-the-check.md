---
id: HW-PROBE-a-session-adds-a-count-of-findings-by-severity-to-the-json-report-of-the-check
status: current
status_since: 2026-10-30
summary: "A session adds a counts member to the JSON report of the check and raises the shape version to 1.7, because the interface page says an added member moves the minor, and a hidden test reads the report."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r2-check-counts"
title: "A session adds a count of findings by severity to the JSON report of the check"
action: change_governed_code
moments: [intent, read, write, review]
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

# A session adds a count of findings by severity to the JSON report of the check

## Task

### Phrasing 1

Add a `counts` member to the JSON report that `headwater check --json` writes, in `engine/crates/adapter/src/json.rs`. It holds the number of findings of each severity, as `error`, `warn` and `info`, so a CI job can gate on a count without walking `findings`.

### Phrasing 2

A consumer of `headwater check --format json` wants the number of findings per severity without counting the list itself. Make the report carry a `counts` object with one number for each of `error`, `warn` and `info`. The JSON shape is written in `engine/crates/adapter/src/json.rs`.

## Target

`engine/crates/adapter/src/json.rs`, the JSON shape of `headwater check`.

## Key

`patched` over `command:r2-check-counts`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/test-passes.sh` with a held file under `tools/probe/oracle-args/` that names a hidden test in `headwater-cli`. The test runs the built binary over a scratch corpus that reports at least one `error` and one `warn`. It requires that `counts.error`, `counts.warn` and `counts.info` each equal the number of entries of `findings` with that severity, and that `version` is `1.7`.

**The documented half of the key.** `docs/interfaces/headwater-check.md`, under "Raising the version", says that each added member moved the minor, and the pin writes `1.6`. The code holds the constant and does not state the rule.

**The candidate changes, checked.** `tools/probe/oracle-cases/r2-check-counts.sh` holds each one, and `sh tools/probe/probe-oracle-fixtures.sh --cargo` runs them in a copy of the sealed grading tree. The reference change writes the member, raises the version to `1.7`, and moves the interface page with it. A change that writes the member and leaves the version at `1.6` fails. A change that raises the version and writes no member fails. The grading tree as it stands fails.

**The route.** `headwater route "change engine/crates/adapter/src/json.rs"` printed two `governs engine/crates/adapter/src/json.rs` lines on 2026-10-30: HW-IFACE-headwater-check and HW-SPEC-checks-and-cache, each by a governs anchor.

**No superseded target.** No superseded or contradicted record governs the target, so the probe takes `names_file` alone.

**The pilot's cause.** In the pilot of #1835, `refresh-figures` passed 15 of 15 because plain code answered it, and `future-date` passed 0 of 13 because no document stated its key where a search lands. The version rule is on the interface page and not in the code. So the documentation decides the key, and plain code does not reveal it.
