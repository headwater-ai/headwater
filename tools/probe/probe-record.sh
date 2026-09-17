#!/bin/sh
# The driver half of a recorder: it starts one probe session and reads what the
# harness emits.
#
# [HW-DR-0059] admits a recorder built as a driver and a transform, on one
# condition, and this script is where that condition is either met or lost.
# [Spec 15] §"A recorder is a driver and a transform" states it: **the log has
# to reach the transform through a channel the model has no handle on.**
#
#     sh tools/probe/probe-record.sh --probe PROBE-X --session one \
#         --task-file task.md --model claude-haiku-4-5 \
#         --workspace /tmp/scratch-copy [--raw raw.jsonl] [--produced docs/x.md]…
#
# Three modes read nothing from the network. `--identity-only` prints the six
# members `headwater probe plan` fixes, `--provider-only <log>` prints the
# three the provider metadata carries, and `--answer-only <log> --answers
# <set>` prints the final answer, each over a log already recorded.
#
# ## The channel, and the two ways to lose it
#
# The channel is the standard output of the harness process:
#
#     claude -p --output-format stream-json --verbose
#
# The session holds no handle on its parent's pipe, so no `Bash` call it makes
# can write a line of what this script reads. `--verbose` is required with
# `-p --output-format stream-json`; without it the harness refuses.
#
# **Never a file under `~/.claude/projects/`.** On the host that runs this
# repository every process of one user can write the live log of a session, and
# a `Bash` call of the session under test is one of those processes. A recorder
# that reads the log from disk states nothing about who wrote a line of it, and
# the transcript it produces is a self-report that no key set can detect.
#
# The second way to lose it is subtler: run the session in the checkout the
# plan was taken against. The `answered` probe's task asks the session to
# produce an artifact, so a session that writes into the corpus moves the
# `tree` digest the plan just fixed. `--workspace` is required for that reason
# and this script refuses to run a session whose working directory is the
# corpus it was planned over.
#
# A workspace built by `cp -a` of a worktree checkout carries the worktree's
# own `.git` file along with it — a pointer of the form `gitdir:
# /path/to/repo/.git/worktrees/<name>` — and neither the workspace guard above
# nor anything downstream strips it. A probed session that finds that pointer
# has a live handle on this repository's real history and branch list from
# inside what the guard treats as fully isolated: it can `git commit` there,
# and with `EnterWorktree` it can register a real worktree and branch against
# the actual repository. Two sessions of the first real eight-probe recording
# did exactly that, caught by reading `git log` and `git worktree list` after
# the run rather than by anything this script refused ([#897]). So this script
# refuses a workspace that carries a `.git` file or directory the same way it
# refuses one inside the corpus: before the session starts, not after.
#
# ## What this half derives, and what the transform derives
#
# [Spec 15] §"The values a session log omits" splits the derivations between
# the two halves. The transform owns the four that read an artifact, and this
# script owns the two that need the plan and the provider metadata:
#
#   step_copy_plan_identity   `lock`, `tree`, `selection`, `read_set`, `seed`
#                             and `harness`, copied from `headwater probe plan`
#                             without change
#   step_derive_provider      `model`, `served_version` and `cost_cents`, read
#                             from the harness `result` line, because a usage
#                             record carries token counts and not cents
#   step_derive_answer        `answer`, read from the same `result` line, for a
#                             probe that expects one and for no other
#
# ## Why `answer` is read here and why it is read so narrowly
#
# [Spec 15] rules that `answer: null` and an absent `answer` key are two facts:
# `null` says the recorder watched and saw no final answer. This script wrote
# `null` for every session it ever drove, because it called the transform with
# no `--answer` and the transform defaults it empty. One session of 2026-09-11
# ended with the single word `present` and the transcript said it ended with
# nothing, so a quarter of this corpus's only efficacy figure graded the
# recorder rather than the corpus ([#803]).
#
# The derivation is deliberately narrow in two directions, and both are the
# same rule: **a transcript holds no model prose.** It runs only for a probe
# whose expectation is `answered`, because the final text of an `opened`
# session is prose. It writes a value only where the whole trimmed final text
# is one answer the probe declares, because anything else is prose too. A
# session that argues its way to `present` over a paragraph is a session that
# gave no answer in the closed set, and `null` is the true record of it.
#
# The remaining three identity members — `tier`, `arm` and `at` — are stated by
# the caller and the clock, and `--tier`/`--arm` default to what the plan says.
#
# It writes only what `--raw` and standard output name, it needs `jq` and the
# `claude` harness, and it spends real money.

