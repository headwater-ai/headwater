#!/bin/sh
# Grade the documents that the two routing paths offered, blind: #1670.
#
# HW-DR-0064 logs, for each person prompt, the documents the deterministic
# route offered and the documents the embedding path would have offered. A
# reading of that log tells how often each path speaks. It does not tell
# whether what a path said was of use. This tool draws a sample of logged
# prompts, writes blind packets in which a rater cannot tell which path
# offered a document, and folds the raters' scores into a consensus. It needs
# only `jq`, `awk`, `git` and `sha256sum`, like its siblings.
#
#     sh tools/run/relevance-grade.sh draw --until <ISO time> --seed <s> [--digest <d>] [--control <n>] [--disjoint <n>] [--owner <n>]
#     sh tools/run/relevance-grade.sh packets --seed <s> [--batch <n>]
#     sh tools/run/relevance-grade.sh ingest <a|b|third|owner>
#     sh tools/run/relevance-grade.sh agreement
#     sh tools/run/relevance-grade.sh third --seed <s>
#     sh tools/run/relevance-grade.sh consensus
#
# The procedure, in order. Each step is a commit, so the order is on record.
#
#  1. `draw` reads the join of `tools/run/shadow-mine.sh --until <t> --rows`,
#     so that the person-prompt rule has one copy. It keeps the joined
#     prompts under one `model_digest` (default the evaluation's model,
#     sha256:f342437e...8e44) and reads each from its earliest log line. It
#     puts each prompt in one stratum:
#       silent    the route was silent
#       disjoint  both paths offered, and no document is in both offers
#       overlap   both paths offered, and at least one document is in both
#     A prompt in none of the three is excluded as `outside-strata`. Then the
#     exclusion rule below runs, before any draw and before any rating. The
#     draw keeps every `silent` and every `disjoint` prompt (or `--disjoint
#     <n>` of them) and `--control <n>` (default 60) `overlap` prompts. Each
#     draw takes the prompts in ascending order of sha256("<seed>\t<prompt
#     id>"), never an `awk` random number, because mawk and gawk give two
#     sequences for one seed. The owner's spot check is the first `--owner
#     <n>` (default 10) prompts of each stratum of the sample in that order.
#     It writes `sample.tsv`, `excluded.tsv`, `owner-sample.tsv` and
#     `draw.txt` to the record directory, and prints the sizes, the seed and
#     the count excluded in each stratum. The sample carries a digest of each
#     prompt's text and never the text: the texts are a person's own typed
#     prompts, they hold home paths, and the repository is public.
#
#     The exclusion rule. A prompt is excluded when
#       bare-command  its text, trimmed, is one slash command with nothing
#                     after it, such as `/compact`;
#       no-word       its text holds no word of three or more letters outside
#                     the reply list: yes no ok okay go done wait test
#                     continue clear status restart both then and the them
#                     push merge. A word is a run of the letters a to z,
#                     after the text is put in lower case. So `b`, `#`, `2`,
#                     `1 and 3` and `ok A then` are excluded;
#       empty-list    neither path offered a document.
#
#  2. `packets` writes the blind packets outside the tree, to the packet
#     directory: `rater-a/` and `rater-b/` hold the same batches of the whole
#     sample, `owner/` holds the owner's prompts, and `key/` holds the labels.
#     A prompt shows its task exactly as the log line's `task` holds it, and
#     then one list of documents. A document that both paths offered appears
#     once. Each document shows a title and a summary, and an opaque label
#     `D1`, `D2` and so on in ascending order of
#     sha256("<seed>\t<prompt id>\t<path>"). The list holds no path, no rank,
#     no score and nothing that names a path.
#
#     The summary source. A route pointer carries its summary inline, and an
#     embedding neighbor carries only `summary_digest`, the sha256 of the
#     summary text. So every document, from either path, is rendered in one
#     way: its digest (a route summary is digested first) names the version
#     in `git log --all -- <path>` whose frontmatter `summary` has that
#     digest, and the title and summary are read from that version. Where
#     both paths offered one document under two digests, the smaller digest
#     is read. A (path, digest) that no version holds is listed in
#     `key/unresolved.tsv` and counted, and that document is left out of the
#     list. It is never rendered from the current text.
#
#     Prompts are put in packet order by sha256("<seed>\tpacket\t<prompt
#     id>"), so that no stratum is grouped, and labelled `P1`, `P2` and so on
#     (`O1` and so on in the owner's packet). A rater writes one score file
#     per batch beside it, `scores-<batch>.tsv`, with one row per document,
#     `P<k>\tD<j>\t<0|1|2>`, and one row per prompt, `P<k>\tmissing\t<0|1>`.
#     A score is whether a session answering the prompt would need the
#     document: 0 no, 1 perhaps, 2 yes. `missing` is 1 when a document such a
#     session would need is not in the list.
#
#  3. `ingest <rater>` maps a rater's score files back through the key and
#     writes `scores-<rater>.tsv` to the record directory: `prompt_id`,
#     `item` (a path, or `missing`) and `value`. A rater never writes a path.
#     It refuses a file that leaves a document unscored or scores one twice.
#
#  4. `agreement` reads `scores-a.tsv` and `scores-b.tsv`. For each stratum,
#     and over all, it reports the share of exact agreement and the
#     quadratic-weighted Cohen's kappa on the 0/1/2 scores, and Cohen's kappa
#     on the `missing` flag, to `agreement.txt`. It lists each item on which
#     the two differ in `disagreements.tsv`.
#
#  5. `third` writes a packet of the disputed prompts to `rater-third/`. It
#     shows the whole list for context, names the labels to score, and shows
#     neither earlier score. `ingest third` reads it back.
#
#  6. `consensus` writes `consensus.tsv`: the agreed value where the two
#     raters agree, and otherwise the median of the three, with `settled_by`
#     `agreed` or `third`. The column also admits `owner`, for an item the
#     owner settles.
#
# Where things live. Each has a variable, which the fixtures use:
#     HEADWATER_RELEVANCE_RECORD   the committed record
#                                  (default tools/run/relevance-grade)
#     HEADWATER_RELEVANCE_PACKETS  the packets, outside the tree
#                                  (default <git common dir>/headwater-relevance-grade)
#     HEADWATER_RELEVANCE_HISTORY  the git repository whose history holds
#                                  the summaries (default this checkout)
#     HEADWATER_SHADOW_LOG_DIR     the shadow log, as shadow-mine.sh reads it
#     HEADWATER_TRANSCRIPT_DIRS    the transcripts, as shadow-mine.sh reads them
#     SHADOW_MINE_TOOL             shadow-mine.sh itself
#
# A non-zero exit is an argument or a missing input (2), no `jq` (3), or a
# score file that does not cover its batch (4).
#
# `sh tools/run/relevance-grade-fixtures.sh` holds it.

