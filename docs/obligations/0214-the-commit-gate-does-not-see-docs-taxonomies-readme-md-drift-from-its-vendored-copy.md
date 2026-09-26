---
id: HW-OBL-0214
status: current
status_since: 2026-09-26
summary: "Only the required Engine tests job catches a drift between the taxonomy bundles and their vendored copy. So a contributor learns of it at CI and not at commit."
last_verified: 2026-09-27
title: "The commit gate does not see docs/taxonomies/README.md drift from its vendored copy"
waiting_on: build
---

# The commit gate does not see docs/taxonomies/README.md drift from its vendored copy

## Context

Issue #994 reported this drift. The adjudication in run `20260924-0411` refused it as kind 2, because no reader outside this repository meets the drift. The owner accepted the refusal. This record carries the issue, which closes as recorded, not planned.

## Obligation

`.githooks/pre-commit` runs `headwater check --strict`, and it exits 0 when `docs/taxonomies/README.md` differs from `.headwater/packages/headwater-standard/bundles/README.md`. A contributor learns of the drift only from the required "Engine tests" job. There, `the_vendored_bundles_agree_with_a_fresh_publish_of_the_maintained_source` in `engine/crates/cli/tests/publish.rs` compares the whole `bundles/` tree. The drift happened twice, in #407 and in the #7 build of run `20260920-2058`.

Since #350, the README is typed `library_doctrine`, and every rule of the house language regime reads it. So a fix of a language finding in it is an edit of the source, and the vendored copy then differs until the next republish. This change adds no drift check, and the obligation stays open.

The same comparison covers `bundles/` only. The adjudication did not confirm that any test compares `taxonomy.yml` or `conformance.yml` under `taxonomy-source/headwater-standard` with the vendored copy.

Since #1139, the verb that the discharge example names exists, as `taxonomy publish --from <dir> --check`. It compares every member of the vendored copy with a fresh publish of the source, `taxonomy.yml` and `conformance.yml` included. The "Engine tests" job runs it. The hook does not run it, so the drift still reaches a contributor at CI and not at commit, and the obligation stays open.

## Discharge

This record discharges when a local gate fails on the drift before a commit. An example is a hook line that runs `taxonomy publish --from taxonomy-source/headwater-standard --check` only when a staged path is under `docs/taxonomies/` or `taxonomy-source/`. A comparison written into the hook is not a discharge, because a rule has no second copy in a script.

The question reopens as an issue when one of four conditions holds. The first is that "Engine tests" leaves the `Protect main` ruleset. The second is that an artifact an adopter receives comes from the vendored copy. The third is that a rule of `check` reads a file under `bundles/` other than `bundle.yml`. The fourth is that a third late drift costs a run an iteration.
