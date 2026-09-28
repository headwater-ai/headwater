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
#         parent.session   each parent session's id prefix, one per line, for the review hook
#         usage.jsonl      one plan-usage sample per start, log and end
#
# Under the common dir rather than under a worktree, so every worktree of the
# clone reaches it and none of them commits it. `git rev-parse
# --git-common-dir` answers the same path from every worktree.
#
#     sh tools/run/run-dir.sh start [run-id]        make it, print its path
#     sh tools/run/run-dir.sh log <dir> '<json>'    append one iteration
#     sh tools/run/run-dir.sh tail <dir> [n]        the last n lines, default 5
#     sh tools/run/run-dir.sh net <dir>             opened minus closed, derived
#     sh tools/run/run-dir.sh import <ledger> <dir> split an old ledger into it
#     sh tools/run/run-dir.sh claim <dir> <issue> <branch> <artifact>...
#                                               claim each artifact, or say who holds it
#     sh tools/run/run-dir.sh release <dir> <issue>  drop every claim the issue holds
#     sh tools/run/run-dir.sh claims <dir>           every claim, one per line
#     sh tools/run/run-dir.sh end <dir>              drop every claim, once the run has closed,
#                                                    then print what `usage` prints
#     sh tools/run/run-dir.sh usage <dir>            plan usage per closed issue, derived
#     sh tools/run/run-dir.sh stage <dir> <issue> <stage> [key=value]...
#                                               write the issue's checkpoint to handover/<issue>
#     sh tools/run/run-dir.sh next <dir>             the next queue issue to dispatch, DRAIN or EMPTY
#     sh tools/run/run-dir.sh session <dir>          append this resumed session to parent.session
#     sh tools/run/run-dir.sh resume <dir>           what each open handover needs, one line per issue
#     sh tools/run/run-dir.sh rule <dir> <issue> gated|deferred|refused|open <reason>
#                                               record a ruling in decisions.md, in the form next reads
#
# Three more files serve a parent that `tools/run/supervise.sh` restarts every
# few merges, so that its context stays small (#1275). A subagent dies with
# its parent's process, so a restart keeps only what is on disk:
#
#         queue.md         written by hw-queue; `next` reads its `N. #<issue>` lines
#         handover/<issue> the last stage the issue reached, and the paths to resume it
#         drain            present while the run drains to zero before a restart
#
# A stage is one of `adjudicated`, `built`, `verified-fail`, `verified-pass`
# and `ruled-merge`, and a field is one of `branch`, `pr`, `rounds`, `note`,
# `build`, `verify`, `attacks` and `footprint` (comma-separated artifacts);
# anything else is refused with exit 2.
# While `drain` exists, `stage`, `claim` and `next` each print the line
# `DRAIN`, which reaches the parent on calls it already makes. In drain,
# `claim` writes nothing and `next` hands out nothing, so no new stage
# starts, and `stage` still writes the checkpoint of the stage that ended.
# `next` skips an issue that holds a claim, has a handover or is in the log,
# which is what keeps a resumed parent from dispatching it a second time. It
# also skips an issue the run ruled out with `rule`, and a queue line marked
# `ruling` that no `OWNER` line answers, because a resumed parent reads only
# the last five ledger lines and would otherwise dispatch a gated issue.
# `parent.session` holds one line per parent session: `start` writes the
# first and `session` appends each resumed one, and a usage sample takes the
# last line, which is the live session.
#
# `start`, `log` and `end` each append one sample of the plan's rate-limit
# usage to `usage.jsonl`, so the parent and the integrator record it without
# doing anything: `usage` then derives how many points of the 5-hour and
# 7-day windows the run spent, over the issues it closed, and how many more
# issues at that rate fit in what is left of each window. The owner plans
# delivery against those two windows, and a run's cost in turns
# ([HW-PD-0003]) does not convert into either of them.
#
# The source is one JSON file per Claude Code session under
# `~/.cache/claude-usage/sessions/` (`HEADWATER_USAGE_DIR` overrides it),
# which the owner's status-line script writes from the `rate_limits` the
# harness hands it. Nothing in this repository writes that directory, so a
# host without it records no sample and says nothing: a sample is an
# observation of the host, and its absence must never fail a run. A
# session's figures are only as fresh as its own last API call, and idle
# sessions keep re-reporting old ones, so a sample takes the parent's own
# session where `parent.session` names it, and otherwise the session whose
# `last_activity` is newest. A sample is raw, and every figure is derived by
# `usage` and never stored, as for the log.
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
# `tools/run/run-dir-fixtures.sh` is what found it, and it stays there so that a
# primitive that stops being atomic is reported rather than trusted.
#
# `start` is create-only: an id that exists is refused rather than reused, so
# two parents cannot share a directory by accident. `log` refuses a line that
# lacks a required key, and it refuses a line that carries a total, because a
# stored total is the fold [HW-DR-0049] rules against. `jq` is a developer
# dependency here and nothing an adopter runs; `sh tools/run/run-dir-fixtures.sh`
# holds every refusal.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
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
    # The session that is this run's parent, so that the review-time hook
    # (.claude/hooks/review.sh) can end that session's turns without the commit
    # gate. The harness names the job directory after the session id's first
    # segment; outside the harness nothing is written and the hook runs as before.
    if [ -n "${CLAUDE_JOB_DIR:-}" ]; then
        printf '%s\n' "${CLAUDE_JOB_DIR##*/}" > "$dir/parent.session"
    fi
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
    sample "$dir" start
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
    # `opened` and `closed` name the issues, as an array of issue numbers, so a
    # reader can say which ones and derive how many. A bare count is still read
    # from an older log, but never written: run 20260923-0733 wrote both shapes
    # into one log and `net` failed on the mix.
    if ! printf '%s' "$line" | jq -e '[.opened, .closed] | all(type == "array" and all(.[]; type == "number"))' >/dev/null; then
        echo "run-dir: \`opened\` and \`closed\` are each an array of issue numbers, such as [] or [1029]." >&2
        exit 1
    fi
    if printf '%s' "$line" | jq -e 'keys[] | select(test("^(net|total|totals|cumulative)"))' >/dev/null; then
        echo "run-dir: the log line stores a total, and a total is derived by the reader and" >&2
        echo "  never stored (HW-DR-0049, HW-PD-0005). Drop it; \`run-dir.sh net\` derives it." >&2
        exit 1
    fi
    printf '%s' "$line" | jq -c . >> "$dir/log.jsonl"
    sample "$dir" log
}

