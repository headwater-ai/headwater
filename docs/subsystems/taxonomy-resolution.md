---
id: HW-SPEC-taxonomy-resolution
status: current
status_since: 2026-09-30
summary: "How six crates load taxonomy sources, prove that overlays commute, merge them, resolve references last, and write the content-hashed lock."
last_verified: 2026-09-30
title: "Taxonomy resolution"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft+revise
  evidence_basis: evidenced
relations:
  governs:
    - engine/crates/yaml/src/**
    - engine/crates/ref/src/**
    - engine/crates/meta/src/**
    - engine/crates/resolve/src/**
    - engine/crates/lock/src/**
    - engine/crates/hash/src/**
  traces_to:
    - HW-SPEC-engine-architecture
    - HW-SPEC-taxonomy-model
    - HW-SPEC-distribution-and-federation
    - HW-SPEC-check-layer
    - HW-DR-0002
    - HW-IFACE-headwater-taxonomy
    - HW-DR-0040
---

# Taxonomy resolution

## Scope

This spec describes the inside of the resolve stage of [spec 6](../spec/06-engine-architecture.md#pipeline). Six crates under `engine/crates/` build that stage: `yaml`, `ref`, `meta`, `resolve`, `lock` and `hash`.

The stage has two inputs. The first is the base package under `.headwater/packages/`, with the bundles that the consumer selects. The second is the adopter overlay, `.headwater/overlay.yml`. The stage gives a `Resolution`, and it writes that result as the content-hashed `Lock` in `.headwater/taxonomy.lock`. Every later stage reads the lock and never the sources.

Other documents state what the stage does, and this spec does not repeat them:

- [Spec 6](../spec/06-engine-architecture.md#pipeline) states the promise of the stage.
- [Spec 2](../spec/02-taxonomy-model.md#customization-by-composition) states the overlay operations, their preconditions, confluence, the `$`-reference sublanguage and the immutable core.
- [Spec 7](../spec/07-distribution-and-federation.md#publishing) states how a package is published, pinned, vendored and migrated.
- [Q2](../decisions/0002-schema-format.md) states the dialect of a taxonomy source, and [Q22](../decisions/0022-q22-the-integrity-posture-of-a-published-package.md) states the integrity posture of a published package.
- The [`headwater taxonomy`](../interfaces/headwater-taxonomy.md) contract states what an author sees of each verb.

The public Rust API of the crates is not in this spec ([HW-DR-0098](../decisions/0098-an-engine-subsystem-is-described-by-a-technical-design-spec-on-a-shelf-of-its-own-and-its-behavior-stays-where-it-is-already-written.md)). The doc comments of the crates state it.

## Design

### A loader on the Q2 dialect

`yaml` is the only crate that knows YAML. `loader.rs` reads an event stream and gives a tree with a span on every node. A syntax error stops the load, because no tree exists to say more about. The loader collects every other refusal, so an author who wrote three anchors reads three lines in one run.

The loader refuses a duplicate key, an anchor, an alias, a merge key and an explicit tag. A `$`-reference is the one reuse mechanism, and an overlay cannot address an alias. `value.rs` has no `Null`, `Bool` or `Int` variant. A scalar keeps its text and its style, because the meta-schema and not the YAML resolver decides a type. `core_schema.rs` gives that type only where a declared type asks for it. So `no` stays the string `no`.

`span.rs` writes each position in the coordinates of an editor: lines and columns from one, and byte offsets into the source. Spec 12 anchors each finding to a line, and this is where that line comes from. `json.rs` holds the JSON writer that three other crates share. Its reader is the loader, because JSON is a subset of the YAML 1.2 core schema.

### The `$`-reference sublanguage and the overlay address

`ref` makes one path production executable. An address is written where only an address is legal: a key under `add:`, `override:` and `add_to:`, and an entry of `remove:`. A reference is written in value position, where a literal is also legal, and the `$` sigil tells the two apart. The root set of a reference is closed to `vocabularies` and `package`.

`ref` parses and does not resolve. `Reference::parse` says that a text is well formed, and it says nothing about whether the node exists. Each refusal carries a byte offset, so a finding puts the caret under the segment that is wrong. `Address::is_disjoint_from` is the predicate that confluence uses.

### The meta-schema, in its own dialect

`meta` reads `meta-schema.yml`, which ships with the engine. The meta-schema is a YAML source on the Q2 dialect, and `yaml` loads it as it loads a taxonomy source. JSON Schema cannot resolve a `$`-reference, and a second schema language would drift from the first.

`shape.rs` reads the shape language: eight forms and two modifiers. `schema.rs` turns an overlay address into the shape at the node that the address names. `validate.rs` runs the four checks that one source can decide. They are structural conformance, reference well-formedness, the reserved `package` root, and the rule that an address never reaches into a list. `validate::skipped` names each check that needs a resolved tree, so a caller never reports a pass that it did not run.

Two error types exist, because two different persons read them. A `SchemaError` is a defect of `meta-schema.yml`. A `MetaError` is a finding about the source of an adopter, with a span.

Ten positions of the meta-schema mark a required declaration that has no stated form ([HW-OBL-0043](../obligations/0043-two-meta-schema-surfaces-have-a-required-declaration.md)). Eight value sets are closed to the values that this corpus writes ([HW-OBL-0049](../obligations/0049-the-meta-schema-closes-eight-value-sets-that-nothing-states.md)). The meta-schema has one notion of a required member, and an overlay that supplies a required declaration needs a second ([HW-OBL-0033](../obligations/0033-a-required-declaration-that-an-overlay-may-supply.md)). The code shows the current reading at each of these points, and each obligation holds the open question.

### Two small languages beside the schema

`pattern.rs` is the glob language of a shelf path and of a corpus exclusion. It has four forms: `**` as a whole segment, `*`, `?` and a literal. Specificity orders two patterns first by the count of leading literal segments, and then by the count of literal characters. `Pattern::overlaps` asks whether any path matches both patterns. The determinism rule asks that question with no corpus, so a collision of two shelves is reported before somebody writes the file that shows it.

`identifier.rs` is the template language of an `identifier_scheme` pattern: `{namespace}`, `{slug}` and `{seq:0Nd}`. A `{slug}` is read only in terminal position, because a free token before another segment has two readings. That restriction makes `Template::disjoint` exact in both directions. The grammar is in `meta` because `resolve` and `check` both read it, and `resolve` cannot see `check`.

### The resolver runs in five steps, and the order is deliberate

`resolve/src/lib.rs` runs five steps in this order:

1. Validate every source against the meta-schema (`source.rs`). No partial load exists.
2. Check confluence, statically, before anything merges.
3. Apply every operation. Each one asserts a precondition about the tree.
4. Resolve every reference, over the merged tree.
5. Check the immutable core, on the result.

The obvious order resolves a reference when it reads the source. That order freezes the base value of `vocabularies.lifecycle_state` before an overlay that overrides it is read. The override then changes nothing that a check can see.

### Confluence is static

`confluence.rs` reads the operations and never the tree, so its answer does not depend on the base. A publisher can then prove before a release that every subset of its bundles resolves. The check runs over pairs from two different sources, because the file orders two operations of one source.

Two `add` operations commute when no leaf that one writes is a prefix of a leaf that the other writes. Every other pair owns its whole subtree, and `Address::is_disjoint_from` decides it. One pair is ordered and not tested: a library entry that writes into an entry it names in `requires` ([HW-DR-0095](../decisions/0095-q67-one-library-entry-may-address-the-keys-of-an-entry-it-names-in-requires-and-confluence-holds-over-the-dependency-order.md)). `order.rs` puts the dependency first. It refuses a cycle among the selected bundles, and a dependent write whose dependency the selection lacks.

### Operations with preconditions

`operation.rs` reads an overlay as operations of five kinds: `add`, `override`, `add_to`, `remove_from` and `remove`. It computes the set of leaves that each `add` writes. The addressed path is the wrong input to the confluence predicate. The design-spec bundle adds `kinds.design_spec`, and the adopter overlay adds `kinds.design_spec.identifier`. The two addresses overlap, and the written leaves do not.

`merge.rs` applies each operation and returns a new tree. A failed operation therefore leaves no half-applied tree. An `add` asserts that the base does not declare each leaf. It does not assert that another overlay has not made the parent mapping. Whether a bundle may extend a list with `add_to` is an open question ([HW-OBL-0031](../obligations/0031-list-extension-in-an-add-only-overlay.md)), and this spec states no answer.

### References resolve last

`references.rs` collects every substitution against the merged tree before it writes one. So no reference reads a value that another reference put there. A scalar is a reference only where the shape at its position carries `reference: allowed`. Elsewhere, `$` is an ordinary character. In a position that admits a reference, `$$` is one literal `$`.

### The core is checked on the result

`core.rs` reads the resolved taxonomy and never an operation. An overlay can rename every shelf and every state, and it passes. A requirement names a role or a purpose, never a name, so the satisfier of a `facet_role` requirement is the facet that carries the role. The refusal names the operation that removed the last satisfier, so the author does not search thirty lines. A relation family is lifecycle-sensitive when a member declares `lifecycle_sensitive: true` or declares `on_target: {set_state: ...}`.

### Validation of the result

`rules.rs` runs the checks of `headwater taxonomy validate` that read a resolved tree. `RULES` lists every item of the list in spec 2 and where it runs. `Ran::Partly` marks a rule that this code decides in part, because the other part needs a declaration that the language does not have. A finding here names an address and no line. A merged tree does not know which source declared a node, and an overlay writes the same address.

### Selection, assembly and templates

`package.rs` reads the consumer declaration and the package manifest, and it gives the resolver an ordered list of sources. The manifest and the taxonomy source are two files, because `package` is a reserved reference root. `selection.rs` finds the bundle that a consumer left out. It derives the answer from the same traversal as the refusal, and it does not read `requires` ([HW-DR-0040](../decisions/0040-q40-whether-extends-bundle-requires-and-an-overlay-s-taxonomy-key-are-a-mechanism-or-a-label.md)). `assembly.rs` resolves a named selection of the bundles of one package.

`template.rs` holds each template of a package against the taxonomy of that package. `publish` calls it, because only that verb sees every bundle a package ships. A placeholder is skipped and not typed. A template with a kind name and no readable front matter is refused, so that no publisher has a bypass.

### Flatten, render and order

`flatten.rs` renders a named assembly as one package with no bundle selection at run time. It returns bytes and writes no path, so a test can hold the equality before publication writes a byte. `render.rs` writes a resolution back as text in one canonical form. It keeps the declaration order, and it writes a scalar plain where every character allows it and double-quoted otherwise. The round-trip fixture reads that text again, and the lock hashes it.

### Publishing, release and migration

`release.rs` holds the digest of a published package. The digest proves that the artifact on disk is the artifact that the consumer pinned. It does not prove who published the artifact ([HW-OBL-0115](../obligations/0115-a-pinned-digest-authenticates-the-pin-and-never-the-publisher.md)). `vendor` reads the name, the version and the engine range from `package.yml`, which the digest covers, and not from the header of the record. No crate of this subsystem opens a socket, and `headwater-fetch` turns a location into a directory before `vendor` reads it.

`migration.rs` reads the migration payload of a major version. The split between a mechanical step and a judgment step is derived from the target list of each step. One target is mechanical, two or more are a closed choice, and none is a re-statement. A `task:` on a mechanical step is refused, not dropped. The publisher holds the target half against the base with every bundle, and the source half against the base alone.

### The lock

`lock` writes one resolution with a digest over it. The identity of a resolution is its canonical text, and `render` makes that text the same for every legal order of an overlay set. A reader can compute that identity from the file in front of them, which a tree identity does not allow.

`write` runs every rule of `taxonomy validate`, and it returns the findings instead of a lock when a rule fires. So a lock is always a validated taxonomy. `read` checks the format, the digest and the `rules` field against the rule set of the engine, and it runs no rule. A lock that another rule set validated is refused.

The `adoption` block is the one authored part of the lock. `taxonomy resolve` carries it through unchanged, and the digest does not cover it. `rewrite_adoption` replaces the block and does not resolve again. `authored_at` reads the block when the rest of the lock does not read. `diverged` names the block only when every line outside it is identical. No document states which lines are authored and which are generated ([HW-OBL-0082](../obligations/0082-the-lock-is-half-generated-and-half-authored-and-nothing.md)), so this paragraph describes the code and does not make the rule.

### One SHA-256

`hash` is FIPS 180-4 SHA-256 in about eighty lines, with no dependency. The lock, the census and the check cache hash, and they must agree on the digest of one byte string. So one implementation exists, and no other crate carries a second. `digest` writes the `sha256:` prefix, so a later engine can change the function and name it. [Q22](../decisions/0022-q22-the-integrity-posture-of-a-published-package.md) records why the package digest uses this function and not a vetted crate.

## Invariants

A change to these crates must keep each of these. Each item names the test that holds it. That test is in the `tests/` directory of the crate or in the unit tests of the module.

- **The loader keeps the Q2 dialect.** Each accepted source loads to its recorded tree (`accepted_sources_load_to_the_recorded_tree`), and each rejected source reports its recorded errors (`rejected_sources_report_the_recorded_errors`). The sources of this repository load (`this_repositorys_own_sources_load`).
- **One grammar reads every address and every reference** (`addresses_parse_to_the_recorded_segments`, `references_parse_to_the_recorded_root_and_address`, `scalars_classify_to_the_recorded_reading`). Disjointness is recorded for each pair (`address_pairs_record_which_subtrees_overlap`).
- **A source validates to its recorded verdict** (`taxonomy_sources_validate_to_the_recorded_verdict`, `overlay_sources_validate_to_the_recorded_verdict`), and the meta-schema reads to its recorded shapes (`the_meta_schema_reads_to_the_recorded_shapes`).
- **No order of an overlay set changes the result** (`no_permutation_of_an_overlay_set_changes_the_result`), and no order changes the digest (`every_order_of_an_overlay_set_produces_one_digest`).
- **A resolved taxonomy resolves to itself** (`a_resolved_taxonomy_resolves_to_itself`), and each case resolves to its recorded result (`every_case_resolves_to_the_recorded_result`).
- **Each rule that runs in `rules.rs` has a case that fails it** (`every_rule_that_runs_here_has_a_case_that_fails_it`).
- **Each overlay of this repository reaches a declaration that is there** (`every_overlay_of_this_repository_reaches_a_declaration_that_is_there`).
- **A declared dependency applies first** (`every_order_of_a_selection_writes_one_lock_and_lists_each_dependency_first`, `the_shipped_set_applies_each_dependency_before_its_dependents`), and a cycle names each bundle in it (`a_cycle_of_three_names_each_bundle_in_it_and_no_other`).
- **The digest is over the canonical text and not the file** (`the_digest_is_over_the_canonical_text_and_not_the_file`), and the committed lock is what the sources resolve to (`the_committed_lock_is_what_the_sources_resolve_to`).
- **A lock from another rule set or format is refused** (`a_lock_whose_rules_field_is_stale_is_refused_and_names_the_remedy`, `an_old_format_lock_is_refused_without_saying_which_engine_is_newer`).
- **The SHA-256 is correct.** It matches the published vectors (`the_published_vectors`) and each length around a block boundary (`every_length_around_a_block_boundary`), and a second implementation agrees on every subject (`a_second_implementation_agrees_on_every_subject`).

The overlay resolver and the lock are correctness roots ([spec 12](../spec/12-check-layer.md#the-correctness-roots)).
