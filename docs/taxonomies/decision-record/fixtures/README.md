# Fixtures for the decision-record taxonomy

A miniature ADR log in the tradition, under `corpus/`, with the obligation register that this entry pairs with it. Beacon is invented, and it is the same invented system that the [design-spec fixtures](../../design-spec/fixtures/README.md) use, so the two entries are visibly two traditions over one project.

Everything here is invented, including the names. `warrant: accepted` requires an `accepted_by` that names a human, so the fixtures write `fixture-acceptor`.

## A runner reads these files now, and that changes what this file is

The design-spec fixtures state their expected findings in prose, and they say why: "No runner reads these files. There is no engine." An engine exists. So this file states what a run reports, and it states it because a run reported it rather than because a reader predicted it.

The run is not automatic, and nothing in CI performs it. `headwater check` reads one corpus root, and this tree sits inside a directory that this repository excludes from its own corpus. To reproduce the output below, assemble a root that holds `.headwater/packages/`, both library entries under `docs/taxonomies/`, and this fixture tree at `docs/`, then bind it to the base package with `bundles: [design-spec, decision-record]`. That assembly is four `cp` commands and a nine-line `.headwater/taxonomy.yml`.

**That a fixture corpus needs assembling is a finding rather than an inconvenience.** A package ships reference corpora, and [spec 7](../../../spec/07-distribution-and-federation.md#upgrading) measures compatibility against them. Nothing declares where such a corpus sits or how a run reaches one. The [doctrine](../doctrine.md#findings) does not carry this, because it is a fact about every entry rather than about this one.

## What each document exercises

| Document | Kind | What it is here for |
|---|---|---|
| `docs/decisions/0001-deliver-at-least-once.md` | `decision` | The superseded end of a succession pair, retained with its successor named |
| `docs/decisions/0002-store-attempts-in-postgres.md` | `decision` | `traces_to` an evaluation, and one half of the planted conflict |
| `docs/decisions/0003-refuse-a-dead-letter-queue.md` | `decision` | A refused proposal, which the state vocabulary cannot tell from an abandoned decision |
| `docs/decisions/0004-deliver-exactly-once-per-destination.md` | `decision` | The successor, and the source of two obligations |
| `docs/decisions/0005-partition-by-tenant.md` | `decision` | A proposal that nobody ruled on, in `draft` since January |
| `docs/decisions/0006-hold-attempt-state-in-redis.md` | `decision` | The other half of the planted conflict, and both are `current` |
| `docs/obligations/0001-window-length-is-unmeasured.md` | `obligation_record` | A discharged obligation, with both halves of `discharges` present |
| `docs/obligations/0002-redis-durability-waits-on-an-operator.md` | `obligation_record` | An obligation with no honest window, which is why no expectation is declared |
| `docs/obligations/0003-integration-guide-states-the-old-guarantee.md` | `obligation_record` | Two planted defects, below |
| `docs/evaluations/retry-ceiling-measurement.md` | `evaluation` | The design-spec kind that `discharges` runs from, so the fixture exercises the dependency |

## What a run reports

`headwater check --no-cache --now 2026-08-13` over the assembled root: **44 files seen, 10 classified, 10 checked, 78 check instances, 3 findings.** Two of the three are planted.

**1. `0003-integration-guide-states-the-old-guarantee.md` has no `Discharge` section.** `section.required.missing`, error, under `OB-SECT-1`. The kind requires `Context`, `Obligation` and `Discharge`, and the third is the one that makes a debt actionable. An obligation whose discharge nobody can state is a wish, and this is the fixture that proves the heading is enforced rather than suggested.

**2. The evaluation declares `discharges: HW-OBL-0003`, and that record does not name it back.** `relation.reciprocity.missing`, error, under `OB-REL-1`, with a mechanical fix. The relation this entry adds is the one under test, and the fixture proves that its `reciprocal: required` reaches the runner.

**3. The evaluation is cited by no register.** `relation.participation.overdue`, warning, under `OB-REL-3`. This one is not planted, and it is the more interesting result. The design-spec entry declares `evidence-cited` on `kinds.evaluation`, with `to_kind: decision_register`. This corpus keeps its decisions as documents rather than in a register, so it holds no `decision_register` at all, and no document in it can ever satisfy the expectation. The finding is unfixable here.

That third finding is the [first doctrine finding](../doctrine.md#findings) arriving as a measurement. An expectation names one target kind, and an endpoint list names concrete kinds, so a second entry that composes with a first inherits its expectations and cannot amend them. It is also the case that design-spec's own [finding 4](../../design-spec/doctrine.md#findings) predicted from the inside.

## What a run does not report, and should

**The two `current` decisions joined by `conflicts_with` pass in silence.** `HW-DR-0002` and `HW-DR-0006` contradict each other, both are `current`, and both declare the edge. The base declares `conflicts_with` with `invalid_when: {both: {status: current}}`. [Spec 2](../../../spec/02-taxonomy-model.md#the-decision-relation-vocabulary) lists that state first among the four checks that the decision-relation vocabulary brings, and [HW-EVAL-adjacent-work](../../../evaluations/adjacent-work.md) calls it "a deterministic, blocking-eligible check" that was in the design all along. No rule in the engine reads it. The declaration reaches one place in the resolver, where it makes the `status` facet count as read for the relevance canon, and it reaches no check.

This is the fixture's most useful result, and it is why the conflict is here rather than in the planted list. A corpus that holds two live contradictory decisions gets a clean run today. The [doctrine](../doctrine.md#findings) carries the finding.

**Neither obligation of this tradition reports anything.** `OB-DR-1` and `OB-DR-2` are declared with `gap` dispositions and no control, so the register prints them as gaps and no rule fires. `0005-partition-by-tenant.md` has stood in `draft` since 2026-01-05 and nothing says so. An accepted record edited after acceptance would be equally invisible, and no fixture can demonstrate an absence, so this file states it instead.

## What ages, and what does not

Two of the three findings are independent of the clock. The third is a windowed expectation measured from `status_since: 2026-06-15`, and it fires further past its window every day. It never stops firing, because no `decision_register` exists to satisfy it. So the verdict above is stable, and a rerun at a later date changes one number in one message.