set -u

probe=
session=
task_file=
model=
workspace=
raw=
tier=
arm=
identity_only=0
provider_only=0
answer_only=0
answers=
transform_args=

while [ $# -gt 0 ]; do
    case $1 in
        --probe) probe=${2:-}; shift 2 ;;
        --session) session=${2:-}; shift 2 ;;
        --task-file) task_file=${2:-}; shift 2 ;;
        --model) model=${2:-}; shift 2 ;;
        --workspace) workspace=${2:-}; shift 2 ;;
        --raw) raw=${2:-}; shift 2 ;;
        --tier) tier=${2:-}; shift 2 ;;
        --arm) arm=${2:-}; shift 2 ;;
        --identity-only) identity_only=1; shift ;;
        --provider-only) provider_only=1; raw=${2:-}; shift 2 ;;
        --answer-only) answer_only=1; raw=${2:-}; shift 2 ;;
        --answers) answers=${2:-}; shift 2 ;;
        --produced) transform_args="$transform_args --produced ${2:-}"; shift 2 ;;
        *) echo "probe-record: unknown argument \`$1\`" >&2; exit 2 ;;
    esac
done

# `cd` and `pwd` are builtins and `dirname` is not. Resolving the root with a
# parameter expansion means every refusal below reaches its own message on a
# host with nothing on `PATH` at all, which is the state a guard is worth
# having in.
case $0 in
    */*) invoked_from=${0%/*} ;;
    *) invoked_from=. ;;
esac
root=$(cd "$invoked_from/../.." && pwd)
engine=$root/engine/target/dev-release/headwater
[ -x "$engine" ] || engine=$root/engine/target/release/headwater

# The arguments and the workspace guard are checked **before anything about
# this host**, because they are the two refusals that are the same everywhere.
# The order was the other way round and it cost a red CI run: on a runner with
# no `claude` on the path the script exited 3 for the missing harness before it
# ever read the workspace, and the case asserting the guard chose its expected
# status from `[ -x "$engine" ]` — a fact about the host, and the wrong one.
# A guard that only fires where the tools happen to be installed is a guard
# that is absent on the machine most likely to need it.
if [ "$identity_only" = 0 ] && [ "$provider_only" = 0 ] && [ "$answer_only" = 0 ]; then
    [ -n "$probe" ] && [ -n "$session" ] && [ -n "$task_file" ] && [ -n "$workspace" ] || {
        echo "usage: probe-record.sh --probe <id> --session <name> --task-file <f> --workspace <dir> [--model <m>]" >&2
        exit 2
    }
    [ -f "$task_file" ] || { echo "probe-record: no task file at $task_file" >&2; exit 2; }
    here=$(cd "$workspace" 2>/dev/null && pwd) || {
        echo "probe-record: no workspace directory at $workspace" >&2
        exit 2
    }
    # A session that runs in the corpus the plan was taken over moves the tree
    # digest that plan just fixed.
    case "$here" in
        "$root"|"$root"/*)
            echo "probe-record: the workspace is inside the corpus the plan was taken over." >&2
            echo "probe-record: a session that writes there moves the \`tree\` digest of the identity. Use a copy." >&2
            exit 6
            ;;
    esac
    # A `cp -a` of a worktree copies its `.git` file (or a full clone's `.git`
    # directory) along with the tree. Either one is a live pointer into this
    # repository's real history, and a probed session that finds it has a
    # handle the workspace guard above was supposed to deny it.
    if [ -e "$here/.git" ]; then
        echo "probe-record: the workspace at $here carries a \`.git\` file or directory." >&2
        echo "probe-record: that is a live pointer into this repository's history, most likely left by \`cp -a\` of a worktree. Strip \`.git\` from the copy before recording." >&2
        exit 4
    fi
fi

command -v jq >/dev/null 2>&1 || {
    echo "probe-record: \`jq\` is not on the path." >&2
    exit 3
}

# step_derive_provider. The harness `result` line carries the model, the usage
# record and the realized cost in dollars; the contract wants whole cents.
#
# **A model name is not a pin.** Measured on 2026-09-08: one `claude-haiku-4-5`
# session wrote `modelUsage` keyed by both `claude-haiku-4-5` and
# `claude-haiku-4-5-20251001`, and the first key of that object is the alias.
# Taking it would write an alias into `served_version`, which is the one member
# of the identity that says which weights answered. So the served version is
# the key that carries a dated suffix where the provider exposes one, and the
# name as a name where none is exposed, which is what [spec 15] asks for.
#
# **A dated key is not this session's dated key.** Measured on 2026-09-17,
# driving `claude-sonnet-5`: `modelUsage` also carried `claude-haiku-4-5-20251001`
# for a small internal call the harness makes regardless of the driven model,
# and that entry's key is the only one with a dated suffix. The first version of
# this derivation took the first dated key of the whole object, so it wrote
# Haiku's pin as the served version of a session that spent 94% of its cost on
# Sonnet. The dated key has to belong to the driven model, which `canonicalModel`
# on the `modelUsage` entry states, so the search below narrows to entries whose
# `canonicalModel` is the name the init line announced before it looks for a
# date on any of them.
step_derive_provider() {
    jq -s -r '
        ([.[] | select(.type == "result")] | last) as $r
        | ([.[] | select(.type == "system" and .subtype == "init")] | last) as $i
        | ($r.modelUsage // {}) as $usage
        | ($usage | keys) as $used
        | ($i.model // ($used | first) // "unknown") as $name
        | ([$used[] | select($usage[.].canonicalModel == $name and test("-[0-9]{8}$"))] | first) as $own_dated
        | {
            model: $name,
            served: ($own_dated // $name),
            cost_cents: (((($r.total_cost_usd // 0) * 100) | round)),
          }
        | "model: \(.model)\nserved_version: \(.served)\ncost_cents: \(.cost_cents)"
    ' < "$raw"
}

# `--provider-only <log>` runs that step alone over a recorded log, so the
# derivation is held by a fixture without spending a session or an engine.
if [ "$provider_only" = 1 ]; then
    [ -f "$raw" ] || { echo "probe-record: no log at $raw" >&2; exit 2; }
    step_derive_provider
    exit 0
fi

# step_derive_answer. The final text of the harness `result` line, and a value
# only where that whole text is one of the answers the probe declares. It
# prints nothing otherwise, and the caller then passes no `--answer`, which is
# the `null` the contract asks for.
#
# The comparison folds case and trims surrounding space, because a harness that
# ends a session with `Present.` has said the same word. It does not strip a
# trailing period or match a substring: `the answer is present` is prose that
# contains an answer, and a recorder that took the word out of it would be
# reading the session rather than observing it.
step_derive_answer() {
    [ -n "$answers" ] || return 0
    said=$(jq -s -r '([.[] | select(.type == "result")] | last | .result // "")' < "$raw" \
        | tr -d '\r' | sed 's/^[[:space:]]*//; s/[[:space:]]*$//; s/\.$//')
    [ -n "$said" ] || return 0
    folded=$(printf '%s' "$said" | tr '[:upper:]' '[:lower:]')
    # `printf '%s\n'` and never `printf '%s'`: a set of one answer carries no
    # comma, so the unterminated form gives `read` a line with no newline, and
    # `while read` stops before the body on that. The fixture below caught it
    # dropping the last answer of every set, which for a three-answer probe is
    # a value the recorder would have written as `null` forever.
    printf '%s\n' "$answers" | tr ',' '\n' | while read -r one; do
        one=$(printf '%s' "$one" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')
        [ -n "$one" ] || continue
        if [ "$folded" = "$(printf '%s' "$one" | tr '[:upper:]' '[:lower:]')" ]; then
            printf '%s' "$one"
            return 0
        fi
    done
}

