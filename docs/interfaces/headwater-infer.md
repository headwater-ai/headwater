---
id: HW-IFACE-headwater-infer
status: draft
status_since: 2026-08-25
summary: "How to report uncovered findings as adoption tasks and optionally append them to the taxonomy lock."
last_verified: 2026-08-25
title: "headwater infer"
relations:
  governs:
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/src/main.rs
---

# headwater infer

## Synopsis

    headwater infer [--owner <name>] [--until <date>] [--write] [--now <date>] [--root <path>]

The command reports findings that no existing adoption task accounts for, grouped into document and rule pairs.

## Description

Without `--write`, the command prints the proposed payload and changes nothing. With `--write`, it appends new tasks to the authored adoption block in `.headwater/taxonomy.lock`.

It mints identifiers above every declared one, deduplicates pairs by document and rule, and prints what it carried through. A second run with the same inputs proposes no new task.

## Preconditions

The repository must load through its committed taxonomy lock and corpus. The host date must exist unless `--now` supplies one. `--until` cannot be before the evaluation date.

`--write` requires `--owner`. The lock's adoption block must be readable if it exists.

## Options

| Option | What it does |
|---|---|
| `--owner <name>` | Names the person or team that owns proposed debt. Required with `--write`. |
| `--until <date>` | Sets the task expiry date. It defaults to ninety days after the evaluation date. |
| `--write` | Appends proposed tasks to the lock. |
| `--now <date>` | Sets the evaluation date in `YYYY-MM-DD` form. |
| `--root <path>` | Selects the repository to load. |
| `--no-color` | Confirms color-free output. |

## Exit status

**0** means that the report completed. Findings in the report do not change this status.

**1** means that the repository, date, options or adoption block could not be read or written.

**1** also means that the payload this run built did not load, which is a defect in this engine and not in your corpus.

## Environment

The command reads the system date when `--now` is absent. It reads no other environment variable.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.lock` and corpus | Read to run checks and inspect existing adoption tasks. |
| `.headwater/taxonomy.lock` | Written only with `--write`, by appending proposed tasks. |

## See also

[`headwater check`](headwater-check.md) produces the findings. [`headwater taxonomy resolve`](headwater-taxonomy.md) refreshes the lock after source changes.
