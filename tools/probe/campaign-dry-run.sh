#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# The dry run of a campaign spec: what the batch would cost and whether its
# trees are what the declaration says, with no session and no spend (#1472).
#
#     sh tools/probe/campaign.sh --dry-run --spec <file> [--repetitions <n>] \
#         [--max-sessions <n>]
#     sh tools/probe/campaign-dry-run.sh <spec> [<repetitions>] [<max-sessions>]
#     sh tools/probe/campaign-dry-run.sh --priced <spec>
#
# `tools/probe/campaign.sh` states what it prints and its exit statuses. This
# file is the mechanism, kept apart from the batch driver because the batch
# spends money and this never does: it calls no harness, and it reads no
# `claude` from the path.
#
# The spec is the batch driver's: one line per tier, arm, category and the
# probes named out of the category.
#
# **The power calculation.** `power:` in `.headwater/probe.yml` declares two
# rates, a two-sided level and a power. The sessions each arm needs are
#
#     n0 = (z(1 - alpha/2) sqrt(2 pbar qbar) + z(power) sqrt(p1 q1 + p2 q2))^2 / (p1 - p2)^2
#     n  = n0 / 4 (1 + sqrt(1 + 4 / (n0 |p1 - p2|)))^2
#
# where pbar is the mean of the two rates, q is 1 - p, and n is n0 with the
# Fleiss continuity correction. A line of a category that `pooled:` names
# pools its probes into one rate, so it runs ceil(n / k) repetitions of each
# of its k probes. A line of any other category prices at the count its tier
# declares. The plan refuses a repetition count above the declared one, so
# every line is planned at the declared count, for its probes and its
# refusals, and priced here.
#
# **A count given.** `--repetitions <n>` is priced as given on every line, so
# the dry run prices what the batch runs. A count at or below the declared
# one is passed to the plan, a lowering, as the batch driver passes it. A
# count above the declared one is admitted on a line only up to the count
# that line prices at, and the plan is then at the declared count: the
# owner's approval of a priced plan permits that raise and nothing above it
# (#1659). A count above the price is refused, with 5, and the line names
# its price. So a line not under `pooled:` admits no raise.
#
# **The priced counts.** `--priced <spec>` prints `<index> <declared>
# <priced> <how>` for each line it can plan and exits 0, with no header, no
# sum, no tree and no leak check. `campaign.sh` reads it to hold a batch to
# the same rule, so the power calculation has one copy. A line whose plan
# prints no arms or is refused for another reason than the ceiling prints
# nothing, and the batch driver's own plan refuses it.
#
# **The sums.** Beside the sum of each arm and each tier, it prints the sum of
# each tier and category, with its share of the total, so a person can see
# where the plan spends before ruling on it (#1472). A line that holds only
# leak-kept probes is summed apart, as `<category> (leak-kept)`. With a slice
# size it prints how many invocations of `--max-sessions` the batch takes.

set -u

priced=0
if [ "${1:-}" = --priced ]; then
    priced=1
    shift
fi
spec=${1:-}
asked=${2:-}
slice=${3:-}
case $slice in
    '') ;;
    *[!0-9]*|0) echo "campaign: --max-sessions takes a positive whole number, not \`$slice\`." >&2; exit 2 ;;
esac
case $asked in
    '') ;;
    *[!0-9]*|0*) echo "campaign: --repetitions takes a positive whole number, not \`$asked\`." >&2; exit 2 ;;
esac