set -u

here=$(cd "$(dirname "$0")/../.." && pwd)
tab=$(printf '\t')

die() { echo "relevance-grade: $1" >&2; exit "${2:-2}"; }
usage() {
    sed -n '/^#     sh tools\/run\/relevance-grade.sh/s/^#     /usage: /p' "$0" >&2
    exit 2
}

if ! command -v jq >/dev/null 2>&1; then
    die "no \`jq\` on the path" 3
fi

common_dir() {
    _c=$(git -C "$here" rev-parse --git-common-dir 2>/dev/null) || die "not inside a git checkout"
    case $_c in
        /*) ;;
        *) _c="$here/$_c" ;;
    esac
    printf '%s' "$_c"
}

record=${HEADWATER_RELEVANCE_RECORD:-$here/tools/run/relevance-grade}
packets=${HEADWATER_RELEVANCE_PACKETS:-}
history=${HEADWATER_RELEVANCE_HISTORY:-$here}
mine=${SHADOW_MINE_TOOL:-$here/tools/run/shadow-mine.sh}
log=${HEADWATER_SHADOW_LOG_DIR:-}
evaluation_digest=sha256:f342437e6e5f16aa1b759f8a329adcfc9328236d2caed6c8f289f7364b1e8e44

need_log() {
    if [ -z "$log" ]; then
        log="$(common_dir)/headwater-shadow-log"
    fi
    [ -d "$log" ] || die "no log directory at $log"
}
need_packets() {
    if [ -z "$packets" ]; then
        packets="$(common_dir)/headwater-relevance-grade"
    fi
}

work=$(mktemp -d) || exit 1
[ -n "${RG_KEEP:-}" ] || trap 'rm -rf "$work"' EXIT HUP INT TERM

# sha <string>: the hex sha256 of the string's bytes.
sha() { printf '%s' "$1" | sha256sum | cut -d ' ' -f 1; }

# line_of <file> <line>: one log line, to $work/line.json.
line_of() {
    sed -n "${2}p" "$log/$1" > "$work/line.json"
}

# The jq definitions every read of a log line uses.
JQDEF='def doc: if type == "string" then (try fromjson catch null) else . end;'

# A document of either path as `path<TAB>digest`, one per line, in the
# order the path logged it. A route summary is digested here, so that both
# paths reach the renderer as the same pair.
offers() {
    jq -r "$JQDEF"' (.route | doc) as $r
        | if ($r | type) == "object" and ($r.silence // null) == null
          then $r.pointers[]? | "\(.path)\t\(.summary // "" | tostring | tojson)" else empty end' "$work/line.json" > "$work/route.raw"
    : > "$work/route.tsv"
    while IFS="$tab" read -r _p _s; do
        printf '%s\tsha256:%s\n' "$_p" "$(jq -nj --argjson s "$_s" '$s' | sha256sum | cut -d ' ' -f 1)" >> "$work/route.tsv"
    done < "$work/route.raw"
    jq -r "$JQDEF"' (.neighbors | doc) as $n
        | if ($n | type) == "object" then $n.neighbors[]? | [.path, (.summary_digest // "")] | @tsv else empty end' "$work/line.json" > "$work/emb.tsv"
}

# exclusion <task file>: the reason a task is excluded, or nothing.
exclusion() {
    LC_ALL=C awk '
        BEGIN { split("yes no ok okay go done wait test continue clear status restart both then and the them push merge", r, " "); for (i in r) reply[r[i]] = 1 }
        { t = t (NR > 1 ? " " : "") $0 }
        END {
            s = t; sub(/^[ \t\r]+/, "", s); sub(/[ \t\r]+$/, "", s)
            if (s ~ /^\/[A-Za-z0-9:_-]+$/) { print "bare-command"; exit }
            s = tolower(s); gsub(/[^a-z]+/, " ", s)
            n = split(s, w, " ")
            for (i = 1; i <= n; i++) if (length(w[i]) >= 3 && !(w[i] in reply)) exit
            print "no-word"
        }' "$1"
}

cmd_draw() {
    until=; seed=; digest=$evaluation_digest; control=60; disjoint=all; owner=10
    while [ $# -gt 0 ]; do
        [ $# -ge 2 ] || usage
        case $1 in
            --until) until=$2 ;;
            --seed) seed=$2 ;;
            --digest) digest=$2 ;;
            --control) control=$2 ;;
            --disjoint) disjoint=$2 ;;
            --owner) owner=$2 ;;
            *) usage ;;
        esac
        shift 2
    done
    [ -n "$until" ] && [ -n "$seed" ] || usage
    for _n in "$control" "$owner"; do
        case $_n in '' | *[!0-9]*) usage ;; esac
    done
    case $disjoint in all) ;; '' | *[!0-9]*) usage ;; esac
    need_log

    HEADWATER_SHADOW_LOG_DIR="$log" sh "$mine" --until "$until" --rows "$work/rows.tsv" > "$work/mine.out" 2> "$work/mine.err" \
        || die "shadow-mine.sh failed: $(tail -n 1 "$work/mine.err")"
    joined=$(awk 'NR > 1' "$work/rows.tsv" | wc -l | tr -d ' ')

    : > "$work/cand.tsv"
    : > "$work/excl.tsv"
    awk -F '\t' -v d="$digest" 'NR > 1 && $5 == d' "$work/rows.tsv" > "$work/rows.d"
    while IFS="$tab" read -r id at file line dig det emb; do
        line_of "$file" "$line"
        jq -j '.task // "" | tostring' "$work/line.json" > "$work/task"
        tdig="sha256:$(sha256sum < "$work/task" | cut -d ' ' -f 1)"
        offers
        rp=$(cut -f 1 "$work/route.tsv" | paste -sd '|' -)
        ep=$(cut -f 1 "$work/emb.tsv" | paste -sd '|' -)
        if [ "$det" = silent ]; then
            st=silent
        elif [ "$det" = offered ] && [ "$emb" = offered ]; then
            cut -f 1 "$work/emb.tsv" > "$work/emb.paths"
            if cut -f 1 "$work/route.tsv" | grep -Fxq -f "$work/emb.paths"; then st=overlap; else st=disjoint; fi
        else
            st=none
        fi
        reason=
        if [ "$st" = none ]; then
            reason=outside-strata
        elif [ -z "$rp" ] && [ -z "$ep" ]; then
            reason=empty-list
        else
            reason=$(exclusion "$work/task")
        fi
        if [ -n "$reason" ]; then
            printf '%s\t%s\t%s\n' "$id" "$st" "$reason" >> "$work/excl.tsv"
        else
            printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$(sha "$seed$tab$id")" "$id" "$st" "$at" "$file" "$line" "$tdig" "$rp" "$ep" >> "$work/cand.tsv"
        fi
    done < "$work/rows.d"

    mkdir -p "$record" || die "cannot write $record"
    printf 'prompt_id\tstratum\tat\tfile\tline\ttask_digest\tdeterministic\tembedding\n' > "$record/sample.tsv"
    printf 'prompt_id\tstratum\n' > "$record/owner-sample.tsv"
    for st in silent disjoint overlap; do
        case $st in
            silent) take=all ;;
            disjoint) take=$disjoint ;;
            overlap) take=$control ;;
        esac
        awk -F '\t' -v s="$st" '$3 == s' "$work/cand.tsv" | LC_ALL=C sort -t "$tab" -k1,1 > "$work/st.$st"
        if [ "$take" = all ]; then cp "$work/st.$st" "$work/drawn.$st"; else head -n "$take" "$work/st.$st" > "$work/drawn.$st"; fi
        cut -f 2- "$work/drawn.$st" >> "$record/sample.tsv"
        head -n "$owner" "$work/drawn.$st" | cut -f 2,3 >> "$record/owner-sample.tsv"
    done
    printf 'prompt_id\tstratum\treason\n' > "$record/excluded.tsv"
    LC_ALL=C sort -t "$tab" -k2,2 -k3,3 -k1,1 "$work/excl.tsv" >> "$record/excluded.tsv"

    {
        printf 'seed: %s\n' "$seed"
        printf 'until: %s\n' "$until"
        printf 'model digest: %s\n' "$digest"
        printf 'joined: %s\n' "$joined"
        printf 'under the digest: %s\n' "$(wc -l < "$work/rows.d" | tr -d ' ')"
        for st in silent disjoint overlap none; do
            _all=$( { awk -F '\t' -v s="$st" '$3 == s' "$work/cand.tsv"; awk -F '\t' -v s="$st" '$2 == s' "$work/excl.tsv"; } | wc -l | tr -d ' ')
            _ex=$(awk -F '\t' -v s="$st" '$2 == s' "$work/excl.tsv" | wc -l | tr -d ' ')
            _kept=$(awk -F '\t' -v s="$st" '$3 == s' "$work/cand.tsv" | wc -l | tr -d ' ')
            _drawn=0
            [ -f "$work/drawn.$st" ] && _drawn=$(wc -l < "$work/drawn.$st" | tr -d ' ')
            printf 'stratum %s: %s, excluded %s, kept %s, drawn %s\n' "$st" "$_all" "$_ex" "$_kept" "$_drawn"
        done
        for r in bare-command no-word empty-list outside-strata; do
            printf 'excluded %s: %s\n' "$r" "$(awk -F '\t' -v r="$r" '$3 == r' "$work/excl.tsv" | wc -l | tr -d ' ')"
        done
        printf 'disjoint draw: %s\n' "$disjoint"
        printf 'overlap draw: %s\n' "$control"
        printf 'sample: %s\n' "$(awk 'NR > 1' "$record/sample.tsv" | wc -l | tr -d ' ')"
        printf 'owner sample: %s\n' "$(awk 'NR > 1' "$record/owner-sample.tsv" | wc -l | tr -d ' ')"
    } > "$record/draw.txt"
    cat "$record/draw.txt"
}

# The versions of every path in $work/paths, as `path<TAB>digest<TAB>title
# <TAB>summary`, to $work/versions.tsv. A version whose frontmatter does not
# parse to a one-line title and summary is left out.
versions() {
    tr '\n' '\0' < "$work/paths" | xargs -0 git -C "$history" log --all -m --format= --raw --no-abbrev --no-renames -- > "$work/raw.log" 2>/dev/null
    awk -F '\t' 'NR == FNR { want[$0] = 1; next }
        ($2 in want) { split($1, a, " "); if (a[4] !~ /^0+$/) print $2 "\t" a[4] }' "$work/paths" "$work/raw.log" \
        | LC_ALL=C sort -u > "$work/blobs.u"
    : > "$work/versions.tsv"
    while IFS="$tab" read -r _p _b; do
        git -C "$history" cat-file -p "$_b" 2>/dev/null | LC_ALL=C awk '
            function val(s,   q, out, c, i) {
                sub(/^[^:]*:[ \t]*/, "", s); sub(/[ \t\r]+$/, "", s)
                q = substr(s, 1, 1)
                if (q == "\"") {
                    if (length(s) < 2 || substr(s, length(s), 1) != "\"") return "\001"
                    s = substr(s, 2, length(s) - 2); out = ""
                    for (i = 1; i <= length(s); i++) {
                        c = substr(s, i, 1)
                        if (c == "\\") {
                            i++; c = substr(s, i, 1)
                            if (c == "\"" || c == "\\" || c == "/") out = out c
                            else return "\001"
                        } else if (c == "\"") return "\001"
                        else out = out c
                    }
                    return out
                }
                if (q == "\047") {
                    if (length(s) < 2 || substr(s, length(s), 1) != "\047") return "\001"
                    s = substr(s, 2, length(s) - 2); gsub(/\047\047/, "\047", s); return s
                }
                if (q == ">" || q == "|" || s == "") return "\001"
                sub(/[ \t]+#.*$/, "", s)
                return s
            }
            NR == 1 { if ($0 != "---") exit; next }
            $0 == "---" { done = 1; exit }
            /^title:/ { title = val($0) }
            /^summary:/ { summary = val($0); has = 1 }
            END {
                if (!done || !has || summary == "\001" || title == "\001" || index(summary title, "\t")) exit
                printf "%s\t%s\n", title, summary
            }' > "$work/fm"
        [ -s "$work/fm" ] || continue
        IFS="$tab" read -r _t _s < "$work/fm"
        printf '%s\tsha256:%s\t%s\t%s\n' "$_p" "$(sha "$_s")" "$_t" "$_s" >> "$work/versions.tsv"
    done < "$work/blobs.u"
    LC_ALL=C sort -u -t "$tab" -k1,2 "$work/versions.tsv" > "$work/versions.u"
}

