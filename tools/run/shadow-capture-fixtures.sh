#!/bin/sh
# What holds `tools/run/shadow-capture.sh`.
#
# The tool exists so that a capture rate that looks acceptable cannot hide
# losses that sit in one cause, which is what the owner's "a test that the
# losses are random" on #917 protects against. So the cases are planted
# transcripts and a planted log whose right answer was worked by hand:
#
#   a  thirty prompts in three sessions of ten, four misses spread over two
#      sessions: neither test rejects.
#   b  the same four misses, all in one session and in a row at its end: both
#      tests reject. A tool whose test is a stub passes `a` and fails here.
#      (Ten prompts and two misses, the size first proposed, cannot reject at
#      this threshold by either test: chi-square 3.75 on two degrees of
#      freedom is p 0.153, and the runs z is -1.22.)
#   c  one prompt logged twice: J counts it once. A raw line count fails here.
#   d  the window: a record at 23:59:59Z the day before and at 00:00:00Z the
#      day after are out, and the first and last instants of the day are in.
#   e  the population: a sidechain, a non-human origin, a session that started
#      in a checkout with no engine and one whose checkout is gone are out.
#   f  a log line that does not parse does not end the read of its file.
#   g  a miss whose text the session logged under another prompt id.
#
# Run it from anywhere:
#     sh tools/run/shadow-capture-fixtures.sh
#
# It needs `jq`, exits 3 without it so CI can skip with a warning, and writes
# only under a temporary directory.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/run/shadow-capture.sh"

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

# A checkout that holds an engine, one that does not, and one that is gone.
mkdir -p "$scratch/co/engine/target/dev-release" "$scratch/bare"
printf '#!/bin/sh\n' > "$scratch/co/engine/target/dev-release/headwater"
chmod +x "$scratch/co/engine/target/dev-release/headwater"

# start <case> <session> <start dir>: the session's first record, which
# carries the directory the session started in.
start() {
    mkdir -p "$scratch/$1/t" "$scratch/$1/log"
    printf '{"type":"system","sessionId":"%s","cwd":"%s"}\n' "$2" "$3" >> "$scratch/$1/t/$2.jsonl"
}
# prompt <case> <session> <timestamp> <id> <text> [origin] [sidechain]
prompt() {
    printf '{"type":"user","isSidechain":%s,"origin":{"kind":"%s"},"promptId":"%s","sessionId":"%s","timestamp":"%s","cwd":"/elsewhere","message":{"role":"user","content":"%s"}}\n' \
        "${7:-false}" "${6:-human}" "$4" "$2" "$3" "$5" >> "$scratch/$1/t/$2.jsonl"
}
# logline <case> <session> <id> <text>
logline() {
    printf '{"at":"2026-09-30T12:00:00Z","session":"%s","prompt_id":"%s","injected":false,"skip":null,"route":null,"task":"%s"}\n' \
        "$2" "$3" "$4" >> "$scratch/$1/log/$2.jsonl"
}
# run <case> [<arguments>...]: the tool over that case's transcripts and log,
# for 2026-09-30 unless arguments are given.
run() {
    _c=$1
    shift
    [ $# -gt 0 ] || set -- 2026-09-30
    HEADWATER_TRANSCRIPT_DIRS="$scratch/$_c/t" HEADWATER_SHADOW_LOG_DIR="$scratch/$_c/log" \
        sh "$tool" "$@" > "$scratch/$_c.out" 2> "$scratch/$_c.err"
    echo $? > "$scratch/$_c.rc"
}

# Ten prompts a session, a minute apart; `miss` lists the positions not logged.
session() {
    _case=$1 _s=$2 _miss=$3
    start "$_case" "$_s" "$scratch/co"
    for _i in 0 1 2 3 4 5 6 7 8 9; do
        prompt "$_case" "$_s" "2026-09-30T01:0$_i:00.000Z" "$_s-$_i" "p $_s $_i"
        case " $_miss " in
            *" $_i "*) ;;
            *) logline "$_case" "$_s" "$_s-$_i" "p $_s $_i" ;;
        esac
    done
}

# a: misses at positions 1 and 5 of sessions A and B, none in C.
session a A "1 5"
session a B "1 5"
session a C ""
run a
has a "$scratch/a.out" "person prompts (N): 30"
has a "$scratch/a.out" "joined distinct prompts (J): 26"
has a "$scratch/a.out" "sessions test: chi-square 2.3077, df 2, p 0.3154, not rejected"
has a "$scratch/a.out" "runs test: runs 11, expected 9.4000, variance 1.5644, z 1.2792, p 0.8996 (lower tail), not rejected"
has a "$scratch/a.out" "randomness: not rejected"

