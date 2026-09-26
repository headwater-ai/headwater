---
id: HW-OBL-0213
status: current
status_since: 2026-09-26
summary: "No CI job runs mkdocs build --strict, and this host has neither mkdocs nor rustfmt, so a new generated page and a clippy change reach main unchecked."
last_verified: 2026-09-24
title: "CI builds no MkDocs site, so a generated page under docs/ is never checked against a strict build before merge"
waiting_on: build
---

# CI builds no MkDocs site, so a generated page under docs/ is never checked against a strict build before merge

## Context

The build of #1051 (PR #1066) and the build of #809 (PR #1065) in run `20260924-0411` each found a check that no gate runs. The product owner ruled both Record.

## Obligation

**No CI job builds the site.** `mkdocs.yml` sets `omitted_files: warn`, and no job runs `mkdocs build --strict`. This host has neither `mkdocs` nor `rustfmt`. So a new generated page under `docs/` reaches main with no check that the site builds.

**The clippy of CI is newer than the local one.** CI clippy 1.98 flags `chunks_exact(3)`, where it wants `as_chunks::<3>()`. It also flags the missing semicolon that a push_str arm keeps after rustfmt wraps it. A build finds both only after a push.

## Discharge

The first part discharges when a required CI job runs `mkdocs build --strict`, or when this record states why the site build stays outside the gate. The second discharges when the local toolchain pin matches CI, or when the build order runs the CI clippy before a push.
