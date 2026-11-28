---
id: HW-DR-0117
status: current
status_since: 2026-11-28
summary: "headwater/standard 5.0.0 deletes the sequence facet. A part of the specification series is named by its slug, a citation names a part by a title link, and the sidebar lists the series in the reading order that builds_on gives. The four-digit identifier shelves keep their numbers."
last_verified: 2026-11-28
title: "117 — The specification series names a part by its slug and carries no sequence"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  constrains:
    - HW-DR-0108
  traces_to:
    - HW-OBL-0175
    - HW-OBL-0106
    - HW-OBL-0056
    - HW-DR-0054
    - HW-SPEC-distribution-and-federation
---

# 117 — The specification series names a part by its slug and carries no sequence

## Context

The first commit of this repository created the parts of the specification at `docs/spec/00-…` to `docs/spec/09-…`, and no record states why. Twelve days later the design-spec bundle typed that shape after the fact. It required a `sequence` facet on `design_spec`, `decision_register` and `obligation_register`, and it laid out the `spec_series` shelf as `{sequence:02d}-{slug}.md`. The only reason ever written was in the design-spec doctrine: "the number is part of the address", and a part is never renumbered.

The repository broke that rule more than once, and no check said so. Parts 10 and 11 were deleted. Part 17 moved off the shelf. Two files open with `09-`. `docs/spec/glossary.md` declares `sequence: 14` and carries no number in its name. [HW-OBL-0175](../obligations/0175-two-documents-take-one-sequence-on-a-sequence-declaring-shelf-and-no-rule-reads-it.md) measured the cause: two parts at one sequence give `check --strict` exit 0, because the scaffolder reads the layout at birth and no rule reads the number after it. The first real corpus that the bundle met, n8n, had no number to write, because a package is not a position in a series ([the n8n evaluation](../evaluations/n8n-worked-example.md)).