# b: four misses in a row at the end of session A.
session b A "6 7 8 9"
session b B ""
session b C ""
run b
has b "$scratch/b.out" "joined distinct prompts (J): 26"
has b "$scratch/b.out" "session with misses: A 4 of 10 missed"
has b "$scratch/b.out" "sessions test: chi-square 9.2308, df 2, p 0.0099, rejected"
has b "$scratch/b.out" "runs test: runs 4, expected 7.8000, variance 2.0267, z -2.6693, p 0.0038 (lower tail), rejected"
has b "$scratch/b.out" "randomness: rejected"

# c: three prompts, one logged twice, one not at all.
start c S "$scratch/co"
prompt c S 2026-09-30T02:00:00Z c1 one
prompt c S 2026-09-30T02:01:00Z c2 two
prompt c S 2026-09-30T02:02:00Z c3 three
logline c S c1 one
logline c S c1 one
logline c S c2 two
run c
has c "$scratch/c.out" "person prompts (N): 3"
has c "$scratch/c.out" "joined distinct prompts (J): 2"
has c "$scratch/c.out" "duplicate lines for joined prompts: 1"
has c "$scratch/c.out" "capture rate (J/N): 0.6667, Wilson 95% interval [0.2077, 0.9385]"

# d: the window. Two records outside it, two at its edges, all logged.
start d S "$scratch/co"
prompt d S 2026-09-29T23:59:59Z d1 before
prompt d S 2026-09-30T00:00:00Z d2 first
prompt d S 2026-09-30T23:59:59.999Z d3 last
prompt d S 2026-10-01T00:00:00Z d4 after
for _i in d1 d2 d3 d4; do logline d S "$_i" x; done
run d
has d "$scratch/d.out" "person prompts (N): 2"
has d "$scratch/d.out" "joined distinct prompts (J): 2"

# e: one prompt counted, and four that are not in the population.
start e S "$scratch/co"
prompt e S 2026-09-30T03:00:00Z e1 counted
prompt e S 2026-09-30T03:01:00Z e2 sidechain human true
prompt e S 2026-09-30T03:02:00Z e3 notification task-notification
start e T "$scratch/bare"
prompt e T 2026-09-30T03:03:00Z e4 "no engine"
start e U "$scratch/gone"
prompt e U 2026-09-30T03:04:00Z e5 gone
run e
has e "$scratch/e.out" "person prompts (N): 1"
has e "$scratch/e.out" "missed prompts: 1"
has e "$scratch/e.out" "excluded, checkout without an engine: 1"
has e "$scratch/e.out" "excluded, checkout gone: 1"

# f: a torn line first in the file, then the line that joins.
start f S "$scratch/co"
prompt f S 2026-09-30T04:00:00Z f1 one
mkdir -p "$scratch/f/log"
printf '{"at":"2026-09-30T04:00:0\n' > "$scratch/f/log/S.jsonl"
logline f S f1 one
run f
has f "$scratch/f.out" "joined distinct prompts (J): 1"

# g: the prompt's text was logged under the id of what it queued behind.
start g S "$scratch/co"
prompt g S 2026-09-30T05:00:00Z g1 "which PRs"
logline g S notification-id "which PRs"
run g
has g "$scratch/g.out" "missed prompts: 1"
has g "$scratch/g.out" "missed prompts whose text the session logged under another id: 1"

# The arguments, over case a's inputs so that only the argument is wrong: an
# empty date, and a date that does not exist.
rc() {
    if [ "$(cat "$scratch/$2.rc")" = "$3" ]; then pass "$1"; else fail "$1" "exit $(cat "$scratch/$2.rc"), wanted $3"; fi
}
mkdir -p "$scratch/z" "$scratch/y"
cp -r "$scratch/a/t" "$scratch/a/log" "$scratch/z/"
cp -r "$scratch/a/t" "$scratch/a/log" "$scratch/y/"
run z ""
rc "an empty date exits 2" z 2
run y 2026-02-30
rc "a date that does not exist exits 2" y 2

# The same inputs give the same bytes.
cp "$scratch/b.out" "$scratch/b.first"
run b
if cmp -s "$scratch/b.first" "$scratch/b.out"; then pass "a second run writes the same bytes"; else fail "a second run writes the same bytes" "outputs differ"; fi

printf '%d passed, %d failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
