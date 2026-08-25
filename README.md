# Headwater

A **documentation governance system**: a typed, validated, self-describing corpus of documentation that machines can check and agents can navigate — where the taxonomy itself is configuration, not code.

> **Status: the engine runs, and two layers of it are unfinished.** M1 to M5 shipped, so this repository types its own corpus, checks it on every commit, and scaffolds, explains, routes, generates and exports it. M6 (distribution) and M7 (the measurement layer) are open, and every efficacy claim in this repository is still marked unmeasured.

**Start here: [Your first governed corpus](docs/tutorials/your-first-governed-corpus.md).** Sixteen steps from an empty directory to a passing check, and it teaches the model rather than the commands. Every step states what you should now see, and `sh .claude/tutorial/fixtures.sh` runs the page against the engine in CI so that no output block on it can go stale quietly.

## What problem this solves

Documentation rots because nothing holds it accountable. Specs drift from code, rationale evaporates, standards multiply and contradict each other, and the AI assistants now reading that documentation as context inherit every one of those faults — silently, and at scale.

The usual answers are a style guide (unenforced), a wiki (unstructured), or a static-site generator (renders whatever you feed it). None of them can answer *"is this corpus still true?"*, because none of them know what kind of document anything is.

Headwater's premise: **make the structure of the corpus a machine-readable contract**, then derive everything else from it — validation, navigation, templates, AI instruction context, publishing, and the evidence that the whole thing is working.

## The three commitments

1. **Taxonomy is data.** What shelves exist, what document kinds live on them, what metadata they carry, what voice and lifecycle they obey, and how they may reference each other — all declared in one versioned schema. Two organizations with different documentation cultures run the same engine over different taxonomies. Customizing the taxonomy never means forking the tooling.

2. **The corpus is a graph.** Documents are typed nodes; front-matter references are typed edges. Every validation rule is a constraint on that graph, every query is a traversal of it, and every derived artifact (indexes, site navigation, AI rules, agent context) is a projection of it.

3. **AI assistants are readers in their own right, and measured ones.** The corpus feeds agents at intent time, read time, write time, and review time — and the system probes whether that context actually changes agent behavior, rather than assuming it does.

## Read the specification

