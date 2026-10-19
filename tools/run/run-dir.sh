#!/bin/sh
# The run directory of a build-order run, and the files in it.
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
#     sh tools/run/run-dir.sh intake <dir> <file>    append a file of intake lines to intake.md
#     sh tools/run/run-dir.sh decide <dir> <text>    append one dated line to decisions.md
#     sh tools/run/run-dir.sh decide <dir> --owner <issue>... -- <quote>
#                                               append the owner's answer, in the form next reads
#     sh tools/run/run-dir.sh queue <dir> <issue> title=… milestone=… reader=… artifacts=… scope=wide|narrow [mark=ruling] [blocked=<M>[,<milestone>]]... [note=…]...
#                                               append one issue to queue.jsonl, and render queue.md
#     sh tools/run/run-dir.sh convert <dir>          turn a run's hand-written queue.md into queue.jsonl, once
#     sh tools/run/run-dir.sh owe <dir> <issue> title=… question=… options=<a>|<b>|defer unblocks=…
#                                               append one ruling the owner owes, and render rulings.md
#     sh tools/run/run-dir.sh rulings <dir>          render rulings.md, each block marked answered or not, and print it
#     sh tools/run/run-dir.sh integrate <dir> <pr> issue=<N|none> branch=… ruling=… footprint=<a,b> [sha=…]
#                                               append one merge ruling to integrate.jsonl; without sha=, read the head from GitHub
#     sh tools/run/run-dir.sh integrations <dir>     each ruled pull request the log has not recorded merged
#     sh tools/run/run-dir.sh lesson <dir> <file>    append a file of prose to lessons.md
#     sh tools/run/run-dir.sh withdraw <dir> <match> <line>
#                                               replace the one intake line that holds <match>
#     sh tools/run/run-dir.sh applies <dir>          each intake line the product owner ruled APPLY,
#                                                    then one count line, APPLY: <n>
#     sh tools/run/run-dir.sh dispatch <dir> <issue|none> <stage> [key=value]...
#                                               compose the stage's dispatch in the issue's scratch directory, print its path
#     sh tools/run/run-dir.sh accept <dir> <issue> branch=… note=… footprint=<a,b|none> [title=…] [attacks=…] [deadline=…]
#                                               a BUILD report: the checkpoint, the claim and the hw-iterate dispatch, in one call
#     sh tools/run/run-dir.sh po-rule <dir> <file>   record each RULING block of a file through owe
#
# `dispatch` is the one renderer of the dispatch template that
# `.claude/commands/next-run.md` describes. A parent used to compose each
# dispatch by hand with `Write`, from a template it had to keep in context,
# and a worktree-isolated parent's shell refuses the compound forms that
# would have scripted it. The fields, by stage:
#
#     adjudicate  issue, run, scratch, [deadline]
#     iterate     + branch, note= (the adjudication) or handover= (a checkpoint), waits-on, [attacks]
#     build       + branch, note or handover, waits-on, [attacks]
#     verify      + branch, pr, slot, [note], [build] (the build note), [attacks]
#     integrate   issue none; one `pull request:` line per ruling `integrations` prints
#
# `title=` is read from queue.jsonl when the call gives none. The scratch
# directory is `$CLAUDE_JOB_DIR/tmp/issue-<N>/`, as the template states, and
# `HEADWATER_SCRATCH` overrides its root for a fixture. An issue of `none`
# with `iterate` or `build` is the run-lessons dispatch, whose note is the run
# id and whose branch is `run-lessons-<run-id>`.
#
# Every <dir>, for every subcommand but `start`, may be a bare run id, such as `20260927-0443`, which names that
# run under the git common dir. A worktree-isolated parent's shell refuses a
# command whose text names `.git`, and `Edit` refuses a path there, so the id
# and the subcommands are its only way to write the ledger. Run
# `20260927-0443` wrote four scripts of its own for exactly this.
#
# Three more files serve a parent that `tools/run/supervise.sh` restarts every
# few merges, so that its context stays small (#1275). A subagent dies with
# its parent's process, so a restart keeps only what is on disk:
#
#         queue.jsonl      one object per issue, written by hw-queue through `queue`; `next` reads it
#         queue.md         rendered from queue.jsonl for a person; nothing reads it
#         handover/<issue> the last stage the issue reached, and the paths to resume it
#         drain            present while the run drains to zero before a restart
#
# Two more are written through a verb for the same reason: a file a script
# reads is structured data, and a malformed entry is refused when it is
# written, not when a later reader meets it after its writer has exited (#1724).
#
#         rulings.jsonl    one object per ruling the owner owes, written by the product owner through `owe`
#         rulings.md       rendered from rulings.jsonl and decisions.md; nothing reads it
#         integrate.jsonl  one object per merge ruling, appended by the parent through `integrate`
#         queue-legacy.md, rulings-legacy.md
#                          a run's hand-written file, kept whole when `convert` or the first `owe` replaces it
#
# Each of the three is per run, so `start` seeds none of them. An entry is
# never rewritten: every verb that writes one of them appends, and a renderer
# writes only its `.md` view. `withdraw` rewrites one line of intake.md, and
# nothing opens any of the three for anything but an append.
#
# A stage is one of `adjudicated`, `built`, `verified-fail`, `verified-pass`
# and `ruled-merge`, and a field is one of `branch`, `pr`, `rounds`, `note`,
# `build`, `verify`, `attacks` and `footprint` (comma-separated artifacts);
# anything else is refused with exit 2.
# While `drain` exists, `stage`, `claim` and `next` each print the line
# `DRAIN`, which reaches the parent on calls it already makes. In drain,
# `claim` writes nothing and `next` hands out nothing, so no new stage
# starts, and `stage` still writes the checkpoint of the stage that ended.
# When the main checkout is behind origin/main, `start` prints the line
# `STALE: ...` that `tools/run/stale-main.sh` writes, exits 1 and makes no
# run directory, because the harness reads the agent definitions a run
# dispatches from that checkout's working tree (#1811). `HW_RUN_ALLOW_STALE=1`
# turns the refusal into the same line as a warning, and the path is still
# the last line. `stage`, `claim` and `next` print the same line first, as
# they print `DRAIN`, and do what they would have done: they report and
# never refuse. The checkout judged is the main checkout of
# `HW_RUN_CHECKOUT`, a directory of a repository, which defaults to this
# repository; a fixture points it at a scratch clone.
# `next` skips an issue that holds a claim, has a handover or is in the log,
# which is what keeps a resumed parent from dispatching it a second time. It
# also skips an issue the run ruled out with `rule`, and a queue line marked
# `ruling` that no `OWNER` line answers, because a resumed parent reads only
# the last five ledger lines and would otherwise dispatch a gated issue.
# A `RULED` line names the run that wrote it and binds that run alone, and an
# `OWNER` line binds every later run too (see `ruled_issues`). A `stage` call
# carries every field it does not name from the checkpoint before.
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
    sed -n '/^#     sh tools\/run\/run-dir.sh/p' "$0" | sed 's/^# *//' >&2
    exit 2
}

