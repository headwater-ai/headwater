# The cases of the `r3-lifecycle-module` oracle, for the probe
# HW-PROBE-a-session-renames-the-module-of-the-check-crate-that-reads-the-state-a-document-stands-in
# (#1874, slice 4b).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree. This file states the
# answer, so it names its probe here and the seal removes it from every
# workspace. The oracle builds `headwater-cli` in every case, so it runs
# under `--cargo` alone.

needs_cargo=yes

c=engine/crates

# The module moves, with `mv`, which keeps the time of the file.
r3_move() {
    mv $c/check/src/lifecycle_state.rs $c/check/src/standing.rs
}

# Every caller names the new module, in its crate and outside it.
r3_code() {
    sed -i 's/^pub mod lifecycle_state;$/pub mod standing;/; s/\[`lifecycle_state`\]/[`standing`]/g' $c/check/src/lib.rs &&
        sed -i 's/`lifecycle_state\.rs`/`standing.rs`/g' $c/check/src/retention.rs &&
        sed -i -e 's/\blifecycle_state::/standing::/g' \
            -e 's/crate::lifecycle_state`/crate::standing`/g' \
            -e 's/headwater_check::lifecycle_state\([]`]\)/headwater_check::standing\1/g' \
            $c/check/src/lib.rs $c/check/src/acceptance_state.rs $c/check/src/dependency.rs \
            $c/check/src/initial_dependency.rs $c/check/src/invalid_pair.rs $c/check/src/reciprocity.rs \
            $c/check/src/retention.rs $c/check/src/shape.rs $c/check/src/state_not_set_by_edge.rs \
            $c/check/src/state_set_twice.rs $c/check/src/transition.rs \
            $c/check/tests/admitted_state.rs $c/check/tests/initial_dependency.rs \
            $c/check/tests/terminal_dependency.rs \
            $c/generate/src/derived.rs $c/generate/src/lib.rs $c/generate/src/probe_result.rs \
            $c/generate/tests/lifecycle_state_admission.rs $c/lock/src/lib.rs $c/scaffold/src/lib.rs
}

# Each record that governs the module names the new path.
r3_records() {
    sed -i 's#engine/crates/check/src/lifecycle_state\.rs#engine/crates/check/src/standing.rs#g' \
        docs/decisions/0026-q26-whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime.md
}

# The sound change: the module moves, every caller follows it, and every
# record that governs it names the new path.
reference() {
    r3_move && r3_code && r3_records
}

# The file and the records move, and no caller does, so the build fails.
# This case runs straight after the reference, and it leaves every file
# under `engine/crates` as old as the grading tree. So a build that trusts
# the times of those files serves the reference's build to it, and the case
# passes. `test-passes.sh` marks the files newer before it builds, and this
# case is what fails without that (#1874, slice 4b).
wrong() {
    r3_move && r3_records
}

# Every caller follows but one, in another crate: generate/src/probe_result.rs.
left_caller() {
    reference && sed -i 's/^use headwater_check::standing::/use headwater_check::lifecycle_state::/' $c/generate/src/probe_result.rs && grep -q '^use headwater_check::lifecycle_state::' $c/generate/src/probe_result.rs
}

# The code moves, and the records still name the old path, so they govern
# nothing and `headwater check` reports each one.
stale_records() {
    r3_move && r3_code
}
more_wrong="left_caller stale_records"
