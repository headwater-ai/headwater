#!/bin/sh
# The capture rate of the shadow-mode routing log over one UTC day, and a test
# that the prompts it lost were lost at random.
#
# HW-DR-0064's evidence is the shadow log that `.claude/hooks/intent.sh`
# writes, one line per prompt, and #917 found that the log missed whole classes
# of prompt. Every figure on that issue was computed by hand, with a join stated
# in its comments, and #927's mining needs the same join again. This is that
# join as a tool, so a later reading counts the way the earlier ones did.
#
#     sh tools/run/shadow-capture.sh <YYYY-MM-DD>
#
# The population is person prompts, the one HW-DR-0064 sets. A person prompt
# is a harness transcript record under the projects directory, in a directory
# named `-home-james-projects-headwater*` on this host (any directory whose
# name starts with the encoded path of this checkout's main tree, in general),
# with `type` "user", `isSidechain` false and `origin.kind` "human". It is
# counted once per `promptId`, and only when its timestamp falls in
# [<day>T00:00:00Z, <day+1>T00:00:00Z). A record whose timestamp does not
# parse is counted on its own line and left out.
#
# The population is restricted to checkouts that held a built engine, because
# the hook cannot write a line without one, and Done-when 5 of #917 put such a
# prompt outside the population. The checkout is the directory the session
# started in, which is where the hook looks for its engine, and it held one
# when `engine/target/release/headwater` or
# `engine/target/dev-release/headwater` is there now. A directory that is gone
# cannot be read either way, so its prompts are left out and counted on their
# own line: a reading taken after a worktree is retired shrinks by that line
# and says so.
#
# A prompt is joined when a distinct `prompt_id` in any file of the log
# directory equals its `promptId`. The join is on distinct ids and never on
# lines, because the hook has written one prompt twice (#917, 2026-09-29), and
# a line that does not parse is skipped rather than ending the read. A line
# with a non-null `skip` is joined, because it is a prompt the hook saw and
# named a reason for; it is counted again on its own line.
#
# Two tests that the misses are random, with the null hypothesis, the
# threshold and the statistic stated before the data are read:
#
#   sessions  H0: a miss is independent of the session the prompt is in.
#             Pearson's chi-square on the 2 x k table of joined and missed
#             prompts by session, df = k - 1. It asks whether the losses sit in
#             a few sessions. It needs at least one miss, one hit and two
#             sessions.
#   runs      H0: within each session, hits and misses in time order are
#             exchangeable. A Wald-Wolfowitz runs test, pooled over sessions:
#             the observed runs, their expectation and variance summed over
#             sessions, z under the normal approximation, and the lower tail,
#             because too few runs is a stretch of misses. It asks whether the
#             losses sit in stretches, such as a silent hour. It needs a
#             session with both outcomes.
#
# The family-wise threshold is 0.05, so each test rejects at p < 0.025
# (Bonferroni over two). Randomness is rejected when either test rejects. A
# test whose data cannot speak prints `n/a` and why, and does not reject.
# Both approximations are weak at small counts: chi-square wants an expected
# count of five a cell, which a day of this log does not give, so a `rejected`
# names where to look for a cause and is not by itself the cause.
#
# Every figure goes to standard output, one `name: value` per line, and the
# whole output is the same bytes for the same inputs. A non-zero exit is an
# argument or a read the tool could not make.
#
# Two variables point it elsewhere, which the fixtures use:
#     HEADWATER_TRANSCRIPT_DIRS   space-separated transcript directories
#                                 (default: ~/.claude/projects/<prefix>*)
#     HEADWATER_SHADOW_LOG_DIR    the log directory, as the hook reads it
#                                 (default: <git common dir>/headwater-shadow-log)
#
# It needs `jq` and `awk`, and exits 3 without `jq`.
#
# `sh tools/run/shadow-capture-fixtures.sh` holds it.

set -u

usage() {
    echo "usage: sh tools/run/shadow-capture.sh <YYYY-MM-DD>" >&2
    exit 2
}

[ $# -eq 1 ] || usage
day=$1
case $day in
    [0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]) ;;
    *) usage ;;
esac

if ! command -v jq >/dev/null 2>&1; then
    echo "shadow-capture: no \`jq\` on the path" >&2
    exit 3
