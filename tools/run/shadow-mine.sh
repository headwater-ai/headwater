#!/bin/sh
# Mine the shadow-mode routing log: steps 1 to 7 of
# `docs/how-to/mine-the-shadow-mode-routing-log.md` as one command.
#
# HW-DR-0064 collects one shadow-log line per prompt, written by
# `.claude/hooks/intent.sh`, and asks whether the offline embedding path would
# have offered what a session went on to read where the deterministic route
# offered nothing. The how-to states the procedure as hand commands, and its
# last paragraph names the failure it exists to prevent: one silence rate over
# a period with unreported gaps, or over two models. This is that procedure as
# a tool, in the way `tools/run/shadow-capture.sh` is #917's join.
#
#     sh tools/run/shadow-mine.sh [--bound <n>] [--until <ISO time>] [--rows <file>]
#
# `--until` cuts the log at a time, so that a reading made later can give the
# population of a reading made earlier. Every log line whose `at` is later
# than the time, compared to the second, is dropped before anything is
# counted, and a typed prompt later than the time is not typed. A line with no
# time that parses is still read, and so are torn and blank lines. The output
# then names the cut and the count of lines it dropped.
#
# `--rows` writes the join itself, one TSV row per joined id in time order of
# its earliest line, under a header: `prompt_id`, the `at`, the log file and
# the line number of that earliest line, `model_digest`, and the state of each
# path (`silent`, `offered`, `absent`, or `none` for an embedding path that did
# not run). `tools/run/relevance-grade.sh` reads it, so that the person-prompt
# rule has one copy. With neither flag, the output is the same bytes as before
# the two flags existed.
#
# The population is person prompts, by the rule of `shadow-capture.sh`: a
# harness transcript record with `type` "user", `isSidechain` false and
# `origin.kind` "human", counted once per `promptId`, at the timestamp of its
# first record. A log line is read when it parses to an object, its
# `probe_session` is empty, and its `prompt_id` is not empty. A line that does
# not parse is torn, an empty line is blank, and each is counted and the read
# goes on. A torn line and a line with an empty `prompt_id` are listed under
# their file and line number.
#
# The period opens at the first log line read, and the typed prompts of the
# period are the person prompts at or after it, and every person prompt the
# log joined. The join is on distinct ids and never on lines, because the hook
# has written one prompt twice. Its three buckets are the joined ids, the
# typed prompts with no line (gaps) and the logged ids that no person prompt
# claims (unclaimed), and both sums are printed: typed = joined + gaps, and
# logged = joined + unclaimed. A prompt with more than one line is read from
# its earliest line.
#
# The bound (default 500 matched person prompts, HW-DR-0064 as lowered on
# 2026-10-03) closes on the `at` of the earliest line of the bound-th joined
# id in time order. The silent person prompts are the joined ids whose route
# has a non-null `silence`, against the floor of 58.
#
# Every rate is per `model_digest` and never pooled. `none` is the row of lines
# that carry no digest, where only the deterministic path ran. Each row is
# printed for all its prompts, then for `injected` true and false apart, and
# the whole table twice: unrestricted, and restricted to sessions that started
# in a checkout that holds an engine now (the rule of `shadow-capture.sh`).
#
#   deterministic silent  `route | fromjson | .silence` is not null, of the
#                         prompts whose route is present
#   embedding silent      `neighbors | fromjson | .neighbors` is empty
#   both offered          both paths offered a document; share: at least one
#                         document is in both offers
#   recall                the documents the session read: `Read` calls of the
#                         session's main thread after the prompt and before its
#                         next person prompt, whose path is under the line's
#                         `corpus_root` and a slash, made relative to it, and
#                         that name a Markdown document, the only file a
#                         pointer names, outside a directory whose name starts
#                         with a dot: `.git` holds a run's ledger, `.claude`
#                         the agent definitions and other worktrees, and no
#                         pointer names either. Each document counts once a
#                         prompt.
#                         Recall is the hits of a path over those documents,
#                         summed over the prompts with a read.
#   rank                  the position at which a path first offers a document
#                         that was read, over the prompts with a read.
#
# Every figure goes to standard output, one `name: value` per line. The output
# is the same bytes for the same inputs, but for the `commit:` and `read at:`
# lines that name what was read and when. A non-zero exit is an argument (2)
# or no `jq` (3).
#
# Two variables point it elsewhere, which the fixtures use:
#     HEADWATER_TRANSCRIPT_DIRS   space-separated transcript directories
#                                 (default: ~/.claude/projects/<prefix>*)
#     HEADWATER_SHADOW_LOG_DIR    the log directory, as the hook reads it
#                                 (default: <git common dir>/headwater-shadow-log)
#
# `sh tools/run/shadow-mine-fixtures.sh` holds it.

