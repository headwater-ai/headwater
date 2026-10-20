#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# One try of the tuning round of the additive campaign, from the plan to the
# canary, as seven verbs (#1821). Before this script, each try had a launch
# script, a wait condition and a canary reader typed by hand, the total spend
# of the round was prose that a person added to by hand, and one hand reader
# read `named 1 path outside` as a blank field because it matched the plural
# alone.
#
#     sh tools/probe/try.sh status   <root>
#     sh tools/probe/try.sh spent    <root>
#     sh tools/probe/try.sh plan     <root>
#     sh tools/probe/try.sh launch   <root>
#     sh tools/probe/try.sh relaunch <root> [<try>]
#     sh tools/probe/try.sh canary   <root> [<try>]
#     sh tools/probe/try.sh wait     <root> [<try>]
#
# `<root>` is the root of the round: one directory that holds every try root
# of the tuning round, such as `<root>/docs-try1`, and the two files a person
# writes once for the round, `<root>/env` and `<root>/base-pin`. A try root is
# the `--root` of one `tools/probe/campaign-drive.sh` driver, and its batch
# directories are `<try>/<batch>/sessions/<job>`. `<try>` names a try root
# under `<root>`. Without it, a verb takes the try of the open layer:
# `<root>/<layer>-try<n>`, from `rounds.tuning.open` in
# `.headwater/probe.yml`.
#
# Every figure is read from the declaration through `tools/probe/declared.py`
# and from the files under `<root>`, and this script stores no total.
#
# ## The two files of the round
#
# - `<root>/env` holds the facts of the host, one `KEY=value` per line:
#   `HEADWATER_DRIVE_CLAUDE_VERSIONS`, `CARGO_TARGET_DIR` and `TMPDIR`, which
#   `launch` and `relaunch` need and export to the driver, and optionally
#   `TRY_FLOOR_ROOT_KB` and `TRY_FLOOR_TRY_KB`, the disk floors of `wait`. A
#   line that sets any other key, or a line that is not a comment, a blank
#   or `KEY=value`, exits 2. The file is read and never run.
# - `<root>/base-pin` is one line, `<scenario> <sha>`: the base scenario of
#   the open layer and the commit at which that base was measured. The
#   declaration cannot carry it, because `declared.py` refuses a seventh
#   field of `open`.
#
# The launch constants (model, harness version, session cents, parallel
# count, overshoot margin, seed) are in `tools/probe/additive-launch.env`,
# which this script sources.
#
# ## The verbs
#
# - `spent <root>` prints the sum of every `cost` and every `spent-before` at
#   `<root>/<try>/<batch>/sessions/<job>/`, over every try and every batch,
#   dot-named directories included, at that depth and no other. A file there
#   that does not hold a whole number of cents exits 2, because a total that
#   skips it is too low.
# - `status <root>` prints each try root with what it spent, its markers and
#   whether its driver runs, and then the total, the approval and the open
#   layer.
# - `plan <root>` exits 0 when a try of the open layer may launch, and prints
#   the figures it read. It refuses with 5 when no layer is open, before it
#   builds anything, and with 5 when `spent <root>` plus the open layer's
#   `cap_cents` is above the tuning round's `approved_cents`. Equal passes.
#   It then builds the tree of the open scenario's base at the pin of
#   `<root>/base-pin` and at `HEAD`, each with `campaign.sh --scenario-tree`,
#   prints every path by which the two differ, and exits 4 when one matches
#   a pattern of `base_invalidators`. In a pattern, `*` and `**` both match
#   any string, `/` included. A refusal of spend is never 4, so a wrapper can
#   tell a stale base from a refusal.
# - `launch <root>` runs `plan`, and exits with its status when it is not 0.
#   It then makes the try root `<root>/<layer>-try<n>`, writes the spec
#   `additive scenario-<n> tuning` there, outside the checkout, and starts
#   `campaign-drive.sh` detached, from this checkout and pinned at its
#   `HEAD`, with the constants of `additive-launch.env`, the open layer's
#   `cap_cents` as `--cap-cents`, and `--on-canary` running `canary` on the
#   try. It writes `driver.argv`, `driver.pid` and `launched` in the try
#   root. A try root that exists already exits 6, and nothing is started.
#   Before it makes the try root, it refuses with 6 the four states of the
#   checkout that the driver's pin check refuses: `HEAD` on a branch, a pin
#   that is not on `origin/main`, a dirty tree, and, for `relaunch`, a
#   `HEAD` that is no longer the pin of `driver.argv`. The driver checks the
#   harness and the token itself.
# - `relaunch <root> [<try>]` starts the driver again on a try root that
#   `launch` made, with the same arguments, read from `driver.argv`. It
#   exits 6 when the try root does not exist, when its driver runs, or when
#   `done`, `capped`, `canary-failed` or `stop` is present. It never writes
#   `a.done`, so a capped batch stays capped until a person rules on it.
# - `canary <root> [<try>]` reads the eight records of the canary, the jobs
#   of `<try>/a/jobs` that have a `status`, and writes `<try>/canary-passed`
#   only when there are eight and each one has `status` 0, a cost of at most
#   `TRY_SESSION_CENTS + TRY_OVERSHOOT_CENTS`, and the sentences of
#   `tools/probe/probe-record.sh` that state 0 calls to a web tool, 0 paths
#   outside its workspace and no refused provider fetch, and that the intent
#   hook was live when the scenario of `<try>/spec` holds the `hook` layer
#   and was not live otherwise. A count of 1 is written in the singular
#   (`1 path`, `1 call`), so the reader matches the sentence of 0 and treats
#   every other sentence, the "not counted" one included, as offending.
#   Otherwise it writes `<try>/canary-needs-reader` with one line for each
#   offence, and exits 7. With `canary-passed` present it reads nothing and
#   exits 0, because a person who read the records may have written it.
# - `wait <root> [<try>]` is the condition for `tools/run/wait-for.sh`. It
#   exits 0 and prints one line when a person must read something or the
#   driver has stopped: `disk-floor`, `done`, `canary-failed`, `capped`,
#   `stop`, `canary-needs-reader`, `canary-recorded` or `driver-exited`. It
#   exits 1 while the driver runs and needs nobody. When the free space
#   under `/` is below `TRY_FLOOR_ROOT_KB`, or on the file system of `<root>`
#   below `TRY_FLOOR_TRY_KB`, it writes `<try>/stop` first, which the driver
#   reads within a minute.
#
# ## The base tree, and `campaign.sh`
#
# Both trees are built by this checkout's `campaign.sh`, with this
# checkout's engine and tools, from `git archive` of each commit. So the
# engine binary in a tree that holds the `engine` layer is the same file in
# both trees and never differs on its own: the engine's sources stand for
# it, under `engine/crates/*/src/**`.
#
# The owner ruled on 2026-10-20 (#1821) that an edit to `campaign.sh` makes a
# base stale only when it touches the code that builds, runs or grades a
# session, and that the gate and refusal checks are exempt. So when
# `tools/probe/campaign.sh` differs, `plan` reads `diff -U0` of the two
# copies. A removed line is neutral when it is a comment, a blank, or inside
# a region of the older copy marked `# base-neutral: begin <reason>` and
# `# base-neutral: end`, and an added line the same against the newer copy.
# `campaign.sh` makes the base stale only when one line is not neutral, and
# `plan` prints that line. A region must close before the line
# `for tool in git jq tar awk cargo claude bwrap`, where the batch's first
# check of its tools runs. A copy with a region that reaches it, or an
# unclosed region, exits 2. `tools/probe/seal.sh` and `tools/probe/leak.py`
# are not on the list (owner, #1819).
#
# `HEADWATER_TRY_DECLARATION`, `HEADWATER_TRY_DRIVER`,
# `HEADWATER_TRY_CAMPAIGN` and `HEADWATER_TRY_REPO` name the declaration, the
# driver, the `campaign.sh` it runs and the checkout the driver runs from, so
# that `tools/probe/try-fixtures.sh` runs every
# verb with stubs. It needs `git`, `awk`, `python3` with PyYAML, `diff`,
# `sha256sum` and `setsid`.
#
# ## Exit status
#
#   0   the verb did what it states: `canary-passed` is written, the plan
#       passes, the driver is started, or `wait` met its condition
#   1   `wait` alone: the driver runs and needs no reader
#   2   a usage error, or a file it reads is missing or does not read:
#       `<root>`, `<root>/env`, `<root>/base-pin`, `driver.argv`, a cost, a
#       marked region of `campaign.sh`, or the declaration
#   3   a tool is missing, or a tree did not build
#   4   the base is stale: a path by which the two trees differ matches
#       `base_invalidators`. Nothing else exits 4
#   5   the spend is refused: no layer is open, or what the round spent plus
#       the open layer's cap is above the round's approval
#   6   the state of the try refuses the verb: `launch` on a try root that
#       exists, `relaunch` on a try that is absent, running, done, capped,
#       stopped or whose canary failed, `launch` or `relaunch` on a
#       checkout the driver's pin check refuses, or `canary` before
#       `canary-recorded`
#   7   `canary` wrote `canary-needs-reader`

