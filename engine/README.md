# The Headwater engine

The first code meant to survive. The [Q1 spike](../spike) retired four risks and says in its own README that it is not the start of an engine; this is, and it starts at the bottom of the pipeline that [spec 6](../docs/spec/06-engine-architecture.md) draws.

    cargo test

Needs Rust 1.85 or later. The floor comes from `saphyr-parser`, which is on edition 2024. A distribution `cargo` older than that reports `feature edition2024 is required` and nothing else, so check the toolchain first when a clean checkout will not build.

## Format and lint

A machine that installed Rust from its distribution usually has neither rustfmt nor clippy, and adding them changes a toolchain that other work depends on. A container answers both, and it pins the floor version at the same time:

    docker run --rm -v "$PWD/..":/w:ro -w /w/engine \
      -e CARGO_HOME=/tmp/cargo -e CARGO_TARGET_DIR=/tmp/target \
      rust:1.85-slim sh -c "rustup component add rustfmt clippy && cargo fmt --check && cargo clippy --all-targets && cargo test"

The mount is read only and the target directory sits inside the container, so a run leaves nothing behind and writes nothing into the checkout. The slim image carries neither component, which is why the command adds them.

## What is here

| Crate | Milestone | What it does |
|---|---|---|
| `headwater-yaml` | [M1](https://github.com/headwater-ai/headwater/milestone/1) | Reads a taxonomy source as YAML 1.2, on the Q2 dialect rules, and keeps a span on every node |
| `headwater-doc` | [M1](https://github.com/headwater-ai/headwater/milestone/1) | Reads a document: the front-matter block through the loader, and the body as CommonMark |
| `headwater-census` | [M1](https://github.com/headwater-ai/headwater/milestone/1) | Walks a corpus root, resolves a kind for each document, and reports what became of every file |

## Why the loader came first

[Q2](../docs/spec/09-decisions.md#q2--schema-format) fixes four loader rulings, and it says that honoring them now costs nothing while retrofitting them is expensive. The reason is the fixture corpus rather than the code: M2 replaces the overlay resolver, and it has to do that without invalidating a single M1 fixture. A fixture recorded against a loader that read `no` as `false` would have to be rewritten, and the rewrite is where a corpus quietly changes meaning.

The four rulings, and where each one lives:

- **YAML 1.2 core schema, so `no` stays the string `no`.** `core_schema.rs`, which the loader never calls.
- **Duplicate keys are an error.** `loader.rs`, reported with the position of the first declaration as well as the second.
- **Anchors, aliases and merge keys are forbidden.** `loader.rs`. The `$`-reference is the reuse mechanism, and an alias is a second one that no overlay can address.
- **Scalar types come from the meta-schema and never from the YAML resolver.** `value.rs` has no `Null` variant and no `Bool`. A scalar keeps its text and the style it was written in, and nothing types it until a declared type asks.

A fifth ruling reached Q2 from this direction rather than the other. The engine derived it from the alias argument, and the decision then accepted it: an explicit tag is refused, because a tag declares a type and the meta-schema is the authority on type. Q2 records why it settled the rule instead of deferring it — a loader can relax a rule later at no cost, and cannot add one later without a finding against every source that already used the form.

## What the parser adds, and the one rule it changed

The body is a real CommonMark parse rather than a scan by regular expression, and the reason is the failure mode rather than completeness. [Spec 12](../docs/spec/12-check-layer.md#the-correctness-roots) names the parser's spans a correctness root because a mis-parsed heading lets a section contract hold with **no finding anywhere**. A leading-`#` matcher reports a heading inside every fenced code block in this repository, and misses every setext heading. Both defects are silent.

Ownership is the second half of that entry. Spec 12 puts the author-owned span on the correctness-root list on measured grounds: over this repository's own specification, the remaining structural share of a lexical checker's errors came from text that quotes another author. A voice rule cannot exempt a quotation, because the rule cannot see one. The parser marks every run of text as `authored`, `quoted` or `code`, and the rule reads the mark.

Front matter goes through `headwater-yaml` rather than through a second tree builder, and [#43](https://github.com/headwater-ai/headwater/issues/43) asked which Q2 rulings survive that move. All but one. The core schema, duplicate keys, anchors, aliases, merge keys, explicit tags and the one-document rule each hold for the reason it was made. The exception is an empty source: a taxonomy source that declares nothing is a mistake, while a document with an empty block is an untyped document, and refusing it would make the census count a file the engine could not read rather than a file nobody has typed. That is the distinction the census exists to draw, so the dialect is a parameter of the loader — `headwater_yaml::Options` — rather than a second loader that drifts from the first.

## What the census adds, and the two questions it had to answer

The census is the denominator. [Spec 12](../docs/spec/12-check-layer.md) puts it before any check runs, so that a document which failed to classify is *visibly* unchecked rather than silently absent, and it names both halves of `headwater-census` a correctness root. The two halves share a crate because they share one failure: each produces a systematically green result when it is wrong. A walk that misses a subtree reports nothing about it. A resolution that assigns the wrong kind runs the wrong checks and passes them.

**A symlink is an entry and it is never followed.** To follow one either duplicates a file that the denominator already holds, or leaves the corpus root, or loops. `crates/census/fixtures/walk/` holds all four link cases plus the rest of the pathological tree that spec 12 asks for by name, and `walk.census` records what the walk makes of each.

**An exclusion changes what a row says and never whether there is one.** The excluded files stay in the walk and stay in the count, because a subtree that vanishes from the report is exactly the conversion of an accounting into a silence that [spec 7](../docs/spec/07-distribution-and-federation.md) forbids under a different mechanism.

Two questions reached this crate with no answer anywhere, and both are now recorded in [13 — Open obligations](../docs/spec/13-open-obligations.md) rather than settled here. Spec 2 says the most specific shelf pattern wins and never says what specificity is; `pattern.rs` states the ordering it uses. Spec 2's fourth resolution step is a path-pattern refinement that nothing gives a syntax, so the step is absent rather than guessed at.

## The fixtures are the deliverable

`crates/yaml/fixtures/` holds the corpus. `accept/` pairs a source with the tree it loads to, span by span. `reject/` pairs a source with the text an author would read. Both expectations are recorded files rather than assertions in Rust, so that the rules survive the replacement of the code under them.

    HEADWATER_BLESS=1 cargo test -p headwater-yaml --test fixtures
    HEADWATER_BLESS=1 cargo test -p headwater-doc --test fixtures
    HEADWATER_BLESS=1 cargo test -p headwater-census --test fixtures

That re-records every expectation. Read the diff before committing it, because a blessed fixture *is* the change.

`crates/doc/fixtures/` follows the same shape, with `.parse` for an accepted document. It adds one file that is not a pair: `corpus.exceptions` records every document under `docs/` that this repository cannot parse, and nothing about the ones it can. A parser is not the component that decides what a corpus should hold — an untyped file is a finding of the census, which is [#44](https://github.com/headwater-ai/headwater/issues/44) — so a refusal is recorded rather than raised. Recording only the exceptions is what keeps the file quiet: adding a well-formed document changes nothing, and adding one the engine cannot read changes a committed file and asks somebody to look.

`crates/census/fixtures/` holds two recorded censuses and the tree that the first one walks. `walk.census` records every row of the pathological tree, because every row of it is the point. `corpus.census` records this repository, and it prints the totals plus every row that is *not* a typed document. That is the same argument the exception list makes, with the count kept: a corpus adds a typed document most weeks, and a recorded file that changes on every commit is a file nobody reads, while the totals still account for every file, so a shrinking denominator still shows up in the diff. A test holds the two records to each other — a file the parser refuses may never come back typed, and an unreadable row may never appear without appearing in the parser's list too.

## What is deliberately absent

The loader knows the dialect and nothing about the meaning. It does not require the root to be a mapping, it does not know that `kinds` is a declaration, and it resolves no `$`-reference. All three are shape, the meta-schema owns shape, and the meta-schema is [M2](https://github.com/headwater-ai/headwater/milestone/2). A loader that guessed at any of them would be a second schema that nobody declared.
