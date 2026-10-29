---
id: HW-PROBE-a-session-records-a-ruling-that-drops-the-help-screen-banner-as-the-settled-successor-of-the-record-the-task-names
status: current
status_since: 2026-10-30
summary: "A session writes a current decision that supersedes HW-DR-0045, which places the masthead banner, and HW-DR-0045 ends at superseded with a link to it."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r6-banner"
title: "A session records a ruling that drops the help-screen banner as the settled successor of the record the task names"
action: supersede_or_retire
moments: [write, review]
axes: [names_file]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-DR-0045
---

# A session records a ruling that drops the help-screen banner as the settled successor of the record the task names

## Task

### Phrasing 1

The owner ruled that no screen of the command line prints the masthead banner any more, and that color stays as it is. `--no-banner` and `HEADWATER_NO_BANNER` stay accepted and do nothing. Record the ruling so that it replaces `docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md`.

### Phrasing 2

`docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md` decides color and where the banner goes. Replace it with a decision that keeps its rules on color and drops the banner from every screen, and settle the change in the corpus.

## Target

[HW-DR-0045](../decisions/0045-coloring-the-cli-and-where-the-banner-goes.md), the decision on color and the masthead banner.

## Key

`patched` over `command:r6-banner`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/successor-typed.sh --check-clean` with the record that must be superseded in a held file under `tools/probe/oracle-args/`, which names this probe. The oracle passes when that record is `status: superseded`, a document the engine types names it under `supersedes`, the record names that successor in its `relations:` block as a whole name, and neither carries a strict error.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r6-banner.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change writes a decision with `headwater new decision --relates supersedes=HW-DR-0045`, sets it `current`, and runs `headwater check --fix`, which settles both ends. It passes. The wrong change writes the same decision and leaves it at `draft`, the state the verb opens it at. The far half is owed until the decision leaves `draft`, so HW-DR-0045 stays `current` and names no successor, and it fails. The grading tree as it stands fails.

**The axes.** Each phrasing names the file of HW-DR-0045, so the probe carries `names_file`. No superseded or contradicted record stands between the task and HW-DR-0045, so the probe does not carry `superseded_target`.