# render <label> <prompt id> <seed>: one prompt of a packet, from
# $work/line.json, to standard output. It reads the documents through
# $work/resolved.tsv and never through the path that offered them.
render() {
    _id=$2
    { cat "$work/route.tsv"; cat "$work/emb.tsv"; } | LC_ALL=C sort -t "$tab" -k1,1 -k2,2 | awk -F '\t' '!($1 in s) { s[$1] = 1; print }' > "$work/merged.tsv"
    : > "$work/labelled.tsv"
    while IFS="$tab" read -r _p _d; do
        _r=$(awk -F '\t' -v p="$_p" -v d="$_d" '$1 == p && $2 == d { print $3 "\t" $4; exit }' "$work/resolved.tsv")
        if [ -z "$_r" ]; then
            printf '%s\t%s\t%s\n' "$_id" "$_p" "$_d" >> "$work/unresolved.tsv"
            continue
        fi
        printf '%s\t%s\t%s\n' "$(sha "$3$tab$_id$tab$_p")" "$_p" "$_r" >> "$work/labelled.tsv"
    done < "$work/merged.tsv"
    printf '=== %s task ===\n' "$1"
    jq -j '.task // "" | tostring' "$work/line.json"
    printf '\n=== %s documents ===\n' "$1"
    LC_ALL=C sort -t "$tab" -k1,1 "$work/labelled.tsv" | awk -F '\t' -v id="$_id" -v keyf="$work/docs.key" '
        { n++; printf "D%d. %s\n    %s\n", n, $3, $4; printf "%s\tD%d\t%s\n", id, n, $2 >> keyf }'
    printf '\n'
}

