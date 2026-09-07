## `headwater check`

47 findings, 29 of them errors across 24 of 26 documents, against `headwater/fixture` 1.0.0 at `9df1915ec6a29614e9cba30132a126a5cc7591315ce0e880fda4b551982640c0`, evaluated at 2026-08-12.

**Scoped to a change.** This run was told what one change carries, so the checks that read the version a document stood at before it could run at all. 10 documents were named: 2 that the change adds, 1 with a prior version this run read, and 3 whose prior version did not read, so every check that needed one was skipped over it. The findings above are over the whole corpus, as they are in a run that names no change.

0 promoted from `asserted` to `accepted` in this change. Nothing declares how many promotions in one change is too many.

4 of those paths named no row of this corpus, so nothing was checked over them:

- `./check/evaluations/delta.md`
- `README.md`
- `check/spec/00-both-halves.markdown`
- `engine/crates/check/src/change.rs`

**Coverage.** This run saw 26 files and classified 24 of them, and left 1 to `headwater generate --check`, which holds a generated file to the bytes its emitter writes. It created 305 check instances, and 49 of them reached no verdict.

- 1 — no `id` facet, which is the key this engine reads an identifier from and which no declaration states
- 1 — the `id` facet is a sequence, and an identifier is a word
- 17 — the shelf is heterogeneous, so the discriminator is the gap metadata fills, and kind resolution already read it
- 2 — `guarded` forbids hedging, and this engine has no pattern set for it
- 2 — `ste_strict` declares ASD-STE100, profile strict, and this engine has no rules for it
- 22 — this document writes no fragment into itself
- 3 — the prior version at `a prior version that no tree holds` did not open: no prior version stands at this name
- 1 — the document at the target end, `FIX-REG-open-questions`, declares no value for the state facet, so there is no state to read there

4 readings of a path the census never walked, so the coverage above is computed over a set that does not hold them:

- `.headwater/ids`
- `.headwater/ids`
- `./check/evaluations/delta.md`
- `check/spec/00-both-halves.markdown`

