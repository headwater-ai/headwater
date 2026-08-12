---
id: REG-HW-open-questions
status: superseded
status_since: 2026-08-11
last_verified: 2026-08-11
summary: A redirect map from the twenty-one open-question anchors to the decision register and the obligation register that replaced them.
doc_type: decision_register
sequence: 9
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: [claude-fable-5, claude-opus-5]
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: unevidenced
relations:
  superseded_by:
    - REG-HW-decisions
    - REG-HW-open-obligations
---

# 9 — Open questions (moved)

This file is a tombstone rather than a chapter of the specification. All twenty-one questions closed. The decisions and the reasoning that settled them are now in [9 — The decision register](09-decisions.md). What each decision left open is now in [13 — Open obligations](13-open-obligations.md).

The file stays at this path because the evaluations under [`docs/evaluations/`](../evaluations/) cite these anchors, more than a hundred times between them. An evaluation is a point-in-time record, and this project does not rewrite one when a later decision changes something. So the anchors have to keep resolving, and this redirect map is how they do.

That is the project's own doctrine applied to itself. [Spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for) forbids a silent pass, and [Q17](09-decisions.md#q17--governed-access-and-the-solution-layer) rules that a filtered view announces its filtering rather than presenting as complete. A deleted file breaks every one of those citations and says nothing about it.

Each heading below carries the text that it carried before, so every anchor still resolves.

## Q1 — Implementation language

