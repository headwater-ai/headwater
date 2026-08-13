---
id: REG-HW-decisions
status: current
status_since: 2026-08-11
last_verified: 2026-08-13
summary: An index of all twenty-one design decisions, where the record of each one lives, and the evaluation that closed it.
doc_type: decision_register
sequence: 9
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  supersedes:
    - REG-HW-open-questions
  cites_evidence:
    - EVAL-HW-default-taxonomy-first-run
    - EVAL-HW-first-contact
    - EVAL-HW-graph-export-and-federation
    - EVAL-HW-language-choice
    - EVAL-HW-language-spike-results
    - EVAL-HW-linkml-worked-example
    - EVAL-HW-relation-storage
    - EVAL-HW-schema-format-walkthrough
    - EVAL-HW-shacl-worked-example
    - EVAL-HW-the-measurement-layer
    - EVAL-HW-the-serving-boundary
    - EVAL-HW-warrant-and-adjudication
    - EVAL-HW-what-a-check-can-know
---

# 9 — The decision register

This file began as a list of decisions that the design phase deferred, with the options and a leaning for each one. Every entry is now argued, and all twenty-one are closed. The last one to close was [Q11](#q11--license-and-distribution-posture). No argument from the design could close it, because a license states what the owner intends for the project. The owner ratified it on 2026-08-11.

**This file is now an index, and it stopped being the place a decision lives.** Each decision is a document on [the decisions shelf](../decisions/README.md), of the base `decision` kind, and it carries the argument that settled it. Every one of the twenty-one is `current`, with a `status_since` of 2026-08-11. The evidence for each closure lives beside it in [`docs/evaluations/`](../evaluations/).

A decision was a heading in a register. Nothing could carry its state, its dates, its acceptance or its edges. A reader who cited one cited a position in a file. Now a citation names a document with an identifier, and a check reads the document that a citation reaches.

**The headings below stay because the corpus cites them.** More than two hundred citations name a decision by the anchor of its heading here. An evaluation is a point-in-time record that this project does not rewrite. So each heading keeps the text that it carried, and each section says which document now holds the argument. This is the same instrument that [the tombstone](09-open-questions.md) is, one step later.

What the decisions leave open is in [13 — Open obligations](13-open-obligations.md). That file gathers the unmeasured claims, the work that waits on a first adopter, and one piece of design work.

A human maintains the list below by hand, and no projection writes it. [Spec 13](13-open-obligations.md#a-human-maintains-this-list-by-hand) records why. A projection kind that writes a heading for each document exists now. This file is a node of the graph, and fourteen other documents name its identifier. A generated document that declares an identity is a node of the graph. No emitter writes such a block, so a projection at this path still strands all fourteen edges.

## Q1 — Implementation language

The language is **Rust**, with a WebAssembly build of the same crate for editor and browser embedding. The record is [DR-repo-0001](../decisions/0001-implementation-language.md).

## Q2 — Schema format

YAML 1.2 is the concrete syntax, and Headwater owns the schema language, the reference sublanguage, the overlay language and the meta-schema. The record is [DR-repo-0002](../decisions/0002-schema-format.md).

## Q3 — How much of the default taxonomy ships in the box

The base package is minimal and derived from the core, and optional content ships as add-only bundles. The record is [DR-repo-0003](../decisions/0003-how-much-of-the-default-taxonomy-ships-in-the-box.md).

## Q4 — Relation storage

Front matter is authoritative, and a relation instance is an object rather than a pointer. The record is [DR-repo-0004](../decisions/0004-relation-storage.md).

## Q5 — Voice checking depth

Voice checking stays lexical, with a curated pattern set, a per-category posture and a reasoned escape hatch. The record is [DR-repo-0005](../decisions/0005-voice-checking-depth.md).

## Q6 — Where the corpus graph lives at rest

The graph never rests. Every run rebuilds it, and no derived artifact is canonical for anything. The record is [DR-repo-0006](../decisions/0006-where-the-corpus-graph-lives-at-rest.md).

## Q7 — Scope of the MCP surface

Three classes of tool, and a landed write never ships. The record is [DR-repo-0007](../decisions/0007-scope-of-the-mcp-surface.md).

## Q8 — Probe cost and cadence

A probe is a document with a declared expectation, and cadence follows the purpose of the run. The record is [DR-repo-0008](../decisions/0008-probe-cost-and-cadence.md).

## Q9 — Multi-repository corpora

A repository holds one or more corpora, the tier above harvests pinned exports, and no merged graph exists. The record is [DR-repo-0009](../decisions/0009-multi-repository-corpora.md).

### The aggregator authors its own facts

The federation layer is a corpus at a higher altitude. It authors the facts that live between repositories, and it reads exports for everything else. This heading stays because [Q17](#q17--governed-access-and-the-solution-layer) and the tombstone both cite it. The text is now [a subsection of the record](../decisions/0009-multi-repository-corpora.md#the-aggregator-authors-its-own-facts).

## Q10 — Naming

The name is **Headwater**, capitalized in prose and lower case as an identifier. The record is [DR-repo-0010](../decisions/0010-naming.md).

## Q11 — License and distribution posture

**Apache-2.0** for the engine, the library, the base package and the bundles, ratified by the owner on 2026-08-11. The record is [DR-repo-0011](../decisions/0011-license-and-distribution-posture.md).

## Q12 — Migration path for an existing corpus

Adoption is a migration from no taxonomy, and `headwater infer` computes the adoption payload. The record is [DR-repo-0012](../decisions/0012-migration-path-for-an-existing-corpus.md).

## Q13 — LinkML and SHACL as substrate

Headwater owns the language, emitters never chain, and LinkML is the last of six siblings. The record is [DR-repo-0013](../decisions/0013-linkml-and-shacl-as-substrate.md).

## Q14 — Discovery surface

The corpus descriptor is a generated projection at `.headwater/corpus.json`, and registration went to Q16. The record is [DR-repo-0014](../decisions/0014-discovery-surface.md).

## Q15 — A synthesized content tier

Every document carries a warrant from a closed set of four, and `asserted` content is admitted with limits. The record is [DR-repo-0015](../decisions/0015-a-synthesized-content-tier.md).

## Q16 — Public presence

Registration needs a channel with an obliged reader, a directory of corpora is refused, and the site is a projection of this corpus. The record is [DR-repo-0016](../decisions/0016-public-presence.md).

## Q17 — Governed access and the solution layer

The serving boundary is the export step of each publishing corpus. A profile filters for an audience, and Headwater has no principals. The record is [DR-repo-0017](../decisions/0017-governed-access-and-the-solution-layer.md).

## Q18 — Recording adjudicated disagreements

The edge does not become a node. An adjudication is a decision document that carries `overrides`. The record is [DR-repo-0018](../decisions/0018-recording-adjudicated-disagreements.md).

## Q19 — Inbound integration: an external system of record

Imported content is `transcribed` against a committed pin, and an imported edge carries full weight from the first release. The record is [DR-repo-0019](../decisions/0019-inbound-integration-an-external-system-of-record.md).

## Q20 — Where scent lives

An optional source-owned cue sits on a relation instance, graded against the alternatives in view at the point of decision. The record is [DR-repo-0020](../decisions/0020-where-scent-lives.md).

## Q21 — Terminological succession, and validity under merge

A retired-term lexicon sits in the language regime, and a verdict reports the read set that a merge may void. The record is [DR-repo-0021](../decisions/0021-terminological-succession-and-validity-under-merge.md).
