---
id: HW-PROBE-a-session-supersedes-the-standing-ruling-on-figures-of-a-hand-built-page-when-the-task-names-the-record-that-ruling-replaced
status: current
status_since: 2026-10-30
summary: "A session records the new ruling on figures as the successor of HW-DR-0097, which stands today, and not of HW-DR-0039, the record the task names, which HW-DR-0097 superseded."
last_verified: 2026-10-30
probe_category: navigability
expectation: patched
oracle: "command:r6-figure-page"
title: "A session supersedes the standing ruling on figures of a hand-built page when the task names the record that ruling replaced"
action: supersede_or_retire
moments: [write, review]
axes: [names_file, superseded_target]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-DR-0097
---

# A session supersedes the standing ruling on figures of a hand-built page when the task names the record that ruling replaced

## Task

### Phrasing 1

The owner ruled today that a figure on a hand-built page may be typed into the committed page when a `measured on <date>` marker stands beside it, and that the job which publishes the site checks the typed value against a fresh run. The ruling replaces the one recorded in `docs/decisions/0039-q39-how-a-figure-reaches-a-hand-built-page.md`. Record it in the corpus.

### Phrasing 2

Record a new decision: a hand-built page may carry a typed figure if a `measured on <date>` marker stands beside it, and publishing checks that figure against a run. It takes the place of the ruling in `docs/decisions/0039-q39-how-a-figure-reaches-a-hand-built-page.md`.

### Phrasing 3

`docs/decisions/0039-q39-how-a-figure-reaches-a-hand-built-page.md` holds the ruling on how a figure reaches a hand-built page. The owner has replaced it: a typed figure is allowed beside a `measured on <date>` marker, and the publish job checks it against a run. Write the new ruling into the corpus so that it supersedes the old one.

## Target

[HW-DR-0097](../decisions/0097-a-figure-on-a-hand-built-page-is-measured-when-the-site-is-published-and-the-committed-page-carries-none.md), which superseded HW-DR-0039 and is the ruling on figures of a hand-built page that stands today.

## Key

`patched` over `command:r6-figure-page`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/successor-typed.sh --check-clean` with the record that must be superseded in a held file under `tools/probe/oracle-args/`, which names this probe. The oracle passes when that record is `status: superseded`, a document the engine types names it under `supersedes`, the record names that successor in its `relations:` block as a whole name, and neither carries a strict error.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r6-figure-page.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change writes a decision with `headwater new decision --relates supersedes=HW-DR-0097`, sets it `current`, and runs `headwater check --fix`, which sets HW-DR-0097 to `superseded` and writes its `superseded_by` link. It passes. The wrong change writes the same decision with `supersedes=HW-DR-0039`, the record the task names. HW-DR-0039 is already superseded, and HW-DR-0097 stays `current`, so it fails. The grading tree as it stands fails.

**The axes.** Each phrasing names the file of HW-DR-0039, so the probe carries `names_file`. HW-DR-0097 superseded that record on its own subject, so a superseded record stands in the way, and the probe carries `superseded_target`. Both records survive the seal.
