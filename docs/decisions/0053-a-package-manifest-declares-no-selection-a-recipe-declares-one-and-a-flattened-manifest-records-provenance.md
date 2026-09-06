---
id: HW-DR-0053
status: current
status_since: 2026-09-06
summary: "A manifest says what a package carries, a recipe says what a composer selected, and a flattened manifest records where it came from. Three enforcement points already hold the split, and this record names it as a rule rather than as specification prose."
last_verified: 2026-09-06
title: "A package manifest declares no selection, a recipe declares one, and a flattened manifest records provenance"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0044
    - HW-DR-0003
---

# A package manifest declares no selection, a recipe declares one, and a flattened manifest records provenance

## Context

[HW-DR-0003](0003-how-much-of-the-default-taxonomy-ships-in-the-box.md) rules that the base package is minimal and derived from the core. [HW-DR-0044](0044-q44-whether-bundles-decompose-into-capabilities-and-assemblies-compose-practices.md) rules that an assembly is a named publisher recipe that composes a practice out of bundles. Between the two sits a question that neither answers: which of the three artifacts is allowed to say what a corpus runs on.

[Spec 7](../spec/07-distribution-and-federation.md#an-assembly-has-two-consumption-forms) already answers it in prose. It states that an arrow into an assembly identifies a recipe input rather than a package dependency. It states that the derivation fields of a flattened manifest carry provenance and create no runtime dependency. That is a correct description of the engine and it is not a rule anybody can cite. This corpus has refused a merge for a false sentence in a specification part. So a specification part is the weakest place to keep an invariant.

**A recipe on this tree now depends on the distinction, for the first time.** `taxonomy-source/headwater-standard/assemblies/starter/assembly.yml` is the first recipe this repository ships. The temptation it creates is concrete. A manifest key that named the base, or one that named a default selection, is a small edit that reads as a convenience. Either one moves a selection into the wrong artifact.

**Three mechanisms already enforce the split, and no reader is told that they are one rule.**

- `assembly::read` refuses a recipe whose `from.package` does not pin the package that carries it. The refusal names both identities and ends "An assembly reads the package that carries it".
- `package::selected` refuses a consumer selection against a package that ships no bundle directory, saying `this selects N bundles and <package> ships none`. A flattened package is exactly that case.
- `flatten::distribution` writes `distribution.form: flattened` beside a `distribution.derived_from` record. The record carries the source package, its version, the taxonomy digest, the selection digest, and one name and digest per selected bundle. It carries a recipe digest, and an overlay digest only where the recipe declares glue.

**A fourth mechanism is absent and belongs to this reading too.** [HW-DR-0040](0040-q40-whether-extends-bundle-requires-and-an-overlay-s-taxonomy-key-are-a-mechanism-or-a-label.md) ruled `requires:` at a bundle root a label rather than a mechanism, and `package::selected` reads no such key. So a manifest does not declare the closure of a selection either. What refuses an incomplete selection is referential integrity over the names the selected bundles leave dangling.

## Decision

**A package manifest declares what the package carries, and nothing about what a consumer runs.** Its `contents` keys name a taxonomy, a conformance set, and the directories of bundles, assemblies, doctrine and templates the artifact ships. No manifest key names a base package that the package composes over, and no manifest key names a bundle selection or a default selection. A package that composes over a base is a flattened package, and the base it composed over is provenance rather than a reference.

**A recipe declares the selection, over exactly one pinned package version.** An assembly names the source package with its version in `from.package`, and names the bundles in `from.bundles`. It may name one overlay that holds the connections which span two or more of those bundles. The selection lives here and in the consumer declaration of a composer, and in no third place.

**A flattened manifest records the recipe as provenance, and creates no dependency on it.** `distribution.derived_from` is a statement about how the artifact was made. Nothing resolves it, nothing fetches it, and an advance of the source package does not reach a flattened consumer.

**The three enforcement points above are the rule, and this record is what they enforce.** A change that weakens any of them is a change to this ruling and is argued here.

## Consequences

**No engine change is owed to close this.** The record is a name for behavior the engine already has. Its value is that a later proposal now has something to be refused by. A `base:` key or a `default_bundles:` key on a manifest is that proposal.

**The two consumption forms keep their different owners, and the reason is now written down.** A composer pins the source package and owns its selection, so it advances the source package when it chooses. A batteries-included consumer takes the flattened package and waits for the publisher to release a new one, because `distribution.derived_from` gives it nothing to resolve.

**A flattened package cannot be repaired by a selection.** An adopter who takes `headwater/starter` and then wants a fourth bundle cannot add it to the consumer declaration. The refusal above is what they get. They move to the composer form, or they wait for a recipe that includes it. Spec 7 states this and now cites a ruling for it.

**This record does not rule on two neighboring questions.** Whether `requires:` should become a mechanism stays with HW-DR-0040. Where a bundle's own doctrine lands inside a flattened artifact stays open. `flatten::assets` copies the source package's own `contents.doctrine` and `contents.templates` into namespaced paths, and it carries no bundle doctrine.