# `--answer-only <log> --answers <set>` runs that step alone, so the derivation
# is held by a fixture without spending a session or an engine.
if [ "$answer_only" = 1 ]; then
    [ -f "$raw" ] || { echo "probe-record: no log at $raw" >&2; exit 2; }
    step_derive_answer
    exit 0
fi

[ -x "$engine" ] || {
    echo "probe-record: no engine at $root/engine/target/{dev-release,release}/headwater." >&2
    echo "probe-record: build one, or the plan members below would be guesses." >&2
    exit 3
}

# step_copy_plan_identity. The six members exist before any session starts, and
# the recorder copies them without change. Reading them from the plan rather
# than recomputing them is the whole point: a recomputed digest is a second
# measurement, and two measurements of a moving tree do not have to agree.
step_copy_plan_identity() {
    "$engine" probe plan --root "$root" ${tier:+--tier "$tier"} > "$1" 2>"$1.err"
}

plan=$(mktemp) || exit 1
trap 'rm -f "$plan" "$plan.err"' EXIT HUP INT TERM
step_copy_plan_identity "$plan" || {
    echo "probe-record: \`headwater probe plan\` failed:" >&2
    cat "$plan.err" >&2
    exit 5
}

member() { sed -n "s/^${1}: *//p" "$plan" | head -1; }

