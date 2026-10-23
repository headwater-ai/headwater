#!/bin/sh
# The offline cross-encoder reranking experiment of #1688.
#
# The blind grade of #1670 scored every document that either routing path
# offered on 361 prompts. This tool asks what a reranker would have offered
# from the same documents. It takes the union of both paths' lists for each
# prompt, drops the obligations, the reviews and the sealed probes, scores
# each remaining (task, document) pair with the cross-encoder that
# `.headwater/reranker.yml` pins, sorts by score, and cuts to 5. Then it
# grades the cut from the consensus the raters already wrote. No pair is
# graded again, and no rater is paid (#1688, Done-when 2).
#
#     sh tools/run/rerank-experiment.sh score [--packets <dir>] [--pin <file>] [--models <dir>]
#     sh tools/run/rerank-experiment.sh grade
#     sh tools/run/rerank-experiment.sh order
#     sh tools/run/rerank-experiment.sh run   (score, then grade)
#
# Each verb takes `--dir <dir>`, the record directory, by default
# `tools/run/relevance-grade`.
#
# `score` reads the packets that `relevance-grade.sh packets` wrote, so the
# reranker reads the text the raters read and nothing is rendered from the
# current tree: the task of each prompt, and the title, a newline and the
# summary of each document. The packets are in `HEADWATER_RELEVANCE_PACKETS`
# or `--packets`, by default `<git common dir>/headwater-relevance-grade`,
# and they are never committed. `score` reads `sample.tsv` and never
# `consensus.tsv`, so no grade reaches the order. It writes
# `rerank-scores.tsv`: prompt id, path and score, and no text.
#
# The runner is `engine/crates/embed/examples/rerank.rs`, which reads the
# files `tools/embed/fetch-model.sh --pin .headwater/reranker.yml` fetched,
# checks each against its digest and opens no socket (HW-REQ-0001).
# `RERANK_RUNNER` names another command that reads the same pairs and
# writes the same lines, which is how the fixtures inject scores.
#
# `grade` reads only `sample.tsv`, `consensus.tsv` and `rerank-scores.tsv`,
# so a clone with no packets and no model reprints `rerank.txt`. For each
# stratum it prints the count of prompts whose top 3 holds a score-2
# document, for the embedding path (the first 3 of its list, as #1670 read
# it) and for the rerank, with the exact McNemar test on the discordant
# prompts; the count whose first pick scores 2; the oracle over the
# filtered union; and the pairs in the cut with no consensus score, which
# are counted and read as not scoring 2. Then the verdict, by #1688's bars:
#     accept        on disjoint prompts the rerank gains (c > b) with an
#                   exact two-sided p under 0.05, and the rerank's silent
#                   first-at-2 is 8 or more
#     reject        the rerank's disjoint top 3 reaches fewer than 30 prompts
#     inconclusive  anything else, and both bars holding at once
# `order` prints each prompt's cut to 5, in order, from the score file alone.
#
# Exit status: 0 done, 1 an environment failure, 2 a usage error, 4 a record
# that does not hold together.

set -u

here=$(cd "$(dirname "$0")/../.." && pwd)
dir="$here/tools/run/relevance-grade"
packets=${HEADWATER_RELEVANCE_PACKETS:-}
pin="$here/.headwater/reranker.yml"
models=${HEADWATER_RERANKER_DIR:-}

die() { echo "rerank-experiment: $*" >&2; exit 1; }
usage() { echo "rerank-experiment: $*" >&2; exit 2; }
refuse() { echo "rerank-experiment: $*" >&2; exit 4; }

verb=${1:-}
[ -n "$verb" ] || usage "name a verb: score, grade, order or run"
shift
case $verb in
    score | grade | order | run) ;;
    *) usage "unknown verb $verb" ;;
esac
while [ $# -gt 0 ]; do
    case $1 in
        --dir) dir=${2:?--dir takes a directory}; shift 2 ;;
        --packets) packets=${2:?--packets takes a directory}; shift 2 ;;
        --pin) pin=${2:?--pin takes a file}; shift 2 ;;
        --models) models=${2:?--models takes a directory}; shift 2 ;;
        *) usage "unknown argument $1" ;;
    esac
done

command -v awk >/dev/null 2>&1 || die "needs awk"
LC_ALL=C
export LC_ALL

