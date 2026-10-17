#!/bin/sh
# The integrator's steps around the one merge call it makes itself.
#
#     sh tools/run/integrate.sh main [<sha>]              CI on the tip of origin/main, read once and never waited on
#     sh tools/run/integrate.sh check <PR> [<sha>]        the head, what the merge closes, and the clause check
#     sh tools/run/integrate.sh settle <run> <PR> [<issue>]
#                                                         one capped wait on the queue, then the closes and the release on a merge
#
# The merge call itself, `gh pr merge <PR> --squash --match-head-commit
# <sha>`, stays the integrator's own. The harness's permission classifier
# reads that command by its text, and a refusal there is the owner's to lift
# and never a script's to route around (`.claude/agents/hw-integrate.md`,
# *Enqueue each once*). Everything before it and after it never varies, and
# each half was three to six commands an integrator typed for each pull
# request, in the shapes worktree isolation refuses when joined.
#
# `main` reads `tools/run/ci-done.sh` on the tip of `origin/main`, or on the
# sha given, with no wait. It prints `MAIN: <sha> green`, `MAIN: <sha> red:
# <checks>` or `MAIN: <sha> unfinished` and exits 0 in every case, because
# the line is the report: a red one is the first line of `LEFT`, and the
# next integrator reads it again. Nothing rebuilds after a merge any more:
# the queue ran CI on the tree each squash commit lands, and CI on `main`
# runs `headwater generate --check` on every push, so a stale fold on `main`
# is this line and never a rebuild in a serial slot (45 of 45 post-merge
# regenerates in runs to 2026-10-17 moved nothing).
#
# `check` prints `HEAD: <sha>`, `CLOSES: [...]` and what `clause-check`
# prints. It exits 1 when `clause-check` refuses, and with `HEAD MOVED:`
# when a sha is given and the pull request's head is not that commit: the
# ruling was on a commit that is no longer the branch, and the parent rules
# again. The integrator enqueues only after a check that exited 0.
#
# `settle` runs `wait-for.sh 'queue-done.sh <PR>'` once, so it is run in the
# foreground with a Bash timeout of 300000 and run again on `RE-ISSUE`, which
# it passes through as exit 2. Once the queue is done it prints, by the case:
#
#     MERGED: <sha> for #<PR>                       then CLOSED: #<N> for each issue the merge closes,
#                                                   and RELEASED: ... for each of those and for <issue>
#     EJECTED: #<PR>: <reason>                      ejected, or unmergeable as merge_conflict (unmergeable)
#     LEFT: #<PR> <what queue-done printed>         stalled, closed, or not queued
#
# `close` re-reads the state and refuses to report a close that did not
# take, as `gh-issue.sh` says, so a `LEFT: #<N> close` line is one the
# integrator carries into its own `LEFT`. The optional <issue> is the ruled
# issue of a `Refs` pull request, whose claims are released although the
# merge closes nothing. `HW_QUEUE_DONE` and `HW_CI_DONE` name the two
# readers, `HW_SETTLE_CAP` and `HW_SETTLE_POLL` the wait's seconds, and
# `HEADWATER_RUN_ROOT` the runs, for a fixture.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
repo=headwater-ai/headwater
queue_done=${HW_QUEUE_DONE:-$root/tools/run/queue-done.sh}
ci_done=${HW_CI_DONE:-$root/tools/run/ci-done.sh}
settle_cap=${HW_SETTLE_CAP:-240}
settle_poll=${HW_SETTLE_POLL:-30}
gh_issue="$root/tools/run/gh-issue.sh"
run_dir="$root/tools/run/run-dir.sh"

usage() {
    sed -n '/^#     sh tools\/run\/integrate.sh/p' "$0" | sed 's/^# *//' >&2
    exit 2
}

need_number() {
    case $1 in
        '' | *[!0-9]*) echo "integrate: \`$2\` wants a number, got \`$1\`." >&2; exit 2 ;;
    esac
}

