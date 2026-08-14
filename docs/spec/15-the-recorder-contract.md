---
id: SPEC-HW-the-recorder-contract
status: draft
status_since: 2026-08-14
summary: Everything an external recorder writes into a transcript, the closed key sets that refuse a file, and the one member of the run identity this engine compares.
last_verified: 2026-08-14
doc_type: design_spec
sequence: 15
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft
  evidence_basis: evidenced
relations:
  traces_to:
    - SPEC-HW-ai-integration
---

# The recorder contract

[Spec 5](05-ai-integration.md#a-transcript-is-recorded-from-outside-the-session-and-never-written-back-by-the-agent) names a component that no part of this repository holds. A **recorder** drives a probe session, observes it from outside, and writes a transcript of what it saw. It is the one part of the measurement layer that reaches the network. So it is not a crate of this engine, and no verb here writes a transcript.

A person who writes a recorder reads this part. Everything below is a contract that `engine/crates/probe/src/intake.rs` enforces, and `engine/crates/probe/tests/contract.rs` holds this text to the constants that carry the four closed key sets. A key added to the engine and not to the table below fails that test.

## The engine fixes six members of the identity, and the recorder supplies the rest

`headwater probe plan` composes a selection and prints the six members of the run identity that exist before any session starts. They are the lock, the corpus tree, the selection, the read set, the seed and the harness version. The plan also prints the probes selected, the task of each one, and the documents each one examines. It prints the read set that the digest covers, one line for each document.

The recorder copies those five into the transcript without change. It supplies the four that belong to the run: the model, the served version, the wall-clock time and the realized cost. It supplies the tier and the arm, which the plan states and which a reader of the transcript alone would otherwise have to guess.

A model name is not a pin. The recorder writes the served version where the provider exposes one, and it writes the name as a name where no version is exposed.

## A transcript is two fenced blocks under two headings

The `probe_transcript` kind requires a `Run identity` section and an `Events` section. Under each heading the recorder writes one fenced `yaml` block. Prose around the blocks is what a person wrote about the run, and no verb of this engine reads a word of it.

The blocks carry four closed key sets. A key outside a set refuses the whole file. That refusal enforces the rule that a transcript holds no model prose. An omission that nothing tests is a request rather than a rule. A transcript with a `reasoning` key beside the tool calls returns the self-report through the field the rule forbids.

### The run identity, which carries twelve keys

| key | what the recorder writes |
|---|---|
| `model` | the model the session ran against |
| `served_version` | the served version, or the name where none is exposed |
| `tree` | the corpus tree digest the plan printed |
| `lock` | the taxonomy lock digest the plan printed |
| `selection` | the selection digest the plan printed |
| `read_set` | the read-set digest the plan printed |
| `seed` | the rotation seed the caller stated |
| `harness` | the harness version the plan printed |
| `tier` | `regression` or `campaign` |
| `arm` | `present` or `absent` |
| `at` | the wall-clock time of the run |
| `cost_cents` | the realized cost, as a whole number of cents |

Every one of the twelve is required. An identity with a member missing is a measurement that nobody can locate again. A run that recorded no cost leaves the cost of the instrument to a guess.

### One event, which carries five keys

| key | what the recorder writes |
|---|---|
| `probe` | the identifier of the probe this session ran |
| `session` | a name for this session, distinct within the probe |
| `calls` | the ordered tool calls, or no key at all |
| `produced` | the artifacts the session produced, or no key at all |
| `answer` | the final answer, `null`, or no key at all |

`probe` and `session` are required of every event. An event that names a probe this corpus does not classify is dropped with a reason, and the reason is printed beside the rate. The last three keys carry the three-state rule below.

### One tool call, which carries three keys

| key | what the recorder writes |
|---|---|
| `tool` | the name of the tool the session called |
| `argument` | the argument, which for a read is the path |
| `result` | the identity of what the call returned, which for a read is the content digest of the document |

A result identity and never a result. The bytes a tool returned are the corpus. A transcript that held them is a second copy of the tree it already names by digest.

The identity of a read is the content digest of the document, in the form that `headwater probe plan` prints. `headwater probe stale` compares that identity against the digest this corpus holds, so an identity of another form decides nothing.

### One produced artifact, which carries four keys

| key | what the recorder writes |
|---|---|
| `path` | where the artifact landed |
| `result` | the identity of the artifact, and never the bytes |
| `cites` | every identifier of this corpus that appears in the artifact |
| `findings` | every rule that reported over the artifact, or no key at all |

## An empty list and an absent key are two facts, and a grader is wrong without both

Four keys carry this distinction: `calls`, `produced` and `answer` on an event, and `findings` on a produced artifact.

`calls: []` says the recorder watched the session and saw no tool call. An event with no `calls` key says that nothing watched. `findings: []` says the artifact was checked and no rule reported. An artifact with no `findings` key says that nothing checked it.

A recorder that wrote an empty list for a thing it did not observe reports a verdict about itself as a verdict about the corpus. The direction of the error is the reason the distinction is here. Every one of those readings returns a pass, so a recorder that collapses the two turns a run that observed nothing into a clean result.

## Two produced keys carry a derivation, and the boundary keeps the recorder out of the grader

`cites` and `findings` are computed rather than observed, and the two forms this contract permits are narrow.

A recorder computes each key **the same way for every probe, and it reads no probe to do it**. `cites` is every identifier of this corpus that appears in the artifact. `findings` is every rule that reported over the artifact. Neither reads an expectation.

A recorder that reads the probe is a grader with no fixture set and no version. That covers a recorder which writes only the identifiers one probe named, and one which writes only the rule an oracle names. Any value whose computation needs the probe belongs to `headwater probe grade`, which is a pure function of the transcript, the expectations and its own version.

## What the engine confirms, and what it records without confirming

`headwater probe record` confirms five things and evaluates no expectation.

1. **The taxonomy.** The `lock` the transcript names is compared against the lock of this tree, and a transcript planned against another one is refused whole.
2. **The identity is complete.** Every one of the eleven keys is present.
3. **Membership.** Every probe an event names is a classified probe of this corpus.
4. **No prose.** Every key of every block is a member of one of the four sets above.
5. **A realized cost.** `cost_cents` is a whole number of cents.

Present is not confirmed, and the difference is what the rest of this section states. Of the six members the plan fixed, this verb compares one. `headwater generate` compares a second and `headwater probe stale` compares a third. The other three are not compared at all, and each of the three has a reason of its own.

**The `selection` digest is compared, and the comparison is reported rather than refused.** `headwater generate` writes the comparison into the probe result. The digest covers the identifiers of the probes selected and nothing else. So it holds still when the prose of a probe is edited. It moves when a probe is added, removed or renamed. That is the one change that makes a recorded run cover a population this corpus no longer declares. A refusal there replaces a graded rate with a notice on the day somebody adds a probe. So the result names both digests, and the reader decides.

**The `read_set` digest is compared by a verb, and no exit status carries the answer.** The digest covers every probe of the selection and every document one of them examines, by path and content. So it moves when a document a session was pointed at changes. It holds still when any other document of this corpus changes. `headwater probe stale` recomposes it over the tree in front of the reader and reports which recorded results the change voided. That verb exits 0 on every answer it reaches. A probe result that goes stale is a fact about a measurement. An exit status that carried it would put the behavior of a model on a build.

**A comparison against the tree in front of a reader does not go into a probe result.** `generate --check` holds every committed projection to its own bytes. A result that compared a recorded digest against the current tree would change its own bytes. It would change them on every edit to a document the selection points at. The gate would then ask for a regeneration, and the staleness of a measurement would stop a merge through a derived document. A regeneration would also write something false, because it would claim the run was taken over a state that the run never met. So the result names the recorded digest, and `headwater probe stale` compares it. The `selection` digest is the one comparison a result carries. It earns the place because it moves only when somebody adds, removes or renames a probe.

**The `tree` digest is recorded and never compared.** It covers every classified document of the corpus, so it moves on any edit to any document. A result that reported it would need a fresh commit after every prose change, and `generate --check` would ask for one on every pull request. A statement that nobody can leave standing is not a statement. The read set above is the narrow instrument that this reasoning defers to. It covers the documents that a whole-tree digest cannot separate from the rest of the corpus.

**The `seed` and the `harness` are provenance.** A seed is a number the caller stated, and this corpus holds nothing to compare it against. A harness version is the version of the engine that planned the run. A comparison against the version that reads the run refuses every transcript on the first release.

## Nothing here separates a recorded transcript from a typed one

Every value in a transcript is a value that a person can type. The lock digest is printed by `headwater probe plan`. So is the selection digest, and so is the read-set digest. The events are lines of YAML. No signature, no key and no witness is part of this contract. An integrity artifact that travels with the file it describes states internal consistency rather than identity.

The distinction between a recorded artifact and a written one lives in `provenance.warrant`. No taxonomy declares that block and no check reads it. A shelf index prints the warrant of every document it covers, and the shelf that holds a transcript has no index. [OBL-repo-0030](../obligations/0030-the-provenance-block-belongs-to-the-engine-and-nothing-states.md) holds that gap. This contract is the sharpest instance of it. The warrant is the only field that separates the two artifacts, and it is the field with no shape.

What the design does buy is narrower and it is worth stating exactly. **A rate is not a value that anybody types.** `headwater generate` derives it from the events, and `generate --check` holds the committed result to the derivation. So a forged rate needs a forged event log. That is a claim about which documents a session opened, rather than a number in a summary. A reader who doubts a result reads the transcript and counts events. Every satisfied verdict names the event it came from.

## This corpus declares three probes and holds no transcript

`docs/probes/` holds three probes. `docs/probe-runs/` is a declared shelf that holds nothing, so `headwater generate` writes no probe result and prints the reason on every run.

The reason is this part, read from the other end. A recorder observes a session from outside it, and no process in this repository does that. An agent that works here and writes a file about the documents it opened produces the self-report that spec 5 refuses. No check here tells that file from a recorded one. So the transcript is owed by a component that this repository does not hold. [OBL-repo-0124](../obligations/0124-a-probe-result-is-printed-and-never-committed-so-nothing-regenerates-one.md) carries the debt.
