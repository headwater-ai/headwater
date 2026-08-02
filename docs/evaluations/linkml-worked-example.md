# The docgov taxonomy in LinkML — a worked example

Evidence for [Q13](../spec/09-open-questions.md#q13--linkml-and-shacl-as-substrate).
[Spec 11](../spec/11-adjacent-work.md#c-linkml--the-uncomfortable-one) claimed LinkML
covers the "structural half" of [spec 2](../spec/02-taxonomy-model.md) and none of the
"governance half". Writing it out shows that framing was wrong — and the real boundary
is more useful than the one I guessed.

## The schema

A meaningful subset of the default taxonomy: three kinds, the core facets, four
relations, one heterogeneous shelf, identifiers, and profiles.

```yaml
id: https://docgov.dev/taxonomy/standard
name: docgov_standard
title: docgov standard taxonomy
description: >-
  The default documentation taxonomy, expressed in LinkML. Governance semantics
  that LinkML has no metaslot for are carried in `annotations` under the docgov
  prefix; see the analysis for which of those it can and cannot act on.
license: https://creativecommons.org/publicdomain/zero/1.0/

prefixes:
  linkml: https://w3id.org/linkml/
  docgov: https://docgov.dev/taxonomy/
  prov: http://www.w3.org/ns/prov#
  skos: http://www.w3.org/2004/02/skos/core#
default_prefix: docgov
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
          docgov:role: initial
      current:
        description: Authoritative and believed true.
        annotations:
          docgov:role: live
      superseded:
        description: Replaced by a successor; retained for lineage.
        annotations:
          docgov:role: terminal-retained
      deprecated:
        description: No longer applicable; retained.
        annotations:
          docgov:role: terminal-retained

  EvidenceBasis:
    description: How the rationale in a document is grounded.
    permissible_values:
      evidenced:      {description: An external auditable artefact supports this.}
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
      syntax: "DR-{namespace}-{seq}"
      interpolated: true

  status:
    description: Lifecycle state.
    range: LifecycleState
    required: true
    annotations:
      docgov:role: state

  last_verified:
    description: The date a human last confirmed this document is true.
    range: date
    required: true
    annotations:
      docgov:role: freshness
      docgov:stale_after_days: 180

  summary:
    description: One sentence. The corpus's scent surface — routing and indexes show this.
    required: true
    annotations:
      docgov:role: scent

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
      docgov:family: succession
      docgov:nuclearity: multinuclear
      docgov:dominance: source
      docgov:inverse: superseded_by
      docgov:reciprocal: required
      docgov:created_by: scaffold

  superseded_by:
    range: Decision
    slot_uri: prov:wasRevisionOf

  conflicts_with:
    description: This decision contradicts the target.
    range: Decision
    multivalued: true
    annotations:
      docgov:family: association
      docgov:reciprocal: symmetric
      docgov:invalid_when: both_current

  derives_from:
    description: A generated artefact and the canonical source it projects.
    range: Standard
    slot_uri: prov:wasDerivedFrom
    annotations:
      docgov:family: derivation
      docgov:nuclearity: nucleus-satellite
      docgov:nucleus: target
      docgov:inherits: "status, last_verified"
      docgov:created_by: generator

  governs:
    description: Source paths this document is authoritative for.
    multivalued: true
    annotations:
      docgov:family: governance
      docgov:dominance: source
      docgov:created_by: author

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
    class_uri: docgov:Decision
    exact_mappings:  [adr:ArchitectureDecisionRecord]   # SKOS mapping, natively
    close_mappings:  [platform:DesignRecord]
    slots: [supersedes, superseded_by, conflicts_with, evidence_basis]
    annotations:
      docgov:purpose: rationale
      docgov:voice: declarative
      docgov:lifecycle: standard
      docgov:authority: 20
      docgov:sections_required: "Context, Decision, Consequences"
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
      docgov:purpose: behaviour
      docgov:voice: declarative
      docgov:authority: 10
      docgov:contracts_allowed: "true"

  Standard:
    is_a: Document
    description: A prescriptive rule set on the governance shelf.
    slots: [doc_type, governs]
    in_subset: [service_repo, docs_only]
    annotations:
      docgov:purpose: constraint
      docgov:authority: 15

# --------------------------------------------------------------------- profiles
subsets:
  service_repo:
    description: A repository that authors its own specs and standards.
  docs_only:
    description: A documentation-only repository.
```

## What LinkML does natively, and does well

Four of these were genuinely surprising — not "can be encoded" but "is the same concept, already named".

| docgov concept | LinkML | Note |
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

Three of the twenty research-derived changes (SKOS mappings, PROV alignment, advisory
severity) are things LinkML already ships. That is a real argument for adoption, and
also mild evidence the modelling instincts in spec 2 were conventional rather than
eccentric.

## Where it stops — and the boundary is not the one I claimed

Spec 11 said the split was **structural versus governance**. It is not. Look at what
actually fails:

- **Reciprocity.** `supersedes` requires the target to link back. LinkML cannot say
  this. Neither can SHACL without dropping to SPARQL.
- **`conflicts_with` invalid when both endpoints are current.** The rule above
  enforces the one constraint whose slots live on a single instance. The constraint
  we actually want reads *the target's* `status` — and per-instance validation does
  not see it.
- **Satellite inheritance.** "A satellite's freshness follows its nucleus" is a
  statement about a pair, resolved by traversal.
- **Sequence expectations.** "A current decision acquires an implementing
  specification within 90 days" is a query over the graph *and over time*.
- **Overlays, core, compatibility.** Operations on the schema itself, not statements
  in it.

None of that is "governance" as opposed to "structure". Reciprocity is as structural
as anything in spec 2. The actual line is:

> **LinkML, SHACL, and JSON Schema all validate one instance against a shape.
> Everything docgov does that they cannot is a property of the graph as a whole, or
> of the corpus over time.**

That reframing matters, because it turns Q13 from *"does LinkML cover enough?"* into
a better question: **is a two-layer architecture — a standard shape layer plus a
docgov graph layer — better than one custom layer?** Every mature validation stack in
this space has that shape. It is not a compromise; it is the normal answer.

## The annotations problem

Everything docgov-specific above sits in `annotations`, and annotations are untyped
pass-through. LinkML carries them and does nothing with them: no validation, no
generator output, no error when `docgov:nuclearity` is misspelled or set to a value
that does not exist.

So for precisely the half that is ours, the meta-schema benefit — the main reason to
adopt LinkML — evaporates. We would still write a validator for the annotation
vocabulary, and authors would face two languages in one file with no visual
distinction between the half that is checked and the half that is not. That scores
badly on role-expressiveness and error-proneness, which is exactly what the
cognitive-dimensions walkthrough in [Q2](../spec/09-open-questions.md#q2--schema-format)
is meant to catch.

Writing it out is what made this concrete. It reads fine until you notice that a
third of the semantics is inert.

## The option this exercise surfaced

Neither "adopt LinkML" nor "stay independent" is right. A third option:

> **Author in docgov's language; emit LinkML as a compilation target.**

The resolved taxonomy compiles to a LinkML schema covering the shape layer — which
then compiles onward to JSON Schema, SHACL, OWL, and Pydantic through LinkML's own
generators. The graph and temporal layers stay in the docgov engine, where they were
always going to live.

That gets the interoperability without the two-languages problem: one authoring
surface, fully validated, with a standards-based export that other tooling can
consume. It also inverts the risk. Adopting LinkML as the authoring surface is close
to irreversible; emitting it is a generator we can add, change, or drop.

Worth noting how this rhymes with the position already taken on distribution: a
resolved artefact, emitted, not authored.

## Recommendation

Take **option 3** into the Q2 walkthrough as the leading candidate, with these
consequences to weigh:

- it removes the Q1 tension entirely — a Rust core emitting LinkML YAML has no
  dependency on LinkML's Python tooling;
- the SHACL question resolves itself: SHACL becomes an output artefact for external
  consumers, so its poor error messages never reach a docgov author;
- the cost is a generator plus fidelity tests proving the emitted schema accepts
  exactly the documents docgov accepts, which is a real and ongoing cost;
- and the shape/graph boundary needs to be stated in [spec 6](../spec/06-engine-architecture.md)
  as an architectural seam, because it is one.
