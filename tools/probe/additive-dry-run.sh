# SPDX-License-Identifier: Apache-2.0
#
# The dry run of an additive campaign spec (#1774). `campaign-dry-run.sh`
# sources this file when every line of the spec is of the `additive` tier, and
# this file always ends the run. It reads that file's `root`, `engine`,
# `server`, `declaration`, `work`, `spec`, `asked`, `slice`, `priced`,
# `kept_probes`, `fleiss` and `finish`. It is not run on its own.
#
# A line is `additive scenario-<n> <round> [<probe>...]`. The round names its
# set, its two rates and its tries under `rounds:` in `.headwater/probe.yml`,
# and the probes after the round are left out of the set. Each round's count is
# computed, never typed: `fleiss` of the round's two rates gives the sessions
# a scenario needs, and ceil(n / k) over the k probes of the round's set is the
# repetitions of each probe. A line that leaves probes out runs the others at
# the same count. A round that declares `repetitions` in place of its rates
# (#1835) runs that count of each probe, and its n is the count times k.
# `--repetitions` is refused, because the rounds fix the count.
#
# Each line is planned with `--arm scenario-<n> --set <set>` and its excludes,
# at the round's count where it is below the tier's declared one. A plan the
# ceiling refuses lists no probe, so the probes of a line are the set's
# members less its excludes, read with `declared.py rounds`, and the plan's
# own count of probes must agree with them. A line whose plan counts other
# probes, prints other arms, or is refused for a reason other than the ceiling
# is refused, with 5.
#
# It prints each line, a sum for each round with the most tries the round may
# take, and one total. It prints each round of the spec as approved, with the
# `approved_cents` of the round under `rounds:`, or as not approved, which the
# batch refuses with 5 (#1775). Then it builds the present tree and seals it once, and
# for each scenario of the spec it builds the scenario's tree with
# `ablate.sh --scenario`, prints its `--diff` against its declared base, lists
# the MCP tools where the scenario holds `mcp`, runs `seal.sh --leak --scenario`
# over the tree for the probes of the scenario's lines, and deletes the tree.
# A leak, a tree that is not its delta, an MCP server that lists nothing, or a
# line that pools a probe with a `harness` hit with another probe exits 8.
#
# `--priced` prints `<index> <declared> <round count> <how>` for each line,
# as it does for every other spec, so the batch driver reads the round's count
# from here.
#
# **The tuning round runs in rounds (#1838).** When the tuning round declares
# the fields of its rounds, `declared.py tuning` computes the two counts the
# owner ruled on 2026-10-22: a screened variant runs ceil(24 / k) repetitions
# of each of the k probes of the set, and a confirmation, or the best so far
# measured at the same count, runs ceil(144 / k). Over five probes that is 5
# and 29, so 25 and 145 sessions. A tuning line then runs one stage, which
# `HEADWATER_TUNING_STAGE` names, `screen` or `confirm`, and which
# `tools/probe/try.sh` sets for each driver it starts. With `--priced`, a
# tuning line with no stage exits 2, so no batch runs a tuning line at a
# count nobody chose, and any other stage exits 2 with or without it.
# Without `--priced` and with no stage, a tuning line is priced as one whole
# round: the variants at the screen's count, one confirmation, and the best
# so far at the confirmation's count, which a round runs only where no
# measurement at that count exists. The sum of the round is that, at the
# most rounds a layer may take.

if [ -n "$asked" ]; then
    echo "campaign: an additive spec takes no --repetitions. Each round's count is computed from its rates under \`rounds:\` in .headwater/probe.yml." >&2
    finish 2
fi
python3 "$root/tools/probe/declared.py" "$declaration" rounds > "$work/rounds" || finish $?
python3 "$root/tools/probe/declared.py" "$declaration" scenarios > "$work/scenarios" || finish $?
python3 "$root/tools/probe/declared.py" "$declaration" tuning > "$work/tuning" || finish $?
stage=${HEADWATER_TUNING_STAGE:-}
if [ -s "$work/tuning" ]; then
    case $stage in
        '' | screen | confirm) ;;
        *)
            echo "campaign: HEADWATER_TUNING_STAGE is \`$stage\`, and a tuning line runs the stage \`screen\` or \`confirm\`." >&2
            finish 2
            ;;
    esac
