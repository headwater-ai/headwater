---
id: HW-SPEC-authoring
status: current
status_since: 2026-10-01
summary: "How two crates write a new document, a fix, a migration step and an imported edge through one all-or-none put and one reciprocal splice."
last_verified: 2026-10-02
title: "Authoring"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft+revise
  evidence_basis: evidenced
relations:
  governs:
    - engine/crates/scaffold/src/**
    - engine/crates/import/src/**
  traces_to:
    - HW-SPEC-engine-architecture
    - HW-SPEC-authoring-and-lifecycle
    - HW-SPEC-taxonomy-model
    - HW-SPEC-distribution-and-federation
    - HW-SPEC-check-layer
    - HW-IFACE-headwater-new
    - HW-IFACE-headwater-capture
    - HW-IFACE-headwater-import
    - HW-IFACE-headwater-check
    - HW-IFACE-headwater-taxonomy
    - HW-DR-0019
    - HW-DR-0054
    - HW-DR-0074
    - HW-DR-0086
    - HW-DR-0098
    - HW-DR-0100
    - HW-DR-0101
---

# Authoring

## Scope

This spec describes the inside of the authoring subsystem of [spec 6](../spec/06-engine-architecture.md#subsystems). Two crates under `engine/crates/` build it: `scaffold` and `import`. The pipeline diagram of spec 6 does not draw this subsystem, because it does not read a corpus to give a verdict. It writes into a corpus.

The subsystem has four writers. Each writer puts bytes into a document that the next `headwater check` reads as authored:

- `headwater new` writes a new document and the reciprocal halves of its edges.
- `headwater check --fix` writes the patches that a run of the check layer offers.
- `headwater taxonomy migrate --apply` writes the mechanical steps of a migration payload into front matter and into the overlay.
- `headwater import --write` writes edges from a committed snapshot of an external system of record.

The inputs are the resolved taxonomy, the census, the graph index, the identifier claim store and, for an import, a pinned snapshot. The outputs are files on the tree. `new` also writes one claim file for each identifier that it mints, and one capture-cost reading for each document.

The bodies of the `new`, `capture`, `import`, `taxonomy migrate` and `check` verbs are in `engine/crates/cli/src/main.rs`. They belong to the Command surface row of spec 6. This spec describes the library that those verbs call.

Other documents state what the writers do, and this spec does not repeat them:

- [Spec 3](../spec/03-authoring-and-lifecycle.md#templates-and-scaffolding) states what a template is and what the scaffolder determines, and its [capture cost](../spec/03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) section states the assisted fraction.
- [Spec 12](../spec/12-check-layer.md#fixability) states what a fix may offer, and its [correctness roots](../spec/12-check-layer.md#the-correctness-roots) section names the scaffolder and the importer.
- [Spec 2](../spec/02-taxonomy-model.md#versioning-by-measured-compatibility) states the migration payload.
- [HW-DR-0019](../decisions/0019-inbound-integration-an-external-system-of-record.md) states inbound integration, and [spec 7](../spec/07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it) states how a tier above a corpus reads a pinned export.
- The contracts of [`headwater new`](../interfaces/headwater-new.md), [`headwater capture`](../interfaces/headwater-capture.md), [`headwater import`](../interfaces/headwater-import.md), [`headwater check`](../interfaces/headwater-check.md) and [`headwater taxonomy`](../interfaces/headwater-taxonomy.md) state the behavior of each verb. The `taxonomy` contract states the `migrate` operation, and spec 2 states what a step of the payload is.

The public Rust API of each crate is not in this spec ([HW-DR-0098](../decisions/0098-an-engine-subsystem-is-described-by-a-technical-design-spec-on-a-shelf-of-its-own-and-its-behavior-stays-where-it-is-already-written.md)). The doc comments of the crates state it.

## Design

### The taxonomy determines a value, or a person owes it

`propose` in `scaffold/src/lib.rs` turns a `Request` and a set of `Sources` into a `Plan` or a `Refusal`. It writes nothing. A `Request` carries the kind, the title, an optional summary and the date. It also carries three inputs from the caller. `relates` holds the relations to write. `given` holds the facet values that the caller states. `directory` names the directory for a shelf whose pattern globs one. The date is injected, so the same request over the same corpus gives the same bytes.

The scaffolder writes what the taxonomy determines and asks for what it does not. Each `Field` of a plan carries an `Origin`. `Origin::Scaffolded` names the declaration that decided the value. `Origin::HandEntry` marks a prompt for a value that a person owes. No template file exists. Spec 3 makes the template a function of the kind declaration, and a file under a package would be prose that no census row covers.

The scaffolder is a correctness root. Spec 12 says that nobody reviews a scaffolded edge one at a time. Thus a defect here makes wrong edges at the scale of the corpus. For this reason, `propose` refuses before it writes a document that a rule would report. The module comment of `lib.rs` has a table that pairs each refusal with the rule that it stands in front of. `Refusal` is a closed set, so that a test names the refusal that it expects.

The minting step runs before the placement step, because a shelf layout can name the sequence of the identifier. `Minting` records the highest value that the run found on the tree. Spec 3 asks for every value that was ever allocated, and a tree holds no history, so this value is a lower bound. After the placement, `propose` asks the census classifier which shelf the path is on. A path that the classifier reads differently is a defect of this crate, and the run refuses.

`Plan::assisted` gives an `Assisted`, which is the four counts that spec 3 names: fields, sections, the identifier and edge halves. It derives the counts from the plan, so a field that changes origin changes the count. The near half of an edge is hand entry, because the author named the target. A far half is the one edge fact that the scaffolder derives.

The scaffolder writes no far half while the new document is at its initial state ([HW-DR-0086](../decisions/0086-a-reciprocal-half-is-owed-once-its-writer-leaves-its-initial-state.md)). It records the half as `Owed`. It writes no far half of a symmetric relation, and it sets no state on the target of `supersedes` ([HW-DR-0101](../decisions/0101-new-writes-no-far-half-of-a-symmetric-relation-and-no-state-on-a-supersedes-target.md)). The check layer reports each of these later, and `check --fix` writes them through the same splice. Thus the rule stays the one writer of each fact.

[HW-OBL-0107](../obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md) records a kind of the base package that the scaffolder refuses to write. [HW-OBL-0113](../obligations/0113-every-check-passes-a-document-that-is-still-the-scaffolder-s-placeholder.md) records that every check passes a document that still holds the placeholder of the scaffolder.

### Three lock members that only the scaffolder reads

`declared.rs` reads three members of the resolved taxonomy that no typed reader carries. They are the `allocation` of a scheme, the `created_by` of a relation and the `type` of a facet. Each member moves into a typed reader when a check reads it. Until then, a second reader would be a second copy of one declaration. The module comment records two members that already moved.

### The reciprocal splice

`write.rs` turns a plan into bytes. `compose` writes the new document whole and splices each far half into the front matter of its target. `splice` assumes that a nested block is indented by two spaces, which no declaration states. It parses its own result and looks for the half that it wrote. A file whose shape the splice guessed wrong fails that read, and the run refuses with `Refusal::ReciprocalUnwritable` and writes nothing.

There is one splice in the engine. `fix.rs` writes a missing reciprocal half through it. `import/src/write.rs` writes an imported edge through it. This is why `import` depends on `scaffold`. A second writer into one `relations:` block would disagree with the first on the day that a document nests differently.

### One put on the tree, all or none

`tree.rs` puts a set of composed files onto the tree. Every writer of an authored document uses it: `write`, `fix`, `migrate`, `overlay` and `import`. Each writer composes every byte before a byte reaches disk. The put is the step that can fail.

`Reserved::over` opens every target for writing before it writes a byte. A target that is read-only, a directory or missing fails there, and the tree stays as it was. `Reserved::making` then creates the one new file, through `create_new`, so the kernel states that this run made the file. `Reserved::commit` writes through the open handles and reads each file back. On a failure, it restores every file from the bytes that it read before, and it removes the file that it made. `Halted` states that the tree is as it was only when it is.

The handles stay open between the check and the write, because a probe that closed them would answer about a moment that has passed. A crash of the process between two writes is not covered. No user-space scheme covers that case without a journal.

### The claim store

The identifier claim store has two writers ([HW-DR-0054](../decisions/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md)). `claim.rs` writes the claim of each identifier that `headwater new` mints. `fix::make` writes a claim that `identifier.claim.missing` reports as missing, from the `Patch::Create` of that rule. The path and the format of the store belong to `headwater_check::claim`. `claim.rs` imports the format, `contents_for`. The path comes from `path_of`, which the minting step of `propose` in `lib.rs` calls, and it reaches `claim.rs` on `Minting::claim`. `fix::make` writes the path and the bytes that `headwater_check::claim` put into the patch. Each writer opens a claim with `create_new`, so the test and the create are one syscall. An existing claim is a refusal. A claim is the only record of which document minted an identifier, so no writer truncates one.

The claim is written before the document. If the document write then fails, the result is a spent number with no document, and spec 3 permits that state. The other order can leave a document with no claim, which `identifier.claim.missing` reports. Whether a mint owes a claim is decided once, at plan time, through `headwater_check::claim::takes_a_claim`. `Minting::claim` carries the answer, so the writer and the rule read one predicate.

### The writer of a fix

`fix.rs` is the only thing that acts on a `headwater_check::Patch`. A text patch passes four guards before it lands, and the module comment states each. The fourth guard parses the patched text again and compares the parse with the parse of the source, with the substitution in it. Spec 12 names this comparison as a correctness root, because a rewrite at a wrong span corrupts a document.

The unit of a fix is one file. All the patches of one file land or none of them do. A refusal over one file is no evidence about another, so the run reports it beside the files that landed.

A fix edits files, and it can also create a file. `compose` puts each `Patch::Create` into a separate list. `fix::apply` puts the edits through `Reserved::over`, with no create step. Then `fix::make` creates each new file with `create_new`, outside `Reserved`. The create runs last because it is the step that can meet a path that another process took. If it fails, the edits are already on the tree, and `make` removes only the files that it created.

A fix applies a third shape, `Patch::Facets`. It replaces the value of a top-level scalar facet that the front matter already declares, and it adds no key. It runs after the text patches, because the offsets of a text patch are into the file as it stood. The module comment states its guards.

### The two writing halves of a migration

Spec 2 splits a migration payload into the steps that the engine applies and the steps that need judgment. `migrate.rs` and `overlay.rs` apply the first set. `migrate.rs` replaces one front-matter value in a document. `overlay.rs` changes one address in `.headwater/overlay.yml`, which has no front matter and no census row. The two writers guard different files, and neither one reaches the file of the other.

Each writer reads its result back against a different reading. `migrate.rs` compares the body byte for byte and compares every front-matter scalar by key path. `overlay.rs` reads the result with `headwater_resolve::operation::read`, which is the reader that the resolver uses. Each operation must keep its position, its kind and its value. Both writers keep the quotes of the value that they replace, and they refuse a site that they cannot read.

Neither writer touches the disk. Each one gives `tree::Composed` files. The verb puts the files of both into one `Reserved`, so the documents and the overlay of one migration land together or not at all.

### The capture-cost store

`reading.rs` holds the capture-cost store of spec 3, at `.headwater/capture-cost.jsonl`. `headwater new` appends one `Reading` for each document that it writes. The run is the only moment that the counts are known. Q4 keeps `created_by` on the relation type, so a committed corpus does not show which edges the scaffolder wrote.

A reading holds the lock digest, the date, the `Surface` that called the verb, the kind, the path and the identifier. It also holds the four counts of `Assisted`. It holds no person, no duration, no prose and no refused run. The module comment states why each member is in the reading and why each other value is not. `total`, `by_kind`, `by_surface` and `reach` derive each aggregate from the store, so a reader of the repository can compute each number again. [HW-OBL-0111](../obligations/0111-the-capture-cost-surface-names-an-entry-point-and-never-the-caller.md) records that a surface names an entry point and never the caller behind it.

`json.rs` writes the store as the JSON document of `headwater capture --json`. It is in this crate and not in `cli`, because each JSON emitter of this engine is in the crate that owns its data. The text report for a person stays in `cli`.

### Inbound integration

`import` is the edge half of [HW-DR-0019](../decisions/0019-inbound-integration-an-external-system-of-record.md). It writes edges with `created_by: import` from a snapshot that an external system of record exported. Spec 12 names the importer as a correctness root. A wrong imported edge gives a correct check result over a wrong graph, so no check finds it. For this reason, the refusals of `plan` are the whole instrument, and an import lands whole or not at all.

A `Declaration` is one entry under `imports` in `.headwater/taxonomy.yml`. A person writes it, and no verb writes it. It names the path, the digest, the channel that carried the digest, and the anchor resolver. `plan` refuses a declaration with no digest, no channel or no resolver. The module comment of `lib.rs` lists the refusals in the order that they fire. A digest proves that the bytes are the pinned bytes, and it does not prove who published them ([HW-OBL-0115](../obligations/0115-a-pinned-digest-authenticates-the-pin-and-never-the-publisher.md)). The channel is the sentence that a person wrote beside the digest.

`snapshot.rs` reads the payload, `snapshot.yml`. The pin over the snapshot is `headwater_resolve::release`, unchanged. Spec 7 says that a taxonomy pin and a snapshot pin work the same way. A second implementation would read one artifact in two ways. A snapshot carries its fetch time and the identity and revision of each item, and a missing one is a refusal.

`import/src/write.rs` decides whether a document already declares an edge at the pinned revision, and it names the attribute that carries the revision. A second import over one snapshot thus writes nothing. The splice is `headwater_scaffold::write::splice`, and the put is `Reserved`. An import creates nothing, so it uses `over` and `commit` and no create step.

[HW-OBL-0015](../obligations/0015-imported-edge-staleness-is-unmeasured-against-scaffolded-edges.md) records that nobody has measured the staleness of an imported edge against a scaffolded edge.

### Two anchor resolvers, one for each anchor kind

Spec 2 says that exactly one resolver owns each anchor kind. A resolver reads repository content or a committed snapshot, and never a live service. `anchors.rs` resolves an item identity against a committed snapshot. `harvest.rs` resolves a document identity against the pinned export of another repository, which is the third class of resolver that spec 2 names. Both resolvers have the same shape:

- Each resolver checks the digest before it reads an item. Bytes that are not the pinned bytes bind nothing.
- A pin that did not open still supplies its resolver. That resolver refuses every string with a reason that names the pin. An absent resolver would send a reader to look for a missing feature instead of a moved byte.
- Each resolver looks an identity up and never normalizes it. The only normalization is `str::trim`, because the far end owns its identities. A resolver that guessed would bind a typo to a real item.

`harvest.rs` digests the one export file with `headwater_hash::digest`, because an export carries no release record. Under the `counted` grain, an identifier that a tombstone lists by digest resolves to `Binding::Withheld` ([HW-DR-0100](../decisions/0100-a-counted-tombstone-lists-a-digest-of-each-withheld-identifier-and-a-sealed-one-lists-nothing.md)). This resolver is the one producer of that binding.

Neither crate opens a socket. Each verb takes a path that the caller already fetched and committed.

## Invariants

A change to these crates must keep each of these. A test holds each one that names a test.

- **A scaffolded document passes the checks of the engine** (`what_the_scaffolder_wrote_passes_the_engines_own_checks` in `engine/crates/scaffold/tests/pipeline.rs`).
- **A far half that the splice cannot write leaves the new document absent** (`a_reciprocal_that_cannot_be_opened_leaves_the_new_document_absent` in `engine/crates/scaffold/tests/writing.rs`).
- **A target that does not open stops the put before a file moves** (`a_target_that_will_not_open_stops_the_run_before_any_file_moves` in `engine/crates/scaffold/src/tree.rs`).
- **A commit that fails removes the file that the run made** (`a_commit_that_fails_removes_the_document_this_run_created` in `engine/crates/scaffold/src/tree.rs`).
- **A claim that exists refuses the second claimant** (`a_claim_that_is_already_there_refuses_and_the_first_claimant_stands` in `engine/crates/scaffold/tests/writing.rs`).
- **A patch whose expected bytes are not in the file is refused** (`a_patch_that_names_bytes_the_file_does_not_hold_is_refused` in `engine/crates/scaffold/tests/fixing.rs`).
- **A migration step refuses a key whose value is not the old value** (`a_key_that_no_longer_holds_the_value_is_refused_with_nothing_composed` in `engine/crates/scaffold/src/migrate.rs`).
- **An overlay step moves each address under it and no value** (`every_address_under_the_step_moves_and_no_value_does` in `engine/crates/scaffold/src/overlay.rs`).
- **A document that opens at an initial state owes its far half** (`a_document_that_opens_at_an_initial_state_owes_its_far_half` in `engine/crates/scaffold/tests/opening_state.rs`), **and writes no far half of a symmetric relation** (`a_draft_writes_no_far_half_of_a_symmetric_relation`).
- **The reading on disk is the fraction that the plan derives** (`the_reading_on_disk_is_the_fraction_the_plan_derives` in `engine/crates/scaffold/tests/telemetry.rs`).
- **Each refusal of an import has a fixture** (`engine/crates/import/tests/fixtures.rs`), **and an edge half that does not open leaves every other document as it was** (`an_edge_half_that_cannot_be_opened_leaves_every_other_document_as_it_was`).
- **A second import over an unchanged snapshot writes nothing** (`a_reimport_over_an_unchanged_snapshot_writes_nothing` in `engine/crates/import/tests/reimport.rs`).
- **A snapshot that is not the pinned artifact binds nothing and names the pin** (`a_snapshot_that_is_not_the_pinned_artifact_binds_nothing_and_names_the_pin` in `engine/crates/import/tests/resolution.rs`).
- **An identity is trimmed and not changed in another way** (`surrounding_space_is_trimmed_and_nothing_else_is` in `engine/crates/import/src/anchors.rs`).
- **An identifier that only a second export holds stays unresolved** (`an_identifier_only_the_other_export_holds_is_unresolved` in `engine/crates/cli/tests/harvest.rs`), **and a withheld identifier binds `withheld` while a typo stays unresolved** (`a_withheld_identifier_binds_withheld_and_a_typo_stays_unresolved`).
