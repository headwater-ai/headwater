#!/bin/sh
# What holds `tools/run/relevance-grade.sh`.
#
# The grade exists so that a rater scores a document without knowing which
# routing path offered it, which is the failure #1670 names. So the cases are
# a planted git history, planted transcripts and a planted shadow log, and
# the right answer to each was worked by hand:
#
#   a  the decisive case. Prompt s1 is logged twice, in two logs. In the
#      first, the route offers A and B and the embedding path offers B, C and
#      D. In the second, the route offers B, C and D and the embedding path
#      offers A and B, with the same summaries. A route pointer carries its
#      summary inline, and a neighbor carries only its digest, which the
#      planted history resolves. The two packets are the same bytes. B
#      appears once, and the packet holds no path and none of the words
#      `route`, `neighbors`, `pointer`, `score` or `summary_digest`. A leak
#      through rendering (an inline summary against a recovered one),
#      through order (the route first) or through a duplicate fails here.
#   b  C's summary was edited after the line was logged, so its current text
#      is not the logged one. C is rendered from the version that the digest
#      names. E's digest names no version: E is listed as unresolved, and is
#      never rendered from its current text.
#   c  the draw: the strata, the exclusion rule, the digest filter, the
#      seeded order (sha256 of "<seed>\t<prompt id>", worked here with
#      sha256sum), the owner's sample, and the same bytes for the same seed.
#   d  ingest, agreement, the third pass and the consensus, against scores
#      whose kappa was worked by hand.
#   e  the arguments, a score file that does not cover its batch or scores
#      an item it was not given, and the exit without `jq`.
#   f  the edges of the draw: a three-letter word, an empty list, and two
#      paths of which one holds the other as a part.
#   g  the third pass's batches of twelve.
#   s  the seal: a probe is named in the record by its token alone (#1384).
#
# No task here is a prompt that a person typed. The owner ruled on
# 2026-10-03 that no prompt text is committed.
#
# Run it from anywhere:
#     sh tools/run/relevance-grade-fixtures.sh
#
# It needs `jq` and `git`, exits 3 without `jq` so CI can skip with a
# warning, and writes only under a temporary directory.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="${RELEVANCE_GRADE_TOOL:-$root/tools/run/relevance-grade.sh}"
mine="$root/tools/run/shadow-mine.sh"

if ! command -v jq >/dev/null 2>&1; then
    echo "no \`jq\` on the path, and the tool under test reads JSON with it." >&2
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
sha() { printf '%s' "$1" | sha256sum | cut -d ' ' -f 1; }
tab=$(printf '\t')

# The planted history: four documents at version 1, then C edited, and E
# whose logged digest names no version.
H="$scratch/hist"
mkdir -p "$H/docs"
doc() { printf -- '---\nid: %s\ntitle: "%s"\nstatus: current\nsummary: %s\n---\n\nBody of %s.\n' "$1" "$2" "$3" "$1" > "$H/docs/$1.md"; }
doc a "Alpha title" '"Alpha summary, with \"quotes\"."'
doc b "Beta title" "Beta summary, plain."
doc c "Gamma title" "'Gamma summary, first version.'"
doc d "Delta title" '"Delta summary."'
doc e "Epsilon title" '"Epsilon summary, current."'
mkdir -p "$H/docs/probes"
doc probes/p "Probe title" '"Probe summary."'
git -C "$H" init -q
git -C "$H" -c user.name=f -c user.email=f@f add docs
git -C "$H" -c user.name=f -c user.email=f@f commit -q -m v1
doc c "Gamma title, renamed" '"Gamma summary, second version."'
git -C "$H" -c user.name=f -c user.email=f@f commit -q -am v2

SA='Alpha summary, with "quotes".'
SB='Beta summary, plain.'
SC1='Gamma summary, first version.'
SD='Delta summary.'
DA="sha256:$(sha "$SA")"
DB="sha256:$(sha "$SB")"
DC1="sha256:$(sha "$SC1")"
DD="sha256:$(sha "$SD")"
DE="sha256:$(sha 'Epsilon summary, as it was never committed.')"
M=sha256:f342437e6e5f16aa1b759f8a329adcfc9328236d2caed6c8f289f7364b1e8e44

# The route member, as the hook writes it: pointers with their summary
# inline. route <path=summary>... or "silent".
route() {
    if [ "$1" = silent ]; then
        jq -cn '{version: "1.0", pointers: [], silence: {reason: "no_purpose_matched"}}'
        return
    fi
    _a='[]'
    for _x in "$@"; do
        _a=$(jq -cn --argjson a "$_a" --arg p "${_x%%=*}" --arg s "${_x#*=}" '$a + [{path: $p, kind: "k", name: "n", summary: $s, evidence: {rank: ($a | length + 1)}}]')
    done
    jq -cn --argjson a "$_a" '{version: "1.0", pointers: $a, silence: null}'
}
# neighbors <path=digest>...
neighbors() {
    _a='[]'
    for _x in "$@"; do
        _a=$(jq -cn --argjson a "$_a" --arg p "${_x%%=*}" --arg d "${_x#*=}" '$a + [{path: $p, score: (0.9 - ($a | length) / 10), summary_digest: $d}]')
    done
    jq -cn --argjson a "$_a" --arg m "$M" '{model_digest: $m, neighbors: $a}'
}
# prompt <case> <timestamp> <id>
prompt() {
    mkdir -p "$scratch/$1/t" "$scratch/$1/log"
    printf '{"type":"user","isSidechain":false,"origin":{"kind":"human"},"promptId":"%s","sessionId":"S","timestamp":"%s"}\n' "$3" "$2" >> "$scratch/$1/t/S.jsonl"
}
# logline <case> <at> <id> <task> <route> <neighbors> [digest]
logline() {
    mkdir -p "$scratch/$1/log"
    jq -cn --arg at "$2" --arg id "$3" --arg task "$4" --arg r "$5" --arg n "$6" --arg d "${7:-$M}" \
        '{at: $at, session: "S", prompt_id: $id, probe_session: "", corpus_root: "/root/c", injected: false, task: $task, route: $r, model_digest: $d, neighbors: $n, tree_digest: "sha256:t"}' \
        >> "$scratch/$1/log/S.jsonl"
}
# rg <case> <subcommand> [<arguments>...]
rg() {
    _c=$1
    shift
    HEADWATER_TRANSCRIPT_DIRS="$scratch/$_c/t" HEADWATER_SHADOW_LOG_DIR="$scratch/$_c/log" \
        HEADWATER_RELEVANCE_RECORD="$scratch/$_c/rec" HEADWATER_RELEVANCE_PACKETS="$scratch/$_c/pk" \
        HEADWATER_RELEVANCE_HISTORY="$H" SHADOW_MINE_TOOL="$mine" \
        sh "$tool" "$@" > "$scratch/$_c.$1.out" 2> "$scratch/$_c.$1.err"
    echo $? > "$scratch/$_c.$1.rc"
}
rc() { if [ "$(cat "$scratch/$2.rc")" = "$3" ]; then pass "$1"; else fail "$1" "exit $(cat "$scratch/$2.rc"), wanted $3: $(tail -n 2 "$scratch/$2.err" 2>/dev/null | tr "\n" "|")"; fi; }

