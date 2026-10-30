#!/bin/sh
# Generic oracle: a named test passes, and at least one case ran (#1874).
# Rows 2 and 3 of the grid in HW-DR-0112.
#
#     sh tools/probe/oracles/test-passes.sh [--check-clean] [--ignored] <crate> <test-target> [<filter>] [-- <document>...]
#
# It runs the test target <test-target> of the crate <crate> in the engine
# workspace, with <filter> as cargo's case filter when it is given and with
# `-- --ignored` when `--ignored` is given. It passes when cargo exits 0 and a
# `test result: ok. <n> passed` line has <n> above 0. A filter that matches no
# case passes cargo and fails here, because a probe that grades nothing must
# not read as a pass.
#
# `--check-clean` adds the condition of check-clean.sh. Each <document> after
# `--` is a path, and with one or more of them the condition counts only the
# strict errors on those documents, as the other tree scripts do, because a
# sealed tree holds errors that the seal made in documents it kept (#1892).
# Row 3 names the documents that govern the files a refactor moves, because a
# rename can break a `governs` path (#1874, slice 4b). With no document, the
# condition reads the whole tree. Through `--args`, `--` and each document
# are lines of the held file, after the other arguments.
#
# A hidden test of one probe goes under `holds` with this script, and its
# header names the probe, as `engine/crates/check/tests/probe_oracle_future_date.rs`
# does, so the seal removes it from every workspace. Mark such a case
# `#[ignore]` when it fails on the pin by design, and pass `--ignored`.
#
# Cargo is `sh tools/hw-cargo` with the `dev-release` profile, as the
# campaign runs it. `HW_ORACLE_CARGO` replaces that command, which is how
# `tools/probe/oracle-fixtures.sh` runs this script against a stub.

set -u
. "${0%/*}/lib.sh"
usage="test-passes.sh [--check-clean] [--ignored] <crate> <test-target> [<filter>] [-- <document>...]"
ignored=""
if [ "${1:-}" = --ignored ]; then
    ignored=--ignored
    shift
fi
crate="" target="" filter="" positional=0
while [ $# -gt 0 ]; do
    case $1 in
        --)
            shift
            [ $# -gt 0 ] || hw_oracle_usage "$usage"
            hw_oracle_subjects="$*"
            break
            ;;
    esac
    positional=$((positional + 1))
    case $positional in
        1) crate=$1 ;;
        2) target=$1 ;;
        3) filter=$1 ;;
        *) hw_oracle_usage "$usage" ;;
    esac
    shift
done
[ "$positional" -ge 2 ] || hw_oracle_usage "$usage"
cargo=${HW_ORACLE_CARGO:-sh tools/hw-cargo}
hw_oracle_tmp
# Added on 2026-10-30 (#1874, slice 4b). Every file under engine/crates is
# marked newer before cargo runs, so each workspace crate builds from this
# tree. `tools/hw-cargo` pools its target directories by slot, and it marks a
# tree newer only where the tree is a git checkout that did not build last in
# the slot (its lines 175-184). A grading copy holds no `.git`, it is made
# with `cp -a`, which keeps each mtime, and the fixture builds every case at
# one path. So a wrong change that leaves a file at its pin mtime would build
# on the unit that the reference change compiled, and pass. The dependency
# graph the pool shares is left alone.
[ -d engine/crates ] && find engine/crates -name target -prune -o -type f -exec touch {} +
status=0
# shellcheck disable=SC2086
$cargo test --profile dev-release -p "$crate" --test "$target" --manifest-path engine/Cargo.toml --locked -- $filter $ignored > "$hw_oracle_scratch/test.out" 2> "$hw_oracle_scratch/test.err" || status=$?
[ "$status" -eq 0 ] || hw_oracle_fail "\`$target\` of $crate exits $status"
ran=$(sed -n 's/^test result: ok\. \([0-9][0-9]*\) passed.*/\1/p' "$hw_oracle_scratch/test.out" | awk '{ n += $1 } END { print n + 0 }')
[ "$ran" -gt 0 ] || hw_oracle_fail "\`$target\` of $crate ran no case${filter:+ that matches $filter}"
hw_oracle_pass_clean "\`$target\` of $crate passed $ran case(s)"
