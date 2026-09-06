---
id: HW-REV-core-concepts-prompt
status: current
status_since: 2026-08-04
last_verified: 2026-08-08
summary: The instrument for the core-concepts review, with the deletion test, the two lenses, and the bar a finding must clear.
doc_type: review_prompt
title: "Review the core concepts of headwater: simplification and robustness"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: [claude-fable-5, claude-opus-5]
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: unevidenced
relations:
  applied_in:
    - HW-REV-core-concepts-findings-1
    - HW-REV-core-concepts-independent
    - HW-REV-core-concepts-findings-2
---

# Review the core concepts of headwater: simplification and robustness

You are reviewing the conceptual design of `headwater`, a documentation governance system. It is a **specification, pre-implementation** — roughly 5,000 lines of spec and nothing built. That is precisely why this review is worth doing now: concepts are free to delete today and expensive to delete once they have a schema, a CLI verb, and adopters.

The specification is written to be argued with. Treat its self-justifications as claims to test, not as settled. Several arguments in it are internally consistent and still wrong.

## Read first

- `README.md` — the three commitments
- `docs/spec/01-conceptual-model.md` — **the primary target**: the whole vocabulary
- `docs/spec/02-taxonomy-model.md` — especially "The twelve declarations", "Relation families, nuclearity, and dominance", "Deliberate limits"
- `docs/spec/00-vision-and-scope.md` — the design principles and stated non-goals
- `docs/spec/04-assurance-model.md` — obligations, controls, cohesion vs coherence
- `docs/spec/12-check-layer.md` — where checks come from

Skim 03, 05, 06, 07 for how the concepts get used. `docs/spec/09-open-questions.md` lists 17 known-open decisions (Q1–Q17) — read it so you do not report them as discoveries.

## Lens 1 — Simplification

The concept count is the thing under review. A governance system that needs a 20-word glossary before an author can file their first document will not be adopted, however correct it is.

Apply these instruments concretely, naming files and concepts:

1. **Deletion test.** For each concept in spec 1 — corpus, shelf, kind, document, external anchor, relation, sequence expectation, facet, regime (voice, lifecycle, freshness), obligation, control, register, projection — state what *specifically* breaks if it is removed. Name the check, projection, or decision that becomes impossible. If nothing concrete breaks, say so plainly: that concept is a name for something the engine would do anyway.

2. **Collapse test.** Which pairs are separated by a distinction that never changes an outcome? Test spec 2's own defences of these in particular — it argues each pair is genuinely distinct, and it may be wrong:
   - nuclearity vs dominance vs authority (three orderings over relations)
   - shelf vs kind (placement vs type)
   - facet vs a relation to a node
   - obligation vs control
   - sequence expectation vs relation cardinality
   - relation family vs nuclearity

3. **The twelve declarations.** Is twelve the right number? Which are one declaration wearing two hats, and which are genuinely orthogonal?

4. **Forced cut.** If the first release could carry only 70% of the concepts, which 30% go, in what order, and what capability is actually lost with each? Rank them.

5. **Learnability gradient.** How many concepts must someone understand to (a) file a conforming document, (b) extend a taxonomy, (c) author one from scratch? If (a) is more than three or four, say what could be deferred to advanced use.

## Lens 2 — Robustness

Robustness here means: does the model survive contact with a real corpus, a real adopter, and an author who does not want to comply?

1. **Edge behaviour.** For each core concept, what is defined at its limits — empty, singleton, cyclic, self-referential, duplicated, missing, mutually contradictory? Find where behaviour is simply undefined. Undefined edges become implementation accidents that later become the spec.

2. **The adversarial author.** Someone who wants the check green without doing the work. Where does the design reward gaming? Spec 4 already catches one case (link-padding to satisfy the focus-shift metric). Find the others — suppressions, escape hatches, waivers, status transitions, summary text, relation choice.

3. **Scale.** What changes at 10 documents, 500, 10,000? At 1 taxonomy, 5, 50 in a federation? Name the concepts whose cost or usefulness is non-linear.

4. **Partial and mid-flight states.** A half-conformant corpus during migration. A taxonomy version landing while documents sit in review. Overlays resolving differently after a pin bump. Identifiers moving. Which concepts have no defined behaviour while the corpus is *between* valid states?

5. **Load-bearing assumptions.** What does the design assume that nobody has measured, and what happens if each is false? Start with: that agent-assisted authoring lowers capture cost; that `summary` carries enough scent for routing; that false-positive rates stay low enough for advisory→blocking promotion.

6. **Single points of failure.** Where does one component's correctness silently determine everything downstream? Kind resolution and the graph projector are two candidates. Find the rest.

## Do not

- Do not edit or restructure prose, fix wording, or comment on style.
- Do not propose new features, new concepts, or new checks. This review only removes, merges, hardens, and defines.
- Do not summarise the specs back to me. I wrote them.
- Do not report Q1–Q17 as findings. *Do* say if one is mis-framed, or if something the specs treat as settled should be reopened.
- Do not be diplomatic at the cost of being useful. A concept you would cut is more valuable to me than three you would keep.

## Output

Start with a table: one row per core concept, verdict in `keep / merge into X / cut / define edges`, and a one-line reason.

Then the arguments, ordered by consequence — the ones that change the design first. Every claim must be falsifiable and anchored: name the file, the concept, and the concrete thing that breaks or fails to break. "This feels complex" is not a finding; "`dominance` and `authority` never disagree because X, so one is redundant" is.

Close with exactly two things:

1. The single change that most reduces concept count without losing a capability.
2. The single thing most likely to break first in a real adoption.

State your confidence on each major claim, and flag explicitly where you are inferring rather than reading.