TASK='How does the cache decide which rules to run again?'

# a: one prompt, logged two ways with the paths swapped.
for c in a1 a2; do prompt "$c" 2026-09-30T10:00:00.000Z s1; done
logline a1 2026-09-30T10:00:05Z s1 "$TASK" "$(route "docs/a.md=$SA" "docs/b.md=$SB")" "$(neighbors "docs/b.md=$DB" "docs/c.md=$DC1" "docs/d.md=$DD")"
logline a2 2026-09-30T10:00:05Z s1 "$TASK" "$(route "docs/b.md=$SB" "docs/c.md=$SC1" "docs/d.md=$SD")" "$(neighbors "docs/a.md=$DA" "docs/b.md=$DB")"
for c in a1 a2; do
    rg "$c" draw --until 2026-10-01T00:00:00Z --seed 7 --control 5 --owner 1
    rc "a: $c draws" "$c.draw" 0
    rg "$c" packets --seed 7
    rc "a: $c writes its packets" "$c.packets" 0
done
P1="$scratch/a1/pk/rater-a/batch-01.md"
P2="$scratch/a2/pk/rater-a/batch-01.md"
has a "$scratch/a1/rec/draw.txt" "stratum overlap: 1, excluded 0, kept 1, drawn 1"
has a "$scratch/a2/rec/draw.txt" "stratum overlap: 1, excluded 0, kept 1, drawn 1"
check "a: swapping which path offered which document leaves the packet the same bytes" 'cmp -s "$P1" "$P2"' "the two packets differ: $(diff "$P1" "$P2" 2>&1 | head -n 6 | tr '\n' '|')"
check "a: the packet lists four documents" '[ "$(grep -c "^D[0-9]*\. " "$P1")" = 4 ]' "got $(grep -c '^D[0-9]*\. ' "$P1")"
check "a: B, offered by both paths, appears once" '[ "$(grep -c "^D[0-9]*\. Beta title$" "$P1")" = 1 ]'
has a "$P1" "$TASK"
has a "$P1" "    $SA"
has a "$P1" "    $SC1"
for w in docs/ .md route neighbors pointer score summary_digest sha256 0.9 rank; do lacks a "$P1" "$w"; done
check "a: the rater packets of a and b are the same bytes" 'cmp -s "$P1" "$scratch/a1/pk/rater-b/batch-01.md"'
check "a: the owner packet carries the same prompt" 'grep -Fqx "$TASK" "$scratch/a1/pk/owner/packet.md"'
lacks a "$scratch/a1/pk/owner/packet.md" "docs/"
# The labels follow sha256("<seed>\t<prompt id>\t<path>"), worked here.
for p in a b c d; do printf '%s\t%s\n' "$(sha "7${tab}s1${tab}docs/$p.md")" "$p"; done | LC_ALL=C sort | cut -f 2 | tr -d '\n' > "$scratch/a.order"
awk -F '\t' 'NR > 1 { print $2 "\t" $3 }' "$scratch/a1/pk/key/documents.tsv" | LC_ALL=C sort -t "$tab" -k1.2,1n | cut -f 2 | sed 's|docs/||; s|\.md||' | tr -d '\n' > "$scratch/a.key"
check "a: the labels follow the seeded order" '[ "$(cat "$scratch/a.order")" = "$(cat "$scratch/a.key")" ]' "want $(cat "$scratch/a.order"), got $(cat "$scratch/a.key")"
cp "$P1" "$scratch/a.first"
rg a1 packets --seed 7
check "a: a rerun with the same seed writes the same bytes" 'cmp -s "$scratch/a.first" "$P1"'
rg a1 packets --seed 8
check "a: another seed writes another order" '! cmp -s "$scratch/a.first" "$P1"'
rg a1 packets --seed 7

# a3: one document under two digests. The route offers C with its second
# summary inline, and the embedding path offers C under the first one's
# digest; then the paths swap. The smaller digest is read in both logs, so
# the choice reads no path, and the packets are the same bytes.
SC2='Gamma summary, second version.'
DC2="sha256:$(sha "$SC2")"
if [ "$DC1" \< "$DC2" ]; then SMALL=$SC1; else SMALL=$SC2; fi
for c in a3 a4; do prompt "$c" 2026-09-30T10:00:00.000Z s3; done
logline a3 2026-09-30T10:00:05Z s3 "$TASK" "$(route "docs/c.md=$SC2" "docs/a.md=$SA")" "$(neighbors "docs/c.md=$DC1" "docs/d.md=$DD")"
logline a4 2026-09-30T10:00:05Z s3 "$TASK" "$(route "docs/c.md=$SC1" "docs/d.md=$SD")" "$(neighbors "docs/c.md=$DC2" "docs/a.md=$DA")"
for c in a3 a4; do
    rg "$c" draw --until 2026-10-01T00:00:00Z --seed 7
    rg "$c" packets --seed 7
