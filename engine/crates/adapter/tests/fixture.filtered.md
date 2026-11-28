## `headwater check`

37 findings, 20 of them errors across 28 of 31 documents, against `headwater/fixture` 1.0.0 at `105e82f4b4435094f6bfd8ba5b3c3f3380341b35aeebc9aca2c3d508726d1601`, over tree `sha256:f3b8a14c1f935359b70991fbc5c6a753c7ca5824e24f08e651db5298ac05dfcf`, evaluated at 2026-08-12.

**Narrowed by a path filter.** Every document was evaluated, and the findings below are only those on a path that one of these patterns matches: `check/spec/0*.md`, `check/nowhere/**`. 33 findings outside the filter were held back.

1 of those patterns matched no file and no finding:

- `check/nowhere/**`

**Coverage.** This run saw 31 files and classified 29 of them, and left 1 to `headwater generate --check`, which holds a generated file to the bytes its emitter writes. It created 483 check instances, and 119 of them reached no verdict.

- 58 — this document declares no warrant, and an absent warrant is not a value outside the closed set
- 1 — no `id` facet, which is the key this engine reads an identifier from and which no declaration states
- 1 — the `id` facet is a sequence, and an identifier is a word
- 22 — the shelf is heterogeneous, so the discriminator is the gap metadata fills, and kind resolution already read it
- 2 — `guarded` forbids hedging, and this engine has no pattern set for it
- 1 — `declarative` binds this kind and this document carries no prose to read: no body and no `summary` facet with a value
- 2 — `ste_strict` declares ASD-STE100, profile strict, and this engine has no rules for it
- 31 — change-scoped-only: the prior version is available only in change-scoped evaluation, and this run carries no change
- 1 — the document at the target end, `FIX-REG-open-questions`, declares no value for the state facet, so there is no state to read there

