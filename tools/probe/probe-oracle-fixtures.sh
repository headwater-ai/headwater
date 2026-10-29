#!/bin/sh
# The decisive fixture of the probe oracles (#1874): every probe of a
# mechanical row runs its own declared oracle over the tree a campaign grades,
# and the oracle separates a session that did the task from one that did not.
#
#     sh tools/probe/probe-oracle-fixtures.sh [--cargo]
#
# `tools/probe/oracle-fixtures.sh` holds each generic script to a tree it
# must pass and one it must fail, and it names no probe. This script holds
# each probe. It builds the tree the way `tools/probe/campaign.sh` does: a
# copy of every file `git ls-files` names, sealed by `tools/probe/seal.sh`
# over every `current` probe on the shelf, and then the grading tree that
# `tools/probe/grading-tree.sh` makes from it, with each held file put back.
# Then, for each `current` probe on the shelf whose action is a row of the
# grid that an oracle grades (rows 2 to 9 and 11 of HW-DR-0112), it reads the
# probe's command from the sealed tree's own `.headwater/probe.yml` and runs
# it three times, each in a fresh copy of the grading tree:
#
# 1. over the grading tree as it stands, where it must exit 1, because a
#    session that does nothing fails;
# 2. with the probe's reference change applied, where it must exit 0;
# 3. with one plausible wrong change applied, where it must exit 1.
#
# An exit of 2 is a usage error of a generic script, such as a held file the
# grading tree does not hold, and it fails every case.
#
# The changes are in `tools/probe/oracle-cases/<oracle>.sh`, one file for
# each oracle, which defines the shell functions `reference` and `wrong`.
# Each runs from the root of the copy it changes. A reference change states
# the answer, so each file names its probe in its first ten lines, and the
# seal removes it from every workspace. This script checks both: a probe with
# no case file fails, a case file that does not name its probe fails, and a
# case file that the sealed tree still holds fails. A probe whose oracle is
# no `command:` or `test:` name awaits one, as `AWAITING_ORACLE` in
# `engine/crates/probe/tests/corpus.rs` lists, and it is skipped with a line
# that says so.
#
# A case file that sets `needs_cargo=yes` compiles a crate in every case. It
# runs only under `--cargo`, with one `CARGO_TARGET_DIR` for every case, and
# it is skipped with a line that says so otherwise. CI runs this script
# without `--cargo`. A case file may set `reference_recorded` to where the
# reference result is recorded by hand, in place of a `reference` function:
# the script prints that line and runs the other two cases.
#
# It prints, as one line, what `headwater check --strict` exits over the
# grading tree, because every `--check-clean` oracle assumes that a sealed
# tree is clean.
#
# It needs a built engine, `git`, `tar`, `jq` and what the seal needs. Exit 0
# when every case holds, 1 when one does not, and 8 when no engine is built.

set -u