fi
tuning_field() { awk -v f="$1" -v c="$2" '$1 == f { print $c }' "$work/tuning"; }
if [ ! -s "$work/rounds" ]; then
    echo "campaign: $spec is an additive spec, and .headwater/probe.yml declares no \`rounds:\`." >&2
    finish 5
fi

# The count of each round: `<round> <set> <tries> <n> <k> <repetitions>`.
: > "$work/round-counts"
while read -r r_name r_set r_high r_low r_tries r_members; do
    # shellcheck disable=SC2086
    set -- $r_members
    r_k=$#
    r_in="in one try"
    [ "$r_tries" -gt 1 ] && r_in="in at most $r_tries tries"
    # A round that declares its own count (#1835) prints `repetitions <n>` in
    # place of its rates, and its n is that count over the set's probes.
    if [ "$r_high" = repetitions ]; then
        r_reps=$r_low
        r_n=$((r_reps * r_k))
        printf '%s %s %s %s %s %s\n' "$r_name" "$r_set" "$r_tries" "$r_n" "$r_k" "$r_reps" >> "$work/round-counts"
        [ "$priced" = 1 ] || printf 'round %s: the set %s of %s probes at the %s repetitions of each probe the round declares, %s sessions per scenario, %s.\n' \
            "$r_name" "$r_set" "$r_k" "$r_reps" "$r_n" "$r_in"
        continue
    fi
    r_needed=$(fleiss "$r_high" "$r_low")
    r_n=${r_needed#* }
    r_reps=$(awk -v n="$r_n" -v k="$r_k" 'BEGIN { r = n / k; c = int(r); if (c < r) c++; print c }')
    printf '%s %s %s %s %s %s\n' "$r_name" "$r_set" "$r_tries" "$r_n" "$r_k" "$r_reps" >> "$work/round-counts"
    if [ "$r_name" = tuning ] && [ -s "$work/tuning" ]; then
        [ "$priced" = 1 ] || printf 'round %s: the owner ruled %s sessions for a screened variant and %s for a confirmation, so ceil(%s / %s) = %s and ceil(%s / %s) = %s repetitions of each probe of the set %s: %s variants a round, in at most %s rounds a layer.\n' \
            "$r_name" "$(tuning_field screen 2)" "$(tuning_field confirm 2)" "$(tuning_field screen 2)" "$r_k" "$(tuning_field screen 3)" \
            "$(tuning_field confirm 2)" "$r_k" "$(tuning_field confirm 3)" "$r_set" "$(tuning_field variants 2)" "$r_tries"
        continue
    fi
    [ "$priced" = 1 ] || printf 'round %s: the set %s of %s probes, %s against %s, needs %s sessions per scenario with the Fleiss continuity correction, so ceil(%s / %s) = %s repetitions of each probe, %s.\n' \
        "$r_name" "$r_set" "$r_k" "$r_high" "$r_low" "$r_n" "$r_n" "$r_k" "$r_reps" "$r_in"
done < "$work/rounds"

: > "$work/costs"
: > "$work/refused"
: > "$work/probes"
status=0
while IFS= read -r line; do
    # shellcheck disable=SC2086
    set -- $line
    index=$1 tier=$2 arm=$3 round=${4:-}
    shift 3
    [ $# -gt 0 ] && shift
    case $arm in
        scenario-[0-9]|scenario-[1-9][0-9]) ;;
        *)
            [ "$priced" = 1 ] && continue
            printf 'line %s (%s %s %s): an additive line names a `scenario-<n>` arm.\n' "$index" "$tier" "$arm" "$round"
            status=5
            continue
            ;;
    esac
    counts=$(awk -v r="$round" '$1 == r' "$work/round-counts")
    if [ -z "$counts" ]; then
        [ "$priced" = 1 ] && continue
        printf 'line %s (%s %s %s): `rounds:` declares no round `%s`. It declares %s.\n' \
            "$index" "$tier" "$arm" "$round" "$round" "$(awk '{ print $1 }' "$work/round-counts" | tr '\n' ' ' | sed 's/ $//')"
        status=5
        continue
    fi
    # shellcheck disable=SC2086
    set -- $counts "$@"
    set_name=$2 tries=$3 round_n=$4 set_k=$5 reps=$6
    shift 6
    # A tuning line runs one stage of a round (#1838), or is priced as one
    # whole round when no stage is named and nothing runs it.
    staged=
    if [ "$round" = tuning ] && [ -s "$work/tuning" ]; then
        if [ -n "$stage" ]; then
            staged=$stage
            round_n=$(tuning_field "$stage" 2)
            reps=$(tuning_field "$stage" 3)
        elif [ "$priced" = 1 ]; then
            echo "campaign: line $index ($tier $arm $round) runs one stage of a tuning round, and HEADWATER_TUNING_STAGE names none. Launch it with tools/probe/try.sh, which names the stage." >&2
            finish 2
        else
            staged=round
            round_n=$(tuning_field confirm 2)
            reps=$(tuning_field confirm 3)
        fi
    fi
    members=$(awk -v r="$round" '$1 == r { for (i = 6; i <= NF; i++) print $i }' "$work/rounds")
    excludes=
    bad=
    for excluded in "$@"; do
        case "
