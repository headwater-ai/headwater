---
id: HW-OBL-0174
status: discharged
status_since: 2026-09-08
summary: "Publishing headwater/standard 4.1.0 prints that the artifact records 51 references resolving nowhere inside it, exits 0, ships the list in the manifest, and taxonomy vendor installs it without a word."
last_verified: 2026-09-08
title: "Publishing the base package reports 51 references that resolve nowhere and exits 0"
waiting_on: adopter
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - taxonomy-source/headwater-standard/package.yml
---

# Publishing the base package reports 51 references that resolve nowhere and exits 0

## Context

`headwater taxonomy publish --from taxonomy-source/headwater-standard --out <dir>` exits 0 on `c574ddc` and writes `headwater/standard 4.1.0` as 31 files under the digest `sha256:29a906802b41769798619adb54a440ebed6ca9cd3c777cd3507d27f75e615425`. On the way it prints one line to standard error:

    headwater: the artifact records 51 references that resolve nowhere inside it, and this publish carries no other

The published manifest holds them under `unresolved_references`, as a mapping from a carried document to the targets the artifact does not hold. Parsed, it names 9 documents and 51 pairs. Every target is a path into the publisher's own corpus: `spec/02-taxonomy-model.md`, `spec/07-distribution-and-federation.md`, `evaluations/linkml-worked-example.md` and the like. The carried documents are the doctrine prose of the bundles, `bundles/README.md` among them.

`headwater taxonomy vendor` accepts the artifact. Run against a scratch root with the digest passed to `--expect`, it exits 0 and reports 31 files, all of them the pinned bytes. The doctrine lands under `packages/headwater-standard/doctrine/`. It says nothing about the 51 pairs. Its own closing paragraph states that the doctrine directory is prose the publisher wrote for a person to read. It adds that nothing resolves that prose and that no check reads it. So the consumer receives 51 dead links inside the one part of the package a person is told to read.

[#633](https://github.com/headwater-ai/headwater/issues/633) asks the ruling about the same 51 pairs, from the resolver side. This record holds the publisher side of the same measurement and asks for no second ruling.

**The count above is of `c574ddc` and it moved before the repair.** The same publish reported 52 pairs when #633 was adjudicated on 2026-09-08, over 126 occurrences in the same 9 carried files. The title and the summary keep 51 because that is what the run of `c574ddc` printed. Read the number as a measurement of one commit rather than as the size of the class.

## Obligation

Two verbs report the same condition at two grains and neither one refuses. The publisher prints a count and exits 0. The consumer prints nothing and exits 0. Between them the artifact carries a manifest field that states the defect and a doctrine directory that demonstrates it. No rule of this engine reads either.

What the corpus owes is not a decision about dead doctrine links, which #633 holds. It owes the record that this behavior reaches a consumer's tree silently. The one message about it is on the publisher's terminal rather than in the consumer's report.

## Discharge

**This record is discharged, and both halves of it closed at once.** #633 ruled the answer. A publisher writes a doctrine reference to a document the artifact does not carry as an absolute URL. [Spec 7](../spec/07-distribution-and-federation.md#publishing) states that ruling for a publisher who is not this repository. The 52 pairs, over 126 occurrences in 9 carried files, are repaired to `https://github.com/headwater-ai/headwater/blob/main/docs/…`. `unresolved_references` is gone from the source manifest. The publish now exits 0 and writes nothing at all to standard error, so the count this record opened on is absent rather than smaller.

**The consumer half is discharged by absence rather than by a change to `vendor`.** This record also asked that `headwater taxonomy vendor` state what the manifest carries. There is no field to state now, and no consumer of this package receives a dead doctrine link. The gap returns the moment any publisher ships a package that records a pair. `vendor` still reads `unresolved_references`, still exits 0, and still says nothing to the consumer. File that against a package that has such a record, rather than against this one. Nothing here can measure it any more.

**What holds the repair.** `every_relative_link_a_carried_file_writes_resolves_inside_the_artifact` in `engine/crates/cli/tests/publish.rs` walks the published artifact. It refuses a relative link that names a path the artifact lacks. It also refuses one that climbs clear of the artifact root. That second class measures 0, and it reached no check at all before this record closed. The #619 publish rule reads only a target inside the artifact. `link.fragment.unresolved` never opens `docs/taxonomies/**`, because that path sits outside the corpus root.