# The STALE line, when the main checkout is behind origin/main, and nothing
# otherwise. Exit 1 when it printed one. `stale_said` silences a second print
# inside `accept`, which runs `stage` and `claim` in one call.
stale_check() {
    [ -z "${stale_said:-}" ] || return 0
    sh "$root/tools/run/stale-main.sh" "${HW_RUN_CHECKOUT:-$root}"
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

# A bare run id names that run under the runs root. A path, or a name that is
# already a directory here, stays as it is.
resolve() {
    case $1 in
        */*) printf '%s' "$1" ;;
        *) if [ -d "$1" ]; then printf '%s' "$1"; else printf '%s/%s' "$(runs_root)" "$1"; fi ;;
    esac
}

need_dir() {
    [ -d "$1" ] || { echo "run-dir: $1 is not a run directory." >&2; exit 1; }
}

intake() {
    dir=$1
    need_dir "$dir"
    [ -f "$2" ] || { echo "run-dir: $2 is not a file of intake lines." >&2; exit 1; }
    cat "$2" >> "$dir/intake.md"
    printf 'INTAKE: %s lines appended\n' "$(wc -l < "$2" | tr -d ' ')"
}

# Lessons are prose under a heading, several lines at a time, so the parent
# composes them in a file with `Write` and this appends the file whole.
lesson() {
    dir=$1
    need_dir "$dir"
    [ -s "$2" ] || { echo "run-dir: $2 is not a file of lesson prose." >&2; exit 1; }
    cat "$2" >> "$dir/lessons.md"
    printf 'LESSON: %s lines appended\n' "$(wc -l < "$2" | tr -d ' ')"
}

nl='
'

refuse() {
    echo "run-dir: $1" >&2
    exit 2
}

# A plain decision is prose. An OWNER line is not: `answered_issues` reads its
# issue numbers in every later run, so it is written only by `--owner`, with
# the issues as arguments. A hand-typed `OWNER (<date>) #990, #1275:` would
# answer neither, and the form that took the first `#N` alone answered one.
decide() {
    dir=$1
    shift
    need_dir "$dir"
    if [ "${1:-}" = --owner ]; then
        shift
        decide_owner "$dir" "$@"
        return
    fi
    [ $# -eq 1 ] || usage
    case $(printf '%s' "$1" | sed 's/^[[:space:]]*//') in
        OWNER*) refuse "a decision that opens \`OWNER\` is the owner's answer, and \`next\` reads one only in the form \`decide <run> --owner <issue>... -- <quote>\` writes." ;;
    esac
    printf -- '- %s — %s\n' "$(date -u +%Y-%m-%d)" "$1" >> "$dir/decisions.md"
}

decide_owner() {
    dir=$1
    shift
    issues=''
    while [ $# -gt 0 ] && [ "$1" != -- ]; do
        case $1 in ''|*[!0-9]*) refuse "\`$1\` is not an issue number, and \`--owner\` takes issue numbers up to \`--\`." ;; esac
        issues="$issues #$1"
        shift
    done
    [ -n "$issues" ] || refuse "an OWNER line names the issue it answers: \`decide <run> --owner <issue>... -- <quote>\`."
    [ "${1:-}" = -- ] || refuse "an OWNER line takes \`--\` between its issues and the owner's words."
    shift
    quote="$*"
    case $quote in *"$nl"*) refuse "the owner's words are one line." ;; esac
    [ -n "$(printf '%s' "$quote" | tr -d '[:space:]')" ] || refuse "an OWNER line quotes the owner's words, and this one quotes nothing."
    day=$(date -u +%Y-%m-%d)
    printf -- '- %s — OWNER (%s)%s: %s\n' "$day" "$day" "$issues" "$quote" >> "$dir/decisions.md"
    render_rulings "$dir"
    printf 'OWNER:%s\n' "$issues"
}

# Exactly one line may hold the match, so a withdrawal never rewrites the
# wrong finding, or two of them.
withdraw() {
    dir=$1
    need_dir "$dir"
    file="$dir/intake.md"
    hits=$(grep -cF -- "$2" "$file" 2>/dev/null)
    if [ "${hits:-0}" -ne 1 ]; then
        echo "run-dir: ${hits:-0} intake lines hold \`$2\`, and a withdrawal replaces exactly one." >&2
        exit 1
    fi
    match=$2 line=$3 awk 'index($0, ENVIRON["match"]) { print ENVIRON["line"]; next } { print }' "$file" > "$file.tmp" &&
        mv "$file.tmp" "$file"
}

# The product owner rules a one-line remedy by ending its intake line with
# ` | APPLY` (`.claude/agents/headwater-product-owner.md`, `## Intake`). The
# end-of-run lessons pull request reads the lines through this verb, so the
# parent never reads intake.md whole. A line that only mentions APPLY in its
# text is not ruled, because the ruling is the last field alone.
applies() {
    dir=$1
    need_dir "$dir"
    if [ ! -f "$dir/intake.md" ]; then
        printf 'APPLY: 0\n'
        return
    fi
    awk '/ [|] APPLY[[:space:]]*$/ { print; n++ } END { printf "APPLY: %d\n", n }' "$dir/intake.md"
}

