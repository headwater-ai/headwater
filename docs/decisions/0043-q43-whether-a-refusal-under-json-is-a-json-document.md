---
id: HW-DR-0043
status: current
status_since: 2026-08-30
summary: "A refusal is an English sentence on standard error and never a JSON document, because `--json` names the shape of an artifact and moves no stream and no grammar."
last_verified: 2026-08-30
title: "Q43 — Whether a refusal under `--json` is a JSON document"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0033
  governs:
    - engine/crates/cli/src/main.rs
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/tests/json.rs
---

# Q43 — Whether a refusal under `--json` is a JSON document

## Context

**[#346](https://github.com/headwater-ai/headwater/issues/346) measures ten command lines that name a JSON target and then refuse.** Each one exits 1, writes zero bytes to standard output, and writes one English sentence to standard error. The verbs are `explain`, `gate`, `route`, `conformance`, `sweep report`, `export` and `check`. A consumer that reads standard output on any of those runs reads nothing at all.

**The behavior is deliberate, and no document a reader meets states it.** [Spec 6](../spec/06-engine-architecture.md) says what `--format` puts on standard output and stops there. One interface contract states the rule for one verb. `docs/interfaces/headwater-explain.md` says that a missing target writes its refusal to standard error and no explanation to standard output. Seven other contracts are silent.

**So the question is whether `--json` is a promise about every byte of a run, or about the artifact alone.** One answer makes a refusal a JSON document, so that a consumer holds one grammar for one verb. The other keeps a refusal as prose on the other stream and writes the rule down where a reader meets it. Issue #346 asks for a ruling before any code.

## Decision

**A refusal is an English sentence on standard error, and it is never a JSON document.** `--json` names the shape of an artifact. It moves no stream and no grammar. The engine behavior stands as it is, and the contracts of the nine verbs state it. Four grounds carry the ruling, and the third one settles it.

**One — the split between an artifact and an account is a rule this engine already states.** [Spec 6](../spec/06-engine-architecture.md) says of `--fix` that the account of what was written goes to standard error, because `--format` puts one artifact on standard output. The doc comment of `export` says the same for its census, so that a redirected artifact is the artifact and nothing else. A refusal is an account. A JSON refusal on standard output would contradict a rule this engine states twice for other reasons.

**Two — the refusal architecture is uniform across the whole binary.** `engine/crates/cli/src/main.rs` holds three refusal helpers, `fail`, `refuse` and `defect`, across 46 and 37 call sites. Every one of them writes an English sentence to standard error. A JSON refusal would make each of those sites read the command line for `--json`. That coupling is what `chosen` exists to prevent, and [HW-DR-0033](0033-q33-whether-the-command-line-is-derived-and-who-a-flag-belongs-to.md) refused the same shape for `--wide`.

**Three — the alternative cannot keep its own promise, and this ground settles the fork.** The benefit claimed for a JSON refusal is one grammar for one consumer, and it cannot be delivered. `clap` refuses `check --json --format json` before the verb is entered, and it refuses a flag with no value the same way. [HW-DR-0033](0033-q33-whether-the-command-line-is-derived-and-who-a-flag-belongs-to.md) records that the parse goes through `try_parse` only to hold exit 1 against the 2 that `clap` returns. To render those as JSON, the binary must catch every `clap` error at a point where the verb and the target are not reliably known. A consumer would still meet an English sentence on some command lines. That is two grammars, plus a new rule about which refusal is which, which is worse than two grammars that one sentence separates.

**Four — the house wording exists already, on one contract.** `docs/interfaces/headwater-explain.md` carries it for a missing target. The seven other contracts are missing a sentence rather than waiting on a design.

## Consequences

**Nine interface contracts state the rule under `## Exit status`.** The ninth is `headwater taxonomy`, whose `publish` took `--json` under [#353](https://github.com/headwater-ai/headwater/issues/353) after this ruling. The overlay declares a closed set of eight headings for an `interface_contract`, and `section.required.missing` reads that set. So the sentence goes inside a heading that exists, and no contract gains a ninth heading of its own.

**`check` takes a two-case form, because "standard output is empty on a non-zero exit" is false for it.** Five of the eleven reasons `check` exits 1 are decided after the report is written. `check --json --read-set /no/such/dir/run.readset` exits 1 and puts a whole JSON report on standard output. So the property is that a refusal writes nothing, and not that a non-zero exit writes nothing. `docs/interfaces/headwater-check.md` draws that line for the text report, and the same line holds for `json`, `sarif` and `markdown`.

**`sweep` takes the unconditional form.** All four of its reasons for exit 1 are refusals, and each one is decided before anything is written.

**A refusal names the spelling that the caller typed.** `chosen` in `engine/crates/cli/src/main.rs` folds `--json` onto `--format json`, so the two write one artifact byte for byte. A message a person reads is not an artifact. Issue #346 measures two refusals of `export` that quote `--format` at a caller who wrote `--json`. That message sends a reader to a flag that is not on the command line in front of them. The helper `typed` carries the name past that fold rather than through it. The two travel as different types, so a site that reached for one and got the other does not compile.

**Two fixtures hold the ruling, and together they state a boundary rather than one side of it.** `a_refusal_writes_no_document_and_accounts_for_itself_on_the_other_stream` reads fourteen refusing command lines in both spellings. `a_run_that_completed_and_then_failed_still_wrote_its_document` reads the two `check` runs that exit 1 with a whole document on standard output. Without the second one, a change that suppressed the report on any non-zero exit would pass the first.

**Nothing here rules on the exit status.** `--json` moves no exit status, `the_json_form_moves_no_exit_status` holds that, and this record does not touch it.

**One cosmetic gap stays open.** The `--at` refusal of `export` tells a caller who named no target to name one with `--format`, and it does not mention `--json`. That branch is unreachable when `--json` is given, so the message is correct where it stands.
