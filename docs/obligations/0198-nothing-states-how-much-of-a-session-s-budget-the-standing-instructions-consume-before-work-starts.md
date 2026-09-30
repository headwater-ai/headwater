---
id: HW-OBL-0198
status: discharged
status_since: 2026-10-01
summary: "Every session loads standing instructions before any task begins. The #1384 batch measured this repository's share at a median of 7,225 tokens, and spec 5 states it."
last_verified: 2026-10-01
title: "Nothing states how much of a session's budget the standing instructions consume before work starts"
waiting_on: measurement
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5.5
  activity: draft+revise
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-ai-integration
---

# Nothing states how much of a session's budget the standing instructions consume before work starts

## Context

Every Claude Code session in this repository loads standing instructions before any task work begins. The load includes the global CLAUDE.md file from `~/.claude/`. It also includes the project CLAUDE.md file at the repository root. The load adds system reminders, routing information from the Headwater governance engine, user context, and git status snapshots. This load is a fixture of every dispatch to an agent and every skill invocation.

The load consumes tokens from a session's budget. When this record was opened on 2026-09-16, it gave a range for the load that nobody had measured. No specification or planning rule stated the cost. No document stated what a session should assume or whether the cost matters.

## Obligation

A measured baseline cost is what the corpus owes. A decision on its significance is also required. The instrument is either a quantified measurement or a decision record. A quantified measurement states the token count under defined conditions. These conditions include agent type, codebase size, and session mode. A decision record rules measurement out of scope and states what a session should assume.

## Discharge

Discharge is satisfied by one of two paths.

First path: a measurement taken over a representative set of sessions. These sessions can be drawn from the test grid or from real sessions. The cost must be recorded in a specification, tutorial, or decision document. That document must state the conditions under which the cost holds.

Second path: a decision record. This record rules measurement out of scope. It states explicitly what an agent or planner should assume when allocating budget.

In either case, the document must be retrievable from this obligations register via a relation. A session or agent reading the corpus for budget guidance must be able to find it.

### How the first path paid this record

The change for #1473 took the first path. [Spec 5](../spec/05-ai-integration.md) states the figure and its conditions, and this record reaches it through its `traces_to` relation.

The source is the batch of #1384, recorded in PR #1465. It ran on 2026-09-30 at the pin `109abba7`, with `claude-sonnet-5`, seed 980 and a turn cap of 80, on one host. The batch has one log for each of its 660 sessions. The table below uses the 540 logs of the `campaign` tier.

The instrument is the prompt before the first turn. The first assistant event of a log carries it as `input_tokens + cache_creation_input_tokens + cache_read_input_tokens`. The harness repeats that usage on each streamed part of one message, so the first line is the whole prompt. For each probe, the median of the present arm minus the median of the absent arm is the difference.

The absent arm removes `CLAUDE.md`, `.claude/`, `.githooks/` and `.headwater/`. In the init event of each log, the present arm shows 42 skills and 14 agents. The absent arm shows 33 skills and 5 agents. The present arm lists 12 more slash commands: the 9 skills and the 3 commands `next`, `next-run` and `product-owner`. So the difference is the project `CLAUDE.md` and the output of the intent hook on the first prompt. It also holds the descriptions of 9 skills, 3 commands and 9 agents. The log does not divide the difference among these five parts.

This command, run as written, printed the table that follows it:

```sh
cd /home/james/.claude/jobs/1338ac1e/tmp/issue-1384/full/campaign/sessions &&
for d in L[1-6]-campaign-*; do
  printf '%s %s\n' "${d%-r*}" "$(jq -r 'select(.type=="assistant") | .message.usage | .input_tokens + .cache_creation_input_tokens + .cache_read_input_tokens' "$d/raw.jsonl" | head -1)"
done | sort -k1,1 -k2,2n | awk '
  { k = $1; v[k, ++n[k]] = $2; if ($2 > hi[k]) { hi[k] = $2; c[k] = 0 } if ($2 == hi[k]) c[k]++ }
  function med(k) { return (n[k] % 2) ? v[k, (n[k] + 1) / 2] : (v[k, n[k] / 2] + v[k, n[k] / 2 + 1]) / 2 }
  END {
    print "| probe | present median | absent median | difference | present sessions at the higher value | absent sessions at the higher value |"
    print "|---|---|---|---|---|---|"
    split("L1:L2:sufficiency:4 L3:L4:navigability:3 L5:L6:discovery:2", pairs, " ")
    for (i = 1; i <= 3; i++) {
      split(pairs[i], f, ":")
      for (p = 1; p <= f[4]; p++) {
        a = f[1] "-campaign-present-p" p; b = f[2] "-campaign-absent-p" p
        printf "| %s p%d | %d | %d | %d | %d of %d | %d of %d |\n", f[3], p, med(a), med(b), med(a) - med(b), c[a], n[a], c[b], n[b]
      }
    }
  }'
```

| probe | present median | absent median | difference | present sessions at the higher value | absent sessions at the higher value |
|---|---|---|---|---|---|
| sufficiency p1 | 43007 | 35586 | 7421 | 29 of 30 | 24 of 30 |
| sufficiency p2 | 42780 | 35530 | 7250 | 28 of 30 | 26 of 30 |
| sufficiency p3 | 42903 | 35562 | 7341 | 25 of 30 | 27 of 30 |
| sufficiency p4 | 42764 | 35499 | 7265 | 28 of 30 | 22 of 30 |
| navigability p1 | 42487 | 35441 | 7046 | 26 of 30 | 26 of 30 |
| navigability p2 | 42521 | 35448 | 7073 | 27 of 30 | 23 of 30 |
| navigability p3 | 42711 | 35496 | 7215 | 29 of 30 | 23 of 30 |
| discovery p1 | 42752 | 35527 | 7225 | 26 of 30 | 21 of 30 |
| discovery p2 | 42399 | 35416 | 6983 | 29 of 30 | 25 of 30 |

Over the 9 probes, the median difference is 7,225 tokens, and the range is 6,983 to 7,421. The absent medians are 35,416 to 35,586, so the difference adds about 20% to the absent baseline.

Each probe takes a higher value and, in some sessions, a value 2,316 tokens lower. The cause is the host and not the tree. Over all 660 sessions of the batch, the init event shows the state of each connector. In all 100 sessions that started before the claude.ai Claude Docs connector was connected, the prompt was lower. In 559 of the 560 sessions that started with it connected, the prompt was at the higher value. The one exception started without a different connector and fell by 194 tokens. The lower value occurs in both arms, so the median of each arm stays at the higher value. The `documentation` absent arm gave the same medians as the `campaign` absent arm. So the removal of `docs/` does not change the prompt before the first turn.

The logs are in a job directory outside the tree, and the tree keeps only this table. The figure is for the tree at `109abba7`. `CLAUDE.md` changed after that tree, so the figure is a dated measurement.