need_sha() {
    case $1 in '' | *[!0-9a-f]*) echo "integrate: \`$1\` is not a commit: seven to forty lower-case hex digits." >&2; exit 2 ;; esac
    [ ${#1} -ge 7 ] && [ ${#1} -le 40 ] || { echo "integrate: \`$1\` is not a commit: seven to forty lower-case hex digits." >&2; exit 2; }
}

main_ci() {
    if [ -n "${1:-}" ]; then
        sha=$1
        need_sha "$sha"
    else
        git fetch -q origin || { echo "integrate: git fetch origin failed." >&2; exit 1; }
        sha=$(git rev-parse origin/main) || exit 1
    fi
    out=$(sh "$ci_done" "$sha" 2>&1)
    status=$?
    if [ "$status" -ne 0 ]; then
        printf 'MAIN: %s unfinished (%s)\n' "$sha" "$(printf '%s\n' "$out" | tail -n 1)"
        return 0
    fi
    last=$(printf '%s\n' "$out" | tail -n 1)
    case $last in
        *' green') printf 'MAIN: %s green\n' "$sha" ;;
        *' red: '*) printf 'MAIN: %s red: %s\n' "$sha" "${last##* red: }" ;;
        *) printf 'MAIN: %s %s\n' "$sha" "$last" ;;
    esac
}

check() {
    pr=$1 ruled=${2:-}
    need_number "$pr" check
    [ -z "$ruled" ] || need_sha "$ruled"
    head=$(gh api "repos/$repo/pulls/$pr" --jq .head.sha 2>/dev/null)
    case $head in
        '' | *[!0-9a-f]*) echo "integrate: the head of #$pr could not be read from GitHub." >&2; exit 1 ;;
    esac
    printf 'HEAD: %s\n' "$head"
    if [ -n "$ruled" ]; then
        case $head in
            "$ruled"*) ;;
            *) printf 'HEAD MOVED: #%s is at %s, and the ruling was at %s. The parent rules it again.\n' "$pr" "$head" "$ruled"; exit 1 ;;
        esac
    fi
    closes=$(sh "$gh_issue" closes "$pr") || { echo "integrate: could not read what #$pr closes." >&2; exit 1; }
    printf 'CLOSES: %s\n' "$closes"
    sh "$gh_issue" clause-check "$pr"
}

settle() {
    run=$1 pr=$2 ruled=${3:-}
    need_number "$pr" settle
    [ -z "$ruled" ] || [ "$ruled" = none ] || need_number "$ruled" settle
    out=$(sh "$root/tools/run/wait-for.sh" --cap "$settle_cap" --poll "$settle_poll" "sh $queue_done $pr" 2>&1)
    status=$?
    if [ "$status" -ne 0 ]; then
        printf '%s\n' "$out" | tail -n 1 >&2
        exit "$status"
    fi
    line=$(printf '%s\n' "$out" | grep '^queue-done:' | tail -n 1)
    # Anchored on the verb, because a stalled line reads "has not merged in".
    case $line in
        "queue-done: #$pr merged "*)
            sha=${line##* }
            printf 'MERGED: %s for #%s\n' "$sha" "$pr"
            closes=$(sh "$gh_issue" closes "$pr" 2>/dev/null) || closes='[]'
            issues=$(printf '%s' "$closes" | tr -d '[] \n' | tr ',' ' ')
            released=''
            for n in $issues; do
                if sh "$gh_issue" close "$n" > /dev/null 2>&1; then
                    printf 'CLOSED: #%s\n' "$n"
                else
                    printf 'LEFT: #%s close did not take; run `sh tools/run/gh-issue.sh close %s`\n' "$n" "$n"
                fi
                sh "$run_dir" release "$run" "$n"
                released="$released $n"
            done
            if [ -n "$ruled" ] && [ "$ruled" != none ]; then
                case " $released " in
                    *" $ruled "*) ;;
                    *) sh "$run_dir" release "$run" "$ruled" ;;
                esac
            fi ;;
        "queue-done: #$pr unmergeable: "*)
            printf 'EJECTED: #%s: merge_conflict (unmergeable)\n' "$pr" ;;
        "queue-done: #$pr ejected: "*)
            printf 'EJECTED: #%s: %s\n' "$pr" "${line#* ejected: }" ;;
        *)
            printf 'LEFT: #%s %s\n' "$pr" "${line#queue-done: #"$pr" }" ;;
    esac
}

case ${1:-} in
    main) [ $# -le 2 ] || usage; main_ci "${2:-}" ;;
    check) [ $# -ge 2 ] && [ $# -le 3 ] || usage; check "$2" "${3:-}" ;;
    settle) [ $# -ge 3 ] && [ $# -le 4 ] || usage; settle "$2" "$3" "${4:-}" ;;
    *) usage ;;
esac