$members
" in
            *"
$excluded
"*) excludes="$excludes --exclude $excluded" ;;
            *) bad=$excluded ;;
        esac
    done
    if [ -n "$bad" ]; then
        [ "$priced" = 1 ] && continue
        printf 'line %s (%s %s %s) leaves out %s, which the set %s does not hold.\n' "$index" "$tier" "$arm" "$round" "$bad" "$set_name"
        status=5
        continue
    fi
    probes=$(printf '%s\n' "$members" | while read -r member; do
        case " $* " in *" $member "*) ;; *) printf '%s\n' "$member" ;; esac
    done)
    k=$(printf '%s\n' "$probes" | awk 'NF' | wc -l | tr -d ' ')
    if [ "$k" = 0 ]; then
        [ "$priced" = 1 ] && continue
        printf 'line %s (%s %s %s) leaves out every probe of the set %s.\n' "$index" "$tier" "$arm" "$round" "$set_name"
        status=5
        continue
    fi
    # shellcheck disable=SC2086
    "$engine" probe plan --root "$root" --tier "$tier" --arm "$arm" --set "$set_name" $excludes \
        > "$work/plan.L$index" 2>&1
    declared=$(sed -n 's/.* x \([0-9]*\) repetitions = .*/\1/p' "$work/plan.L$index" | head -1)
    if [ -n "$declared" ] && [ "$reps" -gt "$declared" ]; then
        [ "$priced" = 1 ] && continue
        printf 'line %s (%s %s %s): the round needs %s repetitions, above the %s the tier declares, and the plan refuses a raise.\n' \
            "$index" "$tier" "$arm" "$round" "$reps" "$declared"
        status=5
        continue
    fi
    if [ -n "$declared" ] && [ "$reps" -lt "$declared" ]; then
        # shellcheck disable=SC2086
        "$engine" probe plan --root "$root" --tier "$tier" --arm "$arm" --set "$set_name" $excludes \
            --repetitions "$reps" > "$work/plan.L$index" 2>&1
    fi
    arms=$(sed -n 's/^arms: \[\(.*\)\]$/\1/p' "$work/plan.L$index" | head -1 | tr -d ' ')
    if [ "$arms" != "$arm" ]; then
        [ "$priced" = 1 ] && continue
        printf 'line %s (%s %s %s): the plan printed the arms [%s] and not [%s]:\n' "$index" "$tier" "$arm" "$round" "$arms" "$arm"
        sed 's/^/    /' "$work/plan.L$index"
        status=5
        continue
    fi
    refusal=$(sed -n '/^## This run does not start/,$p' "$work/plan.L$index" | sed '1,2d' | awk 'NF' | head -1)
    if [ -n "$refusal" ]; then
        case $refusal in
            *" sessions project "*" against a declared ceiling of "*)
                [ "$priced" = 1 ] || printf 'L%s %s\n' "$index" "$refusal" >> "$work/refused"
                ;;
            *)
                [ "$priced" = 1 ] && continue
                printf 'line %s (%s %s %s) is refused by the plan: %s\n' "$index" "$tier" "$arm" "$round" "$refusal"
                status=5
                continue
                ;;
        esac
    fi
    # The plan's own count of the probes it selected, against the set less the
    # line's excludes as `declared.py` read them.
    counted=$(sed -n 's/^\([0-9][0-9]*\) probes x .*/\1/p' "$work/plan.L$index" | head -1)
    if [ "$counted" != "$k" ]; then
        [ "$priced" = 1 ] && continue
        printf 'line %s (%s %s %s): the plan selects %s probes, and the set %s less the probes the line leaves out holds %s.\n' \
            "$index" "$tier" "$arm" "$round" "${counted:-no}" "$set_name" "$k"
        status=5
        continue
    fi
    how="round $round, ceil($round_n / $set_k)"
    [ -n "$staged" ] && [ "$staged" != round ] && how="round $round, the $staged, ceil($round_n / $set_k)"
    if awk -v r="$round" '$1 == r && $3 == "repetitions" { found = 1 } END { exit !found }' "$work/rounds"; then
        how="round $round, declared"
    fi
    if [ "$priced" = 1 ]; then
        printf '%s %s %s %s\n' "$index" "$declared" "$reps" "$how"
        continue
    fi
    unit=$(sed -n 's/.*at a declared \$\([0-9.]*\) each.*/\1/p' "$work/plan.L$index" | head -1)
    ceiling=$(sed -n 's/.*against a ceiling of \$\([0-9.]*\)\..*/\1/p' "$work/plan.L$index" | head -1)
    unit=$(awk -v d="$unit" 'BEGIN { printf "%d", d * 100 + 0.5 }')
    ceiling=$(awk -v d="$ceiling" 'BEGIN { printf "%d", d * 100 + 0.5 }')
    sessions=$((k * reps))
    cents=$((sessions * unit))
    per=
    [ "$tries" -gt 1 ] && per=" a try"
    if [ "$staged" = round ]; then
        variants=$(tuning_field variants 2)
        screen_reps=$(tuning_field screen 3)
        screen_sessions=$((variants * k * screen_reps))
        sessions=$((screen_sessions + 2 * k * reps))
        cents=$((sessions * unit))
        printf 'line %s  %s %s %s: a round of %s variants x %s probes x %s repetitions (ceil(%s / %s)) = %s sessions, a confirmation of %s probes x %s repetitions (%s) = %s sessions, and the best so far at %s sessions where no measurement at that count exists: %s sessions, $%s a round\n' \
            "$index" "$tier" "$arm" "$round" "$variants" "$k" "$screen_reps" "$(tuning_field screen 2)" "$set_k" "$screen_sessions" \
            "$k" "$reps" "$how" "$((k * reps))" "$((k * reps))" "$sessions" \
            "$(awk -v c="$cents" 'BEGIN { printf "%.2f", c / 100 }')"
    else
        printf 'line %s  %s %s %s: %s probes x %s repetitions (%s) = %s sessions, $%s%s\n' \
            "$index" "$tier" "$arm" "$round" "$k" "$reps" "$how" "$sessions" \
            "$(awk -v c="$cents" 'BEGIN { printf "%.2f", c / 100 }')" "$per"
    fi
    has_cued=0
    has_other=0
    for probe in $probes; do
        printf '%s %s %s\n' "$index" "$arm" "$probe" >> "$work/probes"
        case "
