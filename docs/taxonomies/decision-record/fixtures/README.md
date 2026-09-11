# Fixtures for the decision-record taxonomy

A miniature ADR log in the tradition, under `corpus/`, with the obligation register that this entry pairs with it. Beacon is invented, and it is the same invented system that the [design-spec fixtures](../../design-spec/fixtures/README.md) use, so the two entries are visibly two traditions over one project.

Everything here is invented, including the names. `warrant: accepted` requires an `accepted_by` that names a human, so the fixtures write `fixture-acceptor`.

## A runner reads these files now, and that changes what this file is

The design-spec fixtures state their expected findings in prose, and they say why: "No runner reads these files. There is no engine." An engine exists. So this file states what a run reports, and it states it because a run reported it rather than because a reader predicted it.

The run is not automatic, and nothing in CI performs it. `headwater check` reads one corpus root, and this tree sits inside a directory that this repository excludes from its own corpus. To reproduce the output below, assemble a root that holds `packages/`, both library entries under `docs/taxonomies/`, and this fixture tree at `docs/`, then bind it to the base package with `bundles: [design-spec, decision-record]`. That assembly is four `cp` commands and a nine-line `.headwater/taxonomy.yml`.

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

## The external corpus of criterion 4: `inspect_evals`

Everything above this line is invented. Everything below it was written by people who have never heard of Headwater, and that is the whole point of it.

`tools/repo/decision-record-fixtures.sh` runs every case in this section. The corpus is not in the tree as typed documents: the runner assembles it from the pinned files under `sources/` at each run, adding front matter and nothing else, so no typed copy can drift from its source. The Beacon record above stands, and this record is beside it rather than in place of it.

Admission criterion 4 asks for "at least one **external** real or realistic corpus, typed by the entry **and recorded with its source, revision, paths, and run**". This is the first external corpus for this entry, and the fourth in the library: `design-spec`, `standards-spec` and `diataxis-site` carry one each, and `diataxis`, `brd-prd` and `evidence-and-obligation` carry none.

### Source, revision and paths

