# The Headwater engine

The first code meant to survive. The [Q1 spike](../spike) retired four risks and says in its own README that it is not the start of an engine; this is, and it starts at the bottom of the pipeline that [spec 6](../docs/spec/06-engine-architecture.md) draws.

    cargo test
    cargo run -p headwater-cli -- check --root ..

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
| `headwater-check` | [M1](https://github.com/headwater-ai/headwater/milestone/1) | Runs two checks generated from the taxonomy, and computes coverage against the census |
| `headwater-cli` | [M1](https://github.com/headwater-ai/headwater/milestone/1) | The `headwater` binary. One verb, `check`, and it is what CI runs |
| `headwater-ref` | [M2](https://github.com/headwater-ai/headwater/milestone/2) | The `$`-reference sublanguage: an overlay address, and a reference into a vocabulary or a package |
| `headwater-meta` | [M2](https://github.com/headwater-ai/headwater/milestone/2) | The meta-schema, as a file in the dialect it describes, and what it decides over one source |

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

## What the runner adds, and the four things it had to decide

The runner is Phase B of [spec 12](../docs/spec/12-check-layer.md#two-phases-and-why-the-order-matters). Phase A is the census and the graph, and it fixes the denominator before any check starts. Over this repository the run is 61 files seen, 36 classified, 33 checked, 122 check instances and 3 findings. Every one of the three findings is the coverage rule reporting a classified document that no rule read, which is exactly the report OB-COV-2 exists to produce.

**M1 ends with a binary, and the epic decided that rather than taste.** [#46](https://github.com/headwater-ai/headwater/issues/46) held the choice open between a `headwater` binary and a test that CI runs. M1 is done when the loop "runs in this repository's CI over `docs/`, **advisory**", and a test cannot be advisory: a failing test fails the job, which is the opposite posture. Exit status is what CI reads, so the component that owns the posture is the one that exits. `headwater check` exits 0 with findings on stdout, and `--strict` is the gate ([spec 6](../docs/spec/06-engine-architecture.md#cli)). The workflow needs no `continue-on-error`, and a promotion to blocking is one flag with a diff behind it.

**Both checks are generated from the taxonomy, and neither names a declaration.** The reciprocity check reads the relations that declare `reciprocal: required`, and the placement check reads the facets that a heterogeneous shelf uses as its discriminator. A taxonomy that declares one more relation or one more shelf gets one more check, and neither source file changes. That is the property that separates a generated check from a written one, and `crates/check/fixtures/` asserts it directly: the fixture taxonomy declares a second relation with no `reciprocal`, a document writes an edge of it, and no instance appears over that edge.

**A check instance is counted against every document it read, and not against one end.** [Spec 12](../docs/spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on) fixes what an edge-scoped instance reads: "one relation instance **and both endpoints**". A reciprocity instance over a pair therefore counts against both files. Attributing it to the declaring end alone is the silent pass one level up: the document at the receiving end *is* checked, the rule fires when that document is the one missing its half, and a coverage report that called it unchecked would send its author to look at a shelf pattern that is already right. Over this repository the difference is four documents.

**The heterogeneous half of "placement is primary" is a skip and not a pass.** [Spec 2](../docs/spec/02-taxonomy-model.md#placement-is-primary-metadata-fills-the-gap) states one rule with two halves, and only one of them is a check. On a homogeneous shelf a discriminator in front matter is the second truth the rule forbids. On a heterogeneous shelf the discriminator is what metadata is for, and kind resolution already decided it — a document whose discriminator is missing or not admitted carries no kind, and the census reports it. An instance that reported a foregone pass would let coverage count a document as checked by a rule that could never fail, so the instance is skipped with its reason and the report prints the count.

Two things the runner meets are gaps rather than decisions, and [13 — Open obligations](../docs/spec/13-open-obligations.md) carries them. A finding names the obligation it serves, and an obligation is data that no package here declares, so two of the three rules name none and take the engine's own severity. Spec 12 also calls a dangling edge and an unclassifiable path *structural findings* of Phase A, and this runner leaves them in the census and the graph rather than reporting each one twice.

## What the reference grammar settles, and the one thing it cannot

M2 starts with the sublanguage, because the meta-schema and the resolver both read it and neither can wait for it. [Q2](../docs/spec/09-decisions.md#q2--schema-format) counted three uses — a vocabulary reference, a package reference, and an overlay address — and gave them no grammar. [Spec 2](../docs/spec/02-taxonomy-model.md#the--reference-sublanguage) is the definition and `crates/ref` is that definition as a parser.

**The three uses are one path production read from two places.** An address wears no sigil and a reference does, and the difference is not a convention. An address is written where nothing else is legal, which is the key under `add`, `override`, `add_to` and `remove_from`. A reference is written in a value position, where a literal value is equally legal, so something has to separate the two. That rule decides the next surface that needs a sigil, and a convention would not.

**An address is a sequence of segments, and that is what makes confluence decidable.** [#50](https://github.com/headwater-ai/headwater/issues/50) records that this repository's overlay writes `kinds.design_spec.identifier` and `kinds.design_spec.language`, and that both stand-ins skip an address of that depth. The two are disjoint, so the two `add` operations commute. `kinds.playbook` is a textual prefix of `kinds.playbook_step` and neither address contains the other. A confluence check over text answers both cases wrongly, and `disjoint.record` holds the pairs.

**The grammar cannot refuse a list index, and the fixture says so rather than hiding it.** `regimes.lifecycle.standard.transitions.0` parses, because `0` is a legal key name and no lexical rule tells an index from a key. The refusal is the resolver's, and it reads the node the address lands on. That is the one rule of the section that the parser does not carry.

Two rules here are guesses, and both guess in the direction that Q2 settled for the loader. A segment holds letters, digits and `_`, so a hyphen is refused today. A reference points at a value and never at a second reference. Each is cheap to relax and expensive to add later, which is the only asymmetry that decides such a question before a use exists.

## What the meta-schema settles, and the two collisions it found

`crates/meta/meta-schema.yml` is [spec 2](../docs/spec/02-taxonomy-model.md#the-meta-schema)'s meta-schema section as a file, and `crates/meta` is the reader of that file. The eleven declarations, `vocabularies`, and the three fields that name a taxonomy are the closed root set. Every declaration has a shape, every scalar has a type, and every closed value set is written out.

**The file is in the dialect it describes, and that is the whole of "its own dialect".** It goes through `headwater-yaml` on the Q2 rulings, exactly as a taxonomy source does. [Q2](../docs/spec/09-decisions.md#q2--schema-format) rules that JSON Schema is an emitted export and never the validator, and spec 2 shows the reason in its own text: the language carries `$`-references and no JSON Schema keyword resolves one. A second language for the schema would be a second dialect to keep in step with the first, and the two would drift at the first ruling that only one of them heard.

**Four rules of the twenty-one run here, and the other seventeen are named rather than skipped quietly.** Structural conformance, reference well-formedness, the reserved root, and the refusal of an address into a list are decidable over one source. Everything else — referential integrity, confluence, core satisfiability and the rest — reads a *resolved* tree. `validate::SKIPPED` holds each one with the reason it waits, and `fixtures/schema.record` prints the list, so a caller reports what it did not run instead of a pass it did not earn.

**The three rules [#48](https://github.com/headwater-ai/headwater/issues/48) could not put in the grammar land here, and one of them changed shape on the way.** An address into a list is refused by the meta-schema and not by the resolver, because the meta-schema already knows which positions hold lists and the answer needs no tree. `package` is refused as a declared root and as the head of an address. And "a reference points at a value and never at a second reference" is a rule of *shape*: the position that a reference reads is not `reference: allowed`, so `vocabularies.a: $vocabularies.b` fails on form rather than on a rule that the resolver has to carry.

**The base package does not validate, and the collision is in the specification rather than in the file.** `corpus.meta` records this repository's three sources. The design-spec bundle and the adopter overlay are valid. The base package is refused twice, on one word: its only committed copy opens `package: headwater/standard`, and spec 2 makes `package` a reserved reference root that no taxonomy may declare. [Spec 7](../docs/spec/07-distribution-and-federation.md#publishing) names a *package manifest* with exactly that key, so the two files are meant to be two files, and the one that exists is playing both parts. [13 — Open obligations](../docs/spec/13-open-obligations.md) carries the finding beside the one it already had about the base having no file of its own.

**A mapping key in the specification's own example cannot be addressed.** `mappings[].facet_values` is keyed `status.current`, a facet and one of its values joined by a dot, and the same specification says that the meta-schema refuses to declare a key holding a dot, because an address over it would be ambiguous. Both clauses are spec 2's. The file takes the reading that no overlay reaches inside `facet_values`, writes `addressable: false` there, and spec 13 carries the rest.

Six value sets here are closed tighter than the specification states, each marked `guess:` in the file, and eight surfaces are marked `gap:` where a declaration is required and no form is given. Both markers follow the direction Q2 settled: a rule relaxes later at no cost, and cannot be added later without a finding against every source that already used the form.

## The fixtures are the deliverable

`crates/yaml/fixtures/` holds the corpus. `accept/` pairs a source with the tree it loads to, span by span. `reject/` pairs a source with the text an author would read. Both expectations are recorded files rather than assertions in Rust, so that the rules survive the replacement of the code under them.

    HEADWATER_BLESS=1 cargo test -p headwater-yaml --test fixtures
    HEADWATER_BLESS=1 cargo test -p headwater-doc --test fixtures
    HEADWATER_BLESS=1 cargo test -p headwater-census --test fixtures
    HEADWATER_BLESS=1 cargo test -p headwater-graph --test fixtures
    HEADWATER_BLESS=1 cargo test -p headwater-check --test fixtures
    HEADWATER_BLESS=1 cargo test -p headwater-ref --test fixtures
    HEADWATER_BLESS=1 cargo test -p headwater-meta --test fixtures

That re-records every expectation. Read the diff before committing it, because a blessed fixture *is* the change.

`crates/doc/fixtures/` follows the same shape, with `.parse` for an accepted document. It adds one file that is not a pair: `corpus.exceptions` records every document under `docs/` that this repository cannot parse, and nothing about the ones it can. A parser is not the component that decides what a corpus should hold — an untyped file is a finding of the census, which is [#44](https://github.com/headwater-ai/headwater/issues/44) — so a refusal is recorded rather than raised. Recording only the exceptions is what keeps the file quiet: adding a well-formed document changes nothing, and adding one the engine cannot read changes a committed file and asks somebody to look.

`crates/census/fixtures/` holds two recorded censuses and the tree that the first one walks. `walk.census` records every row of the pathological tree, because every row of it is the point. `corpus.census` records this repository, and it prints the totals plus every row that is *not* a typed document. That is the same argument the exception list makes, with the count kept: a corpus adds a typed document most weeks, and a recorded file that changes on every commit is a file nobody reads, while the totals still account for every file, so a shrinking denominator still shows up in the diff. A test holds the two records to each other — a file the parser refuses may never come back typed, and an unreadable row may never appear without appearing in the parser's list too.

`crates/graph/fixtures/` follows the census's shape for the same reasons. `graph/` is a tree with one document per resolution outcome, and `graph.report` records every node, every edge and every link binding it produces. `corpus.graph` records this repository at the exceptions grain, and today it is the totals plus three anchors, because nothing in this corpus fails to resolve. It keeps the node and edge counts and drops the prose-link accounting, which is a narrower grain than the census keeps and it is chosen for the same reason. A node count moves when somebody adds a document, and a link count moves when somebody writes a sentence with a link in it. A recorded file that changes on nearly every commit is a file nobody reads, so what survives here is the number a regression moves: how many links did not resolve. A test holds the graph to the census: every node of the graph is a typed row, and every edge has a source that is a node.

`crates/meta/fixtures/` holds four sets. `taxonomy/` and `overlay/` pair a source with its verdict, and a source that passes records the word `valid`, because a record that was empty on a pass and empty on a missing fixture would say the same thing twice. `schema.record` is the meta-schema as the crate read it, and it is the file to read in a diff when `meta-schema.yml` changes, because a shape edit is otherwise visible only in the cases it moves. `addresses.record` asks the meta-schema where each of a list of addresses lands, which is the question the resolver of [#50](https://github.com/headwater-ai/headwater/issues/50) asks before it merges anything. `corpus.meta` records this repository's own three sources at the grain the graph and the check records use: the verdict per source and nothing else.

`crates/check/fixtures/` carries the floor [spec 12](../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship) sets: "every check ships with at least one fixture that it fails and one that it passes." The tree under `check/` holds both for each of the three rules, including both directions of a missing reciprocal half, and `check.report` records every instance and every finding it produces. `corpus.checks` records this repository at the same grain the graph uses: the coverage totals, the instance count per rule, and every finding, with no per-document rows. Those numbers move when somebody adds a document or declares an edge, which is the event the record exists to show, and they do not move when somebody writes a paragraph.

## What is deliberately absent

The loader knows the dialect and nothing about the meaning. It does not require the root to be a mapping, it does not know that `kinds` is a declaration, and it resolves no `$`-reference. All three are shape, and shape is `headwater-meta`'s. A loader that guessed at any of them would be a second schema that nobody declared. The third one is still open at the far end: `headwater-meta` says where a reference may stand and whether it parses, and nothing resolves one until the resolver of [#50](https://github.com/headwater-ai/headwater/issues/50).

Nothing in the pipeline calls the meta-schema yet, and that is the shape of M2 rather than an omission. `headwater check` reads its taxonomy through the stand-in, which merges three sources without validating any of them, and a verb that validates is [`headwater taxonomy validate`](https://github.com/headwater-ai/headwater/issues/51). To wire the validator in front of the stand-in would report the base package as invalid on every run of CI, over a rule that the resolver is the component to fix.

The runner is the thinnest thing that closes the loop, and four parts of the designed check layer are not in it. A check declares no scope and no view enforces one, which is [#54](https://github.com/headwater-ai/headwater/issues/54). Nothing is cached and nothing is change-scoped, which is [#55](https://github.com/headwater-ai/headwater/issues/55), and a cache before a sound cache key is the correctness root spec 12 warns about. No finding can be suppressed, so there is no suppression inventory, which is [#58](https://github.com/headwater-ai/headwater/issues/58). And no rule can name an obligation, because obligations are declarations that [#52](https://github.com/headwater-ai/headwater/issues/52) supplies.

Where the taxonomy comes from is the fifth. `headwater check` reads it through `headwater_census::standin`, so the binary runs in this repository and nowhere else. The resolver is [#50](https://github.com/headwater-ai/headwater/issues/50), and the stand-in dies with it.