cmd_packets() {
    seed=; batch=12
    while [ $# -gt 0 ]; do
        [ $# -ge 2 ] || usage
        case $1 in
            --seed) seed=$2 ;;
            --batch) batch=$2 ;;
            *) usage ;;
        esac
        shift 2
    done
    [ -n "$seed" ] || usage
    case $batch in '' | *[!0-9]* | 0) usage ;; esac
    [ -f "$record/sample.tsv" ] || die "no sample at $record/sample.tsv: run draw first"
    need_log
    need_packets

    # Every (path, digest) that any sampled prompt offers, to resolve once.
    : > "$work/pairs.tsv"
    awk -F '\t' 'NR > 1' "$record/sample.tsv" > "$work/sample"
    while IFS="$tab" read -r id st at file line tdig rp ep; do
        line_of "$file" "$line"
        offers
        cat "$work/route.tsv" "$work/emb.tsv" >> "$work/pairs.tsv"
    done < "$work/sample"
    cut -f 1 "$work/pairs.tsv" | LC_ALL=C sort -u > "$work/paths"
    versions
    LC_ALL=C sort -u "$work/pairs.tsv" > "$work/pairs.u"
    # The smaller digest wins where one path holds two, so that the choice
    # reads no path.
    LC_ALL=C sort -t "$tab" -k1,1 -k2,2 "$work/pairs.u" | awk -F '\t' 'NR == FNR { v[$1 "\t" $2] = $3 "\t" $4; next } ($1 "\t" $2) in v { print $1 "\t" $2 "\t" v[$1 "\t" $2] }' "$work/versions.u" - > "$work/resolved.tsv"

    rm -rf "$packets/rater-a" "$packets/rater-b" "$packets/owner" "$packets/key"
    mkdir -p "$packets/rater-a" "$packets/rater-b" "$packets/owner" "$packets/key" || die "cannot write $packets"
    : > "$work/docs.key"
    : > "$work/unresolved.tsv"
    printf 'packet\tlabel\tprompt_id\tbatch\n' > "$packets/key/prompts.tsv"

    # The raters' packet: the whole sample, in packet order.
    while IFS="$tab" read -r id rest; do
        printf '%s\t%s\n' "$(sha "$seed${tab}packet$tab$id")" "$id"
    done < "$work/sample" | LC_ALL=C sort -t "$tab" -k1,1 | cut -f 2 > "$work/order"
    k=0
    while IFS= read -r id; do
        k=$((k + 1))
        b=$(( (k - 1) / batch + 1 ))
        bn=$(printf '%02d' "$b")
        row=$(awk -F '\t' -v i="$id" '$1 == i' "$work/sample")
        file=$(printf '%s\n' "$row" | cut -f 4)
        line=$(printf '%s\n' "$row" | cut -f 5)
        line_of "$file" "$line"
        offers
        render "P$k" "$id" "$seed" >> "$work/batch-$bn.md"
        printf 'rater\tP%s\t%s\t%s\n' "$k" "$id" "$bn" >> "$packets/key/prompts.tsv"
    done < "$work/order"
    for f in "$work"/batch-*.md; do
        cp "$f" "$packets/rater-a/"
        cp "$f" "$packets/rater-b/"
    done

    # The owner's packet: the owner's prompts, in packet order, as O1 and on.
    awk -F '\t' 'NR > 1 { print $1 }' "$record/owner-sample.tsv" > "$work/owner.ids"
    grep -Fxf "$work/owner.ids" "$work/order" > "$work/owner.order"
    # The owner's labels are the raters' labels, because a label reads only
    # the seed, the prompt and the path; this pass keeps no second key.
    cp "$work/docs.key" "$work/docs.key.rater"
    {
        printf 'Rate each prompt before you open tools/run/relevance-grade/ or any other\n'
        printf 'file of this grade, because that record names what this packet hides.\n\n'
    } > "$packets/owner/packet.md"
    k=0
    while IFS= read -r id; do
        k=$((k + 1))
        row=$(awk -F '\t' -v i="$id" '$1 == i' "$work/sample")
        line_of "$(printf '%s\n' "$row" | cut -f 4)" "$(printf '%s\n' "$row" | cut -f 5)"
        offers
        render "O$k" "$id" "$seed" >> "$packets/owner/packet.md"
        printf 'owner\tO%s\t%s\t-\n' "$k" "$id" >> "$packets/key/prompts.tsv"
    done < "$work/owner.order"
    printf 'prompt_id\tlabel\tpath\n' > "$packets/key/documents.tsv"
    LC_ALL=C sort -u "$work/docs.key.rater" >> "$packets/key/documents.tsv"
    printf 'prompt_id\tpath\tdigest\n' > "$packets/key/unresolved.tsv"
    LC_ALL=C sort -u "$work/unresolved.tsv" >> "$packets/key/unresolved.tsv"

    nprompts=$(wc -l < "$work/order" | tr -d ' ')
    ndocs=$(wc -l < "$work/docs.key.rater" | tr -d ' ')
    npairs=$(wc -l < "$work/pairs.u" | tr -d ' ')
    nres=$(wc -l < "$work/resolved.tsv" | tr -d ' ')
    nun=$(LC_ALL=C sort -u "$work/unresolved.tsv" | wc -l | tr -d ' ')
    nunpairs=$(cut -f 2,3 "$work/unresolved.tsv" | LC_ALL=C sort -u | wc -l | tr -d ' ')
    {
        printf 'seed: %s\n' "$seed"
        printf 'prompts: %s\n' "$nprompts"
        printf 'batches: %s of up to %s prompts\n' "$(ls "$packets/rater-a" | wc -l | tr -d ' ')" "$batch"
        printf 'documents listed: %s\n' "$ndocs"
        printf 'distinct (path, digest) offered: %s\n' "$npairs"
        printf 'resolved from history: %s\n' "$nres"
        printf 'unresolved (path, digest): %s\n' "$nunpairs"
        printf 'unresolved documents left out of a list: %s\n' "$nun"
        printf 'owner prompts: %s\n' "$(wc -l < "$work/owner.order" | tr -d ' ')"
    } > "$packets/key/packets.txt"
    cat "$packets/key/packets.txt"
}