case $0 in
    */*) invoked_from=${0%/*} ;;
    *) invoked_from=. ;;
esac
root=$(cd "$invoked_from/../.." && pwd -P)
engine=$root/engine/target/dev-release/headwater
[ -x "$engine" ] || engine=$root/engine/target/release/headwater
# The batch driver names the engine its own plans read, so the priced counts
# come from the binary that plans the batch.
engine=${HW_CAMPAIGN_ENGINE:-$engine}
# The server the `mcp` arm starts is this engine's, unless `HW_PROBE_ENGINE`
# names another binary: for the fixtures alone, which need a server that
# lists nothing. Every plan reads this checkout's engine.
server=${HW_PROBE_ENGINE:-$engine}
# `HW_PROBE_YML` names another declaration for the power, the kept leaks,
# the arms' deltas and the leak strings, as it does for `ablate.sh` and
# `seal.sh`, for the fixtures alone. The plans read this checkout's.
declaration=${HW_PROBE_YML:-$root/.headwater/probe.yml}

[ -n "$spec" ] || { echo "usage: campaign.sh --dry-run --spec <file> [--repetitions <n>]" >&2; exit 2; }
[ -f "$spec" ] || { echo "campaign: no spec at $spec" >&2; exit 2; }
for tool in git jq tar awk diff python3; do
    command -v "$tool" >/dev/null 2>&1 || { echo "campaign: \`$tool\` is not on the path." >&2; exit 3; }
done
[ -x "$engine" ] || { echo "campaign: no engine is built at $root/engine/target." >&2; exit 3; }

work=$(mktemp -d "${TMPDIR:-/tmp}/headwater-dry-run.XXXXXX") || exit 3
finish() {
    rm -rf "$work"
    exit "$1"
}

head=$(git -C "$root" rev-parse HEAD) || finish 3
if [ "$priced" = 0 ]; then
    printf 'campaign: dry run of %s at %s. No session starts and nothing is spent.\n' "$spec" "$head"
    if [ -n "$(git -C "$root" status --porcelain --untracked-files=no)" ]; then
        printf 'campaign: the checkout has uncommitted changes. The plans read this checkout and the trees are HEAD.\n'
    fi
fi

# ---------------------------------------------------------------------------
# The power calculation.
# ---------------------------------------------------------------------------
# `power:` is read with PyYAML by `tools/probe/declared.py`, which refuses a
# rate that is not a number (verify round 5: a hand-written reader read a
# quoted `"0.15"` as 0 and priced the plan at exit 0).
python3 "$root/tools/probe/declared.py" "$declaration" power > "$work/power" || finish $?
power_field() { sed -n "s/^$1 //p" "$work/power"; }
p1=$(power_field present)
p2=$(power_field absent)
alpha=$(power_field alpha)
power=$(power_field power)
pooled=$(sed -n 's/^pooled *//p' "$work/power")
needed=$(awk -v p1="$p1" -v p2="$p2" -v alpha="$alpha" -v power="$power" '
    # The inverse of the standard normal distribution (Acklam), to about 1e-9.
    function qnorm(p,    q, r) {
        if (p < 0.02425) {
            q = sqrt(-2 * log(p))
            return (((((-7.784894002430293e-03 * q - 3.223964580411365e-01) * q - 2.400758277161838e+00) * q - 2.549732539343734e+00) * q + 4.374664141464968e+00) * q + 2.938163982698783e+00) / ((((7.784695709041462e-03 * q + 3.224671290700398e-01) * q + 2.445134137142996e+00) * q + 3.754408661907416e+00) * q + 1)
        }
        if (p > 1 - 0.02425) return -qnorm(1 - p)
        q = p - 0.5
        r = q * q
        return (((((-3.969683028665376e+01 * r + 2.209460984245205e+02) * r - 2.759285104469687e+02) * r + 1.383577518672690e+02) * r - 3.066479806614716e+01) * r + 2.506628277459239e+00) * q / (((((-5.447609879822406e+01 * r + 1.615858368580409e+02) * r - 1.556989798598866e+02) * r + 6.680131188771972e+01) * r - 1.328068155288572e+01) * r + 1)
    }
    function ceil(x,    n) { n = int(x); return (n < x) ? n + 1 : n }
    BEGIN {
        za = qnorm(1 - alpha / 2)
        zb = qnorm(power)
        pbar = (p1 + p2) / 2
        d = p1 - p2
        if (d < 0) d = -d
        n0 = (za * sqrt(2 * pbar * (1 - pbar)) + zb * sqrt(p1 * (1 - p1) + p2 * (1 - p2))) ^ 2 / d ^ 2
        n = n0 / 4 * (1 + sqrt(1 + 4 / (n0 * d))) ^ 2
        printf "%d %d\n", ceil(n0), ceil(n)
    }')
n0=${needed% *}
n=${needed#* }
[ "$priced" = 1 ] || printf 'power: %s against %s at a two-sided level of %s and a power of %s needs %s sessions per arm, %s with the Fleiss continuity correction. A line of %s pools its probes into one rate and runs ceil(%s / k) repetitions of each of its k probes.\n' \
    "$p1" "$p2" "$alpha" "$power" "$n0" "$n" "${pooled:-no category}" "$n"

# The probes kept on their own line (`leaks_kept:`), read with the parser
# `tools/probe/leak.py` reads them with, so the pooling gate and the leak check
# agree on every YAML form of the key (verify round 5).
kept_probes=$(python3 "$root/tools/probe/declared.py" "$declaration" leaks_kept) || finish $?

# ---------------------------------------------------------------------------
# The plan of every line.
# ---------------------------------------------------------------------------
awk 'NF && $1 !~ /^#/' "$spec" | awk '{ print NR, $0 }' > "$work/lines"
: > "$work/costs"
: > "$work/refused"
: > "$work/probes"
status=0
while IFS= read -r line; do
    # shellcheck disable=SC2086
    set -- $line
    index=$1 tier=$2 arm=$3 category=$4
    shift 4
    excludes=
    for excluded in "$@"; do
        excludes="$excludes --exclude $excluded"
    done
    # Every line is planned at the tier's declared count first, which is the
    # one plan when no count is given. A count at or below the declared one
    # is a lowering, and the line is planned again at it.
    # shellcheck disable=SC2086
    "$engine" probe plan --root "$root" --tier "$tier" --category "$category" $excludes \
        > "$work/plan.L$index" 2>&1
    declared=$(sed -n 's/.* x \([0-9]*\) repetitions = .*/\1/p' "$work/plan.L$index" | head -1)
    if [ "$priced" = 0 ] && [ -n "$asked" ] && [ -n "$declared" ] && [ "$asked" -le "$declared" ]; then
        # shellcheck disable=SC2086
        "$engine" probe plan --root "$root" --tier "$tier" --category "$category" $excludes \
            --repetitions "$asked" > "$work/plan.L$index" 2>&1
    fi
    if ! grep -q '^arms: \[' "$work/plan.L$index"; then
        [ "$priced" = 1 ] && continue
        printf 'line %s (%s %s %s): the plan printed no arms:\n' "$index" "$tier" "$arm" "$category"
        sed 's/^/    /' "$work/plan.L$index"
        status=5
        continue
    fi
    arms=$(sed -n 's/^arms: \[\(.*\)\]$/\1/p' "$work/plan.L$index" | head -1 | tr -d ' ' | tr ',' ' ')
    case " $arms " in
        *" $arm "*) ;;
        *)
            [ "$priced" = 1 ] && continue
            printf 'line %s (%s %s %s): the tier runs the arms %s and not `%s`.\n' "$index" "$tier" "$arm" "$category" "$arms" "$arm"
            status=5
            continue
            ;;
    esac
    refusal=$(sed -n '/^## This run does not start/,$p' "$work/plan.L$index" | sed '1,2d' | awk 'NF' | head -1)
    if [ -n "$refusal" ]; then
        case $refusal in
            *" sessions project "*" against a declared ceiling of "*)
                [ "$priced" = 1 ] || printf 'L%s %s\n' "$index" "$refusal" >> "$work/refused"
                ;;
            *)
                [ "$priced" = 1 ] && continue
                printf 'line %s (%s %s %s) is refused by the plan: %s\n' "$index" "$tier" "$arm" "$category" "$refusal"
                status=5
                continue
                ;;
        esac
    fi
    # A plan the ceiling refuses lists no probe, so the selection is read from
    # the same plan at one repetition, which the ceiling does not refuse.
    listing=$work/plan.L$index
    if [ -n "$refusal" ]; then
        listing=$work/list.L$index
        # shellcheck disable=SC2086
        "$engine" probe plan --root "$root" --tier "$tier" --category "$category" $excludes \
            --repetitions 1 > "$listing" 2>&1
    fi
    probes=$(sed -n 's/^- \(HW-PROBE-[^ ]*\) (.*/\1/p' "$listing")
    k=$(printf '%s\n' "$probes" | awk 'NF' | wc -l | tr -d ' ')
    # The count this line prices at when no count is given: the powered
    # count for a pooled category, and the tier's declared count otherwise.
    price=$declared
    price_how="the tier's $declared"
    case " $pooled " in
        *" $category "*)
            if [ "$k" -gt 0 ]; then
                price=$(awk -v n="$n" -v k="$k" 'BEGIN { r = n / k; c = int(r); if (c < r) c++; print c }')
                price_how="ceil($n / $k), powered"
            fi
            ;;
    esac
    if [ "$priced" = 1 ]; then
        printf '%s %s %s %s\n' "$index" "$declared" "$price" "$price_how"
        continue
    fi
    # A count above the declared one is admitted only up to the price (#1659).
    if [ -n "$asked" ] && [ "$asked" -gt "$declared" ] && [ "$asked" -gt "$price" ]; then
        printf 'line %s (%s %s %s): --repetitions %s is above the %s the dry run prices for it (%s).\n' \
            "$index" "$tier" "$arm" "$category" "$asked" "$price" "$price_how"
        status=5
        continue
    fi
    unit=$(sed -n 's/.*at a declared \$\([0-9.]*\) each.*/\1/p' "$work/plan.L$index" | head -1)
    ceiling=$(sed -n 's/.*against a ceiling of \$\([0-9.]*\)\..*/\1/p' "$work/plan.L$index" | head -1)
    unit=$(awk -v d="$unit" 'BEGIN { printf "%d", d * 100 + 0.5 }')
    ceiling=$(awk -v d="$ceiling" 'BEGIN { printf "%d", d * 100 + 0.5 }')
    # A count given is the count priced, so the dry run prices what the batch
    # runs. Before #1659 a pooled line was priced at the powered count even
    # when a count was given.
    if [ -n "$asked" ]; then
        reps=$asked
        how=asked
        [ "$asked" -gt "$declared" ] && how="asked; the plan is at the tier's $declared"
    else
        reps=$price
        how=$price_how
        [ "$price_how" != "the tier's $declared" ] && how="$price_how; the plan is at the tier's $declared"
    fi
    sessions=$((k * reps))
    cents=$((sessions * unit))
    printf 'line %s  %s %s %s: %s probes x %s repetitions (%s) = %s sessions, $%s\n' \
        "$index" "$tier" "$arm" "$category" "$k" "$reps" "$how" "$sessions" \
        "$(awk -v c="$cents" 'BEGIN { printf "%.2f", c / 100 }')"

    # A leak-kept probe is reported on its own line and never pooled with one that
    # is not (#1472).
    has_cued=0
    has_other=0
    for probe in $probes; do
        printf '%s %s %s\n' "$index" "$tier" "$probe" >> "$work/probes"
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
    elif [ "$has_cued" = 1 ]; then
        printf 'line %s holds only leak-kept probes, and is reported on its own line.\n' "$index"
    fi
    group=$category
    [ "$has_cued" = 1 ] && [ "$has_other" = 0 ] && group="$category (leak-kept)"
    printf '%s %s %s %s %s %s\n' "$tier" "$arm" "$sessions" "$cents" "$ceiling" "$group" >> "$work/costs"
