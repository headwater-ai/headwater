#!/bin/sh
# The driver half of a recorder: it starts one probe session and reads what the
# harness emits.
#
# [HW-DR-0059] admits a recorder built as a driver and a transform, on one
# condition, and this script is where that condition is either met or lost.
# [Spec 15] §"A recorder is a driver and a transform" states it: **the log has
# to reach the transform through a channel the model has no handle on.**
#
#     sh tools/probe-record.sh --probe PROBE-X --session one \
#         --task-file task.md --model claude-haiku-4-5 \
#         --workspace /tmp/scratch-copy [--raw raw.jsonl] [--produced docs/x.md]…
#
# Two modes read nothing from the network. `--identity-only` prints the six
# members `headwater probe plan` fixes, and `--provider-only <log>` prints the
# three the provider metadata carries, over a log already recorded.
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
        --produced) transform_args="$transform_args --produced ${2:-}"; shift 2 ;;
        *) echo "probe-record: unknown argument \`$1\`" >&2; exit 2 ;;
    esac
done

root=$(cd "$(dirname "$0")/.." && pwd)
engine=$root/engine/target/dev-release/headwater
[ -x "$engine" ] || engine=$root/engine/target/release/headwater

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
step_derive_provider() {
    jq -s -r '
        ([.[] | select(.type == "result")] | last) as $r
        | ([.[] | select(.type == "system" and .subtype == "init")] | last) as $i
        | (($r.modelUsage // {}) | keys) as $used
        | ($i.model // ($used | first) // "unknown") as $name
        | {
            model: $name,
            served: (([$used[] | select(test("-[0-9]{8}$"))] | first) // $name),
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

if [ "$identity_only" = 1 ]; then
    printf 'lock: %s\n' "$(member lock)"
    printf 'tree: %s\n' "$(member tree)"
    printf 'selection: %s\n' "$(member selection)"
    printf 'read_set: %s\n' "$(member read_set)"
    printf 'seed: %s\n' "$(member seed)"
    printf 'harness: %s\n' "$(member harness)"
    exit 0
fi

[ -n "$probe" ] && [ -n "$session" ] && [ -n "$task_file" ] && [ -n "$workspace" ] || {
    echo "usage: probe-record.sh --probe <id> --session <name> --task-file <f> --workspace <dir> [--model <m>]" >&2
    exit 2
}
[ -f "$task_file" ] || { echo "probe-record: no task file at $task_file" >&2; exit 2; }
command -v claude >/dev/null 2>&1 || {
    echo "probe-record: the \`claude\` harness is not on the path, and it is the channel." >&2
    exit 3
}

# The workspace guard. A session that runs in the corpus the plan was taken
# against can move the tree digest the plan just fixed.
here=$(cd "$workspace" 2>/dev/null && pwd) || {
    echo "probe-record: no workspace directory at $workspace" >&2
    exit 2
}
case "$here" in
    "$root"|"$root"/*)
        echo "probe-record: the workspace is inside the corpus the plan was taken over." >&2
        echo "probe-record: a session that writes there moves the \`tree\` digest above. Use a copy." >&2
        exit 6
        ;;
esac

raw=${raw:-$(mktemp)}

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

# The transform reads the same stream this script wrote, on its standard input.
# shellcheck disable=SC2086
sh "$root/tools/probe-transform.sh" --probe "$probe" --session "$session" \
    --root "$root" $transform_args < "$raw"
