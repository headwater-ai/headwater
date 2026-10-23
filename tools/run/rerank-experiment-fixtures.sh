#!/bin/sh
# What holds `tools/run/rerank-experiment.sh`.
#
# The experiment exists to give HW-DR-0105 a verdict on a cross-encoder
# reranker, so the cases plant a sample, a consensus, blind packets and the
# scores a reranker returns. The scores are injected by a stand-in runner,
# not computed by the model, so each count below was worked by hand:
#
#   a  the decisive case. Each prompt has three embedding offers e1, e2, e3,
#      one route offer r1, and an obligation, a review and a sealed probe
#      that score 2 and carry the highest injected score. A prompt is one of
#      these shapes, named by what each path's top 3 holds:
#        BB  e1 scores 2 and the reranker puts it first: both hit.
#        BN  e2 scores 2 and the reranker puts it last: the embedding path
#            hits and the reranker misses.
#        NB  r1 scores 2 and the reranker puts it first.
#        NS  r1 scores 2 and the reranker puts it second, under e1: a top-3
#            hit that is not a first-at-2.
#        TI  r1 scores 2 and ties e2 and e3 under e1. The tie goes to the
#            path in ascending order, so r1 is fourth and the reranker misses.
#        N3  r1 scores 2 and the reranker puts it third: still a top-3 hit.
#        WD  seven documents, none at 2: the cut keeps the first 5.
#        NN  nothing scores 2.
#      Four variants, one per verdict, each worked here:
#        accept   silent 2 BB, 6 NB, 1 NS, 1 NN: embedding 2/10, rerank 9/10,
#                 b 0, c 7, p 2/128 = 0.015625; first-at-2 2 against 8.
#                 disjoint 24 BB, 6 NB, 1 TI, 1 NN: 24/32 against 30/32,
#                 b 0, c 6, p 2/64 = 0.03125. overlap 1 BB, 1 BN, 3 NB, 1 N3,
#                 1 WD, 1 NN: 2/8 against 5/8, b 1, c 4, p 2 x 6/32 = 0.375.
#        reject   disjoint 29 BB, 1 BN, 1 TI, 1 NN: 30/32 against 29/32, b 1,
#                 c 0, p 1. 29 is under the bar of 30.
#        inconclusive  silent 2 BB, 5 NB, 2 NS, 1 NN: first-at-2 7, one under
#                 the bar of 8. disjoint as in accept.
#        weak     disjoint 25 BB, 5 NB, 1 TI, 1 NN: c 5, p 2/32 = 0.0625, which
#                 is not under 0.05. silent as in accept.
#        worse    disjoint 30 BB, 6 BN, 1 TI, 1 NN: 36/38 against 30/38, b 6,
#                 c 0, p 0.03125. Significant, but a loss, so not accept.
#        both     disjoint 23 BB, 6 NB, 1 TI, 2 NN: 23/32 against 29/32, c 6,
#                 p 0.03125. The accept bar and the reject bar both hold.
#      The obligation, the review and the sealed probe never reach the
#      reranker or the cut to 5: if one did, every prompt would hit. The
#      overlap NN prompt's r1 has no consensus score, so one graded pair is
#      unscored and counted. The leak guard: with the consensus values
#      permuted, the score file and the reranked order are the same bytes,
#      and only the counts move.
#   b  the pairs the runner reads: the task text of the packet, its lines
#      joined, and the title, a newline and the summary of the document.
#   c  the refusals: a document the packets do not hold, a runner that
#      returns its scores out of order, a runner that returns fewer scores
#      than pairs, and an unknown argument.
#   k  the committed rerank.txt is what `grade` prints on the committed
#      score file, sample and consensus, so a clone reprints it without the
#      texts.
#
# No task here is a prompt that a person typed (OWNER #1670, "Digests only").
#
# Run it from anywhere:
#     sh tools/run/rerank-experiment-fixtures.sh
#
# It needs `awk` and `sort`, exits 3 without `awk` so CI can skip with a
# warning, and writes only under a temporary directory.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="${RERANK_EXPERIMENT_TOOL:-$root/tools/run/rerank-experiment.sh}"