start() {
    id=${1:-$(date -u +%Y%m%d-%H%M)}
    if ! stale_check && [ "${HW_RUN_ALLOW_STALE:-}" != 1 ]; then
        exit 1
    fi
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
    stale_check
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
    # A comma is the same mistake in the shape the `footprint=` field of
    # `stage` and `integrate` uses: `claim <run> 41 b a,b` stored one slug
    # `a,b` in run 20261012-1113 and printed `HELD: ... by #unknown` against
    # nothing. No artifact of this repository has a comma in its path.
    for artifact in "$@"; do
        case "$artifact" in
            *[[:space:]]*)
                echo "run-dir: \`$artifact\` holds whitespace. Pass each artifact as its own argument." >&2
                exit 2 ;;
            *,*)
                echo "run-dir: \`$artifact\` holds a comma. Pass each artifact as its own argument, not the comma-joined footprint." >&2
                exit 2 ;;
        esac
    done
    mkdir -p "$dir/claims/artifacts" "$dir/claims/issues"
    printf 'issue %s\nbranch %s\n' "$issue" "$branch" > "$dir/claims/issues/$issue"
    waits=''
    for artifact in "$@"; do
        s=$(slug "$artifact")
        content=$(printf 'issue %s\nbranch %s\nartifact %s\n' "$issue" "$branch" "$artifact")
        if create_only "$dir/claims/artifacts/$s" "$content"; then
            printf 'CLAIMED: %s\n' "$artifact"
        else
            # `create_only` opens the file and then writes it, so a reader
            # between the two sees an empty file for a moment. Three reads a
            # second apart outlast any live writer. A file still empty after
            # that is a writer that died between open and write, and a claim
            # is never empty ([HW-PD-0004]): this claimant takes it over,
            # rather than waiting on `#unknown` forever.
            holder=$(sed -n 's/^issue //p' "$dir/claims/artifacts/$s" 2>/dev/null)
            tries=0
            while [ -z "$holder" ] && [ "$tries" -lt 3 ]; do
                sleep 1
                tries=$((tries + 1))
                holder=$(sed -n 's/^issue //p' "$dir/claims/artifacts/$s" 2>/dev/null)
            done
            if [ -z "$holder" ]; then
                printf '%s' "$content" > "$dir/claims/artifacts/$s"
                printf 'CLAIMED: %s (replaced an empty claim file)\n' "$artifact"
            elif [ "$holder" = "$issue" ]; then
                printf 'CLAIMED: %s (already held)\n' "$artifact"
            else
                printf 'HELD: %s by #%s\n' "$artifact" "$holder"
                case " $waits " in
                    *" $holder "*) ;;
                    *) waits="$waits $holder" ;;
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
    done
    # Every field this call does not name carries from the checkpoint before,
    # so a parent that vetoes a PASS names only the stage and the veto file.
    # The fields are written in the order `fields` declares.
    body=$(printf 'stage %s' "$reached")
    for key in $fields; do
        value=''
        for pair in "$@"; do
            [ "${pair%%=*}" = "$key" ] && value=${pair#*=}
        done
        [ -n "$value" ] || value=$(sed -n "s/^$key //p" "$dir/handover/$issue" 2>/dev/null | tail -n 1)
        [ -n "$value" ] && body=$(printf '%s\n%s %s' "$body" "$key" "$value")
    done
    # An adjudicated checkpoint is what the next session claims and iterates
    # from, so it is refused without the note and the footprint. A bare one
    # made `resume` print `claim ` with nothing to claim (verify-2 of #1281).
    if [ "$reached" = adjudicated ]; then
        for key in note footprint; do
            printf '%s\n' "$body" | grep -q "^$key " && continue
            echo "run-dir: an adjudicated checkpoint needs \`$key=\`, which the next session claims or iterates from." >&2
            exit 2
        done
    fi
    stale_check
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

# The last ruling of each issue in decisions.md that this run wrote, one
# `<issue> <kind>` per line. `start` seeds decisions.md from the previous run,
# so a RULED line names the run that wrote it and binds that run alone: a
# deferral is "for this run alone" (hw-run-policy), and a gate or a refusal is
# ruled again by the run that meets the issue again.
ruled_issues() {
    [ -f "$1/decisions.md" ] || return 0
    sed -n 's/^.* — RULED #\([0-9][0-9]*\) \(gated\|deferred\|refused\|open\) (run \([^)]*\)):.*/\1 \2 \3/p' "$1/decisions.md" \
        | awk -v run="$(basename "$1")" '$3 == run { last[$1] = $2 } END { for (i in last) print i, last[i] }'
}

# The issues an OWNER line in decisions.md answers, space-separated. Unlike a
# RULED line, an OWNER line from an earlier run still counts, on purpose: it
# is the owner's own answer, the product owner posts it on the issue, and it
# holds until the owner changes it. So it answers a `ruling` queue line in
# every later run.
#
# It answers every `#N` between `OWNER (<date>)` and the first colon, which is
# the form `decide --owner` writes. Run 20261012-1113 wrote
# `OWNER (2026-10-12) #990 #1275: "Build both"`, and a reader that took the
# first `#N` answered #990 alone. A `#N` after the colon is the owner's prose,
# not an answer: the same run's `OWNER (2026-10-12): ... Target set: #502 #504`
# answers nothing.
answered_issues() {
    [ -f "$1/decisions.md" ] || return 0
    awk 'match($0, /OWNER( [(][^)]*[)])?( #[0-9]+)+:/) {
        s = substr($0, RSTART, RLENGTH)
        sub(/^OWNER( [(][^)]*[)])?/, "", s)
        while (match(s, /#[0-9]+/)) {
            printf "%s ", substr(s, RSTART + 1, RLENGTH - 1)
            s = substr(s, RSTART + RLENGTH)
        }
    }' "$1/decisions.md"
}

# The next issue to dispatch: the first entry of queue.jsonl whose
# issue holds no claim, has no handover, is in no log line, and is not ruled
# out. It is ruled out when its last `RULED` line says gated, deferred or
# refused, or when its entry carries `ruling` and neither an `OWNER`
# line nor a `RULED ... open` line answers it (hw-run-policy: an unanswered
# ruling waits, and a deferral skips the issue for the run). An entry with a
# blocker has an open `blocked by` link on GitHub, and it waits until the
# parent writes `RULED ... open`. An OWNER line does not release it, because
# the owner answered a question and not the order. A RULED line
# binds only the run that wrote it, and an OWNER line binds every run. `DRAIN` when the
# run is draining, and `EMPTY` when nothing is left.
next() {
    dir=$1
    [ -d "$dir" ] || { echo "run-dir: $dir is not a run directory." >&2; exit 1; }
    stale_check
    if [ -e "$dir/drain" ]; then
        echo DRAIN
        return 0
    fi
    # `next` reads queue.jsonl, which only `queue` writes, and never the
    # rendered queue.md. A run whose queue.md was typed by hand, as every run
    # before #1724 was, is refused rather than read as empty.
    if [ ! -f "$dir/queue.jsonl" ]; then
        if [ -f "$dir/queue.md" ]; then
            echo "run-dir: $dir holds a queue.md and no queue.jsonl, and next reads only queue.jsonl, which \`run-dir.sh queue\` writes. Convert this run's queue once with \`sh tools/run/run-dir.sh convert ${dir##*/}\`." >&2
        else
            echo "run-dir: $dir holds no queue.jsonl. hw-queue writes it, one \`run-dir.sh queue\` call per issue." >&2
        fi
        exit 1
    fi
    need_jq
    logged=$(logged_issues "$dir")
    ruled=$(ruled_issues "$dir")
    answered=$(answered_issues "$dir")
    # `<issue> <ruling|-> <blocked|->`, from the fields of each entry. An entry
    # can carry both: in run 20261012-1113 a sed that took only the first mark
    # of a prose line read `ruling; blocked (#1721)` as `ruling`, and the
    # owner's answer would have released an issue whose blocker was open.
    if ! jq -r '"\(.issue) \(if .ruling then "ruling" else "-" end) \(if (.blocked | length) > 0 then "blocked" else "-" end)"' \
        "$dir/queue.jsonl" > "$dir/.next.$$" 2>/dev/null; then
        rm -f "$dir/.next.$$"
        echo "run-dir: $dir/queue.jsonl is not one JSON object per line, and only \`run-dir.sh queue\` writes it." >&2
        exit 1
    fi
    while read -r issue mark blocked; do
        [ -e "$dir/claims/issues/$issue" ] && continue
        [ -e "$dir/handover/$issue" ] && continue
        case " $logged " in *" $issue "*) continue ;; esac
        kind=$(printf '%s\n' "$ruled" | awk -v i="$issue" '$1 == i { print $2 }')
        case $kind in gated|deferred|refused) continue ;; esac
        if [ "$mark" = ruling ] && [ "$kind" != open ]; then
            case " $answered " in *" $issue "*) ;; *) continue ;; esac
        fi
        [ "$blocked" = blocked ] && [ "$kind" != open ] && continue
        rm -f "$dir/.next.$$"
        echo "$issue"
        return 0
    done < "$dir/.next.$$"
    rm -f "$dir/.next.$$"
    echo EMPTY
}

