---
id: HW-DR-0059
status: current
status_since: 2026-09-07
summary: "A filter that keeps the tool calls of a harness session log and drops every block the model wrote is an outside observation, because the type tag it reads is the harness's and not the model's. The condition is the channel the log arrives by, and a file the session can open is not one."
last_verified: 2026-09-07
title: "A transform over a harness session log is an observed transcript when the log arrives by a channel the model cannot write to"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# A transform over a harness session log is an observed transcript when the log arrives by a channel the model cannot write to

## Context

[Spec 5](../spec/05-ai-integration.md#a-transcript-is-recorded-from-outside-the-session-and-never-written-back-by-the-agent) refuses a self-report and names the recorder as a process that drives a session and writes what it observes, event by event. [Spec 15](../spec/15-the-recorder-contract.md) opens by keeping that component out of this engine, because it is the one part of the measurement layer that reaches the network. Neither part says whether a transform over the harness's own log qualifies. A harness writes such a log without being asked, so the question decides whether the first transcript costs a new process or a filter.

The [fourth comment on #170](https://github.com/headwater-ai/headwater/issues/170#issuecomment-5294044542), of 2026-08-14, answers it from an inspection rather than from an argument. A Claude Code session log is JSONL, one line for each turn. `message.content` is an array of blocks, and the harness tags each block `thinking`, `text`, `tool_use` or `tool_result`. The tag is the harness's own, and the model states no tag on any block it produces. A filter that keeps `tool_use` and `tool_result` and drops the rest therefore reads no account that the model wrote of its own process. `tool_use.name` and `tool_use.input` carry the `tool` and the `argument` that [the event contract](../spec/15-the-recorder-contract.md#one-tool-call-which-carries-three-keys) requires.

Four groups of values are absent from such a log. `result` is an identity rather than the bytes a call returned. `cites` and `findings` are computed over each produced artifact after the session. The six identity members that `headwater probe plan` prints are `lock`, `tree`, `selection`, `read_set`, `seed` and `harness`. `served_version` and `cost_cents` come from provider metadata, because `message.usage` carries token counts and not cents.

**The condition on the model carries the whole ruling, and it fails for the log on disk.** On the host that runs this repository the live log of a session sits at `~/.claude/projects/<slug>/<session>.jsonl`. A measurement of 2026-09-07 read the file of a running session. The file is mode 600 and the directory is mode 775, and one user owns both. The write test from inside a `Bash` call of that session returned true, for the file and for the directory. Everything that can write there runs under that one user identity, and the set has six members.

- the harness process, which appends every turn
- every `Bash` call the session makes
- every hook the harness runs at a tool or a prompt position
- every local MCP server the harness started
- every subagent session of the same project
- every other process of that user

The model is inside that set by construction, because a tool call executes as that user. So a transform that opens the file after the session states nothing about who wrote a line of it.

## Decision

Owner ruling 6 of 2026-09-04 holds. The ruling as issued reads:

> A deterministic transform over a harness's own session log qualifies as a transcript observed from outside the session, provided the model cannot write to the log. Record it in spec 5 and build the recorder as that transform.

Restated against the measurement above, and this is the form that holds. A deterministic transform over a harness session log is a transcript observed from outside the session. One condition carries that. The log reaches the transform through a channel that the model has no handle on. The proviso as issued names the model and the log. The measurement puts it on the channel instead, because no harness log on a filesystem the session reaches satisfies it.

The reason is what spec 5's rule protects. The rule protects a reader from a transcript in which some value is the model's account of its own process. A filter by a type tag that the harness assigns reads no such account, whatever the model wrote in the blocks it drops. Judgment is what would return the account by another route, and a filter on a closed set of tags exercises none.

**A file on a filesystem the session reaches is not such a channel**, and the measurement above is why. The standard output of the harness process is one, because the session holds no handle on the parent's pipe. `claude -p --output-format stream-json`, read by the driver from that stream, is the channel the first recorder reads.

So a recorder is two parts rather than one. A **driver** starts the session, reads the stream and reaches the network. A **transform** filters the stream into a transcript and reaches nothing. Both parts sit outside this engine. The transform is the second half of one component, and it stands on the path from a live session to a transcript. That is the position spec 15 keeps out of the engine.

The values the log omits are derived after the session by named steps. `result` for a read is the content digest of the document, in the form `headwater probe plan` prints. `cites` is every identifier of this corpus that appears in the artifact. `findings` is every rule that reported over it. Both are computed the same way for every probe. The six identity members are copied from the plan. `served_version` and `cost_cents` are derived from the provider metadata of the run.

## Consequences

**A reader outside this project gains the one fact that decides how much a published rate is worth.** Anybody who weighs an efficacy rate from this project asks how the transcript under it was obtained. The answer is a channel rather than a promise. A channel is a thing a reader checks against the command the driver ran.

**[Spec 15](../spec/15-the-recorder-contract.md) names the transform, the channel and the derived steps.** It also carries a count that this record corrects. [Its identity section](../spec/15-the-recorder-contract.md#the-engine-fixes-six-members-of-the-identity-and-the-recorder-supplies-the-rest) reads six members out of the plan, and the sentence under the table says five. The sixth is `read_set`.

**[Spec 5](../spec/05-ai-integration.md#a-transcript-is-recorded-from-outside-the-session-and-never-written-back-by-the-agent) states the recorder as one process, and that sentence describes the driver alone.** Spec 5 and spec 15 are one kind on one shelf, so nothing structural separates them. The split above reaches spec 5 as a matter of sequencing alone. [#530](https://github.com/headwater-ai/headwater/issues/530) carries that edit and stays open for it.

**The debt this record does not discharge stays where it is.** [HW-OBL-0124](../obligations/0124-a-probe-result-is-printed-and-never-committed-so-nothing-regenerates-one.md) carries the missing transcript, and what remains under it is the transform and the first committed run. [HW-OBL-0030](../obligations/0030-the-provenance-block-belongs-to-the-engine-and-nothing-states.md) carries `provenance.warrant`, which has no shape. A typed transcript and a transformed one are one file to every check here. So the channel is what a reader trusts in place of a warrant.

**A budget question inherits a sharper subject.** The transform reaches no model and no network, and the driver reaches both. So the mechanism a probe budget would price is the driver, whatever the open ruling on budget scope decides.
