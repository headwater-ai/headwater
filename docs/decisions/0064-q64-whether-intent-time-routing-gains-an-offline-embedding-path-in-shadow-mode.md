---
id: HW-DR-0064
status: current
status_since: 2026-09-11
summary: "Routing gains the shadow-mode embedding path, amended in four places. The intent hook is the only writer, and the vectors are a cache rather than a generated artifact. The model is pinned rather than committed, and the recorder gains a join key and a liveness fact."
last_verified: 2026-09-11
title: "Q64 — Whether intent-time routing gains an offline embedding path in shadow mode"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# Q64 — Whether intent-time routing gains an offline embedding path in shadow mode

## Context

[Spec 5](../spec/05-ai-integration.md#what-we-do-not-do) rules against retrieval by similarity as the primary mechanism, and leaves one door open. Embeddings stay permitted as a fallback for a genuinely fuzzy lookup, and never as the authority. `engine/crates/query/src/route.rs` has no fallback tier today. A run returns a gated hit, or one of the four `Silence` reasons.

[#404](https://github.com/headwater-ai/headwater/issues/404) proposed a second, meaning-based lookup beside the deterministic router. It writes what it would have offered to a local log, and it shows that log to nobody. The issue put three shapes to the owner. They are the routing probe alone, the deterministic-side log alone, and the full shadow log with the embedding path.

**Two facts the issue rests on have moved since it was written against `5fb9518`.** The issue says the routing probe reports nothing until a recorder exists. A recorder exists. `tools/probe/probe-record.sh` and `tools/probe/probe-transform.sh` produced the two transcripts under `docs/probe-runs/`, on 2026-09-09 and on 2026-09-11.

The issue also says that abandonment cannot be graded, because a transcript records the calls a session made and never the pointers `route` offered it. That is half right, and the half that fails is the half that matters. The router is deterministic over the tree, the taxonomy lock and the engine version, and [spec 15](../spec/15-the-recorder-contract.md) pins all three in the identity block. So the offer is recomputable after the fact. Recomputation states what `route` would have offered, and it states nothing about what the hook injected.

**Those two values differ, and the committed transcripts are where they differ.** Every probe session of the 2026-09-09 run carries a route banner from `UserPromptSubmit`. No session of the 2026-09-11 run carries one. That workspace held no built engine, so `.claude/hooks/lib.sh` returned nothing and `.claude/hooks/intent.sh` exited in silence. Nothing in the identity block records which of the two happened. Abandonment on 2026-09-11 was not unmeasured. It was measured, and the measurement is that the instrument was off.

**The measurable defect is precision at the top, and not recall on silence.** The harness session logs across the main checkout and every worktree carry 958 person-typed prompts over 27 days. The rate across the five days to 2026-09-11 is about 45 a day. The deterministic route is silent on about 12 percent of them. On a prompt that is not silent, the median count of candidates the budget withheld is 188. So the router offers five pointers out of about 190 that passed the gate. The integration shape the issue proposes fires on `NoDocumentReached` and `NoPurposeMatched` alone, which reaches the 12 percent and never the median prompt.

## Decision

**The owner ruled on 2026-09-11 for the full shadow log with the embedding path.** The four build clauses of #404 stand, and four amendments correct defects that a design review found in them. Each amendment names what breaks without it.

**The intent hook is the only writer, and clause 2 loses the word "directly".** Only the hook knows whether the pointers reached the agent or the run withheld them, and that fact is the one clause 4 needs. A writer inside the verb also fires from `.claude/hooks/write.sh` on every edit, from the tool in `engine/crates/query/src/mcp.rs`, and from every test that starts the binary. The log then fills with fixture text that no person typed.

**The vectors are a cache, and never a generated artifact.** `headwater generate` compares bytes, and quantized inference gives different bytes on different instruction sets. A committed vector file therefore fails `generate --check` between the self-hosted runner, a hosted runner and a laptop. `.headwater/cache/` already defines the shape, where every entry is recomputable from the tree it was written over. Key each vector by the model digest and the content digest of the summary, so a vector moves only when its summary moves.

**The model is pinned and fetched, and never committed, and the inference is pure Rust.** Three separate limits force this. The `ort` crate turns on binary downloads by default, and it fetches during the build. `--locked` does not prevent that fetch, and spec 6 keeps a socket out of this engine. The model is 23 to 90 megabytes, this repository configures no large-file storage, and every crate here publishes against a 10 megabyte registry cap. The pure-Rust alternative declares a compiler floor of 1.91, against the 1.90 that nine files pin. A floor raise is therefore a separate change that comes first. `taxonomy vendor` is the pattern to copy, where the caller fetches and the verb holds the digest.

**The recorder gains a join key and a liveness fact.** The driver exports a name for its session before it starts the harness. The hook writes that name into its line, and the transcript prose records whether the hook was live. Spec 15's four key sets stay closed, because the prose is the part that no verb reads.

**One line per person prompt, in a file per harness session.** A POSIX append is atomic only below 4096 bytes. One route document with its rendered report already passes that, and ten worktrees write at once on this host. A session submits its prompts in order, so a file per session has one writer. The file lives outside the tree, because `repo-cleanup` deletes a worktree and the data with it. Every line carries the wall-clock time, the session name, the corpus root, the tree digest and the lock digest. It also carries the engine version, the route document as the verb wrote it, and whether the hook injected the result.

**The hook stays silent and fails open.** It writes nothing to either stream, and it exits 0 where the log path refuses a write. `.claude/hooks/fixtures.sh` observes the two streams and the status, and it observes no write, so a fixture case plants an unwritable path and expects silence.

**The collection period is 1,000 person-prompt invocations or 30 days, whichever comes first, and not fewer than 58 silent ones.** At about 45 person prompts a day, 1,000 is about 22 days, and 12 percent silence gives about 120 silent prompts. That is twice the 58 per arm that `.headwater/probe.yml` already states. The period is an invariant on the log rather than a date, because a quiet month and a busy month are different amounts of evidence.

## Consequences

**The build lands in four steps, and step one is the shape the owner declined.** The deterministic half of the log comes first, in the hook alone, and it starts to collect on the day it lands. The model pin, the cache, the tokenizer and a neighbors verb come second. A line written before that step carries no embedding column, and it still serves the deterministic measures. The recorder join key and the liveness record come third. The document that states the mining procedure comes fourth. The chosen shape contains the cheaper shape and cannot skip it. [#819](https://github.com/headwater-ai/headwater/issues/819) carries the four clauses as this record amends them, and it carries the build order.

**The comparison counts from the day the model lands, and a corpus edit does not spoil the earlier lines.** A summary edit moves the deterministic path and the embedding path together. The tree digest on each line, and the summary digest on each pointer, keep two lines comparable. The rule is to compare at the time of the prompt, and never to re-run the router over a later tree. A second model is a second population, and the model digest on the line is what separates them.

**The shadow log is a fifth source, and never a committed rate.** Spec 5 states that its four behavioral measures come from probe transcripts. This log is engineering evidence about one mechanism, it answers to no probe plan, and nothing generates a number from it into the corpus.

**A person's session has no transcript, and that gap is now explicit.** [HW-DR-0059](0059-a-transform-over-a-harness-session-log-is-an-observed-transcript-when-the-log-arrives-by-a-channel-the-model-cannot-write-to.md) refuses a log the session can write as recorder input, and the harness project logs are such a log. This decision licenses them for the shadow comparison as engineering evidence, and never as a probe result. A probe session gives one route firing each, and the regression tier runs four a week. The transcript join alone would therefore reach about 16 prompts a month.

**Spec 5's fallback sentence stands as written.** This decision builds an instrument and integrates nothing. A pointer from the embedding path reaches no agent, no report and no hook output. Whether the fallback tier ever fires is a second ruling, and it waits on the data this decision collects.