| Severity | Where | Rule | Finding |
|---|---|---|---|
| warn | `check/evaluations/epsilon.md:4` | `relation.participation.overdue` | `evidence-cited`: 219 days since `status_since`, and this `evaluation` reaches no `cites_evidence` to a `design_spec` inside the 30 days the taxonomy allows |
| error | `check/evaluations/eta.md:8` | `relation.endpoint.not_permitted` | `assesses` declares `from: [design_spec]`, and `EVAL-FIX-eta` at that end has the kind `evaluation` |
| error | `check/evaluations/gamma.md:3` | `shelf.placement_is_primary` | `evaluations` is homogeneous and carries the kind `evaluation`, and this document restates it as `doc_type: evaluation` |
| error | `check/evaluations/gamma.md:9` | `relation.reciprocity.missing` | `EVAL-FIX-gamma` declares `cited_by: SPEC-FIX-both-halves`, and `cites_evidence` requires both ends, so check/spec/00-both-halves.md owes `cites_evidence` |
| error | `check/spec/01-one-half.md:9` | `relation.reciprocity.missing` | `SPEC-FIX-one-half` declares `cites_evidence: EVAL-FIX-beta`, and `cites_evidence` requires both ends, so check/evaluations/beta.md owes `cited_by` |
| warn | `check/spec/01-one-half.md:18` | `language.controlled.not_met` | `ste_house` holds prose to 25 words a sentence, and this one has 34 |
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
| error | `check/spec/07-prose-defects.md:39` | `language.controlled.not_met` | `ste_house` admits no contraction, and this sentence writes `doesn't` |
| error | `check/spec/07-prose-defects.md:43` | `language.retired_term.used` | `ste_house` retires `reference system`, and this sentence writes it: the corpus renamed it |
| warn | `check/spec/07-prose-defects.md:45` | `language.retired_term.used` | `ste_house` retires `robust`, and this sentence writes it: it stands in for the failure the thing survives |
| error | `check/spec/07-prose-defects.md:56` | `language.source_form.not_met` | `ste_house` writes one paragraph on one line, and this line continues the paragraph above |
| error | `check/spec/07-prose-defects.md:57` | `language.source_form.not_met` | `ste_house` writes one paragraph on one line, and this line continues the paragraph above |
| error | `check/spec/07-prose-defects.md:61` | `link.fragment.unresolved` | `#no-such-section` names no heading of this document |
| error | `check/spec/08-contract-met.md:2` | `identifier.claim.missing` | `DR-FIX-0008` is spent by check/spec/08-contract-met.md and no file of `.headwater/ids` claims it, so this identifier is invisible to the allocator of every other branch and a second document can be minted onto it |
| error | `check/spec/09-contract-missing.md` | `section.required.missing` | `decision_record` requires the section `Decision`, and no heading of this document says so |
| error | `check/spec/09-contract-missing.md:2` | `identifier.claim.missing` | `DR-FIX-0009` is spent by check/spec/09-contract-missing.md and no file of `.headwater/ids` claims it, so this identifier is invisible to the allocator of every other branch and a second document can be minted onto it |
| warn | `check/spec/10-suppressed.md:27` | `language.controlled.not_met` | `ste_house` holds prose to 25 words a sentence, and this one has 31 |
| error | `check/spec/11-identifier-mismatch.md:2` | `identifier.pattern.not_met` | `SPEC-XX-mismatch` does not match `SPEC-FIX-<slug>`, which scheme `spec_id` declares: `XX-mismatch` is where the namespace `FIX` was expected |
| error | `check/spec/13-dangling.md:9` | `relation.target.unresolved` | `SPEC-FIX-dangling` declares `assesses: SPEC-FIX-no-such-document`, and that target resolves to nothing at all |
| error | `check/spec/13-dangling.md:10` | `relation.target.unresolved` | `SPEC-FIX-dangling` declares `assesses: SPEC-FIX-untyped`, and that target names check/spec/04-untyped.md, which is untyped: the census gave it no kind, and an endpoint is a kind |
| error | `check/spec/14-relations-not-a-mapping.md:8` | `relation.declaration.unusable` | `relations` is a sequence, and it names relation types |
| error | `check/spec/15-unusable-entries.md:8` | `relation.declaration.unusable` | `invented_relation` is neither a declared relation nor a declared inverse |
| error | `check/spec/15-unusable-entries.md:11` | `relation.declaration.unusable` | `assesses` names EVAL-FIX-alpha twice, and the second declares no second edge |
| error | `check/spec/15-unusable-entries.md:12` | `relation.declaration.unusable` | an entry of `assesses` is a sequence, and an entry is a target or a mapping with `to` |
| error | `check/spec/15-unusable-entries.md:13` | `relation.declaration.unusable` | an entry of `assesses` declares no `to`, so it names no target |
| error | `check/spec/16-no-identifier.md:1` | `identifier.unusable` | a typed `design_spec` that declares no identifier, so no edge can name it, and every entry of its `relations:` block is lost with it, because an edge is identified by its source |
| error | `check/spec/17-identifier-not-a-word.md:2` | `identifier.unusable` | the identifier is not a scalar, and an identifier is a word |
| error | `check/spec/18-blank-facets.md:6` | `facet.value.blank` | `summary` is declared with no value |
| error | `check/spec/18-blank-facets.md:7` | `facet.value.blank` | `title` is declared as empty text |
| error | `check/spec/18-blank-facets.md:8` | `facet.value.blank` | `oracle` is declared as a sequence, and its declaration says `string` |
| warn | `check/spec/18-blank-facets.md:15` | `language.controlled.not_met` | `ste_house` holds prose to 25 words a sentence, and this one has 32 |
| warn | `engine/crates/check/fixtures/check.taxonomy.yml` | `control.mechanism.unimplemented` | control CT-FIX-15 names the mechanism check:no.such.rule, which this engine does not implement, so nothing discharges OB-FIX-18 |
| warn | `engine/crates/check/fixtures/check.taxonomy.yml` | `control.mechanism.unimplemented` | control CT-FIX-17 names the mechanism phase:no.such.phase, which this engine does not implement, so nothing discharges OB-FIX-20 |
| warn | `engine/crates/check/fixtures/check.taxonomy.yml` | `obligation.disposition.not_one` | obligation OB-FIX-16 carries no disposition: no control discharges it, and it states neither a gap nor an acceptance |
| warn | `engine/crates/check/fixtures/check.taxonomy.yml` | `obligation.disposition.not_one` | obligation OB-FIX-17 carries two dispositions: CT-FIX-14 discharges it, and it also states one for itself |
| warn | `engine/crates/check/fixtures/check.taxonomy.yml` | `obligation.disposition.not_one` | obligation OB-FIX-18 carries no disposition: CT-FIX-15 names a mechanism this engine does not implement, so nothing discharges it, and it states neither a gap nor an acceptance |
| warn | `engine/crates/check/fixtures/check.taxonomy.yml` | `obligation.disposition.not_one` | obligation OB-FIX-20 carries two dispositions: CT-FIX-17 claims to discharge it, and it also states one for itself |

<details><summary>3 not reported: 1 migration-pending, 2 suppressed</summary>

| Held by | Where | Rule | Finding |
|---|---|---|---|
| migration-pending | `check/evaluations/delta.md:8` | `relation.reciprocity.missing` | `EVAL-FIX-delta` declares `cited_by: SPEC-FIX-cited-only`, and `cites_evidence` requires both ends, so check/spec/02-cited-only.md owes `cites_evidence` |
| suppression | `check/spec/10-suppressed.md:17` | `voice.forbidden_construction` | `declarative` forbids future_intent, and this sentence writes `will be` |
| suppression | `check/spec/10-suppressed.md:21` | `language.controlled.not_met` | `ste_house` holds prose to 25 words a sentence, and this one has 33 |

</details>

28 documents in the read set, and 4 barriers that no gate carries across a merge. 27 obligations, 22 verified.