done
check "a: a document under two digests reads the smaller one, whichever path offered it" 'cmp -s "$scratch/a3/pk/rater-a/batch-01.md" "$scratch/a4/pk/rater-a/batch-01.md" && grep -Fqx "    $SMALL" "$scratch/a3/pk/rater-a/batch-01.md"' "got: $(grep -F 'Gamma' "$scratch/a3/pk/rater-a/batch-01.md" "$scratch/a4/pk/rater-a/batch-01.md" | tr '\n' '|')"
check "a: C appears once under two digests" '[ "$(grep -c "^    Gamma summary" "$scratch/a3/pk/rater-a/batch-01.md")" = 1 ]'

# b: C is rendered from the version its digest names, and E from none.
prompt b 2026-09-30T10:00:00.000Z u1
logline b 2026-09-30T10:00:05Z u1 "Where is the gamma rule written down?" "$(route silent)" "$(neighbors "docs/c.md=$DC1" "docs/e.md=$DE")"
rg b draw --until 2026-10-01T00:00:00Z --seed 7
rg b packets --seed 7
rc "b: writes its packets" b.packets 0
PB="$scratch/b/pk/rater-a/batch-01.md"
has b "$PB" "    $SC1"
has b "$PB" "D1. Gamma title"
lacks b "$PB" "second version"
lacks b "$PB" "renamed"
lacks b "$PB" "Epsilon"
has b "$scratch/b/pk/key/unresolved.tsv" "u1${tab}docs/e.md${tab}$DE"
has b "$scratch/b/pk/key/packets.txt" "unresolved (path, digest): 1"
has b "$scratch/b/pk/key/packets.txt" "unresolved documents left out of a list: 1"
has b "$scratch/b/pk/key/packets.txt" "resolved from history: 1"

# c: the draw. Three silent prompts (one a bare command, one with no word),
# two disjoint, three overlap, one under another digest, one outside the
# strata (the route offered, the embedding path was silent), and one typed
# after the cut.
i=0
for id in v1 v2 v3 j1 j2 o1 o2 o3 x1 n1 late; do
    i=$((i + 1))
    prompt c "2026-09-30T10:$(printf '%02d' "$i"):00.000Z" "$id"
done
logline c 2026-09-30T10:01:05Z v1 "Which decision governs the bound?" "$(route silent)" "$(neighbors "docs/a.md=$DA")"
logline c 2026-09-30T10:02:05Z v2 "  /frobnicate " "$(route silent)" "$(neighbors "docs/a.md=$DA")"
logline c 2026-09-30T10:03:05Z v3 "ok Q then 7 and 9" "$(route silent)" "$(neighbors "docs/a.md=$DA")"
logline c 2026-09-30T10:04:05Z j1 "Explain the merge queue rules" "$(route "docs/b.md=$SB")" "$(neighbors "docs/a.md=$DA")"
logline c 2026-09-30T10:05:05Z j2 "/frobnicate 9 --widgets 3" "$(route "docs/b.md=$SB")" "$(neighbors "docs/c.md=$DC1")"
logline c 2026-09-30T10:06:05Z o1 "Summarize the delta" "$(route "docs/d.md=$SD")" "$(neighbors "docs/d.md=$DD")"
logline c 2026-09-30T10:07:05Z o2 "Summarize the beta" "$(route "docs/b.md=$SB")" "$(neighbors "docs/b.md=$DB" "docs/a.md=$DA")"
logline c 2026-09-30T10:08:05Z o3 "Summarize the alpha" "$(route "docs/a.md=$SA")" "$(neighbors "docs/a.md=$DA")"
logline c 2026-09-30T10:09:05Z x1 "Another model entirely" "$(route silent)" "$(neighbors "docs/a.md=$DA")" sha256:other
logline c 2026-09-30T10:10:05Z n1 "Route only, nothing near" "$(route "docs/b.md=$SB")" "$(neighbors)"
logline c 2026-10-02T10:11:05Z late "After the cut" "$(route silent)" "$(neighbors "docs/a.md=$DA")"
rg c draw --until 2026-10-01T00:00:00Z --seed 42 --control 2 --owner 1
rc "c: draws" c.draw 0
D="$scratch/c/rec/draw.txt"
has c "$D" "seed: 42"
has c "$D" "until: 2026-10-01T00:00:00Z"
has c "$D" "joined: 10"
has c "$D" "under the digest: 9"
has c "$D" "stratum silent: 3, excluded 2, kept 1, drawn 1"
has c "$D" "stratum disjoint: 2, excluded 0, kept 2, drawn 2"
has c "$D" "stratum overlap: 3, excluded 0, kept 3, drawn 2"
has c "$D" "stratum none: 1, excluded 1, kept 0, drawn 0"
has c "$D" "excluded bare-command: 1"
has c "$D" "excluded no-word: 1"
has c "$D" "excluded outside-strata: 1"
has c "$D" "sample: 5"
has c "$D" "owner sample: 3"
has c "$scratch/c/rec/excluded.tsv" "v2${tab}silent${tab}bare-command"
has c "$scratch/c/rec/excluded.tsv" "v3${tab}silent${tab}no-word"
has c "$scratch/c/rec/excluded.tsv" "n1${tab}none${tab}outside-strata"
lacks c "$scratch/c/rec/sample.tsv" "late"
lacks c "$scratch/c/rec/sample.tsv" "x1"
# The two overlap prompts drawn are the first two of o1, o2, o3 by the
# seeded order, worked here.
for id in o1 o2 o3; do printf '%s\t%s\n' "$(sha "42${tab}$id")" "$id"; done | LC_ALL=C sort | head -n 2 | cut -f 2 > "$scratch/c.want"
awk -F '\t' '$2 == "overlap" { print $1 }' "$scratch/c/rec/sample.tsv" > "$scratch/c.got"
check "c: the control is drawn in sha256 order of seed and id" 'cmp -s "$scratch/c.want" "$scratch/c.got"' "want $(tr '\n' ' ' < "$scratch/c.want"), got $(tr '\n' ' ' < "$scratch/c.got")"
check "c: the owner's overlap prompt is the first of that order" '[ "$(awk -F "\t" "\$2 == \"overlap\" { print \$1 }" "$scratch/c/rec/owner-sample.tsv")" = "$(head -n 1 "$scratch/c.want")" ]'
TJ="$(printf '%s' 'Explain the merge queue rules' | sha256sum | cut -d ' ' -f 1)"
has c "$scratch/c/rec/sample.tsv" "j1${tab}disjoint${tab}2026-09-30T10:04:05Z${tab}S.jsonl${tab}4${tab}sha256:$TJ${tab}docs/b.md${tab}docs/a.md"
o2row=$(awk -F '\t' '$1 == "o2"' "$scratch/c/rec/sample.tsv" | cut -f 7,8)
if grep -qx o2 "$scratch/c.want"; then
    check "c: a sampled row names each path's offers in their logged order" '[ "$o2row" = "docs/b.md${tab}docs/b.md|docs/a.md" ]' "got $o2row"
