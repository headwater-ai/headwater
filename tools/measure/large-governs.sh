#!/bin/sh
# Measure what one `governs` glob over thousands of files costs.
#
#     sh tools/measure/large-governs.sh [files] [runs]
#
# #1260 asked what `headwater check` and `headwater explain --json` cost when
# one document governs a glob that reaches thousands of files, and nothing had
# measured it. This script is the recorded command. It prints one line per
# figure, and spec 5 and the `headwater explain` interface page quote them.
#
# It builds a scratch corpus in a fresh temporary directory: the taxonomy lock,
# the consumer declaration and the overlay of this repository, one interface
# document whose `governs` is `src/big/**`, and `files` small files under
# `src/big/` (default 5000). It runs `headwater check --fix` once, so that the
# edge carries a `verified_revision` the way an adopter's does, and once more
# with `--strict` to warm the cache. Then it takes the median of `runs` warm
# runs (default 10) of each of these:
#
#   - `headwater check --strict` over the scratch corpus;
#   - `headwater explain --json` of the one document;
#   - the same with `--paths-at-most 20`, the bound the edit hook passes;
#   - `hw_governed_by_document` of `.claude/hooks/lib.sh` on that document,
#     which is the edit-time hook's reader of `reach.members[].paths`;
#   - `headwater check --strict` over this repository, as the base figure.
#
# Each figure is user plus system CPU time, as `time -p` reports it for the
# process and every child it waited for. CPU time is the measure because the
# load on a shared host does not change it, which is how spec 5 measured
# `headwater route`. One sample is the mean of a batch of runs, 10 by default
# and `HW_MEASURE_BATCH` otherwise, because `time -p` counts hundredths of a
# second. It also prints the byte size of each explain document. The hook
# reads its document once for each count it needs, so the size of the bounded
# document is what the hook's cost follows (#1346).
#
# The engine is the `dev-release` build of this checkout, and never `--release`
# (DEVELOPING.md says why). `HW_MEASURE_ENGINE` names another binary. The
# script deletes the scratch corpus when it ends, unless `HW_MEASURE_KEEP` is
# set, and then it prints the path.
set -eu

repo=$(cd "$(dirname "$0")/../.." && pwd)
files=${1:-5000}
runs=${2:-10}
engine=${HW_MEASURE_ENGINE:-$repo/engine/target/dev-release/headwater}
if [ ! -x "$engine" ]; then
    printf 'large-governs: no engine at %s. Build it with:\n  sh tools/hw-cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked\n' "$engine" >&2
    exit 2
fi

tree=$(mktemp -d "${TMPDIR:-/tmp}/hw-large-governs.XXXXXX")
mkdir -p "$tree/.headwater" "$tree/docs/interfaces" "$tree/src/big"
for name in taxonomy.lock taxonomy.yml overlay.yml; do
    cp "$repo/.headwater/$name" "$tree/.headwater/$name"
done
# One awk process writes every file, because a shell loop of 5,000 redirects
# costs longer than the measurement it prepares.
awk -v n="$files" -v dir="$tree/src/big" 'BEGIN {
    for (i = 0; i < n; i++) {
        path = sprintf("%s/f%05d.rs", dir, i)
        printf "// file %d\n", i > path
        close(path)
    }
}'
document=docs/interfaces/headwater-large.md
cat > "$tree/$document" <<EOF
---
id: HW-IFACE-headwater-large
status: current
status_since: 2026-09-29
summary: "An anchor that holds one glob over many files."
last_verified: 2026-09-29
title: "headwater large"
relations:
  governs:
    - "src/big/**"
---

# headwater large

## Synopsis

    headwater large
EOF

scratch=$tree/.measure
mkdir -p "$scratch"
"$engine" check --fix --root "$tree" > "$scratch/fix.out" 2> "$scratch/fix.err" || true
"$engine" check --strict --root "$tree" > "$scratch/warm.out" 2> "$scratch/warm.err" || true
"$engine" check --strict --root "$repo" > "$scratch/warm-repo.out" 2> "$scratch/warm-repo.err" || true

