---
id: HW-SPEC-checks-and-cache
status: current
status_since: 2026-10-01
summary: "How two crates instantiate each rule through one scope trait, key a content-addressed cache, publish the read set, and render one run in four formats."
last_verified: 2026-10-01
title: "Checks and cache"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft+revise
  evidence_basis: evidenced
relations:
  governs:
    - engine/crates/check/src/**
    - engine/crates/adapter/src/**
  traces_to:
    - HW-SPEC-engine-architecture
    - HW-SPEC-check-layer
    - HW-SPEC-assurance-model
    - HW-SPEC-design-departures
    - HW-IFACE-headwater-check
    - HW-IFACE-headwater-gate
    - HW-IFACE-headwater-json
---

# Checks and cache

## Scope

This spec describes the inside of the cache and checks stages of [spec 6](../spec/06-engine-architecture.md#pipeline). Two crates under `engine/crates/` build those stages: `check` and `adapter`.

The stages have these inputs:

- The census and the graph that phase A built, and the resolved taxonomy from the lock.
- The injected values: the clock, the change manifest, the identifier claim store, and the observation snapshot.
- The cache file at `.headwater/cache/checks`, when one exists.

The `check` crate gives a `Run`. A `Run` holds every instance with its outcome, the findings, the coverage, the read set, the suppression inventory, the adoption ledger and the register. The `adapter` crate renders one `Run` as `text`, `json`, `sarif` or `markdown`.

Other documents state what the stages do, and this spec does not repeat them:

- [Spec 12](../spec/12-check-layer.md) states what a check is, its scope, the read set, the two phases, fixability, determinism and the correctness roots.
- [Spec 4](../spec/04-assurance-model.md#findings) states what a finding, a suppression and coverage are.
- The [`headwater check`](../interfaces/headwater-check.md), [`headwater gate`](../interfaces/headwater-gate.md) and [`headwater json`](../interfaces/headwater-json.md) contracts state what a user of each verb sees.

The public Rust API of the crates is not in this spec ([HW-DR-0098](../decisions/0098-an-engine-subsystem-is-described-by-a-technical-design-spec-on-a-shelf-of-its-own-and-its-behavior-stays-where-it-is-already-written.md)). The doc comments of the crates state it.

## Design

### One runner, and one list of rules

`check/src/lib.rs` holds the runner. `run` takes the census and the graph as arguments, and it does not build them. Two passes over one corpus can disagree, and the pair that must agree here is the denominator and the thing that is measured against it.

`RULES` lists every rule template in the order that a report lists them. On this branch the array has 48 entries, and the length of the array is the count. The `//!` header of `lib.rs` and a comment in `run` state older counts, and the array is the one that the compiler holds. `run` names each rule once, and that is the whole of registration.

A rule is a template, and the runner instantiates it for each target. The generation step reads the taxonomy and never a document. It asks about a kind or a relation name, so a check cannot select its own targets from the corpus.

### Origin and grain are two different facts

Each rule has an origin and a grain. The origin is what the rule reads a declaration from: Shape, Graph or Document, as [spec 12](../spec/12-check-layer.md#the-five-origins-of-a-check) names them. The grain is what one instance covers: a document, an edge, a neighbourhood or the corpus.

Most rules are generated. They name no facet, kind, relation or identifier scheme, and they read each one from the resolved taxonomy. A new facet in the taxonomy therefore gives new instances with no new code. `fragment` and `duplicate` read no declaration, because the language has no member that turns them on or off.

The Graph-origin rules span all four grains. `declaration` and `identity` are document-grained, because they report the phase-A defects that stop an edge from existing. An edge-grained instance exists for each edge, so it cannot reach a block that made no edge. `duplicate` is corpus-grained, because two documents that claim one identifier need not share an edge.

The Document-origin rules `voice` and `language` read a regime that names categories or a language and states no set of them. The engine holds a closed set of each. An instance that meets a name outside that set skips, and the reason names the value.

### One trait for each scope

`scope.rs` makes the scope of a check a type. Each grain has one trait: `DocumentCheck`, `EdgeCheck`, `NeighbourhoodCheck` and `CorpusCheck`. The trait is the only way to receive the matching view. Three properties hold the boundary:

1. A view has private fields, and only `scope.rs` builds one. So a check cannot widen its view.
2. `Scope` has no public constructor. The engine derives the reported scope from the trait.
3. The view records each read, with the census digest of the file. So a check cannot under-report what it read.

A trait can also carry the inputs that a check declares: `NEEDS_BODY`, `NEEDS_PHASE_A`, `NEEDS_CLOCK`, `NEEDS_PRIOR`, `NEEDS_DECLARER_PRIOR`, `NEEDS_CLAIMS` and `NEEDS_OBSERVATIONS`. Each trait carries only the constants that its grain can use. A view gives nothing to a check that did not declare the input. The value of `Scope` that carries a declaration goes to the view and to the cache key from one binding, so the two cannot drift.

`VERSION` is the edition of a rule, and an author raises it by hand when a rule decides differently. `tests/editions.rs` and `fixtures/editions.ledger` catch an author who forgets, on the corpora that the ledger records.

#### The four corpus-view constants that are not `Scope` flags

The corpus-scoped view carries four more `NEEDS_` constants: `NEEDS_LINKS`, `NEEDS_ANCHORS`, `NEEDS_GRAPH` and `NEEDS_ORPHANED`. They are not `Scope` flags.

The first three join no cache key. Each one is a reading of documents that the instance already reads, and the read set names those documents already. A key component would hash the same bytes twice. It would also make every cache of every adopter cold for a fact that changes no verdict. `link_path` carries the one case where this argument does not apply: a link whose target is not a document of this corpus.

`NEEDS_ORPHANED` joins the key as a value that the caller injects, and not as a file of the read set. `headwater generate` computes the set of orphaned marked files from the lock and the tree, and the `check` crate cannot repeat that computation. The value goes into the key as a `resolution` line, on the terms of an anchor target. The doc comment of each constant in `engine/crates/check/src/scope.rs` gives the full reason.

### The cache key, and why a damaged file costs one run

`cache.rs` holds a cache that is content-addressed. Its key has these components:

1. The census digest of each input that the view recorded.
2. The digest of the lock.
3. The `VERSION` of the rule.
4. The injected values that the scope declares: the clock and the prior version. The key carries the clock only when the scope declares it, so a rule that no date can move stays warm.
5. The rule and the target, because two instances can read the same documents and give different results.
6. The answer of a resolver about an external anchor, for an instance whose subject is a resolved target ([HW-OBL-0117](../obligations/0117-a-cached-verdict-about-an-anchor-survives-the-change-that-falsifies-it.md)).
7. `rules_digest`, the digest of the sorted list of compiled rules. An engine that drops or adds a rule therefore serves nothing from the previous engine.

A cache hit is a fact about a disk and not about the corpus. So the hit count is in `cache::Report`, the CLI writes it to standard error, and no report on standard output carries it. A skipped instance is never stored, so the engine decides each skip again.

Every doubtful case fails toward running again. An input with no digest gets no key. A record that this engine cannot read is a miss. A file with another `FORMAT` line, or a file that does not parse, is an empty cache. None of these is an error, and none can change a verdict.

The cache serves the instances that nothing touched, and every instance still has an outcome. Whether that makes a run proportional to a change is open ([HW-OBL-0072](../obligations/0072-a-cache-of-check-results-does-not-make-a-run-proportional.md)). The design is measured at spike scale only ([HW-OBL-0003](../obligations/0003-the-rebuild-and-cache-design-is-measured-at-spike-scale-alone.md)).

### The read set, and the gate that reads it

`readset.rs` takes the union of every input that every instance read, with the lock digest, the clock and the rule versions beside it. It adds no measurement, because each instance already carries its inputs for the cache. The CLI writes the union as a section of the report and as a file.

`gate.rs` holds that file against a later tree. It hashes each listed path and reads nothing that the file does not list. Three cases void a verdict, and the file writes each one as a line of its own:

- A `barrier` line, for a corpus-grained rule. Its verdict rests on the absence of a document, and no list of paths can state that absence.
- A `windowed` line, for a rule that reads the clock. The verdict is about one day.
- An input with no content hash. The gate cannot decide that such an input stayed the same.

The union loses which instance read which input, so a gate answers one question about the whole run. Three questions about the read set are open:

- Which instances must run again ([HW-OBL-0081](../obligations/0081-a-published-read-set-never-says-which-instances-must-run-again.md)).
- Whether the read set of a real corpus is small enough ([HW-OBL-0101](../obligations/0101-whether-the-read-set-of-a-real-corpus-is-small-enough.md)).
- How a gate decides about an anchor ([HW-OBL-0118](../obligations/0118-the-published-read-set-names-no-anchor-so-a-gate-decides-nothing-about-one.md)).

### A change is a named set of inputs

`change.rs` reads a manifest that a hook or a CI job writes. A line names a document as added, or names it with a second path that holds its prior bytes. The engine reads and hashes those bytes, and the hash joins the cache key. The caller states which files moved, and the engine reads what they became.

The manifest is read in two steps. `Unbound::read` decides the syntax and reads the bytes, and it refuses a manifest that it cannot parse. A dropped line would read as a document that did not change. `Unbound::bind` holds each path against the census, and a path that matches no row is counted and named, not refused. No path is normalized.

A `verified` line states that the author read a document again in this change. A `verified` line with a third field states that the author read one edge of the document again. The third field is the target of the edge. Only `suspect` reads either line. It stamps an edge of a re-verified document only where the change also names the target of the edge. A three-field line that names the edge also gets a stamp. In a run with a change, `suspect` also reads whether an `added` or a `prior` line names a path the edge reaches. Its finding on a moved digest says so. A reader in a commit hook then knows whether this change moved the digest or an earlier one did. The remedy names the route that fits. A prior version that the engine could not read never reaches a check, and the runner skips the instance with the reason. Whether a change-scoped run is the cache under another name is open ([HW-OBL-0080](../obligations/0080-changed-only-is-the-content-addressed-cache-under-another-name.md)).

### Coverage is computed against the census

`coverage.rs` reads the census as the denominator and never builds one. A classified document with no instance is a finding, because it shows a wrong shelf pattern or a file in the wrong place. An unclassified document with no instance is not a finding here, because the census already reports it.

### Suppression and adoption are the runner's

`suppression.rs` reads each `headwater allow` directive and filters the findings after each instance has its outcome. So a cache holds what a check decided, and never what a reader saw. A suppression is not a skip, and coverage does not move. A directive that names an unknown rule, an unknown reason or a bad date suppresses nothing and is reported as refused. The inventory reports four states, `applied`, `expired`, `unused` and `refused`, by rule and by shelf.

`adoption.rs` reads the `adoption` block of the lock as `(document, rule)` pairs. A pair holds every finding of its rule on its document, and a path with a `*` is refused. A finding that a pair holds is pending, and it does not fail a strict run.

### A patch states what it expects to find

`patch.rs` is the patch that a check offers beside a finding. A finding is fixable when a patch rides with it, and never otherwise ([HW-OBL-0087](../obligations/0087-fixable-has-two-readings-inside-one-engine.md)). A patch has four shapes: `Text`, `Half`, `Create` and `Facets`. A check has no filesystem, so a patch states an offset and the bytes that the check expects there. The applier in `headwater-scaffold` compares those bytes with the file before it writes. `Create` refuses a path that is occupied, because an overwritten claim file loses a fact that nobody can rebuild.

`fill.rs` holds the one text fill of the engine, and the width that every report uses. The `adapter` crate and the CLI call it, so no second copy exists.

### Inputs beside the corpus

`claim.rs` reads the identifier claim store at `.headwater/ids` ([HW-DR-0054](../decisions/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md)). The store forces two branches that claim one identifier to meet as a merge conflict. The two claim rules then hold the store against the corpus. An absent store is an empty store.

`observation.rs` reads `.headwater/observations.yml`, the committed record of which external control ran and which verification a build settled. An absent or unreadable file reads as empty, so every external control reports as not observed. One reader serves both populations, so a caller does not keep two parsers in step.

### The register

`register.rs` reads the `obligations` and `controls` declarations. A finding names its obligation through the control whose `mechanism` names the rule with the `check:` prefix. The `phase:` prefix names a phase of the engine that discharges an obligation with no finding. A control with any other prefix names a mechanism outside the engine, and it binds nothing here. The `Run` carries the register as a projection that nothing authors.

### The adapter crate, and why it is a crate

`adapter/src/lib.rs` has one dispatcher, `render`, and it is the only place where a caller turns a run into bytes. `render` takes the census and the graph as well as the run, because `text` needs them. One dispatcher keeps `headwater check` and the MCP `check` tool on the same text.

The adapter is a crate above `check`, and not a module in it. Nothing in `headwater-check` names `headwater-adapter`, so the compiler stops a renderer that reaches into a check ([spec 8](../spec/08-design-departures.md), departure 6).

Each format has one file. `sarif.rs` writes SARIF 2.1.0, and `markdown.rs` writes a job summary or a review comment. `json.rs` writes the finding shape, and `text.rs` writes the report for a terminal.

### A renderer is what the engine ships, and an adapter is a renderer with a credential

`check --format` writes four vocabularies, and each of them is neutral. SARIF is an OASIS standard. Markdown is a job summary or a review comment. JSON is the finding shape that [spec 4](../spec/04-assurance-model.md#findings) declares. A forge is a forge because of the call that uploads the artifact, and because of the credential on that call. That call is outside the engine. The workflow of this repository is one instance of it.

Two of the four formats exist to hold the adapter boundary open. [Spec 8](../spec/08-design-departures.md) asks for more than one adapter from the start, in departure 6. With one adapter, everything the adapter needs is in the core by definition, so nobody can see the boundary. SARIF and Markdown show the boundary, because each one loses different things.

### Every format declares a loss set, and a census audits it

`Format::loss` gives the loss set of each format. A finding of the run is in the output, or a declared reason accounts for its absence. A finding that no reason covers is a defect in the adapter, and the census fails the run. This is the export rule of [spec 7](../spec/07-distribution-and-federation.md#an-export-is-a-projection-and-it-declares-what-it-dropped), applied to the fields of a run in place of the classes of the graph.

The census in `adapter/src/lib.rs` has two halves. The finding half matches each finding to one record of the artifact, and that record must name the rule and the path of the finding. For SARIF and `json`, a parser reads the artifact, and a record is one result or one member of `findings`. For Markdown and the text report, the census cuts the artifact, and a record is one table row or one finding block. Each record serves one finding only. So a run with two findings on one rule and one path needs two records, and one dropped record fails the census.

The carrier half reads each entry of the loss set. An entry names a path into the artifact and the members under that path, and the census resolves each one in the parsed artifact. Each entry comes to one of three outcomes:

- `held`: the artifact agrees with the run about each member. A member is present where the run has a value, and absent where the run has none.
- `adrift`: the artifact and the run disagree. Like a finding that no reason covers, this is a defect in the adapter.
- `unaudited`: the entry names no member of this artifact. The value went nowhere, or it went to a place outside these bytes.

The three outcomes sum to the entries that the format declares, so the census passes over no entry in silence. `Census::accounts` states that sum.

An entry names its members, so a bag that resolves is not an answer. An entry with no members is `held` only where its path resolves to a value that is not a map. Where the path resolves to a map, the entry is `adrift`, because a map has names that the entry could list. One SARIF entry showed why. It said that the coverage went to a property bag. Four counts went there, and the skip classes went nowhere, and the old audit held the entry for as long as it existed.

An entry whose carrier is conditional is judged against the run, and never against the emitter. Some members are in the artifact of one run and absent from another. Such an entry declares its condition, and the census reads that condition off the run. It shares no function with the emitter. If the census read the run through the emitter, a change to the emitter would move the artifact and the expectation together. An emitter that stopped writing a member would then pass.

The two prose formats declare entries that no member path can reach, because no parser reads them. For those entries, a person who reads the artifact is the only check.

### Three severity scales, and the one that reaches SARIF

Three scales meet at a CI surface. A check reports a severity ([spec 12](../spec/12-check-layer.md#severity-is-the-checks-posture-is-the-controls)). An obligation carries `high`, `medium` or `low` ([spec 4](../spec/04-assurance-model.md#obligations-are-data)). A control carries a posture, `advisory` or `blocking`. SARIF has a fourth scale, with the values `error`, `warning`, `note` and `none`. The SARIF `level` comes from the severity of the check alone, by this table:

| Check severity | SARIF `level` |
|---|---|
| `error` | `error` |
| `warn` | `warning` |
| `info` | `note` |

The other two scales are refused, and not left out by accident. The scale of the obligation describes the invariant, not the finding. It would give one level to each finding of each rule that serves the obligation. The judgment of the control would then arrive in the field of the check. The posture says whether a finding stops a gate. A `level` that said "this blocks" would be the engine ordering what lands. Both values still travel, in the members that SARIF keeps for values that its own vocabulary does not name.

### A suppressed finding is in the output, and it is marked

A live finding, a `migration-pending` finding and a suppressed one are three different things. A surface that shows only the live findings reports a suppression that nobody can count. [Spec 12](../spec/12-check-layer.md#suppression-is-the-runners) rules that such a suppression looks the same as a rule that never fires. So each format writes the escaped findings, and marks each one with its class. SARIF writes a `suppressions` array on each escaped result, so a consumer shows the result as dismissed and not as open.

SARIF has two values for `suppression.kind`, and [spec 4](../spec/04-assurance-model.md#suppression) has three escape classes. `sarif::kind` maps them by where each one is written:

| Escape class | Where it is written | SARIF `suppression.kind` |
|---|---|---|
| `suppression` | A directive in the document that it hides | `inSource` |
| `migration-pending` | A task of the adoption payload, in the lock | `external` |
| `waiver` | A record of a publisher, when a waiver mechanism exists | `external` |

So the two widest classes share one value. `properties.headwater.escape` keeps them apart, and the SARIF loss set records the shared value.

### What a run states beside its findings

A run that names a change says so in every format. `check --change` gives a run the documents of one change, and the promotion count and every transition finding are about that set alone. The text report opens with the count of documents named, the paths that reached no row of the census, and the promotions. `json` writes the `change` member that [the check contract](../interfaces/headwater-check.md#the-json-report-member-by-member) names. `markdown` writes a paragraph and a list. SARIF writes the property bag of the run, and its loss set records that. A run of the full corpus writes none of this, and that absence tells the two kinds of run apart.

A run emits what it evaluated, and never orders what lands. Each format writes the taxonomy lock hash and the [read set](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) beside the findings. `json` writes each input of the read set with its digest, and SARIF writes each input as an artifact. The text report writes the read set under a heading of its own, and Markdown writes a summary of it. A gate that holds the merge result can then decide whether the verdict still applies ([spec 4](../spec/04-assurance-model.md#a-verdict-is-about-one-state-of-the-corpus)). Over this corpus, the answer is always to run again, because every run publishes a barrier.

The corpus tree is a gap ([HW-OBL-0028](../obligations/0028-a-run-cannot-report-the-corpus-tree-because-nothing-computes.md)). Nothing in the engine computes a corpus tree, so no format writes one, and `Subject` in `adapter/src/lib.rs` has no field for it. SARIF has `run.automationDetails.id` for that value, and `sarif.rs` leaves it out. The read set is not a corpus tree: the read set holds what the checks read, and a tree holds what the census walked.

The engine does not hold a queue, choose an order of landing, evaluate a future state, or block a merge. A merge queue answers the merge question completely, and the cost is a serialized landing. That trade belongs to the forge. This is the boundary that [Q7](../spec/09-decisions.md#q7--scope-of-the-mcp-surface) drew for the write path, applied at a second place.

## Invariants

A change to these crates must keep each of these. Each item names the test that holds it, in the `tests/` directory of the crate.

- **A cache changes no verdict.** A cached run, a cold run and a run with no cache write the same report (`a_cached_run_and_a_run_with_no_cache_write_the_same_report`, and over this repository `this_repository_reports_the_same_run_from_a_cache_as_from_none`). The second test reads the live tree ([HW-OBL-0144](../obligations/0144-the-cache-identity-test-reads-the-live-working-tree-so-an-edit-during-the-run-reports-the-correctness-root-as-violated.md)).
- **Each key component invalidates an entry.** An edited document runs again (`an_edited_document_is_evaluated_again`). A moved anchor target, a moved clock, a moved lock and a dropped rule serve nothing stale (`a_moved_anchor_target_is_not_served_from_the_entry_before_it`, `a_clock_that_moved_is_not_served_from_the_entry_before_it`, `a_lock_that_moved_serves_nothing`, `an_engine_upgrade_that_drops_a_rule_serves_nothing`). So does a duplicate that the other file settled (`a_duplicate_settled_in_the_other_file_is_not_served_stale`).
- **A damaged cache file costs one run** (`a_damaged_cache_file_costs_one_run_and_nothing_else`).
- **A rule that changes its decisions raises its edition** (`every_ledgered_rule_decides_what_its_version_recorded`, `a_moved_digest_at_an_unchanged_version_fails_and_bless_keeps_the_row`).
- **The scope comes from the trait, and the read set comes from the view** (`the_scope_of_every_rule_comes_from_the_trait_that_binds_it`, `the_read_set_of_an_instance_comes_from_the_view_and_not_from_the_check`, `a_document_check_receives_the_body_only_when_it_declares_it`). Spec 12 names every `Scope` flag (`the_scope_block_of_spec_12_names_every_flag_scope_declares`) and every grain of `Grain` (`the_scope_block_of_spec_12_names_every_grain_the_engine_has`).
- **The observation snapshot is in the read set** (`a_present_snapshot_joins_the_read_set`), and a gate does not carry a verdict over a deleted entry (`deleting_the_snapshot_entry_between_check_and_gate_does_not_carry`).
- **The denominator is the census** (`the_denominator_is_the_census_and_not_the_classified_set`), and a classified document with no instance is a finding (`a_classified_document_with_no_instance_is_a_finding_and_an_untyped_one_is_not`).
- **A suppression changes the report and not the instance** (`a_suppressed_finding_leaves_the_report_and_stays_in_the_instance`). A pending finding never blocks (`a_pending_finding_never_blocks`).
- **The fixture tree runs to its recorded report** (`the_fixture_tree_runs_to_the_recorded_report`), and the injected clock changes a verdict and nothing else does (`the_injected_clock_changes_a_verdict_and_nothing_else_does`).
- **Every finding reaches every format** (`every_finding_reaches_every_format`), and each format loses only what its loss set declares (`every_entry_of_every_loss_set_is_accounted_for`). Each format renders its recorded fixture (`the_fixture_tree_renders_the_recorded_sarif`, `the_fixture_tree_renders_the_recorded_json`, `the_fixture_tree_renders_the_recorded_markdown`, `the_fixture_tree_renders_the_recorded_text`).
- **The SARIF level is the severity of the check** (`the_level_is_the_checks_severity_and_never_the_obligations`), and the SARIF rule list is the registry that ran (`the_rule_list_is_the_registry_that_ran`).
- **The adapter sections above are what the adapter writes, and they are the one home of these rules.** The severity table is what `sarif::level` writes (`the_level_table_of_checks_and_cache_is_what_sarif_writes`), and the escape-class table is what `sarif::kind` writes (`the_suppression_kind_table_of_checks_and_cache_is_what_sarif_writes`). The three outcomes are the outcome fields of `Census` (`the_census_outcomes_checks_and_cache_names_are_the_fields_census_counts`). Spec 6 keeps a short section that links here (`spec_6_keeps_a_short_ci_adapters_section_that_points_at_checks_and_cache`). It shares no run of eight words with these sections (`spec_6_shares_no_run_of_eight_words_with_the_adapter_sections_of_checks_and_cache`), and no engine comment credits spec 6 with one of these rules (`no_comment_quotes_moved_adapter_text_as_spec_6`).
- **The `headwater check` page names the JSON shape** (`the_check_interface_page_names_every_version_of_the_json_shape`, `every_member_of_a_rules_entry_is_named_on_the_check_page`).

Scope enforcement and the cache are correctness roots ([spec 12](../spec/12-check-layer.md#the-correctness-roots)).