| Severity | Where | Rule | Finding |
|---|---|---|---|
| info | `check/spec/00-both-halves.md:30` | `relation.mention.undeclared` | the prose names `SPEC-FIX-fragment-target` and the front matter declares no relation to it; the taxonomy offers the relation `assesses` from `design_spec` to `design_spec` |
| error | `check/spec/01-one-half.md:9` | `relation.reciprocity.missing` | `SPEC-FIX-one-half` declares `cites_evidence: EVAL-FIX-beta`, and `cites_evidence` requires both ends, so check/evaluations/beta.md owes `cited_by` |
| warn | `check/spec/01-one-half.md:18` | `language.controlled.not_met` | `ste_house` holds prose to 25 words a sentence, and this one has 34 |
| warn | `check/spec/03-no-instance.md` | `coverage.document_unchecked` | this document is classified and all 4 of its check instances were skipped |
| error | `check/spec/04-untyped.md:1` | `shelf.discriminator.unresolved` | the shelf is heterogeneous and the document declares no `doc_type`, so no kind resolves and no check of a kind reaches this document |
| error | `check/spec/05-no-summary.md` | `facet.required.missing` | `design_spec` requires the facet `summary`, and it is not declared |
| warn | `check/spec/05-no-summary.md:12` | `language.controlled.not_met` | `ste_house` holds prose to 25 words a sentence, and this one has 31 |
| error | `check/spec/06-retired.md:4` | `facet.value.not_permitted` | `status` admits draft, current, superseded, and this document declares `retired` |
| warn | `check/spec/06-retired.md:11` | `language.controlled.not_met` | `ste_house` holds prose to 25 words a sentence, and this one has 34 |
| warn | `check/spec/07-prose-defects.md:15` | `voice.forbidden_construction` | `declarative` forbids future_intent, and this sentence writes `will be` |
| warn | `check/spec/07-prose-defects.md:17` | `voice.forbidden_construction` | `declarative` forbids change_narration, and this sentence writes `we renamed` |
| warn | `check/spec/07-prose-defects.md:19` | `voice.forbidden_construction` | `declarative` forbids phased_rollout, and this sentence writes `for now` |
| warn | `check/spec/07-prose-defects.md:23` | `language.controlled.not_met` | `ste_house` holds prose to 25 words a sentence, and this one has 43 |
| error | `check/spec/07-prose-defects.md:25` | `language.controlled.not_met` | `ste_house` admits no contraction, and this sentence writes `doesn't` |
| error | `check/spec/07-prose-defects.md:27` | `language.controlled.not_met` | `ste_house` declares the tag `en-US`, and this sentence writes `behaviour` |
| warn | `check/spec/07-prose-defects.md:29` | `language.controlled.not_met` | `ste_house` admits no semicolon in running prose, and this sentence writes one |
| warn | `check/spec/07-prose-defects.md:35` | `language.controlled.not_met` | `ste_house` holds prose to 25 words a sentence, and this one has 49 |
| warn | `check/spec/07-prose-defects.md:35` | `language.retired_term.used` | `ste_house` retires `robust`, and this sentence writes it: it stands in for the failure the thing survives |
| warn | `check/spec/07-prose-defects.md:35` | `voice.forbidden_construction` | `declarative` forbids future_intent, and this sentence writes `will be` |
| warn | `check/spec/07-prose-defects.md:39` | `language.retired_term.used` | `ste_house` retires `robust`, and this sentence writes it: it stands in for the failure the thing survives |
| warn | `check/spec/07-prose-defects.md:39` | `voice.forbidden_construction` | `declarative` forbids future_intent, and this sentence writes `will be` |
| error | `check/spec/07-prose-defects.md:47` | `language.controlled.not_met` | `ste_house` admits no contraction, and this sentence writes `doesn't` |
| error | `check/spec/07-prose-defects.md:51` | `language.retired_term.used` | `ste_house` retires `reference system`, and this sentence writes it: the corpus renamed it |
| warn | `check/spec/07-prose-defects.md:53` | `language.retired_term.used` | `ste_house` retires `robust`, and this sentence writes it: it stands in for the failure the thing survives |
| error | `check/spec/07-prose-defects.md:64` | `language.source_form.not_met` | `ste_house` writes one paragraph on one line, and this line continues the paragraph above |
| error | `check/spec/07-prose-defects.md:65` | `language.source_form.not_met` | `ste_house` writes one paragraph on one line, and this line continues the paragraph above |
| error | `check/spec/07-prose-defects.md:69` | `link.fragment.unresolved` | `#no-such-section` names no heading of this document |
| error | `check/spec/07-prose-defects.md:71` | `link.path.unresolved` | `19-renamed.md` names no file of this repository: nothing stands at `check/spec/19-renamed.md` |
| error | `check/spec/07-prose-defects.md:73` | `link.path.unresolved` | `19-renamed.md#a-heading` names no file of this repository: nothing stands at `check/spec/19-renamed.md` |
| error | `check/spec/07-prose-defects.md:75` | `link.path.unresolved` | `../../../outside.md` does not name a path of this repository at all: `check/spec/../../../outside.md` climbs above the repository |
| error | `check/spec/07-prose-defects.md:77` | `link.path.unresolved` | `19-renamed.md?v=2` names no file of this repository: nothing stands at `check/spec/19-renamed.md` |
| error | `check/spec/07-prose-defects.md:79` | `link.path.unresolved` | `/spec/00-both-halves.md` names no file of this repository: nothing stands at `/spec/00-both-halves.md` |
| error | `check/spec/07-prose-defects.md:81` | `link.identifier.mismatch` | the link text names `SPEC-FIX-cited-only`, but `20-fragment-target.md` reaches `check/spec/20-fragment-target.md`, which is `SPEC-FIX-fragment-target` |
| info | `check/spec/07-prose-defects.md:81` | `relation.mention.undeclared` | the prose names `SPEC-FIX-cited-only` and the front matter declares no relation to it; the taxonomy offers the relation `assesses` from `design_spec` to `design_spec` |
| error | `check/spec/08-contract-met.md:2` | `identifier.claim.missing` | `DR-FIX-0008` is spent by check/spec/08-contract-met.md and no file of `.headwater/ids` claims it, so this identifier is invisible to the allocator of every other branch and a second document can be minted onto it |
| error | `check/spec/09-contract-missing.md` | `section.required.missing` | `decision_record` requires the section `Decision`, and no heading of this document says so |
| error | `check/spec/09-contract-missing.md:2` | `identifier.claim.missing` | `DR-FIX-0009` is spent by check/spec/09-contract-missing.md and no file of `.headwater/ids` claims it, so this identifier is invisible to the allocator of every other branch and a second document can be minted onto it |

<details><summary>3 not reported: 1 migration-pending, 2 suppressed</summary>

| Held by | Where | Rule | Finding |
|---|---|---|---|
| migration-pending | `check/evaluations/delta.md:8` | `relation.reciprocity.missing` | `EVAL-FIX-delta` declares `cited_by: SPEC-FIX-cited-only`, and `cites_evidence` requires both ends, so check/spec/02-cited-only.md owes `cites_evidence` |
| suppression | `check/spec/10-suppressed.md:17` | `voice.forbidden_construction` | `declarative` forbids future_intent, and this sentence writes `will be` |
| suppression | `check/spec/10-suppressed.md:21` | `language.controlled.not_met` | `ste_house` holds prose to 25 words a sentence, and this one has 33 |

</details>

31 documents in the read set, and 10 barriers that no gate carries across a merge. 29 obligations, 23 verified.
