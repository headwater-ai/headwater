---
id: HW-SPEC-the-recorder-contract
status: current
status_since: 2026-09-06
summary: Everything an external recorder writes into a transcript, the closed key sets that refuse a file, the one member of the run identity this engine compares, and the period a result stays citable.
last_verified: 2026-09-07
doc_type: design_spec
sequence: 15
title: "The recorder contract"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-ai-integration
---

# The recorder contract

[Spec 5](05-ai-integration.md#a-transcript-is-recorded-from-outside-the-session-and-never-written-back-by-the-agent) names a component that no part of this repository holds. A **recorder** drives a probe session, observes it from outside, and writes a transcript of what it saw. It is the one part of the measurement layer that reaches the network. So it is not a crate of this engine, and no verb here writes a transcript.

A person who writes a recorder reads this part. Everything below is a contract that `engine/crates/probe/src/intake.rs` enforces, and `engine/crates/probe/tests/contract.rs` holds this text to the constants that carry the four closed key sets. A key added to the engine and not to the table below fails that test.

## A recorder is a driver and a transform, and the channel between them is the condition

[HW-DR-0059](../decisions/0059-a-transform-over-a-harness-session-log-is-an-observed-transcript-when-the-log-arrives-by-a-channel-the-model-cannot-write-to.md) admits a second shape of recorder. A **driver** starts the session and reads what the harness emits. A **transform** filters that stream into a transcript. The driver reaches the network and the transform reaches nothing, so the sentence above describes the driver.

A harness already tags each block of its own session log by type, and the model states no tag on any block. So a filter that keeps the tool calls and their results reads no account the model wrote of its own process. That filter is an outside observation on one condition. **The log has to reach the transform through a channel that the model has no handle on.**

A file on a filesystem the session reaches is not such a channel. On the host that runs this repository, every process of one user can write the live log of a session. A `Bash` call of that session is one of those processes. The standard output of the harness process is such a channel, because the session holds no handle on the parent's pipe. `claude -p --output-format stream-json`, read by the driver from that stream, is the channel the first recorder reads. A recorder that reads the log from disk states nothing about who wrote a line of it. The transcript it produces is a self-report that no key set can detect.

Both parts sit outside this engine. The transform is the second half of one component, and it stands on the path from a live session to a transcript. That is the position the paragraph above keeps out of the engine.

### The values a session log omits, and the step that derives each one

A harness log carries the calls and not the rest of the contract below. Each value the log omits is derived after the session by a named step. The list is prose rather than a table, because the four tables of this part are the four closed key sets and `engine/crates/probe/tests/contract.rs` counts them.

- `result` on a call is the content digest of the document, in the form `headwater probe plan` prints, and never the bytes returned.
- `cites` on a produced artifact is every identifier of this corpus that appears in the artifact.
- `findings` on a produced artifact is every rule that reported over the artifact. Both are computed the same way for every probe, and neither reads a probe.
- The six identity members are `lock`, `tree`, `selection`, `read_set`, `seed` and `harness`, copied from the plan.
- `served_version` and `cost_cents` are derived from the provider metadata of the run, because a usage record carries token counts and not cents.

## The engine fixes six members of the identity, and the recorder supplies the rest

`headwater probe plan` composes a selection and prints the six members of the run identity that exist before any session starts. They are the lock, the corpus tree, the selection, the read set, the seed and the harness version. The plan also prints the probes selected, the task of each one, and the documents each one examines. It prints the read set that the digest covers, one line for each document.

The recorder copies those six into the transcript without change. It supplies the four that belong to the run: the model, the served version, the wall-clock time and the realized cost. It supplies the tier and the arm, which the plan states and which a reader of the transcript alone would otherwise have to guess.

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
2. **The identity is complete.** Every one of the twelve keys is present.
3. **Membership.** Every probe an event names is a classified probe of this corpus.
4. **No prose.** Every key of every block is a member of one of the four sets above.
5. **A realized cost.** `cost_cents` is a whole number of cents.

Present is not confirmed, and the difference is what the rest of this section states. Of the six members the plan fixed, this verb compares one. `headwater generate` compares a second and `headwater probe stale` compares a third. The other three are not compared at all, and each of the three has a reason of its own.

**The `selection` digest is compared, and the comparison is reported rather than refused.** `headwater generate` writes the comparison into the probe result. The digest covers the identifiers of the probes selected and nothing else. So it holds still when the prose of a probe is edited. It moves when a probe is added, removed or renamed. That is the one change that makes a recorded run cover a population this corpus no longer declares. A refusal there replaces a graded rate with a notice on the day somebody adds a probe. So the result names both digests, and the reader decides.

**The `read_set` digest is compared by a verb, and no exit status carries the answer.** The digest covers every probe of the selection and every document one of them examines, by path and content. So it moves when a document a session was pointed at changes. It holds still when any other document of this corpus changes. `headwater probe stale` recomposes it over the tree in front of the reader and reports which recorded results the change voided. That verb exits 0 on every answer it reaches. A probe result that goes stale is a fact about a measurement. An exit status that carried it would put the behavior of a model on a build.

**A comparison against the tree in front of a reader does not go into a probe result.** `generate --check` holds every committed projection to its own bytes. A result that compared a recorded digest against the current tree would change its own bytes. It would change them on every edit to a document the selection points at. The gate would then ask for a regeneration, and the staleness of a measurement would stop a merge through a derived document. A regeneration would also write something false, because it would claim the run was taken over a state that the run never met. So the result names the recorded digest, and `headwater probe stale` compares it. The `selection` digest is the one comparison a result carries. It earns the place because it moves only when somebody adds, removes or renames a probe.

**The `tree` digest is recorded and never compared.** It covers every classified document of the corpus, so it moves on any edit to any document. A result that reported it would need a fresh commit after every prose change, and `generate --check` would ask for one on every pull request. A statement that nobody can leave standing is not a statement. The read set above is the narrow instrument that this reasoning defers to. It covers the documents that a whole-tree digest cannot separate from the rest of the corpus.

**A refusal is reported by the run and not only by the file it writes.** The refusal text is what `headwater generate` derives for a refused transcript, so `generate --check` regenerates it faithfully and a corpus can carry a result with no verdict through every gate it has. One did. So the run prints a `refused transcripts` section that names the transcript, the confirmation that refused it and the reason.

**The run fails where the corpus says that a reader may rely on the refused transcript, and nowhere else.** The state facet declares a role on every value it admits. A `live` role says that a reader may rely on the document. That is the one state which holds the refusal. An `initial` role says that the recording is unfinished. A `terminal-` role says that the corpus keeps the recording as a record and relies on it no more. Both of those states report the refusal and leave the run green. The remedy for a refusal is a fresh recording rather than an edit. A gate that a contributor cannot clear is a gate that gets removed. Every other reading holds the refusal. Three of them land there: a document with no state, a value the vocabulary does not admit, and a role the engine cannot fold. Each one says that nothing has stated where the recording stands. [HW-DR-0062](../decisions/0062-a-refused-recording-is-held-by-the-reliance-its-state-claims-and-not-by-promotion.md) is the ruling, and it carries the cost this reading measures.

**The `seed` and the `harness` are provenance.** A seed is a number the caller stated, and this corpus holds nothing to compare it against. A harness version is the version of the engine that planned the run. A comparison against the version that reads the run refuses every transcript on the first release.

## Nothing here separates a recorded transcript from a typed one

Every value in a transcript is a value that a person can type. The lock digest is printed by `headwater probe plan`. So is the selection digest, and so is the read-set digest. The events are lines of YAML. No signature, no key and no witness is part of this contract. An integrity artifact that travels with the file it describes states internal consistency rather than identity.

The distinction between a recorded artifact and a written one lives in `provenance.warrant`. No taxonomy declares that block and no check reads it. A shelf index prints the warrant of every document it covers, and the shelf that holds a transcript has no index. [HW-OBL-0030](../obligations/0030-the-provenance-block-belongs-to-the-engine-and-nothing-states.md) holds that gap. This contract is the sharpest instance of it. The warrant is the only field that separates the two artifacts, and it is the field with no shape.

What the design does buy is narrower and it is worth stating exactly. **A rate is not a value that anybody types.** `headwater generate` derives it from the events, and `generate --check` holds the committed result to the derivation. So a forged rate needs a forged event log. That is a claim about which documents a session opened, rather than a number in a summary. A reader who doubts a result reads the transcript and counts events. Every satisfied verdict names the event it came from.

## How long a result stays citable, and why that period is not a number this schema produces

A published rate is a claim that a reader can re-derive. `headwater generate` writes a probe result from three committed inputs and from nothing else. They are the transcript, the expectations the probes of this corpus declare, and the version of the grader. The result states that in its own first paragraph. So the retention window of a transcript is the period in which a reader can still fetch those three. [HW-DR-0008](../decisions/0008-probe-cost-and-cadence.md) commits the transcript and states nothing about how long it stays.

**Storage does not bound the window, and the reason is a choice this contract already made.** A tool call records the identity of what it returned and never the bytes. A produced artifact does the same. The bytes a tool returned are the corpus, which the transcript already names by tree digest. So a transcript of a session that read forty documents is forty digests rather than forty documents.

**The schema fixes the keys and not their number, so no byte figure follows from it.** The four tables above close what a key may be. They do not bound `calls`, which is an ordered list whose length is the behavior of the model. They do not bound `cites` and `findings`, which the artifact bounds rather than the session. A figure multiplied out of the key counts, the probe count and the session count sizes the identity block alone. It undercounts every transcript that carries a produced artifact. The session counts are derived and the byte size is not. `headwater probe plan` reads the probes from the tree, and `.headwater/probe.yml` declares the arms and the repetitions of each tier. Every plan prints the product. The first committed transcript is what turns a byte size from a guess into a measurement, and the one this corpus holds is the measurement to read it off.

**Two of the three inputs never expire, and the third is the whole window.** The transcript and the probes are documents of this corpus. The history of the repository holds them for as long as the repository stands, and no verb of this engine deletes one. The grader version is a string rather than an artifact. To re-derive a result at it a reader needs the source of this engine at that version and a toolchain that builds it. So the window is the period in which somebody can build the named grader, and nothing in this part sets it.

**A release stales every committed result until somebody regenerates it.** The grader version is the engine version. A release that bumps it leaves each committed result a function of a version that is not the current one. The result names the version that graded it. A report over a set of results says how many versions the set spans rather than one average across them. Two results graded by two versions are two instruments.

**`headwater probe stale` names no window, and it answers a different question.** It recomposes the read set over the tree in front of the reader and reports which recorded results a change to the corpus voided. Age expires a result and an edit voids one. They are two facts, and neither one substitutes for the other. A window is a policy of the repository that runs the probes, and `.headwater/probe.yml` is where such a policy already lives.

## This corpus declares its probes on one shelf and holds one refused transcript

`docs/probes/` is the shelf that declares them, and `headwater probe plan` prints the count it reads from the tree. No count is written here, because a hand-kept count of a shelf drifts as the shelf grows. `docs/probe-runs/` holds one transcript, recorded on 2026-09-09, and `headwater generate` writes the result beside it. The first confirmation refuses that transcript: the commit that landed it moved the lock in the same commit, so the `lock` it pins is not the lock of this tree and the result carries no verdict.

The reason the refusal stands is this part, read from the other end. A recorder observes a session from outside it, and `tools/probe/probe-record.sh` is the process in this repository that does. An agent that works here and writes a file about the documents it opened produces the self-report that spec 5 refuses, and no check here tells that file from a recorded one, so the channel the log arrived by is what a reader trusts. What the corpus is owed is no longer a recorder. It is a recording taken against the lock of the tree that reads it. [HW-OBL-0124](../obligations/0124-a-probe-result-is-printed-and-never-committed-so-nothing-regenerates-one.md) carries that debt.
