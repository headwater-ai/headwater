#!/bin/sh
# What holds `tools/run/shadow-mine.sh`.
#
# The tool exists so that a reading of the shadow log cannot give one silence
# rate over two models, or one bucket of the join without the other two, which
# is the failure the last paragraph of
# `docs/how-to/mine-the-shadow-mode-routing-log.md` names. So the cases are
# planted transcripts and a planted log whose right answer was worked by hand:
#
#   a  two model digests whose silence rates differ, and a deterministic-only
#      row, so that a pooled rate equals no row: digest A is silent on 1 of 2
#      prompts on each path, digest B on 0 of 2, and the no-digest row on 1
#      of 1. The log also holds one gap, two ids that no typed prompt claims,
#      one prompt logged twice, one torn line, one empty line (#1420's
#      unlocked writer can leave one), one line with an empty `prompt_id`,
#      one line a recorder submitted, and one typed prompt from
#      before the first line. A pooled rate, a raw line count or a missing
#      bucket fails here, and so does a probe line read as a person's.
#   b  recall and rank against planted `Read` calls. Of nine reads, two count:
#      one before the prompt, one outside `corpus_root`, one under a sibling
#      directory that shares the root as a string prefix, one on a sidechain,
#      one that is not a Markdown document, one under a directory whose name
#      starts with a dot and one after the next person prompt are each
#      dropped, and a read made twice counts once. Each drop moves a
#      denominator.
#   c  the injected and not-injected split, over both cases above.
#   d  the bound: the date the bound closed is read on distinct ids in time
#      order, so a prompt logged twice does not close it early.
#   e  the arguments and the exit without `jq`.
#
# Run it from anywhere:
#     sh tools/run/shadow-mine-fixtures.sh
#
# It needs `jq`, exits 3 without it so CI can skip with a warning, and writes
# only under a temporary directory.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="${SHADOW_MINE_TOOL:-$root/tools/run/shadow-mine.sh}"

if ! command -v jq >/dev/null 2>&1; then
    echo "no \`jq\` on the path, and the tool under test reads JSON with it." >&2
    exit 3
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0
pass() { printf 'ok   %s\n' "$1"; passed=$((passed + 1)); }
fail() { printf 'FAIL %s\n  %s\n' "$1" "$2"; failed=$((failed + 1)); }
# has <case> <output> <exact line>
has() {
    if grep -Fqx -- "$3" "$2"; then
        pass "$1: $3"
    else
        fail "$1: $3" "not in the output; got: $(tr '\n' '|' < "$2")"
    fi
}
# lacks <case> <output> <fixed string>
lacks() {
    if grep -Fq -- "$3" "$2"; then
        fail "$1: no line holds $3" "found: $(grep -F -- "$3" "$2" | tr '\n' '|')"
    else
        pass "$1: no line holds $3"
    fi
}
rc() {
    if [ "$(cat "$scratch/$2.rc")" = "$3" ]; then pass "$1"; else fail "$1" "exit $(cat "$scratch/$2.rc"), wanted $3"; fi
}

# A checkout that holds an engine, and one that does not.
mkdir -p "$scratch/co/engine/target/dev-release" "$scratch/bare"
printf '#!/bin/sh\n' > "$scratch/co/engine/target/dev-release/headwater"
chmod +x "$scratch/co/engine/target/dev-release/headwater"

