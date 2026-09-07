---
id: HW-IFACE-headwater-sweep
status: current
status_since: 2026-09-06
summary: "Why no result of headwater sweep can move an exit status, and the four caller errors that move one anyway."
last_verified: 2026-08-24
title: "headwater sweep"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - engine/crates/cli/src/main.rs
    - engine/crates/sweep/src/lib.rs
---

# headwater sweep

## Synopsis

    headwater sweep plan   [--under <path>] [--root <path>]
    headwater sweep report <path> [--format text|json | --json] [--root <path>]

`plan` takes no operand. `report` takes exactly one, which is the path of the file an agent wrote back. Four inputs are refused. A bare `sweep`. A `sweep report` with no path. A second word that is neither `plan` nor `report`. A `report` with more than one path, which the message calls a verb the binary does not carry.

## Description

The coherence sweep is a sampler and never a check. It reports the five classes below, and no rule of the check layer reads any of them.

- `undeclared_conflict`: two documents contradict each other, both are current, and neither says so
- `quiet_supersession`: a newer document has quietly overtaken an older claim
- `undefined_concept`: a term is used across the slice and defined in none of it
- `audience_mismatch`: the audience the document declares could not act on what it says
- `unwritten_section`: a heading the kind requires, over prose that says nothing about it

**The verb is two halves with a model between them, and no code here performs the middle part.** `plan` writes the briefing an agent reads. A person or an agent then reads the documents and writes one file. `report` reads that file and states what this engine could confirm about it.

**`plan` is deterministic and states its own extent.** Two runs over one tree write the same bytes. There is no sampling rule, because a slice this engine picked would be an unreproducible sample dressed as a reproducible one. The briefing names six things.

- the slice, as the prefix a caller passed
- the count of documents in the slice, against the count in the corpus
- the digest of the lock the plan was taken under
- what the graph already declares between two members of the slice
- the shape of the file to write back, as a YAML skeleton
- the five classes, each with the sentence that defines it

**`report` confirms and never believes.** Every quotation is located in the document it names, and a finding whose quotation is not there is refused. A paraphrase is not a quotation. Five more findings are refused. One names a path that is not a classified document. One names a class outside the five. One proposes an edge the graph already carries, and a relation that is its own inverse carries that edge both ways round. One proposes an edge between two kinds that the relation does not admit at the ends the finding names. One proposes an edge from a document to itself. A file whose `taxonomy:` digest is not the digest of the lock is refused whole. A sweep planned against one taxonomy and read back against another compares two corpora.