set -u

here=$(cd "$(dirname "$0")" && pwd -P)
repo=$(cd "$here/../.." && pwd -P)
self=$here/try.sh
declaration=${HEADWATER_TRY_DECLARATION:-$repo/.headwater/probe.yml}
driver=${HEADWATER_TRY_DRIVER:-$here/campaign-drive.sh}
campaign=${HEADWATER_TRY_CAMPAIGN:-$here/campaign.sh}
checkout=${HEADWATER_TRY_REPO:-$repo}

usage() {
    echo "usage: sh tools/probe/try.sh status|spent|plan|launch <root>, or relaunch|canary|wait <root> [<try>]" >&2
    exit 2
}
say() { printf 'try: %s\n' "$1" >&2; }

[ $# -ge 2 ] || usage
verb=$1 root=$2
try_name=${3:-}
[ $# -le 3 ] || usage
case $verb in
    status | spent | plan | launch) [ $# = 2 ] || usage ;;
    relaunch | canary | wait) ;;
    *) usage ;;
esac
[ -d "$root" ] || { say "no round root at $root."; exit 2; }
root=$(cd "$root" && pwd -P)

# shellcheck source=tools/probe/additive-launch.env
. "$here/additive-launch.env"

# ------------------------------------------------------------ the inputs

# declared <mode>: the printer of the declaration. Its 3 is a missing
# PyYAML, and anything else it refuses is a declaration that does not read.
declared() {
    command -v python3 >/dev/null 2>&1 || { say "python3 is not on the path."; exit 3; }
    declared_out=$(python3 "$here/declared.py" "$declaration" "$1")
    declared_status=$?
    case $declared_status in
        0) printf '%s\n' "$declared_out" ;;
        3) exit 3 ;;
        *) exit 2 ;;
    esac
}

