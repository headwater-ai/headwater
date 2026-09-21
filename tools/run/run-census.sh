#!/bin/sh
# The census of one session transcript: what the parent's turns were spent on.
#
# [HW-PD-0003] sets the unit of orchestration cost at one parent turn at full
# context, and the measurement behind it was made by hand over a 207M-token
# transcript: 76 `gh pr view` calls, spread over 43 pull requests so that no
# single command repeated enough to look like a loop, cost 77 turns and 21.3M
# cache reads, 10.3% of the run, to carry 25 KB of answers. Nothing in a turn
# announces that, and a parent that counts its repeats misses the version
# spread thin. This is that measurement as a tool, so the next run is held to
# the numbers the evaluation records rather than to a memory of them.
#
#     sh tools/run/run-census.sh <session.jsonl> [top]
#
# Three tables. The first groups every tool call by tool name. The second
# groups `Bash` calls by their leading verb, two words for `gh`, `git`,
# `cargo`, `sh` and `headwater`, one for the rest, with a leading environment
# assignment and a leading path dropped; a call is in exactly one row. The
# third counts what a `Bash` command mentions anywhere in its text, so a poll
# inside a `for` or an `until` loop is counted as the poll it is, and a call
# can be in several rows; that is the reading #685 made by hand, and the two
# tables together say both what a turn was for and what it ran. A row is the
# calls in that group, the distinct turns that made one, the cache reads
# those turns re-read, and that as a share of the whole session's cache reads.
#
# A fourth line follows the tables: the turns whose cache write outweighed
# their cache read after a gap past the prompt-cache lifetime, the tokens they
# rewrote, and what that cost at Sonnet 5's cache-write rate, $2.50 per
# million tokens for the five-minute lifetime and $4.00 for the one-hour one,
# against $0.20 to have read the same tokens back.
# #983 measured why the class matters: a wait that ends the turn for longer
# than the cache holds a copy comes back to a discarded one and pays to write
# the whole context again rather than read it, twelve and a half times the
# price, and thirteen such wake-ups in one run cost $14.85 of a $130.73 total.
# A transcript with no timestamp on a turn reports zero rather than guessing.
#
# A fleet section follows the tables, read from the agent transcripts the
# harness writes beside the session file: the share of the span with no agent
# in flight, the mean number in flight, the gap around each compaction, and
# the parent's turns per agent of each type.
#
# Usage is taken once per `message.id`. One response is written to the log as
# several lines, one per content block, each carrying the same usage record,
# so a count per line inflates turns and tokens by about 2.3 times and does so
# unevenly. Every figure the evaluation cites is per message, and so is this.
#
# It gates nothing, it reads one session and the agent transcripts beside it,
# and it needs `jq`.

set -u

file=${1:-}
top=${2:-20}
[ -n "$file" ] && [ -f "$file" ] || {
    echo "usage: sh tools/run/run-census.sh <session.jsonl> [top]" >&2
    exit 2
}
command -v jq >/dev/null 2>&1 || {
    echo "run-census: \`jq\` is not on the path." >&2
    exit 3
}

# A heredoc body is data a command writes and not a command it runs, and the
# measured run wrote its ledger with one, a `cat >> decisions.md << EOF`, a
# body, then `EOF`. Left in, a word the body happens to use, a stray `for`, a
# `cargo test` a subagent quoted in its own report, reads as a mention the
# parent never made. `strip_heredocs` removes the span below, once, before
# either table reads `.cmd`, so a call is judged on what it runs.