**Nothing gates on any of it.** [Spec 12](../spec/12-check-layer.md#four-things-stop-a-sweep-from-gating-and-none-of-them-is-a-rule-that-somebody-keeps) names four mechanisms, and only one of them is an exit status. The sampler is its own crate, and that crate names `headwater-check` for the finding shape. So `headwater-check` can never name the sampler, and the compiler refuses the cycle. No commit gate and no continuous-integration job runs either half. No crate of this engine depends on a network client, so no build waits for a model.

**Neither half writes a byte.** `report` prints the front matter that would declare a proposed edge and writes none of it. It names the document at the `from` end of that edge, because that document is the one that carries the front matter. It never names another document the finding compares. There is no `--write`, and this is the one verb of the write path that has none. A proposal an agent applies to itself is the act that [HW-OBL-0108](../obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md) records at the scale of a whole corpus.

**Both halves render the palette [HW-DR-0045](../decisions/0045-coloring-the-cli-and-where-the-banner-goes.md) rules on, when standard output is a terminal.** `plan` bolds its `##` headings and colors the paths it names. `report` colors a finding's severity, path, obligation and `fix:` label the way `headwater check` does, and dims the repeated labels beside a citation. `report --format json` never colors: a machine reads it. `--no-color` forces the plain text both halves already wrote before this decision. `--no-banner` is accepted and inert on both, the posture every verb takes.

## Preconditions

**`.headwater/taxonomy.lock` is there, for both halves.** `headwater taxonomy resolve` writes it. Both halves load the corpus through the lock, so a repository that never resolved gets 1 from either one.

**`.headwater/taxonomy.yml` reads, and it names the corpus root and the exclusions.**

**For `report`, the file an agent wrote is readable.** It is YAML, and a file that is not parses into a refusal rather than into an error.

**For `report`, the `taxonomy:` line of that file holds the digest the lock holds.** A file written against another taxonomy is refused whole, and the run still exits 0.

There is no precondition about a model, about a network, or about a plan having been run. `report` reads the file it is handed and asks the tree in front of it about the claims in that file.

## Options

| Option | Half | What it does |
|---|---|---|
| `--under <path>` | `plan` | The slice, as a path prefix under the repository root. The whole corpus by default. |
| `--format text\|json` | `report` | The vocabulary. `text` is the default and the one a person reads. `json` is the finding shape that [spec 4](../spec/04-assurance-model.md) declares, with the provenance and the evidence a sweep adds. |
| `--json` | `report` | The same artifact `--format json` writes, byte for byte, on both streams and with the same exit status. A command line that states both is refused, because two names for one target is a question answered twice. |
| `--root <path>` | both | The repository to read. It defaults to the working directory. |
| `--no-color` | both | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | both | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

**There is no `--strict`, and the parser refuses one.** `headwater sweep report <path> --strict` exits 1 and writes no report. Spec 12 states the absence of a `--strict` that does anything. The parse states the absence of the word, so both readings of that sentence hold of this verb. [HW-DR-0033](../decisions/0033-q33-whether-the-command-line-is-derived-and-who-a-flag-belongs-to.md) is the ruling, and `engine/crates/cli/tests/sweep.rs` holds the refusal.

## Exit status

**0 for every fact this verb states about a corpus, a file or a model.** That is the constraint rather than a leniency. `report` exits 0 with findings, exits 0 with every finding refused, and exits 0 when it refuses the whole file. `plan` exits 0 over a slice that holds no document. An exit status that carried any of those would put the output of a model on a build.

**1 for four reasons, and every one of them belongs to the caller.**

| The reason | Half | When it is decided |
|---|---|---|
| A flag that names a value has none after it. Or the command line holds a word this half does not read. Or the words after `sweep` are not `plan`, `report <path>`. Or `report` names one target twice, as `--json` beside `--format` | both | in the parse, before the verb is entered |
| `--format` names a target that is neither `text` nor `json` | `report` | first thing in the verb, before the file is opened |
| The file at the given path did not read | `report` | after the format is decided, before the corpus is loaded |
| The lock is absent, or a declaration under it did not read | both | for `report`, after the return file is read and before it is parsed. For `plan`, before anything |

**The order of the three reasons inside `report` matters to a caller who reads a message.** `--format` is decided first, the path second, and the lock last. A `report` over an unreadable path in a repository that never resolved therefore names the path. [Spec 12](../spec/12-check-layer.md#four-things-stop-a-sweep-from-gating-and-none-of-them-is-a-rule-that-somebody-keeps) carries the same three, and [spec 6](../spec/06-engine-architecture.md) carries the same statement about `plan`.

**The lock is the reason a reader is most likely to be surprised by.** The other two are errors a caller made in the command line. This one is a fact about the repository, and it reaches a caller who typed a command with nothing wrong in it.

**Standard output is empty on all four, because all four are refusals.** Each one is decided before anything is written, so the account is one English sentence on standard error. That holds for `--json` and for `--format json` alike, which is what [HW-DR-0043](../decisions/0043-q43-whether-a-refusal-under-json-is-a-json-document.md) rules. This verb has no reason for exit 1 that is decided after a report, which is the case `headwater check` carries and this one does not.

**Where each status is asserted.** `engine/crates/cli/tests/sweep.rs` starts the binary and holds all four rows above, including the third non-zero reason and the `--strict` that changes nothing. `engine/crates/sweep/tests/fixtures.rs` covers the intake as a library, which is where a refusal is decided, and it starts no process. So the refusal and the status it does not move are asserted in two places, one either side of the process boundary.

## Environment

**No environment variable reaches either half.** The slice is `--under`, the repository is `--root`, and the taxonomy is the lock.

**One variable reaches this binary, and neither half reads it.** `COLUMNS` says how wide the help and the report of `headwater check` are laid out. `engine/crates/cli/src/paint.rs` reads it, and only where the raw command line carries `--wide`, which both halves refuse because neither half lays anything out. That call is the one `std::env::var` under `engine/crates/` outside a test target.

There is no variable that names a model, a key or an endpoint, and there is nowhere for one to be read. No crate of this engine depends on a network client.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.lock` | read, by both halves. |
| `.headwater/taxonomy.yml` | read, by both halves. |
| the corpus | read, by both halves. |
| the path `report` is handed | read. |

**Neither half writes a file.** The briefing and the report both go to standard output, and standard error carries nothing on a run that reaches a report. There is no cache, so two runs of `plan` over one tree do the same work twice and write the same bytes.

## See also

[`headwater check`](headwater-check.md) is the deterministic half of the same reporting pipeline. It shares the finding shape and shares nothing else. No finding of this verb reaches a rule, a cache, a read set or an exit status of that one.

`.claude/skills/headwater-sweep/SKILL.md` is the instruction a model reads between the two halves in this repository.

[The command surface](README.md) lists every verb this binary dispatches, and it marks the ones that no contract describes. `headwater generate` writes it.

[Spec 12](../spec/12-check-layer.md#where-the-llm-coherence-sweep-fits) states where the sweep sits and what stops it from gating. [Spec 4](../spec/04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep) states the obligations a sweep discharges and the finding shape it borrows. [Spec 6](../spec/06-engine-architecture.md) states the grammar of both halves.
