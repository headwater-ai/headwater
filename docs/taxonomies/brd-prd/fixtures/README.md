# Fixtures for the brd-prd taxonomy

A miniature requirements handoff for Beacon, under `corpus/`. Beacon is invented. The [design-spec fixtures](../../design-spec/fixtures/README.md), the [decision-record fixtures](../../decision-record/fixtures/README.md), the [diataxis fixtures](../../diataxis/fixtures/README.md) and the [standards-spec fixtures](../../standards-spec/fixtures/README.md) use the same invented system. A reader can therefore hold five entries over one project.

Everything here is invented, including the names. `warrant: accepted` requires an `accepted_by` that names a human, so the fixtures write `fixture-acceptor`. The namespace is `SCR`, which the assembly overlay declares and no bundle may.

Every number below came from the run this file quotes. Nothing here is predicted.

## How to run this corpus

`headwater check` reads one corpus root, and this tree sits inside a directory that this repository excludes from its own corpus. Assemble a scratch root that holds `.headwater/packages/headwater-standard/`, this entry's `bundle.yml` at `bundles/brd-prd/`, and the tree under `corpus/docs/` at `docs/`. Point the scratch copy of `package.yml` at that directory by rewriting `contents.bundles` to `../../bundles`, which is the one scalar `headwater taxonomy publish` rewrites anyway. Bind it with a `.headwater/taxonomy.yml` that names the package, selects `bundles: [brd-prd]` and sets `corpus.root` to `docs`. Add an overlay that declares a namespace for `decision_id`, `brd_id` and `prd_id`. This entry leaves its own two to a consumer, for the reason the base leaves `decision_id`. Omit any one of the three and `taxonomy validate` refuses the scheme that carries no namespace. Then run `headwater taxonomy resolve` and `headwater check`.

The entry sits outside the corpus root on purpose. A run that walked it would count this file and the doctrine in its census. The recorded numbers below would then move every time the entry gained a page.

