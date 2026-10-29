# The cases of the `future-date` oracle, for the probe
# HW-PROBE-a-rule-that-reads-the-date-takes-it-from-the-run-and-ships-with-the-fixture-that-fails-it
# (#1835, #1874).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree. This file states a
# wrong answer, so it names its probe here and the seal removes it from every
# workspace. The oracle builds the check crate in every case, so it runs
# under `--cargo` alone.

needs_cargo=yes

# The sound patch is a new rule, its registration and a fixture, about a
# hundred lines of Rust. It ran on 2026-10-23 and passed, and the probe's Key
# records it with the three patches that failed.
reference_recorded="the probe's Key, \"The candidate patches, checked\", 2026-10-23"

# A fixture document dated after the run, and no rule: the half of the task
# that needs no code.
wrong() {
    mkdir -p engine/crates/check/fixtures/check/evaluations
    printf -- '---\nid: HW-EVAL-a-date-after-the-run\nstatus: current\nstatus_since: 2031-01-01\nsummary: A record dated after the run.\nlast_verified: 2031-01-01\ntitle: "A record dated after the run"\n---\n\n# A record dated after the run\n' > engine/crates/check/fixtures/check/evaluations/a-date-after-the-run.md
}
