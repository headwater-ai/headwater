# Fixtures for the standards-spec taxonomy

A miniature standards ladder for Beacon, under `corpus/`. Beacon is invented. The [design-spec fixtures](../../design-spec/fixtures/README.md), the [decision-record fixtures](../../decision-record/fixtures/README.md) and the [diataxis fixtures](../../diataxis/fixtures/README.md) use the same invented system. A reader can therefore hold four entries over one project.

The [n8n fixture](n8n/README.md) beside this one is the real corpus this entry types: seven of the rule files n8n's AI code reviewer loads on every pull request, pinned and copied with every body byte-identical. Beacon is where the planted defects are, and n8n is where the entry meets a tradition that did not read it.

Everything here is invented, including the names. `warrant: accepted` requires an `accepted_by` that names a human, so the fixtures write `fixture-acceptor`. The namespace is `SCR`, which the assembly overlay declares and no bundle may.

Every number below came from the run this file quotes. Nothing here is predicted.

## How to run this corpus

`headwater check` reads one corpus root, and this tree sits inside a directory that this repository excludes from its own corpus. Assemble a scratch root that holds `packages/headwater-standard/`, this entry's `bundle.yml` at `bundles/standards-spec/`, and the tree under `corpus/docs/` at `docs/`. Point the scratch copy of `package.yml` at that directory by rewriting `contents.bundles` to `../../bundles`, which is the one scalar `headwater taxonomy publish` rewrites anyway. Bind it with a `.headwater/taxonomy.yml` that names the package, selects `bundles: [standards-spec]` and sets `corpus.root` to `docs`. Add an overlay that declares a namespace for `decision_id`, `standard_id`, `functional_spec_id` and `technical_spec_id`. This entry leaves its own three to a consumer, for the reason the base leaves `decision_id`. Omit any one of the four and `taxonomy validate` refuses the scheme that carries no namespace. Then run `headwater taxonomy resolve` and `headwater check`.

The entry sits outside the corpus root on purpose. A run that walked it would count this file and the doctrine in its census. The recorded numbers below would then move every time the entry gained a page.