# read_env: the host facts of `<root>/env`, set and exported. `need` is 1
# where the three host facts are required.
read_env() {
    need=$1
    env_file=$root/env
    if [ ! -f "$env_file" ]; then
        [ "$need" = 1 ] || return 0
        say "no $env_file. Write HEADWATER_DRIVE_CLAUDE_VERSIONS, CARGO_TARGET_DIR and TMPDIR there, one KEY=value per line."
        exit 2
    fi
    env_lines=$(awk '
        /^[ \t]*(#.*)?$/ { next }
        /^(HEADWATER_DRIVE_CLAUDE_VERSIONS|CARGO_TARGET_DIR|TMPDIR|TRY_FLOOR_ROOT_KB|TRY_FLOOR_TRY_KB)=/ { print; next }
        { printf "bad %d %s\n", NR, $0 }
    ' "$env_file")
    if printf '%s\n' "$env_lines" | grep -q '^bad '; then
        printf '%s\n' "$env_lines" | awk -v f="$env_file" '/^bad / { n = $2; sub(/^bad [0-9]+ /, ""); print "try: line " n " of " f " is not KEY=value of a key it reads: " $0 }' >&2
        exit 2
    fi
    while IFS= read -r env_line; do
        [ -n "$env_line" ] || continue
        env_key=${env_line%%=*}
        env_value=${env_line#*=}
        case $env_key in
            TRY_FLOOR_ROOT_KB) TRY_FLOOR_ROOT_KB=$env_value ;;
            TRY_FLOOR_TRY_KB) TRY_FLOOR_TRY_KB=$env_value ;;
            HEADWATER_DRIVE_CLAUDE_VERSIONS) HEADWATER_DRIVE_CLAUDE_VERSIONS=$env_value; export HEADWATER_DRIVE_CLAUDE_VERSIONS ;;
            CARGO_TARGET_DIR) CARGO_TARGET_DIR=$env_value; export CARGO_TARGET_DIR ;;
            TMPDIR) TMPDIR=$env_value; export TMPDIR ;;
        esac
    done <<EOF
$env_lines
EOF
    if [ "$need" = 1 ]; then
        for env_key in HEADWATER_DRIVE_CLAUDE_VERSIONS CARGO_TARGET_DIR TMPDIR; do
            grep -q "^$env_key=." "$env_file" || { say "$env_file sets no $env_key."; exit 2; }
        done
    fi
}