**That a fixture corpus needs assembling is a finding rather than an inconvenience**, and the [decision-record fixtures](../../decision-record/fixtures/README.md#a-runner-reads-these-files-now-and-that-changes-what-this-file-is) already record it. Nothing in CI runs this corpus. Nothing declares where a reference corpus sits or how a run reaches one.

## What each document exercises

Four initiatives, each in its own subdirectory, which is how the tradition files a pair. The per-initiative directory is also what forces the discriminator: placement cannot tell a `brd.md` from a `prd.md` on a heterogeneous shelf.

| Document | Kind | What it is here for |
|---|---|---|
| `docs/requirements/self-serve-onboarding/brd.md` | `brd` | The clean business document: all three headings, and `elaborated_by` written |
| `docs/requirements/self-serve-onboarding/prd.md` | `prd` | The clean pair to the above, and both halves of `elaborates` are written |
| `docs/requirements/delivery-slo/brd.md` | `brd` | Planted defect 1: the `Success measures` heading is absent |
| `docs/requirements/delivery-slo/prd.md` | `prd` | The clean far end of the above, so planted defect 1 is isolated |
| `docs/requirements/webhook-replay/brd.md` | `brd` | Planted defect 4: its prose narrates a change. It is also the far end of planted defect 2, and it owes an `elaborated_by` that nobody wrote |
| `docs/requirements/webhook-replay/prd.md` | `prd` | Planted defect 2: it declares `elaborates` and the other end does not name it back |
| `docs/requirements/tenant-billing/brd.md` | `brd` | Planted defect 3: it is `current`, its `status_since` is old, and no product document elaborates it |

## What a run reports

`headwater check --no-cache --now 2026-08-25` over the assembled root: **7 files under the corpus root, 7 typed, 0 untyped, 7 checked, 101 check instances, 4 findings.** The census reads 4 `brd` and 3 `prd`. The graph reads 7 nodes and 5 declared edge halves, all 5 of them to a document, and 0 prose links that did not resolve. `headwater check --strict` exits 1.

All four planted defects are the four findings, and the run reports nothing else.

**Planted defect 1.** `docs/requirements/delivery-slo/brd.md` reports `section.required.missing`, an error, under `OB-SECT-1`:

    `brd` requires the section `Success measures`, and no heading of this document says so

**Planted defect 2.** `docs/requirements/webhook-replay/prd.md:15:7` reports `relation.reciprocity.missing`, an error, under `OB-REL-1`:

    `SCR-PRD-webhook-replay` declares `elaborates: SCR-BRD-webhook-replay`, and `elaborates` requires both ends, so docs/requirements/webhook-replay/brd.md owes `elaborated_by`

**Planted defect 3.** `docs/requirements/tenant-billing/brd.md:4:1` reports `relation.participation.overdue`, a warning, under `OB-REL-3`:

    `business-need-elaborated`: 222 days since `status_since`, and this `brd` reaches no `elaborates` to a `prd` inside the 90 days the taxonomy allows

The finding names the window, the elapsed days and the rationale the expectation declares. That is the `elaborates` nuclearity at work. The corpus reports the business document that nothing elaborates. It says nothing about a product document that elaborates nothing, which is the right way round.

**Planted defect 4.** `docs/requirements/webhook-replay/brd.md:23:1` reports `voice.forbidden_construction`, a warning, under `OB-VOICE-1`:

    `prospective` forbids change_narration, and this sentence writes `we changed`

This plant is new to this library. No other fixture corpus here plants a voice finding. It is the evidence that the `prospective` regime is bound to the kind and read by the rule. The finding names the regime by the name this bundle gives it. The severity is a warning because `CT-VOICE-1` is permanently advisory, so this finding alone does not move a strict exit status. The two errors above do.

## What the base's voice regime would have reported

The regime is the one deliberate departure of this entry, so it is measured rather than asserted. Assemble the same root, change `voice: prospective` to `voice: declarative` on `kinds.brd` and `kinds.prd`, and change nothing else. That is two lines of the bundle and no line of any document.

The run reports **8 findings, 2 error and 6 warn**, against 4 findings, 2 error and 2 warn, before it. The four extra findings are all `future_intent`, in four of the seven documents:

| Document | The sentence the rule names |
|---|---|
| `docs/requirements/delivery-slo/brd.md:36:1` | `is planned` |
| `docs/requirements/self-serve-onboarding/brd.md:26:1` | `will be` |
| `docs/requirements/self-serve-onboarding/prd.md:47:1` | `to be decided` |
| `docs/requirements/tenant-billing/brd.md:23:1` | `to be decided` |

Each sentence is one a practitioner writes. Two of them defer a number that finance or a content designer owns. One forecasts the market the initiative answers, and one defers a requirement to the quarter after. Three of the four were added after a first probe measured one document rather than four. The first draft of this corpus was written by an author who was avoiding the detector, which is a bias of its own. Read the four and judge whether any is padding, because that judgment is the whole weight this measurement carries. A requirements document states what a product must do before it exists, and the base's only voice regime forbids the construction that says so. The rule makes one instance per document, and the run prints 7 instances over 7 documents. So four documents is the measure, and not four sentences.

The probe is not committed. No document of this corpus carries a change, and the two lines of the bundle were reverted. [Finding 2](../doctrine.md#findings) of this entry is what the number is evidence for.

## What a run reports that is not planted

**Nothing.** The four findings are the four plants, and no fifth finding of any kind appeared.

Two reasons are worth naming. Both kinds declare an identifier scheme, so no document reports `identifier.unusable`. The diataxis corpus reported that finding six times, because the base's `specification` kind mints under no scheme. And `webhook-replay/brd.md` reports no overdue participation, though it is 222 days old and declares no `elaborated_by`. Its product document declares the other half of the edge. The expectation reads the graph rather than the front matter of one document. The `standards-spec` corpus carries the same pair, and neither entry states the property anywhere a schema author would find it.

**This corpus plants no unknown discriminator value, and the reason is that the measurement already exists.** The `standards-spec` entry ran it in full, with a single-document control arm, and [its fixture README holds the run](../../standards-spec/fixtures/README.md#planted-defect-4-reports-nothing-and-that-is-the-measurement). A value outside the shelf's `kinds` list stops kind resolution. The census carries the row, no rule instantiates, and a strict run over that document alone exits 0. The `requirements` shelf of this entry carries the same property, because every heterogeneous shelf does. Re-planting it here would buy a second copy of one measurement.

## What ages, and what does not

**One finding is windowed, and its date is fixed so that it never stops firing.** `docs/requirements/tenant-billing/brd.md` declares `status_since: 2026-01-15` and the recorded run injects `--now 2026-08-25`, which is 222 days. The window is 90 days. A run with a later clock reports more days and the same finding. A run with no `--now` reads the wall clock, which is later still, so the finding holds there too.

**Three business documents are past the same window and none of them fires.** `self-serve-onboarding/brd.md`, `delivery-slo/brd.md` and `webhook-replay/brd.md` each declare `status_since: 2026-01-15`, which is the same 222 days. Each one reaches a `prd` through an `elaborates` edge, so the expectation is satisfied rather than merely young. The first two declare `elaborated_by` themselves. The third declares nothing and is reached from the far end, which is what planted defect 2 is about.

**Nothing here reports staleness.** `last_verified` carries a `stale_after_days: 180` policy in the base package and no rule this engine carries reads it. The recorded run lists every rule it ran, and none of them is a staleness rule.

**The counts move if the corpus moves and not otherwise.** The entry sits outside the corpus root, so this file, the doctrine and the two templates are absent from the census. Add a document under `corpus/docs/` and every count above changes. Edit this paragraph and none of them does.
