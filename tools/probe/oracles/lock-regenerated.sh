#!/bin/sh
# Generic oracle: the taxonomy lock is what a resolve writes, and the rule
# the change asked for is one the engine applies (#1874). Row 8 of the grid
# in HW-DR-0112, "change taxonomy or config".
#
#     sh tools/probe/oracles/lock-regenerated.sh [--check-clean] [<form>]...
#
# It passes when `headwater taxonomy resolve --check` exits 0: the committed
# `.headwater/taxonomy.lock` is what a run over the taxonomy sources
# produces, so a session that edited the overlay or a package also resolved.
# A probe of row 8 passes `--check-clean`, because the grid grades that row
# on both conditions.
#
# On a tree that nobody changed the lock already matches its sources, so the
# bare form passes the pin, and a probe cannot fail a session that did
# nothing (#1874.3). So a probe names what its change must make the engine
# do, in forms that come from a held file through `--args`, one word to a
# line, because a form states the answer:
#
#     retires <carrier> <term> [<replacement>]
#     binds <path> <term>
#
# Each form is read by the engine's own retired-term rule, never by a match
# on a line of the overlay, so a term that only a comment, a quoted reason
# or another list holds does nothing and fails (round 4 of #1874.3). The
# script copies `.headwater/` and one file into a scratch root, appends one
# sentence that writes <term> to the copy of the file, and runs `headwater
# check --json` there. It writes nothing in the tree it grades.
#
# - `retires` passes when a `language.retired_term.used` finding on the copy
#   of <carrier> says that a regime retires `<term>`. With <replacement>, the
#   finding's remediation must be `write <replacement>`, so the term and its
#   replacement are one entry. <carrier> is a document the probe's task does
#   not touch, and a carrier that is not there is a usage error. A word after
#   <term> is the replacement unless it is `retires` or `binds`.
# - `binds` passes when the same finding lands on <path>: the house rules
#   reach it, which is what `outside_root` does for a file outside the
#   document roots.
#
# With `--check-clean` and at least one form, the strict errors counted are
# those on `.headwater/overlay.yml` and on each path a `binds` form names.

set -u
. "${0%/*}/lib.sh"
usage="lock-regenerated.sh [--check-clean] [retires <carrier> <term> [<replacement>] | binds <path> <term>]..."
command -v jq > /dev/null 2>&1 || hw_oracle_usage "lock-regenerated.sh reads the findings with jq, and no jq is on the path"
hw_oracle_engine
hw_oracle_tmp

# The forms, one a line: `<form><TAB><file><TAB><term><TAB><replacement>`.
: > "$hw_oracle_scratch/forms"
while [ $# -gt 0 ]; do
    form=$1
    case $form in
        retires | binds) ;;
        *) hw_oracle_usage "\`$form\` is no form, and the forms are \`retires\` and \`binds\`: $usage" ;;
    esac
    [ $# -ge 3 ] || hw_oracle_usage "\`$form\` takes a file and a term: $usage"
    file=${2#./} term=$3 replacement=""
    shift 3
    if [ "$form" = retires ] && [ $# -gt 0 ]; then
        case $1 in
            retires | binds) ;;
            *) replacement=$1 && shift ;;
        esac
    fi
    printf '%s\t%s\t%s\t%s\n' "$form" "$file" "$term" "$replacement" >> "$hw_oracle_scratch/forms"
done

status=0
"$hw_engine" taxonomy resolve --check --root . > "$hw_oracle_scratch/resolve.out" 2> "$hw_oracle_scratch/resolve.err" || status=$?
[ "$status" -eq 0 ] || hw_oracle_fail "\`headwater taxonomy resolve --check\` exits $status, so the lock is not what the sources resolve to"

held=""
n=0
while IFS="$(printf '\t')" read -r form file term replacement; do
    n=$((n + 1))
    if [ ! -f "$file" ]; then
        [ "$form" = retires ] && hw_oracle_usage "no carrier at $file, which the retired term is written into"
        hw_oracle_fail "nothing is at $file, which the house rules must reach"
    fi
    root=$hw_oracle_scratch/root.$n
    mkdir -p "$root/$(dirname -- "$file")" || hw_oracle_usage "no scratch root could be made"
    cp -R .headwater "$root/.headwater" && cp -- "$file" "$root/$file" || hw_oracle_usage "the scratch root could not be filled"
    printf '\nThe team ships a %s change.\n' "$term" >> "$root/$file"
    "$hw_engine" check --json --root "$root" > "$root.json" 2> "$root.err" || true
    hits=$(jq -r --arg path "$file" --arg term "$term" --arg replacement "$replacement" '[.findings[]? | select(.rule == "language.retired_term.used" and .path == $path and ((.message // "") | contains(" retires `" + $term + "`, and ")) and ($replacement == "" or .remediation == "write `" + $replacement + "`"))] | length' "$root.json" 2> /dev/null) || hw_oracle_usage "\`headwater check --json\` in the scratch root wrote no findings that jq reads"
    case $form in
        retires)
            if [ "$hits" -eq 0 ]; then
                [ -n "$replacement" ] && hw_oracle_fail "the engine does not retire \`$term\` with the replacement \`$replacement\` in one entry, as its retired-term rule reads the lock"
                hw_oracle_fail "the engine does not retire \`$term\`, as its retired-term rule reads the lock"
            fi
            held="${held:+$held, }\`$term\` retired${replacement:+ for \`$replacement\`}"
            ;;
        binds)
            [ "$hits" -gt 0 ] || hw_oracle_fail "the engine's retired-term rule does not reach $file, so no list of the lock binds it"
            case " $hw_oracle_subjects " in
                *" $file "*) ;;
                *) hw_oracle_subjects="${hw_oracle_subjects:+$hw_oracle_subjects }$file" ;;
            esac
            held="${held:+$held, }$file bound"
            ;;
    esac
done < "$hw_oracle_scratch/forms"
if [ "$n" -gt 0 ]; then
    hw_oracle_subjects=".headwater/overlay.yml${hw_oracle_subjects:+ $hw_oracle_subjects}"
fi
hw_oracle_pass_clean "\`headwater taxonomy resolve --check\` exits 0${held:+, and the engine reads $held}"