fi

# The day's bounds as epoch seconds, from jq rather than `date -d`, which is
# not portable. A day that does not exist fails here.
start=$(jq -n --arg d "$day" '"\($d)T00:00:00Z" | fromdateiso8601') || {
    echo "shadow-capture: not a date: $day" >&2
    exit 2
}
end=$((start + 86400))

here=$(cd "$(dirname "$0")/../.." && pwd)

log=${HEADWATER_SHADOW_LOG_DIR:-}
if [ -z "$log" ]; then
    common=$(git -C "$here" rev-parse --git-common-dir 2>/dev/null) || {
        echo "shadow-capture: not inside a git checkout, and no HEADWATER_SHADOW_LOG_DIR" >&2
        exit 2
    }
    case $common in
        /*) ;;
        *) common="$here/$common" ;;
    esac
    log="$common/headwater-shadow-log"
fi
if [ ! -d "$log" ]; then
    echo "shadow-capture: no log directory at $log" >&2
    exit 2
fi

dirs=${HEADWATER_TRANSCRIPT_DIRS:-}
if [ -z "$dirs" ]; then
    # The harness names a project directory by its path with every `/` and `.`
    # made `-`. The main tree is the parent of the common dir.
    main=$(cd "$(dirname "$log")/.." && pwd)
    prefix=$(printf '%s' "$main" | tr '/.' '--')
    for d in "$HOME/.claude/projects/$prefix"*; do
        [ -d "$d" ] && dirs="$dirs $d"
    done
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

# Every line of the log: its prompt id, whether it carried a `skip`, its
# session, and its text as a JSON string so a tab or a newline in it cannot
# split a field. `fromjson?` skips a line that does not parse; `jq` without it
# stops at the first one and drops the rest of the file.
: > "$scratch/log.tsv"
for f in "$log"/*.jsonl; do
    [ -f "$f" ] || continue
    jq -R -r 'fromjson? | select(type == "object") | [(.prompt_id // "" | tostring), (if .skip == null then "-" else "skip" end), (.session // "" | tostring), ((.task // "") | tostring | tojson)] | @tsv' "$f" >> "$scratch/log.tsv"
done

# Every person prompt of the day: epoch, prompt id, session, and the directory
# the session started in. A timestamp that will not parse is written as `bad`.
#
# The start directory is the `cwd` of the transcript's first record that has
# one. It is the harness's project directory, the one the hook reads its
# engine from (`hw_engine` in `.claude/hooks/lib.sh`, pinned before the root
# is corrected to the prompt's own `cwd`). A session that enters a worktree
# with no build of its own still logs through the start directory's engine,
# so the prompt's own `cwd` is the wrong place to look (2026-09-30: 42 of 45
# person prompts came from worktrees with no `engine/target`, and all 45 were
# logged).
: > "$scratch/records.tsv"
for d in $dirs; do
    for f in "$d"/*.jsonl; do
        [ -f "$f" ] || continue
        startdir=$(jq -R -r 'fromjson? | select(type == "object") | .cwd // empty' "$f" 2>/dev/null | head -n 1)
        jq -R -r --arg start "$startdir" '
            fromjson? | select(type == "object")
            | select(.type == "user" and (.isSidechain | not) and (.origin.kind? // null) == "human")
            | select((.promptId // "") != "")
            | ((.timestamp // "") | tostring | sub("\\.[0-9]+"; "") | (try fromdateiso8601 catch "bad")) as $t
            | (.message.content? // "" | if type == "string" then . elif type == "array" then (map(select(type == "object" and .type == "text") | .text) | join("\n")) else "" end) as $text
            | [($t | tostring), .promptId, (.sessionId // ""), $start, ($text | tojson)] | @tsv' "$f" >> "$scratch/records.tsv"
    done
done

# Whether each start directory held an engine, by the rule `hw_engine` uses:
# `engine`, `none`, or `gone` for a directory that no longer exists.
cut -f4 "$scratch/records.tsv" | sort -u > "$scratch/starts"
: > "$scratch/held.tsv"
while IFS= read -r dir; do
    if [ -z "$dir" ] || [ ! -d "$dir" ]; then
        state=gone
    elif [ -x "$dir/engine/target/release/headwater" ] || [ -x "$dir/engine/target/dev-release/headwater" ]; then
        state=engine
    else
        state=none
    fi
    printf '%s\t%s\n' "$dir" "$state" >> "$scratch/held.tsv"
done < "$scratch/starts"

# Time order, then id, so the first record of a prompt is the one counted and
# the runs test reads each session in the order its prompts were typed.
tab=$(printf '\t')
LC_ALL=C sort -t "$tab" -k1,1n -k2,2 "$scratch/records.tsv" > "$scratch/sorted.tsv"

awk -F '\t' -v start="$start" -v end="$end" -v day="$day" '
function lgamma(x,   c, s, i, t) {
    # Lanczos, g = 7, nine coefficients; good to about 1e-15 for x > 0.
    split("0.99999999999980993 676.5203681218851 -1259.1392167224028 771.32342877765313 -176.61502916214059 12.507343278686905 -0.13857109526572012 9.9843695780195716e-6 1.5056327351493116e-7", c, " ")
    x -= 1
    s = c[1]
    for (i = 1; i < 9; i++) s += c[i + 1] / (x + i)
    t = x + 7.5
    return 0.5 * log(2 * 3.141592653589793) + (x + 0.5) * log(t) - t + log(s)
}
function gammq(a, x,   sum, del, ap, n, b, c, d, h, an, i) {
    # The regularized upper incomplete gamma Q(a, x), as Numerical Recipes
    # computes it: a series below a + 1, a continued fraction above.
    if (x <= 0) return 1
    if (x < a + 1) {
        ap = a; sum = 1 / a; del = sum
        for (n = 1; n <= 500; n++) {
            ap += 1; del *= x / ap; sum += del
            if (del < sum * 1e-15) break
        }
        return 1 - sum * exp(-x + a * log(x) - lgamma(a))
    }
    b = x + 1 - a; c = 1 / 1e-300; d = 1 / b; h = d
    for (i = 1; i <= 500; i++) {
        an = -i * (i - a); b += 2
        d = an * d + b; if (d < 0 ? -d < 1e-300 : d < 1e-300) d = 1e-300
        c = b + an / c; if (c < 0 ? -c < 1e-300 : c < 1e-300) c = 1e-300
        d = 1 / d; del = d * c; h *= del
        if ((del - 1 < 0 ? 1 - del : del - 1) < 1e-15) break
    }
    return exp(-x + a * log(x) - lgamma(a)) * h
}
function phi(z,   t, y, ans) {
    # The standard normal CDF, through the complementary error function of
    # Numerical Recipes (erfcc), whose error is below 1.2e-7 everywhere.
    y = (z < 0 ? -z : z) / sqrt(2)
    t = 1 / (1 + 0.5 * y)
    ans = t * exp(-y * y - 1.26551223 + t * (1.00002368 + t * (0.37409196 + t * (0.09678418 + t * (-0.18628806 + t * (0.27886807 + t * (-1.13520398 + t * (1.48851587 + t * (-0.82215223 + t * 0.17087277)))))))))
    return z >= 0 ? 1 - 0.5 * ans : 0.5 * ans
}
FILENAME == ARGV[1] {
    if ($1 != "") { ids[$1] = 1; lines[$1]++; if ($2 == "skip") skipped[$1]++ }
    said[$3 SUBSEP $4] = 1
    next
}
FILENAME == ARGV[2] { held[$1] = $2; next }
{
    if ($2 in seen) next
    seen[$2] = 1
    if ($1 == "bad") { bad++; next }
    if ($1 + 0 < start || $1 + 0 >= end) next
    if (held[$4] == "gone") { gone++; next }
    if (held[$4] == "none") { noengine++; next }
    n++
    t[n] = $1 + 0; id[n] = $2; ses[n] = $3; text[n] = $5
}
END {
    printf "day: %s\n", day
    printf "window: [%sT00:00:00Z, next day T00:00:00Z)\n", day
    printf "excluded, timestamp unparsed: %d\n", bad
    printf "excluded, checkout without an engine: %d\n", noengine
    printf "excluded, checkout gone: %d\n", gone
    printf "person prompts (N): %d\n", n
    j = 0; dup = 0; sk = 0
    for (i = 1; i <= n; i++) {
        hit[i] = (id[i] in ids) ? 1 : 0
        if (hit[i]) { j++; dup += lines[id[i]] - 1; sk += skipped[id[i]] + 0 }
        k = ses[i]
        if (!(k in sn)) { ns++; sname[ns] = k }
        sn[k]++; if (hit[i]) sh[k]++; else sm[k]++
    }
    printf "joined distinct prompts (J): %d\n", j
    if (n > 0) {
        # The Wilson score interval at 95%, which stays inside [0, 1] and says
        # something at J = N, where the normal interval has width zero.
        zz = 1.959964; c0 = j / n
        mid = (c0 + zz * zz / (2 * n)) / (1 + zz * zz / n)
        half = zz * sqrt(c0 * (1 - c0) / n + zz * zz / (4 * n * n)) / (1 + zz * zz / n)
        printf "capture rate (J/N): %.4f, Wilson 95%% interval [%.4f, %.4f]\n", c0, mid - half, mid + half
    } else printf "capture rate (J/N): n/a (no person prompts)\n"
    printf "missed prompts: %d\n", n - j
    # A miss whose exact text a line of the same session carries was written,
    # under a prompt id the transcript did not keep: on 2026-09-28 a prompt
    # typed while a task notification was in flight was logged under the
    # id of the notification. It stays a miss of the join, which is on ids, and is
    # counted here so a reading can tell a lost write from a lost key.
    tm = 0
    for (i = 1; i <= n; i++) if (!hit[i] && ((ses[i] SUBSEP text[i]) in said)) tm++
    printf "missed prompts whose text the session logged under another id: %d\n", tm
    printf "joined lines with a skip reason: %d\n", sk
    printf "duplicate lines for joined prompts: %d\n", dup
    printf "sessions: %d\n", ns
    for (s = 1; s <= ns; s++) {
        k = sname[s]
        if (sm[k] > 0) printf "session with misses: %s %d of %d missed\n", k, sm[k], sn[k]
    }
    printf "threshold: family-wise alpha 0.05, each test rejects at p < 0.025\n"

    reject = 0
    m = n - j
    if (m == 0 || j == 0 || ns < 2) {
        printf "sessions test: n/a (%s)\n", m == 0 ? "no misses" : (j == 0 ? "no hits" : "one session")
    } else {
        chi = 0
        for (s = 1; s <= ns; s++) {
            k = sname[s]
            eh = sn[k] * j / n; em = sn[k] * m / n
            chi += (sh[k] - eh) ^ 2 / eh + (sm[k] - em) ^ 2 / em
        }
        df = ns - 1
        p = gammq(df / 2, chi / 2)
        printf "sessions test: chi-square %.4f, df %d, p %.4f, %s\n", chi, df, p, p < 0.025 ? "rejected" : "not rejected"
        if (p < 0.025) reject = 1
    }

    # Runs within each session in time order. The records are sorted by time,
    # then by id, before they reach here.
    r = 0; er = 0; vr = 0; mixed = 0
    for (s = 1; s <= ns; s++) last[sname[s]] = -1
    for (i = 1; i <= n; i++) {
        k = ses[i]
        if (last[k] != hit[i]) runs[k]++
        last[k] = hit[i]
    }
    for (s = 1; s <= ns; s++) {
        k = sname[s]
        a = sh[k] + 0; b = sm[k] + 0; c = a + b
        r += runs[k]
        if (a > 0 && b > 0) {
            mixed++
            er += 1 + 2 * a * b / c
            vr += 2 * a * b * (2 * a * b - c) / (c * c * (c - 1))
        } else {
            er += 1
        }
    }
    if (mixed == 0 || vr <= 0) {
        printf "runs test: n/a (%s)\n", m == 0 ? "no misses" : "no session holds both a hit and a miss"
    } else {
        z = (r - er) / sqrt(vr)
        p = phi(z)
        printf "runs test: runs %d, expected %.4f, variance %.4f, z %.4f, p %.4f (lower tail), %s\n", r, er, vr, z, p, p < 0.025 ? "rejected" : "not rejected"
        if (p < 0.025) reject = 1
    }
    printf "randomness: %s\n", reject ? "rejected" : "not rejected"
}
' "$scratch/log.tsv" "$scratch/held.tsv" "$scratch/sorted.tsv"
