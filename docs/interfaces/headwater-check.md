---
id: HW-IFACE-headwater-check
status: draft
status_since: 2026-08-16
summary: "What headwater check reads, what goes to each of its two streams, and the eleven causes behind its one non-zero exit."
last_verified: 2026-08-24
title: "headwater check"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - engine/crates/cli/src/main.rs
    - engine/crates/check/src/lib.rs
---

# headwater check

## Synopsis

    headwater check [--strict] [--fix] [--no-cache] [--now <date>]
                    [--change <manifest>]
                    [--read-set <path>] [--register <path>]
                    [--format text|json|sarif|markdown | --json]
                    [--root <path>] [--no-color]

The verb takes no operand. A word after `check` is refused, and the message names `check` as a verb the binary does not carry.

## Description

`headwater check` runs every check that the taxonomy in `.headwater/taxonomy.lock` generates over the corpus that `.headwater/taxonomy.yml` declares. It reports the census, the graph, every finding, the coverage account and the obligation register. The report is one artifact in one of four vocabularies. `--format` picks the vocabulary, and `--json` is a second spelling of `--format json`.

**The run is advisory unless `--strict` is passed.** A finding of any severity leaves the exit status at 0, which [spec 6](../spec/06-engine-architecture.md#exit-codes) fixes as the default. A tool that blocks on first contact is a tool somebody removes, and a removed tool catches nothing.

**Two streams carry two different facts, and a caller that merges them reads a correct report as a broken one.** The report goes to standard output. The cache accounting goes to standard error, because it is a fact about the disk of one machine rather than about the corpus. `--fix` puts its account of what it wrote on standard error for the same reason. A cached run and a `--no-cache` run write the same bytes to standard output. A line about the cache on that stream is the one thing that would make the two differ.

**The verb reads the lock and never the package sources.** An edit to `packages/` or to `.headwater/overlay.yml` reaches no check until `headwater taxonomy resolve` writes the lock again. This is the trap that makes a test of a rule pass over a declaration that moved.

**Nothing here reaches a network.** No crate under `engine/crates/` depends on an HTTP client, and the checks read the tree in front of them.

## Preconditions

**`.headwater/taxonomy.lock` is there.** `headwater taxonomy resolve` writes it, and it is written only when the taxonomy validates, so a lock is a validated taxonomy. Without one the verb prints the path it looked at, names the verb that writes it, and exits 1. Everything downstream of the load reads the lock, so this is the precondition every other one sits behind.

**`.headwater/taxonomy.yml` reads, and it names the corpus root and the exclusions.** A corpus root that no directory answers is an empty census rather than a refusal.

**The host states a date, or `--now` states one.** The clock is read once, in this verb, before any check runs. [Spec 12](../spec/12-check-layer.md) makes the date an injected value rather than a call inside a check. One corpus, one lock and one date therefore write one set of bytes. A host with no readable clock and no `--now` is refused rather than guessed at.

**With `--change`, the manifest reads.** A manifest this engine cannot open ends the run, because a line that was dropped reads as a document that did not move. A path inside the manifest that reaches no row of the census is counted and named in the report. So a mistyped path is visible rather than absorbed.

## Options

| Option | What it does |
|---|---|
| `--strict` | Exit non-zero when a finding is an error. Without it the run is advisory. |
| `--fix` | Write the patch that rides with a finding, in this working tree, before the report is composed. A finding carries a patch only where the remedy is mechanical and total, and a suppressed finding carries none. The report is the state after the write, so a patch that produced a document the checks reject is reported on the same run. |
| `--no-cache` | Read and write no cache, and evaluate every instance. Standard output is the same either way, and a difference is a defect in the cache rather than a result. |
| `--now <date>` | The date to evaluate against, as `YYYY-MM-DD`. It defaults to the system clock. |
| `--change <manifest>` | The manifest of the change this run is scoped to. The first line is `headwater change 1`. A file that opens with anything else is refused rather than read. Each line after it is `added<TAB><path>` or `prior<TAB><path><TAB><file>`, and the second form names a file holding the bytes that stood before the change. A document the manifest omits did not move. Without the flag, every rule that reads a transition reports each of its instances as skipped rather than as passed. `.githooks/change-manifest` is the producer this repository uses. |
| `--read-set <path>` | Write the read set of this run to a file as well as into the report. `headwater gate` is the reader. |
| `--register <path>` | Write the obligation and control register of this run to a file as well as into the report. The content is the content already in the report. The report lays its copy out at the width of the run. This file is written at no width, because nothing reads it back. |
| `--format text\|json\|sarif\|markdown` | The vocabulary the report is written in. `text` is the default and the one a person reads. `sarif` is what a forge ingests, `markdown` is a job summary or a review comment, and `json` is the finding shape that [spec 4](../spec/04-assurance-model.md) declares. `sarif` writes its own loss set into the artifact. `markdown` declares one in the source and not in the artifact, because nothing it writes is machine-readable. `text` and `json` declare that they drop nothing. The flag moves no verdict and no exit status. |
| `--json` | The same artifact `--format json` writes, byte for byte, on both streams and with the same exit status. A command line that states both is refused, because two names for one target is a question answered twice. |
| `--root <path>` | The repository to read. It defaults to the working directory. |
| `--no-color` | Write no color. Every run of this binary already writes none, on either stream and in every format, so the flag confirms that state and changes no byte. It is declared so that a caller who writes it out of habit gets an answer rather than a refusal. |
| `--wide` | How wide the help and the report of this verb are laid out. `COLUMNS` states the width, and this binary holds the reading to the range 80 to 120. A reading that is absent or is not a number gives 80, which is the width a run with no flag gives. `headwater check --wide`, `headwater check --wide --format text` and `headwater check --wide --help` are each laid out at that width. The flag is refused beside `--format json`, `--format sarif`, `--format markdown` and `--json`, because nothing lays a machine format out. `--json` is named here as well as `--format json`, because the two spellings reach one target. A refusal that read one of them would accept the other. The read-set block of the report is never laid out, for the reason the Files section gives. |

**Every flag above belongs to this verb, and a flag that belongs to another verb is refused here.** `headwater check --level L0` exits 1 and writes no report, because `--level` is a flag of `headwater conformance`. The table above is the set of flags that reach this verb. Two flags are answered before the verb is reached. `--help` and `--version`, on both spellings, each exit 0, write to standard output alone, and produce no report. `--wide` reaches the verb and states the width of its report, and it exits 1 beside a machine format alone, in either spelling of one. [HW-DR-0033](../decisions/0033-q33-whether-the-command-line-is-derived-and-who-a-flag-belongs-to.md) is the ruling that a flag belongs to the verb that reads it. `engine/crates/cli/tests/wiring.rs` holds this paragraph over a corpus where the verb otherwise succeeds, and `engine/crates/cli/tests/width.rs` holds the `--wide` sentence in it at both ends.

## Exit status

**0** where the run completed and no reason below applied. A finding of any severity, including an error, leaves the status at 0 unless `--strict` was passed.

**1** for each of the eleven reasons below. There is no third status, so a caller reads the message to tell them apart.

| The reason | Where it is decided |
|---|---|
| A flag that names a value has none after it. Or `--now` is not `YYYY-MM-DD`. Or the command line holds a word this verb does not read. Or it names one target twice, as `--json` beside `--format` | the parse, before the verb is entered |
| `--format` names a target that is not one of the four | `check`, before the corpus is walked |
| The host has no readable clock and no `--now` was passed | `check`, before the corpus is walked |
| `--change` names a manifest that did not read | `check`, before the corpus is walked |
| `--fix` composed a patch and the write did not land | `fix`, before the report |
| The lock is absent, or a declaration under it did not read | `load`, called by `check` |
| The report lost a finding that no declared loss reason covers | the adapter census, after the report is written |
| `--read-set` names a file that could not be written | after the report is written |
| `--register` names a file that could not be written | after the report is written |
| `--fix` was passed and a file refused its patch | after the report is written |
| `--strict` was passed and at least one finding is an error | the last reading of the verb |

Two of those are worth separating. **A refused patch is not a finding**, so no absence of `--strict` softens it. The verb was asked to write and did not, and a caller who read a 0 would believe a corpus was fixed. And **the report is written before the last five rows are decided**. A run that exits 1 for one of those five still put a complete report on standard output. That holds in `text`, `json`, `sarif` and `markdown` alike.

**The other six rows are refusals, and a refusal writes nothing to standard output.** The account of a refusal is one English sentence on standard error, under `--json` and `--format json` alike. So the property is that a refusal writes no document, and not that a non-zero exit writes none. [HW-DR-0043](../decisions/0043-q43-whether-a-refusal-under-json-is-a-json-document.md) rules it, and `a_refusal_writes_no_document_and_accounts_for_itself_on_the_other_stream` in `engine/crates/cli/tests/json.rs` holds both halves.

## Environment

**No environment variable reaches this verb.** The date is `--now`, the corpus is `--root`, and the taxonomy is the lock. Six variables are read by test targets alone and reach no shipped code path. `HEADWATER_BLESS` re-records a fixture. `HEADWATER_STOCK_VALIDATOR`, `HEADWATER_SARIF_VALIDATOR`, `HEADWATER_SHA256_ORACLE`, `HEADWATER_JSON_ORACLE` and `HEADWATER_SHELL_ORACLE` each turn a reading taken by a tool outside this repository from a note into a requirement. The last of the five is `bash`, which is the only reader that can say a completion script loads. The continuous-integration job sets all five, so a lost dependency fails the job rather than going quiet.

**One variable reaches this binary, and a run of this verb reads it under `--wide` alone.** `COLUMNS` says how wide the help and the report of this verb are laid out. `engine/crates/cli/src/paint.rs` reads it, and only where the raw command line carries `--wide`. That call is the one `std::env::var` under `engine/crates/` outside a test target. The reading is held to the range 80 to 120, and a reading that is absent or is not a number gives 80. So a run of this verb that carries no `--wide` is a function of the command line, the tree and the lock. Nothing a shell exported reaches it.

A reader who met `HEADWATER_NOW` in a continuous-integration job is reading a shell variable of that job, which the job passes to `--now`.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.lock` | read. The taxonomy every check is generated from, and the digest that keys the cache. |
| `.headwater/taxonomy.yml` | read. The corpus root, the exclusions and the package the repository consumes. |
| the corpus | read. Every file under the declared root that no exclusion removes. |
| `.headwater/cache/checks` | read and written, unless `--no-cache`. A file that is absent, unreadable or written by another engine reads as an empty cache. That costs one full run and is not an error. |
| `.headwater/imports/` | read where `.headwater/taxonomy.yml` declares an import, for the anchors an imported snapshot supplies. |
| the path `--read-set` names | written. The report carries the same bytes, indented two spaces and laid out at no width. `headwater gate` reads that file as a grammar, so a line break inside it would refuse the file rather than widen it. |
| the path `--register` names | written. The report carries the same content, laid out at the width of the run. Nothing reads this file back, so the layout costs no consumer anything. |
| the path `--change` names | read. |
| a document of the corpus | written, and only under `--fix`. |

Without `--fix`, this verb writes no byte of the corpus. The cache is outside the corpus root, so a run that wrote one changes nothing a check reads.

## See also

[`headwater taxonomy resolve`](../spec/06-engine-architecture.md#the-verbs) writes the lock this verb reads, and a check that reports a rule nobody declared is a lock that was not written again.

`headwater gate` reads the file that `--read-set` writes, and answers whether the verdicts of this run carry to another tree.

[`headwater sweep`](headwater-sweep.md) reports what no check can see. Its findings reach no rule and no exit status of this verb.

[The command surface](README.md) lists every verb this binary dispatches, and it marks the ones that no contract describes. `headwater generate` writes it.

[Spec 12](../spec/12-check-layer.md) declares the check layer: the scopes, the cache, the read set, and the [fixability](../spec/12-check-layer.md#fixability) bar that decides which findings `--fix` acts on. [Spec 4](../spec/04-assurance-model.md) declares the finding shape and the register. [Spec 6](../spec/06-engine-architecture.md#exit-codes) declares the exit-code convention this verb follows.

The commit gate of this repository is `.githooks/pre-commit`, and it runs `headwater check --strict` after `.githooks/change-manifest` writes the manifest that `--change` reads.