done < "$work/lines"
[ "$priced" = 1 ] && finish 0

# ---------------------------------------------------------------------------
# The sums, by arm and by tier, against each tier's ceiling.
# ---------------------------------------------------------------------------
if [ -s "$work/costs" ]; then
    awk '
        function d(c) { return sprintf("$%.2f", c / 100) }
        {
            key = $1 " " $2
            if (!(key in as)) order[++na] = key
            as[key] += $3; ac[key] += $4
            if (!($1 in ts)) torder[++nt] = $1
            ts[$1] += $3; tc[$1] += $4; ceil[$1] = $5
            group = $6
            for (f = 7; f <= NF; f++) group = group " " $f
            gkey = $1 " " group
            if (!(gkey in gs)) gorder[++ng] = gkey
            gs[gkey] += $3; gc[gkey] += $4
            s += $3; c += $4
        }
        END {
            for (i = 1; i <= na; i++) printf "arm %s: %d sessions, %s\n", order[i], as[order[i]], d(ac[order[i]])
            for (i = 1; i <= ng; i++) {
                g = gorder[i]
                printf "category %s: %d sessions, %s, %.1f%% of the total\n", g, gs[g], d(gc[g]), (c > 0 ? 100 * gc[g] / c : 0)
            }
            for (i = 1; i <= nt; i++) {
                t = torder[i]
                over = tc[t] - ceil[t]
                printf "tier %s: %d sessions, %s against a ceiling of %s: %s\n", t, ts[t], d(tc[t]), d(ceil[t]), (over > 0 ? "over by " d(over) : "inside it")
                cs += ceil[t]
            }
            printf "total: %d sessions, %s against the %s the tiers declare\n", s, d(c), d(cs)
            if (slice != "") printf "slices: %d sessions in %d slices of at most %d\n", s, int((s + slice - 1) / slice), slice
        }
    ' slice="$slice" "$work/costs"
