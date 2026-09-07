#!/bin/sh
# The run directory of a build-order run, and the five files in it.
#
# A run of `.claude/commands/next-run.md` used to keep one ledger at
# `~/.claude/headwater-build-order-ledger.md`, read whole at the top of every
# iteration. It reached 713 KB in eleven runs, nothing had ever been removed
# from it, and every iteration paid for all of it before doing anything.
# [HW-PD-0005] splits it: a directory of small files, the tabular parts as
# JSON lines, and every total derived by the reader and never stored, which is
# [HW-DR-0049] applied to a ledger.
#
#     <git common dir>/headwater-run/<run-id>/
#         doctrine.md      copied from .claude/run/doctrine.md at start
#         log.jsonl        one object per iteration
#         findings.jsonl   one object per finding handed forward
#         lessons.md       prose, seeded from the previous run
#         decisions.md     prose, seeded from the previous run
#
# Under the common dir rather than under a worktree, so every worktree of the
# clone reaches it and none of them commits it. `git rev-parse
# --git-common-dir` answers the same path from every worktree.
#
#     sh tools/run-dir.sh start [run-id]        make it, print its path
#     sh tools/run-dir.sh log <dir> '<json>'    append one iteration
#     sh tools/run-dir.sh tail <dir> [n]        the last n lines, default 5
#     sh tools/run-dir.sh net <dir>             opened minus closed, derived
#     sh tools/run-dir.sh import <ledger> <dir> split an old ledger into it
#     sh tools/run-dir.sh claim <dir> <issue> <branch> <artifact>...
#                                               claim each artifact, or say who holds it
#     sh tools/run-dir.sh release <dir> <issue>  drop every claim the issue holds
#     sh tools/run-dir.sh claims <dir>           every claim, one per line
#
# A claim is a file at `<dir>/claims/artifacts/<artifact>`, opened with
# `set -C`, which dash implements as O_EXCL, so two parents claiming one
# artifact at once cannot both succeed: the kernel serializes the create, and
# the second reads the first's file and reports `WAITS-ON: <issue>`. A claim is
# never empty: the file holds the issue and the branch, which is what
# [HW-PD-0004] asks of a claim and what the identifier claim store learned the
# hard way. Authority stays on the tree: a claim orders the integrator, it
# refuses nobody, and the veto still travels by message.
#
# NOT `mkdir`, which was the first choice and the textbook primitive. On a host
# whose `/bin/mkdir` is uutils coreutils 0.8.0, two racing `mkdir` calls on
# one path both succeeded in 17 of 20 races, while a sequential second call is
# refused, so every test that did not race passed. A noclobber redirect, `ln`,
# `ln -s` and Python's `os.mkdir` each held at 0 of 20. The race case in
# `tools/run-dir-fixtures.sh` is what found it, and it stays there so that a
# primitive that stops being atomic is reported rather than trusted.
#
# `start` is create-only: an id that exists is refused rather than reused, so
# two parents cannot share a directory by accident. `log` refuses a line that
# lacks a required key, and it refuses a line that carries a total, because a
# stored total is the fold [HW-DR-0049] rules against. `jq` is a developer
# dependency here and nothing an adopter runs; `sh tools/run-dir-fixtures.sh`
# holds every refusal.

set -u

root=$(cd "$(dirname "$0")/.." && pwd)
keys='iter issue pr merge verdict proved opened closed'

usage() {
    sed -n '/^#     sh tools\/run-dir.sh/p' "$0" | sed 's/^# *//' >&2
    exit 2
}

need_jq() {
    command -v jq >/dev/null 2>&1 && return
    echo "run-dir: \`jq\` is not on the path, and the ledger's tabular parts are JSON lines." >&2
    exit 2
}