# start <case> <session> <start dir>
start() {
    mkdir -p "$scratch/$1/t" "$scratch/$1/log"
    printf '{"type":"system","sessionId":"%s","cwd":"%s"}\n' "$2" "$3" >> "$scratch/$1/t/$2.jsonl"
}
# prompt <case> <session> <timestamp> <id>
prompt() {
    printf '{"type":"user","isSidechain":false,"origin":{"kind":"human"},"promptId":"%s","sessionId":"%s","timestamp":"%s","message":{"role":"user","content":"task %s"}}\n' \
        "$4" "$2" "$3" "$4" >> "$scratch/$1/t/$2.jsonl"
}
# read <case> <session> <timestamp> <absolute path> [sidechain]
readcall() {
    printf '{"type":"assistant","isSidechain":%s,"sessionId":"%s","timestamp":"%s","message":{"role":"assistant","content":[{"type":"text","text":"x"},{"type":"tool_use","name":"Read","input":{"file_path":"%s"}}]}}\n' \
        "${5:-false}" "$2" "$3" "$4" >> "$scratch/$1/t/$2.jsonl"
}
# route <paths, space-separated, or "silent">: the `route` member as the hook
# writes it, a JSON document held in a string.
route() {
    if [ "$1" = silent ]; then
        jq -cn '{version: "1.0", pointers: [], silence: {reason: "no_purpose_matched"}} | tojson'
    else
        jq -cn --arg p "$1" '{version: "1.0", pointers: ($p | split(" ") | map({path: .})), silence: null} | tojson'
    fi
}
# neighbors <digest> <paths, space-separated, or "">
neighbors() {
    jq -cn --arg d "$1" --arg p "$2" '{model_digest: $d, neighbors: ($p | split(" ") | map(select(. != "")) | map({path: ., score: 0.5}))} | tojson'
}
# logline <case> <file> <at> <id> <injected> <digest or none> <route> <neighbors> [probe] [root]
logline() {
    if [ "$6" = none ]; then
        printf '{"at":"%s","session":"%s","prompt_id":"%s","probe_session":"%s","corpus_root":"%s","injected":%s,"skip":null,"route":%s}\n' \
            "$3" "$2" "$4" "${9:-}" "${10:-/root/c}" "$5" "$7" >> "$scratch/$1/log/$2.jsonl"
    else
        printf '{"at":"%s","session":"%s","prompt_id":"%s","probe_session":"%s","corpus_root":"%s","injected":%s,"skip":null,"route":%s,"model_digest":"%s","neighbors":%s}\n' \
            "$3" "$2" "$4" "${9:-}" "${10:-/root/c}" "$5" "$7" "$6" "$8" >> "$scratch/$1/log/$2.jsonl"
    fi
}
# run <case> [<arguments>...]
run() {
    _c=$1
    shift
    HEADWATER_TRANSCRIPT_DIRS="$scratch/$_c/t" HEADWATER_SHADOW_LOG_DIR="$scratch/$_c/log" \
        sh "$tool" "$@" > "$scratch/$_c.out" 2> "$scratch/$_c.err"
    echo $? > "$scratch/$_c.rc"
}

A=sha256:aaaa
B=sha256:bbbb
silent=$(route silent)