# Where the per-session usage snapshots live. HEADWATER_USAGE_DIR overrides
# it, for a fixture.
usage_source() {
    printf '%s' "${HEADWATER_USAGE_DIR:-$HOME/.cache/claude-usage/sessions}"
}

# Append one usage sample, or nothing. Never an error and never a line on
# either stream: the caller is a ledger write, and the ledger write is what
# must succeed.
sample() {
    dir=$1 event=$2
    command -v jq >/dev/null 2>&1 || return 0
    src=$(usage_source)
    ls "$src"/*.json >/dev/null 2>&1 || return 0
    # One line per parent session, oldest first, so the live one is the last.
    parent=$(tail -n 1 "$dir/parent.session" 2>/dev/null)
    jq -c -n --arg parent "$parent" --arg event "$event" --argjson now "$(date +%s)" '
        [inputs | select(type == "object" and .last_activity != null)] as $all
        | [$all[] | select($parent != "" and ((.session_id // "") | startswith($parent)))] as $own
        | (if ($own | length) > 0 then {from: "parent", s: ($own | max_by(.last_activity))}
           else {from: "freshest", s: ($all | max_by(.last_activity))} end) as $pick
        | select($pick.s != null)
        | {at: ($now | todate), event: $event, from: $pick.from,
           session: (($pick.s.session_id // "")[:8]), age_s: ($now - $pick.s.last_activity),
           five_hour: $pick.s.five_hour, seven_day: $pick.s.seven_day}
    ' "$src"/*.json >> "$dir/usage.jsonl" 2>/dev/null || true
}

# What the run spent of each window, over what it closed. Consecutive samples
# under one `resets_at` add their rise; across a reset the later sample's
# figure is all that can be seen of that window, so it is added and the
# total is marked as a floor. A projection divides what is left of each
# window, at the last sample, by the points per closed issue.
plan_usage() {
    need_jq
    dir=$1
    [ -f "$dir/log.jsonl" ] || { echo "run-dir: no log at $dir." >&2; exit 1; }
    if [ ! -s "$dir/usage.jsonl" ]; then
        echo "no usage samples in $dir: the host wrote no snapshot under $(usage_source)"
        return 0
    fi
    jq -r -n --slurpfile log "$dir/log.jsonl" '
        [inputs] as $s
        | ($log | map(select(.verdict == "merged" or ((.merge | type) == "string" and .merge != ""))) | length) as $merges
        | ($log | map(.closed | if type == "array" then length elif type == "number" then . else 0 end) | add // 0) as $closed
        | def spent($w):
            [$s[] | .[$w] | select(. != null and .used_percentage != null)] as $p
            | if ($p | length) < 2 then null else
                reduce range(1; $p | length) as $i ({pts: 0, resets: 0};
                  if $p[$i].resets_at == $p[$i - 1].resets_at
                  then .pts += ([$p[$i].used_percentage - $p[$i - 1].used_percentage, 0] | max)
                  else .pts += $p[$i].used_percentage | .resets += 1 end)
                + {now: $p[-1].used_percentage}
              end;
          def line($name; $w):
            spent($w) as $x
            | if $x == null then "\($name)  fewer than two samples"
              else "\($name)  spent \($x.pts)\(if $x.resets > 0 then "+ (\($x.resets) reset)" else "" end) points, now at \($x.now)%"
                + (if $closed > 0 and $x.pts > 0
                   then ", \($x.pts / $closed * 10 | round / 10) per closed issue, room for \((100 - $x.now) / ($x.pts / $closed) | floor) more"
                   else "" end)
              end;
          "samples \($s | length) from \($s[0].at) to \($s[-1].at)  merges \($merges)  issues closed \($closed)",
          line("5-hour"; "five_hour"),
          line("7-day "; "seven_day"),
          (($s | map(.session) | unique) as $ids
           | if ($ids | length) > 1 then "warning: samples came from \($ids | length) sessions (\($ids | join(", "))), so a rise between two of them may be another session'"'"'s figure" else empty end),
          (($s | map(select(.from == "freshest")) | length) as $f
           | if $f > 0 then "note: \($f) sample(s) took the freshest session because no parent.session named one" else empty end)
    ' "$dir/usage.jsonl"
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
    # An array names issues and counts its length; a number is an older log's
    # count and counts as itself.
    count='def n: if type == "array" then length elif type == "number" then . else 0 end;'
    jq -s "$count"' map(.opened | n) | add // 0' "$dir/log.jsonl" > /dev/null || exit 1
    opened=$(jq -s "$count"' map(.opened | n) | add // 0' "$dir/log.jsonl")
    closed=$(jq -s "$count"' map(.closed | n) | add // 0' "$dir/log.jsonl")
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
    # In drain nothing starts a new stage, and a claim is how one starts. It
    # writes nothing, because an issue claimed here and never dispatched would
    # be skipped by `next` in every later session of the run.
    if [ -e "$dir/drain" ]; then
        echo DRAIN
        return 0
    fi
    # An artifact is one path or one glob, so whitespace means a footprint was
    # quoted as one argument. That stores one joined slug, which collides with
    # nothing, and in run 20260923-0733 it hid three real overlaps until a
    # claim printed no WAITS-ON against an obvious one. Refuse before writing.
    for artifact in "$@"; do
        case "$artifact" in
            *[[:space:]]*)
                echo "run-dir: \`$artifact\` holds whitespace. Pass each artifact as its own argument." >&2
                exit 2 ;;
        esac
    done
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

# A claim stops being meaningful the moment its run ends: the issue that held
# it either merged (and released it already) or never will, and either way
# nothing is still waiting to happen under that run. This drops the whole
# `claims/` subtree rather than the run directory itself, because
# `log.jsonl`, `findings.jsonl`, `lessons.md` and `decisions.md` are the
# record the next run reads — `start`'s own seeding step reads `lessons.md`
# and `decisions.md` out of the newest other directory, and a run this deleted
# outright could stop being that newest directory's source between one run
# ending and the next one starting. [#926] is the finding this answers: a
# stale claim from a run that will never finish reported `WAITS-ON` against a
# claimant that could otherwise have gone straight through.
end() {
    dir=$1
    [ -d "$dir" ] || { echo "run-dir: $dir is not a run directory." >&2; exit 1; }
    freed=0
    if [ -d "$dir/claims" ]; then
        freed=$(find "$dir/claims/artifacts" -type f 2>/dev/null | wc -l | tr -d ' ')
    fi
    rm -rf "$dir/claims"
    sample "$dir" end
    printf 'ENDED: %s, %s claims dropped\n' "$dir" "${freed:-0}"
    # The parent's closing report quotes what `end` prints, so the figures a
    # run spent reach the owner without a second command in next-run.md.
    if [ -s "$dir/usage.jsonl" ] && [ -f "$dir/log.jsonl" ] && command -v jq >/dev/null 2>&1; then
        plan_usage "$dir"
    fi
}

# The checkpoint of one issue, written at the boundary of each stage by the
# actor that ended it. A restarted parent reads `handover/` and nothing else
# to learn what was in flight, because a subagent dies with its parent's
# process and only the disk survives. Last write wins: one actor owns the
# stage of an issue at a time. Temp file, then `mv`, so a reader never sees
# half a checkpoint.
stages='adjudicated built verified-fail verified-pass ruled-merge'
fields='branch pr rounds note build verify attacks footprint'

stage() {
    dir=$1 issue=$2 reached=$3
    shift 3
    [ -d "$dir" ] || { echo "run-dir: $dir is not a run directory." >&2; exit 1; }
    case " $stages " in
        *" $reached "*) ;;
        *) echo "run-dir: \`$reached\` is not a stage. A stage is one of: $stages." >&2; exit 2 ;;
    esac
    case $issue in
        ''|*[!0-9]*) echo "run-dir: \`$issue\` is not an issue number." >&2; exit 2 ;;
    esac
    body=$(printf 'stage %s' "$reached")
    for pair in "$@"; do
        key=${pair%%=*} value=${pair#*=}
        case $pair in
            *=?*) ;;
            *) echo "run-dir: \`$pair\` is not key=value with a value." >&2; exit 2 ;;
        esac
        case " $fields " in
            *" $key "*) ;;
            *) echo "run-dir: \`$key\` is not a handover field. A field is one of: $fields." >&2; exit 2 ;;
        esac
        case $value in
            *[[:space:]]*) echo "run-dir: the value of \`$key\` holds whitespace, and a field is one line of one word." >&2; exit 2 ;;
        esac
        body=$(printf '%s\n%s %s' "$body" "$key" "$value")
    done
    mkdir -p "$dir/handover"
    tmp="$dir/handover/.$issue.$$"
    printf '%s\n' "$body" > "$tmp" && mv "$tmp" "$dir/handover/$issue" || { rm -f "$tmp"; exit 1; }
    [ -e "$dir/drain" ] && echo DRAIN
    return 0
}

# The issues the log records, space-separated. A log line means the run is
# done with the issue, whatever its verdict, so nothing hands it out again.
logged_issues() {
    [ -s "$1/log.jsonl" ] || return 0
    need_jq
    jq -r '.issue // empty' "$1/log.jsonl" 2>/dev/null | tr '\n' ' '
}

# The last ruling of each issue in decisions.md, one `<issue> <kind>` per line.
ruled_issues() {
    [ -f "$1/decisions.md" ] || return 0
    sed -n 's/^.* — RULED #\([0-9][0-9]*\) \(gated\|deferred\|refused\|open\):.*/\1 \2/p' "$1/decisions.md" \
        | awk '{ last[$1] = $2 } END { for (i in last) print i, last[i] }'
}