# A ruling the parent makes on one issue, appended to decisions.md in the one
# form `next` reads: `- <date> — RULED #<issue> <kind> (run <run-id>): <reason>`. The prose
# around it stays prose, and the last ruling of an issue is the one that holds.
rule() {
    dir=${1%/} issue=$2 kind=$3
    shift 3
    [ -d "$dir" ] || { echo "run-dir: $dir is not a run directory." >&2; exit 1; }
    case $issue in ''|*[!0-9]*) echo "run-dir: \`$issue\` is not an issue number." >&2; exit 2 ;; esac
    case $kind in
        gated|deferred|refused|open) ;;
        *) echo "run-dir: \`$kind\` is not a ruling. A ruling is one of: gated deferred refused open." >&2; exit 2 ;;
    esac
    reason="$*"
    [ -n "$reason" ] || { echo "run-dir: a ruling carries its reason." >&2; exit 2; }
    printf -- '- %s — RULED #%s %s (run %s): %s\n' "$(date -u +%Y-%m-%d)" "$issue" "$kind" "${dir##*/}" "$reason" >> "$dir/decisions.md"
    printf 'RULED: #%s %s\n' "$issue" "$kind"
}

# One value of a key=value argument: present, one line. The key is checked by
# the caller, which knows its own keys.
# A field that holds one value is refused when a call names it twice, rather
# than letting the last value win in silence. `seen` is reset by each caller.
once() {
    case " $seen " in *" $1 "*) refuse "\`$1=\` is given twice, and the field holds one value." ;; esac
    seen="$seen $1"
}

pair_value() {
    case $1 in *=?*) ;; *) refuse "\`$1\` is not key=value with a value." ;; esac
    case ${1#*=} in *"$nl"*) refuse "the value of \`${1%%=*}\` is more than one line." ;; esac
}

need_issue() {
    case $1 in ''|*[!0-9]*) refuse "\`$1\` is not $2 number." ;; esac
}

# The queue of a run, one object per issue in queue.jsonl, in call order,
# which is queue order: no entry stores its place ([HW-DR-0049]). hw-queue
# makes one call per issue. A field the entry needs and does not carry, a key
# or a mark nothing reads, a scope that is neither `wide` nor `narrow`, a
# blocker that is not an issue number, and an issue already queued are each
# refused with exit 2 and named, while the queue agent is still there to read
# the refusal. #1723 refused a malformed line only when `next` read it, after
# the queue agent had exited. `note=` holds anything that is not a mark, and
# `next` never reads it.
queue_fields='title milestone reader artifacts scope mark blocked note'

queue() {
    need_jq
    dir=$1 issue=$2
    shift 2
    need_dir "$dir"
    need_issue "$issue" 'an issue'
    # A queue.md with no queue.jsonl beside it was typed by hand, and the
    # render would overwrite it. `convert` keeps it and carries it in.
    if [ -f "$dir/queue.md" ] && [ ! -f "$dir/queue.jsonl" ]; then
        echo "run-dir: $dir holds a queue.md typed by hand, and the first \`queue\` call would overwrite it. Convert it first with \`sh tools/run/run-dir.sh convert ${dir##*/}\`." >&2
        exit 1
    fi
    entry=$(jq -cn --argjson n "$issue" '{issue: $n, ruling: false, blocked: [], notes: []}')
    seen=''
    for pair in "$@"; do
        pair_value "$pair"
        key=${pair%%=*} value=${pair#*=}
        case $key in blocked|note|mark) ;; *) once "$key" ;; esac
        case $key in
            title|milestone|reader|artifacts)
                # `|` separates the columns of the rendered queue.md.
                case $value in *'|'*) refuse "the value of \`$key\` holds \`|\`, which separates the columns of queue.md." ;; esac
                filter='.[$k] = $v' ;;
            scope)
                case $value in wide|narrow) ;; *) refuse "\`scope=$value\`: the scope is \`wide\` or \`narrow\`." ;; esac
                filter='.scope = $v' ;;
            mark)
                case $value in ruling) ;; *) refuse "\`mark=$value\` is not a mark. The one mark is \`ruling\`, and a blocker is \`blocked=<M>,<milestone>\`." ;; esac
                filter='.ruling = true' ;;
            blocked)
                blocker=${value%%,*}
                need_issue "$blocker" 'a blocking issue'
                milestone=''
                case $value in *,*) milestone=${value#*,} ;; esac
                value=$milestone
                filter='.blocked += [{issue: ($m | tonumber), milestone: $v}]' ;;
            note) filter='.notes += [$v]' ;;
            *) refuse "\`$key\` is not a queue field. A field is one of: $queue_fields." ;;
        esac
        entry=$(printf '%s' "$entry" | jq -c --arg k "$key" --arg v "$value" --arg m "${blocker:-0}" "$filter") || exit 1
    done
    for key in title milestone reader artifacts scope; do
        printf '%s' "$entry" | jq -e --arg k "$key" 'has($k)' >/dev/null ||
            refuse "the queue entry for #$issue needs \`$key=\`, and the call carries none."
    done
    if [ -s "$dir/queue.jsonl" ] && jq -se --argjson n "$issue" 'any(.[]; .issue == $n)' "$dir/queue.jsonl" >/dev/null; then
        refuse "#$issue is already queued in this run, and the queue holds one entry per issue."
    fi
    printf '%s\n' "$entry" >> "$dir/queue.jsonl"
    render_queue "$dir"
    printf 'QUEUED: #%s, entry %s\n' "$issue" "$(wc -l < "$dir/queue.jsonl" | tr -d ' ')"
}

# queue.md is for a person. Its line shape is the one hw-queue used to type
# by hand, so a reader of an older run sees no change. Nothing parses it.
render_queue() {
    {
        printf '# Queue, run %s\n\nRendered by `run-dir.sh queue` from queue.jsonl, which `next` reads. Do not edit this file.\n\n' "${1##*/}"
        jq -rs 'to_entries[] | .key as $k | .value |
            ([if .ruling then "ruling" else empty end]
             + [.blocked[] | if .milestone == "" then "blocked (#\(.issue))" else "blocked (#\(.issue), \(.milestone))" end]
             + .notes) as $marks |
            "\($k + 1). #\(.issue) \(.title) | \(.milestone) | \(.reader) | \(.artifacts) | \(.scope) | \($marks | join("; "))"' "$1/queue.jsonl"
    } > "$1/.queue.md.$$" && mv "$1/.queue.md.$$" "$1/queue.md"
}

