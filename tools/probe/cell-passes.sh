#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# The graded counts the saturation stop of phase B reads (#1875). Phase B of
# the additive campaign (HW-DR-0112, "Phase B: the report") stops a cell at
# 24 sessions when all its sessions pass or all fail, and a cell is one row,
# the probe's `action` facet, in one scenario. `tools/probe/campaign-drive.sh
# --saturate-at <n>` calls this once after batch A, and once for each line of
# the spec of batch B. Run it from the checkout the driver runs in.
#
#     sh tools/probe/cell-passes.sh counts <batch-dir>
#     sh tools/probe/cell-passes.sh probes <tier> <arm> <round> [<exclude>...]
#
# `counts` assembles the batch with `campaign.sh --out <batch-dir>
# --assemble`, and prints one line for each probe of each additive line of
# the batch:
#
#     <arm> <set> <action> <probe> <satisfied> <graded>
#
# The two numbers are the grader's, never a recorder status. For each line
# it wraps the assembled identity and events blocks in a bare transcript,
# `<batch-dir>/assembled/<stem>.transcript.md`, and reads
# `headwater probe compare <t> <t>`, which grades each transcript as `grade`
# grades one and states `<probe>: <x>/<n> against ...`, the satisfied over
# the graded sessions (docs/interfaces/headwater-probe.md). A refused
# session is in no denominator. The grader's verdict on a probe that names
# an oracle rule is the oracle's verdict, because the recorder writes the
# oracle's finding on every artifact. A probe the transcript graded no
# session of prints `0 0`. The transcript is written outside the checkout,
# and `headwater new probe_transcript` is not run, because the grade reads
# the two blocks alone: on 2026-10-30 `probe compare` graded such a file
# under the scratch directory of #1875 with the counts of the committed
# transcript it was cut from. A transcript the intake refuses, or an
# assembly that exits other than 0, exits 4 and prints nothing.
#
# `probes` prints one line for each probe that one spec line selects:
#
#     <arm> <set> <probe> <action>
#
# from `headwater probe plan --tier additive --arm <arm> --set <set>` with
# each `--exclude`, where the set is the round's under `rounds:` in
# `.headwater/probe.yml`, read with `tools/probe/declared.py` as
# `campaign.sh` reads it. The action is the one the plan prints. A line of
# another tier prints nothing. A plan that prints no selection exits 4.
#
# `HEADWATER_CELLS_ENGINE` names the engine, by default the one `campaign.sh`
# builds: `dev-release/headwater` under `CARGO_TARGET_DIR`, or under
# `engine/target`. `HEADWATER_CELLS_CAMPAIGN` names `campaign.sh`. The
# fixtures are a section of `tools/probe/campaign-drive-fixtures.sh`.
#
# Exit status: 0 on success, 2 a usage error, 3 no engine, 4 an assembly,
# a grade or a plan failed.

set -u

usage() {
    echo "usage: sh tools/probe/cell-passes.sh counts <batch-dir> | probes <tier> <arm> <round> [<exclude>...]" >&2
    exit 2
}

target=${CARGO_TARGET_DIR:-$PWD/engine/target}
case $target in
    /*) ;;
    *) target=$PWD/$target ;;
esac
engine=${HEADWATER_CELLS_ENGINE:-$target/dev-release/headwater}
campaign=${HEADWATER_CELLS_CAMPAIGN:-$PWD/tools/probe/campaign.sh}
[ -x "$engine" ] || { echo "cell-passes: no engine at $engine." >&2; exit 3; }

# probes <tier> <arm> <round> [<exclude>...]
probes() {
    [ "$1" = additive ] || return 0
    arm=$2 round=$3
    shift 3
    set_name=$(python3 "$PWD/tools/probe/declared.py" "$PWD/.headwater/probe.yml" rounds \
        | awk -v r="$round" '$1 == r { print $2 }')
    [ -n "$set_name" ] || { echo "cell-passes: .headwater/probe.yml declares no round $round." >&2; return 4; }
    excludes=
    for excluded in "$@"; do
        excludes="$excludes --exclude $excluded"
    done
    # shellcheck disable=SC2086
    plan=$("$engine" probe plan --root "$PWD" --no-color --tier additive --arm "$arm" --set "$set_name" $excludes 2>&1)
    printf '%s\n' "$plan" | grep -q '^## The selection$' || {
        echo "cell-passes: the plan of $arm $round selected nothing: $(printf '%s\n' "$plan" | head -1)" >&2
        return 4
    }
    printf '%s\n' "$plan" | awk -v arm="$arm" -v set="$set_name" '
        /^## / { on = ($0 == "## The selection") ; next }
        on && /^- HW-PROBE-/ { probe = $2; next }
        on && /^    action: / && probe != "" { print arm, set, probe, $2; probe = "" }
    '
}

# counts <batch-dir>
counts() {
    batch=$1
    [ -f "$batch/lines" ] || { echo "cell-passes: no batch at $batch." >&2; return 4; }
    sh "$campaign" --out "$batch" --assemble > "$batch/cells.assemble.out" 2> "$batch/cells.assemble.err"
    status=$?
    [ "$status" = 0 ] || { echo "cell-passes: campaign.sh --assemble on $batch exited $status: $(tail -1 "$batch/cells.assemble.err")" >&2; return 4; }
    : > "$batch/cells.part"
    while IFS= read -r row; do
        set -f
        # shellcheck disable=SC2086
        set -- $row
        set +f
        index=$1 tier=$2 arm=$3 round=$4
        [ "$tier" = additive ] || continue
        shift 4
        stem=$batch/assembled/L$index-$tier-$arm-$round
        [ -f "$stem.identity.yaml" ] && [ -f "$stem.events.yaml" ] || { echo "cell-passes: the assembly wrote no blocks for $stem." >&2; return 4; }
        {
            printf '## Run identity\n\n```yaml\n'
            cat "$stem.identity.yaml"
            printf '```\n\n## Events\n\n```yaml\n'
            cat "$stem.events.yaml"
            printf '```\n'
        } > "$stem.transcript.md"
        graded=$("$engine" probe compare --root "$PWD" "$stem.transcript.md" "$stem.transcript.md" 2>&1)
        case $graded in
            HW-PROBE-* | pooled:*) ;;
            *) echo "cell-passes: the grade of $stem refused it: $(printf '%s\n' "$graded" | head -1)" >&2; return 4 ;;
        esac
        selected=$(probes "$tier" "$arm" "$round" "$@" < /dev/null) || return 4
        printf '%s\n' "$graded" | awk -v selected="$selected" '
            BEGIN {
                n = split(selected, rows, "\n")
                for (i = 1; i <= n; i++) { split(rows[i], f, " "); probe[i] = f[3]; line[i] = f[1] " " f[2] " " f[4] " " f[3] }
            }
            /^HW-PROBE-[^:]*: [0-9]+\/[0-9]+ against / {
                name = substr($1, 1, length($1) - 1)
                split($2, xn, "/")
                x[name] = xn[1]; g[name] = xn[2]
            }
            END { for (i = 1; i <= n; i++) if (probe[i] != "") printf "%s %d %d\n", line[i], x[probe[i]], g[probe[i]] }
        ' >> "$batch/cells.part"
    done < "$batch/lines"
    cat "$batch/cells.part"
}

[ $# -ge 1 ] || usage
verb=$1
shift
case $verb in
    counts) [ $# = 1 ] || usage; counts "$1" ;;
    probes) [ $# -ge 3 ] || usage; probes "$@" ;;
    *) usage ;;
esac