# The issues an OWNER line in decisions.md answers, space-separated.
answered_issues() {
    [ -f "$1/decisions.md" ] || return 0
    sed -n 's/^.*OWNER\( ([^)]*)\)\{0,1\} #\([0-9][0-9]*\).*/\2/p' "$1/decisions.md" | tr '\n' ' '
}

# The next issue to dispatch: the first queue line `N. #<issue> ...` whose
# issue holds no claim, has no handover, is in no log line, and is not ruled
# out. It is ruled out when its last `RULED` line says gated, deferred or
# refused, or when its queue line is marked `ruling` and neither an `OWNER`
# line nor a `RULED ... open` line answers it (hw-run-policy: an unanswered
# ruling waits, and a deferral skips the issue for the run). `DRAIN` when the
# run is draining, and `EMPTY` when nothing is left.
next() {
    dir=$1
    [ -d "$dir" ] || { echo "run-dir: $dir is not a run directory." >&2; exit 1; }
    if [ -e "$dir/drain" ]; then
        echo DRAIN
        return 0
    fi
    [ -f "$dir/queue.md" ] || { echo "run-dir: $dir holds no queue.md." >&2; exit 1; }
    logged=$(logged_issues "$dir")
    ruled=$(ruled_issues "$dir")
    answered=$(answered_issues "$dir")
    # `<issue> ruling` or `<issue> -`, from the last `|` field of the line.
    sed -n 's/^[0-9][0-9]*\. #\([0-9][0-9]*\) .*|[[:space:]]*\(ruling\).*$/\1 \2/p; t; s/^[0-9][0-9]*\. #\([0-9][0-9]*\).*/\1 -/p' "$dir/queue.md" > "$dir/.next.$$"
    while read -r issue mark; do
        [ -e "$dir/claims/issues/$issue" ] && continue
        [ -e "$dir/handover/$issue" ] && continue
        case " $logged " in *" $issue "*) continue ;; esac
        kind=$(printf '%s\n' "$ruled" | awk -v i="$issue" '$1 == i { print $2 }')
        case $kind in gated|deferred|refused) continue ;; esac
        if [ "$mark" = ruling ] && [ "$kind" != open ]; then
            case " $answered " in *" $issue "*) ;; *) continue ;; esac
        fi
        rm -f "$dir/.next.$$"
        echo "$issue"
        return 0
    done < "$dir/.next.$$"
    rm -f "$dir/.next.$$"
    echo EMPTY
}