# The one path from a run whose queue.md was typed by hand to one `next`
# reads. It parses queue.md once, with the parser `next` used before #1724,
# and writes each line through `queue`, so a line `queue` would refuse is
# refused here too, and nothing is written. The old file stays beside the new
# one as queue-legacy.md. Each `;`-separated mark becomes `mark=ruling`,
# `blocked=<M>,<milestone>`, or a `note=` that keeps its words.
convert() {
    need_jq
    dir=$1
    need_dir "$dir"
    [ -f "$dir/queue.jsonl" ] && { echo "run-dir: $dir already holds queue.jsonl, and a run converts its queue once." >&2; exit 1; }
    [ -f "$dir/queue.md" ] || { echo "run-dir: $dir holds no queue.md to convert." >&2; exit 1; }
    mv "$dir/queue.md" "$dir/queue-legacy.md"
    us=$(printf '\037')
    awk -F'|' -v us="$us" '/^[0-9]+\. #[0-9]+/ {
        match($1, /#[0-9]+/)
        out = substr($1, RSTART + 1, RLENGTH - 1)
        title = substr($1, RSTART + RLENGTH)
        gsub(/^[ \t]+|[ \t]+$/, "", title)
        out = out us title
        for (k = 2; k <= NF; k++) { f = $k; gsub(/^[ \t]+|[ \t]+$/, "", f); out = out us f }
        print out
    }' "$dir/queue-legacy.md" > "$dir/.convert.$$"
    converted=0
    set -f
    while IFS=$us read -r issue title milestone reader artifacts scope marks; do
        set -- "$issue"
        [ -n "$title" ] && set -- "$@" "title=$title"
        [ -n "$milestone" ] && set -- "$@" "milestone=$milestone"
        [ -n "$reader" ] && set -- "$@" "reader=$reader"
        [ -n "$artifacts" ] && set -- "$@" "artifacts=$artifacts"
        [ -n "$scope" ] && set -- "$@" "scope=$scope"
        rest=$marks
        while [ -n "$rest" ]; do
            mark=${rest%%;*}
            case $rest in *';'*) rest=${rest#*;} ;; *) rest='' ;; esac
            mark=$(printf '%s' "$mark" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')
            case $mark in
                '') ;;
                ruling) set -- "$@" mark=ruling ;;
                ruling[!a-z]*) set -- "$@" mark=ruling "note=$mark" ;;
                blocked|blocked[!a-z]*)
                    blocker=$(printf '%s' "$mark" | sed -n 's/^blocked *(#\([0-9][0-9]*\).*/\1/p')
                    on=$(printf '%s' "$mark" | sed -n 's/^blocked *(#[0-9][0-9]*, *\([^,)]*\).*/\1/p' | sed 's/[[:space:]]*$//')
                    set -- "$@" "blocked=${blocker:-?}${on:+,$on}"
                    # What a hand-typed blocker said past its milestone is kept as a note.
                    extra=$(printf '%s' "$mark" | sed -n 's/^blocked *(#[0-9][0-9]*, *[^,)]*, *\(.*\))$/\1/p')
                    if [ -n "$extra" ]; then
                        set -- "$@" "note=$extra"
                    elif [ "$mark" != "blocked (#$blocker${on:+, $on})" ]; then
                        set -- "$@" "note=$mark"
                    fi ;;
                *) set -- "$@" "note=$mark" ;;
            esac
        done
        if ! sh "$0" queue "$dir" "$@" >/dev/null 2>"$dir/.convert-err.$$"; then
            echo "run-dir: queue.md line $((converted + 1)), #$issue, is refused: $(sed 's/^run-dir: //' "$dir/.convert-err.$$")" >&2
            echo "run-dir: nothing was converted, and queue.md is as it was. Correct that line, or enter the queue with \`run-dir.sh queue\` by hand." >&2
            rm -f "$dir/queue.jsonl" "$dir/queue.md" "$dir/.convert.$$" "$dir/.convert-err.$$"
            mv "$dir/queue-legacy.md" "$dir/queue.md"
            exit 2
        fi
        converted=$((converted + 1))
    done < "$dir/.convert.$$"
    set +f
    rm -f "$dir/.convert.$$" "$dir/.convert-err.$$"
    [ -f "$dir/queue.jsonl" ] || { : > "$dir/queue.jsonl"; render_queue "$dir"; }
    printf 'CONVERTED: %s entries into queue.jsonl; the old file is queue-legacy.md\n' "$converted"
}

# A ruling the owner owes, one object per decision in rulings.jsonl, written
# by headwater-product-owner. rulings.md renders each in the block shape of
# that definition, with an `answered:` line derived from the OWNER lines of
# decisions.md, so no block is ever edited to mark it answered.
owe_fields='title question options unblocks'

owe() {
    need_jq
    dir=$1 issue=$2
    shift 2
    need_dir "$dir"
    need_issue "$issue" 'an issue'
    entry=$(jq -cn --argjson n "$issue" '{issue: $n}')
    seen=''
    for pair in "$@"; do
        pair_value "$pair"
        key=${pair%%=*} value=${pair#*=}
        once "$key"
        case $key in
            title|question|unblocks) filter='.[$k] = $v' ;;
            options)
                filter='.options = ($v | split("|") | map(gsub("^\\s+|\\s+$"; "")))'
                printf '%s' "$value" | jq -Re 'split("|") | map(gsub("^\\s+|\\s+$"; "")) | length >= 2 and all(. != "") and .[-1] == "defer"' >/dev/null ||
                    refuse "\`options=$value\`: a ruling offers at least two options separated by \`|\`, none empty, and the last is \`defer\`." ;;
            *) refuse "\`$key\` is not a ruling field. A field is one of: $owe_fields." ;;
        esac
        entry=$(printf '%s' "$entry" | jq -c --arg k "$key" --arg v "$value" "$filter") || exit 1
    done
    for key in $owe_fields; do
        printf '%s' "$entry" | jq -e --arg k "$key" 'has($k)' >/dev/null ||
            refuse "the ruling for #$issue needs \`$key=\`, and the call carries none."
    done
    # A rulings.md with no rulings.jsonl beside it was written by hand before
    # #1724. Its blocks are prose for the owner, so it is kept whole as
    # rulings-legacy.md rather than parsed, and the render does not overwrite it.
    if [ -f "$dir/rulings.md" ] && [ ! -f "$dir/rulings.jsonl" ]; then
        mv "$dir/rulings.md" "$dir/rulings-legacy.md"
        echo "run-dir: the hand-written rulings.md is kept as rulings-legacy.md." >&2
    fi
    printf '%s\n' "$entry" >> "$dir/rulings.jsonl"
    render_rulings "$dir"
    printf 'OWED: #%s, ruling %s\n' "$issue" "$(wc -l < "$dir/rulings.jsonl" | tr -d ' ')"
}

render_rulings() {
    [ -f "$1/rulings.jsonl" ] || return 0
    need_jq
    jq -r --arg answered " $(answered_issues "$1")" '
        .issue as $i |
        "RULING #\(.issue) \(.title)\n  question: \(.question)\n  options:  \(.options | join(" | "))\n  unblocks: \(.unblocks)\n  answered: \(if ($answered | contains(" \($i) ")) then "yes, an OWNER line in decisions.md" else "no" end)\n"' \
        "$1/rulings.jsonl" > "$1/.rulings.md.$$" && mv "$1/.rulings.md.$$" "$1/rulings.md"
}

rulings() {
    need_dir "$1"
    [ -f "$1/rulings.jsonl" ] || { echo "run-dir: $1 holds no rulings.jsonl; the product owner writes it with \`run-dir.sh owe\`." >&2; exit 1; }
    render_rulings "$1"
    cat "$1/rulings.md"
}