$kept_probes
" in
            *"
$probe
"*) has_cued=1 ;;
            *) has_other=1 ;;
        esac
    done
    if [ "$has_cued" = 1 ] && [ "$has_other" = 1 ]; then
        printf 'line %s pools a probe under `leaks_kept:` with one that is not, so its rate would hold the leak string.\n' "$index"
        [ "$status" = 5 ] || status=8
    fi
    unit_word=tries
    [ "$staged" = round ] && unit_word=rounds
    [ -n "$staged" ] && [ "$staged" != round ] && tries=1
    printf '%s %s %s %s %s %s\n' "$round" "$tries" "$((sessions * tries))" "$((cents * tries))" "$ceiling" "$unit_word" >> "$work/costs"
done < "$work/lines"
[ "$priced" = 1 ] && finish 0

# ---------------------------------------------------------------------------
# The sums: each round at the most tries it may take, and one total.
# ---------------------------------------------------------------------------
if [ -s "$work/costs" ]; then
    awk '
        function d(c) { return sprintf("$%.2f", c / 100) }
        {
            if (!($1 in rs)) order[++nr] = $1
            rs[$1] += $3; rc[$1] += $4; tries[$1] = $2; ceil = $5; word[$1] = $6
            s += $3; c += $4
        }
        END {
            for (i = 1; i <= nr; i++) {
                r = order[i]
                printf "round %s: %d sessions, %s%s\n", r, rs[r], d(rc[r]), (tries[r] > 1 ? " (" tries[r] " " word[r] " at most)" : "")
            }
            printf "total: %d sessions, %s against the %s the additive tier declares\n", s, d(c), d(ceil)
            if (slice != "") printf "slices: %d sessions in %d slices of at most %d\n", s, int((s + slice - 1) / slice), slice
        }
    ' slice="$slice" "$work/costs"
