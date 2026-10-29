#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# The phrasings of a probe's task, read the one way that `campaign.sh` and
# `try.sh` both read them (#1875).
#
#     sh tools/probe/phrasing.sh count <root> <probe>
#     sh tools/probe/phrasing.sh cut <root> <probe> [<n>]
#     sh tools/probe/phrasing.sh files <root> <probe> <dir>
#     sh tools/probe/phrasing.sh draw <seed> <layer> <round> <probe> <repetition> <count>
#     sh tools/probe/phrasing.sh pick <probe> <repetition> <count> [<draw-file>]
#
# HW-DR-0112, "The form of a phrasing": under `## Task`, a probe has two or
# three subsections `### Phrasing 1`, `### Phrasing 2` and `### Phrasing 3`,
# and each one states the whole task. The probe document is the file under
# `<root>/docs/probes` that holds the line `id: <probe>`.
#
# `count` prints the number of `### Phrasing <n>` headings under `## Task`,
# and 0 for a task with none or for a probe that no document declares.
#
# `cut` prints the body of `## Task` without its blank lines at either end.
# With `<n>`, it prints the body of `### Phrasing <n>` alone, without its
# heading, up to the next heading of level three or above, so no other
# phrasing reaches it. That body
# is the prompt (spec 15). It prints nothing for a probe that no document
# declares, and nothing for a phrasing the task does not hold.
#
# `files` writes the task files of the probe under `<dir>`, as
# `campaign.sh` keeps them in `<out>/tasks/`: `<probe>.md`, the whole
# `## Task`, for a task with no phrasing, and otherwise `<probe>-<n>.md`, the
# cut of phrasing `<n>`, for each `<n>` from 1 to the count. It writes the
# count in `<probe>.phrasings`, and it removes a task file of the probe that
# the count does not name. It prints the path of the first file it wrote
# empty, and nothing when each one holds a body.
#
# `draw` prints the phrasing that a fixed draw gives the session, from 1 to
# `<count>`, and 1 where `<count>` is 0, because a task with no phrasing is
# one phrasing. It is the first eight hexadecimal digits of the sha256 of
# `<seed> <layer> <round> <probe> <repetition>`, modulo the count, plus 1. So
# the draw is a function of the seed, the open layer, the round and the
# probe, and every variant of a round reads the same draw. It reads no file.
#
# `pick` prints the phrasing of the job of `<probe>` at `<repetition>`, and 0
# where `<count>` is 0, which names the whole `## Task`. With no draw file,
# repetition `r` takes phrasing `((r - 1) mod count) + 1`, so the phrasings
# are spread evenly over the probe's sessions: a confirmation, a best so far
# and a base. With a draw file, the job takes the `r`-th `draw: <probe> <n>`
# line of the file for its probe: the screen's fixed draw. It exits 5 when
# the file names the probe fewer than `r` times, or names a phrasing above
# the count (above 1 for a task with no phrasing).
#
# Exit status: 0, 2 on a usage error, and 5 on a draw that does not cover the
# job.

set -u

usage() {
    echo "usage: sh tools/probe/phrasing.sh count <root> <probe> | cut <root> <probe> [<n>] | files <root> <probe> <dir> | draw <seed> <layer> <round> <probe> <repetition> <count> | pick <probe> <repetition> <count> [<draw-file>]" >&2
    exit 2
}
whole() {
    case $1 in '' | *[!0-9]*) return 1 ;; esac
}

# file <root> <probe>: the probe's document, or nothing.
file() {
    [ -d "$1/docs/probes" ] || return 0
    grep -rlx -- "id: $2" "$1/docs/probes" | head -1
}

[ $# -ge 1 ] || usage
verb=$1
shift
case $verb in
    count)
        [ $# = 2 ] || usage
        p_file=$(file "$1" "$2")
        if [ -z "$p_file" ]; then
            echo 0
            exit 0
        fi
        awk '/^## Task$/ { on = 1; next } on && /^## / { exit } on && /^### Phrasing [0-9]+$/ { n++ } END { print n + 0 }' "$p_file"
        ;;
    cut)
        [ $# = 2 ] || [ $# = 3 ] || usage
        p_n=${3:-}
        [ -z "$p_n" ] || whole "$p_n" || usage
        p_file=$(file "$1" "$2")
        [ -n "$p_file" ] || exit 0
        awk -v n="$p_n" '
            /^## Task$/ { on = 1; next }
            on && /^## / { exit }
            !on { next }
            n == "" { print; next }
            /^(#|##|###) / { inside = ($0 == "### Phrasing " n); next }
            inside { print }' "$p_file" \
            | awk 'NF { seen = 1 } seen' \
            | awk '{ lines[NR] = $0 } END { n = NR; while (n > 0 && lines[n] == "") n--; for (i = 1; i <= n; i++) print lines[i] }'
        ;;
    files)
        [ $# = 3 ] || usage
        f_count=$(sh "$0" count "$1" "$2") || exit $?
        [ -d "$3" ] || { echo "phrasing: no directory at $3." >&2; exit 2; }
        rm -f "$3/$2.md" "$3/$2"-[0-9]*.md
        printf '%s\n' "$f_count" > "$3/$2.phrasings"
        if [ "$f_count" = 0 ]; then
            sh "$0" cut "$1" "$2" > "$3/$2.md"
            [ -s "$3/$2.md" ] || echo "$3/$2.md"
            exit 0
        fi
        f_n=1
        while [ "$f_n" -le "$f_count" ]; do
            sh "$0" cut "$1" "$2" "$f_n" > "$3/$2-$f_n.md"
            [ -s "$3/$2-$f_n.md" ] || { echo "$3/$2-$f_n.md"; exit 0; }
            f_n=$((f_n + 1))
        done
        ;;
    draw)
        [ $# = 6 ] || usage
        whole "$5" && whole "$6" || usage
        d_count=$6
        [ "$d_count" -ge 1 ] || d_count=1
        d_hex=$(printf '%s %s %s %s %s' "$1" "$2" "$3" "$4" "$5" | sha256sum | cut -c1-8)
        echo $((0x$d_hex % d_count + 1))
        ;;
    pick)
        [ $# = 3 ] || [ $# = 4 ] || usage
        k_probe=$1 k_rep=$2 k_count=$3 k_draw=${4:-}
        whole "$k_rep" && whole "$k_count" && [ "$k_rep" -ge 1 ] || usage
        if [ -z "$k_draw" ]; then
            if [ "$k_count" = 0 ]; then echo 0; else echo $(((k_rep - 1) % k_count + 1)); fi
            exit 0
        fi
        if [ ! -f "$k_draw" ]; then
            echo "phrasing: no draw file at $k_draw." >&2
            exit 5
        fi
        k_n=$(awk -v p="$k_probe" -v r="$k_rep" '$1 == "draw:" && $2 == p { seen++; if (seen == r) { print $3; exit } }' "$k_draw")
        if [ -z "$k_n" ]; then
            echo "phrasing: the draw $k_draw names $k_probe fewer than $k_rep times, so its session $k_rep has no phrasing." >&2
            exit 5
        fi
        k_most=$k_count
        [ "$k_most" -ge 1 ] || k_most=1
        if ! whole "$k_n" || [ "$k_n" -lt 1 ] || [ "$k_n" -gt "$k_most" ]; then
            echo "phrasing: the draw $k_draw gives $k_probe phrasing $k_n, and its task holds $k_count phrasings." >&2
            exit 5
        fi
        if [ "$k_count" = 0 ]; then echo 0; else echo "$k_n"; fi
        ;;
    *) usage ;;
esac
