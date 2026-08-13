---
"headwater:generated": "shelf_sections. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: REG-HW-open-questions
doc_type: decision_register
relations:
  superseded_by:
    - REG-HW-decisions
    - REG-HW-open-obligations
---

# decisions

21 documents on this shelf, in the reading order this corpus derives. Each heading below is the name that the document declares, so a citation of a heading is a citation of a document.

## Q1 — Implementation language

[DR-repo-0001](../decisions/0001-implementation-language.md) — The language is Rust, with a WebAssembly build of the same crate for editor and browser embedding.

## Q2 — Schema format

[DR-repo-0002](../decisions/0002-schema-format.md) — YAML 1.2 is the concrete syntax, and Headwater owns the schema language, the reference sublanguage, the overlay language and the meta-schema.

## Q3 — How much of the default taxonomy ships in the box

[DR-repo-0003](../decisions/0003-how-much-of-the-default-taxonomy-ships-in-the-box.md) — The base package is minimal and derived from the core, and optional content ships as add-only bundles.

## Q4 — Relation storage

[DR-repo-0004](../decisions/0004-relation-storage.md) — Front matter is authoritative, and a relation instance is an object rather than a pointer.

## Q5 — Voice checking depth

[DR-repo-0005](../decisions/0005-voice-checking-depth.md) — Voice checking stays lexical, with a curated pattern set, a per-category posture and a reasoned escape hatch.

## Q6 — Where the corpus graph lives at rest

[DR-repo-0006](../decisions/0006-where-the-corpus-graph-lives-at-rest.md) — The graph never rests. Every run rebuilds it, and no derived artifact is canonical for anything.

## Q7 — Scope of the MCP surface

[DR-repo-0007](../decisions/0007-scope-of-the-mcp-surface.md) — Three classes of tool, and a landed write never ships.

## Q8 — Probe cost and cadence

[DR-repo-0008](../decisions/0008-probe-cost-and-cadence.md) — A probe is a document with a declared expectation, and cadence follows the purpose of the run.

## Q9 — Multi-repository corpora

[DR-repo-0009](../decisions/0009-multi-repository-corpora.md) — A repository holds one or more corpora, the tier above harvests pinned exports, and no merged graph exists.

## Q10 — Naming

[DR-repo-0010](../decisions/0010-naming.md) — The name is Headwater, capitalized in prose and lower case as an identifier.

## Q11 — License and distribution posture

[DR-repo-0011](../decisions/0011-license-and-distribution-posture.md) — Apache-2.0 for the engine, the library, the base package and the bundles, ratified by the owner on 2026-08-11.

## Q12 — Migration path for an existing corpus

[DR-repo-0012](../decisions/0012-migration-path-for-an-existing-corpus.md) — Adoption is a migration from no taxonomy, and `headwater infer` computes the adoption payload.

## Q13 — LinkML and SHACL as substrate

[DR-repo-0013](../decisions/0013-linkml-and-shacl-as-substrate.md) — Headwater owns the language, emitters never chain, and LinkML is the last of six siblings.

## Q14 — Discovery surface

[DR-repo-0014](../decisions/0014-discovery-surface.md) — The corpus descriptor is a generated projection at `.headwater/corpus.json`, and registration went to Q16.

## Q15 — A synthesized content tier

[DR-repo-0015](../decisions/0015-a-synthesized-content-tier.md) — Every document carries a warrant from a closed set of four, and `asserted` content is admitted with limits.

## Q16 — Public presence

[DR-repo-0016](../decisions/0016-public-presence.md) — Registration needs a channel with an obliged reader, a directory of corpora is refused, and the site is a projection of this corpus.

## Q17 — Governed access and the solution layer

[DR-repo-0017](../decisions/0017-governed-access-and-the-solution-layer.md) — The serving boundary is the export step of each publishing corpus. A profile filters for an audience, and Headwater has no principals.

## Q18 — Recording adjudicated disagreements

[DR-repo-0018](../decisions/0018-recording-adjudicated-disagreements.md) — The edge does not become a node. An adjudication is a decision document that carries `overrides`.

## Q19 — Inbound integration: an external system of record

[DR-repo-0019](../decisions/0019-inbound-integration-an-external-system-of-record.md) — Imported content is `transcribed` against a committed pin, and an imported edge carries full weight from the first release.

## Q20 — Where scent lives

[DR-repo-0020](../decisions/0020-where-scent-lives.md) — An optional source-owned cue sits on a relation instance, graded against the alternatives in view at the point of decision.

## Q21 — Terminological succession, and validity under merge

[DR-repo-0021](../decisions/0021-terminological-succession-and-validity-under-merge.md) — A retired-term lexicon sits in the language regime, and a verdict reports the read set that a merge may void.