On 2026-11-27 the owner ruled on [#2137](https://github.com/headwater-ai/headwater/issues/2137): "I actually don't know why we have this sequence number concept. I don't like the specs being named with numbers and would prefer to drop the numbering." Four rulings followed on the issue. The package goes major and deletes the facet. A citation is a title link. The sidebar orders the series by relation order. The four-digit identifier shelves stay as they are. On 2026-11-28 the owner answered the question of what gives the series its reading order once the numbers go: a relation from a part to the part it builds on, declared in the package.

## Decision

**The `spec_series` shelf names a part by its slug, and no kind on it requires a sequence.** headwater/standard 5.0.0 deletes `facets.sequence` from the design-spec bundle and drops it from the three `require` lists. The layout is `{slug}.md`. The file of a part takes the slug of its identifier, so `HW-SPEC-check-layer` is `check-layer.md` and `HW-REG-decisions` is `decisions.md`. The identifiers do not change. The templates write no `sequence:` line, and a part opens with `# {{title}}`.

**The doctrine rule "the number is part of the address" is retired.** The address of a part is its slug. A rename keeps the old address answering with a redirect, which is what a renumber needed before.

**The facet is deleted, not deprecated.** A declared facet that no kind requires and no rule reads fails the relevance canon of `taxonomy validate` with "nothing reads it". So the deletion is the only shape that resolves. The package ships its first migration payload, `migrations/4-to-5.yml`. Its one step is an `overlay_address` step from `facets.sequence` to nothing, with a task that asks the adopter to delete the entry.

**A citation names a part by a link whose text is the title, with the article that English needs.** "[the check layer](../spec/12-check-layer.md)" replaces "spec 12". The phrase for each part is in the table below, and a sentence that opens with a phrase capitalizes it.

| identifier | file | phrase |
|---|---|---|
| HW-SPEC-vision-and-scope | vision-and-scope.md | the vision and scope |
| HW-SPEC-conceptual-model | conceptual-model.md | the conceptual model |
| HW-SPEC-taxonomy-model | taxonomy-model.md | the taxonomy model |
| HW-SPEC-authoring-and-lifecycle | authoring-and-lifecycle.md | authoring and lifecycle |
| HW-SPEC-assurance-model | assurance-model.md | the assurance model |
| HW-SPEC-ai-integration | ai-integration.md | AI integration |
| HW-SPEC-engine-architecture | engine-architecture.md | the engine architecture |
| HW-SPEC-distribution-and-federation | distribution-and-federation.md | distribution and federation |
| HW-SPEC-design-departures | design-departures.md | the design departures |
| HW-REG-decisions | decisions.md | the decision register |
| HW-REG-open-questions | open-questions.md | the register of open questions |
| HW-SPEC-check-layer | check-layer.md | the check layer |
| HW-REG-open-obligations | open-obligations.md | the register of open obligations |
| HW-SPEC-the-recorder-contract | the-recorder-contract.md | the recorder contract |
| HW-SPEC-harness-support | harness-support.md | harness support |
| HW-SPEC-glossary | glossary.md | the glossary |

**The relation `builds_on` gives the series its reading order.** The design-spec bundle adds it in 5.0.0, from any kind of the shelf to any kind of the shelf. Its family is `derivation` and its nucleus is `to`, so the part built on governs the reading. `by_precedence` puts it first, and so does every reader that orders by precedence: a routing result, a shelf index and the sidebar. An author types each edge. Without the relation the shelf has no order but its paths, and once the paths carry no number that order is alphabetical.

**The sidebar lists the specification series in relation order.** This record amends [HW-DR-0108](0108-the-site-sidebar-lists-a-shelf-in-path-order-and-the-shelf-index-keeps-the-reading-order.md) for that shelf and no other. A shelf whose files carry a minted number keeps path order, which is numeric order there, for the reason HW-DR-0108 gives.

**The `{seq:04d}-{slug}.md` shelves and the Q-numbers are out of scope.** The decisions, obligations, process records and requirement-verification shelves take their number from an identifier that the run mints and that the claim store holds distinct ([HW-DR-0054](0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md)). Two branches cannot take one value there without an `add/add` conflict, so HW-OBL-0175 does not reach them.

## Consequences

- An adopter who upgrades from 4.x runs `taxonomy vendor`, then `taxonomy diff`, then `taxonomy migrate --apply`. The diff names the removal and the payload step, with each entry of the adopter's overlay at or under `facets.sequence`. A layout override that names `{sequence…}` meets a validate refusal, because nothing fills the placeholder. The [design-spec doctrine](../taxonomies/design-spec/doctrine.md#upgrading-from-4x) states the whole route.
- A numbered file name and a `sequence:` line that 4.x left on disk raise no finding. No rule reads a layout after birth ([HW-OBL-0106](../obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md)), and no rule reads a front-matter key that the taxonomy does not declare. `taxonomy audit` counts a numbered name as one that does not match `{slug}.md`, and that count is a measurement, not a finding. The audit renders `{slug}` from the title, so it also counts a name such as `taxonomy-model.md` against the title "The taxonomy model".
- `headwater taxonomy diff` measured 5.0.0 before the bump. Over this repository it reports `projection` BROKEN, 1 of them: the generated register `09-open-questions.md` loses its `sequence: 9` line. Over the n8n fixture it reports `instance_validity` and `consequence` BROKEN, 4 failed each, because each of the four documents fails one requirement fewer. A broken dimension requires a major version, so the number obeys the measurement as well as the owner's ruling.
- `takes_a_claim` in `engine/crates/check/src/claim.rs` still answers true for `spec_series`, because the shelf keeps a layout. That is harmless, and its doc comment, which uses the numbered layout as its example, moves with the rename of the parts.
- The first measurement that reopens the "no fourth subject" ruling of [distribution and federation](../spec/07-distribution-and-federation.md#the-migration-payload) is now met: 5.0.0 ships a payload against a real upgrade. It measured that no fourth subject is needed. The `overlay_address` subject reaches the removal, a left-over `sequence:` key is inert, and a file rename is no break because no rule reads a layout.
- HW-OBL-0175 is discharged. The series carries no sequence, and the remaining numbered shelves take theirs from a claimed identifier. HW-OBL-0106 narrows to the identifier shelves: the specification shelf lays out as `{slug}.md` and every kind on it requires `title`, so the audit measures it. HW-OBL-0056 is discharged: `{slug}.md` holds `glossary.md`.
- The rename of the parts, the rewrite of every link and citation, the redirects and the sidebar order land in one later pull request, so that a branch in flight meets one conflict. Until then the parts keep their numbered names, and a citation inside the package names the new file, which 404s on the default branch for that window. No gate reads a `blob/main` URL.
