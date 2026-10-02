---
name: hw-corpus
description: The rulings, paths and kinds of this repository's own corpus that the four shipped skills and the maintainer agent do not carry, because an adopter has none of them. A settled document goes into its pull request at `status: current`, not `draft` (HW-DR-0052). Use beside headwater-authoring, headwater-taxonomy, headwater-sweep, headwater-orient or headwater-maintainer whenever the work is on this repository's corpus.
---

# This corpus, beside the shipped skills

`headwater init --harness` writes the four shipped skills and the maintainer agent from text compiled into the binary, and no file of that set names this corpus ([the harness step](../../../docs/interfaces/headwater-init.md#the-harness-step)). Edit `engine/crates/cli/harness/`, rebuild, and run `headwater init --harness`. Never edit an installed copy, because `headwater init --harness --check` fails on it. This file carries what this repository adds to each one. It restates nothing the shipped text says.

## Beside headwater-orient

`.claude/hooks/intent.sh` runs `headwater route` on every prompt, and prints the documents it matched above the first tool call.

## Beside headwater-authoring

The concrete kinds are `acceptance_criterion`, `decision`, `decision_register`, `design_spec`, `evaluation`, `explanation`, `how_to`, `interface_contract`, `library_doctrine`, `obligation_record`, `obligation_register`, `probe`, `probe_result`, `probe_transcript`, `process_decision`, `process_evaluation`, `process_explanation`, `process_obligation`, `process_spec`, `requirement`, `review_prompt`, `review_record`, `specification`, `subsystem_spec`, `tutorial` and `verification`.

A `Write` of a new document under `docs/` is refused by the `PreToolUse` hook, which names `headwater new`. An `Edit` of a document that already exists passes, and that is the repair path: scaffold first, then edit the file the verb wrote. [HW-OBL-0106](../../../docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md) records that no rule reads a shelf layout after birth.

**An obligation record keeps three conventions, because a later run re-measures it.** An `obligation_record` states the context, what the corpus owes, and what would discharge it. The Discharge section states the closing condition, what must become true, and not a description of the present state. A record names a function, a job key or another stable symbol, never a line number or a count: records that named a function or a job key held over weeks, and records that named a line or a count went false (#1486). A record that bundles gaps is split when it passes five gaps.

**`obligation_record` meets `FacetUndeterminable` by design.** It requires `waiting_on`, which is closed to `ruling`, `build`, `measurement` and `adopter`, and no role derives a value for it. Read the Discharge section you are about to write, and pass `--facet waiting_on=<value>`. [HW-DR-0030](../../../docs/decisions/0030-q30-whether-what-an-obligation-waits-on-is-a-state-or-a-property-and-how-many-values-it-takes.md) states what each value means.

**The provenance block.** No taxonomy declares the members after `warrant` — [HW-OBL-0030](../../../docs/obligations/0030-the-provenance-block-belongs-to-the-engine-and-nothing-states.md) holds that gap.

**The `accepted_by` line answers to the merge rather than to the byte you write.** [HW-DR-0034](../../../docs/decisions/0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md) rules that acceptance is the merge onto `main`, so a provenance block on a branch states a proposal and nothing more. You may write `accepted_by` and `warrant: accepted` on a document you draft, where that document goes into a pull request a named human reads before the merge. Say in the proposal that the stamp is part of what you are asking them to accept. Where nobody will read the document before it lands, and an autonomous run that merges its own request is that case, write `warrant: asserted` and no `accepted_by`. [HW-OBL-0108](../../../docs/obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md) holds the count that raised the ruling.

**The `status` line answers to the merge the same way.** [HW-DR-0052](../../../docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md) rules that an author writes the state the document will hold once the branch lands, and that the merge activates it. Move it to `current` before you propose a document as finished, and leave `draft` only where you are asking a reader to comment rather than to rely. A merged document standing at `draft` is the defect that ruling was written for, and copying the front matter of a sibling is how it spreads.

**`governs`, `traces_to` and `cited_in` declare `created_by: agent`, and an agent writes them with `headwater new --relates`.** [HW-DR-0104](../../../docs/decisions/0104-an-agent-writes-governs-traces-to-and-cited-in-through-the-verb-and-the-review-of-its-pull-request-is-the-acceptance.md) is the ruling: the merge ruling on a reviewed pull request accepts each edge. [HW-OBL-0105](../../../docs/obligations/0105-nothing-plays-the-hook-role-that-two-relations-name.md) recorded the two relations that named `hook` where nothing mechanical wrote them. A `code_path` anchor is a pattern in the language of `headwater_meta::pattern` ([HW-DR-0074](../../../docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md)). The far halves the verb writes and leaves are [HW-DR-0086](../../../docs/decisions/0086-a-reciprocal-half-is-owed-once-its-writer-leaves-its-initial-state.md) and [HW-DR-0101](../../../docs/decisions/0101-new-writes-no-far-half-of-a-symmetric-relation-and-no-state-on-a-supersedes-target.md).

Prose under `docs/spec/`, `docs/subsystems/`, `docs/decisions/`, `docs/evaluations/`, `docs/obligations/`, `docs/interfaces/`, `docs/requirements/`, `docs/acceptance-criteria/`, `docs/tutorials/`, `docs/how-to/` and every shelf under `docs/process/` answers to the `ste_house` language regime. Invoke the `ste-editor` skill for the rules that no check reads. A probe result over an edited document is under `docs/probe-results/`.

Adding or removing a document under `docs/` moves three recorded fixtures. Run `cargo test --workspace --manifest-path engine/Cargo.toml`, re-record with `HEADWATER_BLESS=1`, and read the diff: `check --strict` and `generate --check` both pass while all three are stale.

## Beside headwater-taxonomy

Four sources resolve into this repository's lock, in this order.

| source | what belongs there |
|---|---|
| `taxonomy-source/headwater-standard/taxonomy.yml` | the invariant core, and what every adopter of the package gets. `.headwater/packages/headwater-standard/` is the vendored copy of it. Edit here and republish. Never edit there |
| `docs/taxonomies/design-spec/bundle.yml` | the specification tradition: numbered parts, registers, reviews |
| `docs/taxonomies/decision-record/bundle.yml` | the decision-record tradition: one decision per document, obligation records |
| `.headwater/overlay.yml` | what is true of this repository and of no adopter |

Each bundle carries a `doctrine.md` beside it that states why every declaration in it is there. A declaration added to a bundle owes a paragraph in its doctrine. `taxonomy resolve --check` is a blocking CI step. [HW-OBL-0105](../../../docs/obligations/0105-nothing-plays-the-hook-role-that-two-relations-name.md) recorded two base relations that named `hook` where nothing mechanical wrote them. [HW-OBL-0107](../../../docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md) records the kind the base package ships that the scaffolder refuses.

**Adding a rule.** A rule is declared in the taxonomy and implemented in `engine/crates/check/`. **A check without a failing fixture does not ship.** The fixture is the evidence that the rule can fire at all. A green run over the corpus is not that evidence. That is the firing condition, and it is the floor. **A check also owes the discriminating condition: an input on which the implemented rule and a plausible neighboring rule disagree.** Name the neighboring rule, and point at the fixture that separates the two. [Spec 12](../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship) states both conditions and carries the worked case, `link.fragment.unresolved` before #209.

After a taxonomy change, also run `cargo test --workspace --manifest-path engine/Cargo.toml`, and re-record with `HEADWATER_BLESS=1`. **Two of the things a change moves are not recorded fixtures, so a bless run leaves both failing.** `engine/crates/ref/tests/fixtures.rs` asserts by hand how many `add` operations this repository's overlay and the design-spec bundle declare between them, and the number is a Rust literal that you edit. This file prints the concrete kinds, and `.claude/skills/fixtures.sh` compares that list against the list `headwater new` admits, so a new kind is an edit here. Neither is reached by `HEADWATER_BLESS=1`, and `cargo test` stops at the first target that fails, so fix them one at a time and count the targets that ran rather than the failures.

## Beside headwater-sweep

[Spec 4](../../../docs/spec/04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep) says that structural checks cannot discharge coherence. No gate and no CI job runs the sweep. [HW-OBL-0113](../../../docs/obligations/0113-every-check-passes-a-document-that-is-still-the-scaffolder-s-placeholder.md) is why `unwritten_section` exists. [Q24](../../../docs/decisions/0024-q24-readability-and-what-a-sweep-can-be-asked-about.md) rules out a readability class and a source-file member, and a readability pass here uses the `ste-editor` skill. The owner ruled on [#496](https://github.com/headwater-ai/headwater/issues/496#issuecomment-5770898999) on 2026-09-22 that a stated practice no path backs is a sweep pattern and not a check rule. Here, add `grep -v -e '^docs/reviews/' -e '/fixtures/'` after the `git ls-files` of its command. [The n8n evaluation](../../../docs/evaluations/n8n-worked-example.md#what-this-taxonomy-would-report-and-what-it-does-not) records one run of it over this repository, with its denominator. A finding goes to an obligation record, or to one line of the run's intake file during a build-order run.

## Beside headwater-maintainer

Build the engine of the tree you review through `tools/hw-cargo`, never a bare `cargo`, with your own reserved slot:

    HW_CARGO_SLOT=maintainer sh tools/hw-cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked

When `hw-build` dispatches you, set `HW_CARGO_SLOT=maintainer-<N>` for the build's issue instead, because builders run in parallel and two reviews on one slot share its target directory. Remove `~/.cache/headwater/cargo-pool/target-maintainer-<N>` and its `.root` file when you report. One review of a taxonomy change ran a two-day-old binary from a scratch directory and reported five errors on a document that has none.

For the `governs` half, run the write hook the way the harness does, once per changed path:

    printf '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":"<path>"}}' | sh .claude/hooks/write.sh

It answers by string equality against the value on each edge, and a pattern edge answers for each entry it admits. A silent result is not evidence that no document governs the area — see HW-OBL-0104.

A crate file that an `interface_contract` governs answers with the contract, and each answer carries in parentheses the name the document declares. For a contract that name is the command a caller types, so `engine/crates/check/src/lib.rs` answers `docs/interfaces/headwater-check.md (headwater check)`. Report the command rather than the path, because the path is a file name somebody chose and the name is a declaration the taxonomy reads.
