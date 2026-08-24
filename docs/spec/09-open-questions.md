---
"headwater:generated": "shelf_sections. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-REG-open-questions
doc_type: decision_register
relations:
  superseded_by:
    - HW-REG-decisions
    - HW-REG-open-obligations
---

# decisions

34 documents on this shelf, in the reading order this corpus derives. Each heading below is the name that the document declares, so a citation of a heading is a citation of a document.

## Q1 — Implementation language

[HW-DR-0001](../decisions/0001-implementation-language.md) — The language is Rust, with a WebAssembly build of the same crate for editor and browser embedding.

## Q2 — Schema format

[HW-DR-0002](../decisions/0002-schema-format.md) — YAML 1.2 is the concrete syntax, and Headwater owns the schema language, the reference sublanguage, the overlay language and the meta-schema.

## Q3 — How much of the default taxonomy ships in the box

[HW-DR-0003](../decisions/0003-how-much-of-the-default-taxonomy-ships-in-the-box.md) — The base package is minimal and derived from the core, and optional content ships as add-only bundles.

## Q4 — Relation storage

[HW-DR-0004](../decisions/0004-relation-storage.md) — Front matter is authoritative, and a relation instance is an object rather than a pointer.

## Q5 — Voice checking depth

[HW-DR-0005](../decisions/0005-voice-checking-depth.md) — Voice checking stays lexical, with a curated pattern set, a per-category posture and a reasoned escape hatch.

## Q6 — Where the corpus graph lives at rest

[HW-DR-0006](../decisions/0006-where-the-corpus-graph-lives-at-rest.md) — The graph never rests. Every run rebuilds it, and no derived artifact is canonical for anything.

## Q7 — Scope of the MCP surface

[HW-DR-0007](../decisions/0007-scope-of-the-mcp-surface.md) — Three classes of tool, and a landed write never ships.

## Q8 — Probe cost and cadence

[HW-DR-0008](../decisions/0008-probe-cost-and-cadence.md) — A probe is a document with a declared expectation, and cadence follows the purpose of the run.

## Q9 — Multi-repository corpora

[HW-DR-0009](../decisions/0009-multi-repository-corpora.md) — A repository holds one or more corpora, the tier above harvests pinned exports, and no merged graph exists.

## Q10 — Naming

[HW-DR-0010](../decisions/0010-naming.md) — The name is Headwater, capitalized in prose and lower case as an identifier.

## Q11 — License and distribution posture

[HW-DR-0011](../decisions/0011-license-and-distribution-posture.md) — Apache-2.0 for the engine, the library, the base package and the bundles, ratified by the owner on 2026-08-11.

## Q12 — Migration path for an existing corpus

[HW-DR-0012](../decisions/0012-migration-path-for-an-existing-corpus.md) — Adoption is a migration from no taxonomy, and `headwater infer` computes the adoption payload.

## Q13 — LinkML and SHACL as substrate

[HW-DR-0013](../decisions/0013-linkml-and-shacl-as-substrate.md) — Headwater owns the language, emitters never chain, and LinkML is the last of six siblings.

## Q14 — Discovery surface

[HW-DR-0014](../decisions/0014-discovery-surface.md) — The corpus descriptor is a generated projection at `.headwater/corpus.json`, and registration went to Q16.

## Q15 — A synthesized content tier

[HW-DR-0015](../decisions/0015-a-synthesized-content-tier.md) — Every document carries a warrant from a closed set of four, and `asserted` content is admitted with limits.

## Q16 — Public presence

[HW-DR-0016](../decisions/0016-public-presence.md) — Registration needs a channel with an obliged reader, a directory of corpora is refused, and the site is a projection of this corpus.

## Q17 — Governed access and the solution layer

[HW-DR-0017](../decisions/0017-governed-access-and-the-solution-layer.md) — The serving boundary is the export step of each publishing corpus. A profile filters for an audience, and Headwater has no principals.

## Q18 — Recording adjudicated disagreements

[HW-DR-0018](../decisions/0018-recording-adjudicated-disagreements.md) — The edge does not become a node. An adjudication is a decision document that carries `overrides`.

## Q19 — Inbound integration: an external system of record

[HW-DR-0019](../decisions/0019-inbound-integration-an-external-system-of-record.md) — Imported content is `transcribed` against a committed pin, and an imported edge carries full weight from the first release.

## Q20 — Where scent lives

[HW-DR-0020](../decisions/0020-where-scent-lives.md) — An optional source-owned cue sits on a relation instance, graded against the alternatives in view at the point of decision.

## Q21 — Terminological succession, and validity under merge

[HW-DR-0021](../decisions/0021-terminological-succession-and-validity-under-merge.md) — A retired-term lexicon sits in the language regime, and a verdict reports the read set that a merge may void.