# The CPU seconds of one run of the command, as user plus system time. `time
# -p` counts in hundredths of a second, which reads a 5 ms run as 0 or 10. So
# one sample times a batch of `batch` runs in one shell and divides, and the
# shell that loops adds its own few forks to the batch.
batch=${HW_MEASURE_BATCH:-10}
cpu() {
    /usr/bin/time -p sh -c 'k=$1; shift; i=0; while [ "$i" -lt "$k" ]; do "$@" > /dev/null 2>&1 || true; i=$((i + 1)); done' batch "$batch" "$@" 2> "$scratch/time" || true
    awk -v k="$batch" '$1 == "user" { u = $2 } $1 == "sys" { s = $2 } END { printf "%.4f\n", (u + s) / k }' "$scratch/time"
}

# The median of `runs` runs of the command, in milliseconds.
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

cat > "$scratch/hook.sh" <<EOF
HEADWATER_HOOK_ROOT='$tree'
hw_engine_pin='$engine'
. '$repo/.claude/hooks/lib.sh'
hw_governed_by_document '$document' > '$scratch/hook.out'
EOF

"$engine" explain --json "$document" --root "$tree" > "$scratch/explain.json"
bytes=$(wc -c < "$scratch/explain.json" | tr -d ' ')
"$engine" explain --json --paths-at-most 20 "$document" --root "$tree" > "$scratch/bounded.json"
bounded_bytes=$(wc -c < "$scratch/bounded.json" | tr -d ' ')
documents=$(awk '$2 == "typed" && NF == 2 { print $1; exit }' "$scratch/warm-repo.out")

# The batch loop discards each run's exit status, so a command that failed
# fast would read as a fast result. One run of each, outside the loop, states
# the status beside the figure. A scratch `check --strict` exits 1, because the
# scratch corpus carries setup findings, and it still runs every check.
status() {
    "$@" > /dev/null 2>&1 && printf '0' || printf '%s' "$?"
}
check_status=$(status "$engine" check --strict --root "$tree")
explain_status=$(status "$engine" explain --json "$document" --root "$tree")
bounded_status=$(status "$engine" explain --json --paths-at-most 20 "$document" --root "$tree")
hook_status=$(status sh "$scratch/hook.sh")
repo_status=$(status "$engine" check --strict --root "$repo")

check_ms=$(median "$engine" check --strict --root "$tree")
explain_ms=$(median "$engine" explain --json "$document" --root "$tree")
bounded_ms=$(median "$engine" explain --json --paths-at-most 20 "$document" --root "$tree")
hook_ms=$(median sh "$scratch/hook.sh")
repo_ms=$(median "$engine" check --strict --root "$repo")

printf 'engine            %s\n' "$engine"
printf 'files             %s under src/big/, one governs edge onto src/big/**\n' "$files"
printf 'runs              median of %s warm samples, each the mean of %s runs, user+system CPU\n' "$runs" "$batch"
printf 'check --strict    %s ms (generated tree), exit %s\n' "$check_ms" "$check_status"
printf 'explain --json    %s ms, %s bytes, exit %s\n' "$explain_ms" "$bytes" "$explain_status"
printf 'explain bounded   %s ms, %s bytes, exit %s (--paths-at-most 20)\n' "$bounded_ms" "$bounded_bytes" "$bounded_status"
printf 'hook              %s ms (hw_governed_by_document), exit %s\n' "$hook_ms" "$hook_status"
printf 'check --strict    %s ms (this repository, %s typed documents), exit %s\n' "$repo_ms" "$documents" "$repo_status"
printf 'hook output       %s\n' "$(head -n 2 "$scratch/hook.out" | tr '\n' ' ')"

if [ -n "${HW_MEASURE_KEEP:-}" ]; then
    printf 'tree              %s\n' "$tree"
else
    rm -rf "$tree"
fi
