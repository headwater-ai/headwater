---
id: HW-OBL-0136
status: current
status_since: 2026-09-06
title: "Seven rustdoc-relative links in comments point at pages the doc tree does not have"
summary: "Seven links in engine comments are written for rustdoc output that no job ever builds, so nothing has ever resolved them."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
last_verified: 2026-08-26
waiting_on: adopter
---

# Seven rustdoc-relative links in comments point at pages the doc tree does not have

## Context

[Issue #191](https://github.com/headwater-ai/headwater/issues/191) fixed twenty-nine comment links into `docs/`, and split off the rustdoc-relative case as a separate class. `headwater_check::fragment::comment_links` resolves a comment link against the source tree, so it correctly declines every link whose base is `target/doc/` rather than the source. That decline is not a report. A run on `issue-191-comment-prose` built the tree and resolved each comment link against the output by hand. The build ran `cargo doc --no-deps` over headwater-probe, headwater-sweep, headwater-meta, and headwater-generate.

## Obligation

`engine/**/*.rs` carries eight relative links in comments that do not point into `docs/`. One resolves, `suppression.rs:191`, because it targets the source tree directly. The other seven are written as if rustdoc output were the base, and it is not the base anything checks.

Five of the seven are crate-to-crate links that name a package's short name as a directory. Rustdoc in fact writes each crate under its lib target name, so `sweep/` never exists and `headwater_sweep/` does.

- `probe/src/grade.rs:8` links `../../sweep/index.html`, which stays outside the built tree.
- `probe/src/lib.rs:36` and `probe/src/lib.rs:170` link the same dead `../../sweep/index.html`.
- `probe/src/lib.rs:79` links `../../generate/probe_result/index.html`, one level above anything `cargo doc` writes.
- `generate/src/probe_result.rs:19` links `../../../probe/index.html`, also above the built tree.

The other two, `meta/src/lib.rs:7` and `meta/src/shape.rs:5`, both link `../../meta-schema.yml`, a source path at `engine/crates/meta/meta-schema.yml` that reaches no page of any doc tree. Nobody has noticed, because no job here runs `cargo doc`. So nothing has ever clicked, or resolved, one of the seven. `rustdoc::broken_intra_doc_links` is warn-by-default and would catch a bad intra-doc reference, but it cannot see a hand-written HTML path. Its warning reaches nobody when no build step ever runs `cargo doc` at all.

## Discharge

Each crate-to-crate link becomes an intra-doc link, or is rewritten to name the lib target directory rather than the package's short name. The two `meta-schema.yml` links either resolve inside the doc tree or stop being links. The workspace also gains a read `cargo doc` step. That step is a `[lints.rustdoc]` entry in `engine/Cargo.toml` beside the clippy floor [Q23](../decisions/0023-the-engine-lint-floor.md) declared, or a CI step. The choice between them needs an argument, not an assumption. Discharge holds once a deliberately broken intra-doc link fails whatever holds it, and the failure names the file. Until an adopter decides that instrument, the seven links stay dead and unread.