# open_fields: `rounds.tuning.open`, as `layer scenario try cap`, or `none`.
open_fields() {
    open_line=$(declared open) || exit $?
    if [ "$open_line" = none ]; then
        echo none
        return
    fi
    printf '%s\n' "$open_line" | awk '{ print $1, $2, $3, $4 }'
}

# resolve_try: the try root a verb acts on, in `$try`.
resolve_try() {
    if [ -n "$try_name" ]; then
        case $try_name in
            */* | . | ..) say "a try is the name of a directory under $root, not \`$try_name\`."; exit 2 ;;
        esac
        try=$root/$try_name
        return
    fi
    resolve_open=$(open_fields) || exit $?
    if [ "$resolve_open" = none ]; then
        say "no layer is open under rounds.tuning.open, so name the try: sh tools/probe/try.sh $verb $root <try>."
        exit 2
    fi
    # shellcheck disable=SC2086
    set -- $resolve_open
    try=$root/$1-try$3
}

# ------------------------------------------------------------ the spend

# spent_round: every cost and spent-before at `<root>/<try>/<batch>/sessions/<job>/`.
spent_round() {
    spent_files=$(mktemp "${TMPDIR:-/tmp}/try-spent.XXXXXX") || exit 3
    for spent_try in "$root"/*/ "$root"/.[!.]*/ "$root"/..?*/; do
        [ -d "$spent_try" ] || continue
        for spent_batch in "$spent_try"*/ "$spent_try".[!.]*/ "$spent_try"..?*/; do
            [ -d "${spent_batch}sessions" ] || continue
            find "${spent_batch}sessions" -mindepth 2 -maxdepth 2 -type f \( -name cost -o -name spent-before \) -print >> "$spent_files"
        done
    done
    spent_sum=$(awk '
        {
            line = ""
            if ((getline line < $0) <= 0) line = ""
            close($0)
            if (line !~ /^[0-9]+$/) { bad = bad "\n  " $0 " holds \"" line "\""; next }
            s += line
        }
        END {
            if (bad != "") { printf "bad%s\n", bad; exit 1 }
            print s + 0
        }
    ' "$spent_files")
    spent_status=$?
    rm -f "$spent_files"
    if [ "$spent_status" != 0 ]; then
        say "a file of what a session spent does not hold a whole number of cents, so the round's total cannot be counted:"
        printf '%s\n' "$spent_sum" | sed 1d >&2
        exit 2
    fi
    printf '%s\n' "$spent_sum"
}

# try_spent <try>: the same sum, over one try root.
try_spent() {
    try_spent_sum=0
    for try_batch in "$1"/*/ "$1"/.[!.]*/ "$1"/..?*/; do
        [ -d "${try_batch}sessions" ] || continue
        try_spent_sum=$((try_spent_sum + $(find "${try_batch}sessions" -mindepth 2 -maxdepth 2 -type f \( -name cost -o -name spent-before \) -exec cat {} + 2>/dev/null | awk '{ s += $1 } END { print s + 0 }')))
    done
    echo "$try_spent_sum"
}

# ------------------------------------------------------------ the base

# manifest <tree>: `<path>\t<sha256>` for every file, and `<path>\t-> <target>`
# for every link, sorted.
manifest() {
    (
        cd "$1" || exit 3
        find . -type f -print0 | xargs -0 -r sha256sum | awk '{ h = $1; sub(/^[0-9a-f]+ [ *]/, ""); print $0 "\t" h }'
        find . -type l -printf '%p\t-> %l\n'
    ) | LC_ALL=C sort
}

# regions <copy>: the line numbers inside the marked regions of one copy of
# campaign.sh. A region that is unclosed, nested, or that reaches the check
# of the tools exits 2.
regions() {
    awk -v file="$2" '
        /^[ \t]*# base-neutral: begin/ {
            if (on) { bad = "a region begins inside another at line " NR; exit }
            on = 1; start = NR
        }
        /^for tool in git jq tar awk cargo claude bwrap/ {
            sentinel = 1
            if (on) { bad = "the region that begins at line " start " reaches the check of the tools at line " NR; exit }
        }
        on { print NR }
        /^[ \t]*# base-neutral: end/ {
            if (!on) { bad = "a region ends at line " NR " and none began"; exit }
            on = 0; regions++
            if (sentinel) { bad = "the region that ends at line " NR " is past the check of the tools"; exit }
        }
        END {
            if (bad == "" && on) bad = "the region that begins at line " start " never ends"
            if (bad != "") { print "campaign.sh copy " file ": " bad > "/dev/stderr"; exit 2 }
        }
    ' "$1"
}

