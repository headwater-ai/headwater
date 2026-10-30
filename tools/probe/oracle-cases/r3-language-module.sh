# The cases of the `r3-language-module` oracle, for the probe
# HW-PROBE-a-session-renames-the-module-of-the-language-rule-and-moves-the-records-that-govern-it
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
    mv $c/check/src/language.rs $c/check/src/prose.rs
}

# Every caller names the new module, in its crate and outside it.
r3_code() {
    sed -i 's/^pub mod language;$/pub mod prose;/; s/\blanguage::/prose::/g' $c/check/src/lib.rs &&
        sed -i 's/crate::language`/crate::prose`/g' $c/check/src/voice.rs $c/check/src/source_form.rs &&
        sed -i 's/headwater_check::language::/headwater_check::prose::/g' $c/scaffold/src/lib.rs &&
        sed -i '33s/ language,/ prose,/; s/\blanguage::RULE/prose::RULE/g' $c/check/tests/fixtures.rs &&
        sed -i '40s/^    language, /    prose, /; s/\blanguage::RULE/prose::RULE/g' $c/scaffold/tests/pipeline.rs
}

# Each record that governs the module names the new path.
r3_records() {
    sed -i 's#engine/crates/check/src/language\.rs#engine/crates/check/src/prose.rs#g' \
        docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md \
        docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md \
        docs/evaluations/what-a-check-can-know.md
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

# Every caller follows but one, in another crate: scaffold/src/lib.rs.
left_caller() {
    reference && sed -i 's/headwater_check::prose::known(/headwater_check::language::known(/' $c/scaffold/src/lib.rs && grep -q 'headwater_check::language::known(' $c/scaffold/src/lib.rs
}

# The code moves, and the records still name the old path, so they govern
# nothing and `headwater check` reports each one.
stale_records() {
    r3_move && r3_code
}
more_wrong="left_caller stale_records"