# A ruling the parent makes on one issue, appended to decisions.md in the one
# form `next` reads: `- <date> — RULED #<issue> <kind>: <reason>`. The prose
# around it stays prose, and the last ruling of an issue is the one that holds.
rule() {
    dir=$1 issue=$2 kind=$3
    shift 3
    [ -d "$dir" ] || { echo "run-dir: $dir is not a run directory." >&2; exit 1; }
    case $issue in ''|*[!0-9]*) echo "run-dir: \`$issue\` is not an issue number." >&2; exit 2 ;; esac
    case $kind in
        gated|deferred|refused|open) ;;
        *) echo "run-dir: \`$kind\` is not a ruling. A ruling is one of: gated deferred refused open." >&2; exit 2 ;;
    esac
    reason="$*"
    [ -n "$reason" ] || { echo "run-dir: a ruling carries its reason." >&2; exit 2; }
    printf -- '- %s — RULED #%s %s: %s\n' "$(date -u +%Y-%m-%d)" "$issue" "$kind" "$reason" >> "$dir/decisions.md"
    printf 'RULED: #%s %s\n' "$issue" "$kind"
}

# What a resumed parent does with each open handover, one line per issue:
# `<issue> <stage> <action>`. An open handover is one the log does not record.
#
#     adjudicated, no claim   claim <artifacts>   claim the footprint, then dispatch hw-iterate
#     adjudicated, claimed    iterate             dispatch hw-iterate with the note
#     built, verified-fail    iterate             a fresh hw-iterate given the handover path
#     verified-pass           rule                the parent rules, from the notes it names
#     ruled-merge             integrate           append to the integrator queue
#
# An adjudicate that reports while the run drains is not claimed: `claim`
# writes nothing in drain. Its checkpoint carries the footprint instead, and
# the next session claims it here, so the issue is never adjudicated twice.
resume() {
    dir=$1
    [ -d "$dir" ] || { echo "run-dir: $dir is not a run directory." >&2; exit 1; }
    if [ -e "$dir/drain" ]; then
        echo DRAIN
        return 0
    fi
    [ -d "$dir/handover" ] || return 0
    logged=$(logged_issues "$dir")
    for h in "$dir"/handover/*; do
        [ -f "$h" ] || continue
        issue=${h##*/}
        case " $logged " in *" $issue "*) continue ;; esac
        reached=$(sed -n 's/^stage //p' "$h")
        case $reached in
            adjudicated)
                if [ -e "$dir/claims/issues/$issue" ]; then
                    action=iterate
                else
                    action="claim $(sed -n 's/^footprint //p' "$h" | tr ',' ' ')"
                fi ;;
            built|verified-fail) action=iterate ;;
            verified-pass) action=rule ;;
            ruled-merge) action=integrate ;;
            *) action=unknown ;;
        esac
        printf '%s %s %s\n' "$issue" "$reached" "$action"
    done
}

