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

# `--args <file>`, after `--check-clean` where both are given, reads the
# script's own arguments from <file>, one argument per line. A line that
# opens with `#` and an empty line are skipped. Nothing may follow <file> on
# the command line. The command line in `.headwater/probe.yml` is a fold
# that every workspace keeps, so an argument that states the answer, such as
# a shelf, a kind or a ruling, goes in a held file that names its probe in
# its first ten lines, and the seal removes it from every workspace
# (HW-DR-0112, "The form of an oracle"). A missing file is a usage error.
case ${1:-} in
    --args)
        [ $# -eq 2 ] || hw_oracle_usage "--args takes one file and is the last option: --args <file>"
        hw_args_file=$2
        [ -f "$hw_args_file" ] || hw_oracle_usage "no file of arguments at $hw_args_file"
        set --
        while IFS= read -r hw_args_line || [ -n "$hw_args_line" ]; do
            case $hw_args_line in
                '#'* | '') continue ;;
            esac
            set -- "$@" "$hw_args_line"
        done < "$hw_args_file"
        ;;
esac

# Whether a file holds <name> as a whole name: not preceded and not followed
# by a letter, a digit, `_`, `-` or `.`, so `HW-DR-0112` is not found in
# `HW-DR-01120` and `0112-x.md` is not found in `draft-0112-x.md`.
#
#     hw_oracle_names_whole <name> <file>
hw_oracle_names_whole() {
    hw_whole_pattern=$(printf '%s\n' "$1" | sed 's/[][\.*^$+?(){}|/]/\\&/g')
    grep -E -q -- "(^|[^A-Za-z0-9_.-])$hw_whole_pattern([^A-Za-z0-9_-]|\\.[^A-Za-z0-9_-]|\\.\$|\$)" "$2"
}

#
# A script that names its documents sets `hw_oracle_subjects` to their paths,
# separated by spaces, and then the condition reads the strict errors on
# those documents alone: the findings of `headwater check --json` of
# severity `error` that no directive or adoption task holds (`escape` is
# `none`), which are the findings a strict run fails on. A sealed tree holds
# errors the seal made in documents it kept, such as a relation to a removed
# document, so a whole-tree condition fails every session there (#1874.2).
# `check-clean.sh` and every script that sets no subject read the whole tree.
hw_oracle_subjects=""

hw_oracle_pass_clean() {
    if [ "$hw_oracle_clean" = yes ] && [ -n "$hw_oracle_subjects" ]; then
        command -v jq > /dev/null 2>&1 || hw_oracle_usage "--check-clean on a named document reads the findings with jq, and no jq is on the path"
        hw_oracle_engine
        hw_oracle_tmp
        "$hw_engine" check --json --root . > "$hw_oracle_scratch/check.json" 2> "$hw_oracle_scratch/check.err"
        hw_errors=$(jq -r --arg subjects " $hw_oracle_subjects " '[.findings[] | (.path // "") as $path | select(.severity == "error" and .escape == "none" and ($subjects | contains(" " + $path + " ")))] | length' "$hw_oracle_scratch/check.json" 2> /dev/null) || hw_oracle_usage "the engine wrote no findings that jq reads"
        [ "$hw_errors" -eq 0 ] || hw_oracle_fail "$*, and \`headwater check --strict\` reports $hw_errors error(s) on it"
        hw_oracle_pass "$*, and \`headwater check --strict\` reports no error on it"
    fi
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
