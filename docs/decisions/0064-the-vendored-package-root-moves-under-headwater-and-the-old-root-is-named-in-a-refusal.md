---
id: HW-DR-0064
status: current
status_since: 2026-09-11
summary: "The vendored package root moves to `.headwater/packages/`, on the ground that the visibility it bought is one explainer, and a tree at the old root now meets a named refusal."
last_verified: 2026-09-11
title: "The vendored package root moves under .headwater and the old root is named in a refusal"
relations:
  governs:
    - engine/crates/resolve/src/package.rs
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# The vendored package root moves under .headwater and the old root is named in a refusal

## Context

`headwater taxonomy vendor` installed a package at `packages/<name>` in the adopter's tree. Every other piece of consumer state this engine writes lands under `.headwater/`. [HW-OBL-0184](../obligations/0184-nothing-states-why-the-vendored-package-root-is-visible-in-an-adopter-tree.md) held that exception open and refused to choose, because [spec 7](../spec/07-distribution-and-federation.md) names an outside adopter as the evidence that settles it.

[#792](https://github.com/headwater-ai/headwater/issues/792) asked the question. Its body argued that [#381](https://github.com/headwater-ai/headwater/issues/381) had already settled a case for visibility. It had not. Spec 7's ruling paragraph, at *"That path is the only destination"*, gives three reasons and no reason of the three is about whether a person sees the directory.

**The three reasons, and what each one says about this question.** The first is the census cost: a doctrine page copied into an adopter's corpus root is one untyped file for every page copied. The second is the upgrade: a copy outside the package root has no replacement story, and no kind types it. The third is the price the ruling accepts, that prose under the package root answers to no language regime and to no check. Each of the three is about a *second* copy of the prose, somewhere the adopter chose. None is about where the one copy sits. A leading dot changes no one of them.

**What made the directory discoverable was never its name.** The paragraph above the three reasons states the mechanism: the report of `vendor` names the installed path, and it says that the directory is prose for a person and not schema. The engine interpolates that path from the one constant that names the root, so the printed path followed the move on the day the constant did.

## Decision

**The vendored package root is `.headwater/packages/`.** `vendor` writes `.headwater/packages/<name>`, `.headwater/packages/~staging/<name>` and `.headwater/packages/<name>~aside`, and it writes no other path in the consumer tree. The contracts for [`headwater taxonomy`](../interfaces/headwater-taxonomy.md), [`headwater check`](../interfaces/headwater-check.md) and [`headwater conformance`](../interfaces/headwater-conformance.md) state the new root row by row.

**The reason the owner accepted.** What `vendor` installs is four YAML files, a tree of bundle definitions, and exactly one file a person reads: `doctrine/starter.md`, about seven kilobytes of it. So the visibility the old root bought was the visibility of one explainer, against a directory of machine state that every adopter meets in every other tool as a dot directory. The engine prints the path of that explainer at the end of every `vendor`, which is the route a reader takes to it.

**A tree at the old root meets one named refusal.** Nothing reads `packages/` now. An adopter who vendored before this ruling carries a complete package there, and a lookup that reported only *no package declares this name* would tell that adopter their taxonomy does not exist. So `find` computes a sentence from the tree in front of it: where a directory stands at `packages/`, both refusals it can reach name that directory and say to move what is under it. Where no such directory stands, neither refusal mentions one. The second half is what keeps the sentence a report about that tree rather than an advisory printed on every tree.

**The ruling was taken without the measurement spec 7 names.** Spec 7's reopening clause asks for an adopter who reports that nobody finds the prose. No such adopter exists, and milestone *M6 — Distribution* exists to produce one. A later reader must not read this record as that evidence arriving. It is a ruling on this repository's own judgment of the cost, taken while the cost of the move is 38 renamed files and one constant, and before an adopter tree exists that the move would break.

**What reopens this:** an adopter who reports that they did not find the vendored doctrine, and who says where they looked. That report asks for the kind spec 7's own reopening clause asks for, and a kind gives the copied page a shelf, a language regime and a check. It does not ask for the root to move back.

## Consequences

**HW-OBL-0184 is discharged by this record, and discharged by a ruling rather than by a measurement.** Its own Discharge section says that a ruling of *move* discharges it as fully as a ruling of *stay*, because what it held open was the silence and not the path.

**One gap that record names is not discharged, and this record does not close it.** No dimension of the compatibility measurement reads an adopter-facing path. `headwater taxonomy diff` runs the six dimensions [spec 2](../spec/02-taxonomy-model.md#versioning-by-measured-compatibility) fixes, and each of the six asks about a corpus or about the surface an overlay addresses. This move breaks a promise three interface contracts state row by row, and no dimension of the six reports it. [#821](https://github.com/headwater-ai/headwater/issues/821) carries that question.

**The engine version moves one minor step, by hand.** Nothing derives the engine version from a measurement, and `headwater taxonomy diff` measures a published package artifact rather than the engine crate. So this is a judgment: the move breaks a stated contract for a consumer and breaks no data format, and the workspace goes to `0.2.0`.

**The base package goes to 4.3.1, because its own prose names the root.** The remediation text of the conformance rule `pin.current` tells an adopter which directory to move aside before a `vendor`, and after this ruling that sentence named a root nothing reads. So the authored source changed, and the installed artifact had to change with it: `the_vendored_record_matches_the_source_it_was_published_from` holds the two to the same bytes, and it is the check that would otherwise let the pair drift. The route was the four-step process the manifest itself documents — publish from source, pin the digest that run printed, vendor against that pin, resolve — and nothing was edited under the package root by hand. It is a patch step because comments moved and no declaration did. It is not a republish of 4.3.0 under new bytes, because a release tag already carries that version and a digest is not coupled to a version by anything.

**A tag's tree does not move, and the tools that read one now read both roots.** `tools/repo/readme-fixtures.sh` reads the release record of the base package out of the tree a taxonomy release tag names. Every tag cut before this record carries that record at `packages/`. The reader takes the new root first and falls back to the old one, and a tag that is in the clone and carries the record at neither root is a third finding with its own sentence, because fetching repairs the first case and repairs nothing here.
