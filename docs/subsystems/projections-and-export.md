---
id: HW-SPEC-projections-and-export
status: current
status_since: 2026-10-01
summary: "How two crates plan each derived file, write or compare it under the generated-file marker, and audit an export with a loss census."
last_verified: 2026-10-01
title: "Projections and export"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft+revise
  evidence_basis: evidenced
relations:
  governs:
    - engine/crates/generate/src/**
    - engine/crates/mark/src/**
  traces_to:
    - HW-SPEC-engine-architecture
    - HW-IFACE-headwater-generate
    - HW-IFACE-headwater-export
    - HW-IFACE-headwater-derived
    - HW-IFACE-headwater-site
    - HW-DR-0063
    - HW-SPEC-distribution-and-federation
    - HW-SPEC-authoring-and-lifecycle
    - HW-DR-0036
    - HW-DR-0038
---

# Projections and export

## Scope

This spec describes the inside of the projections stage and the export stage of [spec 6](../spec/06-engine-architecture.md#pipeline). Two crates under `engine/crates/` build these stages: `generate` and `mark`.

The stages have four inputs. They are the census of the corpus, the graph and its query surface, the lock, and the `projections` block that the lock carries. The stages give three outputs:

- the derived files that `headwater generate` writes and that `headwater generate --check` holds to regeneration,
- the payload that `headwater export` writes, with the loss census over it,
- the verdict of `headwater site` on a built site, against the plan of the corpus that built it.

Other documents state what the stages do, and this spec does not repeat them:

- [Spec 6](../spec/06-engine-architecture.md#projections) states the projection kinds, the marker rule, the loss set and the export profile.
- [Spec 7](../spec/07-distribution-and-federation.md#arriving-at-a-corpus-cold) states what the corpus descriptor carries and why its path is fixed.
- [Spec 3](../spec/03-authoring-and-lifecycle.md) states the warrant of a generated document.
- [HW-DR-0063](../decisions/0063-every-required-facet-of-a-generated-document-is-derived-and-the-emitter-composes-the-summary.md) states why the engine derives each required facet of a generated document.
- [HW-DR-0036](../decisions/0036-q36-which-of-mkdocs-docusaurus-or-astro-this-corpus-emits-navigation-for-and-why.md) states the navigation format, and [HW-DR-0038](../decisions/0038-q38-which-link-relation-a-rendered-page-carries-to-the-served-corpus-descriptor.md) states how a rendered page links to the descriptor.
- The [`headwater generate`](../interfaces/headwater-generate.md), [`headwater export`](../interfaces/headwater-export.md), [`headwater site`](../interfaces/headwater-site.md) and [derived artifacts](../interfaces/headwater-derived.md) contracts state what an author sees of each verb and each artifact.

The public Rust API of the crates is not in this spec ([HW-DR-0098](../decisions/0098-an-engine-subsystem-is-described-by-a-technical-design-spec-on-a-shelf-of-its-own-and-its-behavior-stays-where-it-is-already-written.md)). The doc comments of the crates state it.

## Design

### The marker crate sits below both of its readers

`mark` holds the generated-file marker: one wording and one predicate. Two crates read it. `generate` writes the marker and reads it before an overwrite. `census` reads it to put a generated file in the census as generated and not as an authored document. `generate` depends on `census`, so `census` cannot depend on `generate`. A copy of the rule in each crate would let the writer and the reader disagree about which files this engine wrote. So the rule is in `mark`, below both, in the same way that `hash` holds one digest.

The marker is a claim that this engine wrote the file, and it is not proof. Any author can type the line. `kind_named` reports what the marker says and does not check the name against a declared kind. The test of the claim needs the plan, so `generate` makes it (see the next section). A corpus that never runs `generate --check` has no test of a marker ([HW-OBL-0038](../obligations/0038-a-corpus-that-never-runs-generate-check-has-no-test.md)).

The format decides where the marker is. A format with comments carries it on the first line. JSON has no comment, so a JSON output carries it as a top-level member. A Markdown document with front matter carries it as a member of the block. A comment above the block moves the block off line 1. `carries_marker` reads the marker only in the one region that the format admits. So a document that quotes the marker in its prose is still an authored document. `marks_format` names the formats that the census can read a marker in.

### Plan first, then write or compare

`plan` builds a `Plan` from the query surface, the census, the declared `Projections`, the corpus identity, the committed probe runs and the verb list. It reads no file of its own, so two plans over one tree hold the same bytes. A plan holds each output with its bytes, each declaration that this engine does not write with its reason, and each refusal. A `site_nav` declaration reads the outputs of every other declaration, so `plan` runs it last.

`orphaned` reads the census beside the plan. A marked file inside the corpus root that no output claims is `Orphaned`. It is a file from a declaration that was removed or moved, or a marker that a person typed. Without this test, the marker is an exemption that any author can add, because the census excuses a generated file from every document check.

`write`, `publish` and `check` each call one function, `run`, in a different mode:

- `write` is `headwater generate`. It writes each committed output.
- `publish` is `headwater export`. It also writes an output that states `committed: false`, and no other writer does.
- `check` is `headwater generate --check`. It compares each committed output with the file on disk and writes nothing.

`run` first skips an output whose own bytes carry no marker that the census can read (`MarkerUnread`). The census could not find such a file again after its declaration moves, and this verb could not overwrite it again. Then `run` reads the file on disk and gives one verdict. The verdict compares bytes first. Equal bytes give `Unchanged`, and the marker is not read. Unequal bytes and no marker give `Occupied`, and the file stays as it is, because a file without the marker is an authored file. Unequal bytes and a marker give `Differs` under `--check`, or a write. Because equal bytes come first, a committed file that lost its marker still passes `--check` ([HW-OBL-0164](../obligations/0164-the-regeneration-check-tests-byte-equality-before-the-marker-so-a-file-the-census-stopped-counting-as-generated-still-passes.md)). This paragraph describes the code and does not make the rule.

`run` knows every verdict before it writes a byte. Where `Report::refuses_before_writing` names a refusal, `generate` writes nothing and marks each pending output `Withheld`. So a refusal from a later output cannot follow a write that an earlier output made.

### One run of `generate` is several passes

One output can feed another. A generated document is a node of the census, and an index lists it. So `write_settled` plans from the tree, writes, and plans again, up to `PASSES` times. It stops at the first pass that writes nothing, and that pass is the proof that `--check` holds the result. A run that still writes on the last pass reports `unsettled` and fails. A pass reads the tree through the same loader as every other run, so no declaration has to state what each emitter reads and writes. The report states each path once, with the verdict of the first pass that changed it.

### Thirteen kinds, two value sets, and the unbuilt set

`Kind` names thirteen projection kinds. Eleven are declarable, and a taxonomy names the kind and its output path. Two are engine-defined: the register and the corpus descriptor. The engine fixes the path of each, because a reader who must read the taxonomy to find one already knows what it says. `Kind::declarable` separates the two sets.

`unbuilt` gives the reason that this engine writes no output for a kind, or nothing where it writes one. The reason is a property of the engine, so a run prints it for a kind that no declaration names. The match is exhaustive, so a new kind fails to compile until it has an answer. `engine_defined` lists the register as unwritten with its reason from `unbuilt`. The register depends on the clock, so a committed copy could not pass `--check` ([HW-OBL-0037](../obligations/0037-the-register-is-not-a-projection-of-the-lock-alone-so-generate.md)).

No output states when it was generated, because `--check` compares bytes. `headwater export --at` injects a time into an output that leaves the repository, and `plan` passes none.

### A generated document: its identity block and its derived facets

A projection that writes a document at the path of a document carries an `identity` block (`identity.rs`). The block is closed at three scalars: the identifier, the kind and the name. A member that took prose would move prose into the taxonomy source, which no census row covers and no language regime binds. The shelf that claims the output path decides how the kind is written. A heterogeneous shelf takes the discriminator. A homogeneous shelf takes no facet, and the declared kind must agree with the shelf. The edges of the document are the reciprocal halves of the edges that other documents declare into it, and no other edge.

`derived.rs` writes every other required facet of the kind, as HW-DR-0063 rules. It finds each facet by its role and never by its name. Each value is a function of committed bytes. A date folds the dates of the documents that the projection read, and it never reads a clock or git. So one pass gives the same answer as every later pass. The date that a state was entered folds to the newest input, and freshness folds to the stalest input. The emitter composes the summary and hands it to this module. `derived.rs` holds a computed state against the lifecycle regime of the kind of the document, because no check reads a generated document.

### The corpus descriptor

`descriptor.rs` writes `.headwater/corpus.json` at the path that the engine fixes. The root, the identity and the lock hash come from the consumer declaration and the lock. They arrive as strings, so `generate` does not depend on the resolver. An entry point is derived. It is the first document of the reading order of a shelf, from the same `by_precedence` call that `route` and the shelf index make. So the three cannot disagree about which document a reader opens first. An entry point carries no summary, because a wording edit then fails the gate. An export profile states its name, target, output and grain, and never its filter. The descriptor also states each exclusion of the root, with its reason. The marker is a top-level member, which is also the member that tells a reader that the file is a descriptor.

`emitters.rs` holds `EMITTER_SET`, a number that identifies the emitters of this build. `write` records it in the descriptor. `check` reads the committed number before it compares bytes. On a difference it says that two engines wrote the two sides, and it gives no remedy, because it cannot tell which side moved. A person raises the number by hand, and nothing catches a raise that was skipped ([HW-OBL-0074](../obligations/0074-a-check-version-is-raised-by-hand-and-nothing-catches-a-stale.md)).

### Indexes and pages

- **Shelf index** (`shelf_index.rs`). One bullet for each document on a shelf, in the reading order of the corpus, from `by_precedence`. `{shelf}` in an output path is the literal prefix of the path glob of the shelf. That reading is the engine's, and no document states it ([HW-OBL-0045](../obligations/0045-what-shelf-expands-to-in-a-projection-output.md)).
- **Shelf sections** (`shelf_sections.rs`). One heading for each document, so a reader can cite one row. The heading is the facet in the `name` role. A document with no name gives no file for its shelf, and so do two documents with one name. The plan names the document that stopped the file.
- **Site navigation** (`site_nav.rs`). One file in the shape of the MkDocs `nav:` key, over every shelf. Each path is relative to the corpus root, and each scalar is quoted. A group opens with the generated index of its shelf, which `site_nav` reads from the paths that the rest of the plan wrote. `Plan::navigation` keeps the same list for `headwater site`.
- **Verb index** (`verb_index.rs`). One row for each verb in the dispatch table of the binary, joined to the contract whose name is the command line. A verb with no contract has a mark in its last column. A contract that names no verb, or that has no name, gives no file.
- **Consumer surface** (`consumer_surface.rs`). The `surface` block of the taxonomy as a page that an adopter reads ([HW-DR-0077](../decisions/0077-the-consumer-surface-is-what-an-adopter-receives-runs-and-must-have-installed-and-it-is-a-closed-and-declared-list.md)). A taxonomy with no block gives no page, and the plan reports it unwritten.

### Export: the filter, the emitter, then the census

`profile.rs` reads an export profile from the `projections` entries that name it. A name groups entries, and an entry that names none is in `default`. Two entries of one profile that declare two different filters are refused, because one audience cannot see two sets of documents. `Emitter` names each target that a profile can declare. A target that this engine does not build refuses and names what it waits on.

`emit` runs three steps in a fixed order. `withhold` applies the filter first, so that an emitter never sees a document that the profile withholds. The emitter runs second and reports what it carried. `audit` runs last, over the graph and not over the account that the emitter gives of itself. Each node and each edge must be in the output or have a declared loss reason. A drop that no reason covers fails the run. The node set is every classified document and every external anchor, so a document with no identifier still counts. A scalar leaves as the string that the document wrote, so the native export round-trips without a loss. An identifier scheme names no prefix, so a document IRI in an RDF target comes from a file path ([HW-OBL-0141](../obligations/0141-an-identifier-scheme-names-no-prefix-so-an-rdf-projection-derives-a-document-iri-from-a-file-path.md)).

### The probe result

`probe_result.rs` writes one result document for each committed transcript. The result comes from the transcript, the probes and the grader version, and it reads no clock. `{run}` in the output path is the file stem of the transcript. The grader runs inside `--check`, and the gate compares derivations and never the behavior of a model. A low rate passes. A result that does not agree with its inputs fails. A transcript that a confirmation refuses is a `RefusedTranscript`. Where the state of the transcript holds the refusal, the whole run refuses before its first write. Where the state releases it, the result is written, and it holds the refusal and no verdict.

### `headwater site`

`site.rs` reads a built site directory and the plan, and it writes nothing. It knows one layout: a source `a/b.md` is served at `a/b/index.html` or at `a/b.html`. It reports three defects. A page that the navigation names is not in the build. A page is left from a document that the corpus does not hold. A link or a fragment reaches nothing. The navigation comes from `Plan::navigation` and not from the committed file, because a stale committed file is the finding of `--check`. It scans only `href` and `id` over the bytes, so it needs no HTML parser.

## Invariants

A change to these crates must keep each of these. Each item names the test that holds it. That test is in the `tests/` directory of the crate or in the unit tests of the module.

- **The marker is read in one region for each format** (`a_commented_format_carries_the_marker_on_the_first_line_and_nowhere_else`, `json_carries_the_marker_as_a_member_wherever_it_sits`, `json_on_one_line_does_not_carry_the_marker_inside_a_string_value`, `a_markdown_document_carries_the_marker_inside_its_front_matter_block`), and the kind reads back out of the line the writer wrote (`the_kind_reads_back_out_of_the_line_the_writer_wrote`).
- **Every output carries its own marker** (`every_output_carries_its_own_marker`), and the census reads it as generated (`every_output_the_fixture_writes_is_censused_as_generated`). An output whose marker the census would not read is not written (`an_output_whose_marker_the_census_would_not_read_is_not_written`).
- **An authored file at an output path is not overwritten** (`the_descriptor_path_obeys_the_marker_rule`).
- **A marked file that nothing writes is orphaned** (`a_generated_file_is_censused_as_generated_and_orphaned_when_nothing_writes_it`, `a_graph_export_left_behind_by_a_repointed_declaration_is_orphaned`).
- **The fixture tree generates its recorded projections** (`the_fixture_tree_generates_the_recorded_projections`, against `fixtures/generate.record`), and two plans over one tree agree (`two_plans_over_one_tree_agree`).
- **This repository generates what it declares and accounts for the rest** (`this_repository_generates_its_sixty_artifacts_and_accounts_for_the_rest`).
- **One run of `generate` settles** (`one_call_after_adding_a_document_leaves_a_tree_the_check_accepts`, `a_path_two_passes_changed_is_reported_once`), and a run that never settles fails (`a_run_that_never_settles_says_so_and_fails`).
- **A refusal known before the first write leaves the tree unchanged** (`every_refusal_known_before_the_first_write_leaves_the_tree_unchanged`, `a_refusal_on_a_later_pass_does_not_claim_the_run_wrote_nothing`).
- **Spec 6 names the kinds this engine declares and emits** (`the_projection_kinds_block_of_spec_6_names_the_thirteen_kinds_this_engine_declares`, `the_runs_group_of_spec_6_is_exactly_the_projection_kinds_this_engine_emits`), and every unbuilt reason reaches a reader (`every_unbuilt_reason_reaches_a_reader_of_a_corpus_that_declares_none`).
- **The derived facets are written, and none folds its own output** (`the_derived_facets_are_written_into_the_block`, `a_generated_document_on_the_shelf_it_reads_does_not_fold_its_own_date`). A state that the regime of the kind does not admit is refused (`a_state_the_kinds_own_regime_does_not_admit_is_refused`), and a state no edge sets takes the newest date (`a_state_no_edge_sets_is_dated_by_the_newest_document_the_projection_read`).
- **A shelf page states no stored count** (`a_shelf_page_states_no_count_of_the_documents_it_lists`), and a filtered shelf index names only what its filter admits (`a_filtered_shelf_index_names_only_what_the_filter_admits`).
- **The verb index has one row for each verb** (`the_committed_index_carries_one_row_for_every_verb_this_binary_dispatches`), and it goes stale when a verb is added (`a_verb_added_to_the_dispatch_table_makes_the_committed_index_stale`).
- **Each emitter labels a document with its declared name** (`every_emitter_that_labels_a_document_prints_its_declared_name`).
- **The descriptor names its emitter set**, and a different set withholds the remedy (`a_committed_descriptor_from_another_emitter_set_withholds_the_remedy`).
- **The native export round-trips the graph** (`the_native_export_round_trips_the_graph`). The census fails a projector that drops a node with no reason (`the_census_fails_a_projector_that_dropped_a_node_with_no_reason`). A filtered profile withholds a document and its edges (`a_filtered_profile_withholds_a_document_and_its_edges`), and two filters for one profile are refused (`two_entries_of_one_profile_may_not_declare_two_filters`).
- **The JSON Schema export agrees with the engine** (`the_emitted_schema_and_the_engine_agree_document_for_document`, in `differential.rs` against `differential_oracle.py`), and spec 2 names no emitter that this engine does not build (`no_other_format_in_spec_2_names_an_emitter_this_engine_does_not_build`).
- **A probe result is generated from its transcript** (`the_fixture_tree_generates_the_recorded_result`), and it goes stale when the transcript changes (`a_result_goes_stale_when_its_transcript_changes`). A transcript planned against another taxonomy fails the run (`a_transcript_planned_against_another_taxonomy_fails_the_run`).
- **`headwater site` resolves a served path in either form** (`a_source_is_served_in_either_form_and_an_index_in_one`), and a navigation path outside the corpus root is missing (`a_navigation_path_outside_the_corpus_root_is_missing`).

The generated-file marker is the rule that keeps this engine from destroying an authored file ([spec 6](../spec/06-engine-architecture.md#projections)).
