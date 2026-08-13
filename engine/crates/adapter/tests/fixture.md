## `headwater check`

34 findings, 16 of them errors across 18 of 21 documents, against `headwater/fixture` 1.0.0 at `74633eb89de77b0e4c90e110f481d2f4a41fa4901b1ace760b37d66351822015`, evaluated at 2026-08-12.

| Severity | Where | Rule | Finding |
|---|---|---|---|
| warn | `check/evaluations/epsilon.md:4` | `relation.participation.overdue` | `evidence-cited`: 219 days since `status_since`, and this `evaluation` reaches no `cites_evidence` to a `design_spec` inside the 30 days the taxonomy allows |
| error | `check/evaluations/eta.md:8` | `relation.endpoint.not_permitted` | `assesses` declares `from: [design_spec]`, and `EVAL-FIX-eta` at that end has the kind `evaluation` |
| error | `check/evaluations/gamma.md:3` | `shelf.placement_is_primary` | `evaluations` is homogeneous and carries the kind `evaluation`, and this document restates it as `doc_type: evaluation` |
| error | `check/evaluations/gamma.md:9` | `relation.reciprocity.missing` | `EVAL-FIX-gamma` declares `cited_by: SPEC-FIX-both-halves`, and `cites_evidence` requires both ends, so check/spec/00-both-halves.md owes `cites_evidence` |
| error | `check/spec/01-one-half.md:9` | `relation.reciprocity.missing` | `SPEC-FIX-one-half` declares `cites_evidence: EVAL-FIX-beta`, and `cites_evidence` requires both ends, so check/evaluations/beta.md owes `cited_by` |
| warn | `check/spec/01-one-half.md:18` | `language.controlled.not_met` | `ste_house` holds prose to 25 words a sentence, and this one has 34 |
| warn | `check/spec/03-no-instance.md` | `coverage.document_unchecked` | this document is classified and all 2 of its check instances were skipped |
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
| error | `check/spec/07-prose-defects.md:35` | `language.retired_term.used` | `ste_house` retires `reference system`, and this sentence writes it: the corpus renamed it |
| warn | `check/spec/07-prose-defects.md:37` | `language.retired_term.used` | `ste_house` retires `robust`, and this sentence writes it: it stands in for the failure the thing survives |
| error | `check/spec/07-prose-defects.md:48` | `language.source_form.not_met` | `ste_house` writes one paragraph on one line, and this line continues the paragraph above |
| error | `check/spec/07-prose-defects.md:49` | `language.source_form.not_met` | `ste_house` writes one paragraph on one line, and this line continues the paragraph above |
| error | `check/spec/07-prose-defects.md:53` | `link.fragment.unresolved` | `#no-such-section` names no heading of this document |
| error | `check/spec/09-contract-missing.md` | `section.required.missing` | `decision_record` requires the section `Decision`, and no heading of this document says so |
| warn | `check/spec/10-suppressed.md:27` | `language.controlled.not_met` | `ste_house` holds prose to 25 words a sentence, and this one has 31 |
| error | `check/spec/11-identifier-mismatch.md:2` | `identifier.pattern.not_met` | `SPEC-HW-mismatch` does not match `SPEC-FIX-<slug>`, which scheme `spec_id` declares: `HW-mismatch` is where the namespace `FIX` was expected |
| error | `check/spec/13-dangling.md:9` | `relation.target.unresolved` | `SPEC-FIX-dangling` declares `assesses: SPEC-FIX-no-such-document`, and that target resolves to nothing at all |
| error | `check/spec/13-dangling.md:10` | `relation.target.unresolved` | `SPEC-FIX-dangling` declares `assesses: SPEC-FIX-untyped`, and that target names check/spec/04-untyped.md, which is untyped: the census gave it no kind, and an endpoint is a kind |
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

19 documents in the read set. 24 obligations, 19 verified.
