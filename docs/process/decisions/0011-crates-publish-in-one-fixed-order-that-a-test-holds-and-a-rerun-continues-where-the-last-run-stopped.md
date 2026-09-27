---
id: HW-PD-0011
status: current
status_since: 2026-09-27
summary: "publish-crates.yml walks one leaves-first crate list that publish_order.rs holds against the workspace, skips a published crate so a rerun continues, and waits out a 429."
last_verified: 2026-09-27
title: "Crates publish in one fixed order that a test holds, and a rerun continues where the last run stopped"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: evidenced
relations:
  governs:
    - to: .github/workflows/publish-crates.yml
      verified_revision: sha256:779cd2d5d7fdcfe330d8fa714fc6b753ca9a59a5c0b0af9ae710d21cc6687763
  traces_to:
    - engine/crates/cli/tests/publish_order.rs
---

# Crates publish in one fixed order that a test holds, and a rerun continues where the last run stopped

## Context

Before this record, the reasons lived only in comments of `.github/workflows/publish-crates.yml`: the header (lines 21 to 65 on 2026-09-27) and the comments on the publish loop (lines 116 to 127 and 147 to 152).

`cargo install headwater-cli` needs every workspace crate that the binary depends on to be on crates.io. A crate resolves its dependencies through the registry when it publishes. So each crate must publish after every crate that it depends on.

Three behaviors of crates.io shape the loop:

- `cargo publish` refuses a version that the registry already has.
- The crates.io index makes a new crate available to the next `cargo publish` after an interval that crates.io does not state.
- crates.io limits how fast a new crate name publishes. A new version of a name that already exists has no such limit. A response with status 429 names the time in GMT at which crates.io accepts the next one.

## Decision

One step publishes every crate, in one loop over a fixed list. The list is in the `order` variable of the workflow, leaves first. A person computed it once from the dependency graph, and no run computes it again. A dependency that a crate adds is then a diff in the list, and not a change of order that nobody sees. `engine/crates/cli/tests/publish_order.rs` holds the list against the members of the workspace. It holds the set exactly, and it holds the order wherever one member depends on another.

Before it publishes a crate, the loop asks crates.io whether the crate has this version. If it has, the loop skips the crate. So a second run continues where the first run stopped, and does not start again at the first crate. The request sends a `User-Agent` of its own. The crates.io API refuses the default `User-Agent` of `curl` with a 403, which the loop cannot tell apart from a 404.

After each publish, the loop waits 30 seconds for the index. Nobody measured that interval.

When `cargo publish` fails with a 429, the loop reads the time from the error. It waits until that time and 5 seconds more, and tries the same crate again. Any other failure stops the run, because waiting cannot fix it.

## Consequences

A maintainer recovers from a partial publish with a second run of the job, and never with a yank. At `v0.3.0` the first run stopped at `headwater-compat`, because the index did not yet list `headwater-scaffold` after the 30-second wait. A second run published the four crates that were missing. So the wait is not always enough, and the skip is what makes a short wait safe to retry.

The wait for a 429 has no ceiling. [HW-OBL-0182](../../obligations/0182-the-publish-crates-retry-loop-has-no-retry-ceiling.md) records that gap, and it stays open.

This record states no count of crates. `publish_order.rs` holds the set, and a count in prose goes stale when a crate joins the workspace.
