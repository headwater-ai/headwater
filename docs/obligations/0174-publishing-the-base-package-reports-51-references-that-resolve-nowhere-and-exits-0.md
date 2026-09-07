---
id: HW-OBL-0174
status: current
status_since: 2026-09-07
summary: "Publishing headwater/standard 4.1.0 prints that the artifact records 51 references resolving nowhere inside it, exits 0, ships the list in the manifest, and taxonomy vendor installs it without a word."
last_verified: 2026-09-07
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

## Obligation

Two verbs report the same condition at two grains and neither one refuses. The publisher prints a count and exits 0. The consumer prints nothing and exits 0. Between them the artifact carries a manifest field that states the defect and a doctrine directory that demonstrates it. No rule of this engine reads either.

What the corpus owes is not a decision about dead doctrine links, which #633 holds. It owes the record that this behavior reaches a consumer's tree silently. The one message about it is on the publisher's terminal rather than in the consumer's report.

## Discharge

The ruling on #633 settles what a doctrine link to the publisher's own documents becomes in a consumer's tree. Any of its answers gives this record its discharge. A rewrite of the doctrine to carry no reference outside the artifact empties `unresolved_references` and closes it from the other side.

Independent of the ruling, `headwater taxonomy vendor` can state what the manifest already carries. The field is in the artifact the verb just verified, and reading a count out of it costs no new computation.