# neutral <old> <new>: 0 when every line of `diff -U0` is neutral, and
# otherwise 1 and the first line that is not.
neutral() {
    neutral_dir=$(mktemp -d "${TMPDIR:-/tmp}/try-neutral.XXXXXX") || exit 3
    regions "$1" older > "$neutral_dir/old" || { rm -rf "$neutral_dir"; exit 2; }
    regions "$2" newer > "$neutral_dir/new" || { rm -rf "$neutral_dir"; exit 2; }
    diff -U0 "$1" "$2" > "$neutral_dir/diff"
    awk -v oldr="$neutral_dir/old" -v newr="$neutral_dir/new" '
        BEGIN {
            while ((getline n < oldr) > 0) ro[n] = 1
            while ((getline n < newr) > 0) rn[n] = 1
        }
        function quiet(text) { return text ~ /^[ \t]*(#.*)?$/ }
        NR <= 2 && (/^--- / || /^\+\+\+ /) { next }
        /^@@ / {
            split($2, a, ","); split($3, b, ",")
            o = substr(a[1], 2) + 0; n = substr(b[1], 2) + 0
            next
        }
        /^-/ {
            text = substr($0, 2)
            if (!quiet(text) && !(o in ro)) { print "removed line " o ": " text; bad = 1; exit }
            o++; next
        }
        /^\+/ {
            text = substr($0, 2)
            if (!quiet(text) && !(n in rn)) { print "added line " n ": " text; bad = 1; exit }
            n++; next
        }
        END { exit bad }
    ' "$neutral_dir/diff"
    neutral_status=$?
    rm -rf "$neutral_dir"
    return "$neutral_status"
}

# base_check <base> <sha>: build both trees, print what differs, and set
# `stale` to 1 when a path invalidates the base.
base_check() {
    invalidators=$(declared base_invalidators) || exit $?
    base_dir=$(mktemp -d "${TMPDIR:-/tmp}/try-plan.XXXXXX") || exit 3
    printf '%s\n' "$invalidators" > "$base_dir/invalidators"
    for base_side in pin head; do
        if [ "$base_side" = pin ]; then base_rev=$2; else base_rev=HEAD; fi
        sh "$campaign" --scenario-tree "$1" --out "$base_dir/$base_side" --at "$base_rev" \
            > "$base_dir/$base_side.out" 2> "$base_dir/$base_side.err"
        base_status=$?
        if [ "$base_status" != 0 ] || [ ! -d "$base_dir/$base_side/trees/additive-scenario-$1" ]; then
            say "the tree of scenario $1 at $base_rev did not build (campaign.sh exited $base_status):"
            tail -5 "$base_dir/$base_side.err" >&2
            rm -rf "$base_dir"
            exit 3
        fi
        manifest "$base_dir/$base_side/trees/additive-scenario-$1" > "$base_dir/$base_side.manifest"
    done
    awk -F'\t' '
        NR == FNR { a[$1] = $2; next }
        { seen[$1] = 1; if (!($1 in a) || a[$1] != $2) d[$1] = 1 }
        END { for (p in a) if (!(p in seen)) d[p] = 1; for (p in d) print p }
    ' "$base_dir/pin.manifest" "$base_dir/head.manifest" | sed 's|^\./||' | LC_ALL=C sort > "$base_dir/differs"
    stale=0
    echo "base: scenario $1 at $2 against HEAD $(git -C "$repo" rev-parse HEAD 2>/dev/null), $(wc -l < "$base_dir/differs" | tr -d ' ') paths differ"
    while IFS= read -r base_path; do
        base_pattern=
        while IFS= read -r base_candidate; do
            # shellcheck disable=SC2254
            case $base_path in
                $base_candidate) base_pattern=$base_candidate; break ;;
            esac
        done < "$base_dir/invalidators"
        if [ -z "$base_pattern" ]; then
            echo "differs: $base_path"
        elif [ "$base_path" = tools/probe/campaign.sh ] \
            && [ -f "$base_dir/pin/trees/additive-scenario-$1/$base_path" ] \
            && [ -f "$base_dir/head/trees/additive-scenario-$1/$base_path" ]; then
            base_line=$(neutral "$base_dir/pin/trees/additive-scenario-$1/$base_path" "$base_dir/head/trees/additive-scenario-$1/$base_path")
            case $? in
                0) echo "neutral: $base_path (every changed line is a comment, a blank or inside a base-neutral region)" ;;
                1) echo "invalidates: $base_path ($base_pattern): $base_line"; stale=1 ;;
                *) rm -rf "$base_dir"; exit 2 ;;
            esac
        else
            echo "invalidates: $base_path ($base_pattern)"
            stale=1
        fi
    done < "$base_dir/differs"
    rm -rf "$base_dir"
}