[`docs/spec/README.md`](docs/spec/README.md) is the index of the specification shelf: every document the corpus checks, with the summary each one declares. It is generated. `headwater generate` writes it from the front matter of the documents themselves, and `headwater generate --check` fails the build when the two drift apart. A table of the specification documents used to sit here, kept aligned by hand against the files it described. That copy is the defect [principle 2](docs/spec/00-vision-and-scope.md#design-principles) names, and [13 — Open obligations](docs/spec/13-open-obligations.md#a-human-maintains-this-list-by-hand) filed it against this repository. Deleting it is the remedy that file asks for.

The order in the index is the reading precedence this corpus derives from its own relations, and not the sequence number in each file name. So [13 — Open obligations](docs/spec/13-open-obligations.md) comes before [HW-EVAL-theoretical-foundations](docs/evaluations/theoretical-foundations.md), rather than after it.

[`09-open-questions.md`](docs/spec/09-open-questions.md) stays at its old path as a redirect map. This corpus cites its anchors 136 times, and this project does not rewrite a point-in-time record. A declared projection writes the file now, so the index above lists the fifteen documents that a person maintains and leaves this one out.

The [glossary](docs/spec/glossary.md) lists every named concept in one place, with a link to the section that defines it. It also marks the eight terms that an author needs to file a document, which is the whole of the vocabulary that most readers meet.

Evidence gathered for specific open questions lives in [`docs/evaluations/`](docs/evaluations/). Worked examples of the taxonomy in [LinkML](docs/evaluations/linkml-worked-example.md) and the checks in [SHACL](docs/evaluations/shacl-worked-example.md) supplied the substance for Q13. A [language evaluation](docs/evaluations/language-choice.md) and the [spike results](docs/evaluations/language-spike-results.md) closed Q1, a [cognitive-dimensions walkthrough](docs/evaluations/schema-format-walkthrough.md) closed Q2, a [first-run walkthrough](docs/evaluations/default-taxonomy-first-run.md) closed Q3, and a [relation-storage evaluation](docs/evaluations/relation-storage.md) closed Q4. Six later evaluations closed the rest in groups: [graph export and federation](docs/evaluations/graph-export-and-federation.md) for Q6, Q9 and Q13, [the serving boundary](docs/evaluations/the-serving-boundary.md) for Q7, Q14 and Q17, [warrant and adjudication](docs/evaluations/warrant-and-adjudication.md) for Q15, Q18 and Q19, [what a check can know](docs/evaluations/what-a-check-can-know.md) for Q5 and Q21, [the measurement layer](docs/evaluations/the-measurement-layer.md) for Q8 and Q20, and [first contact](docs/evaluations/first-contact.md) for Q11, Q12 and Q16.

One evaluation closes nothing and is not meant to. [The taxonomy in OWL and SKOS](docs/evaluations/owl-skos-worked-example.md) is the third worked example beside LinkML and SHACL, and it reports what an RDF projection cannot carry. Q13 stages that emitter fourth, behind a named external consumer, so the document is a statement of what the emitter will owe rather than an argument to write it. Unlike the other two it was run rather than written: the probe is at [`tools/rdf-probe/`](tools/rdf-probe/).

The canonical taxonomy library lives in [`docs/taxonomies/`](docs/taxonomies/) — a curated set of taxonomies, each one modeling a named documentation tradition. Its [index](docs/taxonomies/README.md) states what admits an entry, what an entry ships, and where a draft lives before there is an engine to run it. No entry is admitted yet.

Doctrine prose — what an adopter should do with the mechanisms the specification fixes — lives in [`docs/doctrine/`](docs/doctrine/). It currently holds [the maturity ladder](docs/doctrine/maturity-model.md), which names the ordered positions a consumer moves through and states the one thing a position is never allowed to mean. A level is a named subset of the conformance rule set and nothing more: it adds no declaration, and it never decides whether a check blocks. That file is prose about a package rather than a document in this corpus, so no shelf claims it and the census reports it as unclassified, in the same way it reports [`docs/w3id/README.md`](docs/w3id/README.md).

Design reviews and the prompts that commission them live in [`docs/reviews/`](docs/reviews/) — currently a [core-concepts review](docs/reviews/core-concepts-review-prompt.md) aimed at simplification and robustness. The prompts are committed alongside their findings on purpose: a review whose instrument is unrecorded cannot be repeated against a later draft, and comparing two runs of the same instrument is the only way to tell whether the design improved or the reviewer changed.

**This corpus is typed against that library entry.** Every document under `docs/spec/`, `docs/evaluations/` and `docs/reviews/` carries front matter with a state, a summary, a kind, an identifier, a warrant and its declared edges. [`.headwater/`](.headwater/) holds the binding, [`packages/headwater-standard/`](packages/headwater-standard/) holds the base package it takes, and `headwater check` runs the result in CI. What the typing could not express is recorded in [13 — Open obligations](docs/spec/13-open-obligations.md#what-the-first-typing-of-this-corpus-found), together with what the pass cost.

Start at 0 and 2 if you only read two. [HW-EVAL-theoretical-foundations](docs/evaluations/theoretical-foundations.md) is where the design gets tested against the literature, and all twenty-one changes it called for are applied.

## License

Code is licensed under the Apache License, Version 2.0. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).

The prose under `docs/` is licensed under Creative Commons Attribution 4.0 International. See [`docs/LICENSE`](docs/LICENSE). Code samples inside that prose stay under Apache-2.0, so copying an example into your own project carries no attribution obligation.

Contributions arrive under the [Developer Certificate of Origin](https://developercertificate.org/), with a `Signed-off-by` line and no contributor license agreement. [`CONTRIBUTING.md`](CONTRIBUTING.md) states why. [`SECURITY.md`](SECURITY.md) carries the disclosure process, and [`TRADEMARKS.md`](TRADEMARKS.md) states the trademark position, which is that there is no registered mark.

[Q11](docs/spec/09-decisions.md#q11--license-and-distribution-posture) records the reasoning, and the [first-contact evaluation](docs/evaluations/first-contact.md) carries the full argument. The design narrowed the field and did not choose within it: internal-only and source-available terms are refused by rulings the specification already made, and network copyleft is refused for the library that spec 6 requires. The choice among the terms that remained belongs to the owner, who made it on 2026-08-11.