| | |
|---|---|
| Upstream | [`UKGovernmentBEIS/inspect_evals`](https://github.com/UKGovernmentBEIS/inspect_evals) |
| License | MIT, so the vendored copy below is clean with attribution. `sources/inspect-evals/LICENSE` is the upstream file at the same pin |
| Fork | [`headwater-ai/inspect_evals`](https://github.com/headwater-ai/inspect_evals), created 2026-09-11 |
| Pin | `360484a06383f9260279938262d78ed646ddbca1`, committed 2026-09-11T14:42:16Z on `main` |
| Paths | the 10 files of `adr/` at that commit, unedited |
| SHA-256 of `sources/inspect-evals/LICENSE` | `c593c2afc81388521eaeb03f0d1a4b01bc3e5147ed9cb16c1f129e5be353b6af` |

The runner reads the table below. Each row names the kind, the shelf its documents stand on, the pinned file, the path the assembled document takes, and the SHA-256 of the pinned file. A file under `sources/inspect-evals/adr/` with no row here fails the run, and a row naming a file that is not there fails it too.

| Kind | Shelf | Pinned source | Assembled path | SHA-256 of the pinned file |
|---|---|---|---|---|
| `decision` | `decisions` | `sources/inspect-evals/adr/0001-eval-metadata-location.md` | `docs/decisions/0001-eval-metadata-location.md` | `731e1b25f334f4eb11d73c6a09cacc1a3fa87bf32e191d14158c119cde88a67e` |
| `decision` | `decisions` | `sources/inspect-evals/adr/0002-versioning-metadata.md` | `docs/decisions/0002-versioning-metadata.md` | `85b0d71f0f5ff26ccadd3df0d91bfd0325a2a9f0aad6f4b52596baeea26996c3` |
| `decision` | `decisions` | `sources/inspect-evals/adr/0003-use-huggingface-hub-for-asset-hosting.md` | `docs/decisions/0003-use-huggingface-hub-for-asset-hosting.md` | `d0a6b87a6191af9d5206c53c9910d9274f083fd77a02c4eb2d1617c2ea4659c4` |
| `decision` | `decisions` | `sources/inspect-evals/adr/0004-no-floating-refs-for-external-assets.md` | `docs/decisions/0004-no-floating-refs-for-external-assets.md` | `a4a1bba67f33580bd0a25ed935d2c54f5c2bc7aede1dbd20ca13654cbe9bc31a` |
| `decision` | `decisions` | `sources/inspect-evals/adr/0005-asset-manifest-in-per-eval-config.md` | `docs/decisions/0005-asset-manifest-in-per-eval-config.md` | `a7defd579ef81351d94db669b40ac8d39e2465f895a637745a127f6788e8e8a3` |
| `decision` | `decisions` | `sources/inspect-evals/adr/0006-manual-upload-workflow-before-automation.md` | `docs/decisions/0006-manual-upload-workflow-before-automation.md` | `1c79a5b5379b334f69957f917c3e67eff799eaa1796088eb4e1b3ca837719f0c` |
| `decision` | `decisions` | `sources/inspect-evals/adr/0007-dataset-hosting-escalation-policy.md` | `docs/decisions/0007-dataset-hosting-escalation-policy.md` | `57ffbee40f7be6d1baf06a0f3f9568b76e0a30536bfeb8f2506c56ae3aa7e433` |
| `decision` | `decisions` | `sources/inspect-evals/adr/0008-task-configurability-uses-standard-inspect-layers.md` | `docs/decisions/0008-task-configurability-uses-standard-inspect-layers.md` | `3c84dbb08977b9ea4764eff7d3cb99d392b9591775e901e7baee936f36d288bf` |
| `decision` | `decisions` | `sources/inspect-evals/adr/0009-dependency-isolation-for-conflict-heavy-evals.md` | `docs/decisions/0009-dependency-isolation-for-conflict-heavy-evals.md` | `bdc896e9d51376c5244784e49fb5503b62205dd1818f87ce59df9ed700e26745` |
| `decision` | `decisions` | `sources/inspect-evals/adr/0010-hardened-cross-repo-ci-for-fork-pr-tests.md` | `docs/decisions/0010-hardened-cross-repo-ci-for-fork-pr-tests.md` | `251cdebf704006baaf2f10f58cf451ab59440778fe2229ace4789a2dcd5bebee` |

The identifier of each assembled document comes from the `pattern` of `identifier_schemes.decision_id` in the base package, with the namespace `IE` and the sequence read off the file name. No identifier is written into the runner.

### The status map, which is where this corpus bites

Front matter must carry `status`, and the value set is the base `lifecycle_state` vocabulary. The corpus writes its own state in a `## Status` heading, in prose. The runner maps one to the other by longest matching prefix of the table below, which is the ladder the [doctrine](../doctrine.md#the-lifecycle-ladder-maps-and-two-of-its-rungs-do-not) already states. A `## Status` value matching no row fails the run, so a re-pin that brings a new state word reddens this suite instead of being mapped away.

| The prose the corpus writes | The state the front matter carries |
|---|---|
| `Proposed` | `draft` |
| `Accepted` | `current` |
| `Superseded` | `superseded` |
| `Deprecated` | `deprecated` |
| `Rejected` | `deprecated` |

**A map by prefix is a lossy map, and the runner counts what it drops.** Four of the ten `## Status` values carry a second clause that the enum has no cell for: `0003`, `0005` and `0006` read `Accepted — implementation deferred`, and `0007` reads `Accepted — Stage 0→1 active, Stage 2+ deferred`. Each maps to `current`, and the clause after the dash is gone. The run prints that count, and the [evaluation](../../../evaluations/inspect-evals-worked-example.md) states what the dropped clause means.

### The case table

| Case | Expected result |
|---|---|
| Population guard | the source table carries at least one row, and `sources/inspect-evals/adr/` holds at least one file |
| Source coverage | the source table and the vendored directory name the same files, in both directions |
| Kind coverage | every kind the source table names is a kind the resolved taxonomy declares |
| License | `sources/inspect-evals/LICENSE` is present and is the file its digest seals |
| Digest seal | each pinned file's SHA-256 is the one the source table records |
| Byte identity | each assembled document, below its front-matter block, is its pinned source byte for byte |
| Status map | every `## Status` value in the corpus matches a row of the status map |
| Resolution | a root that selects `decision-record` resolves over the assembled corpus |
| External corpus typing | every document of the assembled corpus is typed, and all of them at `decision` |
| Section contract | `section.required.missing` is reported for no document of the corpus |

**The last case is an assertion and the finding denominators below are not.** The section contract is the one property of this corpus that is a claim about the tradition rather than a property of one team's prose, and it is the claim the entry makes in its own doctrine: Nygard's three sections are a convention the tradition carries, so a real ADR log should satisfy a contract written from it. A run that started reporting a missing section would be evidence against the entry, and that is worth a red suite. Everything else the run reports is printed and asserted by nothing, because a warning count is a property of somebody else's prose and a suite that asserted it would redden the day a rule of the engine widened.

**Two cases hold the pinning, and each holds only half of it.** The byte-identity case strips the front matter the runner wrote and compares what is left against the pinned file with `cmp`. It cannot fail on the content of a pinned source, because the runner assembles the document out of the same file it compares back to: it holds the stripper and it says nothing about whether the vendored file is still what the cited commit holds. The digest case is the other half, and it is what catches an edit to a vendored file. The whole value of an external corpus is that its prose was not written to pass, so a run that is green because somebody quietly repaired a heading, a status line or a link has measured this repository against itself again. Nothing else here detects that.

**Neither case reaches the upstream commit, and nothing in this repository does.** A digest recorded beside the file it was computed from seals against local drift and proves no provenance. Confirming that `360484a0` still holds these bytes takes a network fetch that no fixture runner here performs.

### What the runner enumerates rather than lists

No file name, kind name, shelf path, identifier or expected finding count is written into `tools/repo/decision-record-fixtures.sh`. The source rows come from the table above, the kinds come from the resolved taxonomy of the scratch root, the identifier pattern comes from `identifier_schemes.decision_id`, the status map comes from the status table above, and `status_since` comes from each document's own `## Date` heading. Each group opens with a population floor of zero, because a judge whose population came back empty reports green for the wrong reason.

**One value is a constant the runner writes, and it is a finding.** `summary` is required on every `governed_document` and nothing in an ADR of this tradition supplies one. The tradition writes a title and then a Context section; it has no one-sentence scent line, and no mechanical rule derives one from the prose without inventing it. So the runner writes the same sentence into all ten documents, and the corpus cannot satisfy that facet honestly. The evaluation carries this as a finding rather than a defect in their writing.

### The run record that criterion 4 asks for

Criterion 4 asks for a recorded run. It does not ask for a clean one. Recorded on 2026-09-11 by `sh tools/repo/decision-record-fixtures.sh`, against engine 0.1.2 and `headwater/standard` 4.2.0:

| Denominator | Reading |
|---|---|
| Files under the corpus root | RUN_DOCS |
| Typed | RUN_TYPED, all at `decision` |
| Findings | RUN_FINDINGS |
| Errors | RUN_ERRORS |
| Warnings | RUN_WARNS |
| Exit status | RUN_EXIT_PLAIN from `headwater check`, RUN_EXIT_STRICT from `headwater check --strict` |
| `section.required.missing` | 0, in 0 of the 10 documents |
| `## Status` values the enum has no cell for | 4 of 10 |

RUN_BY_RULE

### What the two link populations are, and why they are counted apart

The fourth Done-when box of [#488](https://github.com/headwater-ai/headwater/issues/488) asks whether the cross-links resolve, because nothing in the upstream `ruff` / `mdformat` / `markdownlint` stack resolves a link. Measured at the pin over the 10 files, with fenced code stripped: **43 inline markdown links, 27 absolute and 16 relative.** One of the 16 is a pure fragment. Of the remaining 15, **15 resolve against the upstream tree at `360484a0`**, including the 5 links into `internal/implementation-plans/external-asset-hosting.md`, the 7 ADR-to-ADR links, the 2 into `../docs/task-configurability.md` and the 1 into `../.github/security/incident-response-runbook.md`. The one fragment, `#related-work` in `0009`, resolves against that file's own `## Related Work` heading. There are no reference-style links and no autolinks in the set.

**A run inside the assembled corpus answers a different question, and its answer is not a defect in their prose.** 8 of the 15 point out of `adr/`, so a corpus root that holds the 10 files alone cannot resolve them and `headwater check` reports `link.path.unresolved`. That is a property of lifting files out of a repository, recorded rather than repaired, and it is the same distinction the [`diataxis-site` run record](../../diataxis-site/fixtures/README.md#the-run-record-that-criterion-4-asks-for) draws. The question the issue asked is the upstream one, and the upstream answer is 15 of 15.

**The cross-reference convention the issue describes is mostly prose that no tool can follow.** No file carries a `## Related ADRs` heading; `0007` carries the literal words as a prose lead-in on one line. Across the set there are 21 textual `ADR-00NN` references, of which 10 are each file's own identifier in its `#` title. Of the 11 that point at another record, 7 are markdown links and 4 are bare text. A cross-reference written as bare text is invisible to a link checker on either side of the fence.
