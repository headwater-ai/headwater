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
#     sh tools/run-census.sh <session.jsonl> [top]
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
# Usage is taken once per `message.id`. One response is written to the log as
# several lines, one per content block, each carrying the same usage record,
# so a count per line inflates turns and tokens by about 2.3 times and does so
# unevenly. Every figure the evaluation cites is per message, and so is this.
#
# It gates nothing, it reads one file, and it needs `jq`.

set -u

file=${1:-}
top=${2:-20}
[ -n "$file" ] && [ -f "$file" ] || {
    echo "usage: sh tools/run-census.sh <session.jsonl> [top]" >&2
    exit 2
}
command -v jq >/dev/null 2>&1 || {
    echo "run-census: \`jq\` is not on the path." >&2
    exit 3
}

# One record per turn: id, cache reads, and the tool calls the turn made.
turns=$(jq -c -n '
    [inputs
     | select(.type == "assistant")
     | {id: .message.id,
        cr: (.message.usage.cache_read_input_tokens // 0),
        tools: [.message.content[]? | select(.type == "tool_use")
                | {name: .name, cmd: (.input.command // "")}]}]
    | group_by(.id)
    | map({id: .[0].id, cr: .[0].cr, tools: (map(.tools) | add)})
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