case $0 in
    */*) invoked_from=${0%/*} ;;
    *) invoked_from=. ;;
esac
checkout=$(cd "$invoked_from/../.." && pwd -P)

cargo=no
case ${1:-} in
    --cargo) cargo=yes ;;
    "") ;;
    *)
        echo "usage: probe-oracle-fixtures.sh [--cargo]" >&2
        exit 2
        ;;
esac

engine=$checkout/engine/target/dev-release/headwater
[ -x "$engine" ] || engine=$checkout/engine/target/release/headwater
[ -x "$engine" ] || {
    echo "probe-oracle-fixtures: no engine under $checkout/engine/target. Build it first: cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked" >&2
    exit 8
}
HW_ORACLE_ENGINE=$engine
export HW_ORACLE_ENGINE

work=$(mktemp -d "${TMPDIR:-/tmp}/probe-oracle-fixtures.XXXXXX") || exit 1
source_tree=$work/source
sealed=$work/sealed
grading=$work/grading
mkdir -p "$source_tree"
git -C "$checkout" ls-files -z > "$work/files" || exit 1
tar -C "$checkout" --null -T "$work/files" -cf - | tar -C "$source_tree" -xf - || {
    echo "probe-oracle-fixtures: the source tree could not be copied." >&2
    exit 1
}

# The front matter value of <key> in <file>, unquoted.
facet() {
    awk -v key="$1" 'NR == 1 { next } /^---$/ { exit } index($0, key ": ") == 1 { sub("^" key ": *", ""); gsub(/^"|"$/, ""); print; exit }' "$2"
}

mechanical=" change_governed_code refactor_across_files add_document edit_depended_document supersede_or_retire move_document change_taxonomy repair_check ungoverned_control "

: > "$work/current"
: > "$work/graded"
for file in "$checkout"/docs/probes/*.md; do
    id=$(facet id "$file")
    [ -n "$id" ] || continue
    [ "$(facet status "$file")" = current ] || continue
    printf '%s\n' "$id" >> "$work/current"
    case $mechanical in
        *" $(facet action "$file") "*) printf '%s\t%s\n' "$id" "$(facet oracle "$file")" >> "$work/graded" ;;
    esac
done

cp -a "$source_tree" "$sealed"
# shellcheck disable=SC2046 # one probe identifier per word
sh "$checkout/tools/probe/seal.sh" "$sealed" $(cat "$work/current") > "$work/seal.out" 2> "$work/seal.err" || {
    echo "probe-oracle-fixtures: the seal refused, exit $?:" >&2
    sed -n '1,10p' "$work/seal.err" >&2
    rm -rf "$work"
    exit 1
}
sh "$checkout/tools/probe/grading-tree.sh" "$sealed" "$source_tree" "$grading" > "$work/grading.out" 2> "$work/grading.err" || {
    echo "probe-oracle-fixtures: the grading tree could not be built:" >&2
    sed -n '1,10p' "$work/grading.err" >&2
    rm -rf "$work"
    exit 1
}

# The campaign builds the engine at the pin into the grading copy
# (`build_oracle` in `tools/probe/campaign.sh`), and an oracle such as
# `refresh-figures` runs the tree's own binary. This checkout's binary is
# that build.
mkdir -p "$grading/engine/target/dev-release"
cp "$engine" "$grading/engine/target/dev-release/headwater"

strict=0
(cd "$grading" && "$engine" check --strict --root .) > "$work/strict.out" 2> "$work/strict.err" || strict=$?
echo "the grading tree: \`headwater check --strict\` exits $strict"
[ "$strict" -eq 0 ] || sed -n '/error/p' "$work/strict.out" | sed -n '1,10p' | sed 's/^/    /'

"$engine" probe plan --root "$grading" --oracles > "$work/oracles" 2> "$work/oracles.err" || {
    echo "probe-oracle-fixtures: the engine refused the sealed tree's oracles." >&2
    rm -rf "$work"
    exit 1
}

failures=0
cases=0
CARGO_TARGET_DIR=$work/cargo-target
export CARGO_TARGET_DIR

fail() {
    echo "FAIL $*"
    failures=$((failures + 1))
}

# run_case <want> <label> <key> <command> [<change>]
run_case() {
    want=$1 label=$2 key=$3 command=$4 change=${5:-}
    cases=$((cases + 1))
    tree=$work/case
    rm -rf "$tree"
    cp -a "$grading" "$tree"
    if [ -n "$change" ]; then
        (cd "$tree" && . "$case_file" && "$change") > "$work/change.out" 2>&1 < /dev/null || {
            fail "$label: the $change change of $key did not apply: $(sed -n '1p' "$work/change.out")"
            return
        }
    fi
    got=0
    (cd "$tree" && sh -c "$command") > "$work/out" 2> "$work/err" < /dev/null || got=$?
    if [ "$got" -ne "$want" ]; then
        fail "$label: exits $got, and the case wants $want: $(sed -n '$p' "$work/out")"
        sed 's/^/    stderr: /' "$work/err" | sed -n '1,5p'
    else
        echo "ok   $label: $(sed -n '$p' "$work/out")"
    fi
}

while IFS="$(printf '\t')" read -r id oracle; do
    case $oracle in
        command:* | test:*) key=${oracle#*:} ;;
        *)
            echo "skip $id: \`oracle: $oracle\` is no command or test, so it awaits an oracle (AWAITING_ORACLE)"
            continue
            ;;
    esac
    case_file=$checkout/tools/probe/oracle-cases/$key.sh
    if [ ! -f "$case_file" ]; then
        fail "$id: no case file tools/probe/oracle-cases/$key.sh"
        continue
    fi
    # The seal is read first. A case file that names no current probe stays
    # in the sealed tree and fails here. One that names another probe is
    # removed and fails the naming check below. With the naming check first,
    # a file the seal keeps names no probe and fails that check, so this one
    # could never fire (#1874.3).
    if [ -e "$sealed/tools/probe/oracle-cases/$key.sh" ]; then
        fail "$id: the sealed tree still holds tools/probe/oracle-cases/$key.sh"
        continue
    fi
    sed -n '1,10p' "$case_file" | grep -q -F -- "$id" || {
        fail "$id: tools/probe/oracle-cases/$key.sh does not name the probe in its first ten lines, so the seal leaves its reference change in every workspace"
        continue
    }
    command=$(awk -F '\t' -v key="$key" '$1 == key && $2 == "command" { sub(/^[^\t]*\t[^\t]*\t/, ""); print; exit }' "$work/oracles")
    if [ -z "$command" ]; then
        fail "$id: the sealed tree's probe.yml declares no command for \`$key\`, so the seal cut the entry or nothing declares it"
        continue
    fi
    needs_cargo=no
    reference_recorded=""
    # shellcheck disable=SC1090 # the case file of this oracle
    . "$case_file"
    if [ "$needs_cargo" = yes ] && [ "$cargo" = no ]; then
        echo "skip $id: \`$key\` compiles a crate in every case, so it runs under --cargo"
        continue
    fi
    run_case 1 "$key: the grading tree as it stands fails" "$key" "$command"
    if [ -n "$reference_recorded" ]; then
        echo "note $key: the reference change is recorded by hand: $reference_recorded"
    else
        run_case 0 "$key: the reference change passes" "$key" "$command" reference
    fi
    run_case 1 "$key: the wrong change fails" "$key" "$command" wrong
    unset -f reference wrong 2> /dev/null
done < "$work/graded"

rm -rf "$work"
if [ "$failures" -ne 0 ]; then
    echo "probe-oracle-fixtures: $failures failed, of the $cases cases run and the probes refused before a case."
    exit 1
fi
echo "probe-oracle-fixtures: all $cases cases hold."