# The plan prints one indented block per selected probe, and the `answers` line
# is present only for a probe whose expectation is `answered`. Reading the set
# from the plan rather than from the probe document keeps every value in this
# script on the one channel the plan already fixed.
declared_answers() {
    awk -v want="$1" '
        /^- / { here = (index($0, "- " want " (") == 1) }
        here && /^    answers: / { sub(/^    answers: /, ""); print; exit }
    ' "$plan"
}

if [ "$identity_only" = 1 ]; then
    printf 'lock: %s\n' "$(member lock)"
    printf 'tree: %s\n' "$(member tree)"
    printf 'selection: %s\n' "$(member selection)"
    printf 'read_set: %s\n' "$(member read_set)"
    printf 'seed: %s\n' "$(member seed)"
    printf 'harness: %s\n' "$(member harness)"
    exit 0
fi

command -v claude >/dev/null 2>&1 || {
    echo "probe-record: the \`claude\` harness is not on the path, and it is the channel." >&2
    exit 3
}

raw=${raw:-$(mktemp)}

# Step 3 of #819. Two variables the session inherits, and both exist so that a
# later reader of the shadow-mode log can tell this session's prompts from a
# person's.
#
# `HEADWATER_PROBE_SESSION` is the name this run already carries, and
# `.claude/hooks/intent.sh` writes it into every line it logs.
# HW-DR-0064 counts person prompts, so a line that names a probe session is a
# line a count subtracts rather than one it reads.
#
# `HEADWATER_SHADOW_LOG_DIR` sends those lines to a directory of this run
# instead. A probe workspace carries no `.git` (#898), so the hook could find no
# common dir there and would write nothing at all. That silence would read as a
# hook that never ran, which is exactly the question the liveness sentence
# below answers. A directory of this run's own makes the two distinguishable,
# and it keeps every probe line out of the collection.
#
# Expect "not live" from a real run of this script. `hw_engine` looks for the
# binary under the session's own project directory, and a probe workspace is a
# corpus copy with no `engine/target` in it, so the hook exits before it routes.
# That is the state this sentence records rather than one it repairs: routing a
# probe session would change what the probe measures, because a cold agent that
# is handed pointers is no longer cold.
probe_log=${HEADWATER_PROBE_LOG_DIR:-$(mktemp -d "${TMPDIR:-/tmp}/headwater-probe-shadow.XXXXXX")}
HEADWATER_PROBE_SESSION=$session
HEADWATER_SHADOW_LOG_DIR=$probe_log
export HEADWATER_PROBE_SESSION HEADWATER_SHADOW_LOG_DIR

