# docgov

A **documentation governance system**: a typed, validated, self-describing corpus of documentation that machines can check and agents can navigate — where the taxonomy itself is configuration, not code.

> **Status: specification, pre-implementation.** Nothing here is built yet. This repository currently holds a high-level specification written to be argued with. The next phase drills into each area before any implementation begins.

## What problem this solves

Documentation rots because nothing holds it accountable. Specs drift from code, rationale evaporates, standards multiply and contradict each other, and the AI assistants now reading that documentation as context inherit every one of those faults — silently, and at scale.

The usual answers are a style guide (unenforced), a wiki (unstructured), or a static-site generator (renders whatever you feed it). None of them can answer *"is this corpus still true?"*, because none of them know what kind of document anything is.

docgov's premise: **make the structure of the corpus a machine-readable contract**, then derive everything else from it — validation, navigation, templates, AI instruction context, publishing, and the evidence that the whole thing is working.

## The three commitments

1. **Taxonomy is data.** What shelves exist, what document kinds live on them, what metadata they carry, what voice and lifecycle they obey, and how they may reference each other — all declared in one versioned schema. Two organisations with different documentation cultures run the same engine over different taxonomies. Customising the taxonomy never means forking the tooling.

2. **The corpus is a graph.** Documents are typed nodes; front-matter references are typed edges. Every validation rule is a constraint on that graph, every query is a traversal of it, and every derived artefact (indexes, site navigation, AI rules, agent context) is a projection of it.

3. **AI assistants are readers in their own right, and measured ones.** The corpus feeds agents at intent time, read time, write time, and review time — and the system probes whether that context actually changes agent behaviour, rather than assuming it does.

## Read the specification

| # | Document | What it settles |
|---|---|---|
| 0 | [Vision and scope](docs/spec/00-vision-and-scope.md) | What we are building, for whom, and explicitly not building |
| 1 | [Conceptual model](docs/spec/01-conceptual-model.md) | The vocabulary — corpus, shelf, kind, facet, relation, obligation |
| 2 | [Taxonomy model](docs/spec/02-taxonomy-model.md) | **The core change**: taxonomy as a composable, validated schema |
| 3 | [Authoring and lifecycle](docs/spec/03-authoring-and-lifecycle.md) | States, front matter, voice regimes, templates, identifiers |
| 4 | [Assurance model](docs/spec/04-assurance-model.md) | Invariants, controls, the evidence register, defence in depth |
| 5 | [AI integration](docs/spec/05-ai-integration.md) | Routing, rules, hooks, agents, the corpus MCP surface, efficacy |
| 6 | [Engine architecture](docs/spec/06-engine-architecture.md) | One parse, one graph, pluggable checks; CLI and library shape |
| 7 | [Distribution and federation](docs/spec/07-distribution-and-federation.md) | Publishing a taxonomy, consuming one, overlays, pins, drift |
| 8 | [Prior art and departures](docs/spec/08-prior-art-and-departures.md) | What the reference system got right, and what we change |
| 9 | [Open questions](docs/spec/09-open-questions.md) | Decisions deliberately deferred to the design phase |
| 10 | [Theoretical foundations](docs/spec/10-theoretical-foundations.md) | The research the design rests on, and the 20 changes it forced — all applied |
| 11 | [Adjacent work](docs/spec/11-adjacent-work.md) | Existing tools and patterns that overlap — what to adopt, and what we already specified twice |
| 12 | [The check layer](docs/spec/12-check-layer.md) | Where checks come from, how scope makes caching and change-scoping sound |

Evidence gathered for specific open questions lives in [`docs/evaluations/`](docs/evaluations/) — currently worked examples of the taxonomy in [LinkML](docs/evaluations/linkml-worked-example.md) and the checks in [SHACL](docs/evaluations/shacl-worked-example.md), both feeding Q13.

Design reviews and the prompts that commission them live in [`docs/reviews/`](docs/reviews/) — currently a [core-concepts review](docs/reviews/core-concepts-review-prompt.md) aimed at simplification and robustness. The prompts are committed alongside their findings on purpose: a review whose instrument is unrecorded cannot be repeated against a later draft, and comparing two runs of the same instrument is the only way to tell whether the design improved or the reviewer changed.

Start at 0 and 2 if you only read two. Spec 10 is where the design gets tested against the literature; the five structural changes it called for are applied.

## Provenance

This is an independent, clean-room implementation. It is informed by a study of a prior internal governance framework — the concepts it proved out, and the specific places where so much came to depend on its architecture that it resisted change. That study is summarised, in design terms only, in [prior art and departures](docs/spec/08-prior-art-and-departures.md). No code, prose, or configuration is carried across.

`docgov` is a working name.