# One record per turn: id, cache reads, cache writes, timestamp, and the tool
# calls the turn made. `group_by(.id)` would answer the three tables above,
# which only sum and count, but the expiry class below reads the gap between
# one turn and the next, so the turns have to stay in the order the file
# wrote them. A message's lines are written together, so a reduce that folds
# a line into the previous turn when the id repeats and opens a new one when
# it does not keeps that order at one pass.
turns=$(jq -c -n '
    def strip_heredocs:
        gsub("<<-?[ \t]*['"'"'\"]?(?<marker>[A-Za-z_][A-Za-z0-9_]*)['"'"'\"]?\n(?:(?!^\\k<marker>$).)*\n[ \t]*\\k<marker>";
             ""; "sm");
    reduce (inputs | select(.type == "assistant")) as $l
        ([];
         ({id: $l.message.id,
           cr: ($l.message.usage.cache_read_input_tokens // 0),
           cw: ($l.message.usage.cache_creation_input_tokens // 0),
           cw1h: ($l.message.usage.cache_creation.ephemeral_1h_input_tokens // 0),
           ts: ($l.timestamp // null),
           tools: [$l.message.content[]? | select(.type == "tool_use")
                   | {name: .name, cmd: (.input.command // "" | strip_heredocs)}]}) as $t
         | if length > 0 and .[-1].id == $t.id
           then .[-1].tools += $t.tools
           else . + [$t] end)
' "$file")

total_turns=$(printf '%s' "$turns" | jq 'length')
total_cr=$(printf '%s' "$turns" | jq 'map(.cr) | add // 0')
total_calls=$(printf '%s' "$turns" | jq 'map(.tools | length) | add // 0')

printf 'turns %s  tool calls %s  cache reads %s\n' "$total_turns" "$total_calls" "$total_cr"

# A table over a key function: calls, distinct turns, cache reads of those
# turns, and the share. `$key` maps one tool call to its group, to an array of
# groups, or to null to leave it out.
table() {
    title=$1 key=$2
    printf '\n%s\n' "$title"
    printf '%-32s %8s %8s %14s %7s\n' group calls turns cache_reads share
    printf '%s' "$turns" | jq -r --argjson total "$total_cr" "
        def key: $key;
        [ .[] as \$t
          | \$t.tools[]
          | (key) as \$ks
          | (\$ks | if type == \"array\" then . else [.] end)[] as \$k
          | select(\$k != null)
          | {k: \$k, id: \$t.id, cr: \$t.cr} ]
        | group_by(.k)
        | map({group: .[0].k,
               calls: length,
               turns: (map(.id) | unique | length),
               cr: ([group_by(.id)[] | .[0].cr] | add // 0)})
        | sort_by(-.cr)
        | .[:$top][]
        | [.group, .calls, .turns, .cr,
           (if \$total > 0 then (.cr * 1000 / \$total | round / 10) else 0 end)]
        | @tsv
    " | awk -F'\t' '{ printf "%-32s %8s %8s %14s %6s%%\n", $1, $2, $3, $4, $5 }'
}

table 'by tool' '.name'

# The leading verb of a Bash command. A compound command is split on `&&`,
# `||`, `;`, `|` and a newline, and the verb is the first segment's that is
# not setup: the run this was measured on wrote nearly every call as
# `cd <dir> && <verb> …` or `set -e; <verb> …`, and a census by first word
# reported `cd` and `set` and hid the poll it exists to find. Within a
# segment, `VAR=value` prefixes go, a path becomes its basename, and the
# second word is kept for the five families whose first word says nothing.
table 'Bash, by leading verb' '
    def verb_of: split(" ") | map(select(. != ""))
       | (if length == 0 then [""] else . end)
       | . as $w
       | ([range(0; length)] | map(select($w[.] | test("^[A-Za-z_][A-Za-z0-9_]*=$|^[A-Za-z_][A-Za-z0-9_]*=") | not)) | .[0] // 0) as $i
       | $w[$i:]
       | (.[0] // "" | split("/") | last) as $v
       | if ($v | IN("gh", "git", "cargo", "sh", "headwater")) and (.[1] != null)
         then $v + " " + (.[1] | split("/") | last)
         else $v end;
    if .name != "Bash" then null else
      (.cmd
       | gsub("&&|\\|\\||;|\\||\n"; "\u0001")
       | split("\u0001")
       | map(gsub("^ +| +$"; "") | select(. != ""))
       | map(verb_of)
       | (map(select(IN("cd", "set", "export", "unset", "trap", "true", ":", "(", "{") | not)) | .[0]) // (.[0] // ""))
    end'

# What a Bash command mentions, anywhere in its text. The families are the
# ones the evaluation prices: the polls, the builds, the engine verbs, and the
# two loop keywords that hide a poll from the table above.
table 'Bash, by what the command mentions (a call can be in several rows)' '
    if .name != "Bash" then null else
      (.cmd as $c
       | ["gh pr", "gh pr view", "gh pr list", "gh issue", "gh api", "gh run", "cargo test", "cargo build",
          "headwater check", "headwater generate", "git fetch", "git log",
          "git merge", "git rebase", "until ", "while ", "for ", "sleep "]
       | map(select(. as $p | $c | test("(^|[^A-Za-z0-9_/.-])" + $p))))
    end'

# Past the cache lifetime the harness discards its copy of a turn's context,
# and the turn that ends up reading the notification pays to write the whole
# thing back rather than to read it. The lifetime is not one number. A
# subagent writes to the five-minute cache and the parent to the one-hour one,
# which HW-PD-0007 measured on 2026-09-21, and each usage record says which
# in `cache_creation`. A turn that wrote to the one-hour cache is held to
# 3600 seconds, and every other turn to 300, which is also the reading of a
# transcript that predates the split. A turn is in this class when its cache
# write outweighs its cache read and the gap since the previous turn passed
# its own lifetime. The write rates are Sonnet 5's, $2.00 per million input
# tokens times the API's 1.25x for a five-minute write and 2x for a one-hour
# write, because that is what the run's agents run under.
expiry=$(printf '%s' "$turns" | jq -c '
    def ts: sub("\\.[0-9]+Z$"; "Z") | fromdateiso8601;
    [.[] | select(.ts != null)] as $timed
    | [range(1; $timed | length) as $i
       | $timed[$i] as $cur | $timed[$i - 1] as $prev
       | (($cur.ts | ts) - ($prev.ts | ts)) as $gap
       | (if $cur.cw1h > 0 then 3600 else 300 end) as $lifetime
       | select($gap > $lifetime and $cur.cw > $cur.cr)
       | {tokens: $cur.cw, cost: (($cur.cw - $cur.cw1h) * 2.50 + $cur.cw1h * 4.00) / 1000000}]
    | {n: length, tokens: (map(.tokens) | add // 0), cost: (map(.cost) | add // 0)}
')
expiry_n=$(printf '%s' "$expiry" | jq '.n')
expiry_tokens=$(printf '%s' "$expiry" | jq '.tokens')
expiry_cost=$(printf '%s' "$expiry" | jq -r '.cost' | awk '{ printf "%.2f", $1 }')
printf '\nexpiry-class wake-ups %s  tokens rewritten %s  cost $%s\n' "$expiry_n" "$expiry_tokens" "$expiry_cost"

# The fleet: what the agents this session dispatched were doing while its
# turns were spent. The harness writes each agent's own transcript beside the
# session's, as `<session>/subagents/agent-<id>.jsonl` with a `.meta.json`
# naming its type, its depth and the `Agent` call that dispatched it, and the
# first and last timestamps of that file are the interval the agent was in
# flight. The parent's transcript is not enough for this: a completion notice
# that lands while the parent is mid-turn is written as an attachment and
# not as a message, so a count of notices misses a fifth of the fleet. A
# sweep over the depth-one intervals gives the share of the span with no
# agent in flight and the mean number in flight, which are the two figures
# the evaluation's baseline row took by hand. Each compaction is listed with
# the gap from the parent's last turn before it and the gap to its first
# dispatch after it. Turns per dispatch is this session's turns over the
# agents of that type, so the row for the integrator is the parent's turns
# per merge.
printf '\nfleet\n'
subdir="${file%.jsonl}/subagents"
if ! ls "$subdir"/agent-*.meta.json >/dev/null 2>&1; then
    echo "no agent transcripts beside the session file, so no fleet"
    exit 0
fi
spans=$(jq -R -c -n '
    reduce (inputs | try fromjson catch null) as $l ({};
        if ($l | type) != "object" or $l.timestamp == null then . else
          (input_filename | split("/") | last | sub("\\.jsonl$"; "")) as $k
          | .[$k] |= {min: ([.min, $l.timestamp] | map(select(. != null)) | min),
                      max: ([.max, $l.timestamp] | map(select(. != null)) | max)}
        end)
' "$subdir"/agent-*.jsonl)
metas=$(jq -c '
    {id: (input_filename | split("/") | last | sub("\\.meta\\.json$"; "")),
     type: (.agentType // "default"), depth: (.spawnDepth // 1)}
' "$subdir"/agent-*.meta.json | jq -s .)
jq -r -n --argjson turns "$total_turns" --argjson spans "$spans" --argjson metas "$metas" '
    def ts: sub("\\.[0-9]+Z$"; "Z") | fromdateiso8601;
    def mins: . / 6 | round / 10;
    def hours: . / 360 | round / 10;
    def pct: . * 1000 | round / 10;
    def pad($w): tostring | " " * ($w - length) + .;
    [inputs] as $all
    | ($all | map(select(.type == "assistant" and .timestamp != null) | .timestamp | ts)) as $tt
    | [ $all[] | select(.type == "assistant" and .timestamp != null) | (.timestamp | ts) as $t
        | .message.content[]? | select(.type == "tool_use" and .name == "Agent") | $t ] as $disp
    | [ $metas[] | select($spans[.id] != null)
        | . + {t: ($spans[.id].min | ts), end: ($spans[.id].max | ts)} ] as $every
    | ($every | map(select(.depth == 1))) as $agents
    | "dispatched \($disp | length) by Agent calls, \($agents | length) agents at depth one, \($every | length - ($agents | length)) deeper",
      (if ($agents | length) == 0 then "no agent at depth one, so no span" else
        (($agents | map({t: .t, d: 1})) + ($agents | map({t: .end, d: -1})) | sort_by(.t, .d)) as $ev
        | (reduce $ev[] as $e ({n: 0, t: null, busy: 0, area: 0, gap: 0};
              (if .t == null then 0 else ($e.t - .t) end) as $dt
              | {n: (.n + $e.d), t: $e.t,
                 busy: (.busy + (if .n > 0 then $dt else 0 end)),
                 area: (.area + .n * $dt),
                 gap: (if .n == 0 and .t != null and $dt > .gap then $dt else .gap end)})) as $sw
        | (($agents | map(.end) | max) - ($agents | map(.t) | min)) as $span
        | "span \($span | hours) h from the first agent'"'"'s start to the last agent'"'"'s end",
          "idle \(if $span > 0 then ((1 - $sw.busy / $span) | pct) else 0 end)% of the span with no agent in flight, \(($span - $sw.busy) | hours) h, largest window \($sw.gap | mins) min",
          "mean concurrency \(if $span > 0 then ($sw.area / $span * 100 | round / 100) else 0 end) agents in flight"
       end),
      ($all | map(select(.isCompactSummary == true) | .timestamp | ts)) as $comp
      | "compactions \($comp | length)",
        ($comp[] | . as $c
          | "  \($c | todate)  \(($c - ([$tt[] | select(. < $c)] | max // $c)) | mins) min since the parent'"'"'s last turn, \((([$disp[] | select(. > $c)] | min // $c) - $c) | mins) min to its next dispatch"),
      (if ($agents | length) > 0 then
        "by agent type                    agents   mean_min   turns_per_agent",
        ($agents | group_by(.type) | sort_by(-length)[]
          | "\(.[0].type | . + " " * (32 - length))\(length | pad(6))\((map(.end - .t) | add) / length | mins | pad(11))\($turns / length * 10 | round / 10 | pad(18))")
       else empty end)
' "$file"