if ! command -v awk >/dev/null 2>&1; then
    echo "no \`awk\` on the path, and the tool under test grades with it." >&2
    exit 3
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0
pass() { printf 'ok   %s\n' "$1"; passed=$((passed + 1)); }
fail() { printf 'FAIL %s\n  %s\n' "$1" "$2"; failed=$((failed + 1)); }
check() { if eval "$2"; then pass "$1"; else fail "$1" "${3:-the condition is false}"; fi; }
has() {
    if grep -Fqx -- "$3" "$2"; then pass "$1: $3"; else fail "$1: $3" "not in $2; got: $(tr '\n' '|' < "$2" 2>/dev/null)"; fi
}
lacks() {
    if grep -Fq -- "$3" "$2"; then fail "$1: no line holds $3" "found: $(grep -F -- "$3" "$2" | head -n 3 | tr '\n' '|')"; else pass "$1: no line holds $3"; fi
}

# The stand-in runner: it logs what it reads and returns the injected score
# of each (prompt, path) from the table `RERANK_FAKE_TABLE` names.
fake="$scratch/fake-runner.sh"
cat > "$fake" <<'EOF'
#!/bin/sh
tee "$RERANK_FAKE_LOG" | awk -F '\t' -v table="$RERANK_FAKE_TABLE" -v short="${RERANK_FAKE_SHORT:-0}" '
    BEGIN { OFS = "\t"; while ((getline line < table) > 0) { split(line, f, "\t"); s[f[1] SUBSEP f[2]] = f[3] } }
    { n++; if (short == "first" && n == 1) next; if (n > 1 && short == "last") print held; held = $1 "\t" $2 "\t" s[$1 SUBSEP $2]; if (short != "last") print held }
'
EOF

# plant <dir> <stratum>:<shape>:<count>...
# Writes sample.tsv, consensus.tsv, the injected table and the packets.
plant() {
    d=$1
    shift
    mkdir -p "$d/rec" "$d/packets/rater-a" "$d/packets/key"
    printf '%s\n' "$@" | awk -F ':' -v d="$d" '
        BEGIN {
            OFS = "\t"
            sample = d "/rec/sample.tsv"; cons = d "/rec/consensus.tsv"; table = d "/table.tsv"
            batch = d "/packets/rater-a/batch-01.md"
            pkeys = d "/packets/key/prompts.tsv"; dkeys = d "/packets/key/documents.tsv"
            print "prompt_id", "stratum", "at", "file", "line", "task_digest", "deterministic", "embedding" > sample
            print "prompt_id", "item", "value", "settled_by" > cons
            print "packet", "label", "prompt_id", "batch" > pkeys
            print "prompt_id", "label", "path" > dkeys
            n = 0
        }
        function item(id, path, value, score, label) {
            if (value != "") print id, path, value, "agreed" > cons
            print id, path, score > table
            print id, "D" label, path > dkeys
            if (path ~ /^sealed:/) title = "Sealed probe"; else title = "Title of " substr(path, length("docs/t/" id "-") + 1)
            sub(/\.md$/, "", title)
            printf "D%d. %s\n    Summary of %s\n", label, title, path > batch
        }
        {
            stratum = $1; shape = $2
            for (k = 1; k <= $3; k++) {
                n++
                id = sprintf("%s-%s-%02d", substr(stratum, 1, 1), tolower(shape), k)
                p = "docs/t/" id
                e1 = 0; e2 = 0; e3 = 0; r1 = 0
                s1 = 5; s2 = 4; s3 = 3; sr = 2
                if (shape == "BB") { e1 = 2; s1 = 9 }
                if (shape == "BN") { e2 = 2; s1 = 5; s2 = 1; s3 = 4; sr = 3 }
                if (shape == "NB") { r1 = 2; sr = 9 }
                if (shape == "NS") { r1 = 2; s1 = 9; sr = 8 }
                if (shape == "TI") { r1 = 2; s1 = 9; s2 = 3; s3 = 3; sr = 3 }
                if (shape == "N3") { r1 = 2; s1 = 9; s2 = 8; sr = 7; s3 = 1 }
                if (shape == "NN" || shape == "WD") { e2 = 1; sr = 9 }
                emb = p "-e1.md|" p "-e2.md|" p "-e3.md|docs/obligations/" id ".md|docs/reviews/" id ".md|sealed:" id
                if (shape == "WD") emb = emb "|" p "-e4.md|" p "-e5.md|" p "-e6.md"
                if (stratum == "silent") { det = ""; emb = emb "|" p "-r1.md" } else det = p "-r1.md"
                print id, stratum, "2026-10-01T00:00:00Z", "log.jsonl", n, "sha256:" id, det, emb > sample
                print "rater", "P" n, id, "01" > pkeys
                printf "=== P%d task ===\nTask of %s\nsecond line\n=== P%d documents ===\n", n, id, n > batch
                item(id, p "-e1.md", e1, s1, 1)
                item(id, p "-e2.md", e2, s2, 2)
                item(id, p "-e3.md", e3, s3, 3)
                # The overlap NN prompt leaves its r1 unscored.
                item(id, p "-r1.md", (stratum == "overlap" && shape == "NN") ? "" : r1, sr, 4)
                item(id, "docs/obligations/" id ".md", 2, 99, 5)
                item(id, "docs/reviews/" id ".md", 2, 98, 6)
                item(id, "sealed:" id, 2, 97, 7)
                if (shape == "WD") {
                    item(id, p "-e4.md", 0, 1.5, 8)
                    item(id, p "-e5.md", 0, 1.4, 9)
                    item(id, p "-e6.md", 0, 1.3, 10)
                }
                print "" > batch
            }
        }
    '
}

