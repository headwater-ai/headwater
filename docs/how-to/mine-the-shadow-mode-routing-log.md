---
id: HW-HOW-mine-the-shadow-mode-routing-log
status: current
status_since: 2026-09-17
summary: "Join the shadow log to the session transcripts by prompt identifier, and read the deterministic route against the embedding path one prompt at a time."
last_verified: 2026-09-17
title: "Mine the shadow-mode routing log"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: draft
  evidence_basis: reconstructed
relations:
  traces_to:
    - HW-DR-0064
---

# Mine the shadow-mode routing log

**Audience:** a contributor to this repository. An adopter of Headwater needs no step of this guide, and the consumer surface in `.headwater/overlay.yml` does not list it.

## Before you start

[HW-DR-0064](../decisions/0064-q64-whether-intent-time-routing-gains-an-offline-embedding-path-in-shadow-mode.md) sets the collection period. Wait for 1,000 person prompts or 30 days, whichever comes first. The period also needs 58 silent invocations or more. A count below either bound is a measurement of the collector rather than of the router.

Two sources join here. The first is the shadow log, which `.claude/hooks/intent.sh` writes to `<git common dir>/headwater-shadow-log/<harness session>.jsonl`. The second is the harness session logs under `~/.claude/projects/`. HW-DR-0064 licenses the session logs for this comparison as engineering evidence. [HW-DR-0059](../decisions/0059-a-transform-over-a-harness-session-log-is-an-observed-transcript-when-the-log-arrives-by-a-channel-the-model-cannot-write-to.md) still refuses them as a probe result, because the session can write them.

Every command below was run on 2026-09-17, against the log of this host and the session logs beside it. The period held 21 lines that day. So these commands are reconstructed from a collection far under the bound above, and never from a finished one.

You need `jq`. Set two variables first. The examples below use them.

    log=$(git rev-parse --path-format=absolute --git-common-dir)/headwater-shadow-log
    sessions=~/.claude/projects/-home-james-projects-headwater*

Keep the trailing star. The harness keeps one project directory per working directory, so every worktree of this clone has a directory of its own beside the main checkout's. A glob without the star reads the main checkout alone, and it reports every worktree session as a gap.

Three members of a line need care. `route` and `neighbors` are JSON documents held as strings, so every read of them passes through `fromjson`. `neighbors` is `null` where the run loaded no model, and it is absent on every line that step 2 of [#819](https://github.com/headwater-ai/headwater/issues/819) predates. `probe_session` names the recorder that submitted the prompt, and it is empty for a person.

## Steps

**1. Take the population.** Drop every line that a recorder submitted, and count what remains.

    jq -s 'map(select((.probe_session // "") == "")) | length' "$log"/*.jsonl
    jq -s 'map(select((.probe_session // "") == "") | select(.injected == false)) | length' "$log"/*.jsonl

The first count is the period against the 1,000. The second is the silent count against the 58.

**2. Separate the models.** A second model digest is a second population, and no report mixes them.

    jq -sr 'map(.model_digest // "none") | group_by(.) | map({digest: .[0], lines: length})' "$log"/*.jsonl

Read every later step against one digest. The `tree_digest` and `lock_digest` on each line say which corpus the line was written over. Compare at the time of the prompt. Never re-run the router over a later tree, because a summary edit moves both paths at once.

**3. Join to the transcripts.** The shadow log's `prompt_id` is the transcript's `promptId`, and the log file name is the session identifier. A typed prompt carries `origin.kind == "human"`.

    jq -r 'select((.probe_session // "") == "") | select(.prompt_id != "") | [.prompt_id, .session] | @tsv' "$log"/*.jsonl | LC_ALL=C sort > /tmp/logged
    jq -Rr 'try fromjson | select(.origin.kind == "human") | [.promptId, .sessionId, .timestamp] | @tsv' $sessions/*.jsonl | LC_ALL=C sort > /tmp/typed

The prompt identifier is the first field of each file, because that is the key the next step joins on. A join on the session identifier instead would hide every gap inside a session that logged anything at all.

Both filters earn their place. `LC_ALL=C` fixes the collation, because `join` refuses two files that two locales ordered differently. The empty `prompt_id` test drops the lines that a harness wrote before it passed an identifier. Those lines hold a route and no join key, so count them and report the count beside the gaps.

    jq -s 'map(select((.probe_session // "") == "") | select(.prompt_id == "")) | length' "$log"/*.jsonl

`try fromjson` is not decoration. A session log holds lines that are not JSON, and a plain `jq` stops at the first one.

**4. Report the discontinuities.** A typed prompt with no line is a gap in collection. Report it as a gap. A gap is never a silence, because a silent route still writes a line of its own.

    first=$(jq -sr 'map(.at) | min' "$log"/*.jsonl)
    awk -F'\t' -v first="$first" '$3 >= first' /tmp/typed > /tmp/typed-in-period
    join -v2 -t"$(printf '\t')" /tmp/logged /tmp/typed-in-period | cut -f2 | sort | uniq -c

The period starts at the first line of the log, so the third field of `/tmp/typed` cuts the prompts that this collection never covered. Without that cut the gap count holds every prompt anybody typed before the log existed.

The other direction matters as much, and it is the one a reader forgets. A logged line that no typed prompt claims is a prompt that the harness submitted rather than a person.

    join -v1 -t"$(printf '\t')" /tmp/logged /tmp/typed-in-period | wc -l

Report that count apart. HW-DR-0064 counts person prompts, so these lines leave the population. The three buckets are the whole of the join: the joined rows, the gaps of the step above, and these.

**5. Normalize the paths.** A pointer path is relative to the corpus root, such as `docs/spec/05-ai-integration.md`. A transcript records a `Read` call with an absolute path, such as `/home/james/projects/headwater/docs/spec/05-ai-integration.md`. Each line carries its own `corpus_root`, so strip that prefix and one slash from the absolute path. A read whose path does not start with the line's `corpus_root` is outside the corpus. Drop it. A session reads its own dependencies and its own machine's files, and neither is a document this corpus governs.

**6. Read the two paths against each other.** For one prompt, take three sets. The first is what the deterministic route offered, which is `route | fromjson | .pointers[].path`. The second is what the embedding path would have offered, which is `neighbors | fromjson | .neighbors[].path`. The third is what the session read after the prompt, normalized by step 5.

Then count, over the population:

- The silence rate of each path, where the deterministic reason is `route | fromjson | .silence`.
- The agreement, which is the overlap of the first two sets.
- The recall of each path against the third set, which is the only set a person did not choose.
- The rank at which each path first offers a document that the session read.

**7. State what the numbers cannot say.** The third set is what the session read with the pointers in front of it, so it is not a control. A prompt whose `injected` member is `true` had the pointers in context, and a prompt whose member is `false` did not. Report those two groups apart.

## How to know it worked

The counts reconcile in both directions. The typed prompts of the period equal the joined rows plus the gaps. The logged lines of the period equal the joined rows plus the lines that no typed prompt claims. A procedure that states one of those two sums alone has hidden the other bucket.

The report names one model digest, one tree digest range and the date of the first and last line it read.

The report states the two groups of step 7 apart, and it states the gap count beside the silence count. A report that gives one silence rate over a period with unreported gaps is the failure this procedure exists to prevent.
