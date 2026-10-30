---
id: HW-PROBE-a-session-lets-the-graph-export-list-the-patterns-of-every-anchor
status: current
status_since: 2026-10-30
summary: "A session makes the graph export write patterns for an anchor of one pattern too and raises the export version to 1.4, because the interface page says a member written in more cases moves the minor, and a hidden test reads the export."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r2-export-patterns"
title: "A session lets the graph export list the patterns of every anchor"
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

# A session lets the graph export list the patterns of every anchor

## Task

### Phrasing 1

`headwater export --format json` writes `patterns` on an anchor only when it holds more than one pattern, in `engine/crates/generate/src/export.rs`. Make it write `patterns` for every anchor, and for every edge onto an anchor, so a consumer reads one member in every case.

### Phrasing 2

A consumer of the graph export has to read `patterns` on some anchors and `id` on others. Change `engine/crates/generate/src/export.rs` so that each anchor, and each edge target bound to an anchor, carries `patterns`, and an anchor of one pattern lists that one pattern.

## Target

`engine/crates/generate/src/export.rs`, the JSON shape of `headwater export`.

## Key

`patched` over `command:r2-export-patterns`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/test-passes.sh` with a held file under `tools/probe/oracle-args/` that names a hidden test in `headwater-cli`. The test runs the built binary over a scratch corpus with one record that governs one path and a list of two. Each entry of `graph.anchors`, and each edge target bound to an anchor, must carry `patterns`, and an anchor of one pattern must list its `id`. `version` and `export_version` must be `1.4`.

**The documented half of the key.** `docs/interfaces/headwater-export.md`, under "Raising the version", says that a member the export writes in more cases than before moves the minor, and the pin writes `1.3`. The code holds the constant and does not state the rule.

**The candidate changes, checked.** `tools/probe/oracle-cases/r2-export-patterns.sh` holds each one, and `sh tools/probe/probe-oracle-fixtures.sh --cargo` runs them in a copy of the sealed grading tree. The reference change writes `patterns` for every anchor, raises the version to `1.4`, and moves the interface page with it. A change that writes the member and leaves the version at `1.3` fails. A change that raises the version alone fails. The grading tree as it stands fails.

**The route.** `headwater route "change engine/crates/generate/src/export.rs"` printed two `governs engine/crates/generate/src/export.rs` lines on 2026-10-30: HW-IFACE-headwater-export and HW-SPEC-projections-and-export, each by a governs anchor.

**No superseded target.** No superseded or contradicted record governs the target, so the probe takes `names_file` alone.

**The pilot's cause.** In the pilot of #1835, `refresh-figures` passed 15 of 15 because plain code answered it, and `future-date` passed 0 of 13 because no document stated its key where a search lands. The version rule is on the interface page and not in the code. So the documentation decides the key, and plain code does not reveal it.
