# The shared part of the generic oracle scripts (#1874). Each script under
# `tools/probe/oracles/` sources this file. It is not run on its own.
#
# The form an oracle script follows is stated once, above `oracles:` in
# `.headwater/probe.yml`, and in HW-DR-0112. In short:
#
# - It runs from the root of the tree it grades, which in a campaign is the
#   grading copy, and it reads no history, because that copy holds none.
# - It exits 0 when the probe passes and 1 when it fails. A usage error
#   exits 2, which the `exit` parser also reads as a failure.
# - It prints exactly one line on standard output, the evidence, on a pass
#   and on a fail. What the engine prints goes to a scratch directory and to
#   standard error, never to standard output.
#
# The engine is `HW_ORACLE_ENGINE` when that is set, which is how
# `tools/probe/oracle-fixtures.sh` points a script at the checkout's binary
# from a tree that has none. Otherwise it is the tree's own
# `engine/target/dev-release/headwater`, which the campaign builds at the pin,
# and then `engine/target/release/headwater`.

hw_oracle_name=${0##*/}
hw_oracle_name=${hw_oracle_name%.sh}
hw_oracle_scratch=""

# Print the one line of evidence and exit with the status given.
hw_oracle_end() {
    hw_status=$1
    shift
    [ -n "$hw_oracle_scratch" ] && rm -rf "$hw_oracle_scratch"
    printf '%s: %s\n' "$hw_oracle_name" "$*"
    exit "$hw_status"
}

hw_oracle_pass() { hw_oracle_end 0 "pass: $*"; }
hw_oracle_fail() { hw_oracle_end 1 "fail: $*"; }
hw_oracle_usage() { hw_oracle_end 2 "usage: $*"; }

# A scratch directory for what the engine prints, removed by hw_oracle_end.
hw_oracle_tmp() {
    if [ -z "$hw_oracle_scratch" ]; then
        hw_oracle_scratch=$(mktemp -d "${TMPDIR:-/tmp}/hw-oracle.XXXXXX") || hw_oracle_end 2 "no scratch directory could be made"
    fi
}

hw_oracle_engine() {
    if [ -n "${HW_ORACLE_ENGINE:-}" ]; then
        hw_engine=$HW_ORACLE_ENGINE
    elif [ -x engine/target/dev-release/headwater ]; then
        hw_engine=engine/target/dev-release/headwater
    else
        hw_engine=engine/target/release/headwater
    fi
    [ -x "$hw_engine" ] || hw_oracle_end 2 "no engine at $hw_engine, so nothing can be graded"
}

# Run `headwater check --strict` over the tree. Return its exit status, and
# leave its report in "$hw_oracle_scratch/check.out".
hw_oracle_check_strict() {
    hw_oracle_engine
    hw_oracle_tmp
    "$hw_engine" check --strict --root . > "$hw_oracle_scratch/check.out" 2> "$hw_oracle_scratch/check.err"
}

# `--check-clean` as the first argument adds the condition of check-clean.sh
# after the script's own: the scripts call hw_oracle_pass_clean in place of
# hw_oracle_pass, and it fails when the strict check does not exit 0.
hw_oracle_clean=no
case ${1:-} in
    --check-clean)
        hw_oracle_clean=yes
        shift
        ;;
esac

hw_oracle_pass_clean() {
    if [ "$hw_oracle_clean" = yes ]; then
        hw_check_status=0
        hw_oracle_check_strict || hw_check_status=$?
        [ "$hw_check_status" -eq 0 ] || hw_oracle_fail "$*, and \`headwater check --strict\` exits $hw_check_status"
        hw_oracle_pass "$*, and \`headwater check --strict\` exits 0"
    fi
    hw_oracle_pass "$*"
}

# The path of a document, from its identifier or its path, as the first line
# `headwater explain` prints. Empty when the engine refuses it.
hw_oracle_path_of() {
    hw_oracle_engine
    "$hw_engine" explain --root . "$1" 2> /dev/null | sed -n '1p'
}

# The front matter of a file: the lines between the first two `---` lines.
hw_oracle_front_matter() {
    awk 'NR == 1 && $0 != "---" { exit } NR > 1 && $0 == "---" { exit } NR > 1 { print }' "$1"
}

# The `relations:` block of the front matter of a file, the key line
# included. A summary or a title that mentions a document is not a link, so
# the scripts that ask for a link read this block alone.
hw_oracle_relations() {
    hw_oracle_front_matter "$1" | awk '/^[^ #]/ { inside = ($0 ~ /^relations:/) } inside { print }'
}

# The identifier a file declares, from the `id:` line of its front matter.
hw_oracle_id_of() {
    hw_oracle_front_matter "$1" | sed -n 's/^id: *"\{0,1\}\([^" ]*\)"\{0,1\} *$/\1/p' | sed -n '1p'
}
