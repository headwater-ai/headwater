---
id: HW-OBL-0184
status: discharged
status_since: 2026-09-11
summary: "Issue #381 ruled where doctrine lands inside a package and gave three reasons, none about visibility. The root moved under `.headwater/` on a ruling and not on that measurement, which HW-DR-0064 states."
last_verified: 2026-09-11
title: "Nothing states why the vendored package root is visible in an adopter tree"
waiting_on: ruling
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# Nothing states why the vendored package root is visible in an adopter tree

## Context

**This section states the tree as it stood on 2026-09-11, before the ruling in the Discharge section below moved the root.** `headwater taxonomy vendor` installed a package at `packages/<name>` in the adopter's tree. Every other consumer-side artifact of this engine lands under `.headwater/`. [Spec 7](../spec/07-distribution-and-federation.md) states that this path is outside the corpus root. No census row covers it, and no rule reads it. So `packages/` is the one machine-owned directory that a person meets in an adopter repository. Nothing in this corpus states a reason for the exception. Issue [#792](https://github.com/headwater-ai/headwater/issues/792) asks whether the root moves to `.headwater/packages/`.

Issue #792 records a case against the move, and it cites [#381](https://github.com/headwater-ai/headwater/issues/381). That citation is wrong. Issue #381 asked whether a publisher may send doctrine *prose* to a place where a reader is more likely to look. Spec 7 answers no, at the paragraph that opens *"That path is the only destination"*. It writes down three reasons. The first is the census cost of one untyped file for every page copied into a corpus root. The second is the absence of an upgrade story for a copy outside `packages/`. It also names the absence of a kind that types such a copy, and it rests on the fork ban of spec 2. The third is the price the ruling accepts, that prose under `packages/` answers to no language regime and to no check.

**None of the three is about whether a reader can see the directory.** The paragraph above them states the mechanism that makes vendored doctrine discoverable. The report of `vendor` *"names the installed path, and it states that the directory is prose for a person rather than schema"*. A printed path is indifferent to a leading dot. Issue #381 therefore rules on the destination of doctrine inside the package. It says nothing about the visibility of the package root. It neither blocks the question of #792 nor answers it.

## Obligation

State why the vendored package root is visible, or move it. This corpus owes one of the two, and it states neither today.

- **The root stays at `packages/`.** Then the corpus owes the reason. The directory remains the one exception to the consumer-state boundary of spec 7, and the exception is silent.
- **The root moves to `.headwater/packages/`.** Then a promise that three published interface contracts state row by row breaks, and the corpus owes the migration.

**This record does not choose, and a later reader must not read it as a choice.** The choice belongs to the owner of the product. Spec 7 names the evidence that settles it, in the reopening clause of the same ruling. The clause reads: *"An adopter who reports that nobody finds the prose under `packages/` is the evidence that would ask for one."* No such adopter exists yet, and the milestone *M6 — Distribution* exists to produce one. A ruling taken today is a ruling taken without the measurement that the specification says decides it. It is therefore taken on this repository's own convenience.

**The cost of the move, measured on this tree at `d8690dae`.** 109 tracked files name the path. 17 of the 109 mean another project's `packages/`, which are the n8n fixtures under `docs/taxonomies/` and the two notes beside them. That leaves 92. **38 of the 92 live under the directory itself, and the move renames every one.** Twelve paragraphs of spec 7 and four of [spec 13](../spec/13-open-obligations.md) name the path in prose. Three published interface contracts state it: [headwater-taxonomy](../interfaces/headwater-taxonomy.md), [headwater-check](../interfaces/headwater-check.md) and [headwater-conformance](../interfaces/headwater-conformance.md). The engine holds it in one constant, `PACKAGES` in `engine/crates/resolve/src/package.rs`.

**One gap is real whichever answer wins, and both answers pass over it.** No dimension of the compatibility measurement reads an adopter-facing path. `headwater taxonomy diff` runs the six dimensions that [spec 2](../spec/02-taxonomy-model.md#versioning-by-measured-compatibility) fixes. They are `classification`, `instance_validity`, `consequence`, `projection`, `identifier` and `addressability`. Each of the six asks about a corpus, or about the surface that an overlay addresses. A move of `packages/` breaks a promise that an interface contract states row by row. No dimension of the six reports it. Of a compatible result the verb says only *"no dimension is broken, so nothing here requires a major version. Whether it is a minor or a patch is not a question these six ask"*. The verb also measures a published package artifact rather than the engine crate. Nothing in `docs/spec/`, `docs/decisions/` or `DEVELOPING.md` derives the engine version from a measurement.

## Discharge

This record discharges when the owner rules on the root. The ruling reaches a decision record that states the reason and the condition that reopens it. A ruling of *stay* discharges it as fully as a ruling of *move*. What this record holds open is the silence and not the path.

The ruling waits on one outside adopter. That adopter says where they looked for vendored doctrine, and what they found. Until that adopter reports, a change that moves the root is a change made without the measurement.

The gap in the paragraph above is not discharged by either ruling. It belongs to spec 13, and this record states it rather than closes it.

**This record is discharged.** The owner ruled *move* on 2026-09-11, and [HW-DR-0064](../decisions/0064-the-vendored-package-root-moves-under-headwater-and-the-old-root-is-named-in-a-refusal.md) states the ruling, the reason accepted and the condition that reopens it. The silence this record held open is gone, which is what it asked for.

**It is discharged by a ruling and not by a measurement, and the record says so.** No outside adopter has reported. The paragraph above stands as written: the ruling was taken without the measurement the specification names, and HW-DR-0064 repeats that in its own words so that a later reader of either document is not misled into thinking the evidence arrived.

**The compatibility gap is not discharged with it.** [#821](https://github.com/headwater-ai/headwater/issues/821) carries it, and the ruling of *move* is exactly the change that made it visible: three interface contracts moved a path row by row, and no dimension of `headwater taxonomy diff` reported it.
