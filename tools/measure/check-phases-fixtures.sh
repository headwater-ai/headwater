#!/bin/sh
# Hold `check-phases.sh` to its refusals.
#
#     sh tools/measure/check-phases-fixtures.sh
#
# `check-phases.sh` is worth its figures only if it refuses to print them when
# the stages do not account for the run (#1450). This script runs it five ways
# and fails when any one of them ends with the wrong exit status:
#
#   - over the engine the commit gate runs, which carries no `phase-times`
#     feature, and it must refuse;
#   - over the measurement build as it is, and it must print;
#   - over a stand-in that drops the `run.edges` line, which is a stage that the
#     build did not mark, and it must refuse;
#   - over a stand-in that adds 50 ms to the `start` stage, so that the stages no
#     longer sum to the run, and it must refuse;
#   - over a stand-in that moves 50 ms from `run.edges` into `load`, so that the
#     sum holds and the part past Phase A does not, and it must refuse.
#
# The stand-in runs the measurement build and rewrites its `phase` lines. It
# needs both engines built, and it names the command for each one it misses.
# No hook and no CI job runs it, because CI builds no measurement engine.
set -eu

repo=$(cd "$(dirname "$0")/../.." && pwd)
phase=$repo/engine/target/phase/headwater
plain=$repo/engine/target/dev-release/headwater
for engine in "$phase" "$plain"; do
    if [ ! -x "$engine" ]; then
        printf 'check-phases-fixtures: no engine at %s. Build both with:\n  sh tools/hw-cargo build --profile phase --features phase-times -p headwater-cli --manifest-path engine/Cargo.toml --locked\n  sh tools/hw-cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked\n' "$engine" >&2
        exit 2
    fi
done

scratch=$(mktemp -d "${TMPDIR:-/tmp}/hw-check-phases-fixtures.XXXXXX")
cat > "$scratch/stand-in" <<EOF
#!/bin/sh
"$phase" "\$@" 2> "$scratch/stand-in.err"
code=\$?
awk -v mode="\$HW_SKEW" '
    \$1 == "phase" && mode == "drop-edges" && \$2 == "run.edges" { next }
    \$1 == "phase" && mode == "pad-start" && \$2 == "start" { \$3 += 50000 }
    \$1 == "phase" && mode == "shift-load" && \$2 == "load" { \$3 += 50000 }
    \$1 == "phase" && mode == "shift-load" && \$2 == "run.edges" { \$3 -= 50000; if (\$3 < 0) \$3 = 0 }
    { print }' "$scratch/stand-in.err" >&2
exit \$code
EOF
chmod +x "$scratch/stand-in"

failed=0
# One case: a name, the exit status it must end with, and the engine.
case_of() {
    name=$1
    want=$2
    engine=$3
    got=0
    HW_MEASURE_ENGINE=$engine sh "$repo/tools/measure/check-phases.sh" 3 > "$scratch/out" 2> "$scratch/err" || got=$?
    if [ "$got" -eq "$want" ]; then
        printf 'ok    %-12s exit %s\n' "$name" "$got"
    else
        printf 'FAIL  %-12s exit %s, expected %s\n' "$name" "$got" "$want"
        sed 's/^/      /' "$scratch/err"
        failed=1
    fi
}

case_of no-feature 1 "$plain"
export HW_SKEW=none
case_of as-built 0 "$scratch/stand-in"
export HW_SKEW=drop-edges
case_of drop-edges 1 "$scratch/stand-in"
export HW_SKEW=pad-start
case_of pad-start 1 "$scratch/stand-in"
export HW_SKEW=shift-load
case_of shift-load 1 "$scratch/stand-in"

rm -rf "$scratch"
exit "$failed"
