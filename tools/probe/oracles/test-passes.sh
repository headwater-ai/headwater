#!/bin/sh
# Generic oracle: a named test passes, and at least one case ran (#1874).
# Rows 2 and 3 of the grid in HW-DR-0112.
#
#     sh tools/probe/oracles/test-passes.sh [--check-clean] [--ignored] <crate> <test-target> [<filter>]
#
# It runs the test target <test-target> of the crate <crate> in the engine
# workspace, with <filter> as cargo's case filter when it is given and with
# `-- --ignored` when `--ignored` is given. It passes when cargo exits 0 and a
# `test result: ok. <n> passed` line has <n> above 0. A filter that matches no
# case passes cargo and fails here, because a probe that grades nothing must
# not read as a pass.
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
ignored=""
if [ "${1:-}" = --ignored ]; then
    ignored=--ignored
    shift
fi
[ $# -eq 2 ] || [ $# -eq 3 ] || hw_oracle_usage "test-passes.sh [--check-clean] [--ignored] <crate> <test-target> [<filter>]"
crate=$1 target=$2 filter=${3:-}
cargo=${HW_ORACLE_CARGO:-sh tools/hw-cargo}
hw_oracle_tmp
status=0
# shellcheck disable=SC2086
$cargo test --profile dev-release -p "$crate" --test "$target" --manifest-path engine/Cargo.toml --locked -- $filter $ignored > "$hw_oracle_scratch/test.out" 2> "$hw_oracle_scratch/test.err" || status=$?
[ "$status" -eq 0 ] || hw_oracle_fail "\`$target\` of $crate exits $status"
ran=$(sed -n 's/^test result: ok\. \([0-9][0-9]*\) passed.*/\1/p' "$hw_oracle_scratch/test.out" | awk '{ n += $1 } END { print n + 0 }')
[ "$ran" -gt 0 ] || hw_oracle_fail "\`$target\` of $crate ran no case${filter:+ that matches $filter}"
hw_oracle_pass_clean "\`$target\` of $crate passed $ran case(s)"