# ingest <rater>: the rater's score files, mapped back through the key.
cmd_ingest() {
    [ $# -eq 1 ] || usage
    r=$1
    case $r in a | b | third | owner) ;; *) usage ;; esac
    need_packets
    case $r in
        owner) dir="$packets/owner"; pk=owner ;;
        *) dir="$packets/rater-$r"; pk=rater ;;
    esac
    [ -f "$packets/key/prompts.tsv" ] || die "no key under $packets/key"
    ls "$dir"/scores-*.tsv > /dev/null 2>&1 || die "no score files in $dir"
    cat "$dir"/scores-*.tsv | tr -d '\r' | awk -F '\t' 'NF >= 3 && $1 !~ /^#/' > "$work/raw"
    # Which items a file must cover: every document and the flag of each
    # prompt it was given, or for the third pass the items in dispute.
    if [ "$r" = third ]; then
        [ -f "$packets/rater-third/items.tsv" ] || die "no third-pass items: run third first"
        cp "$packets/rater-third/items.tsv" "$work/want"
    else
        awk -F '\t' -v pk="$pk" 'NR == FNR { if ($1 == pk) { lab[$3] = $2 }; next }
            FNR > 1 && ($1 in lab) { print lab[$1] "\t" $2 "\t" $1 "\t" $3 }' "$packets/key/prompts.tsv" "$packets/key/documents.tsv" > "$work/want.docs"
        awk -F '\t' -v pk="$pk" '$1 == pk { print $2 "\tmissing\t" $3 "\tmissing" }' "$packets/key/prompts.tsv" > "$work/want.flags"
        # A rater packet covers only the batches that have a score file.
        if [ "$pk" = rater ]; then
            for f in "$dir"/scores-*.tsv; do
                bn=$(basename "$f" .tsv | sed 's/^scores-//')
                awk -F '\t' -v b="$bn" '$1 == "rater" && $4 == b { print $2 }' "$packets/key/prompts.tsv"
            done > "$work/labels"
            cat "$work/want.docs" "$work/want.flags" | awk -F '\t' 'NR == FNR { l[$1] = 1; next } $1 in l' "$work/labels" - > "$work/want"
        else
            cat "$work/want.docs" "$work/want.flags" > "$work/want"
        fi
    fi
    mkdir -p "$record"
    awk -F '\t' -v out="$work/scores" '
        NR == FNR { w[$1 "\t" $2] = $3 "\t" $4; next }
        {
            k = $1 "\t" $2
            if (!(k in w)) { print "an item not in the packet: " $1 " " $2 > "/dev/stderr"; bad = 1; next }
            if (k in got) { print "scored twice: " $1 " " $2 > "/dev/stderr"; bad = 1; next }
            v = $3
            if ($2 == "missing" ? v !~ /^[01]$/ : v !~ /^[012]$/) { print "not a score: " $1 " " $2 " " v > "/dev/stderr"; bad = 1; next }
            got[k] = 1; print w[k] "\t" v > out
        }
        END { for (k in w) if (!(k in got)) { print "not scored: " k > "/dev/stderr"; bad = 1 } ; exit bad ? 4 : 0 }' "$work/want" "$work/raw" || exit 4
    printf 'prompt_id\titem\tvalue\n' > "$record/scores-$r.tsv"
    LC_ALL=C sort -t "$tab" -k1,1 -k2,2 "$work/scores" >> "$record/scores-$r.tsv"
    printf 'scores-%s: %s items\n' "$r" "$(wc -l < "$work/scores" | tr -d ' ')"
}

