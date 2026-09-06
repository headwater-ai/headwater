---
id: HW-OBL-0162
status: current
status_since: 2026-09-06
summary: "Spec 7's assembly example and the shipped recipe both pin one package version, and nothing compares them. The example was stale for four minor versions and no rule, no fixture and no CI job reported it."
last_verified: 2026-09-06
title: "Spec 7 prints a package version that no check compares with the recipe"
waiting_on: build
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# Spec 7 prints a package version that no check compares with the recipe

## Context

[Spec 7](../spec/07-distribution-and-federation.md#an-assembly-has-two-consumption-forms) prints a worked assembly recipe in a fenced block. Until 2026-09-06 that block read `from.package: headwater/standard@3.4.0` and selected `[design-spec, decision-record, standards-spec]`.

**Both halves were wrong and nothing reported either.** The package stood at `4.0.0`, so the version had been stale across at least four minor releases. The selection did not resolve at all. `design-spec` and `decision-record` name purposes, kinds and a voice regime that `evidence-and-obligation` declares. Referential integrity refuses the triple over seven dangling names. So the specification printed a recipe the engine refuses, in the section that a first adopter reads to learn what a recipe is.

`headwater check --strict` was green over that block on every run. No rule reads a fenced code block for anything, and `link.fragment.unresolved` reads links rather than YAML. No fixture and no CI job compares a specification example with the source it describes.

**The example is now correct and nothing holds it correct.** It was rewritten to be the recipe this repository ships, at `taxonomy-source/headwater-standard/assemblies/starter/assembly.yml`. So one version number now stands in two places that must agree: the fenced block, and the `from.package` line of the recipe. A third place already constrains the recipe, because `assembly::read` refuses a recipe whose `from.package` does not pin the version `package.yml` declares. The engine holds two of the three and reads none of the prose.

[HW-DR-0053](../decisions/0053-a-package-manifest-declares-no-selection-a-recipe-declares-one-and-a-flattened-manifest-records-provenance.md) rules where a selection may be declared. This record is about the copy of it that lives in prose.

## Obligation

**A version or a selection that this corpus prints as a worked example is a claim about the tree, and something has to read it.** The corpus owes one of two things. A check that compares a specification example with the artifact it names, or a ruling that an example is illustrative and names no real version.

The general shape is wider than this one block. This corpus states values in prose that no rule reads. The specification part on distribution is where such a value is most likely to be copied by a stranger. A reader who copies a stale example does not get a correction from any tool this repository ships. They get seven referential-integrity errors that name none of the three things they need.

## Discharge

Any of three closes it.

**A check that reads a fenced example against its source.** The narrow form compares the `from.package` of every YAML block in `docs/spec/` that carries an `assembly:` key against the package version `taxonomy-source/headwater-standard/package.yml` declares. This is mechanical and total, so it is an error rather than an advisory rule under the [fixability](../spec/12-check-layer.md#fixability) bar.

**A generated example.** The block is written by `headwater generate` from the recipe, and `generate --check` then refuses a hand edit to it. This costs a new projection and it removes the class rather than the instance.

**A ruling that an example names no real version.** The block reads `headwater/standard@<version>` and the prose says where the real number is. This costs nothing and it weakens the example, because a recipe that cannot be copied is a recipe a reader cannot run.

Nothing here is discharged by rewriting the example again. The example is correct today, and it was correct on the day it was written too.