# run <dir> <verb> [args...]: the tool against a planted directory, with the
# stand-in runner. Status, stdout and stderr land beside it.
run() {
    d=$1
    verb=$2
    shift 2
    RERANK_RUNNER="sh $fake" RERANK_FAKE_TABLE="$d/table.tsv" RERANK_FAKE_LOG="$d/runner.log" \
        sh "$tool" "$verb" --dir "$d/rec" --packets "$d/packets" "$@" > "$d/$verb.out" 2> "$d/$verb.err"
    echo $? > "$d/$verb.rc"
}
rc() {
    got=$(cat "$2" 2>/dev/null)
    if [ "$got" = "$3" ]; then pass "$1"; else fail "$1" "exit $got, wanted $3: $(head -c 400 "${2%.rc}.err" 2>/dev/null)"; fi
}

overlap="overlap:BB:1 overlap:BN:1 overlap:NB:3 overlap:N3:1 overlap:WD:1 overlap:NN:1"

# a: accept.
A="$scratch/accept"
plant "$A" silent:BB:2 silent:NB:6 silent:NS:1 silent:NN:1 disjoint:BB:24 disjoint:NB:6 disjoint:TI:1 disjoint:NN:1 $overlap
run "$A" run
rc "a accept: run exits 0" "$A/run.rc" 0
T="$A/rec/rerank.txt"
has "a accept" "$T" "silent: top 3 holds a score-2 document: embedding 2 of 10, rerank 9 of 10"
has "a accept" "$T" "silent: McNemar b 0, c 7, exact p 0.015625"
has "a accept" "$T" "silent: first pick scores 2: embedding 2 of 10, rerank 8 of 10"
has "a accept" "$T" "silent: oracle over the filtered union: 9 of 10 (unfiltered 10 of 10)"
has "a accept" "$T" "disjoint: top 3 holds a score-2 document: embedding 24 of 32, rerank 30 of 32"
has "a accept" "$T" "disjoint: McNemar b 0, c 6, exact p 0.03125"
has "a accept" "$T" "disjoint: oracle over the filtered union: 31 of 32 (unfiltered 32 of 32)"
has "a accept" "$T" "overlap: top 3 holds a score-2 document: embedding 2 of 8, rerank 5 of 8"
has "a accept" "$T" "overlap: McNemar b 1, c 4, exact p 0.375"
has "a accept" "$T" "overlap: unscored pairs in the cut to 5: 1 of 33"
has "a accept" "$T" "verdict: accept"
lacks "a accept: no filtered item reaches the runner" "$A/runner.log" "docs/obligations/"
lacks "a accept: no review reaches the runner" "$A/runner.log" "docs/reviews/"
lacks "a accept: no sealed probe reaches the runner" "$A/runner.log" "sealed:"
lacks "a accept: no filtered item is in the score file" "$A/rec/rerank-scores.tsv" "docs/obligations/"
check "a accept: the score file holds one row per filtered pair, 4 for each of 50 prompts, 3 more for WD, and a header" '[ "$(wc -l < "$A/rec/rerank-scores.tsv")" -eq 204 ]'
run "$A" order
has "a accept: the TI prompt's tie goes to the path in ascending order" "$A/order.out" "d-ti-01	docs/t/d-ti-01-e1.md|docs/t/d-ti-01-e2.md|docs/t/d-ti-01-e3.md|docs/t/d-ti-01-r1.md"
has "a accept: the N3 prompt's score-2 document is third" "$A/order.out" "o-n3-01	docs/t/o-n3-01-e1.md|docs/t/o-n3-01-e2.md|docs/t/o-n3-01-r1.md|docs/t/o-n3-01-e3.md"
has "a accept: the WD prompt is cut to its first 5" "$A/order.out" "o-wd-01	docs/t/o-wd-01-r1.md|docs/t/o-wd-01-e1.md|docs/t/o-wd-01-e2.md|docs/t/o-wd-01-e3.md|docs/t/o-wd-01-e4.md"
has "a accept: the BN prompt's score-2 document is cut to last" "$A/order.out" "o-bn-01	docs/t/o-bn-01-e1.md|docs/t/o-bn-01-e3.md|docs/t/o-bn-01-r1.md|docs/t/o-bn-01-e2.md"

