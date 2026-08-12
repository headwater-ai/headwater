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
| `headwater-graph` | [M1](https://github.com/headwater-ai/headwater/milestone/1) | Resolves `relations:` into edges, indexes identifiers, binds external anchors, and reports what did not resolve |

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

## What the graph build adds, and the three things it had to decide

The graph build is the phase [spec 6](../docs/spec/06-engine-architecture.md) puts between the census and every check: it "resolves relations into edges, indexes identifiers, binds external anchors, and reports what it could not resolve". Over this repository it finds 36 nodes, 203 declared edge halves, 3 anchors and 1906 prose links, and nothing in either set fails to resolve. `tools/abox-check.py` counts the same 36 documents and the same 203 halves, which is the cross-check that the census pass also has while both stand-ins last.

**The census now hands over the document it read.** A row carries the parsed document rather than the path to parse again. Two passes over one corpus can differ — by an edit between them, or by one rule drifting from the other — and the two accounts that would differ here are the denominator and the graph, which is the pair that coverage is computed from. The cost is that a census holds the corpus in memory, and the phase that makes that a streaming pass is the runner in [M3](https://github.com/headwater-ai/headwater/milestone/3).

**Either half of a reciprocal pair may be the one that is written.** This corpus writes both: `cites_evidence` on the citing document and `cited_by` on the cited one, 81 times each. So an edge carries the name its author wrote and the relation that name resolves to, and `Edge::declared_triple` normalizes the two halves of one pair to one triple. A build that read only the forward name would report every reciprocal half in this corpus as an undeclared relation.

**A dangling edge and an untyped target are two reports.** The identifier index has two shelves. The typed one is the node set, because both ends of a declared relation are kinds. The second holds every other document that still declares an identifier, and it exists so that an edge naming one of them says *fix that document* rather than *fix this link*. Told apart, each report reaches an author who can act on it; reported as one class, the first sends its author to repair a link that is already correct.

Three questions reached this crate with no answer anywhere, and all three are in [13 — Open obligations](../docs/spec/13-open-obligations.md) rather than settled here. Spec 2 requires that an anchor string normalize and states no rule that does it. Nothing orders the two ends of a relation that admits a document and an anchor alike. And an anchor that resolves inside a declared corpus exclusion matches none of the three outcomes that spec 1 fixes.

## The fixtures are the deliverable

`crates/yaml/fixtures/` holds the corpus. `accept/` pairs a source with the tree it loads to, span by span. `reject/` pairs a source with the text an author would read. Both expectations are recorded files rather than assertions in Rust, so that the rules survive the replacement of the code under them.

    HEADWATER_BLESS=1 cargo test -p headwater-yaml --test fixtures
    HEADWATER_BLESS=1 cargo test -p headwater-doc --test fixtures
    HEADWATER_BLESS=1 cargo test -p headwater-census --test fixtures
    HEADWATER_BLESS=1 cargo test -p headwater-graph --test fixtures

That re-records every expectation. Read the diff before committing it, because a blessed fixture *is* the change.

`crates/doc/fixtures/` follows the same shape, with `.parse` for an accepted document. It adds one file that is not a pair: `corpus.exceptions` records every document under `docs/` that this repository cannot parse, and nothing about the ones it can. A parser is not the component that decides what a corpus should hold — an untyped file is a finding of the census, which is [#44](https://github.com/headwater-ai/headwater/issues/44) — so a refusal is recorded rather than raised. Recording only the exceptions is what keeps the file quiet: adding a well-formed document changes nothing, and adding one the engine cannot read changes a committed file and asks somebody to look.

`crates/census/fixtures/` holds two recorded censuses and the tree that the first one walks. `walk.census` records every row of the pathological tree, because every row of it is the point. `corpus.census` records this repository, and it prints the totals plus every row that is *not* a typed document. That is the same argument the exception list makes, with the count kept: a corpus adds a typed document most weeks, and a recorded file that changes on every commit is a file nobody reads, while the totals still account for every file, so a shrinking denominator still shows up in the diff. A test holds the two records to each other — a file the parser refuses may never come back typed, and an unreadable row may never appear without appearing in the parser's list too.

`crates/graph/fixtures/` follows the census's shape for the same reasons. `graph/` is a tree with one document per resolution outcome, and `graph.report` records every node, every edge and every link binding it produces. `corpus.graph` records this repository at the exceptions grain, and today it is the totals plus three anchors, because nothing in this corpus fails to resolve. It keeps the node and edge counts and drops the prose-link accounting, which is a narrower grain than the census keeps and it is chosen for the same reason. A node count moves when somebody adds a document, and a link count moves when somebody writes a sentence with a link in it. A recorded file that changes on nearly every commit is a file nobody reads, so what survives here is the number a regression moves: how many links did not resolve. A test holds the graph to the census: every node of the graph is a typed row, and every edge has a source that is a node.

## What is deliberately absent

The loader knows the dialect and nothing about the meaning. It does not require the root to be a mapping, it does not know that `kinds` is a declaration, and it resolves no `$`-reference. All three are shape, the meta-schema owns shape, and the meta-schema is [M2](https://github.com/headwater-ai/headwater/milestone/2). A loader that guessed at any of them would be a second schema that nobody declared.
