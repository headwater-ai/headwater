---
id: HW-DR-0099
status: current
status_since: 2026-09-29
summary: "Two corpora in one tree, or a vendored corpus beside its host, can mint one identifier, because one namespace plus a literal can spell another. The engine loads one root and one lock, so no rule compares schemes across two locks. An adopter picks namespaces that no pattern can spell from another, or resolves a scratch overlay that declares both corpora's schemes."
last_verified: 2026-09-29
title: "A namespace does not keep two corpora apart, and no rule compares identifier schemes across two locks"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  constrains:
    - HW-DR-0009
---

# A namespace does not keep two corpora apart, and no rule compares identifier schemes across two locks

## Context

[HW-DR-0009](0009-multi-repository-corpora.md#context) says that namespaced identifiers stop two corpora in one tree from colliding. [Spec 1](../spec/01-conceptual-model.md#the-corpus) and [spec 3](../spec/03-authoring-and-lifecycle.md#identifiers) made the same promise, and so does one paragraph of [HW-EVAL-graph-export-and-federation](../evaluations/graph-export-and-federation.md). The promise is false, and [#1368](https://github.com/headwater-ai/headwater/issues/1368) reported it.

**A namespace does not make two schemes disjoint.** One namespace plus a literal can spell another namespace. The scheme `{namespace}-{slug}` under the namespace `ZED-SPECIFICATION` and the scheme `{namespace}-SPECIFICATION-{slug}` under `ZED` both admit `ZED-SPECIFICATION-scope`. The test `a_namespace_that_spells_another_with_its_literal_is_not_disjoint` in `engine/crates/meta/src/identifier.rs` holds that pair as overlapping ([#1357](https://github.com/headwater-ai/headwater/issues/1357)).

**Inside one lock, a rule already refuses such a pair.** The identifier-integrity rule of [spec 2](../spec/02-taxonomy-model.md#the-meta-schema) compares every pair of schemes in the resolved taxonomy, whatever their namespaces. `headwater taxonomy resolve` writes no lock while one pair overlaps. `headwater new` reads the same comparison when it proposes a scheme. Both readers see one resolved taxonomy, and so one lock.

**Across two locks, nothing compares them, and the engine has no place to do it.** Three facts, measured on 2026-09-30 against `main` at `a0736315`, stop a rule from being written now:

1. Every verb takes one `--root` and reads one `.headwater/taxonomy.lock`. No code in `engine/crates/` looks for a second lock under a root.
2. Spec 1 says that a path resolves to exactly one corpus, and that the most specific root wins. No code implements that rule, so the engine has no index of the corpora in one tree.
3. The tier above a corpus that would read two exports side by side does not exist. [HW-OBL-0006](../obligations/0006-route-latency-at-a-harvesting-tier-is-unmeasured.md), [HW-OBL-0020](../obligations/0020-whether-a-solution-corpus-vendors-each-source-export.md) and [HW-OBL-0093](../obligations/0093-whether-a-harvesting-tier-owes-conformance-rules-of-its-own.md) hold what that tier still owes.

A cross-lock rule would therefore need a new model of how the engine finds corpora. That is a product decision and not a repair.

## Decision

**No rule compares identifier schemes across two locks in one tree, or across a host corpus and a corpus vendored into it.** The identifier-integrity rule covers one lock only. A namespace groups the identifiers of one corpus and makes a collision unlikely. It does not rule one out.

**Until a rule exists, the adopter keeps two corpora apart.** Two steps do this:

1. Choose namespaces so that no namespace, followed by a hyphen, begins another. `ACME` and `BETA` are safe. `ZED` and `ZED-SPECIFICATION` are not.
2. To test a pair, copy the `.headwater/` directory of one corpus to a scratch directory. Add each scheme of the other corpus to the copy's overlay, under a scheme name the copy does not use, and run `headwater taxonomy resolve --root <scratch>`. The verb exits 1 and names the pair when two schemes admit one string. It exits 0 when every pair is disjoint.

The second step was measured on 2026-09-30. The probe added a scheme `{namespace}-{seq:04d}` under the namespace `HW-DR` to a copy of the overlay of this repository. The verb exited 1 and named `decision_id` as the other half of the pair. The same scheme under `ACME` let the verb exit 0.

**This record constrains HW-DR-0009 and does not rewrite it.** The sentence in its Context that says namespaced identifiers stop two corpora from colliding is superseded by this record. The same sentence in [HW-EVAL-graph-export-and-federation](../evaluations/graph-export-and-federation.md) is superseded too. The evaluation stays as written, because it is evidence of what its authors believed at the time.

## Consequences

Spec 1 and spec 3 now say that a namespace makes a collision unlikely and does not prevent one, and each links here. Spec 1 also stops saying that the root rule reuses machinery that exists.

**The question opens again when the engine gains a place to compare two locks.** That place is one of two things. The first is an index of the corpora in one tree, which the root rule of spec 1 needs. The second is a harvest tier that reads two pinned exports. The comparison then goes there, and it reuses `Template::disjoint` from `engine/crates/meta/src/identifier.rs`, which already decides the pair.

The claim store under `.headwater/ids/` does not change this. It stops two branches of one corpus from minting one value ([HW-DR-0054](0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md)). It never compares two schemes, and it never reads a second lock.