fi
lacks c "$scratch/c/rec/sample.tsv" "Explain the merge"
cp "$scratch/c/rec/sample.tsv" "$scratch/c.s1"
cp "$D" "$scratch/c.d1"
rg c draw --until 2026-10-01T00:00:00Z --seed 42 --control 2 --owner 1
check "c: the same seed draws the same bytes" 'cmp -s "$scratch/c.s1" "$scratch/c/rec/sample.tsv" && cmp -s "$scratch/c.d1" "$D"'
rg c packets --seed 42
rc "c: writes its packets" c.packets 0
has c "$scratch/c/pk/key/packets.txt" "prompts: 5"
has c "$scratch/c/pk/key/packets.txt" "owner prompts: 3"
lacks c "$scratch/c/pk/rater-a/batch-01.md" "silent"
lacks c "$scratch/c/pk/rater-a/batch-01.md" "overlap"
lacks c "$scratch/c/pk/rater-a/batch-01.md" "disjoint"
# The prompts of a packet are in ascending order of
# sha256("<seed>\tpacket\t<prompt id>"), worked here, so no stratum groups.
awk -F '\t' 'NR > 1 { print $1 }' "$scratch/c/rec/sample.tsv" | while IFS= read -r id; do
    printf '%s\t%s\n' "$(sha "42${tab}packet${tab}$id")" "$id"
done | LC_ALL=C sort | cut -f 2 > "$scratch/c.porder"
awk -F '\t' '$1 == "rater" { print $3 }' "$scratch/c/pk/key/prompts.tsv" > "$scratch/c.pgot"
check "c: the packet orders its prompts by the seeded key" 'cmp -s "$scratch/c.porder" "$scratch/c.pgot"' "want $(tr '\n' ' ' < "$scratch/c.porder"), got $(tr '\n' ' ' < "$scratch/c.pgot")"
check "c: the owner packet opens by asking the owner to rate before opening the record" \
    '[ "$(head -n 1 "$scratch/c/pk/owner/packet.md")" = "Rate each prompt before you open tools/run/relevance-grade/ or any other" ]' \
    "got: $(head -n 1 "$scratch/c/pk/owner/packet.md")"
rg c packets --seed 42 --batch 2
has c "$scratch/c/pk/key/packets.txt" "batches: 3 of up to 2 prompts"
# A rewrite clears the packets it replaces, so no batch of the last run is
# left for a rater to open.
rg c packets --seed 42
check "c: a rewrite with one batch leaves no second batch of the rewrite before" \
    '[ ! -e "$scratch/c/pk/rater-a/batch-02.md" ] && [ ! -e "$scratch/c/pk/rater-b/batch-03.md" ]' \
    "found: $(ls "$scratch/c/pk/rater-a" | tr '\n' ' ')"

# d: scores for case a's prompt s1, by path, written through the key.
K="$scratch/a1/pk/key/documents.tsv"
lab() { awk -F '\t' -v p="docs/$1.md" '$3 == p { print $2 }' "$K"; }
scores() {
    printf 'P1\t%s\t%s\nP1\t%s\t%s\nP1\t%s\t%s\nP1\t%s\t%s\nP1\tmissing\t%s\n' \
        "$(lab a)" "$1" "$(lab b)" "$2" "$(lab c)" "$3" "$(lab d)" "$4" "$5"
}
scores 2 1 0 0 0 > "$scratch/a1/pk/rater-a/scores-01.tsv"
scores 2 0 0 2 1 > "$scratch/a1/pk/rater-b/scores-01.tsv"
rg a1 ingest a
rc "d: ingests rater a" a1.ingest 0
has d "$scratch/a1/rec/scores-a.tsv" "s1${tab}docs/b.md${tab}1"
has d "$scratch/a1/rec/scores-a.tsv" "s1${tab}missing${tab}0"
lacks d "$scratch/a1/rec/scores-a.tsv" "${tab}D"
rg a1 ingest b
rg a1 agreement
rc "d: agreement runs" a1.agreement 0
G="$scratch/a1/rec/agreement.txt"
# a: 2 1 0 0, b: 2 0 0 2. Weighted disagreement observed 1.25/4 = 0.3125,
# expected 0.4375, so kappa = 1 - 0.3125/0.4375 = 0.2857.
has d "$G" "stratum overlap exact agreement on scores: 2 of 4 (0.5000)"
has d "$G" "stratum overlap quadratic-weighted kappa on scores: 0.2857"
has d "$G" "stratum overlap exact agreement on missing: 0 of 1 (0.0000)"
has d "$G" "stratum overlap kappa on missing: 0.0000"
has d "$G" "stratum silent documents: 0"
has d "$G" "stratum all quadratic-weighted kappa on scores: 0.2857"
has d "$G" "disagreements: 3"
has d "$scratch/a1/rec/disagreements.tsv" "s1${tab}docs/d.md${tab}overlap"
rg a1 third --seed 7
rc "d: writes the third packet" a1.third 0
T3="$scratch/a1/pk/rater-third/batch-01.md"
check "d: the third pass rates only the items in dispute" 'grep -qx "rate only: $(lab b) $(lab d) missing" "$T3" || grep -qx "rate only: $(lab d) $(lab b) missing" "$T3"' "got: $(grep '^rate only' "$T3")"
has d "$T3" "$TASK"
printf 'P1\t%s\t2\nP1\t%s\t1\nP1\tmissing\t1\n' "$(lab b)" "$(lab d)" > "$scratch/a1/pk/rater-third/scores-01.tsv"
rg a1 ingest third
rc "d: ingests the third pass" a1.ingest 0
rg a1 consensus
rc "d: consensus runs" a1.consensus 0
C="$scratch/a1/rec/consensus.tsv"
has d "$C" "s1${tab}docs/a.md${tab}2${tab}agreed"
has d "$C" "s1${tab}docs/b.md${tab}1${tab}third"
has d "$C" "s1${tab}docs/c.md${tab}0${tab}agreed"
has d "$C" "s1${tab}docs/d.md${tab}1${tab}third"
has d "$C" "s1${tab}missing${tab}1${tab}third"