set -u

usage() {
    echo "usage: sh tools/run/shadow-mine.sh [--bound <n>] [--until <ISO time>] [--rows <file>]" >&2
    exit 2
}

bound=500
until=
rows=
while [ $# -gt 0 ]; do
    case $1 in
        --bound)
            [ $# -ge 2 ] || usage
            case $2 in
                '' | *[!0-9]*) usage ;;
            esac
            bound=$2
            shift 2
            ;;
        --until)
            [ $# -ge 2 ] && [ -n "$2" ] || usage
            until=$2
            shift 2
            ;;
        --rows)
            [ $# -ge 2 ] && [ -n "$2" ] || usage
            rows=$2
            shift 2
            ;;
        *) usage ;;
    esac
done
[ "$bound" -gt 0 ] || usage

if ! command -v jq >/dev/null 2>&1; then
    echo "shadow-mine: no \`jq\` on the path" >&2
    exit 3
fi

# The cut in epoch milliseconds, or empty for none.
untilms=
if [ -n "$until" ]; then
    untilms=$(jq -rn --arg u "$until" '$u | sub("\\.[0-9]+"; "") | try (fromdateiso8601 * 1000 | tostring) catch empty') || untilms=
    [ -n "$untilms" ] || usage
fi

here=$(cd "$(dirname "$0")/../.." && pwd)