common_dir() {
    _c=$(git -C "$here" rev-parse --path-format=absolute --git-common-dir 2>/dev/null) || die "not inside a git checkout"
    printf '%s\n' "$_c"
}

work=$(mktemp -d) || exit 1
trap 'rm -rf "$work"' EXIT HUP INT TERM

# The filter #1688 names: an obligation, a review or a sealed probe is never
# reranked. The same expression is written in each awk program below.
score() {
    [ -n "$packets" ] || packets="$(common_dir)/headwater-relevance-grade"
    for f in key/prompts.tsv key/documents.tsv; do
        [ -f "$packets/$f" ] || die "no $f under $packets; run relevance-grade.sh packets first"
    done
    set -- "$packets"/rater-a/batch-*.md
    [ -f "$1" ] || die "no rater-a batches under $packets"
    [ -f "$dir/sample.tsv" ] || die "no sample.tsv in $dir"

    # The pairs, in sample order and in the order of each union.
    awk -F '\t' -v OFS='\t' -v sample_file="$dir/sample.tsv" -v pkeys="$packets/key/prompts.tsv" -v dkeys="$packets/key/documents.tsv" '
        function esc(s) { gsub(/\\/, "\\\\", s); gsub(/\t/, "\\t", s); gsub(/\n/, "\\n", s); return s }
        function close_task() {
            if (mode == "t") { sub(/\n+$/, "", text); task[label] = text }
        }
        BEGIN {
            while ((getline line < pkeys) > 0) {
                split(line, f, "\t")
                if (f[1] == "rater") prompt_of[f[2]] = f[3]
            }
            while ((getline line < dkeys) > 0) {
                split(line, f, "\t")
                if (f[1] != "prompt_id") path_of[f[1] SUBSEP f[2]] = f[3]
            }
        }
        FILENAME != sample_file {
            if (match($0, /^=== P[0-9]+ task ===$/)) {
                close_task()
                label = $0; sub(/^=== /, "", label); sub(/ task ===$/, "", label)
                mode = "t"; text = ""; first = 1; next
            }
            if (match($0, /^=== P[0-9]+ documents ===$/)) { close_task(); mode = "d"; next }
            if (mode == "t") { text = first ? $0 : text "\n" $0; first = 0; next }
            if (mode == "d" && match($0, /^D[0-9]+\. /)) {
                dlabel = substr($0, 1, RLENGTH - 2); title = substr($0, RLENGTH + 1); next
            }
            if (mode == "d" && /^    /) {
                id = prompt_of[label]
                k = id SUBSEP path_of[id SUBSEP dlabel]
                doc[k] = title "\n" substr($0, 5)
                tasked[id] = label
            }
            next
        }
        FNR == 1 { close_task(); mode = ""; next }
        {
            id = $1
            n = 0
            delete seen
            m = split($7 "|" $8, items, "|")
            for (i = 1; i <= m; i++) {
                p = items[i]
                if (p == "" || (p in seen)) continue
                seen[p] = 1
                if (p ~ /^docs\/obligations\// || p ~ /^docs\/reviews\// || p ~ /^sealed:/) continue
                k = id SUBSEP p
                if (!(k in doc) || !((id) in tasked)) { missing++; if (missing <= 5) print "no packet text for " id " " p > "/dev/stderr"; continue }
                print id, p, esc(task[tasked[id]]), esc(doc[k])
            }
        }
        END { if (missing) { print missing " pairs have no packet text" > "/dev/stderr"; exit 4 } }
    ' "$@" "$dir/sample.tsv" > "$work/pairs.tsv"
    status=$?
    [ "$status" -eq 0 ] || refuse "the packets do not hold every pair the sample offers"

    if [ -n "${RERANK_RUNNER:-}" ]; then
        # The fixtures' stand-in: a command line, split on spaces.
        # shellcheck disable=SC2086
        $RERANK_RUNNER < "$work/pairs.tsv" > "$work/scores.tsv" || die "the runner failed"
    else
        [ -n "$models" ] || models="$(common_dir)/headwater-models/reranker"
        sh "$here/tools/hw-cargo" run --quiet --profile dev-release --example rerank -p headwater-embed \
            --manifest-path "$here/engine/Cargo.toml" --locked -- "$pin" "$models" \
            < "$work/pairs.tsv" > "$work/scores.tsv" || die "the rerank example failed"
    fi

    # One score per pair, in the order of the pairs, or nothing is written.
    awk -F '\t' -v OFS='\t' '
        NR == FNR { want[FNR] = $1 "\t" $2; n = FNR; next }
        {
            got++
            if ($1 "\t" $2 != want[got]) { print "score line " got " is for " $1 " " $2 ", not " want[got] > "/dev/stderr"; bad = 1; exit }
            if ($3 !~ /^-?[0-9]+(\.[0-9]+)?([eE][-+]?[0-9]+)?$/) { print "score line " got " is not a number: " $3 > "/dev/stderr"; bad = 1; exit }
        }
        END { if (bad) exit 4; if (got != n) { print got " scores for " n " pairs" > "/dev/stderr"; exit 4 } }
    ' "$work/pairs.tsv" "$work/scores.tsv" || refuse "the runner's scores do not match the pairs"
    {
        printf 'prompt_id\tpath\tscore\n'
        cat "$work/scores.tsv"
    } > "$work/rerank-scores.tsv"
    mv "$work/rerank-scores.tsv" "$dir/rerank-scores.tsv"
    echo "wrote $dir/rerank-scores.tsv: $(($(wc -l < "$dir/rerank-scores.tsv") - 1)) scored pairs"
}

# The ranked cut to 5 of each prompt: score descending, path ascending on a tie.
ranked() {
    [ -f "$dir/rerank-scores.tsv" ] || die "no rerank-scores.tsv in $dir; run score first"
    tail -n +2 "$dir/rerank-scores.tsv" | sort -t "$(printf '\t')" -k1,1 -k3,3gr -k2,2 | awk -F '\t' -v OFS='\t' '
        $1 != last { if (last != "") print last, list; last = $1; list = ""; n = 0 }
        n < 5 { list = (n ? list "|" : "") $2; n++ }
        END { if (last != "") print last, list }
    '
}

grade() {
    for f in sample.tsv consensus.tsv rerank-scores.tsv; do
        [ -f "$dir/$f" ] || die "no $f in $dir"
    done
    ranked > "$work/cut.tsv" || exit 1
    awk -F '\t' -v cutf="$work/cut.tsv" -v scoresf="$dir/rerank-scores.tsv" '
        function filtered(p) { return p ~ /^docs\/obligations\// || p ~ /^docs\/reviews\// || p ~ /^sealed:/ }
        function two(id, p) { return ((id SUBSEP p) in value) && value[id SUBSEP p] == 2 }
        # Exact two-sided McNemar: a binomial on the discordant prompts, p = 0.5.
        function mcnemar(b, c,    n, k, lo, t, sum) {
            n = b + c
            if (n == 0) return 1
            lo = (b < c) ? b : c
            t = 2 ^ (-n); sum = 0
            for (k = 0; k <= lo; k++) { sum += t; t = t * (n - k) / (k + 1) }
            sum *= 2
            return (sum > 1) ? 1 : sum
        }
        FILENAME == consf {
            if (FNR == 1) next
            if ($3 !~ /^[012]$/) { print "consensus line " FNR " has value " $3 > "/dev/stderr"; bad = 1; exit }
            value[$1 SUBSEP $2] = $3
            next
        }
        FNR == 1 { next }
        {
            id = $1; s = $2
            if (s != "silent" && s != "disjoint" && s != "overlap") { print "stratum " s " is not one of the three" > "/dev/stderr"; bad = 1; exit }
            stratum[id] = s; order[++np] = id; total[s]++
            m = split($8, e, "|")
            ehit = 0
            for (i = 1; i <= 3 && i <= m; i++) if (two(id, e[i])) ehit = 1
            efirst = (m >= 1 && two(id, e[1]))
            delete seen
            m = split($7 "|" $8, u, "|")
            orc = 0; orcall = 0; want = 0
            for (i = 1; i <= m; i++) {
                p = u[i]
                if (p == "" || (p in seen)) continue
                seen[p] = 1
                if (two(id, p)) orcall = 1
                if (filtered(p)) continue
                if (two(id, p)) orc = 1
                need[id SUBSEP p] = 1; want++
            }
            emb[s] += ehit; embfirst[s] += efirst; oracle[s] += orc; oracleall[s] += orcall
            ehits[id] = ehit; wants[id] = want
        }
        END {
            if (bad) exit 4
            while ((getline line < scoresf) > 0) {
                split(line, f, "\t")
                if (f[1] == "prompt_id") continue
                if (!((f[1] SUBSEP f[2]) in need)) { print "the score file scores " f[1] " " f[2] ", which the sample does not offer after the filter" > "/dev/stderr"; exit 4 }
                if (filtered(f[2])) { print "the score file scores a filtered item " f[2] > "/dev/stderr"; exit 4 }
                have[f[1]]++
            }
            for (i = 1; i <= np; i++) if (have[order[i]] != wants[order[i]]) { print "prompt " order[i] " has " have[order[i]] + 0 " scores for " wants[order[i]] " pairs" > "/dev/stderr"; exit 4 }
            while ((getline line < cutf) > 0) {
                split(line, f, "\t")
                id = f[1]; s = stratum[id]
                k = split(f[2], c, "|")
                rhit = 0
                for (i = 1; i <= k; i++) {
                    pairs[s]++
                    if (!((id SUBSEP c[i]) in value)) unscored[s]++
                    if (i <= 3 && two(id, c[i])) rhit = 1
                }
                if (k >= 1 && two(id, c[1])) rerfirst[s]++
                rer[s] += rhit
                if (ehits[id] && !rhit) b[s]++
                if (!ehits[id] && rhit) cc[s]++
            }
            print "# The offline cross-encoder rerank of the #1670 sample (#1688)."
            print "# Written by `sh tools/run/rerank-experiment.sh grade` from sample.tsv,"
            print "# consensus.tsv and rerank-scores.tsv. Do not edit it by hand."
            print "# The rerank is the union of both paths'"'"' lists with obligations, reviews"
            print "# and sealed probes dropped, sorted by the pinned cross-encoder and cut to 5."
            print "# b counts prompts the embedding top 3 reaches and the rerank misses; c the reverse."
            ns = split("silent disjoint overlap", names, " ")
            for (j = 1; j <= ns; j++) {
                s = names[j]; n = total[s] + 0
                pval[s] = mcnemar(b[s] + 0, cc[s] + 0)
                print ""
                printf "%s: top 3 holds a score-2 document: embedding %d of %d, rerank %d of %d\n", s, emb[s], n, rer[s], n
                printf "%s: McNemar b %d, c %d, exact p %.6g\n", s, b[s], cc[s], pval[s]
                printf "%s: first pick scores 2: embedding %d of %d, rerank %d of %d\n", s, embfirst[s], n, rerfirst[s], n
                printf "%s: oracle over the filtered union: %d of %d (unfiltered %d of %d)\n", s, oracle[s], n, oracleall[s], n
                printf "%s: unscored pairs in the cut to 5: %d of %d\n", s, unscored[s], pairs[s]
            }
            accept = (pval["disjoint"] < 0.05 && cc["disjoint"] > b["disjoint"] && rerfirst["silent"] >= 8)
            reject = (rer["disjoint"] < 30)
            print ""
            print "bars: accept when disjoint gains with exact p < 0.05 and silent first-at-2 >= 8; reject when disjoint top 3 < 30"
            if (accept && !reject) print "verdict: accept"
            else if (reject && !accept) print "verdict: reject"
            else print "verdict: inconclusive"
            if (accept && reject) print "reason: both the accept bar and the reject bar hold"
            else if (!accept && !reject) printf "reason: disjoint p %.6g with b %d and c %d, silent first-at-2 %d, disjoint top 3 %d\n", pval["disjoint"], b["disjoint"], cc["disjoint"], rerfirst["silent"], rer["disjoint"]
        }
    ' consf="$dir/consensus.tsv" "$dir/consensus.tsv" "$dir/sample.tsv" > "$work/rerank.txt"
    status=$?
    [ "$status" -eq 0 ] || refuse "the record does not hold together"
    mv "$work/rerank.txt" "$dir/rerank.txt"
    tail -n 2 "$dir/rerank.txt"
}

case $verb in
    score) score ;;
    grade) grade ;;
    order) ranked ;;
    run) score && grade ;;
esac