# The integrator queue: one object per merge ruling, appended by the parent,
# and never rewritten. The pull request keys an entry, because a ruled pull
# request can have no issue (PR #1723 had none). A second ruling of one pull
# request at a new head is appended, because an ejected pull request returns
# for a new ruling at a new commit, and the last entry for a pull request is
# the one that holds. The same ruling of the same pull request at the same
# commit is refused: that is the parent appending one ruling twice. An
# ejection is recorded as one more entry, `ruling=ejected`, which takes the
# pull request off `integrations` until the parent rules it again.
integrate_fields='issue branch sha ruling footprint'

integrate() {
    need_jq
    dir=$1 pr=$2
    shift 2
    need_dir "$dir"
    need_issue "$pr" 'a pull request'
    entry=$(jq -cn --argjson n "$pr" '{pr: $n}')
    seen=''
    for pair in "$@"; do
        pair_value "$pair"
        key=${pair%%=*} value=${pair#*=}
        once "$key"
        case $key in
            issue)
                [ "$value" = none ] || need_issue "$value" 'an issue'
                filter='.issue = (if $v == "none" then null else ($v | tonumber) end)' ;;
            sha)
                case $value in *[!0-9a-f]*) refuse "\`sha=$value\` is not a commit: seven to forty lower-case hex digits." ;; esac
                [ ${#value} -ge 7 ] && [ ${#value} -le 40 ] || refuse "\`sha=$value\` is not a commit: seven to forty lower-case hex digits."
                filter='.sha = $v' ;;
            branch|footprint)
                case $value in *[[:space:]]*) refuse "the value of \`$key\` holds whitespace, and it is one word." ;; esac
                filter='.[$k] = $v' ;;
            ruling) filter='.ruling = $v' ;;
            *) refuse "\`$key\` is not an integrator field. A field is one of: $integrate_fields." ;;
        esac
        entry=$(printf '%s' "$entry" | jq -c --arg k "$key" --arg v "$value" "$filter") || exit 1
    done
    # Without `sha=`, the head is read from GitHub, the one field, so the
    # parent types the number it ruled on and never a commit it copied from a
    # report. A head that cannot be read refuses the ruling rather than
    # recording one at no commit.
    if ! printf '%s' "$entry" | jq -e 'has("sha")' >/dev/null; then
        head=$(gh api "repos/headwater-ai/headwater/pulls/$pr" --jq .head.sha 2>/dev/null)
        case $head in
            ''|*[!0-9a-f]*) refuse "the head of PR #$pr could not be read from GitHub (\`$head\`); pass \`sha=\` or retry." ;;
        esac
        entry=$(printf '%s' "$entry" | jq -c --arg v "$head" '.sha = $v') || exit 1
    fi
    for key in $integrate_fields; do
        printf '%s' "$entry" | jq -e --arg k "$key" 'has($k)' >/dev/null ||
            refuse "the merge ruling for PR #$pr needs \`$key=\`, and the call carries none."
    done
    sha=$(printf '%s' "$entry" | jq -r .sha)
    if [ -s "$dir/integrate.jsonl" ] && jq -se --argjson n "$pr" --arg s "$sha" --arg r "$(printf '%s' "$entry" | jq -r .ruling)" 'any(.[]; .pr == $n and .sha == $s and .ruling == $r)' "$dir/integrate.jsonl" >/dev/null; then
        refuse "PR #$pr is already queued at $sha with this ruling. A new ruling of it is appended at its new head."
    fi
    printf '%s\n' "$entry" >> "$dir/integrate.jsonl"
    printf 'INTEGRATE: PR #%s queued, entry %s\n' "$pr" "$(wc -l < "$dir/integrate.jsonl" | tr -d ' ')"
}

# Each pull request of integrate.jsonl that no log line records and whose
# last ruling is not `ejected`, in the order it was first ruled, with its
# last ruling. The integrator merges these.
integrations() {
    need_jq
    need_dir "$1"
    [ -s "$1/integrate.jsonl" ] || return 0
    merged=$( [ -s "$1/log.jsonl" ] && jq -cs '[.[].pr | numbers]' "$1/log.jsonl" 2>/dev/null)
    jq -rs --argjson merged "${merged:-[]}" '
        reduce .[] as $e ({order: [], last: {}};
            (if .last[$e.pr | tostring] then . else .order += [$e.pr] end) | .last[$e.pr | tostring] = $e)
        | . as $s | $s.order[] | select(. as $p | $merged | any(.[]; . == $p) | not) | $s.last[tostring] | select(.ruling != "ejected")
        | "PR #\(.pr) issue=\(.issue // "none") branch=\(.branch) sha=\(.sha) footprint=\(.footprint) ruling=\(.ruling)"' "$1/integrate.jsonl"
}

# Whether integrate.jsonl already holds a ruling for this issue or this pull
# request, so a resumed parent is not told to append it twice.
#
# It reads the last entry for the pull request, or for the issue where the
# handover names no pull request. One whose last ruling is `ejected` is not
# held: the parent owes a new ruling at the new head, and a resumed parent
# told `queued` would never append it, so the pull request would never merge
# (verify-1 of PR #1726).
integrate_holds() {
    [ -s "$1/integrate.jsonl" ] || return 1
    need_jq
    jq -se --argjson i "$2" --arg p "${3:-}" '
        [.[] | select(if $p != "" then .pr == ($p | tonumber) else .issue == $i end)] | last
        | . != null and .ruling != "ejected"' "$1/integrate.jsonl" >/dev/null 2>&1
}

# What a resumed parent does with each open handover, one line per issue:
# `<issue> <stage> <action>`. An open handover is one the log does not record.
#
#     adjudicated, no claim   claim <artifacts>   claim the footprint, then dispatch hw-iterate
#     adjudicated, claimed    iterate             dispatch hw-iterate with the note
#     built, verified-fail    iterate             a fresh hw-iterate given the handover path
#     verified-pass           rule                the parent rules, from the notes it names
#     ruled-merge, not queued integrate           append to the integrator queue with `integrate`
#     ruled-merge, queued     queued              nothing: integrate.jsonl already holds its PR or issue
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
                footprint=$(sed -n 's/^footprint //p' "$h")
                if [ -e "$dir/claims/issues/$issue" ] || [ "$footprint" = none ]; then
                    action=iterate
                else
                    action="claim $(printf '%s' "$footprint" | tr ',' ' ')"
                fi ;;
            built|verified-fail) action=iterate ;;
            verified-pass) action=rule ;;
            ruled-merge)
                if integrate_holds "$dir" "$issue" "$(sed -n 's/^pr //p' "$h")"; then
                    action=queued
                else
                    action=integrate
                fi ;;
            *) action=unknown ;;
        esac
        printf '%s %s %s\n' "$issue" "$reached" "$action"
    done
}

# The dispatch of one stage, composed into the scratch directory and passed
# to the Agent tool as a path. The template is stated in prose in
# `.claude/commands/next-run.md`, and this verb is its one renderer, so a
# field the parent would have typed by hand into each dispatch is typed
# nowhere. `tools/run/run-dir-fixtures.sh` holds the rendered lines to the
# fields the command names. A stage that lacks a field it needs is refused
# before anything is written, and so is a field no stage reads.
dispatch_stages='adjudicate iterate build verify integrate'
dispatch_fields='title branch note handover pr build waits-on attacks deadline'