# a: reject.
R="$scratch/reject"
plant "$R" silent:BB:2 silent:NB:6 silent:NS:1 silent:NN:1 disjoint:BB:29 disjoint:BN:1 disjoint:TI:1 disjoint:NN:1 $overlap
run "$R" run
rc "a reject: run exits 0" "$R/run.rc" 0
has "a reject" "$R/rec/rerank.txt" "disjoint: top 3 holds a score-2 document: embedding 30 of 32, rerank 29 of 32"
has "a reject" "$R/rec/rerank.txt" "disjoint: McNemar b 1, c 0, exact p 1"
has "a reject" "$R/rec/rerank.txt" "verdict: reject"

# a: inconclusive, first-at-2 one under the bar.
I="$scratch/inconclusive"
plant "$I" silent:BB:2 silent:NB:5 silent:NS:2 silent:NN:1 disjoint:BB:24 disjoint:NB:6 disjoint:TI:1 disjoint:NN:1 $overlap
run "$I" run
rc "a inconclusive: run exits 0" "$I/run.rc" 0
has "a inconclusive" "$I/rec/rerank.txt" "silent: first pick scores 2: embedding 2 of 10, rerank 7 of 10"
has "a inconclusive" "$I/rec/rerank.txt" "verdict: inconclusive"

# a: inconclusive, a significant loss.
V="$scratch/worse"
plant "$V" silent:BB:2 silent:NB:6 silent:NS:1 silent:NN:1 disjoint:BB:30 disjoint:BN:6 disjoint:TI:1 disjoint:NN:1 $overlap
run "$V" run
rc "a worse: run exits 0" "$V/run.rc" 0
has "a worse" "$V/rec/rerank.txt" "disjoint: McNemar b 6, c 0, exact p 0.03125"
has "a worse" "$V/rec/rerank.txt" "verdict: inconclusive"

# a: inconclusive, both bars hold.
B="$scratch/both"
plant "$B" silent:BB:2 silent:NB:6 silent:NS:1 silent:NN:1 disjoint:BB:23 disjoint:NB:6 disjoint:TI:1 disjoint:NN:2 $overlap
run "$B" run
rc "a both: run exits 0" "$B/run.rc" 0
has "a both" "$B/rec/rerank.txt" "disjoint: top 3 holds a score-2 document: embedding 23 of 32, rerank 29 of 32"
has "a both" "$B/rec/rerank.txt" "verdict: inconclusive"
has "a both" "$B/rec/rerank.txt" "reason: both the accept bar and the reject bar hold"