# Where runs live. HEADWATER_RUN_ROOT overrides it, for a fixture.
runs_root() {
    if [ -n "${HEADWATER_RUN_ROOT:-}" ]; then
        printf '%s' "$HEADWATER_RUN_ROOT"
        return
    fi
    common=$(git -C "$root" rev-parse --git-common-dir) || exit 2
    case $common in
        /*) ;;
        *) common="$root/$common" ;;
    esac
    printf '%s/headwater-run' "$common"
}

start() {
    id=${1:-$(date -u +%Y%m%d-%H%M)}
    runs=$(runs_root)
    mkdir -p "$runs"
    dir="$runs/$id"
    if ! mkdir "$dir" 2>/dev/null; then
        echo "run-dir: $dir exists, and a run directory is never reused. Pick another id." >&2
        exit 1
    fi
    cp "$root/.claude/run/doctrine.md" "$dir/doctrine.md"
    : > "$dir/log.jsonl"
    : > "$dir/findings.jsonl"
    # Seed the prose from the most recent run that has any, so a lesson
    # survives the run that learned it. The newest directory by name, which
    # is the newest by date under the default id.
    previous=$(ls -1 "$runs" | grep -vx "$id" | sort | tail -1)
    for file in lessons.md decisions.md; do
        if [ -n "$previous" ] && [ -s "$runs/$previous/$file" ]; then
            cp "$runs/$previous/$file" "$dir/$file"
        else
            : > "$dir/$file"
        fi
    done
    printf '%s\n' "$dir"
}

log() {
    need_jq
    dir=$1 line=$2
    [ -d "$dir" ] || { echo "run-dir: $dir is not a run directory." >&2; exit 1; }
    if ! printf '%s' "$line" | jq -e 'type == "object"' >/dev/null 2>&1; then
        echo "run-dir: the log line is not a JSON object." >&2
        exit 1
    fi
    for key in $keys; do
        if ! printf '%s' "$line" | jq -e --arg k "$key" 'has($k)' >/dev/null; then
            echo "run-dir: the log line lacks \`$key\`. Every line carries: $keys." >&2
            exit 1
        fi
    done
    if printf '%s' "$line" | jq -e 'keys[] | select(test("^(net|total|totals|cumulative)"))' >/dev/null; then
        echo "run-dir: the log line stores a total, and a total is derived by the reader and" >&2
        echo "  never stored (HW-DR-0049, HW-PD-0005). Drop it; \`run-dir.sh net\` derives it." >&2
        exit 1
    fi
    printf '%s' "$line" | jq -c . >> "$dir/log.jsonl"
}

tail_log() {
    dir=$1 n=${2:-5}
    [ -f "$dir/log.jsonl" ] || { echo "run-dir: no log at $dir." >&2; exit 1; }
    tail -n "$n" "$dir/log.jsonl"
}

net() {
    need_jq
    dir=$1
    [ -f "$dir/log.jsonl" ] || { echo "run-dir: no log at $dir." >&2; exit 1; }
    jq -s 'map(.opened) | add // 0' "$dir/log.jsonl" > /dev/null || exit 1
    opened=$(jq -s 'map(.opened) | add // 0' "$dir/log.jsonl")
    closed=$(jq -s 'map(.closed) | add // 0' "$dir/log.jsonl")
    lines=$(wc -l < "$dir/log.jsonl" | tr -d ' ')
    printf 'iterations %s  opened %s  closed %s  net %s\n' "$lines" "$opened" "$closed" $((opened - closed))
}

# The one-time import of the old single-file ledger. Its three prose sections
# become the three prose files, and its Log section, which was prose rows
# rather than records, is kept whole as `log-imported.md` for a reader rather
# than parsed into lines that would be guesses.
import() {
    ledger=$1 dir=$2
    [ -f "$ledger" ] || { echo "run-dir: no ledger at $ledger." >&2; exit 1; }
    [ -d "$dir" ] || { echo "run-dir: $dir is not a run directory." >&2; exit 1; }
    section() {
        awk -v want="$1" '
            /^## / { if (on) done = 1; on = (!done && $0 == "## " want); next }
            /^# / { if (on) done = 1; on = 0 }
            on { print }
        ' "$ledger"
    }
    section 'Lessons' >> "$dir/lessons.md"
    section 'Decisions needing an owner' >> "$dir/decisions.md"
    section 'Open findings' > "$dir/findings-imported.md"
    section 'Log' > "$dir/log-imported.md"
    for file in lessons.md decisions.md findings-imported.md log-imported.md; do
        printf '%s %s lines\n' "$file" "$(wc -l < "$dir/$file" | tr -d ' ')"
    done
}

# An artifact name is a path segment, so a slash in it would make a claim on
# `docs/foo` a file two levels down and a different claim from `docs-foo`.
# Slashes fold to `-` before the claim is made. A leading dot goes too: a claim
# on `.headwater/export.json` would otherwise be a dotfile that the `*` glob in
# `release` and `claims` never sees, which the suite caught as a claim that was
# made and could not be freed.
slug() {
    printf '%s' "$1" | tr '/' '-' | sed 's/^\.*//'
}

