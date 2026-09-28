---
id: HW-OBL-0107
title: "The base package ships a kind that the scaffolder refuses to write"
status: current
status_since: 2026-08-14
waiting_on: build
last_verified: 2026-09-29
summary: "`kinds.specification` names no identifier scheme and four relations may name a document of it, so `headwater new specification` refuses on the stock package and no check reports the cause."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-taxonomy-model
    - HW-SPEC-authoring-and-lifecycle
---

# The base package ships a kind that the scaffolder refuses to write

## Context

`headwater/standard` declares two concrete kinds. `decision` names the scheme `decision_id`. `specification` names no identifier scheme at all.

Four relations of the same package admit `governed_document` at an end, and `specification` is a `governed_document`. They are `supersedes`, `governs`, `traces_to` and `assesses`. The base taxonomy declares the first three, and the design-spec bundle declares `assesses`. So a relation may name a document of the kind, and a document of the kind carries no name for a relation to use.

The scaffolder refuses in front of that. On 2026-08-14, a run over every concrete kind of this repository reported the refusal in these words:

    headwater new specification --title "A constructor audit probe"
    headwater: `specification` names no identifier scheme, and `supersedes`, `governs`,
    `traces_to`, `assesses` may name a document of it. Such a document is neither end of
    any edge, which `identifier.unusable` reports on every run. Declare
    `kinds.specification.identifier` first

The overlay of this repository observes half of the fact. It says that the base gives `decision` an identifier and gives `specification` none. It then mints a scheme for each of the six kinds that the design-spec bundle adds. It mints none for `specification`, because no document of that kind is here.

## Obligation

The check layer cannot report this. `identifier.unusable` is a rule over documents, and the specifications shelf holds none. A kind with no document of it is invisible to every rule in this engine. Only the constructor reaches it. Since #1264, one engine test runs the constructor over every concrete kind of the stock package. Nothing runs it over the taxonomy of an adopter.

So the defect that a first adopter meets is a defect that a green run of this repository asserts is absent. An adopter takes the package alone and writes a specification. The verb refuses, and it names a declaration inside the package that the adopter has just installed.

The base package is the layer that owes the repair, rather than this overlay. An overlay that minted a scheme here would leave every other adopter in the same position. The second entry of the taxonomy library would then meet it again.

## Discharge

`kinds.specification.identifier` in `.headwater/packages/headwater-standard/taxonomy.yml`, under a scheme the package declares beside `decision_id`.

That route is not taken yet, and #1264 took a smaller step for the adopter. The refusal now prints two lines to add under `add:` in `.headwater/overlay.yml`. They declare a scheme and the kind's identifier. The scheme takes its namespace from a scheme that already resolves, and on the stock package that is the namespace the adopter gave `decision_id`. So the adopter pastes the lines, resolves, and writes the specification. The package route costs every adopter more today. `taxonomy resolve` refuses a scheme that has no namespace, and a package cannot ship one ([HW-DR-0025](../decisions/0025-q25-where-the-namespace-goes-in-an-identifier-and-who-declares-it.md)). A second scheme in the package would make `resolve` refuse for every adopter until they add a second namespace line. The package route opens again when an overlay can declare one namespace that every scheme inherits.

Beside that, a rule or a verb that reads the taxonomy rather than the corpus. `headwater new` over every concrete kind is a taxonomy audit with no home. It finds a kind on no shelf, a kind on two shelves, and a required facet that nothing determines. It finds a scheme whose pattern cannot be read, and it finds this. [Spec 12](../spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on) has no grain for a rule whose subject is a declaration and whose corpus is empty, and [HW-OBL-0067](0067-a-rule-that-reads-the-taxonomy-has-no-grain-in-spec-12s-list.md) holds that gap.
