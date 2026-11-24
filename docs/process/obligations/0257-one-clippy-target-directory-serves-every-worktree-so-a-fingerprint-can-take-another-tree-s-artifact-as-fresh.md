---
id: HW-OBL-0257
status: current
status_since: 2026-11-24
summary: "tools/engine/clippy.sh keeps one target directory for every worktree. This gave a false red on 2026-11-24, and the same trap can give a false green."
last_verified: 2026-11-24
title: "One clippy target directory serves every worktree, so a fingerprint can take another tree's artifact as fresh"
waiting_on: build
provenance:
  warrant: asserted
---

# One clippy target directory serves every worktree, so a fingerprint can take another tree's artifact as fresh

## Context

`hw-run-policy` sends a finding about the build order, its tooling or the CI of this repository to this shelf. Run `20261122-0114` wrote intake line 138. The product owner ruled it RECORD into this record, which its end-of-run report calls P13. This record checked the script again on `0cdd96298`. The incident is as the builder of #1573 reported it.

## Obligation

`tools/engine/clippy.sh` mounts every worktree at `/w` and keeps one target directory under `~/.cache/headwater/clippy/<version>/` for every caller. The mtime fingerprint of cargo can then take a newer artifact of another worktree as fresh. On 2026-11-24 it reported `cannot find Untracked in headwater_vcs` for a function that the tree held, and a `touch` of the changed sources cleared it. That was a false red, and the same trap can give a false green.

## Discharge

This record discharges when a clippy run cannot read an artifact that another worktree built, for example with a target directory for each tree. A change that records why the shared directory stays also discharges it.
