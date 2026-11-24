# Fixtures for the requirement-verification taxonomy

A miniature verification chain for Beacon, under `corpus/`. Beacon is invented, and it is the system that the [brd-prd fixtures](../../brd-prd/fixtures/README.md) and the other entries of this library also use, so a reader can hold every entry over one project. Its `webhook-replay` initiative there is the source of the requirement here.

Everything here is invented, including the names. `warrant: accepted` requires an `accepted_by` that names a human, so the fixtures write `fixture-acceptor`. The namespace is `SCR`, which `overlay.yml` beside this file declares and no bundle may.

Every number below came from the run this file quotes. Nothing here is predicted.

## The source, the revision and the run

- **Source:** the files under `corpus/`, written for this entry on 2026-11-24 (#2017). `corpus/docs/` is the corpus root, `corpus/src/` holds the one test file that a verification cites, and `corpus/.headwater/ids/` holds the claim of each identifier.
- **Revision:** the commit that carries this file.
- **Taxonomy:** `headwater/standard` 4.25.0, with `bundles: [evidence-and-obligation, decision-record, requirement-verification]`.
- **Run:** `headwater check --no-cache --now 2026-11-24` over the assembled root, then the same with `--strict`.

## How to run this corpus

`headwater check` reads one corpus root, and this tree sits inside a directory that this repository excludes from its own corpus. So assemble a scratch root.

1. Publish the package: `headwater taxonomy publish --from taxonomy-source/headwater-standard --out <artifact>`. It prints the digest.
2. In the scratch root, copy `corpus/docs/` to `docs/`, `corpus/src/` to `src/`, and `corpus/.headwater/ids/` to `.headwater/ids/`. Copy `overlay.yml` to `.headwater/overlay.yml`.
3. Write `.headwater/taxonomy.yml`. It names `package: headwater/standard`, the version and the digest that step 1 printed, the three bundles above, `overlay: .headwater/overlay.yml` and `corpus.root: docs`.
4. Run `headwater taxonomy vendor <artifact>`, then `headwater taxonomy resolve`, then the run above.

**The selection names three entries, and the entry requires one.** This entry requires `evidence-and-obligation` for `purposes.evidence`. That bundle also declares `purposes.obligation`, and it ships no kind that serves it, so `taxonomy resolve` over a selection of the two refuses with a purpose-completeness error. `decision-record` supplies `obligation_record`, which serves it. [Finding 3](../doctrine.md#findings) records this.

The overlay is six lines of `add`, and it declares no kind, relation, shelf or purpose. That is the claim this entry makes to an adopter, and `engine/crates/cli/tests/requirement_bundle.rs` holds it on every run of the workspace suite.

## What each document exercises

| Document | Kind | What it is here for |
|---|---|---|
| `docs/verified-requirements/0001-an-operator-can-replay-a-failed-delivery.md` | `requirement` | The clean requirement. Three criteria verify it, and its body holds a sentence in the future tense that the `requirement_prospective` regime admits |
| `docs/acceptance-criteria/0001-a-replay-reaches-the-endpoint-within-one-minute.md` | `acceptance_criterion` | The clean criterion, method `test`, with both `verifies` and `proven_by` written |
| `docs/acceptance-criteria/0002-the-replay-log-names-the-operator-and-the-reason.md` | `acceptance_criterion` | Method `inspection`, 176 days at `current` and no verification. The expectation does not read it, so it is silent by design |
| `docs/acceptance-criteria/0003-a-replay-keeps-the-original-event-identifier.md` | `acceptance_criterion` | Planted defect 1: method `test`, 176 days at `current`, and no verification proves it |
| `docs/verified-requirements/0002-a-delivery-reaches-its-endpoint-within-five-seconds.md` | `requirement` | Planted defect 2: 176 days at `current`, and nothing verifies it |
| `docs/verifications/0001-the-replay-latency-test-measures-twenty-replays.md` | `verification` | The clean verification. It proves `SCR-AC-0001`, and `cited_in` binds to `src/replay_latency_test.rs` through the comment there |

## What a run reports

**6 files under the corpus root, 6 typed, 0 untyped, 6 checked, 153 check instances, 2 findings.** The census reads 2 `requirement`, 3 `acceptance_criterion` and 1 `verification`. The graph reads 6 nodes and 9 declared edge halves: 8 to a document and 1 to an anchor, the `test_site` `src/replay_latency_test.rs` through `comment-scan`. 0 prose links did not resolve. `headwater check` exits 0, and `headwater check --strict` exits 0, because both findings are warnings.

**Planted defect 1.** `docs/acceptance-criteria/0003-a-replay-keeps-the-original-event-identifier.md:4:1` reports `relation.participation.overdue`, a warning, under `OB-REL-3`:

    `acceptance-criterion-proven`: 176 days since `status_since`, and this `acceptance_criterion` reaches no `proven_by` to a `verification` inside the 90 days the taxonomy allows

**Planted defect 2.** `docs/verified-requirements/0002-a-delivery-reaches-its-endpoint-within-five-seconds.md:4:1` reports `relation.participation.overdue`, a warning, under `OB-REL-3`:

    `requirement-verified`: 176 days since `status_since`, and this `requirement` reaches no `verified_by` inside the 90 days the taxonomy allows

## What a run reports that is not planted

**Nothing.** The two findings are the two plants.

Three silences are worth naming. `SCR-AC-0002` is past the window and reaches no verification, and nothing reports it, because the expectation reads only a criterion whose method is `test`. `SCR-REQ-0001` says that a replay "will reuse" a signing key, and no voice finding names it, because `requirement_prospective` does not forbid `future_intent`. No reciprocity finding appears, because each of the four pairs is written at both ends.

**An unclaimed identifier breaks the citation as well as the claim.** A first run of this corpus had no `.headwater/ids/`. It reported 7 errors: one `identifier.claim.missing` for each of the 6 documents, and one `relation.target.unresolved` for the `cited_in` edge. The graph section said that `SCR-VER-0001` "is shaped like an identifier this corpus mints, and no document mints it", though the document that mints it was in the corpus. The claim files are what removed all seven.

## What ages, and what does not

**Both findings are windowed, and their date is fixed so that they never stop firing.** Every document declares `status_since: 2026-06-01`, and the recorded run injects `--now 2026-11-24`, which is 176 days. The window is 90 days. A later clock reports more days and the same findings.

**The counts move if the corpus moves and not otherwise.** The entry sits outside the corpus root of this repository, so this file, the doctrine and the three templates are absent from its census. Add a document under `corpus/docs/` and every count above changes.