scratch_root() {
    if [ -n "${HEADWATER_SCRATCH:-}" ]; then
        printf '%s' "$HEADWATER_SCRATCH"
    elif [ -n "${CLAUDE_JOB_DIR:-}" ]; then
        printf '%s/tmp' "$CLAUDE_JOB_DIR"
    else
        refuse "no CLAUDE_JOB_DIR and no HEADWATER_SCRATCH, so there is no scratch directory to compose a dispatch in."
    fi
}

# dispatch_line <key> <value>: one line of the template, the key padded as
# the template pads it. Nothing is printed for an empty value.
dispatch_line() {
    [ -n "$2" ] || return 0
    printf '%-14s%s\n' "$1:" "$2"
}

dispatch() {
    dir=$1 issue=$2 stage=$3
    shift 3
    need_dir "$dir"
    case " $dispatch_stages " in
        *" $stage "*) ;;
        *) refuse "\`$stage\` is not a stage to dispatch. A stage is one of: $dispatch_stages." ;;
    esac
    [ "$issue" = none ] || need_issue "$issue" 'an issue'
    runid=${dir##*/}
    title='' branch='' note='' handover='' pr='' build='' waits='' attacks='' deadline=''
    seen=''
    for pair in "$@"; do
        pair_value "$pair"
        key=${pair%%=*} value=${pair#*=}
        once "$key"
        case $key in
            title) title=$value ;;
            branch) branch=$value ;;
            note) note=$value ;;
            handover) handover=$value ;;
            pr) need_issue "$value" 'a pull request'; pr=$value ;;
            build) build=$value ;;
            waits-on) waits=$value ;;
            attacks) attacks=$value ;;
            deadline) need_issue "$value" 'a number of minutes'; deadline=$value ;;
            *) refuse "\`$key\` is not a dispatch field. A field is one of: $dispatch_fields." ;;
        esac
    done
    root_scratch=$(scratch_root) || exit 2
    integrations_lines=''
    case $stage in
        adjudicate)
            [ "$issue" != none ] || refuse "an adjudicate dispatch names an issue."
            scratch="$root_scratch/issue-$issue" ;;
        iterate|build)
            if [ "$issue" = none ]; then
                [ -n "$branch" ] || branch="run-lessons-$runid"
                [ -n "$note" ] || note="the run id, $runid: \`sh tools/run/run-dir.sh applies $runid\` is the note"
                scratch="$root_scratch/run-lessons-$runid"
            else
                [ -n "$branch" ] || refuse "a $stage dispatch needs \`branch=\`."
                [ -n "$note$handover" ] || refuse "a $stage dispatch needs \`note=\` (the adjudication) or \`handover=\` (a checkpoint)."
                scratch="$root_scratch/issue-$issue"
            fi
            [ -n "$waits" ] || waits=none ;;
        verify)
            [ -n "$branch" ] && [ -n "$pr" ] || refuse "a verify dispatch needs \`branch=\` and \`pr=\`."
            if [ "$issue" = none ]; then scratch="$root_scratch/run-lessons-$runid"; else scratch="$root_scratch/issue-$issue"; fi ;;
        integrate)
            [ "$issue" = none ] || refuse "an integrate dispatch takes \`none\` for the issue and reads the integrator queue."
            integrations_lines=$(integrations "$dir")
            [ -n "$integrations_lines" ] || refuse "the integrator queue holds no ruling to dispatch: \`run-dir.sh integrations\` prints nothing."
            scratch="$root_scratch/integrate-$(date -u +%H%M%S)" ;;
    esac
    if [ "$issue" != none ] && [ -z "$title" ] && [ -s "$dir/queue.jsonl" ] && command -v jq >/dev/null 2>&1; then
        title=$(jq -r --argjson n "$issue" 'select(.issue == $n) | .title' "$dir/queue.jsonl" 2>/dev/null | head -n 1)
    fi
    case $issue in
        none) if [ "$stage" = integrate ]; then issue_line="none (the integrator queue)"; else issue_line="none (run lessons $runid)"; fi ;;
        *) issue_line="#$issue${title:+ $title}" ;;
    esac
    mkdir -p "$scratch" || exit 1
    out="$scratch/dispatch-$stage.md"
    {
        dispatch_line issue "$issue_line"
        dispatch_line run "$dir"
        dispatch_line scratch "$scratch/"
        dispatch_line branch "$branch"
        [ -n "$pr" ] && dispatch_line 'pull request' "#$pr"
        if [ -n "$integrations_lines" ]; then
            printf '%s\n' "$integrations_lines" | while IFS= read -r line; do
                dispatch_line 'pull request' "$line"
            done
        fi
        dispatch_line 'waits-on' "$waits"
        dispatch_line note "$note"
        dispatch_line handover "$handover"
        dispatch_line 'build note' "$build"
        if [ "$stage" = verify ]; then
            if [ "$issue" = none ]; then dispatch_line slot "HW_CARGO_SLOT=verify-lessons"; else dispatch_line slot "HW_CARGO_SLOT=verify-$issue"; fi
        fi
        dispatch_line attacks "$attacks"
        [ -n "$deadline" ] && dispatch_line deadline "$deadline minutes"
        dispatch_line report 'the fixed block your definition names and nothing before it'
    } > "$out.$$" && mv "$out.$$" "$out" || { rm -f "$out.$$"; exit 1; }
    printf 'DISPATCH: %s\n' "$out"
}

