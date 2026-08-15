---
id: HW-EVAL-linkml-worked-example
status: current
status_since: 2026-08-02
last_verified: 2026-08-11
summary: The Headwater taxonomy written out in LinkML, and the boundary where the standard stops covering what spec 2 declares.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: [claude-fable-5, claude-opus-5]
  activity: evaluate+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cited_by:
    - HW-REG-decisions
    - HW-SPEC-adjacent-work
---

# The Headwater taxonomy in LinkML — a worked example

Evidence for [Q13](../spec/09-open-questions.md#q13--linkml-and-shacl-as-substrate). [Spec 11](../spec/11-adjacent-work.md#c-linkml--the-uncomfortable-one) claimed LinkML covers the "structural half" of [spec 2](../spec/02-taxonomy-model.md) and none of the "governance half". Writing it out shows that framing was wrong — and the real boundary is more useful than the one I guessed.

## The schema

A meaningful subset of the default taxonomy: three kinds, the core facets, four relations, one heterogeneous shelf, identifiers, and profiles.

```yaml
id: https://w3id.org/headwater/taxonomy/standard
name: headwater_standard
title: Headwater standard taxonomy
description: >-
  The default documentation taxonomy, expressed in LinkML. Governance semantics
  that LinkML has no metaslot for are carried in `annotations` under the headwater
  prefix; see the analysis for which of those it can and cannot act on.
license: https://creativecommons.org/publicdomain/zero/1.0/

prefixes:
  linkml: https://w3id.org/linkml/
  headwater: https://w3id.org/headwater/taxonomy/
  prov: http://www.w3.org/ns/prov#
  skos: http://www.w3.org/2004/02/skos/core#
default_prefix: headwater
default_range: string

imports:
  - linkml:types

settings:
  namespace: "[A-Z]{2,6}"
  seq: "\\d{4}"

# ---------------------------------------------------------------- vocabularies
enums:

  LifecycleState:
    description: Where a document sits in its lifecycle.
    permissible_values:
      draft:
        description: Being written; not yet authoritative.
        annotations:
          headwater:role: initial
      current:
        description: Authoritative and believed true.
        annotations:
          headwater:role: live
      superseded:
        description: Replaced by a successor; retained for lineage.
        annotations:
          headwater:role: terminal-retained
      deprecated:
        description: No longer applicable; retained.
        annotations:
          headwater:role: terminal-retained

  EvidenceBasis:
    description: How the rationale in a document is grounded.
    permissible_values:
      evidenced:      {description: An external auditable artifact supports this.}
      reconstructed:  {description: Written after the fact; basis must be stated.}
      gap:            {description: No evidence exists and none is claimed.}

  GovernanceDocType:
    description: Discriminator for the heterogeneous governance shelf.
    permissible_values:
      standard:    {description: A prescriptive rule set.}
      methodology: {description: A repeatable procedure for producing something.}
      runbook:     {description: Steps to carry out an operational task.}
      register:    {description: A list maintained as the record of something.}

# ----------------------------------------------------------------------- facets
slots:

  id:
    description: Stable, globally unique, resolvable without its document.
    identifier: true
    structured_pattern:
      syntax: "{namespace}-DR-{seq}"
      interpolated: true

  status:
    description: Lifecycle state.
    range: LifecycleState
    required: true
    annotations:
      headwater:role: state

  last_verified:
    description: The date a human last confirmed this document is true.
    range: date
    required: true
    annotations:
      headwater:role: freshness
      headwater:stale_after_days: 180

  summary:
    description: One sentence. The corpus's scent surface — routing and indexes show this.
    required: true
    annotations:
      headwater:role: scent

  audience:
    description: Who this is written for.
    recommended: true          # advisory severity, natively
    multivalued: true

  doc_type:
    description: Which kind this is, on a shelf that holds several.
    range: GovernanceDocType
    designates_type: true      # LinkML's own discriminator concept

  evidence_basis:
    range: EvidenceBasis
    required: true

# -------------------------------------------------------------------- relations
  supersedes:
    description: This decision replaces the target.
    range: Decision
    multivalued: true
    slot_uri: prov:wasRevisionOf
    annotations:
      headwater:family: succession
      headwater:nuclearity: multinuclear
      headwater:dominance: source
      headwater:inverse: superseded_by
      headwater:reciprocal: required
      headwater:created_by: scaffold

  superseded_by:
    range: Decision
    slot_uri: prov:wasRevisionOf

  conflicts_with:
    description: This decision contradicts the target.
    range: Decision
    multivalued: true
    annotations:
      headwater:family: association
      headwater:reciprocal: symmetric
      headwater:invalid_when: both_current

  derives_from:
    description: A generated artifact and the canonical source it projects.
    range: Standard
    slot_uri: prov:wasDerivedFrom
    annotations:
      headwater:family: derivation
      headwater:nuclearity: nucleus-satellite
      headwater:nucleus: target
      headwater:inherits: "status, last_verified"
      headwater:created_by: generator

  governs:
    description: Source paths this document is authoritative for.
    multivalued: true
    annotations:
      headwater:family: governance
      headwater:dominance: source
      headwater:created_by: author

# ------------------------------------------------------------------------ kinds
classes:

  Document:
    abstract: true
    description: Anything in the corpus.
    slots: [id, status, last_verified, summary, audience]
    unique_keys:
      primary:
        unique_key_slots: [id]

  Decision:
    is_a: Document
    description: Why a choice was made and what it forecloses.
    class_uri: headwater:Decision
    exact_mappings:  [adr:ArchitectureDecisionRecord]   # SKOS mapping, natively
    close_mappings:  [platform:DesignRecord]
    slots: [supersedes, superseded_by, conflicts_with, evidence_basis]
    annotations:
      headwater:purpose: rationale
      headwater:voice: declarative
      headwater:lifecycle: standard
      headwater:authority: 20
      headwater:sections_required: "Context, Decision, Consequences"
    rules:
      - description: >-
          A superseded decision must name its successor. This one LinkML can
          enforce, because both slots belong to the same instance.
        preconditions:
          slot_conditions:
            status: {equals_string: superseded}
        postconditions:
          slot_conditions:
            superseded_by: {value_presence: PRESENT}

  Specification:
    is_a: Document
    description: What a component does, as it is now.
    slots: [governs]
    annotations:
      headwater:purpose: behavior
      headwater:voice: declarative
      headwater:authority: 10
      headwater:contracts_allowed: "true"

  Standard:
    is_a: Document
    description: A prescriptive rule set on the governance shelf.
    slots: [doc_type, governs]
    in_subset: [service_repo, docs_only]
    annotations:
      headwater:purpose: constraint
      headwater:authority: 15

# --------------------------------------------------------------------- profiles
subsets:
  service_repo:
    description: A repository that authors its own specs and standards.
  docs_only:
    description: A documentation-only repository.
```

## What LinkML does natively, and does well

Four of these were genuinely surprising — not "can be encoded" but "is the same concept, already named".

| Headwater concept | LinkML | Note |
|---|---|---|
| Kind | `classes` with `is_a`, `abstract` | Direct |
| Facet | `slots` with `range`, `required`, `pattern` | Direct |
| Controlled vocabulary | `enums` + `permissible_values` | Plus `meaning:` for ontology grounding, which we did not have |
| Advisory facet severity | **`recommended: true`** | A native metaslot for exactly our warn-level |
| Heterogeneous-shelf discriminator | **`designates_type: true`** | The identical concept, independently arrived at |
| Cross-taxonomy mapping | **`exact_mappings`, `close_mappings`, `broad_mappings`…** | SKOS mappings on every element, natively — change #9, for free |
| Identifier scheme | `identifier: true` + `structured_pattern` + `settings` | Better than our sketch: the pattern is composable |
| Profile | `subsets` + `in_subset` | Direct |
| Lineage semantics | `slot_uri: prov:wasRevisionOf` | PROV alignment, natively — change #11, for free |
| Relation endpoints and cardinality | `range` to a class, `multivalued` | Direct |

Three of the twenty research-derived changes (SKOS mappings, PROV alignment, advisory severity) are things LinkML already ships. That is a real argument for adoption, and also mild evidence the modeling instincts in spec 2 were conventional rather than eccentric.

## Where it stops — and the boundary is not the one I claimed

Spec 11 said the split was **structural versus governance**. It is not. Look at what actually fails:

- **Reciprocity.** `supersedes` requires the target to link back. LinkML cannot say this. Neither can SHACL without dropping to SPARQL.
- **`conflicts_with` invalid when both endpoints are current.** The rule above enforces the one constraint whose slots live on a single instance. The constraint we actually want reads *the target's* `status` — and per-instance validation does not see it.
- **Satellite inheritance.** "A satellite's freshness follows its nucleus" is a statement about a pair, resolved by traversal.
- **Sequence expectations.** "A current decision acquires an implementing specification within 90 days" is a query over the graph *and over time*.
- **Overlays, core, compatibility.** Operations on the schema itself, not statements in it.

None of that is "governance" as opposed to "structure". Reciprocity is as structural as anything in spec 2. The actual line is:

> **LinkML, SHACL, and JSON Schema all validate one instance against a shape. Everything Headwater does that they cannot is a property of the graph as a whole, or of the corpus over time.**

That reframing matters, because it turns Q13 from *"does LinkML cover enough?"* into a better question: **is a two-layer architecture — a standard shape layer plus a Headwater graph layer — better than one custom layer?** Every mature validation stack in this space has that shape. It is not a compromise; it is the normal answer.

## The annotations problem

Everything Headwater-specific above sits in `annotations`, and annotations are untyped pass-through. LinkML carries them and does nothing with them: no validation, no generator output, no error when `headwater:nuclearity` is misspelled or set to a value that does not exist.

So for precisely the half that is ours, the meta-schema benefit — the main reason to adopt LinkML — evaporates. We would still write a validator for the annotation vocabulary, and authors would face two languages in one file with no visual distinction between the half that is checked and the half that is not. That scores badly on role-expressiveness and error-proneness, which is exactly what the cognitive-dimensions walkthrough in [Q2](../spec/09-open-questions.md#q2--schema-format) is meant to catch.

Writing it out is what made this concrete. It reads fine until you notice that a third of the semantics is inert.

## The option this exercise surfaced

Neither "adopt LinkML" nor "stay independent" is right. A third option:

> **Author in Headwater's language; emit LinkML as a compilation target.**

The resolved taxonomy compiles to a LinkML schema covering the shape layer — which then compiles onward to JSON Schema, SHACL, OWL, and Pydantic through LinkML's own generators. The graph and temporal layers stay in the Headwater engine, where they were always going to live.

That gets the interoperability without the two-languages problem: one authoring surface, fully validated, with a standards-based export that other tooling can consume. It also inverts the risk. Adopting LinkML as the authoring surface is close to irreversible; emitting it is a generator we can add, change, or drop.

Worth noting how this rhymes with the position already taken on distribution: a resolved artifact, emitted, not authored.

## Recommendation

Take **option 3** into the Q2 walkthrough as the leading candidate, with these consequences to weigh:

- it removes the Q1 tension entirely — a Rust core emitting LinkML YAML has no dependency on LinkML's Python tooling;
- the SHACL question resolves itself: SHACL becomes an output artifact for external consumers, so its poor error messages never reach a Headwater author;
- the cost is a generator plus fidelity tests proving the emitted schema accepts exactly the documents Headwater accepts, which is a real and ongoing cost;
- and the shape/graph boundary needs to be stated in [spec 6](../spec/06-engine-architecture.md) as an architectural seam, because it is one.
