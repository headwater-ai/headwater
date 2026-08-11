# Headwater

A **documentation governance system**: a typed, validated, self-describing corpus of documentation that machines can check and agents can navigate — where the taxonomy itself is configuration, not code.

> **Status: specification, pre-implementation.** Nothing here is built yet. This repository currently holds a high-level specification written to be argued with. The next phase drills into each area before any implementation begins.

## What problem this solves

Documentation rots because nothing holds it accountable. Specs drift from code, rationale evaporates, standards multiply and contradict each other, and the AI assistants now reading that documentation as context inherit every one of those faults — silently, and at scale.

The usual answers are a style guide (unenforced), a wiki (unstructured), or a static-site generator (renders whatever you feed it). None of them can answer *"is this corpus still true?"*, because none of them know what kind of document anything is.

Headwater's premise: **make the structure of the corpus a machine-readable contract**, then derive everything else from it — validation, navigation, templates, AI instruction context, publishing, and the evidence that the whole thing is working.

## The three commitments

1. **Taxonomy is data.** What shelves exist, what document kinds live on them, what metadata they carry, what voice and lifecycle they obey, and how they may reference each other — all declared in one versioned schema. Two organizations with different documentation cultures run the same engine over different taxonomies. Customizing the taxonomy never means forking the tooling.

2. **The corpus is a graph.** Documents are typed nodes; front-matter references are typed edges. Every validation rule is a constraint on that graph, every query is a traversal of it, and every derived artifact (indexes, site navigation, AI rules, agent context) is a projection of it.

3. **AI assistants are readers in their own right, and measured ones.** The corpus feeds agents at intent time, read time, write time, and review time — and the system probes whether that context actually changes agent behavior, rather than assuming it does.

## Read the specification

| # | Document | What it settles |
|---|---|---|
| 0 | [Vision and scope](docs/spec/00-vision-and-scope.md) | What we are building, for whom, and explicitly not building |
| 1 | [Conceptual model](docs/spec/01-conceptual-model.md) | The vocabulary — corpus, shelf, kind, facet, relation, obligation |
| 2 | [Taxonomy model](docs/spec/02-taxonomy-model.md) | **The core change**: taxonomy as a composable, validated schema |
| 3 | [Authoring and lifecycle](docs/spec/03-authoring-and-lifecycle.md) | States, front matter, voice regimes, templates, identifiers |
| 4 | [Assurance model](docs/spec/04-assurance-model.md) | Invariants, controls, the evidence register, defense in depth |
| 5 | [AI integration](docs/spec/05-ai-integration.md) | Routing, rules, hooks, agents, the corpus MCP surface, efficacy |
| 6 | [Engine architecture](docs/spec/06-engine-architecture.md) | One parse, one graph, pluggable checks; CLI and library shape |
| 7 | [Distribution and federation](docs/spec/07-distribution-and-federation.md) | Publishing a taxonomy, consuming one, overlays, pins, drift |
| 8 | [Design departures](docs/spec/08-design-departures.md) | The recurrent failure modes of governance tooling, and what we do instead |
| 9 | [Open questions](docs/spec/09-open-questions.md) | Every decision the design phase deferred, and the argument that settled it. 20 of 21 are closed |
| 10 | [Theoretical foundations](docs/spec/10-theoretical-foundations.md) | The research the design rests on, and the 20 changes it forced — all applied |
| 11 | [Adjacent work](docs/spec/11-adjacent-work.md) | Existing tools and patterns that overlap — what to adopt, and what we already specified twice |
| 12 | [The check layer](docs/spec/12-check-layer.md) | Where checks come from, how scope makes caching and change-scoping sound |

The [glossary](docs/spec/glossary.md) lists every named concept in one place, with a link to the section that defines it. It also marks the eight terms that an author needs to file a document, which is the whole of the vocabulary that most readers meet.

Evidence gathered for specific open questions lives in [`docs/evaluations/`](docs/evaluations/). Worked examples of the taxonomy in [LinkML](docs/evaluations/linkml-worked-example.md) and the checks in [SHACL](docs/evaluations/shacl-worked-example.md) supplied the substance for Q13. A [language evaluation](docs/evaluations/language-choice.md) and the [spike results](docs/evaluations/language-spike-results.md) closed Q1, a [cognitive-dimensions walkthrough](docs/evaluations/schema-format-walkthrough.md) closed Q2, a [first-run walkthrough](docs/evaluations/default-taxonomy-first-run.md) closed Q3, and a [relation-storage evaluation](docs/evaluations/relation-storage.md) closed Q4. Six later evaluations closed the rest in groups: [graph export and federation](docs/evaluations/graph-export-and-federation.md) for Q6, Q9 and Q13, [the serving boundary](docs/evaluations/the-serving-boundary.md) for Q7, Q14 and Q17, [warrant and adjudication](docs/evaluations/warrant-and-adjudication.md) for Q15, Q18 and Q19, [what a check can know](docs/evaluations/what-a-check-can-know.md) for Q5 and Q21, [the measurement layer](docs/evaluations/the-measurement-layer.md) for Q8 and Q20, and [first contact](docs/evaluations/first-contact.md) for Q11, Q12 and Q16.

Design reviews and the prompts that commission them live in [`docs/reviews/`](docs/reviews/) — currently a [core-concepts review](docs/reviews/core-concepts-review-prompt.md) aimed at simplification and robustness. The prompts are committed alongside their findings on purpose: a review whose instrument is unrecorded cannot be repeated against a later draft, and comparing two runs of the same instrument is the only way to tell whether the design improved or the reviewer changed.

Start at 0 and 2 if you only read two. Spec 10 is where the design gets tested against the literature, and all twenty-one changes it called for are applied.

## License

Code is licensed under the Apache License, Version 2.0. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).

The prose under `docs/` is licensed under Creative Commons Attribution 4.0 International. See [`docs/LICENSE`](docs/LICENSE). Code samples inside that prose stay under Apache-2.0, so copying an example into your own project carries no attribution obligation.

Contributions arrive under the [Developer Certificate of Origin](https://developercertificate.org/), with a `Signed-off-by` line and no contributor license agreement. [`CONTRIBUTING.md`](CONTRIBUTING.md) states why. [`SECURITY.md`](SECURITY.md) carries the disclosure process, and [`TRADEMARKS.md`](TRADEMARKS.md) states the trademark position, which is that there is no registered mark.

[Q11](docs/spec/09-open-questions.md#q11--license-and-distribution-posture) records the reasoning, and the [first-contact evaluation](docs/evaluations/first-contact.md) carries the full argument. The design narrowed the field and did not choose within it: internal-only and source-available terms are refused by rulings the specification already made, and network copyleft is refused for the library that spec 6 requires. The choice among the terms that remained belongs to the owner, who made it on 2026-08-11.
