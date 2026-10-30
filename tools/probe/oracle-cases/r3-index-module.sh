# The cases of the `r3-index-module` oracle, for the probe
# HW-PROBE-a-session-renames-the-identifier-index-of-the-graph-crate-and-its-callers
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
    mv $c/graph/src/index.rs $c/graph/src/identifiers.rs
}

# Every caller names the new module, in its crate and outside it.
r3_code() {
    sed -i 's/^pub mod index;$/pub mod identifiers;/; s/^pub use index::/pub use identifiers::/; s/\bindex::Reported/identifiers::Reported/g' $c/graph/src/lib.rs &&
        sed -i 's/crate::index::/crate::identifiers::/g' $c/graph/src/links.rs $c/graph/src/edges.rs $c/graph/src/anchors.rs &&
        sed -i 's/headwater_graph::index::/headwater_graph::identifiers::/g' \
            $c/scaffold/tests/opening_state.rs $c/sweep/tests/fixtures.rs $c/scaffold/src/lib.rs \
            $c/scaffold/tests/fixtures.rs $c/import/tests/fixtures.rs $c/import/tests/chain/mod.rs \
            $c/scaffold/tests/telemetry.rs $c/scaffold/tests/pipeline.rs $c/cli/src/main.rs \
            $c/check/src/scope.rs $c/import/src/lib.rs $c/check/src/claim.rs $c/check/src/identity.rs \
            $c/check/src/duplicate.rs $c/check/src/lib.rs &&
        sed -i 's/`index::Defect`/`identifiers::Defect`/' $c/check/tests/fixtures.rs
}

# Each record that governs the module names the new path.
r3_records() {
    sed -i 's#engine/crates/graph/src/index\.rs#engine/crates/graph/src/identifiers.rs#g' \
        docs/decisions/0068-the-front-matter-key-that-carries-a-minted-identifier-is-id.md
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

# Every caller follows but one, in another crate: cli/src/main.rs.
left_caller() {
    reference && sed -i 's/headwater_graph::identifiers::Index::build/headwater_graph::index::Index::build/' $c/cli/src/main.rs && grep -q 'headwater_graph::index::Index::build' $c/cli/src/main.rs
}

# The code moves, and the records still name the old path, so they govern
# nothing and `headwater check` reports each one.
stale_records() {
    r3_move && r3_code
}
more_wrong="left_caller stale_records"