log=${HEADWATER_SHADOW_LOG_DIR:-}
if [ -z "$log" ]; then
    common=$(git -C "$here" rev-parse --git-common-dir 2>/dev/null) || {
        echo "shadow-mine: not inside a git checkout, and no HEADWATER_SHADOW_LOG_DIR" >&2
        exit 2
    }
    case $common in
        /*) ;;
        *) common="$here/$common" ;;
    esac
    log="$common/headwater-shadow-log"
fi
if [ ! -d "$log" ]; then
    echo "shadow-mine: no log directory at $log" >&2
    exit 2
fi

dirs=${HEADWATER_TRANSCRIPT_DIRS:-}
if [ -z "$dirs" ]; then
    main=$(cd "$(dirname "$log")/.." && pwd)
    prefix=$(printf '%s' "$main" | tr '/.' '--')
    for d in "$HOME/.claude/projects/$prefix"*; do
        [ -d "$d" ] && dirs="$dirs $d"
    done
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

commit=$(git -C "$here" rev-parse HEAD 2>/dev/null) || commit=unknown
now=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# One row per log line, in file and line order. Each line is read as a string
# and parsed on its own, because a stream read stops at the first torn line.
# Times are epoch milliseconds. Path lists are joined with `|`.
: > "$scratch/log.tsv"
nfiles=0
for f in "$log"/*.jsonl; do
    [ -f "$f" ] || continue
    nfiles=$((nfiles + 1))
    jq -R -r --arg file "$(basename "$f")" --arg until "$untilms" '
        def ms: tostring as $s
            | ($s | sub("\\.[0-9]+"; "") | try fromdateiso8601 catch null) as $e
            | if $e == null then "" else ($e * 1000 | tostring) end;
        def doc: if type == "string" then (try fromjson catch null) else . end;
        input_line_number as $n
        | if . == "" then ["blank", $file, $n]
          else (fromjson? // null) as $o
          | if ($o | type) != "object" then ["torn", $file, $n]
            elif $until != "" and (($o.at // "" | ms) as $t | $t != "" and ($t | tonumber) > ($until | tonumber)) then ["after", $file, $n]
            elif ($o.probe_session // "" | tostring) != "" then ["probe", $file, $n]
            elif ($o.prompt_id // "" | tostring) == "" then ["emptyid", $file, $n, "", ($o.at // "" | tostring), ($o.at // "" | ms)]
            else
              ($o.model_digest // "none" | tostring) as $d
              | ($o.route | doc) as $r
              | (if ($r | type) != "object" then ["absent", ""]
                 elif ($r.silence // null) != null then ["silent", ""]
                 else ["offered", ([$r.pointers[]?.path // empty] | join("|"))] end) as $det
              | ($o.neighbors | doc) as $nb
              | (if $d == "none" then ["none", ""]
                 elif ($nb | type) != "object" or ($nb.neighbors | type) != "array" then ["absent", ""]
                 elif ($nb.neighbors | length) == 0 then ["silent", ""]
                 else ["offered", ([$nb.neighbors[].path // empty] | join("|"))] end) as $emb
              | ["ok", $file, $n, ($o.prompt_id | tostring), ($o.at // "" | tostring), ($o.at // "" | ms),
                 ($o.corpus_root // "" | tostring), ($o.injected | tostring), $d, $det[0], $det[1], $emb[0], $emb[1]]
            end
          end
        | map(tostring) | @tsv' "$f" >> "$scratch/log.tsv"
done

# Person prompts (P: time, id, session, start directory) and main-thread
# `Read` calls (R: time, session, path) from every transcript. The start
# directory is the `cwd` of the transcript's first record that has one.
: > "$scratch/tr.tsv"
ndirs=0
for d in $dirs; do
    ndirs=$((ndirs + 1))
    for f in "$d"/*.jsonl; do
        [ -f "$f" ] || continue
        startdir=$(jq -R -r 'fromjson? | select(type == "object") | .cwd // empty' "$f" 2>/dev/null | head -n 1)
        jq -R -r --arg start "$startdir" '
            def ms: tostring as $s
                | ($s | sub("\\.[0-9]+"; "") | try fromdateiso8601 catch null) as $e
                | ($s | try (capture("\\.(?<f>[0-9]+)").f | (. + "00")[0:3] | tonumber) catch 0) as $f
                | if $e == null then null else $e * 1000 + $f end;
            fromjson? | select(type == "object") | select(.isSidechain | not)
            | (.timestamp // "" | ms) as $t | select($t != null)
            | if .type == "user" and (.origin.kind? // null) == "human" and (.promptId // "") != "" then
                ["P", $t, .promptId, (.sessionId // ""), $start]
              elif .type == "assistant" then
                (.message.content? // [] | if type == "array" then .[] else empty end
                 | select(type == "object" and .type == "tool_use" and .name == "Read")
                 | .input.file_path? // empty | select(type == "string")) as $p
                | ["R", $t, (.sessionId // ""), $p]
              else empty end
            | map(tostring) | @tsv' "$f" >> "$scratch/tr.tsv" 2>/dev/null
    done
done

tab=$(printf '\t')
awk -F '\t' '$1 == "P"' "$scratch/tr.tsv" | LC_ALL=C sort -t "$tab" -k2,2n -k3,3 > "$scratch/persons.tsv"
awk -F '\t' '$1 == "R"' "$scratch/tr.tsv" | LC_ALL=C sort -t "$tab" -k3,3 -k2,2n -k4,4 > "$scratch/reads.tsv"

# Whether each start directory holds an engine now: engine, none, or gone.
cut -f5 "$scratch/persons.tsv" | LC_ALL=C sort -u > "$scratch/starts"
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

printf 'commit: %s\n' "$commit"
printf 'read at: %s\n' "$now"
printf 'log files: %s\n' "$nfiles"
printf 'transcript directories: %s\n' "$ndirs"

if [ -n "$until" ]; then
    printf 'until: %s\n' "$until"
    printf 'lines after until: %s\n' "$(awk -F '\t' '$1 == "after"' "$scratch/log.tsv" | wc -l | tr -d ' ')"
fi

LC_ALL=C awk -F '\t' -v bound="$bound" -v untilms="$untilms" -v rows="$rows" '
function rate(a, b) { return b == 0 ? "n/a" : sprintf("%.4f", a / b) }
function frac(a, b) { return a " of " b " (" rate(a, b) ")" }
# The 1-based position of the first path of list `l` that is in the set
# `rset` (a string "|p1|p2|"), or 0.
function firstrank(l, rset,   n, a, i) {
    if (l == "") return 0
    n = split(l, a, "|")
    for (i = 1; i <= n; i++) if (index(rset, "|" a[i] "|")) return i
    return 0
}
function hits(l, rset,   n, a, i, h) {
    if (l == "") return 0
    n = split(l, a, "|"); h = 0
    for (i = 1; i <= n; i++) if (index(rset, "|" a[i] "|")) h++
    return h
}
function bucket(r) { return r == 0 ? "none" : (r >= 4 ? "4+" : r) }
# By name and not by `FNR == 1`, because an empty file has no first record.
FNR == 1 { part = FILENAME ~ /\/held\.tsv$/ ? 1 : FILENAME ~ /\/log\.tsv$/ ? 2 : FILENAME ~ /\/persons\.tsv$/ ? 3 : 4 }
part == 1 { held[$1] = $2; next }
part == 2 {
    k = $1
    if (k == "after") next
    loglines++
    if (k == "blank") { blank++; next }
    if (k == "torn") { torn++; excl[++nexcl] = "torn: " $2 ":" $3; next }
    if (k == "probe") { probe++; next }
    if ($6 != "") {
        if (first == "" || $6 + 0 < first + 0) { first = $6; firstat = $5 }
        if (last == "" || $6 + 0 > last + 0) { last = $6; lastat = $5 }
    }
    if (k == "emptyid") { emptyid++; excl[++nexcl] = "empty-id: " $2 ":" $3; next }
    id = $4
    nlines[id]++
    if (!(id in lep) || ($6 != "" && $6 + 0 < lep[id] + 0)) {
        lep[id] = $6; lat[id] = $5; lroot[id] = $7; linj[id] = $8; ldig[id] = $9
        ldet[id] = $10; ldp[id] = $11; lemb[id] = $12; lep2[id] = $13
        lfile[id] = $2; lline[id] = $3
    }
    next
}
part == 3 {
    if ($3 in pts) next
    pts[$3] = $2; psess[$3] = $4; pstart[$3] = $5
    sp[$4, ++spn[$4]] = $2
    next
}
part == 4 {
    i = ++rn[$3]; rts[$3, i] = $2; rpath[$3, i] = $4
    next
}
END {
    for (id in lep) { L++; if (id in pts) J++; else U++ }
    for (id in pts) {
        if ((id in lep) || (first != "" && pts[id] + 0 >= first + 0 && (untilms == "" || pts[id] + 0 <= untilms + 0))) {
            T++; typed[id] = 1
            st = (pstart[id] in held) ? held[pstart[id]] : "gone"
            starts[st]++
            if (st == "engine") { HT++; if (id in lep) HJ++ }
        }
    }
    G = T - J
    printf "log lines: %d\n", loglines
    printf "torn lines: %d\n", torn
    printf "blank lines: %d\n", blank
    printf "probe lines: %d\n", probe
    printf "empty-id lines: %d\n", emptyid
    for (i = 1; i <= nexcl; i++) print excl[i]
    printf "first line: %s\n", (firstat == "" ? "none" : firstat)
    printf "last line: %s\n", (lastat == "" ? "none" : lastat)
    printf "typed: %d\n", T
    printf "logged: %d\n", L
    printf "joined: %d\n", J
    printf "gaps: %d\n", G
    printf "unclaimed: %d\n", U
    printf "typed = joined + gaps: %d = %d + %d\n", T, J, G
    printf "logged = joined + unclaimed: %d = %d + %d\n", L, J, U

    # The joined ids in time order of their earliest line, for the bound.
    n = 0
    for (id in lep) if (id in pts) {
        n++; jid[n] = id; jl += nlines[id]; if (nlines[id] > 1) dup++
        if (ldet[id] == "silent") silentp++
    }
    for (i = 2; i <= n; i++) {
        x = jid[i]; j = i - 1
        while (j >= 1 && (lep[jid[j]] + 0 > lep[x] + 0 || (lep[jid[j]] + 0 == lep[x] + 0 && jid[j] > x))) { jid[j + 1] = jid[j]; j-- }
        jid[j + 1] = x
    }
    if (rows != "") {
        printf "prompt_id\tat\tfile\tline\tmodel_digest\tdeterministic\tembedding\n" > rows
        for (i = 1; i <= n; i++) {
            id = jid[i]
            printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\n", id, lat[id], lfile[id], lline[id], ldig[id], ldet[id], lemb[id] > rows
        }
        close(rows)
    }
    printf "joined lines: %d\n", jl
    printf "joined ids with more than one line: %d\n", dup
    printf "bound: %d\n", bound
    if (n >= bound) printf "bound reached: %s\n", lat[jid[bound]]
    else printf "bound reached: no, %d of %d\n", n, bound
    printf "silent person prompts: %d (floor 58: %s)\n", silentp, (silentp >= 58 ? "met" : "not met")
    printf "held typed: %d\n", HT
    printf "held joined: %d\n", HJ
    printf "held gaps: %d\n", HT - HJ
    printf "start directory engine: %d\n", starts["engine"]
    printf "start directory none: %d\n", starts["none"]
    printf "start directory gone: %d\n", starts["gone"]

    # The documents each joined prompt read, as "|p1|p2|", and their count.
    for (i = 1; i <= n; i++) {
        id = jid[i]; s = psess[id]; t = pts[id] + 0; next_t = -1
        for (q = 1; q <= spn[s]; q++) if (sp[s, q] + 0 > t) { next_t = sp[s, q] + 0; break }
        root = lroot[id]; rset = "|"; rc = 0
        if (root != "") for (q = 1; q <= rn[s]; q++) {
            r = rts[s, q] + 0
            if (r <= t || (next_t >= 0 && r >= next_t)) continue
            p = rpath[s, q]
            if (index(p, root "/") != 1) continue
            p = substr(p, length(root) + 2)
            if (p !~ /\.md$/ || p ~ /(^|\/)\./) continue
            if (index(rset, "|" p "|")) continue
            rset = rset p "|"; rc++
        }
        rs[id] = rset; rcount[id] = rc
    }

    nd = 0
    for (i = 1; i <= n; i++) if (!(ldig[jid[i]] in seen)) { seen[ldig[jid[i]]] = 1; dg[++nd] = ldig[jid[i]] }
    for (i = 2; i <= nd; i++) { x = dg[i]; j = i - 1; while (j >= 1 && dg[j] > x) { dg[j + 1] = dg[j]; j-- } dg[j + 1] = x }
    sc[1] = "unrestricted"; sc[2] = "restricted"
    gr[1] = "all"; gr[2] = "injected"; gr[3] = "not-injected"
    for (a = 1; a <= 2; a++) for (b = 1; b <= nd; b++) for (c = 1; c <= 3; c++) {
        d = dg[b]
        np = 0; ab = 0; ds = 0; dn = 0; es = 0; en = 0; both = 0; share = 0; dsoe = 0
        wr = 0; dh = 0; eh = 0; rd = 0
        split("", drk); split("", erk)
        for (i = 1; i <= n; i++) {
            id = jid[i]
            if (ldig[id] != d) continue
            if (a == 2 && held[pstart[id]] != "engine") continue
            if (c == 2 && linj[id] != "true") continue
            if (c == 3 && linj[id] == "true") continue
            np++
            if (ldet[id] == "absent") ab++
            else { dn++; if (ldet[id] == "silent") ds++ }
            if (lemb[id] == "silent" || lemb[id] == "offered") { en++; if (lemb[id] == "silent") es++ }
            dl = (ldet[id] == "offered") ? ldp[id] : ""
            el = (lemb[id] == "offered") ? lep2[id] : ""
            if (dl != "" && el != "") {
                both++
                if (hits(dl, "|" el "|")) share++
            }
            if (ldet[id] == "silent" && el != "") dsoe++
            if (rcount[id] > 0) {
                wr++; rd += rcount[id]
                dh += hits(dl, rs[id]); eh += hits(el, rs[id])
                drk[bucket(firstrank(dl, rs[id]))]++
                erk[bucket(firstrank(el, rs[id]))]++
            }
        }
        pre = sc[a] " " d " " gr[c] " "
        printf "%sprompts: %d\n", pre, np
        printf "%sroute absent: %d\n", pre, ab
        printf "%sdeterministic silent: %s\n", pre, frac(ds, dn)
        if (d == "none") printf "%sembedding silent: n/a, no model\n", pre
        else printf "%sembedding silent: %s\n", pre, frac(es, en)
        printf "%sboth offered: %d, share a document: %d\n", pre, both, share
        printf "%sdeterministic silent, embedding offered: %d\n", pre, dsoe
        printf "%sprompts with a read: %d\n", pre, wr
        printf "%sdeterministic recall: %s\n", pre, frac(dh, rd)
        if (d == "none") printf "%sembedding recall: n/a, no model\n", pre
        else printf "%sembedding recall: %s\n", pre, frac(eh, rd)
        printf "%sdeterministic first read at rank: 1=%d 2=%d 3=%d 4+=%d none=%d\n", pre, drk[1], drk[2], drk[3], drk["4+"], drk["none"]
        if (d == "none") printf "%sembedding first read at rank: n/a, no model\n", pre
        else printf "%sembedding first read at rank: 1=%d 2=%d 3=%d 4+=%d none=%d\n", pre, erk[1], erk[2], erk[3], erk["4+"], erk["none"]
    }
}' "$scratch/held.tsv" "$scratch/log.tsv" "$scratch/persons.tsv" "$scratch/reads.tsv"