fi
# The approval of each round of the spec (#1775). The batch refuses a line of
# a round whose approval is 0, with 5, before it builds a tree.
python3 "$root/tools/probe/declared.py" "$declaration" approved > "$work/approved" || finish $?
awk '{ print $4 }' "$work/lines" | awk '!seen[$0]++' | while read -r a_round; do
    a_cents=$(awk -v r="$a_round" '$1 == r { print $2 }' "$work/approved")
    if [ "${a_cents:-0}" -gt 0 ]; then
        awk -v r="$a_round" -v c="$a_cents" 'BEGIN { printf "round %s: approved to $%.2f over every attempt, so the batch runs its lines\n", r, c / 100 }'
    else
        printf 'round %s: not approved, so the batch refuses its lines with 5\n' "$a_round"
    fi
done
if [ -s "$work/refused" ]; then
    printf 'The ceiling refuses %s of %s plans, and only a person who agrees to spend more may move it. The first:\n' \
        "$(wc -l < "$work/refused" | tr -d ' ')" "$(wc -l < "$work/lines" | tr -d ' ')"
    head -1 "$work/refused" | sed 's/^/  /'
fi

# ---------------------------------------------------------------------------
# The trees: the present tree once, then each scenario's tree in turn, its
# delta against its base, its MCP tools and its leak check. A scenario tree is
# deleted once checked, so the run holds at most one beside the present tree.
# ---------------------------------------------------------------------------
mkdir -p "$work/present"
git -C "$root" archive "$head" | tar -x -C "$work/present" || finish 3
mkdir -p "$work/present/engine/target/dev-release"
cp "$engine" "$work/present/engine/target/dev-release/headwater" || finish 3
sh "$root/tools/probe/ablate.sh" --present "$work/present" > /dev/null || finish 3
all_probes=$(awk '{ print $3 }' "$work/probes" | sort -u)
if [ -z "$all_probes" ]; then
    printf 'campaign: no line of the spec selected a probe, so no tree was built.\n'
    [ "$status" != 0 ] && finish "$status"
    finish 5