# f: the edges of the draw. f1 holds one word of three letters outside the
# reply list, so it is kept. f2 is silent and its embedding path offered
# nothing, so it is excluded as empty-list. f3's route offers a path that
# holds the embedding path's one path as a part, and the two share no
# document, so f3 is disjoint.
i=0
for id in f1 f2 f3; do
    i=$((i + 1))
    prompt f "2026-09-30T10:0$i:00.000Z" "$id"
done
logline f 2026-09-30T10:01:05Z f1 "fix it" "$(route silent)" "$(neighbors "docs/a.md=$DA")"
logline f 2026-09-30T10:02:05Z f2 "Where does the gamma rule live?" "$(route silent)" "$(neighbors)"
logline f 2026-09-30T10:03:05Z f3 "Which page holds the alpha rule?" "$(route "docs/old/docs/a.md=$SA")" "$(neighbors "docs/a.md=$DA")"
rg f draw --until 2026-10-01T00:00:00Z --seed 7
rc "f: draws" f.draw 0
has f "$scratch/f/rec/draw.txt" "stratum silent: 2, excluded 1, kept 1, drawn 1"
has f "$scratch/f/rec/draw.txt" "stratum disjoint: 1, excluded 0, kept 1, drawn 1"
has f "$scratch/f/rec/draw.txt" "stratum overlap: 0, excluded 0, kept 0, drawn 0"
has f "$scratch/f/rec/draw.txt" "excluded empty-list: 1"
has f "$scratch/f/rec/draw.txt" "excluded no-word: 0"
has f "$scratch/f/rec/excluded.tsv" "f2${tab}silent${tab}empty-list"
check "f: a task with one three-letter word outside the reply list is kept" 'awk -F "\t" "\$1 == \"f1\"" "$scratch/f/rec/sample.tsv" | grep -q .'

# g: the third pass writes batches of twelve prompts. Thirteen silent
# prompts each list one document, and the two raters differ on each.
i=0
for n in 01 02 03 04 05 06 07 08 09 10 11 12 13; do
    prompt g "2026-09-30T10:$n:00.000Z" "g$n"
    logline g "2026-09-30T10:$n:05Z" "g$n" "Which rule bounds case $n?" "$(route silent)" "$(neighbors "docs/a.md=$DA")"
done
rg g draw --until 2026-10-01T00:00:00Z --seed 7
rg g packets --seed 7 --batch 20
awk -F '\t' '$1 == "rater" { print $2 "\tD1\t0\n" $2 "\tmissing\t0" }' "$scratch/g/pk/key/prompts.tsv" > "$scratch/g/pk/rater-a/scores-01.tsv"
awk -F '\t' '$1 == "rater" { print $2 "\tD1\t1\n" $2 "\tmissing\t0" }' "$scratch/g/pk/key/prompts.tsv" > "$scratch/g/pk/rater-b/scores-01.tsv"
rg g ingest a
rg g ingest b
rg g agreement
rg g third --seed 7
rc "g: writes the third packet" g.third 0
has g "$scratch/g.third.out" "third pass: 13 prompts, 13 items"
check "g: the third pass puts twelve prompts in its first batch and one in its second" \
    '[ "$(grep -c "^rate only: " "$scratch/g/pk/rater-third/batch-01.md")" = 12 ] && [ "$(grep -c "^rate only: " "$scratch/g/pk/rater-third/batch-02.md")" = 1 ]' \
    "got: $(grep -c '^rate only: ' "$scratch/g/pk/rater-third"/batch-*.md | tr '\n' ' ')"

# s: the seal. The route offers a probe and the embedding path offers B.
# The record names the probe by its token, the key keeps its path, the
# packet renders it, and the third pass and the consensus read it through
# the token.
SP='Probe summary.'
TOKEN="sealed:$(sha docs/probes/p.md | cut -c 1-16)"
prompt s 2026-09-30T10:00:00.000Z w1
logline s 2026-09-30T10:00:05Z w1 "$TASK" "$(route "docs/probes/p.md=$SP")" "$(neighbors "docs/b.md=$DB")"
rg s draw --until 2026-10-01T00:00:00Z --seed 7
rg s packets --seed 7
rc "s: writes its packets" s.packets 0
lacks s "$scratch/s/rec/sample.tsv" "probes"
check "s: the sample names the probe by its token" 'awk -F "\t" "\$1 == \"w1\" { print \$7 }" "$scratch/s/rec/sample.tsv" | grep -Fqx "$TOKEN"' \
    "got: $(awk -F '\t' '$1 == "w1"' "$scratch/s/rec/sample.tsv")"