# a: inconclusive, p not under 0.05.
W="$scratch/weak"
plant "$W" silent:BB:2 silent:NB:6 silent:NS:1 silent:NN:1 disjoint:BB:25 disjoint:NB:5 disjoint:TI:1 disjoint:NN:1 $overlap
run "$W" run
rc "a weak: run exits 0" "$W/run.rc" 0
has "a weak" "$W/rec/rerank.txt" "disjoint: McNemar b 0, c 5, exact p 0.0625"
has "a weak" "$W/rec/rerank.txt" "verdict: inconclusive"

# a: the leak guard. The consensus values move one row down, cyclically.
L="$scratch/leak"
plant "$L" silent:BB:2 silent:NB:6 silent:NS:1 silent:NN:1 disjoint:BB:24 disjoint:NB:6 disjoint:TI:1 disjoint:NN:1 $overlap
awk -F '\t' 'BEGIN { OFS = "\t" } NR == 1 { print; next } { id[NR] = $1; it[NR] = $2; v[NR] = $3; s[NR] = $4; n = NR }
    END { for (i = 2; i <= n; i++) print id[i], it[i], v[(i == 2) ? n : i - 1], s[i] }' "$L/rec/consensus.tsv" > "$L/permuted.tsv"
mv "$L/permuted.tsv" "$L/rec/consensus.tsv"
run "$L" run
run "$L" order
rc "a leak: run exits 0 on permuted grades" "$L/run.rc" 0
check "a leak: the score file is the same bytes with the grades permuted" 'cmp -s "$A/rec/rerank-scores.tsv" "$L/rec/rerank-scores.tsv"'
check "a leak: the reranked order is the same bytes with the grades permuted" 'cmp -s "$A/order.out" "$L/order.out"'
check "a leak: and the counts moved" '! cmp -s "$A/rec/rerank.txt" "$L/rec/rerank.txt"'

# b: the pairs the runner reads.
has "b: a pair is the joined task text and the title, a newline and the summary" "$A/runner.log" "d-bb-01	docs/t/d-bb-01-e1.md	Task of d-bb-01\\nsecond line	Title of e1\\nSummary of docs/t/d-bb-01-e1.md"

# c: refusals.
C="$scratch/c1"
plant "$C" silent:BB:1 disjoint:BB:1 overlap:BB:1
grep -v 'd-bb-01-e2' "$C/packets/key/documents.tsv" > "$C/k" && mv "$C/k" "$C/packets/key/documents.tsv"
run "$C" score
rc "c: a document the packets do not hold exits 4" "$C/score.rc" 4
check "c: and writes no score file" '[ ! -e "$C/rec/rerank-scores.tsv" ]'
C2="$scratch/c2"
plant "$C2" silent:BB:1 disjoint:BB:1 overlap:BB:1
RERANK_FAKE_SHORT=first
export RERANK_FAKE_SHORT
run "$C2" score
rc "c: a runner whose scores are out of step with the pairs exits 4" "$C2/score.rc" 4
check "c: and writes no score file" '[ ! -e "$C2/rec/rerank-scores.tsv" ]'
RERANK_FAKE_SHORT=last
run "$C2" score
rc "c: a runner that returns fewer scores than pairs exits 4" "$C2/score.rc" 4
check "c: and writes no score file" '[ ! -e "$C2/rec/rerank-scores.tsv" ]'
grep -q 'scores for' "$C2/score.err" && pass "c: and says how many scores it read" || fail "c: and says how many scores it read" "$(cat "$C2/score.err")"
unset RERANK_FAKE_SHORT
run "$C2" bogus
rc "c: an unknown verb exits 2" "$C2/bogus.rc" 2

# k: the committed rerank.txt is what grade prints on the committed record.
K="$scratch/k"
mkdir -p "$K/rec"
for f in sample.tsv consensus.tsv rerank-scores.tsv; do cp "$root/tools/run/relevance-grade/$f" "$K/rec/" 2>/dev/null; done
run "$K" grade
rc "k: grade exits 0 on the committed record" "$K/grade.rc" 0
check "k: the committed rerank.txt is what grade prints on the committed record" 'cmp -s "$K/rec/rerank.txt" "$root/tools/run/relevance-grade/rerank.txt"'

printf '%d passed, %d failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