## Q22 — The integrity posture of a published package

[HW-DR-0022](../decisions/0022-q22-the-integrity-posture-of-a-published-package.md) — A package digest checks a fetched artifact against a pin that a person committed, and it is never a signature. The in-house SHA-256 is held against a second implementation rather than replaced by a dependency.

## Q23 — The engine lint floor

[HW-DR-0023](../decisions/0023-the-engine-lint-floor.md) — A published Rust guideline set is not installed as a skill, because a skill carries no rule of its own. Twenty-two lints are declared once in the workspace manifest, and each was chosen by measuring the corpus rather than by adopting a list.

## Q24 — Readability, and what a sweep can be asked about

[HW-DR-0024](../decisions/0024-q24-readability-and-what-a-sweep-can-be-asked-about.md) — Readability is not a sweep class and gets no verb, because every class of a sweep names two things that do not fit and a readability finding names one. A source file is not a slice member, because admitting one degrades the membership refusal for every class. (asserted, and no human has accepted it)

## Q25 — Where the namespace goes in an identifier, and who declares it

[HW-DR-0025](../decisions/0025-q25-where-the-namespace-goes-in-an-identifier-and-who-declares-it.md) — The namespace opens every identifier, and a published source declares no namespace at all, because any constant it wrote would be minted by every adopter of it at once. (asserted, and no human has accepted it)

## Q26 — Whether terminality belongs to a state, or to a state and a regime

[HW-DR-0026](../decisions/0026-q26-whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime.md) — Terminality stays a property of a state alone, because the per-regime reading is available at both call sites and costs the deferral that keeps one defect one finding. No published tradition wants the collision, and two that met it minted a second state name. (asserted, and no human has accepted it)

## Q27 — Whether a decision record is governed prose

[HW-DR-0027](../decisions/0027-q27-whether-a-decision-record-is-governed-prose.md) — A decision record is governed prose, and the overlay binds the house language regime to it. The shelf carries a prose problem rather than a quotation problem, and collapsing all 37 inline quotations moves 64 findings to 63. (asserted, and no human has accepted it)

## Q28 — Whether an evaluation is governed prose

[HW-DR-0028](../decisions/0028-q28-whether-an-evaluation-is-governed-prose.md) — An evaluation is governed prose, and the overlay binds the house language regime to it. The shelf carries 432 advisory findings, and the measurement that explains them is whether a rule was reading the prose when it was written rather than who wrote it. (asserted, and no human has accepted it)

## Q29 — Whether a corpus root may contain code, and what an interface contract may reach

[HW-DR-0029](../decisions/0029-q29-whether-a-corpus-root-may-contain-code-and-what-an-interface-contract-may-reach.md) — The corpus root stays `docs`, because a path is corpus content only where a file with no front matter is a defect, and 185 of the 186 Markdown files under `engine/` exist to be defective. An interface contract lives inside the root and reaches a crate by a `governs` edge that binds on existence alone. (asserted, and no human has accepted it)

## Q30 — Whether what an obligation waits on is a state or a property, and how many values it takes

[HW-DR-0030](../decisions/0030-q30-whether-what-an-obligation-waits-on-is-a-state-or-a-property-and-how-many-values-it-takes.md) — What an obligation waits on is a state and not a property, and the closed set holds four values. A required `waiting_on` facet on `obligation_record` takes `ruling`, `build`, `measurement` or `adopter`, and the decision-record bundle declares it. (asserted, and no human has accepted it)

## Q31 — Whether this repository becomes public, and when

[HW-DR-0031](../decisions/0031-q31-whether-this-repository-becomes-public-and-when.md) — This repository becomes public, and the date is unset. A settled posture with a pending date is not an open question, and the record is what stops each run from raising it as one. (asserted, and no human has accepted it)

## Q32 — Which test the self-audit label states

[HW-DR-0032](../decisions/0032-q32-which-test-the-self-audit-label-states.md) — The `self-audit` label states the reader test and not the provenance test. How a finding was found is not the test, and the only question is whether a reader outside this repository is better off. (asserted, and no human has accepted it)

## Q33 — Whether the command line is derived, and who a flag belongs to

[HW-DR-0033](../decisions/0033-q33-whether-the-command-line-is-derived-and-who-a-flag-belongs-to.md) — `clap` derives the command line for 17 lock entries, and a flag belongs to the verb that reads it rather than to the binary. (asserted, and no human has accepted it)

## Q34 — Whether acceptance means merged to main, and what an agent may write before that

[HW-DR-0034](../decisions/0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md) — Acceptance is the merge onto main. A provenance block on an unmerged branch is a proposal, so an agent may write `accepted_by` there, and stop rule 5 binds main rather than the byte. (asserted, and no human has accepted it)