has s "$scratch/s/pk/rater-a/batch-01.md" "    $SP"
check "s: the key keeps the probe's path" 'grep -Fq "${tab}docs/probes/p.md" "$scratch/s/pk/key/documents.tsv"'
SK="$scratch/s/pk/key/documents.tsv"
slab() { awk -F '\t' -v p="$1" '$3 == p { print $2 }' "$SK"; }
printf 'P1\t%s\t0\nP1\t%s\t1\nP1\tmissing\t0\n' "$(slab docs/probes/p.md)" "$(slab docs/b.md)" > "$scratch/s/pk/rater-a/scores-01.tsv"
printf 'P1\t%s\t2\nP1\t%s\t1\nP1\tmissing\t0\n' "$(slab docs/probes/p.md)" "$(slab docs/b.md)" > "$scratch/s/pk/rater-b/scores-01.tsv"
rg s ingest a
rg s ingest b
rc "s: ingests rater b" s.ingest 0
for f in scores-a.tsv scores-b.tsv; do lacks s "$scratch/s/rec/$f" "probes"; done
has s "$scratch/s/rec/scores-a.tsv" "w1${tab}$TOKEN${tab}0"
rg s agreement
has s "$scratch/s/rec/disagreements.tsv" "w1${tab}$TOKEN${tab}disjoint"
rg s third --seed 7
has s "$scratch/s/pk/rater-third/batch-01.md" "rate only: $(slab docs/probes/p.md) "
printf 'P1\t%s\t1\n' "$(slab docs/probes/p.md)" > "$scratch/s/pk/rater-third/scores-01.tsv"
rg s ingest third
rc "s: ingests the third pass" s.ingest 0
rg s consensus
has s "$scratch/s/rec/consensus.tsv" "w1${tab}$TOKEN${tab}1${tab}third"
for f in "$scratch/s/rec"/*; do lacks s "$f" "docs/probes/"; done

# e: the arguments, a short score file, and no `jq`.
rg a1 draw --seed 7
rc "e: a draw with no --until exits 2" a1.draw 2
rg a1 nonsense
rc "e: an unknown subcommand exits 2" a1.nonsense 2
scores 2 1 0 0 0 | sed '$d' > "$scratch/a1/pk/rater-a/scores-01.tsv"
rg a1 ingest a
rc "e: a score file that leaves an item unscored exits 4" a1.ingest 4
scores 2 1 0 0 0 > "$scratch/a1/pk/rater-a/scores-01.tsv"
printf 'P1\t%s\t1\n' "$(lab a)" >> "$scratch/a1/pk/rater-a/scores-01.tsv"
rg a1 ingest a
rc "e: a score file that scores an item twice exits 4" a1.ingest 4
scores 2 1 0 0 3 > "$scratch/a1/pk/rater-a/scores-01.tsv"
rg a1 ingest a
rc "e: a missing flag that is not 0 or 1 exits 4" a1.ingest 4
scores 2 1 0 0 2 > "$scratch/a1/pk/rater-a/scores-01.tsv"
rg a1 ingest a
rc "e: a missing flag of 2 exits 4" a1.ingest 4
scores 2 1 0 3 0 > "$scratch/a1/pk/rater-a/scores-01.tsv"
rg a1 ingest a
rc "e: a score of 3 exits 4" a1.ingest 4
scores 2 1 0 0 0 > "$scratch/a1/pk/rater-a/scores-01.tsv"
printf 'P1\tD9\t1\n' >> "$scratch/a1/pk/rater-a/scores-01.tsv"
rg a1 ingest a
rc "e: a score of a label the packet does not list exits 4" a1.ingest 4
scores 2 1 0 0 0 > "$scratch/a1/pk/rater-a/scores-01.tsv"
printf 'P7\tmissing\t0\n' >> "$scratch/a1/pk/rater-a/scores-01.tsv"
rg a1 ingest a
rc "e: a flag of a prompt the packet does not hold exits 4" a1.ingest 4
mkdir -p "$scratch/nojq"
PATH="$scratch/nojq" /bin/sh "$tool" draw > "$scratch/e.out" 2> "$scratch/e.err"
echo $? > "$scratch/e.rc"
rc "e: no jq exits 3" e 3

# k: the grade. Each record is planted by hand, with no draw, so the answer
# is worked from the rows alone. The Wilson bounds were worked by hand: for
# k = n = 1 the lower bound is 1 / (1 + 1.96^2) = 0.2065, and for k = 0, n = 1
# the upper bound is 1.96^2 / (1 + 1.96^2) = 0.7935.
krec() { mkdir -p "$scratch/$1/rec"; printf 'prompt_id\tstratum\tat\tfile\tline\ttask_digest\tdeterministic\tembedding\n' > "$scratch/$1/rec/sample.tsv"; printf 'prompt_id\titem\tvalue\tsettled_by\n' > "$scratch/$1/rec/consensus.tsv"; }
# ksample <case> <id> <stratum> <route offer> <embedding offer>
ksample() { printf '%s\t%s\t2026-09-30T10:00:05Z\tS.jsonl\t1\tsha256:t\t%s\t%s\n' "$2" "$3" "$4" "$5" >> "$scratch/$1/rec/sample.tsv"; }
# kscore <case> <id> <item> <value>
kscore() { printf '%s\t%s\t%s\tagreed\n' "$2" "$3" "$4" >> "$scratch/$1/rec/consensus.tsv"; }

# k1, the decisive case: one disjoint prompt whose route offers A and B and
# whose embedding path offers B, C and D. A=2, B=1, C=0, D=0, missing=1. B
# is credited to both paths.
krec k1
ksample k1 p1 disjoint 'docs/a.md|docs/b.md' 'docs/b.md|docs/c.md|docs/d.md'
kscore k1 p1 docs/a.md 2
kscore k1 p1 docs/b.md 1
kscore k1 p1 docs/c.md 0
kscore k1 p1 docs/d.md 0
kscore k1 p1 missing 1
rg k1 grade
rc "k1: grade exits 0" k1.grade 0
G="$scratch/k1/rec/grade.txt"
has k1 "$G" 'stratum disjoint path route: offered on 1 of 1 prompts, 2 documents'
has k1 "$G" 'stratum disjoint path route score 2: documents 1 of 2 (0.5000); prompts 1 of 1 (1.0000, Wilson 95% 0.2065 to 1.0000); first offered 1 of 1 (1.0000)'
has k1 "$G" 'stratum disjoint path route score 1 or more: documents 2 of 2 (1.0000); prompts 1 of 1 (1.0000, Wilson 95% 0.2065 to 1.0000); first offered 1 of 1 (1.0000)'
has k1 "$G" 'stratum disjoint path embedding: offered on 1 of 1 prompts, 3 documents'
has k1 "$G" 'stratum disjoint path embedding score 2: documents 0 of 3 (0.0000); prompts 0 of 1 (0.0000, Wilson 95% 0.0000 to 0.7935); first offered 0 of 1 (0.0000)'
has k1 "$G" 'stratum disjoint path embedding score 1 or more: documents 1 of 3 (0.3333); prompts 1 of 1 (1.0000, Wilson 95% 0.2065 to 1.0000); first offered 1 of 1 (1.0000)'
has k1 "$G" 'stratum disjoint missing: 1 of 1 prompts (1.0000)'
has k1 "$G" 'stratum silent path route: offered on 0 of 0 prompts, 0 documents'
lacks k1 "$G" 'stratum all'

# k2: the same prompt with the two offer columns swapped. The route and
# embedding lines exchange their figures, and no other byte moves.
krec k2
ksample k2 p1 disjoint 'docs/b.md|docs/c.md|docs/d.md' 'docs/a.md|docs/b.md'
tail -n +2 "$scratch/k1/rec/consensus.tsv" >> "$scratch/k2/rec/consensus.tsv"
rg k2 grade
rc "k2: grade exits 0" k2.grade 0
sed 's/ path route/ path X/; s/ path embedding/ path route/; s/ path X/ path embedding/' "$scratch/k2/rec/grade.txt" | LC_ALL=C sort > "$scratch/k2.relabelled"
LC_ALL=C sort "$G" > "$scratch/k1.sorted"
check "k2: swapping the offer columns swaps the two paths' lines and nothing else" 'cmp -s "$scratch/k1.sorted" "$scratch/k2.relabelled"' "$(diff "$scratch/k1.sorted" "$scratch/k2.relabelled" | head -n 6 | tr '\n' '|')"
check "k2: the swap moves the figures" '! cmp -s "$G" "$scratch/k2/rec/grade.txt"'

# k3: the gate. A record with one document unscored, one prompt with no
# `missing` value, or a value outside 0, 1 and 2 exits 4 and writes no
# grade.txt. No consensus or no sample exits 2.
for c in k3a k3b k3c k3d; do krec "$c"; ksample "$c" p1 disjoint 'docs/a.md|docs/b.md' 'docs/b.md|docs/c.md|docs/d.md'; done
grep -v 'docs/d.md' "$scratch/k1/rec/consensus.tsv" > "$scratch/k3a/rec/consensus.tsv"
grep -v 'missing' "$scratch/k1/rec/consensus.tsv" > "$scratch/k3b/rec/consensus.tsv"
sed 's/docs\/c.md\t0/docs\/c.md\t3/' "$scratch/k1/rec/consensus.tsv" > "$scratch/k3c/rec/consensus.tsv"
cp "$scratch/k1/rec/consensus.tsv" "$scratch/k3d/rec/consensus.tsv"
kscore k3d p9 docs/a.md 2
for c in k3a k3b k3c k3d; do rg "$c" grade; done
rc "k3a: an offered document with no consensus value exits 4" k3a.grade 4
check "k3a: and writes no grade.txt" '[ ! -e "$scratch/k3a/rec/grade.txt" ]'
rc "k3b: a sampled prompt with no missing value exits 4" k3b.grade 4
check "k3b: and writes no grade.txt" '[ ! -e "$scratch/k3b/rec/grade.txt" ]'
rc "k3c: a consensus value of 3 exits 4" k3c.grade 4
check "k3c: and writes no grade.txt" '[ ! -e "$scratch/k3c/rec/grade.txt" ]'
rc "k3d: a consensus item for a prompt the sample does not hold exits 4" k3d.grade 4
check "k3d: and writes no grade.txt" '[ ! -e "$scratch/k3d/rec/grade.txt" ]'
krec k3e
rm "$scratch/k3e/rec/consensus.tsv"
rg k3e grade
rc "k3e: no consensus.tsv exits 2" k3e.grade 2
krec k3f
rm "$scratch/k3f/rec/sample.tsv"
rg k3f grade
rc "k3f: no sample.tsv exits 2" k3f.grade 2

# k4: a silent prompt, whose route column is empty, counts toward no route
# denominator, and its embedding documents count in the silent stratum. Its
# first neighbor is a sealed probe, graded like a path.
krec k4
ksample k4 p1 disjoint 'docs/a.md|docs/b.md' 'docs/b.md|docs/c.md|docs/d.md'
ksample k4 p2 silent '' 'sealed:0123456789abcdef|docs/e.md'
tail -n +2 "$scratch/k1/rec/consensus.tsv" >> "$scratch/k4/rec/consensus.tsv"
kscore k4 p2 sealed:0123456789abcdef 2
kscore k4 p2 docs/e.md 0
kscore k4 p2 missing 0
rg k4 grade
rc "k4: grade exits 0" k4.grade 0
G4="$scratch/k4/rec/grade.txt"
has k4 "$G4" 'stratum silent path route: offered on 0 of 1 prompts, 0 documents'
has k4 "$G4" 'stratum silent path route score 2: documents 0 of 0 (n/a); prompts 0 of 0 (n/a, Wilson 95% n/a); first offered 0 of 0 (n/a)'
has k4 "$G4" 'stratum silent path embedding: offered on 1 of 1 prompts, 2 documents'
has k4 "$G4" 'stratum silent path embedding score 2: documents 1 of 2 (0.5000); prompts 1 of 1 (1.0000, Wilson 95% 0.2065 to 1.0000); first offered 1 of 1 (1.0000)'
has k4 "$G4" 'stratum silent missing: 0 of 1 prompts (0.0000)'
has k4 "$G4" 'stratum disjoint path route: offered on 1 of 1 prompts, 2 documents'

# k5: a rerun writes the same bytes.
s1=$(sha256sum < "$G4")
rg k4 grade
check "k5: a rerun writes the same bytes" '[ "$s1" = "$(sha256sum < "$G4")" ]'

# k6: three strata at once. Document A is scored 2 on the disjoint prompt
# p1 and 0 on the overlap prompt p3, so a value is looked up by prompt and
# item together. The silent prompt p2 flags `missing`, which counts in its
# stratum although the route offered nothing there.
krec k6
ksample k6 p1 disjoint 'docs/a.md|docs/b.md' 'docs/b.md|docs/c.md|docs/d.md'
ksample k6 p2 silent '' 'docs/g.md|docs/h.md'
ksample k6 p3 overlap 'docs/a.md|docs/e.md' 'docs/a.md|docs/f.md'
tail -n +2 "$scratch/k1/rec/consensus.tsv" >> "$scratch/k6/rec/consensus.tsv"
kscore k6 p2 docs/g.md 0
kscore k6 p2 docs/h.md 1
kscore k6 p2 missing 1
kscore k6 p3 docs/a.md 0
kscore k6 p3 docs/e.md 2
kscore k6 p3 docs/f.md 1
kscore k6 p3 missing 0
rg k6 grade
rc "k6: grade exits 0" k6.grade 0
G6="$scratch/k6/rec/grade.txt"
has k6 "$G6" 'stratum disjoint path route score 2: documents 1 of 2 (0.5000); prompts 1 of 1 (1.0000, Wilson 95% 0.2065 to 1.0000); first offered 1 of 1 (1.0000)'
has k6 "$G6" 'stratum overlap: 1 prompts'
has k6 "$G6" 'stratum overlap path route: offered on 1 of 1 prompts, 2 documents'
has k6 "$G6" 'stratum overlap path route score 2: documents 1 of 2 (0.5000); prompts 1 of 1 (1.0000, Wilson 95% 0.2065 to 1.0000); first offered 0 of 1 (0.0000)'
has k6 "$G6" 'stratum overlap path route score 1 or more: documents 1 of 2 (0.5000); prompts 1 of 1 (1.0000, Wilson 95% 0.2065 to 1.0000); first offered 0 of 1 (0.0000)'
has k6 "$G6" 'stratum overlap path embedding score 2: documents 0 of 2 (0.0000); prompts 0 of 1 (0.0000, Wilson 95% 0.0000 to 0.7935); first offered 0 of 1 (0.0000)'
has k6 "$G6" 'stratum overlap path embedding score 1 or more: documents 1 of 2 (0.5000); prompts 1 of 1 (1.0000, Wilson 95% 0.2065 to 1.0000); first offered 0 of 1 (0.0000)'
has k6 "$G6" 'stratum overlap missing: 0 of 1 prompts (0.0000)'
has k6 "$G6" 'stratum silent path embedding score 1 or more: documents 1 of 2 (0.5000); prompts 1 of 1 (1.0000, Wilson 95% 0.2065 to 1.0000); first offered 0 of 1 (0.0000)'
has k6 "$G6" 'stratum silent missing: 1 of 1 prompts (1.0000)'
# Both thresholds are named in the header, and the waiver is stated.
has k6 "$G6" '# Both thresholds are printed and neither is chosen: score 2 (would need) and score 1 or more (perhaps).'
has k6 "$G6" '# The gate did not wait on owner scores: the owner waived the spot check on 2026-10-03.'

# k7: the rest of the gate, each case on a copy of k1's record: a `missing`
# flag of 2, a stratum outside the three, an item scored twice, a document
# listed twice in one offer, a sample row of seven columns and a consensus
# row of three. Each exits 4 and writes no grade.txt.
for c in k7a k7b k7c k7d k7e k7f; do krec "$c"; done
for c in k7a k7c k7f; do ksample "$c" p1 disjoint 'docs/a.md|docs/b.md' 'docs/b.md|docs/c.md|docs/d.md'; done
ksample k7b p1 Disjoint 'docs/a.md|docs/b.md' 'docs/b.md|docs/c.md|docs/d.md'
ksample k7d p1 disjoint 'docs/a.md|docs/b.md' 'docs/b.md|docs/c.md|docs/d.md|docs/b.md'
printf 'p1\tdisjoint\t2026-09-30T10:00:05Z\tS.jsonl\t1\tsha256:t\tdocs/a.md|docs/b.md\n' >> "$scratch/k7e/rec/sample.tsv"
sed 's/missing\t1/missing\t2/' "$scratch/k1/rec/consensus.tsv" > "$scratch/k7a/rec/consensus.tsv"
for c in k7b k7c k7d; do cp "$scratch/k1/rec/consensus.tsv" "$scratch/$c/rec/consensus.tsv"; done
kscore k7c p1 docs/c.md 0
# k7e scores only what its short row offers, and k7f scores every offered
# item, so each is refused by the column test alone.
grep -v -e 'docs/c.md' -e 'docs/d.md' "$scratch/k1/rec/consensus.tsv" > "$scratch/k7e/rec/consensus.tsv"
grep -v 'docs/d.md' "$scratch/k1/rec/consensus.tsv" > "$scratch/k7f/rec/consensus.tsv"
printf 'p1\tdocs/d.md\t0\n' >> "$scratch/k7f/rec/consensus.tsv"
for c in k7a k7b k7c k7d k7e k7f; do rg "$c" grade; done
rc "k7a: a missing flag of 2 exits 4" k7a.grade 4
rc "k7b: a stratum outside the three exits 4" k7b.grade 4
rc "k7c: an item scored twice exits 4" k7c.grade 4
rc "k7d: a document listed twice in one offer exits 4" k7d.grade 4
rc "k7e: a sample row of seven columns exits 4" k7e.grade 4
rc "k7f: a consensus row of three columns exits 4" k7f.grade 4
check "k7: and none writes grade.txt" '[ ! -e "$scratch/k7a/rec/grade.txt" ] && [ ! -e "$scratch/k7b/rec/grade.txt" ] && [ ! -e "$scratch/k7c/rec/grade.txt" ] && [ ! -e "$scratch/k7d/rec/grade.txt" ] && [ ! -e "$scratch/k7e/rec/grade.txt" ] && [ ! -e "$scratch/k7f/rec/grade.txt" ]'

# k8: the committed grade.txt is what `grade` prints on the committed
# record, so an edit to the record that leaves it stale fails here.
mkdir -p "$scratch/k8/rec"
cp "$root/tools/run/relevance-grade/sample.tsv" "$root/tools/run/relevance-grade/consensus.tsv" "$scratch/k8/rec/"
rg k8 grade
rc "k8: grade exits 0 on the committed record" k8.grade 0
check "k8: the committed grade.txt is what grade prints on the committed record" 'cmp -s "$scratch/k8/rec/grade.txt" "$root/tools/run/relevance-grade/grade.txt"'

printf '%d passed, %d failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
