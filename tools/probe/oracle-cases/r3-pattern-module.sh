# The cases of the `r3-pattern-module` oracle, for the probe
# HW-PROBE-a-session-renames-the-pattern-module-of-the-meta-crate-and-its-callers
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
    mv $c/meta/src/pattern.rs $c/meta/src/glob.rs
}

# Every caller names the new module, in its crate and outside it.
r3_code() {
    sed -i 's/^pub mod pattern;$/pub mod glob;/; s/^pub use pattern::Pattern;$/pub use glob::Pattern;/' $c/meta/src/lib.rs &&
        sed -i 's/crate::pattern::Pattern/crate::glob::Pattern/' $c/meta/src/identifier.rs &&
        sed -i 's/headwater_meta::pattern\b/headwater_meta::glob/g' \
            $c/graph/src/scope.rs $c/graph/src/edges.rs $c/graph/src/anchors.rs \
            $c/census/src/lib.rs $c/census/src/shelves.rs $c/census/src/outside.rs $c/census/src/walk.rs \
            $c/cli/tests/classify.rs
}

# Each record that governs the module names the new path.
r3_records() {
    sed -i 's#engine/crates/meta/src/pattern\.rs#engine/crates/meta/src/glob.rs#g' \
        docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md
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

# Every caller follows but one, in another crate: census/src/walk.rs.
left_caller() {
    reference && sed -i 's/^use headwater_meta::glob::Pattern;$/use headwater_meta::pattern::Pattern;/' $c/census/src/walk.rs && grep -q '^use headwater_meta::pattern::Pattern;$' $c/census/src/walk.rs
}

# The code moves, and the records still name the old path, so they govern
# nothing and `headwater check` reports each one.
stale_records() {
    r3_move && r3_code
}
more_wrong="left_caller stale_records"
