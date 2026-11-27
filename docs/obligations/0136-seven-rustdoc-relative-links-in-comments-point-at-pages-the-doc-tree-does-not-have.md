---
id: HW-OBL-0136
status: discharged
status_since: 2026-11-28
title: "Seven rustdoc-relative links in comments point at pages the doc tree does not have"
summary: "Seven links in engine comments are written for rustdoc output that no job ever builds, so nothing has ever resolved them."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
last_verified: 2026-11-28
waiting_on: build
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

**Measured on 2026-11-27, on `b6890e117`.** The seven links are still dead. `cargo doc --no-deps --workspace --locked --keep-going` reports 55 broken intra-doc links at distinct places in nine crates. `headwater-check` has 29 and `headwater-resolve` has 8. `headwater-generate`, `headwater-scaffold` and `headwater-lock` have 4 each. `headwater-census` and `headwater-vcs` have 2 each, and `headwater-compat` and `headwater-query` have 1 each. The seven links above are not among them, because rustdoc reads a hand-written HTML path as a URL and does not resolve it. The first run took 52 seconds of wall time on the build host, and a second run with a warm target took 4 seconds. So a `deny` level on `broken_intra_doc_links`, read by a CI step, fails on `main` until the 55 are repaired. Slice D of [#1494](https://github.com/headwater-ai/headwater/issues/1494) left this record at `current` for that reason, under the split rule of its adjudication. Three things were owed then, in this order. First, repair the 55 broken intra-doc links. Second, repair or unlink the seven hand-written links. Third, add the `deny` level and the CI step that reads it. Slice E of #1494 paid all three, and the Discharge says how.

## Discharge

Each crate-to-crate link becomes an intra-doc link, or is rewritten to name the lib target directory rather than the package's short name. The two `meta-schema.yml` links either resolve inside the doc tree or stop being links. The workspace also gains a read `cargo doc` step. That step is a `[lints.rustdoc]` entry in `engine/Cargo.toml` beside the clippy floor [Q23](../decisions/0023-the-engine-lint-floor.md) declared, or a CI step. The choice between them needs an argument, not an assumption. Discharge holds once a deliberately broken intra-doc link fails whatever holds it, and the failure names the file. Until an adopter decides that instrument, the seven links stay dead and unread.

**Discharged on 2026-11-28 by slice E of [#1494](https://github.com/headwater-ai/headwater/issues/1494).** The change repaired 63 links. It repaired the 55 broken intra-doc links, 41 unresolved and 14 that named both the `write` function and the `write!` macro. It also repaired eight hand-written links: the seven above and `probe/src/grade.rs:10`, which resolved by accident. A link to a crate that is not a dependency became a code span. Each `write` link now says `fn@write`. The rule-version sentence that 23 rule files copy now points at the version constant of the trait that the file implements. The probe links to `headwater_probe` and to its own `intake` module became intra-doc links.

**The instrument has two halves, and each half is inert alone.** `engine/Cargo.toml` now has a `[workspace.lints.rustdoc]` table that denies `broken_intra_doc_links`. The `engine` job of `.github/workflows/ci.yml` now has the step "Every intra-doc link in the engine resolves", which runs `cargo doc --no-deps --workspace --locked`. Clippy and `cargo test` do not read the rustdoc table, so a lint entry with no step is read by nothing. A step with no declared level only warns, and a warning in CI reaches nobody. The triage of 2026-09-30 said that a lint entry costs no run time. That is true only because nothing reads it. The step costs run time: 30 seconds on the build host, and 27 seconds in CI's pinned 1.99.0 image.

**The gate reads public items only.** The step documents what `cargo doc` documents by default, so it reads the public population. A comment on a private item is not read, and on 2026-11-28 that population held 20 more broken links, with `--document-private-items`. `private_intra_doc_links` is not denied either: a public comment that links to a private item renders as plain text, and that lint had 159 hits. The table is the first entry outside the rust and clippy lints that [HW-DR-0023](../decisions/0023-the-engine-lint-floor.md) measured. It obeys the rule of that decision, because the same change fixed its hits.

**The decisive fixture.** With the level in place and nothing repaired, `cargo doc --no-deps --workspace --locked --keep-going` exited 101 on the 55. After the repair, the same command with no `--keep-going` exited 0. Then a probe added the line ``/// See [`NoSuchThing`].`` above the first public item of `engine/crates/probe/src/lib.rs`. The command exited 101 with ``error: unresolved link to `NoSuchThing` `` and `--> crates/probe/src/lib.rs:124:1`. A second probe put back ``[`write`]`` at `engine/crates/lock/src/lib.rs:48`. The command exited 101 with ``error: `write` is both a function and a macro`` and `--> crates/lock/src/lib.rs:48:7`. With the `deny` entry removed, the second probe exited 0 and printed the same text as a warning. So the manifest entry is what makes the step fail.