fi
# shellcheck disable=SC2086
sh "$root/tools/probe/seal.sh" "$work/present" $all_probes > /dev/null || finish 3
mkdir -p "$work/config"
tree_status=0
awk '{ print $2 }' "$work/probes" | sort -u | sed 's/^scenario-//' | sort -n > "$work/scenario-list"
while read -r number; do
    arm=scenario-$number
    layers=$(awk -v n="$number" '$1 == n { for (i = 3; i <= NF; i++) printf "%s ", $i }' "$work/scenarios")
    base=$(awk -v n="$number" '$1 == n { print $2 }' "$work/scenarios")
    rm -rf "$work/scenario"
    cp -a "$work/present" "$work/scenario" || finish 3
    if ! sh "$root/tools/probe/ablate.sh" --scenario "$number" "$work/scenario" > /dev/null 2> "$work/build.err"; then
        printf 'tree additive %s did not build:\n' "$arm"
        sed 's/^/    /' "$work/build.err"
        tree_status=8
        continue
    fi
    if [ -z "$base" ] || [ "$base" = - ]; then
        printf 'tree additive %s: plain code, the floor that the scenario of the docs layer is compared with\n' "$arm"
    elif sh "$root/tools/probe/ablate.sh" --diff --scenario "$number" "$work/present" "$base" > "$work/diff" 2>&1; then
        printf 'tree additive %s against scenario-%s: %s\n' "$arm" "$base" "$(tr '\n' ' ' < "$work/diff" | sed 's/ $//')"
    else
        printf 'tree additive %s is not its delta against scenario-%s:\n' "$arm" "$base"
        sed 's/^/    /' "$work/diff"
        tree_status=8
    fi
    case " $layers " in
        *" mcp "*)
            tools=$(printf '%s\n' \
                '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}' \
                '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
                '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
                | "$server" mcp --root "$work/scenario" 2>/dev/null \
                | jq -r 'select(.id == 2) | .result.tools[].name' 2>/dev/null | tr '\n' ' ' | sed 's/ $//')
            if [ -n "$tools" ]; then
                printf 'mcp, %s: `headwater mcp --root <the scenario tree>` lists %s\n' "$arm" "$tools"
            else
                printf 'mcp, %s: `headwater mcp` listed no tool in the scenario tree, so the scenario would measure no server.\n' "$arm"
                tree_status=8
            fi
            ;;
    esac
    scenario_probes=$(awk -v a="$arm" '$2 == a { print $3 }' "$work/probes" | sort -u)
    # shellcheck disable=SC2086
    sh "$root/tools/probe/seal.sh" --leak --config "$work/config" --scenario "$work/scenario" $scenario_probes > "$work/leak" 2>&1
    leak_status=$?
    sed "s/^/leak check, $arm tree: /" "$work/leak"
    [ "$leak_status" = 0 ] || tree_status=8
    # A probe whose answer the harness layer states runs on a line of its own
    # (clause 5): its rate then holds the harness text and no other probe's.
    for probe in $(awk '$1 == "harness" { print $2 }' "$work/leak" | sort -u); do
        for index in $(awk -v a="$arm" -v p="$probe" '$2 == a && $3 == p { print $1 }' "$work/probes" | sort -u); do
            pooled_with=$(awk -v i="$index" '$1 == i' "$work/probes" | wc -l | tr -d ' ')
            if [ "$pooled_with" -gt 1 ]; then
                printf 'line %s pools %s, whose answer the harness layer of the %s tree states, with %s other probes, so its rate would hold the harness text.\n' \
                    "$index" "$probe" "$arm" "$((pooled_with - 1))"
                tree_status=8
            fi
        done
    done
    rm -rf "$work/scenario"
done < "$work/scenario-list"

[ "$status" != 0 ] && finish "$status"
finish "$tree_status"