# plan_check: the whole of `plan`, which `launch` runs first. It sets
# `open_layer`, `open_scenario`, `open_try` and `open_cap`, and exits with
# the status of a refusal.
plan_check() {
    plan_open=$(open_fields) || exit $?
    if [ "$plan_open" = none ]; then
        echo "open: none"
        say "the tuning round is open for no layer: \`open:\` under \`rounds.tuning\` in the declaration is \`none\`, so no try launches."
        exit 5
    fi
    # shellcheck disable=SC2086
    set -- $plan_open
    open_layer=$1 open_scenario=$2 open_try=$3 open_cap=$4
    approvals=$(declared approved) || exit $?
    approved=$(printf '%s\n' "$approvals" | awk '$1 == "tuning" { print $2 }')
    spent=$(spent_round) || exit $?
    echo "open: $open_layer on scenario $open_scenario, try $open_try"
    echo "spent: $spent"
    echo "cap: $open_cap"
    echo "approved: $approved"
    if [ $((spent + open_cap)) -gt "${approved:-0}" ]; then
        say "the round spent $spent cents, and a try of $open_cap more is $((spent + open_cap)), above the $approved cents approved for the tuning round. Nothing is launched."
        exit 5
    fi
    scenarios=$(declared scenarios) || exit $?
    base=$(printf '%s\n' "$scenarios" | awk -v s="$open_scenario" '$1 == s { print $2 }')
    if [ -z "$base" ] || [ "$base" = - ]; then
        say "scenario $open_scenario names no base in the declaration."
        exit 2
    fi
    pin_file=$root/base-pin
    [ -f "$pin_file" ] || { say "no $pin_file. Write one line there: <scenario> <sha>, the base scenario and the commit it was measured at."; exit 2; }
    # shellcheck disable=SC2046
    set -- $(head -1 "$pin_file")
    if [ $# != 2 ] || [ "$1" != "$base" ]; then
        say "$pin_file reads \`$(head -1 "$pin_file")\`, and it must be \`$base <sha>\`, the base of scenario $open_scenario and the commit it was measured at."
        exit 2
    fi
    case $2 in
        *[!0-9a-f]*) say "the pin in $pin_file is not a commit: $2."; exit 2 ;;
    esac
    [ ${#2} -ge 7 ] || { say "the pin in $pin_file is not a commit: $2."; exit 2; }
    base_check "$base" "$2"
    if [ "$stale" = 1 ]; then
        say "the base, scenario $base at $2, is stale: a path above that differs is on base_invalidators. Re-measure the base, or have the owner accept the difference."
        exit 4
    fi
    echo "plan: a try of $open_layer on scenario $open_scenario may launch"
}

# ------------------------------------------------------------ the driver

# alive <try>: 0 when the pid in `driver.pid` runs a driver of that root.
alive() {
    [ -f "$1/driver.pid" ] || return 1
    alive_pid=$(head -1 "$1/driver.pid")
    case $alive_pid in '' | *[!0-9]*) return 1 ;; esac
    kill -0 "$alive_pid" 2>/dev/null || return 1
    tr '\0' ' ' < "/proc/$alive_pid/cmdline" 2>/dev/null | grep -qF -- "--root $1 "
}

# start <try>: the driver detached, with the arguments of `<try>/driver.argv`.
start() {
    start_try=$1
    set --
    while IFS= read -r start_arg; do
        set -- "$@" "$start_arg"
    done < "$start_try/driver.argv"
    command -v setsid >/dev/null 2>&1 || { say "setsid is not on the path."; exit 3; }
    cd "$checkout" || exit 3
    setsid nohup sh "$driver" "$@" >> "$start_try/drive.out" 2>&1 < /dev/null &
    echo $! > "$start_try/driver.pid"
    date -u +%Y-%m-%dT%H:%M:%SZ >> "$start_try/launched"
    echo "launched: $start_try, driver pid $(cat "$start_try/driver.pid")"
}

# pin_ready <pin>: the checkout is in the state the driver's own pin check
# accepts, so that a launch never reports a driver that exits at once.
pin_ready() {
    command -v git >/dev/null 2>&1 || { say "git is not on the path."; exit 3; }
    pin_head=$(git -C "$checkout" rev-parse HEAD 2>/dev/null)
    if [ "$pin_head" != "$1" ]; then
        say "the checkout $checkout is at ${pin_head:-no commit}, not the pin $1."
        exit 6
    fi
    if git -C "$checkout" symbolic-ref -q HEAD >/dev/null 2>&1; then
        say "the checkout $checkout is on a branch, and the driver refuses that. Detach it at the pin: git switch --detach $1."
        exit 6
    fi
    if ! git -C "$checkout" merge-base --is-ancestor "$1" origin/main 2>/dev/null; then
        say "the pin $1 is not on origin/main, and the driver refuses that."
        exit 6
    fi
    if [ -n "$(git -C "$checkout" status --porcelain 2>/dev/null)" ]; then
        say "the checkout $checkout is dirty, and the driver refuses that."
        exit 6
    fi
}

markers() {
    marker_list=
    for marker in canary-recorded canary-passed canary-needs-reader canary-failed a.done capped stop done; do
        [ -e "$1/$marker" ] && marker_list="$marker_list $marker"
    done
    printf '%s\n' "${marker_list# }"
}

# ------------------------------------------------------------ the verbs

case $verb in
    spent)
        spent_round
        ;;

    status)
        for status_try in "$root"/*/ "$root"/.[!.]*/ "$root"/..?*/; do
            [ -d "$status_try" ] || continue
            status_try=${status_try%/}
            status_driver=none
            if [ -f "$status_try/driver.pid" ]; then
                if alive "$status_try"; then status_driver=running; else status_driver=exited; fi
            fi
            echo "${status_try##*/} spent=$(try_spent "$status_try") driver=$status_driver markers=$(markers "$status_try")"
        done
        total=$(spent_round) || exit $?
        approvals=$(declared approved) || exit $?
        open_line=$(declared open) || exit $?
        echo "spent: $total of $(printf '%s\n' "$approvals" | awk '$1 == "tuning" { print $2 }') approved"
        echo "open: $open_line"
        ;;

    plan)
        read_env 0
        plan_check
        ;;

    launch)
        read_env 1
        plan_check
        try=$root/$open_layer-try$open_try
        if [ -e "$try" ]; then
            say "$try exists already, so this is not a first launch. Use relaunch to start its driver again."
            exit 6
        fi
        case $root in
            *"'"* | *[[:space:]]*) say "the round root $root holds a quote or a space, which the canary command cannot carry."; exit 2 ;;
        esac
        command -v git >/dev/null 2>&1 || { say "git is not on the path."; exit 3; }
        pin=$(git -C "$checkout" rev-parse HEAD) || exit 3
        pin_ready "$pin"
        mkdir -p "$try" || exit 2
        printf 'additive scenario-%s tuning\n' "$open_scenario" > "$try/spec"
        printf '%s\n' --root "$try" --pin "$pin" --claude-version "$TRY_HARNESS" \
            --spec-a "$try/spec" --model "$TRY_MODEL" --session-cents "$TRY_SESSION_CENTS" \
            --cap-cents "$open_cap" --parallel "$TRY_PARALLEL" --overshoot-cents "$TRY_OVERSHOOT_CENTS" \
            --seed "$TRY_SEED" --on-canary "sh '$self' canary '$root' '${try##*/}'" > "$try/driver.argv"
        start "$try"
        ;;

    relaunch)
        resolve_try
        [ -d "$try" ] || { say "no try root at $try."; exit 6; }
        for marker in done capped canary-failed stop; do
            if [ -e "$try/$marker" ]; then
                say "$try/$marker is present, so the driver is not started again. A person rules on it first."
                exit 6
            fi
        done
        if alive "$try"; then
            say "the driver of $try runs, as pid $(cat "$try/driver.pid")."
            exit 6
        fi
        [ -s "$try/driver.argv" ] || { say "no $try/driver.argv, so this try was not launched by try.sh."; exit 2; }
        read_env 1
        pin_ready "$(awk 'previous == "--pin" { print; exit } { previous = $0 }' "$try/driver.argv")"
        start "$try"
        ;;

    canary)
        resolve_try
        [ -f "$try/canary-passed" ] && { echo "canary-passed: present in $try"; exit 0; }
        [ -f "$try/canary-recorded" ] || { say "no $try/canary-recorded: the canary has not recorded."; exit 6; }
        # shellcheck disable=SC2046
        set -- $(head -1 "$try/spec" 2>/dev/null)
        case ${2:-} in
            scenario-[0-9]*) canary_scenario=${2#scenario-} ;;
            *) say "$try/spec does not name a scenario."; exit 2 ;;
        esac
        scenarios=$(declared scenarios) || exit $?
        canary_layers=$(printf '%s\n' "$scenarios" | awk -v s="$canary_scenario" '$1 == s { $1 = ""; $2 = ""; print }')
        case " $canary_layers " in
            *" hook "*) hook_expected=live ;;
            *) hook_expected="not live" ;;
        esac
        bound=$((TRY_SESSION_CENTS + TRY_OVERSHOOT_CENTS))
        offences=
        records=0
        for job in $(awk 'NF { print $1 }' "$try/a/jobs" 2>/dev/null); do
            session=$try/a/sessions/$job
            [ -f "$session/status" ] || continue
            records=$((records + 1))
            record=$session/record.md
            status=$(head -1 "$session/status")
            cost=$(head -1 "$session/cost" 2>/dev/null)
            [ "$status" = 0 ] || offences="$offences
