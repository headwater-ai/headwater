![Headwater. Documentation you can validate.](assets/headwater-social-preview.png)

# Headwater

**Documentation rots because nothing holds it accountable.** Headwater types your corpus, checks it as a graph, and accounts for every file it saw. Nothing else can answer *"is this corpus still true?"*

[![CI](https://github.com/headwater-ai/headwater/actions/workflows/ci.yml/badge.svg)](https://github.com/headwater-ai/headwater/actions/workflows/ci.yml) [![License](https://img.shields.io/github/license/headwater-ai/headwater)](LICENSE)

## Obtaining a named version

`v0.1.0` is the first tagged release, and it is the version to build if you want a tree that does not move under you. There is no published binary and no package registry entry: the install path is a source build, and it needs a Rust toolchain at **1.85 or later**, a floor `engine/README.md` explains.

```
git clone https://github.com/headwater-ai/headwater.git
cd headwater
git checkout v0.1.0
cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml --locked
```

The binary lands at `engine/target/release/headwater`, and `headwater --version` prints the number the tag names. The GitHub release for that tag states one digest: the value of the `release.digest` field inside `packages/headwater-standard/release.yml`. It is not what `sha256sum` prints for that file, because the field covers the files the record lists and cannot cover the record itself. Pass the stated value to `headwater taxonomy vendor --expect <digest>`. The check then rests on a number published outside the artifact rather than on one read out of it.

**Start here: [Your first governed corpus](docs/tutorials/your-first-governed-corpus.md).** Sixteen steps from an empty directory to a passing check, and it teaches the model rather than the commands. Every step states what you should now see, and `sh .claude/tutorial/fixtures.sh` runs the page against the engine in CI so that no output block on it can go stale quietly.

The tutorial above builds whatever tree you cloned, which is the default branch and moves. That is deliberate, because the tutorial is a claim about the default branch and CI holds it there. This paragraph is where the fixed version is.

<https://headwater.tools/> is the project site, and its documentation pages are the `docs/` tree of this repository, rendered.

> **Status: the engine runs, and two layers of it are unfinished.** M1 to M5 shipped, so this repository types its own corpus, checks it on every commit, and scaffolds, explains, routes, generates and exports it. M6 (distribution) and M7 (the measurement layer) are open, and every efficacy claim in this repository is still marked unmeasured.

## What problem this solves

Specs drift from code, rationale evaporates, standards multiply and contradict each other, and the AI assistants now reading that documentation as context inherit every one of those faults — silently, and at scale.

The usual answers are a style guide (unenforced), a wiki (unstructured), or a static-site generator (renders whatever you feed it). None of them can answer *"is this corpus still true?"*, because none of them know what kind of document anything is.

Headwater's premise: **make the structure of the corpus a machine-readable contract**, then derive everything else from it — validation, navigation, templates, AI instruction context, publishing, and the evidence that the whole thing is working.

## The three commitments

1. **Taxonomy is data.** What shelves exist, what document kinds live on them, what metadata they carry, what voice and lifecycle they obey, and how they may reference each other — all declared in one versioned schema. Two organizations with different documentation cultures run the same engine over different taxonomies. Customizing the taxonomy never means forking the tooling.

2. **The corpus is a graph.** Documents are typed nodes; front-matter references are typed edges. Every validation rule is a constraint on that graph, every query is a traversal of it, and every derived artifact (indexes, site navigation, AI rules, agent context) is a projection of it.

3. **AI assistants are readers in their own right, and measured ones.** The corpus feeds agents at intent time, read time, write time, and review time — and the system probes whether that context actually changes agent behavior, rather than assuming it does.

## Read the specification

[`docs/spec/README.md`](docs/spec/README.md) is the index of the specification shelf: every document the corpus checks, with the summary each one declares, in the reading order this corpus derives from its own relations rather than the sequence number in each file name. It is generated. `headwater generate` writes it from the front matter of the documents themselves, and `headwater generate --check` fails the build when the two drift apart. A table of the specification documents used to sit here, kept aligned by hand against the files it described. That copy is the defect [principle 2](docs/spec/00-vision-and-scope.md#design-principles) names, and deleting it is the remedy [13 — Open obligations](docs/spec/13-open-obligations.md#a-human-maintains-this-list-by-hand) asks for.

## License

Code is licensed under the Apache License, Version 2.0. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).

The prose under `docs/` is licensed under Creative Commons Attribution 4.0 International. See [`docs/LICENSE`](docs/LICENSE). Code samples inside that prose stay under Apache-2.0, so copying an example into your own project carries no attribution obligation.

Contributions arrive under the [Developer Certificate of Origin](https://developercertificate.org/), with a `Signed-off-by` line and no contributor license agreement. [`CONTRIBUTING.md`](CONTRIBUTING.md) states why. [`SECURITY.md`](SECURITY.md) carries the disclosure process, and [`TRADEMARKS.md`](TRADEMARKS.md) states the trademark position, which is that there is no registered mark.

[Q11](docs/spec/09-decisions.md#q11--license-and-distribution-posture) records the reasoning, and the [first-contact evaluation](docs/evaluations/first-contact.md) carries the full argument. The design narrowed the field and did not choose within it: internal-only and source-available terms are refused by rulings the specification already made, and network copyleft is refused for the library that spec 6 requires. The choice among the terms that remained belongs to the owner, who made it on 2026-08-11.