# A resumed parent session names itself, so the review hook exempts it and a
# usage sample reads its figures. Appended, never rewritten, and once.
session() {
    dir=$1
    [ -d "$dir" ] || { echo "run-dir: $dir is not a run directory." >&2; exit 1; }
    if [ -z "${CLAUDE_JOB_DIR:-}" ]; then
        echo "run-dir: no CLAUDE_JOB_DIR, so this is not a harness session and there is nothing to record." >&2
        exit 2
    fi
    id=${CLAUDE_JOB_DIR##*/}
    if [ "$(tail -n 1 "$dir/parent.session" 2>/dev/null)" != "$id" ]; then
        printf '%s\n' "$id" >> "$dir/parent.session"
    fi
    printf 'SESSION: %s\n' "$id"
}

case ${1:-} in
    start) shift; start "$@" ;;
    stage) [ $# -ge 4 ] || usage; shift; stage "$@" ;;
    next) [ $# -eq 2 ] || usage; next "$2" ;;
    resume) [ $# -eq 2 ] || usage; resume "$2" ;;
    rule) [ $# -ge 5 ] || usage; shift; rule "$@" ;;
    session) [ $# -eq 2 ] || usage; session "$2" ;;
    claim) [ $# -ge 5 ] || usage; shift; claim "$@" ;;
    release) [ $# -eq 3 ] || usage; release "$2" "$3" ;;
    claims) [ $# -eq 2 ] || usage; claims "$2" ;;
    end) [ $# -eq 2 ] || usage; end "$2" ;;
    log) [ $# -eq 3 ] || usage; log "$2" "$3" ;;
    tail) [ $# -ge 2 ] || usage; tail_log "$2" "${3:-5}" ;;
    net) [ $# -eq 2 ] || usage; net "$2" ;;
    usage) [ $# -eq 2 ] || usage; plan_usage "$2" ;;
    import) [ $# -eq 3 ] || usage; import "$2" "$3" ;;
    *) usage ;;
esac