$job: status $status"
            case $cost in
                '' | *[!0-9]*) offences="$offences
$job: cost \`$cost\` is not a whole number of cents" ;;
                *) [ "$cost" -le "$bound" ] || offences="$offences
$job: cost $cost above $bound" ;;
            esac
            if [ ! -f "$record" ]; then
                offences="$offences
$job: no record.md"
                continue
            fi
            grep -qF 'The session made 0 calls to a web tool.' "$record" || offences="$offences
$job: web: $(grep -oE 'The session made [0-9]+ calls? to a web tool|The calls this session made to a web tool were not counted' "$record" | head -1)"
            grep -qF 'The session named 0 paths outside its workspace' "$record" || offences="$offences
$job: outside: $(grep -oE 'The session named [0-9]+ paths? outside its workspace|The paths outside its workspace this session named were not counted' "$record" | head -1)"
            grep -qF 'It refused no request of that kind' "$record" || offences="$offences
$job: provider: $(grep -oE 'It refused [^,]* of that kind' "$record" | head -1)"
            grep -qF "The intent hook was $hook_expected in this session." "$record" || offences="$offences
$job: hook: the scenario declares it $hook_expected, and the record says: $(grep -oE 'The intent hook was (not )?live in this session' "$record" | head -1)"
        done
        [ "$records" = 8 ] || offences="$offences
$records records of the canary, not 8"
        if [ -n "$offences" ]; then
            printf '%s\n' "${offences#?}" > "$try/canary-needs-reader"
            echo "canary-needs-reader: $try"
            cat "$try/canary-needs-reader"
            exit 7
        fi
        rm -f "$try/canary-needs-reader"
        echo "8 records, each status 0, cost at most $bound, no web call, no outside path, no refused provider fetch, the hook $hook_expected" > "$try/canary-passed"
        echo "canary-passed: $try"
        ;;

    wait)
        resolve_try
        read_env 0
        wait_root=$(df -Pk / | awk 'NR == 2 { print $4 }')
        wait_try=$(df -Pk "$root" | awk 'NR == 2 { print $4 }')
        if [ "${wait_root:-0}" -lt "$TRY_FLOOR_ROOT_KB" ] || [ "${wait_try:-0}" -lt "$TRY_FLOOR_TRY_KB" ]; then
            [ -d "$try" ] && [ ! -e "$try/stop" ] && echo "disk floor: / ${wait_root}K free, $root ${wait_try}K free" > "$try/stop"
            echo "disk-floor: / ${wait_root}K, $root ${wait_try}K"
            exit 0
        fi
        for marker in done canary-failed capped stop; do
            [ -e "$try/$marker" ] && { echo "$marker"; exit 0; }
        done
        if [ ! -e "$try/canary-passed" ]; then
            [ -e "$try/canary-needs-reader" ] && { echo canary-needs-reader; exit 0; }
            [ -e "$try/canary-recorded" ] && { echo canary-recorded; exit 0; }
        fi
        alive "$try" && exit 1
        echo driver-exited
        ;;
esac
exit 0