The language is **Rust**, with a WebAssembly build of the same crate for editor and browser embedding. Decision: [09-decisions.md](09-decisions.md#q1--implementation-language). Nothing stays open.

## Q2 — Schema format

YAML 1.2 is the concrete syntax, and Headwater owns the schema language, the reference sublanguage, the overlay language and the meta-schema. Decision: [09-decisions.md](09-decisions.md#q2--schema-format). Open work: [the `$`-reference grammar](13-open-obligations.md#design-work-that-nothing-blocks).

## Q3 — How much of the default taxonomy ships in the box

The base package is minimal and derived from the core, and optional content ships as add-only bundles. Decision: [09-decisions.md](09-decisions.md#q3--how-much-of-the-default-taxonomy-ships-in-the-box). Open work: [the bundle set](13-open-obligations.md#what-waits-on-a-first-adopter).

## Q4 — Relation storage

Front matter is authoritative, and a relation instance is an object rather than a pointer. Decision: [09-decisions.md](09-decisions.md#q4--relation-storage). Open work: [an unmeasured claim](13-open-obligations.md#unmeasured-claims) and [the friction signal](13-open-obligations.md#what-else-each-decision-left-open).

## Q5 — Voice checking depth

Voice checking stays lexical, with a curated pattern set, a per-category posture and a reasoned escape hatch. Decision: [09-decisions.md](09-decisions.md#q5--voice-checking-depth). Open work: [an unmeasured claim](13-open-obligations.md#unmeasured-claims).

## Q6 — Where the corpus graph lives at rest

The graph never rests. Every run rebuilds it, and no derived artifact is canonical for anything. Decision: [09-decisions.md](09-decisions.md#q6--where-the-corpus-graph-lives-at-rest). Open work: [an unmeasured claim](13-open-obligations.md#unmeasured-claims).

## Q7 — Scope of the MCP surface

Three classes of tool, and a landed write never ships. Decision: [09-decisions.md](09-decisions.md#q7--scope-of-the-mcp-surface). Open work: [the shape of a hosted server](13-open-obligations.md#what-waits-on-a-first-adopter) and [an unmeasured claim](13-open-obligations.md#unmeasured-claims).

## Q8 — Probe cost and cadence

A probe is a document with a declared expectation, and cadence follows the purpose of the run. Decision: [09-decisions.md](09-decisions.md#q8--probe-cost-and-cadence). Open work: [probe kind placement](13-open-obligations.md#what-waits-on-a-first-adopter), [three further items](13-open-obligations.md#what-else-each-decision-left-open) and [an unmeasured claim](13-open-obligations.md#unmeasured-claims).

## Q9 — Multi-repository corpora

A repository holds one or more corpora, the tier above harvests pinned exports, and no merged graph exists. Decision: [09-decisions.md](09-decisions.md#q9--multi-repository-corpora). Open work: [the vendoring question](13-open-obligations.md#what-waits-on-a-first-adopter), [tier conformance](13-open-obligations.md#what-else-each-decision-left-open) and [an unmeasured claim](13-open-obligations.md#unmeasured-claims).

### The aggregator authors its own facts

The federation layer is a corpus at a higher altitude. It authors the facts that live between repositories, and it reads exports for everything else. This subsection is now [in the decision register](09-decisions.md#the-aggregator-authors-its-own-facts).

## Q10 — Naming

The name is **Headwater**, capitalized in prose and lower case as an identifier. Decision: [09-decisions.md](09-decisions.md#q10--naming). Nothing stays open.

## Q11 — License and distribution posture

**Apache-2.0** for the engine, the library, the base package and the bundles, ratified by the owner on 2026-08-11. Decision: [09-decisions.md](09-decisions.md#q11--license-and-distribution-posture). Open work: [two items](13-open-obligations.md#what-else-each-decision-left-open) and [an unmeasured claim](13-open-obligations.md#unmeasured-claims).

## Q12 — Migration path for an existing corpus

Adoption is a migration from no taxonomy, and `headwater infer` computes the adoption payload. Decision: [09-decisions.md](09-decisions.md#q12--migration-path-for-an-existing-corpus). Open work: [an unmeasured claim](13-open-obligations.md#unmeasured-claims).

## Q13 — LinkML and SHACL as substrate

Headwater owns the language, emitters never chain, and LinkML is the last of six siblings. Decision: [09-decisions.md](09-decisions.md#q13--linkml-and-shacl-as-substrate). Open work: [a named consumer for emitters 3 through 6](13-open-obligations.md#what-waits-on-a-first-adopter) and [an unmeasured claim](13-open-obligations.md#unmeasured-claims).

## Q14 — Discovery surface

The corpus descriptor is a generated projection at `.headwater/corpus.json`, and registration went to Q16. Decision: [09-decisions.md](09-decisions.md#q14--discovery-surface). Open work: [an unmeasured claim](13-open-obligations.md#unmeasured-claims), and nothing else.

## Q15 — A synthesized content tier

Every document carries a warrant from a closed set of four, and `asserted` content is admitted with limits. Decision: [09-decisions.md](09-decisions.md#q15--a-synthesized-content-tier). Open work: [whether anything is ever promoted](13-open-obligations.md#what-else-each-decision-left-open) and [an unmeasured claim](13-open-obligations.md#unmeasured-claims).

## Q16 — Public presence

Registration needs a channel with an obliged reader, a directory of corpora is refused, and the site is a projection of this corpus. Decision: [09-decisions.md](09-decisions.md#q16--public-presence). Open work: [the documentation-site kind set](13-open-obligations.md#what-else-each-decision-left-open) and [an unmeasured claim](13-open-obligations.md#unmeasured-claims).

## Q17 — Governed access and the solution layer

The serving boundary is the export step of each publishing corpus. A profile filters for an audience, and Headwater has no principals. Decision: [09-decisions.md](09-decisions.md#q17--governed-access-and-the-solution-layer). Open work: [a second export profile and the hosted server](13-open-obligations.md#what-waits-on-a-first-adopter), [the withheld-anchor class](13-open-obligations.md#what-else-each-decision-left-open) and [an unmeasured claim](13-open-obligations.md#unmeasured-claims).

## Q18 — Recording adjudicated disagreements

The edge does not become a node. An adjudication is a decision document that carries `overrides`. Decision: [09-decisions.md](09-decisions.md#q18--recording-adjudicated-disagreements). Open work: [partial adjudication](13-open-obligations.md#what-else-each-decision-left-open) and [an unmeasured claim](13-open-obligations.md#unmeasured-claims).

## Q19 — Inbound integration: an external system of record

Imported content is `transcribed` against a committed pin, and an imported edge carries full weight from the first release. Decision: [09-decisions.md](09-decisions.md#q19--inbound-integration-an-external-system-of-record). Open work: [the transcription projection](13-open-obligations.md#what-waits-on-a-first-adopter), [snapshot size](13-open-obligations.md#what-else-each-decision-left-open) and [an unmeasured claim](13-open-obligations.md#unmeasured-claims).

## Q20 — Where scent lives

An optional source-owned cue sits on a relation instance, graded against the alternatives in view at the point of decision. Decision: [09-decisions.md](09-decisions.md#q20--where-scent-lives). Open work: [cue volume](13-open-obligations.md#what-waits-on-a-first-adopter) and [an unmeasured claim](13-open-obligations.md#unmeasured-claims).

## Q21 — Terminological succession, and validity under merge

A retired-term lexicon sits in the language regime, and a verdict reports the read set that a merge may void. Decision: [09-decisions.md](09-decisions.md#q21--terminological-succession-and-validity-under-merge). Open work: [whether another corpus needs a lexicon](13-open-obligations.md#what-waits-on-a-first-adopter), [read-set size](13-open-obligations.md#what-else-each-decision-left-open) and [an unmeasured claim](13-open-obligations.md#unmeasured-claims).
