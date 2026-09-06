---
id: HW-REV-core-concepts-assessment
status: current
status_since: 2026-08-05
last_verified: 2026-08-08
summary: An assessment of the first findings report, made against a later revision, with the places where it would revise them.
doc_type: review_record
title: "Assessment of the existing core-concepts findings"
provenance:
  warrant: accepted
  agency: agent
  activity: review+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  assesses:
    - HW-REV-core-concepts-findings-1
---

# Assessment of the existing core-concepts findings

Reviewed 2026-08-05 after completing [core-concepts-independent-review.md](core-concepts-independent-review.md). The existing report was produced against commit `6057872`; this assessment used revision `b7ca526`.

## Overall assessment

The existing report is strong and substantially correct. Its most consequential findings are the missing temporal inputs and the authority-rank contradiction. Both are falsifiable implementation blockers rather than preferences about vocabulary.

I agree with its conclusions on:

- cutting authority rank;
- merging sequence expectations into relation participation while defining the window origin;
- rejecting reference-valued facets as an ungoverned second edge mechanism;
- defining lifecycle-history inputs;
- defining relation cycles, self-reference, inheritance conflicts, and anchor semantics;
- reducing the declaration count;
- treating partial migration as a core validity problem;
- kind, lifecycle, relation-choice, suppression, and waiver gaming;
- the overlay resolver, scope enforcer, census, parser, and scaffolder as correctness roots;
- human-maintained relation capture as the leading adoption risk.

I found no current specification text that resolves those issues.

## Where I would revise the findings

### Relation family should not be the sole classifier

The report assumes family determines endpoint nuclearity and dominance. Relation direction remains taxonomy-defined, however: `derives_from` and its inverse share a family but reverse the likely nucleus; `part_of` and `comprises` do likewise.

Family can supply defaults only when direction conventions make them sound. Cut authority, derive dominance where succession or nuclearity already determines it, and audit overrides. Do not make family silently carry endpoint information it does not contain. (Confidence: medium-high.)

### Dominance does not logically equal nuclearity

Whether a document can stand alone and which document should control routing, report targeting, or index order are different questions. The examples align, which supports deriving or deleting dominance in those cases, but it is not a universal logical entailment. If no counterexample produces a different concrete outcome, dominance should be cut rather than justified by an overbroad identity claim. (Confidence: medium-high.)

### Simplification must preserve visible assurance and context safety

The phrases "merge the register", "merge freshness regime", and "merge size regime" can be read as weakening them. That is not the intended design change.

- The **register remains** a mandatory generated and checked view of obligations, controls, dispositions, coverage, control health, suppressions, and waivers. Only its status as a third independently authored object disappears.
- **Freshness remains** an engine-significant facet role with mandatory applicable policy, drift weighting, staleness findings, and trend reporting. The policy sits at the facet or is referenced there; it cannot silently disappear inside a kind.
- **Size remains** a mandatory enforced budget for each agent-facing kind or projection, with findings and trends surfaced in assurance. A projection without an applicable budget is invalid.

This placement reduces vocabulary while making the protections closer to the outputs they govern. It must not turn them into optional anonymous fields. (Confidence: high.)

### External-anchor failure modes need more weight

The existing report notes that document-oriented relation semantics do not fit anchors. It should also cover missing resolvers, multiple resolvers claiming the same anchor type, normalisation collisions, duplicated anchors, and inconsistent identity across plugins. Write-time impact detection depends on these identities; anchor resolution is therefore a correctness root, not merely an endpoint edge case. (Confidence: high.)

### Maintenance machinery should be treated as required infrastructure

The existing closing recommendation says default relations should be creatable by scaffold, generator, or hook. That is right but incomplete. A rich, taxonomy-defined information architecture also needs machinery capable of applying it consistently:

- a specialised IA skill for explainable shelf and kind resolution;
- authoring and review skills for relations, evidence, summaries, lifecycle, and section contracts;
- harness hooks that invoke the applicable machinery at intent, write, and review time;
- human acceptance for judgement-bearing outputs;
- attribution and assisted-fraction telemetry so the assurance model can detect when the machinery is not maintaining the graph;
- freshness and size checks before generated governance context reaches an agent.

These use concepts already specified in docs 3, 4, 5, 6, and 12. They do not require a new taxonomy concept. They change the delivery conclusion: scaffolding and agent integration are not conveniences that can safely follow the core validator; they are necessary to test the capture-cost thesis and prevent stale governance context from becoming an efficient source of wrong agent behaviour. (Confidence: high on the dependency; medium-high on the adoption prediction.)

## Resulting disposition

The existing findings should remain as an independent review record, not be edited to manufacture agreement. The companion independent review adopts most of its findings, narrows the relation-family claim, strengthens anchor robustness, makes the preservation of register/freshness/size protections explicit, and elevates specialised skills and harness hooks to required maintenance machinery.