# The stratum of each sampled prompt, for the joins below.
strata() { awk -F '\t' 'NR > 1 { print $1 "\t" $2 }' "$record/sample.tsv" > "$work/strata"; }

cmd_agreement() {
    [ -f "$record/scores-a.tsv" ] && [ -f "$record/scores-b.tsv" ] || die "need scores-a.tsv and scores-b.tsv"
    strata
    LC_ALL=C awk -F '\t' -v dis="$record/disagreements.tsv" '
        function kappa(pre, cats,   i, j, n, po, pe, num, den, w, k, ci, cj) {
            n = cnt[pre]; if (n == 0) return "n/a"
            k = split(cats, c, " ")
            num = 0; den = 0
            for (i = 1; i <= k; i++) for (j = 1; j <= k; j++) {
                ci = c[i]; cj = c[j]
                w = (k == 2) ? (ci == cj ? 0 : 1) : ((ci - cj) ^ 2) / ((k - 1) ^ 2)
                num += w * (obs[pre, ci, cj] + 0) / n
                den += w * (ma[pre, ci] + 0) * (mb[pre, cj] + 0) / (n * n)
            }
            return den == 0 ? "n/a" : sprintf("%.4f", 1 - num / den)
        }
        function add(pre, x, y) { cnt[pre]++; obs[pre, x, y]++; ma[pre, x]++; mb[pre, y]++; if (x == y) eq[pre]++ }
        function share(pre) { return cnt[pre] ? sprintf("%d of %d (%.4f)", eq[pre], cnt[pre], eq[pre] / cnt[pre]) : "0 of 0 (n/a)" }
        FILENAME ~ /\/strata$/ { st[$1] = $2; next }
        FNR == 1 { next }
        FILENAME ~ /scores-a\.tsv$/ { a[$1 "\t" $2] = $3; next }
        { b[$1 "\t" $2] = $3 }
        END {
            print "prompt_id\titem\tstratum" > dis
            for (k in a) {
                if (!(k in b)) { print "relevance-grade: an item scored by a and not by b: " k > "/dev/stderr"; bad = 1; continue }
                split(k, p, "\t"); s = st[p[1]]; kind = (p[2] == "missing") ? "flag" : "score"
                add(kind " " s, a[k], b[k]); add(kind " all", a[k], b[k])
                if (a[k] != b[k]) print k "\t" s > dis
            }
            for (k in b) if (!(k in a)) { print "relevance-grade: an item scored by b and not by a: " k > "/dev/stderr"; bad = 1 }
            if (bad) exit 4
            split("silent disjoint overlap all", ss, " ")
            for (i = 1; i <= 4; i++) {
                s = ss[i]
                printf "stratum %s documents: %d\n", s, cnt["score " s]
                printf "stratum %s exact agreement on scores: %s\n", s, share("score " s)
                printf "stratum %s quadratic-weighted kappa on scores: %s\n", s, kappa("score " s, "0 1 2")
                printf "stratum %s prompts: %d\n", s, cnt["flag " s]
                printf "stratum %s exact agreement on missing: %s\n", s, share("flag " s)
                printf "stratum %s kappa on missing: %s\n", s, kappa("flag " s, "0 1")
            }
        }' "$work/strata" "$record/scores-a.tsv" "$record/scores-b.tsv" > "$work/agreement" || exit 4
    {
        printf 'statistic: share of exact agreement, and Cohen kappa (quadratic weights on the 0/1/2 scores, unweighted on the missing flag)\n'
        cat "$work/agreement"
        printf 'disagreements: %s\n' "$(awk 'NR > 1' "$record/disagreements.tsv" | wc -l | tr -d ' ')"
    } > "$record/agreement.txt"
    { head -n 1 "$record/disagreements.tsv"; awk 'NR > 1' "$record/disagreements.tsv" | LC_ALL=C sort; } > "$work/dis" && cp "$work/dis" "$record/disagreements.tsv"
    cat "$record/agreement.txt"
}

