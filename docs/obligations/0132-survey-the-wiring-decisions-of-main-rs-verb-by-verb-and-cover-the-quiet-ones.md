---
id: HW-OBL-0132
status: current
status_since: 2026-09-06
last_verified: 2026-11-20
title: "Survey the wiring decisions of main.rs, verb by verb, and cover the quiet ones"
summary: "The survey of main.rs covers 50 command lines. One quiet wiring decision is still uncovered: the default of neighbors --top, which no fixture can reach."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
waiting_on: build
---

# Survey the wiring decisions of main.rs, verb by verb, and cover the quiet ones

## Context

`engine/crates/cli/tests/wiring.rs` holds one case each for `check --change`, `probe grade`, and the `Loaded::runs()` call site that feeds `generate`, from [#179](https://github.com/headwater-ai/headwater/issues/179). Each case was watched failing against the pre-fix form of a defect a person found only by reading the code after something else broke. None of the three defects moved an exit code, so nothing before that reading would have reported them.

A wiring decision is a place where `main.rs` chooses which library value to trust, rather than a place where a library computes one. The three fixes already made took three shapes. One read a rule at the wrong grain. One parsed an input and did not inject it where it needed to reach. One composed a value and handed it on with part of it dropped. `main.rs` runs to about 3,700 lines across thirty verbs, and nothing has enumerated where else it makes a decision of this shape.

Two further sites are already named. `probe stale` is the third caller of `Plan::gradable()`, and it is the one caller not already covered by type or by test. The `taxonomy vendor` doctrine report, added by [#81](https://github.com/headwater-ai/headwater/issues/81) after this issue was filed, makes two wiring decisions of its own. It re-derives `flattened` from `record.package` where `package::vendor` derives it from `identity`. That is safe only because `identity` refuses a record whose header disagrees with the manifest. It also reads the installed manifest with `manifest_at(&installed).ok()`, so a read error prints nothing at all. Both decisions were confirmed by hand and by nothing else, during construction and again during verification.

## Survey

The second slice of [#1494](https://github.com/headwater-ai/headwater/issues/1494) made this survey on 2026-11-20. `main.rs` was then 9,227 lines. Its `dispatch` matches 27 arms of `Verb`, and four of them take a second word from `JsonWord`, `SweepWord`, `ProbeWord` or `TaxonomyWord` in `engine/crates/cli/src/lib.rs`. So the table has 50 rows. 23 rows are the arms that take no second word, and 23 rows are the second words. The other 4 rows are the four verbs given no second word. The Discharge section below says thirty verbs, which was the count when this record was filed. The 50 command lines are the unit that the table uses now.

Two passes found the decisions. The first read each arm of `dispatch` and the function it calls. The second listed every site in `main.rs` that can turn a failure into a default: `.ok()`, `unwrap_or`, `unwrap_or_default`, `unwrap_or_else`, `filter_map` and `let _ =`. Such a site is a quiet failure when it is wrong. It found 81 sites. 37 of them are `let _ = writeln!` into a `String`, which cannot fail. The other 44 were read one at a time, and each decision they make is in a row below.

| command line | the decision `main.rs` makes | verdict | evidence |
|---|---|---|---|
| `check` | It builds `Asked` from the flags: the format, the cache, the change manifest and the read set | covered | `wiring.rs` `a_change_the_flag_named_reaches_the_verdict_and_not_only_the_parser`, and `cache_key.rs` `a_warm_check_gives_the_cold_verdict_after_the_lock_moves` |
| `change` | It refuses unless a base and a directory are both given | covered | `wiring.rs` `a_change_the_flag_named_reaches_the_verdict_and_not_only_the_parser` |
| `gate` | It reads each input of a published read set, and digests the claim store as a listing because it is a directory | covered | `import_pin.rs` `each_snapshot_file_joins_the_read_set_and_a_gate_sees_it_move`, and the three gate runs in `harvest.rs` |
| `derived` | none | decides nothing | It prints what the loaded producers derive |
| `site` | It refuses with no directory | covered | `site.rs` |
| `merge-driver` | It uses the current side and the path, and it ignores the ancestor and the other side | covered | `merge_driver.rs` |
| `route` | It joins the words of the task into one description | covered | `route.rs`, `governed_paths.rs` |
| `neighbors` | It joins the words of the task, and it sets `--model` to `.headwater/models` and `--top` to ten where neither is given | uncovered | The `--top` default is quiet, because a moved default exits 0. No fixture carries the model, so no case reaches the ranking. `governed_pipe.rs` reaches the default model directory only as far as a missing `model.onnx` |
| `explain` | It refuses with no target | covered | `classify.rs`, `show.rs`, `json.rs` |
| `show` | It refuses with no target | covered | `show.rs` |
| `mcp` | It gives the server the two write tools under `--write`, and no write tool without it | covered | `wiring.rs` `mcp_offers_the_write_tools_under_write_and_only_there`, added by this slice. `route.rs` drives the read tools |
| `new` | It refuses with no kind | covered | `new_agent_edge.rs`, `new_symmetric_far_half.rs` and 17 other test files |
| `capture` | It chooses the format from `--json` and `--format` | covered | `wiring.rs`, `json.rs` |
| `sweep`, with no second word | none | decides nothing | It refuses and names the second words, with exit status 1 |
| `sweep plan` | It reads no `--under` as the whole corpus | covered | `sweep.rs` |
| `sweep report` | It refuses with no path, and it chooses the format | covered | `sweep.rs`, `json.rs` |
| `sweep <other>` | none | decides nothing | It refuses with exit status 1 |
| `probe`, with no second word | none | decides nothing | It refuses and names the second words, with exit status 1 |
| `probe plan` | It narrows by tier, arm, action, moment, set and category, and it prints a delta, the folds, the oracles or the answer keys | covered | `wiring.rs` `probe_plan_folds_and_answer_keys_print_the_parsed_lists` and `probe_plan_oracles_print_the_parsed_declaration`, and `probe_facets.rs` |
| `probe record` | It reads the budget declaration, composes a plan and gives its selection to the intake. With no declaration there is no plan | covered | `wiring.rs` `probe_record_reads_a_moved_lock_transcript_through_the_plan_it_composed`, added by this slice |
| `probe grade` | It grades only through `Plan::gradable()`, and it grades each transcript against the part of the selection it was planned over | structurally impossible, and covered | `Plan::gradable()`. `wiring.rs` `a_plan_that_stopped_partway_grades_nothing_through_the_verb`, and `probe_grade_grades_a_part_planned_run_against_the_part`, added by this slice |
| `probe compare` | It grades both transcripts through the function that `probe grade` uses | covered | `probe_compare.rs`. The two `probe grade` cases hold the function they share |
| `probe stale` | It gives the intake `plan.gradable().ok()` as the selection | covered | `wiring.rs` `probe_stale_holds_a_whole_selection_transcript_against_the_whole` |
| `probe <other>` | none | decides nothing | It refuses with exit status 1 |
| `generate` | It gives the generator the plan and its refusal through `Loaded::runs()` | covered | `wiring.rs` `the_generator_is_handed_the_refusal_beside_the_selection` |
| `import` | It chooses the declared imports to read | covered | `wiring.rs` `a_refusal_the_consumer_declaration_caused_does_not_name_the_grammar`, and `import_pin.rs` |
| `export` | It writes the declared outputs, writes to standard output, or checks the outputs | covered | `export.rs` |
| `init` | It chooses the harness, the git configuration, or the declaration and the overlay | covered | `init.rs`, `init_harness.rs`, `init_hooks.rs` |
| `infer` | It sets `--until` to ninety days after the clock where no date is given, and it writes only under `--write` | covered | `wiring.rs` `infer_dates_its_debt_ninety_days_out_unless_until_names_a_date`, added by this slice, and `a_payload_this_verb_writes_loads_whatever_a_path_or_an_owner_holds` |
| `conformance` | It reads `--level` and chooses the format | covered | `conformance_lock.rs`, `json.rs` |
| `query` | none | decides nothing | It refuses every expression. `wiring.rs` `a_verb_this_engine_never_implemented_does_not_name_the_grammar` |
| `json`, with no second word | none | decides nothing | It refuses and names the second words, with exit status 1 |
| `json field` | none | decides nothing | It prints what `headwater_yaml::json::field` returns, and it refuses `None` with exit status 1 |
| `json count` | none | decides nothing | It prints what `headwater_yaml::json::count` returns, and it refuses `None` with exit status 1 |
| `json quote` | none | decides nothing | It prints the literal that `Json::string` renders |
| `json <other>` | none | decides nothing | It refuses with exit status 1 |
| `help` | It finds the command that the words name and prints its help | covered | `help.rs` `every_command_line_answers_both_help_flags_through_the_binary` |
| `completions` | none | decides nothing | It gives the shell to `clap_complete`. `completions.rs` holds the output |
| `taxonomy`, with no second word | none | decides nothing | It refuses and names the second words, with exit status 1 |
| `taxonomy validate` | It chooses what to print of the loaded repository | covered | `validate.rs` `validate_states_that_no_operation_makes_what_it_addresses` |
| `taxonomy resolve` | It carries the authored payload through the write of the lock. Under `--check`, a lock it cannot read compares as empty, which reports a move and exits 1 | covered | `wiring.rs` `a_payload_this_verb_writes_loads_whatever_a_path_or_an_owner_holds` |
| `taxonomy audit` | It drops the entries that git ignores from the scope | covered | `scope.rs` `the_audit_leaves_a_file_git_ignores_out_of_the_governed_scope` |
| `taxonomy publish` | It chooses between `--check` and a write | covered | `publish.rs` `a_named_assembly_publishes_one_flattened_package` |
| `taxonomy vendor` | It prints the installed directory and the doctrine path that `package::vendor` returns, and it reads no manifest | structurally impossible, and covered | `Vendored::under` and `Vendored::doctrine`. `wiring.rs` `taxonomy_vendor_reports_the_doctrine_directory_the_artifact_declares`, added by this slice |
| `taxonomy diff` | It compares the fetched artifact to the stated version | covered | `diff.rs` `a_version_bump_alone_moves_no_dimension` |
| `taxonomy migrate` | It writes only under `--apply` | covered | `migration.rs` `a_run_without_apply_writes_no_document` |
| `taxonomy graph` | It chooses the view | covered | `taxonomy_graph.rs` |
| `taxonomy kinds` | It chooses what to read beside the lock | covered | `taxonomy_kinds.rs` |
| `taxonomy <other>` | none | decides nothing | It refuses with exit status 1 |
| an unknown verb | none | decides nothing | It refuses with exit status 1 |

The table has 34 rows `covered`, 2 of them also `structurally impossible`, 15 rows `decides nothing` and 1 row `uncovered`.

**What the survey cost.** This slice added 5 cases to `wiring.rs`, which went from 34 cases to 39. Over three runs with the 5 cases skipped, the target ran in 0.47 to 0.48 seconds. Over three runs with them, it ran in 0.36 to 0.68 seconds. The cases run as threads of one process. So the 5 new cases moved the wall time by less than the spread between runs on a loaded host.

## Obligation

The survey below is done, and the two named sites have their cases. `probe stale` is covered. `taxonomy vendor` reads both halves of its doctrine path from `Vendored`, so it re-derives no name and reads no manifest.

One row is still owed. `neighbors` sets `--top` to ten where the flag is not given, and a moved default exits 0. No fixture carries the model that the verb loads, so no case of `crates/cli/tests/wiring.rs` reaches the ranking. The remedy is a fixture model small enough to commit, or a parser default that `main.rs` does not restate. Either one discharges the row.

Every other uncovered decision with a quiet failure now has a case in `crates/cli/tests/wiring.rs`, and each was watched failing against the form its defect names.

## Discharge

This is discharged when the survey exists as a table over all thirty verbs. Each row names a verb as covered, structurally impossible, or uncovered, and no verb is left unlisted.

It is discharged further when every uncovered decision whose failure is quiet has a case in `crates/cli/tests/wiring.rs`, each one watched failing first.

Where a decision can be made structurally impossible instead of tested, that treatment discharges it in place of a case. `Plan::gradable()` and `Unbound::bind` set that pattern already.

The two named sites, `probe stale` and the `taxonomy vendor` doctrine report, discharge on the same terms once their cases exist and pass.

This waits on an adopter to run the survey, choose the treatment for each finding, and add the resulting cases without adding a new gate.
