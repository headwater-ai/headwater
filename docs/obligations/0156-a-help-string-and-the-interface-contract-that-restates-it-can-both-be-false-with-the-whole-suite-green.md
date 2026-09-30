---
id: HW-OBL-0156
status: current
status_since: 2026-09-06
summary: "Two suites read the shape of the help and the headings of the contract, and neither reads a word of either."
last_verified: 2026-10-01
title: "A help string and the interface contract that restates it can both be false with the whole suite green"
waiting_on: ruling
---

# A help string and the interface contract that restates it can both be false with the whole suite green

## Context

`engine/crates/cli/tests/help.rs` holds every argument of this binary to carrying help text. It reads whether a string is there. It never reads what the string says.

`engine/crates/cli/tests/interface_contract.rs` holds each document under `docs/interfaces/` to its headings. It declines to read the content, and it states the ground. A kind whose whole purpose is a description must not imply that a check read the description.

So the two artifacts that describe one verb answer to two suites. Neither suite reads a word of either.

[The help-string audit](../reviews/the-sixty-four-restored-help-strings-checked-against-the-binary.md) is the measurement. It ran all 64 help strings that PR #335 carried whole, against the binary, and found three false.

One more defect sat outside that set, and it is the clearest instance. The help for `check --change` never named the `headwater change 1` header. `engine/crates/check/src/change.rs` requires that header, and `docs/interfaces/headwater-check.md` omitted the same line. Both copies were wrong for nine days, across a parser rewrite. `headwater check --strict`, four fixture suites and every CI job passed over them.

Three of the 64 now have a case in `engine/crates/cli/tests/wiring.rs`. Each one fails when the sentence goes wrong. The other 61 are true today, and nothing reads them tomorrow.

## Obligation

The corpus owes a statement of whether a rule can reach this class of defect. It owes the reasoning either way. Issue [#339](https://github.com/headwater-ai/headwater/issues/339) put the general case outside its own scope, and no document answers it.

The hard part is that one side of the comparison is English prose, and the other is engine behavior. Nothing this engine carries compares the two. So the answer may be that no rule reaches it. That answer is worth writing down. A reader who assumes the check layer covers a claim about the engine is reading a guarantee that does not exist.

The cheaper part is the duplication, and [#1487](https://github.com/headwater-ai/headwater/issues/1487) built part of it. `every_options_table_names_the_flags_the_parser_admits` in `engine/crates/cli/tests/interface_options.rs` reads the long flags that `headwater_cli::command()` admits on each command line. It compares them, in both directions, with the flags that the `## Options` table of the interface document names. It compares the placeholder of each flag too, such as `--change <manifest>`. A second case holds `headwater-help.md` to the global flags. On the day they landed, the cases found two tables that named a `--format` value the parser does not print.

So three things are true now. The new cases hold that each row exists and that its placeholder is right. Nothing reads the words of a row or of a help string. So a row and its help string can still both be false with the whole suite green. The first half, whether a rule can read a claim about this engine, waits on the owner.

## Discharge

A ruling that states whether a rule can read a claim about this engine, and what it would read.

If the ruling is that no rule can, then the sentence that says so discharges this record. That sentence names what stands in place of a rule, which today is one case in `wiring.rs` for each corrected string.

A rule or a case that holds the words of a help string against its row discharges the second half alone. A case that holds the flag name and the placeholder alone does not.