# a: session S1 started in a checkout with an engine, S2 in one without.
start a S1 "$scratch/co"
prompt a S1 2026-09-30T09:00:00.000Z p0
prompt a S1 2026-09-30T10:00:00.000Z p1
prompt a S1 2026-09-30T10:10:00.000Z p2
prompt a S1 2026-09-30T10:20:00.000Z p3
prompt a S1 2026-09-30T10:30:00.000Z p4
start a S2 "$scratch/bare"
prompt a S2 2026-09-30T11:00:00.000Z q1
prompt a S2 2026-09-30T11:10:00.000Z q2
logline a S1 2026-09-30T10:00:05Z p1 false "$A" "$silent" "$(neighbors "$A" docs/x.md)"
logline a S1 2026-09-30T10:10:05Z p2 true "$A" "$(route 'docs/x.md docs/y.md')" "$(neighbors "$A" '')"
printf '{"at":"2026-09-30T10:1\n' >> "$scratch/a/log/S1.jsonl"
logline a S1 2026-09-30T10:20:05Z p3 true "$B" "$(route docs/y.md)" "$(neighbors "$B" 'docs/y.md docs/z.md')"
logline a S1 2026-09-30T10:10:30Z p2 true "$A" "$(route 'docs/x.md docs/y.md')" "$(neighbors "$A" '')"
logline a S1 2026-09-30T10:30:05Z p4 false none "$silent" ""
logline a S2 2026-09-30T11:00:05Z q1 true "$B" "$(route docs/z.md)" "$(neighbors "$B" docs/z.md)"
logline a S2 2026-09-30T11:05:00Z "" true "$B" "$(route docs/z.md)" "$(neighbors "$B" docs/z.md)"
printf '\n' >> "$scratch/a/log/S2.jsonl"
logline a S2 2026-09-30T11:20:00Z u1 true "$B" "$(route docs/z.md)" "$(neighbors "$B" docs/z.md)"
logline a S2 2026-09-30T11:30:00Z u2 false "$A" "$silent" "$(neighbors "$A" docs/x.md)"
logline a S2 2026-09-30T11:40:00Z pr1 false "$A" "$silent" "$(neighbors "$A" docs/x.md)" rec-1
run a
o="$scratch/a.out"
rc "a exits 0" a 0
has a "$o" "log lines: 12"
has a "$o" "torn lines: 1"
has a "$o" "blank lines: 1"
lacks a "$o" "torn: S2.jsonl"
has a "$o" "probe lines: 1"
has a "$o" "empty-id lines: 1"
has a "$o" "torn: S1.jsonl:3"
has a "$o" "empty-id: S2.jsonl:2"
has a "$o" "first line: 2026-09-30T10:00:05Z"
has a "$o" "last line: 2026-09-30T11:30:00Z"
has a "$o" "typed: 6"
has a "$o" "logged: 7"
has a "$o" "joined: 5"
has a "$o" "gaps: 1"
has a "$o" "unclaimed: 2"
has a "$o" "typed = joined + gaps: 6 = 5 + 1"
has a "$o" "logged = joined + unclaimed: 7 = 5 + 2"
has a "$o" "joined lines: 6"
has a "$o" "joined ids with more than one line: 1"
has a "$o" "silent person prompts: 2 (floor 58: not met)"
has a "$o" "bound reached: no, 5 of 500"
has a "$o" "held typed: 4"
has a "$o" "held joined: 4"
has a "$o" "held gaps: 0"
has a "$o" "start directory engine: 4"
has a "$o" "start directory none: 2"
has a "$o" "start directory gone: 0"
has a "$o" "unrestricted $A all prompts: 2"
has a "$o" "unrestricted $A all deterministic silent: 1 of 2 (0.5000)"
has a "$o" "unrestricted $A all embedding silent: 1 of 2 (0.5000)"
has a "$o" "unrestricted $A all both offered: 0, share a document: 0"
has a "$o" "unrestricted $A all deterministic silent, embedding offered: 1"
has a "$o" "unrestricted $B all prompts: 2"
has a "$o" "unrestricted $B all deterministic silent: 0 of 2 (0.0000)"
has a "$o" "unrestricted $B all embedding silent: 0 of 2 (0.0000)"
has a "$o" "unrestricted $B all both offered: 2, share a document: 2"
has a "$o" "unrestricted none all prompts: 1"
has a "$o" "unrestricted none all deterministic silent: 1 of 1 (1.0000)"
has a "$o" "unrestricted none all embedding silent: n/a, no model"
has a "$o" "unrestricted $A all prompts with a read: 0"
has a "$o" "unrestricted $A all deterministic recall: 0 of 0 (n/a)"
has a "$o" "restricted $B all prompts: 1"
has a "$o" "restricted $A all deterministic silent: 1 of 2 (0.5000)"
has a "$o" "restricted none all prompts: 1"
# No pooled rate: 2 of 5 deterministic, 1 of 4 embedding.
lacks a "$o" "2 of 5"
lacks a "$o" "1 of 4"
lacks a "$o" "0.4000"
lacks a "$o" "0.2500"
if grep -q '^commit: [0-9a-f]\{40\}$' "$o"; then pass "a: names the commit"; else fail "a: names the commit" "no commit line"; fi
if grep -q '^read at: [0-9]\{4\}-[0-9][0-9]-[0-9][0-9]T[0-9:]\{8\}Z$' "$o"; then pass "a: names the time of the reading"; else fail "a: names the time of the reading" "no read at line"; fi

