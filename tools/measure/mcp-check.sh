#!/bin/sh
# Measure what one `check` call of `headwater mcp` costs over this repository.
#
#     sh tools/measure/mcp-check.sh [runs]
#
# Spec 5 states the cost of one `check` call of the MCP server, and until #1346
# nothing recorded the command that took it, so the figure could not be taken
# again. This script is that command. It prints one line per figure, and spec 5
# quotes them.
#
# The server walks the corpus once, before it accepts a message, and it keeps
# no cache. So one session costs a walk plus what each call costs. The script
# times two sessions over standard input, each of which ends at end of input:
#
#   - `initialize` alone, which is the walk and the start of the process;
#   - `initialize` and then one `tools/call` of `check` in the `markdown`
#     format, which is the walk plus one call.
#
# The cost of the call is the second figure less the first. The script also
# times `headwater check --strict` and `headwater check --strict --no-cache`
# over the same repository, because spec 5 compares the call with those two.
# And it times `headwater explain --json` of one document, which walks, parses
# and builds the graph and runs no check. HW-OBL-0080 calls that part Phase A,
# and its discharge condition reads this figure.
#
# Each figure is user plus system CPU time, as `time -p` reports it for the
# process, which the load on a shared host does not change much. One sample is
# the mean of a batch of runs, 3 by default and `HW_MEASURE_BATCH` otherwise.
# Each figure is the median of `runs` samples, 5 by default. The engine is the
# `dev-release` build of this checkout, and `HW_MEASURE_ENGINE` names another
# binary. The script writes only under a fresh temporary directory, and it
# deletes that directory when it ends.
set -eu

repo=$(cd "$(dirname "$0")/../.." && pwd)
runs=${1:-5}
batch=${HW_MEASURE_BATCH:-3}
engine=${HW_MEASURE_ENGINE:-$repo/engine/target/dev-release/headwater}
if [ ! -x "$engine" ]; then
    printf 'mcp-check: no engine at %s. Build it with:\n  sh tools/hw-cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked\n' "$engine" >&2
    exit 2
fi

scratch=$(mktemp -d "${TMPDIR:-/tmp}/hw-mcp-check.XXXXXX")
init='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}'
ready='{"jsonrpc":"2.0","method":"notifications/initialized"}'
call='{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"check","arguments":{"format":"markdown"}}}'
printf '%s\n%s\n' "$init" "$ready" > "$scratch/walk.in"
printf '%s\n%s\n%s\n' "$init" "$ready" "$call" > "$scratch/call.in"

# The CPU seconds of one run of the command, as user plus system time, read
# from standard input `$1`. `time -p` counts in hundredths of a second, so one
# sample times a batch of runs in one shell and divides.
cpu() {
    input=$1
    shift
    /usr/bin/time -p sh -c 'k=$1; in=$2; shift 2; i=0; while [ "$i" -lt "$k" ]; do "$@" < "$in" > /dev/null 2>&1 || true; i=$((i + 1)); done' batch "$batch" "$input" "$@" 2> "$scratch/time" || true
    awk -v k="$batch" '$1 == "user" { u = $2 } $1 == "sys" { s = $2 } END { printf "%.4f\n", (u + s) / k }' "$scratch/time"
}

# The median of `runs` samples, in milliseconds.
median() {
    : > "$scratch/samples"
    i=0
    while [ "$i" -lt "$runs" ]; do
        cpu "$@" >> "$scratch/samples"
        i=$((i + 1))
    done
    sort -n "$scratch/samples" | awk '{ v[NR] = $1 } END {
        m = (NR % 2) ? v[(NR + 1) / 2] : (v[NR / 2] + v[NR / 2 + 1]) / 2
        printf "%.0f\n", m * 1000
    }'
}

# One session of each, outside the batch loop, so that a server that failed
# fast does not read as a fast result. The call's answer must carry id 2 and
# a `result`, and the script refuses to print a figure without one.
"$engine" mcp --root "$repo" < "$scratch/call.in" > "$scratch/call.out" 2> "$scratch/call.err" || true
if ! grep -q '"id":2,"result"' "$scratch/call.out"; then
    printf 'mcp-check: the check call returned no result. Standard error:\n' >&2
    cat "$scratch/call.err" >&2
    rm -rf "$scratch"
    exit 1
fi
"$engine" check --strict --root "$repo" > "$scratch/warm.out" 2> "$scratch/warm.err" || true
documents=$(awk '$2 == "typed" && NF == 2 { print $1; exit }' "$scratch/warm.out")

walk_ms=$(median "$scratch/walk.in" "$engine" mcp --root "$repo")
call_ms=$(median "$scratch/call.in" "$engine" mcp --root "$repo")
warm_ms=$(median /dev/null "$engine" check --strict --root "$repo")
cold_ms=$(median /dev/null "$engine" check --strict --no-cache --root "$repo")
phase_ms=$(median /dev/null "$engine" explain --json docs/spec/06-engine-architecture.md --root "$repo")

printf 'engine            %s\n' "$engine"
printf 'corpus            %s (%s typed documents)\n' "$repo" "$documents"
printf 'runs              median of %s samples, each the mean of %s runs, user+system CPU\n' "$runs" "$batch"
printf 'mcp walk          %s ms (initialize alone)\n' "$walk_ms"
printf 'mcp check         %s ms (initialize and one check call)\n' "$call_ms"
printf 'mcp call          %s ms (the check call alone, the difference)\n' "$((call_ms - walk_ms))"
printf 'check --strict    %s ms (warm cache)\n' "$warm_ms"
printf 'check --no-cache  %s ms\n' "$cold_ms"
printf 'explain --json    %s ms (Phase A: walk, parse and graph, no check)\n' "$phase_ms"

rm -rf "$scratch"