# The channel. Standard output of the harness process, read by this script.
# Never a file under `~/.claude/projects/`. The session runs in the workspace
# the guard above cleared, so nothing it writes moves the `tree` digest the
# plan fixed; the subshell keeps that `cd` out of this script's own state.
task=$(cat "$task_file")
(
    cd "$here" || exit 7
    claude -p --output-format stream-json --verbose \
        ${model:+--model "$model"} \
        "$task"
) > "$raw" 2>"$raw.err"
status=$?
if [ "$status" != 0 ]; then
    echo "probe-record: the harness exited $status." >&2
    tail -5 "$raw.err" >&2
    exit "$status"
fi


printf '# raw harness log: %s\n' "$raw" >&2

# The identity block, then the event the transform filters out of the stream.
provider=$(step_derive_provider)
printf '```yaml\n'
printf '%s\n' "$provider" | grep '^model: '
printf '%s\n' "$provider" | grep '^served_version: '
printf 'tree: %s\n' "$(member tree)"
printf 'lock: %s\n' "$(member lock)"
printf 'selection: %s\n' "$(member selection)"
printf 'read_set: %s\n' "$(member read_set)"
printf 'seed: %s\n' "$(member seed)"
printf 'harness: %s\n' "$(member harness)"
printf 'tier: %s\n' "${tier:-regression}"
printf 'arm: %s\n' "${arm:-present}"
printf 'at: %s\n' "$(date -u +%Y-%m-%d)"
printf '%s\n' "$provider" | grep '^cost_cents: '
printf '```\n'

# Whether the intent hook ran in this session, stated rather than left to a
# reader (#917, step 3 of #819). The two committed transcripts of 2026-09-09 and
# 2026-09-11 disagree about this, and nothing in either one says which of them
# had a live hook. The observation is the log this run named above: a line that
# carries this session's name is a line the hook wrote, and no line at all means
# the hook did not reach the engine. The count is of lines rather than of files,
# because one session may submit several prompts.
hook_lines=0
if [ -d "$probe_log" ]; then
    hook_lines=$(cat "$probe_log"/*.jsonl 2>/dev/null \
        | grep -c "\"probe_session\":\"$session\"" || true)
fi
printf '\n'
if [ "$hook_lines" -gt 0 ]; then
    printf 'The intent hook was live in this session. It wrote %s shadow-mode log %s under `probe_session: %s`, in a directory of this run rather than in the log HW-DR-0064 collects, because a probe prompt is not a person prompt.\n' \
        "$hook_lines" "$([ "$hook_lines" = 1 ] && echo line || echo lines)" "$session"
else
    printf 'The intent hook was not live in this session. It wrote no shadow-mode log line under `probe_session: %s`, so no pointer reached this session before it read a file. A workspace with no built engine is the usual reason (#917).\n' \
        "$session"
fi
printf '\n'

answers=$(declared_answers "$probe")
answer=$(step_derive_answer)

# The transform reads the same stream this script wrote, on its standard input.
# It is told the workspace because every artifact the session wrote is in that
# copy and not in the corpus, and `produced` is read from there (#911).
# shellcheck disable=SC2086
sh "$root/tools/probe/probe-transform.sh" --probe "$probe" --session "$session" \
    --root "$root" --workspace "$here" $transform_args ${answer:+--answer "$answer"} < "$raw"
