---
id: HW-SPEC-measurement
status: current
status_since: 2026-10-01
summary: "How probe, conformance and sweep plan and grade a probe run, evaluate a consumer rule set, and verify a sweep return file with no model."
last_verified: 2026-10-01
title: "Measurement"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft+revise
  evidence_basis: evidenced
relations:
  governs:
    - engine/crates/probe/src/**
    - engine/crates/conformance/src/**
    - engine/crates/sweep/src/**
  traces_to:
    - HW-SPEC-engine-architecture
    - HW-SPEC-ai-integration
    - HW-SPEC-the-recorder-contract
    - HW-SPEC-assurance-model
    - HW-SPEC-check-layer
    - HW-SPEC-distribution-and-federation
    - HW-IFACE-headwater-probe
    - HW-IFACE-headwater-sweep
    - HW-IFACE-headwater-conformance
    - HW-DR-0008
    - HW-DR-0024
    - HW-DR-0076
---

# Measurement

## Scope

This spec describes the inside of the measurement subsystem of [spec 6](../spec/06-engine-architecture.md#subsystems). Three crates under `engine/crates/` build it: `probe`, `conformance` and `sweep`.

Each crate measures a different thing:

- `probe` plans a probe run, reads back the transcript of the run, grades it, and tells which committed results an edit made stale.
- `conformance` evaluates a consumer repository against the rule set that its taxonomy package ships, and reports the level that the met rules reach.
- `sweep` writes the briefing for a coherence sweep and verifies the return file that an agent wrote.

The verbs over these crates are `probe plan`, `probe record`, `probe grade`, `probe stale`, `conformance`, `sweep plan` and `sweep report`. Their bodies are in the `cli` crate, which belongs to the Command surface row of spec 6. This spec describes the libraries that those bodies call.

Other documents state what the subsystem does, and this spec does not repeat them:

- [Spec 5](../spec/05-ai-integration.md#measuring-whether-any-of-this-works) states what a probe is, the run identity, the tiers, and what earns the grader its right to grade.
- [Spec 15](../spec/15-the-recorder-contract.md) states the transcript that a recorder writes and the keys of each block.
- [Spec 4](../spec/04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep) states the sweep, its four constraints and the [finding shape](../spec/04-assurance-model.md#findings).
- [Spec 7](../spec/07-distribution-and-federation.md#conformance) states conformance, levels and waivers.
- [Spec 12](../spec/12-check-layer.md#four-things-stop-a-sweep-from-gating-and-none-of-them-is-a-rule-that-somebody-keeps) states the four facts that stop a sampler from gating.
- The contracts of [`headwater probe`](../interfaces/headwater-probe.md), [`headwater conformance`](../interfaces/headwater-conformance.md) and [`headwater sweep`](../interfaces/headwater-sweep.md) state the flags, the output and the exit status of each verb.
- [HW-DR-0008](../decisions/0008-probe-cost-and-cadence.md) rules the cost and the cadence of probes. [HW-DR-0076](../decisions/0076-a-probe-budget-prices-a-run-identity-fixed-before-the-run-and-a-committed-transcript-and-a-sweep-has-neither.md) rules what a budget prices. [HW-DR-0024](../decisions/0024-q24-readability-and-what-a-sweep-can-be-asked-about.md) rules what a sweep can be asked.

The public Rust API of each crate is not in this spec ([HW-DR-0098](../decisions/0098-an-engine-subsystem-is-described-by-a-technical-design-spec-on-a-shelf-of-its-own-and-its-behavior-stays-where-it-is-already-written.md)). The doc comments of the crates state it.

## Design

### What the three crates share

No code in the three crates reaches a model. A probe run and a sweep each need a model in the middle. The engine does the parts before and after the middle part, and a recorder or an agent does the middle part. So no build waits for a model. A model that nobody can reach means only that no file came back.

Each of the three crates names `headwater-check` in its `Cargo.toml`. `probe` and `sweep` name it for the rule set and the finding shape. `conformance` names it for `Date` and for the `fill` and `paint` modules of its text renderer. So `headwater-check` can name none of them, because the compiler refuses a cycle. No rule reads a measurement, and `check --strict` cannot see one.

Two crates outside this subsystem name one of these crates as a dependency, and the `Cargo.toml` of each one states why:

- `cli` links all three, because the verbs are there.
- `generate` links `probe`. A probe result is a projection over a committed transcript. So each `generate` run and each `generate --check` gate calls the intake and the grader.

`conformance` links `generate`, because the `projections.current` reading calls `headwater_generate::check`. So `conformance` reaches `probe` through `generate`, and `compat` does the same, because it links `generate` too.

None of the three links `headwater-fetch`, which is the crate that holds the HTTP client. `cli` links it under an optional feature, and `engine/crates/cli/tests/network_boundary.rs` holds the crate graph to that ([HW-DR-0075](../decisions/0075-the-vendor-verb-may-take-a-location-and-the-fetch-lives-only-in-a-crate-the-checking-loop-never-links.md)).

### The probe vocabulary is four closed sets

`probe/src/lib.rs` holds the four enums that a probe document and a budget name: `Category`, `Expectation`, `Tier` and `Arm`. Each one is closed, because a grader evaluates it and a report totals it. Each enum has an `ALL` constant that a person keeps by hand, and its `read` function looks a name up in that constant. [HW-OBL-0172](../obligations/0172-nine-hand-kept-constants-enumerate-an-enum-and-nothing-holds-one-against-the-variants.md) records that nothing holds the constant against the enum.

`Expectation::names_documents` marks the three forms that are predicates over the documents that a probe `examines`. `Tier::pairs_arms` marks the tiers that run two arms, so a caller asks the tier and does not compare it with `Tier::Campaign`.

### A plan fixes the run identity before the run

`plan.rs` takes a `Plan` over the census, the graph, the rule set, the budgets and the lock digest. `Plan::over` never fails. A run that does not happen gives a plan that holds a `Refusal` and the numbers that caused it. The reason is what the caller needs.

The plan fixes the members of the run identity that exist before a run, which [spec 5](../spec/05-ai-integration.md#the-harness-confirms-five-things-and-one-component-after-it-grades) lists. Three of them are digests, and each digest has one function:

- `selection_digest` takes the selected identifiers in order. A grade takes it again to find which part of a selection a transcript was planned over.
- `tree_digest` takes the path and the content digest of every typed row of the census. It moves on every commit.
- `read_set_over` takes the probes of the selection and the documents that they examine. It moves on an edit to one of those documents, and on no other edit.

The seed is the number that the caller states. A seed derived from the corpus changes the rotation whenever the corpus changes. Two runs then differ in phrasing and in corpus together, and no reader can separate the two causes.

Every `Refusal` stops the whole run and never one probe. A selection that dropped a probe in silence reports a rate over a denominator that nobody declared. `Refusal::stops_a_grade` is an exhaustive match that tells which refusals also stop a later grade, so a new refusal must answer that question. [HW-OBL-0155](../obligations/0155-probe-s-fourteen-refusals-have-no-coverage-test-and-three-of-them-are-named-by-no-test-at-all.md) records the refusals that no test names.

### A budget is a policy, and nothing recomputes it

`budget.rs` reads `.headwater/probe.yml` into `Budgets`, which holds one `Envelope` for each tier. An envelope holds the ceiling, the believed cost of one session, the repetitions, the arms and the paths that an ablated arm removes. `Budgets` also holds the instrument paths that every arm removes, the folds and the answer keys.

The file is outside the corpus root. No rule reads it and no kind classifies it, so it is a policy of the repository and not a fact about a document. Every field is required except `max_turns`, because a default is the engine deciding how much money to spend. `Unreadable` names each way that the file fails, and a tier with no usable envelope does not run.

`session_cost` is the estimate of a person. No run computes it, so the projection of a plan is arithmetic over a declared number and not a forecast.

### The intake confirms a transcript and grades nothing

`intake.rs` reads a `probe_transcript` document into a `Record`. `Record::read` is a pure function of the transcript, the tree and this code. It confirms the five things that `CONFIRMATIONS` names: the taxonomy, a complete identity, the membership of every probe, no prose, and a recorded cost.

The "no prose" confirmation is four closed key sets: `IDENTITY_KEYS`, `EVENT_KEYS`, `CALL_KEYS` and `PRODUCED_KEYS`. One key outside them refuses the file. `engine/crates/probe/tests/contract.rs` holds the four sets equal to the four tables of spec 15, so the contract and the code cannot drift.

A moved lock refuses a transcript only where the read set moved too. A lock move that reaches no probe and no examined document changes no verdict, so the intake does not void a batch for it.

`Record` carries no verdict. A record that carried one is a grader in a place where no reviewer looks for one.

### The grader is a pure function, and its version is its own

`grade.rs` evaluates the expectations of the selected probes over one `Record`. `Results::over` reads the record and the selection and nothing else. It opens no file and compares no arms.

`grade.rs` is the one module of this subsystem that evaluates an expectation. Three properties of its types permit it:

- Its inputs carry no prose. A transcript has only closed keys, and an expectation is one of five predicates.
- `Verdict::Satisfied` holds a `Witness`, so a satisfied verdict with no event behind it does not compile. A `Miss` holds the extent of the search.
- Where a pass costs nothing, it refuses. Three examples are a `patched` form with no oracle, an artifact that nothing checked, and a `not_opened` over a session with no call. Each gives a `Refusal` and never `Satisfied`.

`Refusal::is_session` separates a refusal that the session caused from one that the recorder or the declaration caused. A session refusal counts for its arm. A recorder refusal is a defect, and the remedy is to record the session again.

`VERSION` is the version of the grader and not the release of the engine. A change that moves a verdict or the rendered grade moves it, and a release does not. Each result prints it, so a series over two graders shows the change. `Interval` gives a Wilson interval for each rate and a Newcombe interval for the difference between two arms.

### A read set has two halves, and each half has a different judge

`read_set.rs` tells which committed results an edit made stale, for `probe stale`. The tree digest is the wrong instrument for this, because it moves on every commit and so marks every result stale.

The declared half is a digest. The recorder copies `Plan::read_set` into the identity, so a recomposed digest either matches or does not. A digest names no document, and the observed half names one.

The observed half is decided path by path. Each recorded call carries the path that it opened and the identity of what came back, so the call is a `Witness` for one document. A call can also reach a document that no probe examines, and `Provenance::Opened` marks that member. A Bash call names a document and never witnesses it, because its result is not the content of one file.

Each `Member` takes a `Verdict`, and `Tally` counts them. Where the digest holds and a witness disagrees, the recorder wrote an identity that is not the content digest of this engine. `Staleness` reports that as a fact about the recorder. A session with no `calls` key gives no witness, and the report names it on its own line.

`Staleness::render` returns a string. It has no status and no error arm, so no caller can fail a build on a stale result.

### A conformance rule is two halves in two places

The package ships the rule: its name, its text and its remediation. `conformance/src/lib.rs` holds the code that decides it against a tree. `read` and `at` load the rule set into a `RuleSet`. A tree rule whose name is not in `READINGS` ends the run with `SetError::NoReading`. A level over a rule that nothing evaluated is a false green.

`reading` sends each name of `READINGS` to its function: `pin_current`, `lock_current`, `corpus_classified` and `projections_current`. `lock_current` calls the comparison of `taxonomy resolve --check`, and `projections_current` calls the comparison of `generate --check`. So a reading and a verb never give two answers to one question. A rule that `DecidedBy::Attestation` marks gets `Verdict::NotDecided` and no reading.

`evaluate` takes the verdicts, and `assemble` computes the levels from them. `assemble` reads no tree, so a test can hold the ladder without a corpus. `pin_check` runs before `assemble`. It tells which rule read the pin, or what the installed release record declares where no rule read it.

### A level is not a score, and a waiver cannot buy one

A `Level` is a named subset of the rules, and no key declares one. `waivers` reads the `conformance` block of the consumer declaration and refuses every key other than `waivers`. A level with no rule is `SetError::EmptyLevel`.

`LevelState::reached` reads `Verdict::Met` and no waiver. `Reading::passes_gate` answers the other question, which `--level` asks, and a live waiver answers it. So a waiver can make the gate green, and it never moves a rung. A `Waiver` has four required fields. An expired waiver is `Cover::Expired`, which covers nothing.

The waivers live in the consumer declaration and not in the package. `taxonomy vendor` replaces a vendored package whole, so a waiver beside the shipped rule is lost at the next upgrade.

### Two renderers of one conformance report

`render.rs` writes the report for the person who closes a gap. It prints the package identity above every rung and the remediation under every gap. It fills with `headwater_check::fill` and paints after the fill, because an escape sequence counted as text moves every break.

`json.rs` writes the same `Report` for a program. It keeps `reached` and `gate` apart, and it writes `gate` only under `--level`. Each function destructures its source in full, so a new field on `Report` that `json.rs` omits does not compile.

### A sweep is three parts, and this crate is two

`sweep/src/lib.rs` holds the closed set `Class` and its hand-kept `ALL`, which HW-OBL-0172 counts. Each class names two things that do not fit, so a reader can judge it in seconds. `PROVENANCE` is `"agent"` on every finding.

`plan.rs` writes the briefing. A `Plan` carries pointers: paths, identifiers, kinds, titles and summaries. It never carries the content of a document. The agent opens each file itself, because a passage that it never opened is a passage that it cannot cite. The plan also lists, as `Declared`, every edge that the graph already holds among its members. The plan states its extent as its members against the classified documents of the corpus.

`intake.rs` reads the return file into a `Report`. `Report::read` is a pure function of the file, the tree and this code. It refuses the whole file when the file does not parse or names another lock. It refuses one finding with a `Reason` for an unknown class or for a path that is not a classified document. It also refuses a quotation that the document does not hold. It also refuses a proposed edge that the taxonomy does not admit. A finding whose class implies a relation that the graph already declares is refused too, because a restated edge is a defect of the sweep.

`locate` finds a quotation after `collapse` reduces each run of whitespace to one space on both sides. Nothing else is relaxed, so a paraphrase is refused. A finding that passes is `Verified`, and a proposal prints the front matter that declares its edge. The crate opens no file for writing.

### The sweep JSON is the finding shape with two more members

`json.rs` writes one sweep in the finding shape of spec 4, with `provenance` and `evidence` on each finding. `headwater_adapter::json` writes the same finding shape for a run of the checks, without those two members. The two are separate functions over separate types, because `headwater-check` cannot name this crate and so a `Run` cannot hold a sweep finding. The `sample` member states in the artifact that the set is a sample, so a consumer that counts findings meets the warning.

## Invariants

A change to these crates must keep each of these. A test holds each one that names a test.

- **A plan, a record and a grade are each the same bytes twice** (`a_plan_a_record_and_a_grade_are_each_the_same_bytes_twice` in `engine/crates/probe/tests/fixtures.rs`). The sweep holds the same for a plan and a report (`a_plan_is_the_same_bytes_twice`, `a_report_is_the_same_bytes_twice` in `engine/crates/sweep/tests/fixtures.rs`).
- **Every satisfied verdict carries a witness that a reader can check** (`every_satisfied_verdict_carries_a_witness_a_reader_can_check`). **A key that the recorder never wrote reaches a refusal and never a pass** (`a_key_the_recorder_never_wrote_reaches_a_refusal_and_never_a_pass`).
- **A key outside the closed sets refuses the transcript** (`a_key_outside_the_closed_set_refuses_the_transcript`). The closed sets equal the tables of spec 15 (`the_four_tables_of_the_contract_are_the_four_closed_sets_of_the_intake` in `engine/crates/probe/tests/contract.rs`).
- **The read set is not the corpus tree** (`the_read_set_of_the_fixture_selection_is_not_the_corpus_tree`). **An edit that the read set does not cover voids no result** (`an_edit_the_read_set_does_not_cover_voids_no_result`). **An absent `calls` key is not a session that opened nothing** (`an_absent_calls_key_is_not_a_session_that_opened_nothing`).
- **A tree rule with no reading ends the run** (`a_tree_rule_this_engine_holds_no_reading_for_ends_the_run` in `engine/crates/conformance/tests/rules.rs`), and **a live waiver moves the gate and never the level** (`a_live_waiver_moves_the_gate_and_never_the_level`).
- **A quotation that the document does not hold is refused** (`a_quotation_the_document_does_not_hold_is_refused`), **a paraphrase is not a quotation** (`a_paraphrase_is_not_a_quotation`), and **a restated edge is refused** (`a_finding_that_restates_a_declared_edge_is_refused`).
- **No check names the placeholder document, and the sweep does** (`no_check_names_the_placeholder_document_and_the_sweep_does`). This pair holds that the sweep finds what the checks cannot.
- **The recorded plans, records, grades and reports are the assertion.** Record them again with `HEADWATER_BLESS=1` and read the diff. Move `grade::VERSION` with any change that moves a verdict.

The probe grader is a correctness root ([spec 12](../spec/12-check-layer.md#the-correctness-roots)). A grader that evaluates a predicate wrongly gives a systematically green rate, and no probe finds it. The recorded fixtures in `engine/crates/probe/tests/fixtures.rs` hold each verdict form and each refusal of the grader.