# The create-only primitive. Exit 0 and the file written with the given
# content, or exit 1 and nothing written because the path already existed.
create_only() {
    path=$1 content=$2
    (set -C; printf '%s' "$content" > "$path") 2>/dev/null
}

claim() {
    dir=$1 issue=$2 branch=$3
    shift 3
    [ -d "$dir" ] || { echo "run-dir: $dir is not a run directory." >&2; exit 1; }
    [ $# -gt 0 ] || { echo "run-dir: claim needs at least one artifact." >&2; exit 2; }
    mkdir -p "$dir/claims/artifacts" "$dir/claims/issues"
    printf 'issue %s\nbranch %s\n' "$issue" "$branch" > "$dir/claims/issues/$issue"
    waits=''
    for artifact in "$@"; do
        s=$(slug "$artifact")
        if create_only "$dir/claims/artifacts/$s" "$(printf 'issue %s\nbranch %s\nartifact %s\n' "$issue" "$branch" "$artifact")"; then
            printf 'CLAIMED: %s\n' "$artifact"
        else
            holder=$(sed -n 's/^issue //p' "$dir/claims/artifacts/$s" 2>/dev/null)
            if [ "$holder" = "$issue" ]; then
                printf 'CLAIMED: %s (already held)\n' "$artifact"
            else
                printf 'HELD: %s by #%s\n' "$artifact" "${holder:-unknown}"
                case " $waits " in
                    *" ${holder:-unknown} "*) ;;
                    *) waits="$waits ${holder:-unknown}" ;;
                esac
            fi
        fi
    done
    if [ -n "$waits" ]; then
        printf 'WAITS-ON:%s\n' "$waits"
        printf 'WAITS-ON:%s\n' "$waits" >> "$dir/claims/issues/$issue"
    fi
}

release() {
    dir=$1 issue=$2
    [ -d "$dir/claims" ] || { echo "run-dir: no claims under $dir." >&2; exit 1; }
    freed=0
    for owner in "$dir"/claims/artifacts/*; do
        [ -f "$owner" ] || continue
        [ "$(sed -n 's/^issue //p' "$owner")" = "$issue" ] || continue
        rm -f "$owner"
        freed=$((freed + 1))
    done
    rm -f "$dir/claims/issues/$issue"
    printf 'RELEASED: %s artifacts held by #%s\n' "$freed" "$issue"
}

claims() {
    dir=$1
    [ -d "$dir/claims/artifacts" ] || return 0
    for owner in "$dir"/claims/artifacts/*; do
        [ -f "$owner" ] || continue
        if [ ! -s "$owner" ]; then
            printf 'EMPTY: %s\n' "$(basename "$owner")"
            continue
        fi
        printf '%s #%s %s\n' "$(sed -n 's/^artifact //p' "$owner")" "$(sed -n 's/^issue //p' "$owner")" "$(sed -n 's/^branch //p' "$owner")"
    done
}

case ${1:-} in
    start) shift; start "$@" ;;
    claim) [ $# -ge 5 ] || usage; shift; claim "$@" ;;
    release) [ $# -eq 3 ] || usage; release "$2" "$3" ;;
    claims) [ $# -eq 2 ] || usage; claims "$2" ;;
    log) [ $# -eq 3 ] || usage; log "$2" "$3" ;;
    tail) [ $# -ge 2 ] || usage; tail_log "$2" "${3:-5}" ;;
    net) [ $# -eq 2 ] || usage; net "$2" ;;
    import) [ $# -eq 3 ] || usage; import "$2" "$3" ;;
    *) usage ;;
esac