# b: reads against one prompt's window, on one digest.
start b R "$scratch/co"
readcall b R 2026-09-30T09:59:00.000Z /root/c/docs/e.md
prompt b R 2026-09-30T10:00:00.000Z r1
readcall b R 2026-09-30T10:01:00.000Z /root/c/docs/b.md
readcall b R 2026-09-30T10:02:00.000Z /root/c/docs/c.md
readcall b R 2026-09-30T10:03:00.000Z /elsewhere/docs/a.md
readcall b R 2026-09-30T10:04:00.000Z /root/cx/docs/d.md
readcall b R 2026-09-30T10:05:00.000Z /root/c/docs/d.md true
readcall b R 2026-09-30T10:06:00.000Z /root/c/tools/x.sh
readcall b R 2026-09-30T10:06:30.000Z /root/c/.claude/agents/a.md
readcall b R 2026-09-30T10:07:00.000Z /root/c/docs/b.md
prompt b R 2026-09-30T10:30:00.000Z r2
readcall b R 2026-09-30T10:31:00.000Z /root/c/docs/a.md
logline b R 2026-09-30T10:00:01Z r1 true "$A" "$(route 'docs/a.md docs/b.md docs/c.md')" "$(neighbors "$A" 'docs/c.md docs/d.md')"
logline b R 2026-09-30T10:30:01Z r2 false "$A" "$silent" "$(neighbors "$A" docs/e.md)"
run b
o="$scratch/b.out"
has b "$o" "unrestricted $A all prompts: 2"
has b "$o" "unrestricted $A all prompts with a read: 2"
has b "$o" "unrestricted $A all deterministic recall: 2 of 3 (0.6667)"
has b "$o" "unrestricted $A all embedding recall: 1 of 3 (0.3333)"
has b "$o" "unrestricted $A all deterministic first read at rank: 1=0 2=1 3=0 4+=0 none=1"
has b "$o" "unrestricted $A all embedding first read at rank: 1=1 2=0 3=0 4+=0 none=1"
has b "$o" "unrestricted $A all both offered: 1, share a document: 1"

# c: the split by `injected`.
has c "$scratch/a.out" "unrestricted $A injected deterministic silent: 0 of 1 (0.0000)"
has c "$scratch/a.out" "unrestricted $A injected embedding silent: 1 of 1 (1.0000)"
has c "$scratch/a.out" "unrestricted $A not-injected deterministic silent: 1 of 1 (1.0000)"
has c "$scratch/a.out" "unrestricted $A not-injected embedding silent: 0 of 1 (0.0000)"
has c "$o" "unrestricted $A injected deterministic recall: 2 of 2 (1.0000)"
has c "$o" "unrestricted $A injected embedding recall: 1 of 2 (0.5000)"
has c "$o" "unrestricted $A not-injected deterministic recall: 0 of 1 (0.0000)"
has c "$o" "unrestricted $A not-injected embedding recall: 0 of 1 (0.0000)"

# d: the bound closes on the third distinct id, p3, and not on the third
# line, the second line of p2.
mkdir -p "$scratch/d"
cp -r "$scratch/a/t" "$scratch/a/log" "$scratch/d/"
run d --bound 3
has d "$scratch/d.out" "bound: 3"
has d "$scratch/d.out" "bound reached: 2026-09-30T10:20:05Z"

# e: the arguments, and no `jq`.
mkdir -p "$scratch/e1" "$scratch/e2"
cp -r "$scratch/a/t" "$scratch/a/log" "$scratch/e1/"
run e1 --bound x
rc "e: a bound that is not a number exits 2" e1 2
cp -r "$scratch/a/t" "$scratch/a/log" "$scratch/e2/"
run e2 extra
rc "e: an unknown argument exits 2" e2 2
mkdir -p "$scratch/nojq"
HEADWATER_TRANSCRIPT_DIRS="$scratch/a/t" HEADWATER_SHADOW_LOG_DIR="$scratch/a/log" PATH="$scratch/nojq" \
    /bin/sh "$tool" > "$scratch/e3.out" 2> "$scratch/e3.err"
echo $? > "$scratch/e3.rc"
rc "e: no jq exits 3" e3 3

# The same inputs give the same bytes, but for the commit and the time.
grep -v '^commit: \|^read at: ' "$scratch/a.out" > "$scratch/a.first"
run a
grep -v '^commit: \|^read at: ' "$scratch/a.out" > "$scratch/a.second"
if cmp -s "$scratch/a.first" "$scratch/a.second"; then pass "a second run writes the same bytes"; else fail "a second run writes the same bytes" "outputs differ"; fi

printf '%d passed, %d failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