# What the parent does with a `VERDICT: BUILD` report, in one call: the
# `adjudicated` checkpoint, the claim on the footprint, and the hw-iterate
# dispatch carrying whatever `WAITS-ON` the claim printed. Three calls that
# every BUILD report cost the parent, and the footprint typed once instead
# of twice, in two shapes. In drain the checkpoint is written and nothing
# else: `claim` writes nothing then, and the next session claims from the
# checkpoint, as `resume` says.
accept() {
    dir=$1 issue=$2
    shift 2
    need_dir "$dir"
    need_issue "$issue" 'an issue'
    branch='' note='' footprint='' title='' attacks='' deadline=''
    seen=''
    for pair in "$@"; do
        pair_value "$pair"
        key=${pair%%=*} value=${pair#*=}
        once "$key"
        case $key in
            branch) branch=$value ;;
            note) note=$value ;;
            footprint) footprint=$value ;;
            title) title=$value ;;
            attacks) attacks=$value ;;
            deadline) deadline=$value ;;
            *) refuse "\`$key\` is not an accept field. A field is one of: branch note footprint title attacks deadline." ;;
        esac
    done
    [ -n "$branch" ] && [ -n "$note" ] && [ -n "$footprint" ] ||
        refuse "accept needs \`branch=\`, \`note=\` and \`footprint=\`, from the BUILD report."
    staged=$(stage "$dir" "$issue" adjudicated "note=$note" "branch=$branch" "footprint=$footprint") || exit $?
    # The STALE line is printed once, from the stage, and the claim after it
    # is silenced, so its exact match on DRAIN below still reads only its own
    # output.
    stale=$(printf '%s\n' "$staged" | grep '^STALE: ')
    if [ -n "$stale" ]; then
        printf '%s\n' "$stale"
        stale_said=1
    fi
    # A whole line, because the STALE line carries the checkout's path, and a
    # path that holds the word must not read as the drain.
    if printf '%s\n' "$staged" | grep -qx DRAIN; then
        echo DRAIN
        return 0
    fi
    waits=none
    if [ "$footprint" != none ]; then
        set -f
        oldifs=$IFS
        IFS=,
        set -- $footprint
        IFS=$oldifs
        set +f
        claimed=$(claim "$dir" "$issue" "$branch" "$@") || exit $?
        printf '%s\n' "$claimed"
        case $claimed in
            DRAIN) return 0 ;;
        esac
        held=$(printf '%s\n' "$claimed" | sed -n 's/^WAITS-ON://p' | head -n 1)
        [ -z "$held" ] || waits=$(printf '%s' "$held" | sed 's/ \([0-9]\)/ #\1/g; s/^ //')
    fi
    set -- "branch=$branch" "note=$note" "waits-on=$waits"
    [ -z "$title" ] || set -- "$@" "title=$title"
    [ -z "$attacks" ] || set -- "$@" "attacks=$attacks"
    [ -z "$deadline" ] || set -- "$@" "deadline=$deadline"
    dispatch "$dir" "$issue" iterate "$@"
}

# The rulings the owner owes, written as one file of blocks in the shape
# the product owner's definition states, and recorded through `owe` one
# block at a time. The product owner composes the file with `Write`, which
# is how a long quoted field reaches the ledger without a shell argument
# that worktree isolation refuses. Every block is checked before the first
# is written, because `owe` appends and nothing rewrites: a file with one
# bad block records nothing.
po_rule() {
    need_jq
    dir=$1 file=$2
    need_dir "$dir"
    [ -s "$file" ] || { echo "run-dir: $file is not a file of RULING blocks." >&2; exit 1; }
    us=$(printf '\037')
    parsed=$(awk -v us="$us" '
        function flush() { if (issue != "") printf "%s%s%s%s%s%s%s%s%s\n", issue, us, title, us, question, us, options, us, unblocks; issue = "" }
        function trim(s) { sub(/^[ \t]+/, "", s); sub(/[ \t]+$/, "", s); return s }
        /^RULING #[0-9]+/ { flush(); match($0, /#[0-9]+/); issue = substr($0, RSTART + 1, RLENGTH - 1); title = trim(substr($0, RSTART + RLENGTH)); question = options = unblocks = ""; next }
        /^[ \t]*question:/ { sub(/^[ \t]*question:/, ""); question = trim($0); next }
        /^[ \t]*options:/ { sub(/^[ \t]*options:/, ""); options = trim($0); next }
        /^[ \t]*unblocks:/ { sub(/^[ \t]*unblocks:/, ""); unblocks = trim($0); next }
        /^[ \t]*answered:/ { next }
        /^[ \t]*$/ { next }
        { printf "run-dir: line %d of the rulings file is not a RULING line or one of its fields: %s\n", NR, $0 > "/dev/stderr"; bad = 1 }
        END { flush(); if (bad) exit 2 }
    ' "$file") || exit 2
    [ -n "$parsed" ] || refuse "$file holds no \`RULING #<N> <title>\` block."
    n=0
    printf '%s\n' "$parsed" | while IFS=$us read -r issue title question options unblocks; do
        n=$((n + 1))
        for field in title question options unblocks; do
            eval "value=\$$field"
            [ -n "$value" ] || refuse "RULING #$issue, block $n, lacks \`$field:\`, and nothing was recorded."
        done
        printf '%s' "$options" | jq -Re 'split("|") | map(gsub("^\\s+|\\s+$"; "")) | length >= 2 and all(. != "") and .[-1] == "defer"' >/dev/null ||
            refuse "RULING #$issue, block $n: the options are at least two, separated by \`|\`, none empty, and the last is \`defer\`; nothing was recorded."
    done || exit 2
    printf '%s\n' "$parsed" | while IFS=$us read -r issue title question options unblocks; do
        owe "$dir" "$issue" "title=$title" "question=$question" "options=$options" "unblocks=$unblocks" || exit 2
    done || exit 2
    printf 'PO-RULE: %s rulings recorded from %s\n' "$(printf '%s\n' "$parsed" | wc -l | tr -d ' ')" "$file"
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

run=${2:+$(resolve "$2")}

case ${1:-} in
    start) shift; start "$@" ;;
    stage) [ $# -ge 4 ] || usage; shift 2; stage "$run" "$@" ;;
    next) [ $# -eq 2 ] || usage; next "$run" ;;
    resume) [ $# -eq 2 ] || usage; resume "$run" ;;
    rule) [ $# -ge 5 ] || usage; shift 2; rule "$run" "$@" ;;
    session) [ $# -eq 2 ] || usage; session "$run" ;;
    claim) [ $# -ge 5 ] || usage; shift 2; claim "$run" "$@" ;;
    release) [ $# -eq 3 ] || usage; release "$run" "$3" ;;
    claims) [ $# -eq 2 ] || usage; claims "$run" ;;
    end) [ $# -eq 2 ] || usage; end "$run" ;;
    log) [ $# -eq 3 ] || usage; log "$run" "$3" ;;
    tail) [ $# -ge 2 ] || usage; tail_log "$run" "${3:-5}" ;;
    net) [ $# -eq 2 ] || usage; net "$run" ;;
    usage) [ $# -eq 2 ] || usage; plan_usage "$run" ;;
    import) [ $# -eq 3 ] || usage; import "$2" "$(resolve "$3")" ;;
    intake) [ $# -eq 3 ] || usage; intake "$run" "$3" ;;
    decide) [ $# -ge 3 ] || usage; shift 2; decide "$run" "$@" ;;
    queue) [ $# -ge 3 ] || usage; shift 2; queue "$run" "$@" ;;
    convert) [ $# -eq 2 ] || usage; convert "$run" ;;
    owe) [ $# -ge 3 ] || usage; shift 2; owe "$run" "$@" ;;
    rulings) [ $# -eq 2 ] || usage; rulings "$run" ;;
    integrate) [ $# -ge 3 ] || usage; shift 2; integrate "$run" "$@" ;;
    integrations) [ $# -eq 2 ] || usage; integrations "$run" ;;
    lesson) [ $# -eq 3 ] || usage; lesson "$run" "$3" ;;
    withdraw) [ $# -eq 4 ] || usage; withdraw "$run" "$3" "$4" ;;
    applies) [ $# -eq 2 ] || usage; applies "$run" ;;
    dispatch) [ $# -ge 4 ] || usage; shift 2; dispatch "$run" "$@" ;;
    accept) [ $# -ge 4 ] || usage; shift 2; accept "$run" "$@" ;;
    po-rule) [ $# -eq 3 ] || usage; po_rule "$run" "$3" ;;
    *) usage ;;
esac
