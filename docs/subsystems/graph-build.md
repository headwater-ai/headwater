---
id: HW-SPEC-graph-build
status: current
status_since: 2026-09-30
summary: "How the graph crate indexes identifiers and paths, turns relation blocks into edges, binds links and anchors, and reports each miss without a severity."
last_verified: 2026-09-30
title: "Graph build"
relations:
  governs:
    - engine/crates/graph/src/**
  traces_to:
    - HW-SPEC-engine-architecture
    - HW-SPEC-conceptual-model
    - HW-SPEC-check-layer
    - HW-DR-0074
    - HW-IFACE-headwater-explain
---

# Graph build

## Scope

This spec describes the inside of the graph build stage of [spec 6](../spec/06-engine-architecture.md#pipeline). One crate builds that stage, and it is `graph` under `engine/crates/`.

The stage takes three inputs. The first is the census, with the parsed document of each row. The second is the relation and anchor declarations of the resolved lock. The third is the set of anchor resolvers over the corpus. The stage gives one value, `Graph`, to every consumer after it: checks, queries, projections, export and explain.

Other documents state what the stage does, and this spec does not repeat them:

- [Spec 6](../spec/06-engine-architecture.md#pipeline) states the promise of the stage, and [Nothing stores the graph](../spec/06-engine-architecture.md#nothing-stores-the-graph) states why every run builds the graph again.
- [Spec 1](../spec/01-conceptual-model.md#external-anchor) states what a relation, an external anchor and a prose link are.
- [HW-DR-0074](../decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md) states how a `code_path` pattern binds.
- The [`headwater explain`](../interfaces/headwater-explain.md) contract states what a reader sees of an edge.

The public Rust API of the crate is not in this spec ([HW-DR-0098](../decisions/0098-an-engine-subsystem-is-described-by-a-technical-design-spec-on-a-shelf-of-its-own-and-its-behavior-stays-where-it-is-already-written.md)). The doc comments of the crate state it.

## Design

### One build in four passes

`Graph::build` in `lib.rs` runs four passes in a fixed order. `Index::build` makes the indexes, `edges::build` resolves each `relations:` block, `links::bind` binds each prose link, and `Scope::reach` walks each declared governed scope. Each pass reads the rows of the census and opens no document file. A second read of the corpus could disagree with the census, and coverage is a function of the census and the graph together.

The census decides the node set. A row is a node when `Outcome::node` of the census crate gives it a kind. That set includes a generated document that declares an identifier. Spec 6 excuses a generated file from checks and not from identity. So a projection that other documents cite can be an end of an edge.

`Config` names the front matter key of the identifier (`id`, [HW-DR-0068](../decisions/0068-the-front-matter-key-that-carries-a-minted-identifier-is-id.md)) and of the relation block (`relations`, [Q4](../spec/09-decisions.md#q4--relation-storage)). Both are parameters and not literals, so a change to either declaration changes one default.

### Two indexes

`index.rs` holds an identifier index and a path index. Q4 makes an edge target an identifier and never a path, so edge resolution reads the identifier index. A prose link is a relative path, so link binding reads the path index. One index with both keys would accept a path as an edge target, which Q4 forbids.

The identifier index has two parts. `Index::typed` is the node set. `Index::untyped` holds each other document that declares an identifier, and nothing in it is a node. It lets a report tell a target that is a document with no kind from a target that is nothing. The first is a defect of the target document, and the second is a defect of the link. So each report goes to the author who can repair it.

`Index::near_miss` answers the second question: it finds the document with no kind that carries the identifier an edge names. The path index holds every row of the census. So a link to a file that the corpus excludes says so, and it is not reported as broken. `Defect` records what the index could not make of a document: no identifier, an identifier that is not a scalar, or a duplicate.

### Declarations

`declarations.rs` reads the `relations` and `anchors` declarations of the lock. It reads only the fields that edge resolution needs, and it validates nothing, because the meta-schema owns the shape. It refuses a relation that declares no `to`, once, against the declaration. Otherwise every target of that relation reports the same defect.

It reads `inverse` because an author can write either half of a pair. It carries `from`, `reciprocal` and `sets_target_state` for the checks, and resolution never reads them. A relation is lifecycle-sensitive when its own declaration says so or when `core.requires` marks its family. The crate reads `core.requires` through `headwater_resolve::core` and does not read it a second time.

### Edges

`edges::build` reads the `relations:` block of each node. An entry is a scalar target or a mapping with `to:` and instance attributes, and both produce the same edge ([spec 1](../spec/01-conceptual-model.md#the-authored-form)). One entry and a sequence of one entry are also the same.

The identity of an edge is the triple of source identifier, relation name and normalized target (Q4). List order carries no meaning, so nothing downstream reads it. A repeated triple in one document is a `Problem::RepeatedTriple`. A document with no identifier gets one `Problem::SourceHasNoIdentifier`, and not one report per target.

An edge carries the direction the author wrote and the direction the taxonomy declares. `Edge::declared_triple` gives both halves of a reciprocal pair the same triple. A reciprocity check then finds the missing half and does not need to know which name the author used.

The declaration decides whether a target is a document or an anchor. Where the `to` set holds a kind and an anchor kind, the index goes first and the resolvers after it. No document states that order, so [spec 13](../spec/13-open-obligations.md) carries it. `Target` has four variants: `Document`, `Anchor`, `Withheld` and `Unbound`. `Unbound` names one of six reasons. The target is nothing, or it is a document with no kind. No resolver claims it, or its anchor did not resolve. Two anchor kinds claim one string, or the target is the path of a typed document.

The last reason is `Unbound::DocumentByPath` (#1410). A literal path to a document that has a kind and an identifier binds nothing, and the report names the identifier to write. An anchor onto that file would give the document a second name.

### Anchor resolvers

`anchors.rs` defines the `Resolver` trait. One resolver owns the identity of one anchor type. Each binding goes through `Resolver::resolve_for`, which also receives the identifier of the document that declares the edge. No resolver opens a socket, because check time stays offline and a result stays reproducible.

`Resolvers::over` gives a run the `source-tree` resolver alone. A caller adds other resolvers with `Resolvers::with`, which refuses a second resolver of one name. The snapshot resolver and the harvest resolver live in the `import` crate, because that crate depends on this one.

`SourceTree` first calls `normalize`, which is lexical. It changes `\` to `/`, removes `.` segments and a trailing `/`, and lets `..` remove the segment before it. It refuses an absolute path, an empty path and a path above the base. A path library that resolves `..` on the disk follows a symbolic link out of the repository. Two strings would then be one node for a reason that the corpus does not state. [HW-OBL-0061](../obligations/0061-an-anchor-resolver-normalizes-and-nothing-states-how.md) is open because no document states these steps as a rule. This paragraph describes the code and does not make it the rule.

A literal pattern binds when its path exists. It binds to an excluded file as well, and it carries the exclusion that matched. A pattern with a wildcard walks the subtree under its literal prefix. An excluded entry is not a match there, and a pattern that opens on a wildcard is refused. `SourceTree` walks each prefix once per process and keeps no walk from one run to the next. `tree_revision` gives an anchor a revision, which is a digest of the entries it matched.

`CommentScan` binds an edge when the comments of the target file cite the identifier of the document that declares the edge (#967). It reads only identifiers that have a claim file under `.headwater/ids/` ([HW-DR-0054](../decisions/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md)). The resolvers exist before the index does, so the claim store answers the question the index cannot answer yet. `comments.rs` finds the comment bytes of a Rust file. It walks byte by byte through four states, so a `//` inside a string is not a comment. The check crate calls the same function, so the walk has one copy.

### Prose links

`links::bind` binds every prose link to what it points at, and no link becomes an edge ([spec 1](../spec/01-conceptual-model.md#prose-links-are-not-relations)). A link inside a block quote is counted and never bound, because it is a claim of another author. A destination binds when the raw text or the percent-decoded text names a file, and it is broken only when neither does. A query string comes off before the path is bound, and a leading `/` is the root of the corpus. The fragment stays on the record and is not resolved, because a heading is the business of a check.

### Governed scope

`scope.rs` is the one reader of `anchors.<kind>.scope`. `Scope::contains` tests one path with no walk, `Scope::unmatched` finds each pattern that matches no entry, and `Scope::reach` lists what each pattern admits. The last two go through the resolver of the anchor kind, so a scope and an anchor use one matcher. A scope counts only files that git does not ignore, and a root with no tree beside it counts nothing. `Graph::scope` is empty when the taxonomy declares no scope.

## Invariants

A change to this crate must keep each of these. A test holds each one that names a test, in `engine/crates/graph/tests/fixtures.rs` or in the unit tests of the module.

- **The stage decides identity and nothing about legality.** It writes no severity. Whether a missing target stops a run is the decision of a check.
- **Every node is a census row that resolved a kind.** The stage does no second walk of the corpus (`every_node_of_the_graph_is_a_row_of_the_census_that_resolved_a_kind`).
- **Two spellings of one anchor are one node** (`two_spellings_of_one_anchor_are_one_node`). `normalize` stays lexical.
- **Both halves of a reciprocal pair give one triple** (`both_halves_of_a_reciprocal_pair_normalize_to_one_triple`), and each edge half falls in exactly one class (`every_edge_half_falls_in_exactly_one_class`).
- **A target with no kind and a target that is nothing are two reports** (`an_untyped_target_and_a_target_that_is_nothing_are_two_reports`).
- **A quoted link is counted and never bound** (`a_quoted_link_is_counted_and_never_bound`).
- **An edge onto a list anchor binds when each of its patterns matches at least one entry** ([HW-DR-0074](../decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md)). A literal path to an excluded file still binds (`an_anchor_inside_an_exclusion_resolves_and_names_the_rule`).
- **No resolver opens a socket, and one name has one resolver.** The walk cache of `SourceTree` never outlives one process.
- **The relation names that a document can write and the names that a template can write are one set** (`engine/crates/graph/tests/vocabulary.rs`).
- **The recorded graph of the fixture tree and of this repository is the assertion.** Record it again with `HEADWATER_BLESS=1` and read the diff.

The anchor resolver and the identifier index are correctness roots ([spec 12](../spec/12-check-layer.md#the-correctness-roots)). A wrong answer from either one causes no error, and every check over the graph passes on the wrong graph. [HW-OBL-0064](../obligations/0064-two-identity-components-are-correctness-roots-that-spec-12.md) records that no fixture set is asked of either one yet.
