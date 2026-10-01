---
id: HW-SPEC-parse-and-census
status: current
status_since: 2026-10-01
summary: "How three crates parse each document, walk the corpus root, resolve each kind with its derivation, give each file one census outcome, and run git."
last_verified: 2026-10-01
title: "Parse and census"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft+revise
  evidence_basis: evidenced
relations:
  governs:
    - engine/crates/doc/src/**
    - engine/crates/census/src/**
    - engine/crates/vcs/src/**
  traces_to:
    - HW-SPEC-engine-architecture
    - HW-SPEC-taxonomy-model
    - HW-SPEC-check-layer
    - HW-SPEC-assurance-model
    - HW-IFACE-headwater-change
    - HW-IFACE-headwater-derived
    - HW-IFACE-headwater-merge-driver
    - HW-DR-0072
    - HW-DR-0084
    - HW-DR-0049
---

# Parse and census

## Scope

This spec describes the inside of the parse stage of [spec 6](../spec/06-engine-architecture.md#pipeline), with the census that the stage emits. Three crates under `engine/crates/` build that stage: `doc`, `census` and `vcs`.

The stage has two inputs. The first is the corpus root on disk. The second is the shelves and kinds of the resolved taxonomy, which the [Taxonomy resolution](taxonomy-resolution.md) subsystem writes into the lock. The stage gives a `Census`, with one `Row` for each file under the corpus root. Each row carries the parsed `Document` where the stage read one. [Graph build](graph-build.md) reads the documents through the census and opens no document file itself.

The `vcs` crate is in this subsystem because `census` and `graph` use it, as the row of spec 6 states. It writes the change manifest that `headwater check --change` reads. It also answers two questions about the tree for other crates. Which paths does git ignore, and which merge attribute does git give a path?

Other documents state what the stage does, and this spec does not repeat them:

- [Spec 6](../spec/06-engine-architecture.md#pipeline) states the promise of the parse stage and of the census.
- [Spec 2](../spec/02-taxonomy-model.md#kind-resolution) states the four steps of kind resolution.
- [Spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for) states OB-COV-1, which the census discharges.
- [Spec 12](../spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version) states why the check path runs no version control command.
- The [`headwater change`](../interfaces/headwater-change.md) contract states the manifest, and the [merge driver](../interfaces/headwater-merge-driver.md) contract states what the driver does with a merge attribute.
- The [`headwater derived`](../interfaces/headwater-derived.md) contract states the population of derived files.

The public Rust API of each crate is not in this spec ([HW-DR-0098](../decisions/0098-an-engine-subsystem-is-described-by-a-technical-design-spec-on-a-shelf-of-its-own-and-its-behavior-stays-where-it-is-already-written.md)). The doc comments of the crates state it.

## Design

### Two sources in one document

`split.rs` cuts a document into its front matter and its body. It is the only part of the parser that reads the source as bytes and not through a grammar. The opening fence is the first line of the file, so a thematic break later in the body is never front matter. `split` also removes a byte-order mark first, because an editor on Windows writes one, and without it every file of such a corpus is untyped.

The front matter loads through `headwater_yaml`, so the parser has no second YAML tree builder. The loader takes one parameter for this crate: empty front matter is an untyped document and not an unreadable file. A block that loads to a sequence or a scalar is `Reason::NotAMapping`, because facets have names.

`parse` refuses a file with no front matter, and the census uses it for every file under the corpus root. `parse_prose` reads a file with no block as an empty mapping. It is for prose outside the corpus root that a language regime lists ([HW-DR-0084](../decisions/0084-a-language-rule-reaches-front-door-prose-outside-the-corpus-root-and-no-other-rule-does.md)), and for the read-back of `check --fix`.

### A CommonMark parse, not a regular expression

`body.rs` scans the body with a CommonMark parser. Spec 12 names the spans of the parser a correctness root, and the failure is a silent pass. A scan by regular expression reads a `# heading` inside a fenced code block as a heading, and a section contract then holds with no finding. A setext heading has no leading `#`, so a pattern match does not see it at all.

The scan keeps three things. `Block` is each block in document order, with its span, its soft breaks and the info string of a fence. `Link` is each link with its destination as written, because normalization belongs to the anchor resolvers. `Ownership` marks each run of text as `Authored`, `Quoted` or `Code`. A voice rule that reads a block quote as the author's own prose reports the grammar of another author as a defect. No exemption in the rule can repair that, because the rule cannot see the quote, and the parser can.

`lines.rs` turns a byte offset into a line and a column. A column is a count of characters from one, as the loader counts it. A byte column puts the caret inside an em dash, and this corpus has many.

### Sentences

`sentences.rs` splits the text that the author wrote into sentences. Spec 12 puts segmentation on the correctness-root list, and [Q5](../spec/09-decisions.md#q5--voice-checking-depth) measured that 32 of 58 sentence-length errors on one landing were defects in the splitter. The module works on the runs that `body.rs` gives it, so it removes no Markdown itself.

It has three rules. A sentence ends at a terminator that a new sentence follows. A terminator inside code is not a terminator, because the parser marks that run `Code`. A quoted block holds no sentence of this document, so no rule downstream declares an exemption for a quotation. [HW-OBL-0153](../obligations/0153-the-rewritten-sentence-splitter-has-no-dedicated-conformance-fixtures.md) is open because the splitter has no data-driven fixture corpus.

### Errors and the provenance readers

`error.rs` gives each rejection a `Reason` and a span, and `render` writes one line per error in source order. The doc fixtures record that form. A file that cannot be split reports one error, because nothing after that point is readable.

`lib.rs` holds the names of the provenance block and its readers. Spec 3 gives the shape of that block to the engine and not to a taxonomy, so the four warrant values are constants here. The checks, the query layer and the audit read the set from these constants and do not spell the values again. `is_warrant` compares case exactly, because a reader that folded case would guess what the author meant.

### Why resolution and the census share a crate

The `census` crate holds kind resolution and the census. Spec 12 names both a correctness root, and both fail in the same way: the result is green when it is wrong. A walk that misses a subtree reports nothing about it. A resolution that assigns the wrong kind runs the wrong checks and passes them. Thus the rules of the walk are written out in the code, and a fixture tree holds them.

### The walk

`walk.rs` enumerates the corpus root, and its rules are stated in its module comment. Every file gives exactly one entry. A symbolic link is never followed, and it gives an entry of its own with its target. A declared exclusion does not stop the walk, so an excluded file gives an entry with the rule that excluded it. A directory that the walk cannot read gives an entry, because that entry is the only record of the files under it. A missing root gives one entry, so a misconfigured root is a visible row and not an empty census.

Order is the path, in bytes. Thus two runs over one tree give the same census, and a recorded census is a fixture. A named pipe, a socket or a device is `Special`, and nothing opens one. A read of a pipe with no writer does not end (#1333). `Classification` puts a path that nobody has written yet into exactly one of four places, by its name alone (#319).

The fixture tree `engine/crates/census/fixtures/walk/` holds one case for each rule.

### Shelves and kinds

`shelves.rs` reads two declarations from the resolved taxonomy: the shelves, and which kinds are abstract. It validates nothing else, and a key that it does not read passes through. A component that guessed at shape would be a second schema that nobody declared. The meta-schema owns shape.

### Kind resolution and its derivation

`resolve.rs` resolves a kind from a path and the front matter, and it records each step as a `Step`. `headwater explain` prints the derivation, and the MCP surface reads the same value, so the two say the same thing. Every resolution returns a derivation, the failed ones too, because the derivation of a failure tells an author which step to repair.

`resolve` returns a `Resolution`, never a `Result`. A document that no shelf claims, and a document with no discriminator, are ordinary states of a corpus that spec 4 counts. A `Result` would let a caller use `?`, and the file would leave the denominator at that line.

Three steps run. `shelf_for` is step 1 alone, because it needs only a path, so the census can ask it of a file with no front matter. It sorts the shelves that match by the specificity of their patterns, and it records the shelves that lost. Where two shelves tie, it reports the tie and picks neither. [HW-OBL-0060](../obligations/0060-the-most-specific-shelf-wins-names-no-order.md) is open because no document names that order. Step 2 reads the kind from a homogeneous shelf, and step 3 reads a discriminator facet on a heterogeneous shelf. An unknown discriminator value stops the derivation, and the census keeps the row. [HW-OBL-0191](../obligations/0191-an-unknown-discriminator-value-removes-a-document-from-every-check-and-no-gate-reports-it.md) is open because no gate reports that row.

Step 4 is absent. Its input has no declared form in any schema, and an engine that invented one would own a declaration that the taxonomy owns. [HW-OBL-0059](../obligations/0059-kind-resolution-declares-a-fourth-step-that-no-syntax-supports.md) carries that finding.

### The census rows

`census::take` walks the root, and it gives each entry one `Row` with one `Outcome`. The outcomes are a closed set, and each match over them is exhaustive: `Typed`, `Generated`, `Untyped`, `Unreadable`, `Excluded`, `NotADocument` and `Unwalkable`. The census reports outcomes and never a severity. Whether an untyped document stops a run is the decision of a control.

`Untyped` and `Unreadable` are two outcomes on purpose. A file with no front matter is an ordinary corpus state, and a file whose block does not load is a defect. If the two were one outcome, a malformed file could hide inside the untyped count that OB-COV-1 reads. The parser draws that line already: `Reason::NoFrontMatter` is untyped, and each other reason is unreadable.

A row carries the document it parsed and a digest of the bytes it read. Graph build then reads the same parse, and a cache key names the same bytes that the run checked. A second read of the file could see a different file. The digest is over the bytes and not over the parse. Two documents that differ only where the parser discards text would otherwise share a key.

A file that carries the generated-file marker gets the outcome `Generated`. No author can repair its content in the file, so no check reads it, and `headwater generate --check` is the one verb that reports drift in it. A generated file that declares an identity still resolves a kind, so it can be a node. `Outcome::node` is the one definition of a node of the graph. The census cannot test the claim of the marker, because it does not read the projection declarations, and the generator tests it. [HW-OBL-0164](../obligations/0164-the-regeneration-check-tests-byte-equality-before-the-marker-so-a-file-the-census-stopped-counting-as-generated-still-passes.md) is open on the projection side of that row.

A file can be several things at once, and a row has one outcome. So the module comment of `census.rs` fixes a precedence of ten steps. A statement about who wrote a file outranks what the engine would derive from its content. Thus an exclusion and the marker come before the parse.

### Paths outside the corpus root

`outside.rs` keeps the paths outside the corpus root that a language regime lists ([HW-DR-0084](../decisions/0084-a-language-rule-reaches-front-door-prose-outside-the-corpus-root-and-no-other-rule-does.md)). They are not census rows. A row is a member of the denominator, and a file that no kind binds would count against OB-COV-1 as a document that nothing checked. So the census keeps a second list, and its report prints the list on a line of its own. A pattern that matches nothing stays in the list and is reported by name.

### The derived population

`derived.rs` computes which files of a tree a producer writes, from four producers and the rule of each. It holds no list, because each hand statement of that population was wrong ([HW-DR-0049](../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md)). It is in the census crate because it reads the walk and the generated-file marker. The [`headwater derived`](../interfaces/headwater-derived.md) contract states the population and its treatments, and this spec does not repeat them.

### The one crate that runs git

`vcs` is the only crate of the workspace that starts a version control command. The check path runs none, and it opens no file that the manifest does not name. So the manifest has to come from outside that path. Before this crate, a shell script of this repository was the only producer. [HW-DR-0072](../decisions/0072-the-binary-is-the-only-interface-an-adopter-must-run-and-every-integration-point-outside-it-is-declared.md) rules that an adopter reaches the whole loop through the binary, so the script moved into `produce`.

`produce` runs the git commands that the script ran: `rev-parse`, `diff --name-status`, `show` and `ls-files`. It reads no corpus and parses no document. A deleted document goes into the manifest with its prior version. [HW-OBL-0127](../obligations/0127-a-deletion-is-invisible-where-the-version-that-stood-there-does-not-parse.md) is open because a deletion reaches no rule where that prior version does not parse.

`ignored` gives the paths that git ignores. `taxonomy audit` and `derived.rs` read it, so git decides what it ignores and no second reader of ignore rules exists. `merge_attributes` asks git for the `merge` attribute of each path. `names_merge` asks whether any attributes file names that attribute for a path. Git reads `!merge` and no line as the same answer, so `names_merge` adds a probe file. It fails closed: a probe that did not match is an error and never an answer. Each of the three gives `None` outside a git work tree, and an error where git refused inside one.

## Invariants

A change to these crates must keep each of these. A test holds each one that names a test.

- **A document parses to the recorded shape, and a rejected one to the recorded errors** (`accepted_documents_parse_to_the_recorded_shape`, `rejected_documents_report_the_recorded_errors` in `engine/crates/doc/tests/fixtures.rs`).
- **Every document of this repository parses, except the ones a list records** (`the_corpus_parses_except_where_this_records_otherwise`), and that list and the census agree (`the_census_and_the_parsers_exception_list_agree` in `engine/crates/census/tests/fixtures.rs`).
- **The pathological tree walks to the recorded census** (`the_pathological_tree_walks_to_the_recorded_census`). Each rule of the walk has a case in that tree.
- **This repository takes the recorded census** (`this_repository_takes_the_recorded_census`), **in path order** (`the_census_rows_are_in_path_order`).
- **The derived population is computed and never listed.** The producer cases of `engine/crates/census/tests/fixtures.rs`, from `every_producer_output_is_declared_and_every_declared_path_has_a_producer` on, hold it.
- **A declared derived file refuses a text merge** (the eight cases of `engine/crates/census/tests/merge_driver.rs`).
- **The manifest is the one the shell script wrote** (`a_known_add_and_modify_produces_the_hand_written_manifest` and the other unit tests of `engine/crates/vcs/src/lib.rs`). **A probe that did not match is an error** (`a_probe_that_did_not_match_is_an_error_and_never_an_answer`).
- **The recorded census of the fixture tree and of this repository is the assertion.** Record it again with `HEADWATER_BLESS=1` and read the diff.

The parser, the sentence splitter, the census walker and kind resolution are correctness roots ([spec 12](../spec/12-check-layer.md#the-correctness-roots)). A wrong answer from any one of them causes no error, and spec 12 states what each one owes.