fi
if [ -s "$work/refused" ]; then
    printf 'The ceiling refuses these plans, and only a person who agrees to spend more may move it:\n'
    sed 's/^/  /' "$work/refused"
fi

# ---------------------------------------------------------------------------
# The trees: the present tree once, and the delta of every arm against it.
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
tree_status=0
awk '{ print $2, $3 }' "$work/lines" | sort -u > "$work/arms"
while read -r tier arm; do
    [ "$arm" = present ] && continue
    if sh "$root/tools/probe/ablate.sh" --diff "$tier" "$arm" "$work/present" > "$work/diff" 2>&1; then
        printf 'tree %s %s: %s\n' "$tier" "$arm" "$(tr '\n' ' ' < "$work/diff" | sed 's/ $//')"
    else
        printf 'tree %s %s is not its delta:\n' "$tier" "$arm"
        sed 's/^/    /' "$work/diff"
        tree_status=8
    fi
done < "$work/arms"

# The MCP arm: the server starts in the arm's tree and lists its tools.
leak_trees="$work/present"
if grep -q ' mcp$' "$work/arms"; then
    mcp_tier=$(awk '$2 == "mcp" { print $1; exit }' "$work/arms")
    cp -a "$work/present" "$work/mcp" || finish 3
    sh "$root/tools/probe/ablate.sh" "$mcp_tier" "$work/mcp" mcp > /dev/null || finish 3
    tools=$(printf '%s\n' \
        '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}' \
        '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
        '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
        | "$server" mcp --root "$work/mcp" 2>/dev/null \
        | jq -r 'select(.id == 2) | .result.tools[].name' 2>/dev/null | tr '\n' ' ' | sed 's/ $//')
    if [ -n "$tools" ]; then
        printf 'mcp: `headwater mcp --root <the mcp tree>` lists %s\n' "$tools"
    else
        printf 'mcp: `headwater mcp` listed no tool in the mcp tree, so the arm would measure no server.\n'
        tree_status=8
    fi
    leak_trees="$leak_trees $work/mcp"
fi

# The leak check, over the present tree and the mcp arm's tree. Each session
# of a batch runs under a configuration directory of its own, made empty by
# `campaign.sh` and holding only a copy of the credentials while the session
# runs (#1467). The check reads one made the same way, so it reads the host
# level a session meets, and that is nothing.
mkdir -p "$work/config"
for tree in $leak_trees; do
    # shellcheck disable=SC2086
    sh "$root/tools/probe/seal.sh" --leak --config "$work/config" "$tree" $all_probes > "$work/leak" 2>&1
    leak_status=$?
    label=present
    [ "$tree" = "$work/mcp" ] && label=mcp
    sed "s/^/leak check, $label tree: /" "$work/leak"
    [ "$leak_status" = 0 ] || tree_status=8
done

[ "$status" != 0 ] && finish "$status"
finish "$tree_status"
