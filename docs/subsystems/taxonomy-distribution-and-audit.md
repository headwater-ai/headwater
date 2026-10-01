---
id: HW-SPEC-taxonomy-distribution-and-audit
status: current
status_since: 2026-10-01
summary: "How three crates outside the checking loop fetch a published taxonomy, compare two taxonomies over one corpus, and measure a taxonomy against a corpus."
last_verified: 2026-10-01
title: "Taxonomy distribution and audit"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft+revise
  evidence_basis: evidenced
relations:
  governs:
    - engine/crates/fetch/src/**
    - engine/crates/compat/src/**
    - engine/crates/audit/src/**
  traces_to:
    - HW-SPEC-engine-architecture
    - HW-SPEC-taxonomy-model
    - HW-SPEC-distribution-and-federation
    - HW-SPEC-check-layer
    - HW-IFACE-headwater-taxonomy
    - HW-DR-0075
    - HW-AC-0001
    - HW-OBL-0119
    - HW-OBL-0008
    - HW-OBL-0154
---

# Taxonomy distribution and audit

## Scope

This spec describes the inside of three crates under `engine/crates/`: `fetch`, `compat` and `audit`. They are one row of the subsystem table in [spec 6](../spec/06-engine-architecture.md#subsystems).

None of the three builds the lock. The [Taxonomy resolution](taxonomy-resolution.md) subsystem builds it. These crates move a taxonomy between repositories, or they measure a taxonomy against a corpus:

- `fetch` downloads a published taxonomy artifact for `headwater taxonomy vendor`.
- `compat` compares two taxonomies over one corpus for `headwater taxonomy diff` and `headwater taxonomy migrate`.
- `audit` measures the taxonomy of the lock against the corpus for `headwater taxonomy audit`.

`headwater-cli` is the only crate that depends on any of the three. No crate of the checking loop links them, and no stage of [the pipeline](../spec/06-engine-architecture.md#pipeline) waits for them.

Other documents state what these verbs do, and this spec does not repeat them:

- [Spec 6](../spec/06-engine-architecture.md#taxonomy-validate-versus-taxonomy-audit) states what `taxonomy audit` reads and why it gates nothing. The `taxonomy vendor` paragraph of the same part states why one crate holds the network.
- [Spec 2](../spec/02-taxonomy-model.md#versioning-by-measured-compatibility) states the six dimensions of compatibility and the version bump that each result requires.
- [Spec 7](../spec/07-distribution-and-federation.md#consuming) states how a consumer vendors and fetches a package. Its [Upgrading](../spec/07-distribution-and-federation.md#upgrading) section states what `diff` and `migrate` report. Its [adoption store](../spec/07-distribution-and-federation.md#what-the-adoption-store-records-and-what-it-refuses-to) section states what one reading holds.
- The [`headwater taxonomy`](../interfaces/headwater-taxonomy.md) contract states the commands, their flags and their exit codes.
- [HW-DR-0075](../decisions/0075-the-vendor-verb-may-take-a-location-and-the-fetch-lives-only-in-a-crate-the-checking-loop-never-links.md) rules that the fetch lives in a crate that the checking loop never links. [HW-AC-0001](../acceptance-criteria/0001-no-engine-source-outside-the-fetch-crate-names-a-network-api-and-no-crate-of-the-checking-loop-reaches-a-network-package.md) is the criterion that holds it.

The public Rust API of each crate is not in this spec ([HW-DR-0098](../decisions/0098-an-engine-subsystem-is-described-by-a-technical-design-spec-on-a-shelf-of-its-own-and-its-behavior-stays-where-it-is-already-written.md)). The doc comments of the crates state it.

## Design

### Why the three crates are one subsystem

The three crates share one boundary. Each one is a consumer of finished phases, and no phase consumes it. `fetch` gives a directory to `headwater_resolve::package::vendor`. `compat` and `audit` read the census, the graph, a check `Run` and a generate `Plan` that other crates already built. The only `Cargo.toml` that names `headwater-fetch`, `headwater-compat` or `headwater-audit` as a dependency is the one of `headwater-cli`.

So a change to one of these crates cannot change a verdict of `headwater check`. A change to the census, the graph or the check layer can change what these crates report. These crates read those phases and never compute them again.

### `fetch`: the one crate that opens a socket

`fetch` takes the URL of a published artifact zip and gives back a `Fetched`, which is a directory that holds the unpacked artifact. It checks nothing about the bytes. The digest check stays in `headwater_resolve::package::vendor`, which reads the directory as it reads one that a person fetched by hand. So a fetched package and a hand-fetched package go through one check.

One gate, `allowed`, decides each request. The gate reads the first request and each redirect again. An https fetch stays on https for every hop. Plain http goes only to a loopback host, and never through a proxy. So a redirect cannot change an https fetch to plain http, and it cannot send plain http to a different machine. The client follows no redirect itself. `follow` reads each hop and refuses a fetch after `MAX_REDIRECTS` redirects, before it sends the next request.

An https request uses the proxy that the first of `ALL_PROXY`, `HTTPS_PROXY` and `HTTP_PROXY` names, unless `NO_PROXY` excludes the host. TLS still authenticates the named host through the tunnel.

Two caps stop a wrong URL from filling the disk. `LIMIT` caps the bytes that the fetch reads. `UNPACKED_LIMIT` caps the bytes that the unpack writes. The unpack reads the sizes that the archive declares first, as a fast refusal. It then counts the bytes that it writes, because a deflate member can decompress past its declared size. The unpack also refuses a member whose name goes out of the directory, and a symbolic link.

`scratch` makes a new directory for each fetch. Its name carries the process id, the time and a counter, because `cargo test` runs its cases as threads of one process. `Fetched` removes the directory when it drops, so a refused unpack leaves nothing on the disk.

`headwater-cli` puts `headwater-fetch` behind its `fetch` feature, which is on by default. A binary built without that feature opens no socket.

### `compat`: the comparison and nothing else

`compat` compares two readings of one corpus, one under each taxonomy. Each input is a finished phase that the caller runs twice:

| dimension | the input it compares |
|---|---|
| `classification` | the census |
| `instance_validity`, `consequence` | the check `Run` |
| `projection` | the generate `Plan` |
| `identifier` | the graph |
| `addressability` | the founding records of each resolution |

The corpus is the same tree in both runs, so a difference comes from the taxonomy. A dimension that computed a kind or a verdict again would report on a reading that no run made.

No dimension reads a file digest, a manifest, a description or a declaration order. Two publishes of an unchanged package differ in such trivia. A report that reads bytes reports each release as a change, and then a reader cannot tell it from a report that does not work. So a taxonomy that resolves to different text and gives the same corpus reading is compatible on all six dimensions.

`Outcome` has three arms: `Preserved`, `Broken` and `NotMeasured`. `Outcome::over` is the only constructor of `Broken`, and it gives `Preserved` for an empty list of breaks. A candidate taxonomy that does not resolve gives no census, no run, no plan and no graph. `Measured::against_nothing` is the only route to `NotMeasured`, and it takes the reason. So a dimension that did not run never reads as preserved.

`Report` carries the package, the two versions, the `Base` and the six dimensions. `Base` has three states, `Same`, `Moved` and `Unresolved`, because a candidate that did not resolve has no text to compare. `Bump` has three states too. `Major` means a dimension is broken. `NoMajor` means every dimension ran and none is broken. `Undecided` means that not every dimension ran. The six dimensions ask what a change did to a corpus, so they cannot decide between a minor and a patch version.

A `Caveat` is a fact about one comparison that the run cannot settle. It is never a refusal, because the run reaches both arms on correct input. `SourcesMoved` names the sources that the lock records whose bytes on disk changed. The ordinary publisher edits the package source in place, so a refusal there would refuse the ordinary case. `FoundingOutsideTheDigest` is a `Base::Same` beside a broken `addressability`. The founding record is the one reading of the previous side that the lock digest does not cover, so the run prints both facts.

`migrate.rs` answers where in the corpus the value of one migration step is. It gives one `Site` for each document that a step names, in one of three arms:

- `Front` is a front matter key that holds the value. A writer changes that value and nothing else.
- `Placement` is a document whose shelf carries the kind. No byte of the document holds the kind, so the remedy is to move the file. The arm carries the shelf, so a report says why it writes nothing.
- `Overlay` is one operation of the adopter's overlay that addresses the path of the step. The overlay file is outside the corpus root and in no census row, so `sites` reads the overlay beside the census.

`transition` reads the two versions of the lock header and the artifact, and it accepts only a version that is strictly greater. The lock digest does not cover the header, so this is the one place that holds the version pair. It is not a currency check. `taxonomy migrate` runs in a tree where the candidate already replaced the source of the lock, so `taxonomy resolve --check` would refuse every correct run.

`payload.rs` accounts a migration payload against the corpus. It reads from the census the documents of this corpus that each step names. It puts them beside the documents that the measurement reports as not valid under the candidate. A step that names no document has two states. Nobody here uses the value, or this repository's overlay already removed it, and `Accounted::declared` tells the two apart. `stands` reads the candidate taxonomy, resolved under this repository's selection and overlays. A step whose old value that taxonomy still declares did not happen for this consumer. `STANDS` is the one sentence that both `diff` and `migrate` print for it.

### `audit`: a measurement that gates nothing

`take` makes one `Audit` from the census, the graph, the shape of the taxonomy, the relation declarations and the adoption `Series`. It opens no file. Two passes over one corpus can disagree, so a report that read the tree again would be a second account of it. The adoption series is a parameter for the same reason. Its inputs are a file and a run of the check layer, and the caller makes both.

A finding needs a declared input. The `Finding` type has two arms, and each one reads a declaration of the taxonomy:

- `Stale` reads `stale_after_days` on the facet in the `freshness` role. It is a relation with a half on a document past that window.
- `Ungoverned` reads a scope pattern that the taxonomy declares for an anchor kind. It is one pattern with at least one entry that no `governance` edge reaches, and it lists the entries.

Every other reading is a distribution, printed with its population and with no verdict. Nothing declares how narrow a facet can be, or how low a capture rate can fall. A bar that this crate invented would be a verdict that no corpus declared. [HW-OBL-0119](../obligations/0119-an-audit-reading-carries-no-declared-bar-so-a-distribution-cannot-become-a-finding.md) holds that gap.

`Creators` holds the creator reading at the grain that [spec 6](../spec/06-engine-architecture.md#taxonomy-validate-versus-taxonomy-audit) rules, one row for each relation. It walks the closed set `CREATORS` and not the values in use.

The freshness of an edge is the freshness of the document that carries its half. The facet in the `freshness` role records when somebody last verified the document, its front matter included. The report states the reading in those words, because no corpus keeps a verification record for each edge.

A reading that this verb does not take is a `Waiting`, with a list of `Need` values. Each `Need` has a `Supply` that the run evaluates against the corpus in front of it. `Supplied`, `Undeclared` and `Unauthored` are three arms, because a declaration and an authoring pass end different waits. Before 2026-08-15 these waits were string literals. One of them stated a fact about the corpus that had gone false, and the verb printed it. So a wait now ends when the corpus supplies what it waits on, and never when somebody edits a string.

The layout reading renders each file name again through `headwater_scaffold::render_layout`, which is the function that `headwater new` names a file with. `rendered` reads the three sources of a placeholder in the order of the scaffolder. They are the slug of the facet in the `name` role, a facet of the document, and the sequence of its identifier. A second renderer would measure adherence to a template that the scaffolder does not follow. When a placeholder has no value, `hole` states whether the absence is in the schema or in the corpus.

### The adoption store

`reading.rs` keeps the series of adoption readings in `STORE`, which is `.headwater/adoption.jsonl`. The store is outside the corpus root, so no census row covers it and no rule reads it. Only `taxonomy audit --record` writes it.

A store is necessary because no crate of this engine walks git history. [Spec 12](../spec/12-check-layer.md) gives a change to the engine as a named set of inputs, and never as a second tree. So a past day's tree is not an input of any run, and the series grows one reading for each invocation that records one. [HW-OBL-0008](../obligations/0008-an-adoption-payload-has-a-first-reading-and-no-elapsed-time.md) names this series as the instrument it needs.

One line holds one reading of one invocation, and never one task. A corpus that declares no payload records `tasks: []`, and that line is different from no line. A task entry holds the `state` that the check layer decided, and this crate does not compare `until` against the date a second time.

`append` adds a line and never changes one. It refuses a second reading at one lock digest and one date. So two audits of one tree at one date write the same bytes. The pair is the key because two taxonomies over one tree on one day are two measurements. `load` reads an absent store as an empty store. It names each line that it cannot read by its line number, and it counts that line nowhere.

### The text report

`render.rs` writes the audit as text. Each section states its population before it states a number. A creator that no relation declares and a rate over an empty population are written out in words, and never printed as a zero.

`render.rs` paints a token after it folds a line. `headwater_check::filled` measures a line in characters, and a color sequence is characters that use no column. So a token painted before the fold uses part of the line width that a terminal never shows. `painted_in_place` substitutes each painted token into the folded text.

## Invariants

A change to these crates must keep each of these. A test holds each one that names a test.

- **Only the binary reaches the crate that opens a socket** (`only_the_binary_reaches_the_crate_that_opens_a_socket` in `engine/crates/cli/tests/network_boundary.rs`). The test reads `engine/Cargo.lock`. It holds the crate graph, and it does not hold the call sites inside `headwater-cli`.
- **A fetch over loopback unpacks, follows a redirect and refuses what the gate refuses** (the five cases of `engine/crates/fetch/tests/loopback.rs`, and the fourteen unit cases of `engine/crates/fetch/src/lib.rs`). A redirect from loopback to plain http on a different host is refused before it is followed. A zip that unpacks past the bound is refused.
- **A dimension with no break is preserved, and a run that did not measure decides nothing about the version** (the sixteen unit cases of `engine/crates/compat/src/lib.rs`). No caveat line reads as a dimension line.
- **A version transition goes forward only, in numeric order** (the five unit cases of `engine/crates/compat/src/migrate.rs`). The two unit cases of `engine/crates/compat/src/payload.rs` hold only its color. The end-to-end cases of `engine/crates/cli/tests/diff.rs` and `engine/crates/cli/tests/migration.rs` hold the verbs over a published artifact. `the_ordinary_upgrade_is_not_refused` fails if `transition` becomes a currency check.
- **The fixture tree gives the recorded audit** (`the_fixture_tree_produces_the_recorded_audit` in `engine/crates/audit/tests/readings.rs`, against `engine/crates/audit/fixtures/audit.report`). The recorded report holds the adoption section in its empty shape only, and [HW-OBL-0154](../obligations/0154-the-adoption-decay-section-is-fixture-recorded-only-in-its-empty-shape.md) holds the rest.
- **Two audits of one corpus at one date are byte-identical** (`two_audits_of_one_corpus_at_one_date_are_byte_identical`), and **the creator reading accounts for every half of this corpus** (`the_creator_reading_accounts_for_every_half_of_this_corpus`). These two run over this repository, so they assert a property and not a recorded report.
- **A finding comes from a declared input** (`a_finding_arrives_from_the_declared_window_and_leaves_when_the_date_moves` and `a_scope_pattern_with_an_ungoverned_entry_is_one_finding_that_lists_it`).
- **A wait ends when the corpus supplies what it waits on** (`a_wait_ends_when_the_corpus_supplies_what_it_waits_on`, `a_reading_that_still_waits_says_where_the_absence_lives`).
- **The store refuses a second reading at one lock and one date** (`a_reading_the_store_already_holds_is_not_appended_twice`, among the twelve unit cases of `engine/crates/audit/src/reading.rs`).
- **The colored audit strips to the plain audit** (`the_colored_audit_strips_to_the_plain_audit`), and **a painted name stays whole in the folded prose** (`the_layout_name_is_painted_inside_the_folded_prose`).

Record the audit report again with `HEADWATER_BLESS=1 cargo test -p headwater-audit --test readings`, and read the diff. A blessed fixture is the change.