**That a fixture corpus needs assembling is a finding rather than an inconvenience**, and the [decision-record fixtures](../../decision-record/fixtures/README.md#a-runner-reads-these-files-now-and-that-changes-what-this-file-is) already record it. Nothing in CI runs this corpus. Nothing declares where a reference corpus sits or how a run reaches one.

## What each document exercises

| Document | Kind | What it is here for |
|---|---|---|
| `docs/standards/api-design.md` | `standard` | The clean standard: three requirements, a `Conformance` heading, and `regulates` edges to two functional specs |
| `docs/standards/logging-baseline.md` | `standard` | Planted defect 1: the `Conformance` heading is absent |
| `docs/component-specs/delivery/functional.md` | `functional_spec` | The clean rung: two standards regulate it and one technical spec realizes it |
| `docs/component-specs/delivery/technical.md` | `technical_spec` | The clean pair to the above, and both halves of `realizes` are written |
| `docs/component-specs/signing/functional.md` | `functional_spec` | The far end of planted defect 2, which owes a `realized_by` that nobody wrote |
| `docs/component-specs/signing/technical.md` | `technical_spec` | Planted defect 2: it declares `realizes` and the other end does not name it back |
| `docs/component-specs/webhooks/functional.md` | `functional_spec` | Planted defect 3: it is `current`, its `status_since` is old, and no technical spec realizes it |
| `docs/component-specs/ingest/functional.md` | none | Planted defect 4: `spec_layer: interface_spec`, a value the shelf does not admit, which produces no finding at all |

## What a run reports

`headwater check --no-cache --now 2026-08-25` over the assembled root: **8 files under the corpus root, 7 typed, 1 untyped, 7 checked, 110 check instances, 3 findings.** The census reads 3 `functional_spec`, 2 `standard` and 2 `technical_spec`. The graph reads 7 nodes and 9 declared edge halves, all 9 of them to a document, and 0 prose links that did not resolve. `headwater check --strict` exits 1.

Three of the four planted defects are the three findings. The fourth produces none, and the section below it is the sharpest thing this corpus measures.

**Planted defect 1.** `docs/standards/logging-baseline.md` reports `section.required.missing`, an error, under `OB-SECT-1`:

    `standard` requires the section `Conformance`, and no heading of this document says so

**Planted defect 2.** `docs/component-specs/signing/technical.md:15:7` reports `relation.reciprocity.missing`, an error, under `OB-REL-1`:

    `SCR-TS-signing` declares `realizes: SCR-FS-signing`, and `realizes` requires both ends, so docs/component-specs/signing/functional.md owes `realized_by`

**Planted defect 3.** `docs/component-specs/webhooks/functional.md:4:1` reports `relation.participation.overdue`, a warning, under `OB-REL-3`:

    `functional-realized`: 222 days since `status_since`, and this `functional_spec` reaches no `realizes` to a `technical_spec` inside the 90 days the taxonomy allows

The finding names the window, the elapsed days and the rationale the expectation declares. That is the `realizes` nuclearity at work. The corpus reports the functional spec that nothing realizes, and it says nothing about the technical spec, which is the right way round.

## Planted defect 4 reports nothing, and that is the measurement

`docs/component-specs/ingest/functional.md` declares `spec_layer: interface_spec`. The `component_specs` shelf is heterogeneous and `spec_layer` is its discriminator, so kind resolution stops at step 3. The census carries one row:

    untyped: `spec_layer: interface_spec` is not a kind this shelf admits (functional_spec, technical_spec)

No rule reads the document after that. `engine/crates/check/src/coverage.rs` states the ruling in its own comment: "An **unclassified** document with no instance is not a finding here. It is already a row of the census with its own outcome, and a second report of one fact sends its author to two places."

**The control that makes this decisive.** Assemble the same root with `docs/component-specs/ingest/functional.md` as the only document and change nothing else. The run reports **1 file under the corpus root, 1 untyped, 0 classified, 0 checked, 2 check instances, 0 findings**, and `headwater check --strict` exits **0**. One typo in one discriminator value takes a whole document out of every rule this engine carries, and the commit gate passes.

This is [finding 6](../doctrine.md#findings) of this entry. It is not a defect of the base package and it is not a defect of this bundle. It is a property of every heterogeneous shelf, and the design-spec entry's `spec_series` shelf has carried it since the library opened.

## The run against the same corpus with the bundle deselected

The comparison that shows the declaration is what types the corpus. Assemble the same root with `bundles: []` and change nothing else. The run reports **8 files under the corpus root, 8 untyped, 0 classified, 0 checked, 2 check instances, 0 findings**, and `headwater check --strict` exits **0**.

Every number that matters moves, and that is the difference between this entry and the diataxis entry. A facet-only entry adds one finding to a corpus the base already types. This entry declares the two shelves that claim `docs/standards/**` and `docs/component-specs/**`. With the bundle deselected no shelf pattern claims any of the eight paths, and the corpus is invisible.

**The exit status discriminates here, and in the diataxis corpus it did not.** A strict run over the deselected arm exits 0 because it found nothing to check. A strict run over the selected arm exits 1 on two errors. So the declaration turns a corpus that no rule reads into a corpus that 110 check instances read.

## What a run reports that is not planted

**Nothing.** The three findings are the three plants, and no fourth finding of any kind appeared. That is a different result from the other three entries in this library. One reason is worth naming: every kind here declares an identifier scheme, so no document reports `identifier.unusable`. The diataxis corpus reported that finding six times, because the base's `specification` kind mints under no scheme. This entry declares `standard_id`, `functional_spec_id` and `technical_spec_id`, and [HW-OBL-0107](../../../obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md) does not reach it.

**One prediction of an earlier entry was measured against this corpus, and it holds.** `design-spec`'s fixture README predicts an error for `naming-survey.md`, which carries `doc_type` on a homogeneous shelf whose kind forbids the facet. Spec 2 says that "no check reports a document that states a facet its kind forbids". A reader takes that for a refusal of the prediction, and it is not one. A probe added `spec_layer: functional_spec` to `docs/standards/api-design.md`, which is the same case. The run reported an error at `docs/standards/api-design.md:6:1`:

    shelf.placement_is_primary (OB-PLACE-1): `standards` is homogeneous and carries the kind `standard`, and this document restates it as `spec_layer: functional_spec`

So the finding arrives through the placement rule rather than through the facet contract, and the prediction is right about the outcome. The probe is not committed and no document of this corpus carries it.

**One consequence of that measurement reaches this bundle.** `kinds.standard` declares `facets: {forbid: [spec_layer]}`, and the run above shows that the placement rule already refuses the same document without it. The line still earns its place for two reasons. `forbid` is what stops the generated value check from making an instance over `standard`. A later reader of the kind also learns from the declaration that the facet does not belong there. It buys no finding that the corpus does not already report.

## What ages, and what does not

**One finding is windowed, and its date is fixed so that it never stops firing.** `docs/component-specs/webhooks/functional.md` declares `status_since: 2026-01-15` and the recorded run injects `--now 2026-08-25`, which is 222 days. The window is 90 days. A run with a later clock reports more days and the same finding. A run with no `--now` reads the wall clock, which is later still, so the finding holds there too.

**Two documents are inside a window that a later clock closes.** `docs/standards/api-design.md` and `docs/standards/logging-baseline.md` each declare `status_since: 2026-03-01`, and the `standard-applied` expectation allows 180 days. Neither one fires, and neither one ever will. Each standard declares a `regulates` edge to a functional spec, so the expectation is satisfied rather than merely young. The window is unreachable for both.

**Nothing here reports staleness.** `last_verified` carries a `stale_after_days: 180` policy in the base package and no rule this engine carries reads it. The recorded run lists every rule it ran, and none of them is a staleness rule.

**The counts move if the corpus moves and not otherwise.** The entry sits outside the corpus root, so this file, the doctrine and the three templates are absent from the census. Add a document under `corpus/docs/` and every count above changes. Edit this paragraph and none of them does.