cmd_third() {
    seed=
    while [ $# -gt 0 ]; do
        [ $# -ge 2 ] || usage
        case $1 in --seed) seed=$2 ;; *) usage ;; esac
        shift 2
    done
    [ -n "$seed" ] || usage
    [ -f "$record/disagreements.tsv" ] || die "no disagreements.tsv: run agreement first"
    need_packets
    rm -rf "$packets/rater-third"
    mkdir -p "$packets/rater-third"
    : > "$packets/rater-third/items.tsv"
    awk -F '\t' 'NR > 1 { print $1 }' "$record/disagreements.tsv" | LC_ALL=C sort -u > "$work/dprompts"
    awk -F '\t' 'NR == FNR { d[$1] = 1; next } $1 == "rater" && ($3 in d) { print $2 "\t" $3 "\t" $4 }' "$work/dprompts" "$packets/key/prompts.tsv" > "$work/dlab"
    n=0
    while IFS="$tab" read -r lab id bn; do
        n=$((n + 1))
        tb=$(printf '%02d' $(( (n - 1) / 12 + 1 )))
        # The prompt's block from the raters' batch, verbatim.
        awk -v l="$lab" '
            $0 == "=== " l " task ===" { on = 1 }
            on && /^=== P[0-9]+ task ===$/ && $0 != "=== " l " task ===" { exit }
            on { print }' "$packets/rater-a/batch-$bn.md" > "$work/block"
        items=$(awk -F '\t' -v i="$id" 'NR == FNR { if ($1 == i) want[$2] = 1; next }
            FNR > 1 && $1 == i && ($3 in want) { printf "%s ", $2 }' "$record/disagreements.tsv" "$packets/key/documents.tsv")
        flag=$(awk -F '\t' -v i="$id" 'NR > 1 && $1 == i && $2 == "missing" { print "missing"; exit }' "$record/disagreements.tsv")
        {
            cat "$work/block"
            printf 'rate only: %s%s\n\n' "$items" "$flag"
        } >> "$packets/rater-third/batch-$tb.md"
        for it in $items $flag; do
            if [ "$it" = missing ]; then
                printf '%s\tmissing\t%s\tmissing\n' "$lab" "$id" >> "$packets/rater-third/items.tsv"
            else
                p=$(awk -F '\t' -v i="$id" -v l="$it" '$1 == i && $2 == l { print $3; exit }' "$packets/key/documents.tsv")
                printf '%s\t%s\t%s\t%s\n' "$lab" "$it" "$id" "$p" >> "$packets/rater-third/items.tsv"
            fi
        done
    done < "$work/dlab"
    printf 'third pass: %s prompts, %s items\n' "$n" "$(wc -l < "$packets/rater-third/items.tsv" | tr -d ' ')"
}

cmd_consensus() {
    for r in a b; do [ -f "$record/scores-$r.tsv" ] || die "need scores-$r.tsv"; done
    third="$record/scores-third.tsv"
    [ -f "$third" ] || { printf 'prompt_id\titem\tvalue\n' > "$work/third"; third="$work/third"; }
    LC_ALL=C awk -F '\t' '
        FNR == 1 { f++; next }
        f == 1 { a[$1 "\t" $2] = $3; next }
        f == 2 { b[$1 "\t" $2] = $3; next }
        { c[$1 "\t" $2] = $3 }
        END {
            for (k in a) {
                if (a[k] == b[k]) { print k "\t" a[k] "\tagreed"; continue }
                if (!(k in c)) { print "relevance-grade: a disagreement with no third score: " k > "/dev/stderr"; bad = 1; continue }
                x = a[k]; y = b[k]; z = c[k]
                m = (x <= y) ? ((y <= z) ? y : ((x <= z) ? z : x)) : ((x <= z) ? x : ((y <= z) ? z : y))
                print k "\t" m "\tthird"
            }
            exit bad ? 4 : 0
        }' "$record/scores-a.tsv" "$record/scores-b.tsv" "$third" > "$work/cons" || exit 4
    printf 'prompt_id\titem\tvalue\tsettled_by\n' > "$record/consensus.tsv"
    LC_ALL=C sort -t "$tab" -k1,1 -k2,2 "$work/cons" >> "$record/consensus.tsv"
    printf 'consensus: %s items, %s agreed, %s settled by the third pass\n' \
        "$(wc -l < "$work/cons" | tr -d ' ')" "$(awk -F '\t' '$4 == "agreed"' "$work/cons" | wc -l | tr -d ' ')" \
        "$(awk -F '\t' '$4 == "third"' "$work/cons" | wc -l | tr -d ' ')"
}

[ $# -ge 1 ] || usage
sub=$1
shift
case $sub in
    draw) cmd_draw "$@" ;;
    packets) cmd_packets "$@" ;;
    ingest) cmd_ingest "$@" ;;
    agreement) cmd_agreement "$@" ;;
    third) cmd_third "$@" ;;
    consensus) cmd_consensus "$@" ;;
    *) usage ;;
esac
