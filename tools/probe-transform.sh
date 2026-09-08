#!/bin/sh
# The transform half of a recorder: a harness session log on standard input,
# one probe event on standard output.
#
# [HW-DR-0059] admits a recorder built as a driver and a transform. The driver
# starts the session and reads what the harness emits; the transform filters
# that stream into a transcript. [Spec 15] states the condition the pair rests
# on: the log has to reach the transform through a channel the model holds no
# handle on. `tools/probe-record.sh` is the driver and owns that condition.
# This script reads a stream and reaches nothing, so it can be run over a
# recorded log in a fixture and it can be read by a person who wants to know
# what a transcript kept and what it dropped.
#
#     tools/probe-record.sh … | tools/probe-transform.sh --probe PROBE-X \
#         --session one --root . [--produced docs/x.md]…
#
# ## What it keeps
#
# A harness tags each block of its own log by type and the model states no tag
# on any block. So the filter is by tag and never by position, by index, or by
# "the block shapes I saw in my sample". Two tags survive: `tool_use` and
# `tool_result`. Every other block — `thinking`, `text`, `redacted_thinking`,
# and any tag a later harness adds — is dropped whole, including any of its
# text that happens to be shaped like a field of this contract. That last part
# is what `tools/probe-record-fixtures.sh` holds: a `text` block carrying a
# literal `calls:` fragment and a `tool: Read` line must leave no byte behind.
#
# Content blocks sit under `.message.content` of a line whose top-level `type`
# is `assistant` (a call) or `user` (its result). The top-level `type` is not
# the block tag, and a stream also carries `rate_limit_event`, `system` and
# `result` lines that hold no content at all.
#
# ## The five derivation steps, each named
#
# A harness log carries the calls and not the rest of the contract. [Spec 15]
# §"The values a session log omits" names the step behind each value, and each
# one below is a function of this script with that name:
#
#   step_filter_blocks    keep `tool_use` and `tool_result`, drop the rest
#   step_map_call         `tool_use.name` -> `tool`, `tool_use.input` -> `argument`
#   step_derive_result    the content digest of the document the call named,
#                         in the form `headwater probe plan` prints, read from
#                         the corpus and never from the bytes the call returned
#   step_derive_produced  `path`, and `result` as that same digest
#   step_derive_cites     every identifier of this corpus in the artifact
#   step_derive_findings  every rule that reported over the artifact
#
# `step_derive_cites` and `step_derive_findings` are computed the same way for
# every probe and neither reads a probe, which is the boundary [spec 15]
# §"Two produced keys carry a derivation" draws between a recorder and a
# grader. The two identity steps — the six members copied from the plan, and
# `served_version` with `cost_cents` from the provider metadata — belong to the
# driver, which is the half that has the plan and the metadata.
#
# ## The three-state rule, which is where a wrong transform passes silently
#
# `calls: []` means the session was watched and made no call. An absent `calls`
# key means nothing watched it. A transform that emits `[]` for a stream it
# never received turns an unobserved run into a clean result, and every reading
# of that result returns a pass. So this script refuses a stream that carries
# no `system`/`init` line rather than emitting an event: no observation, no
# event. Where the stream did arrive, `calls` is always written, empty or not.
#
# It gates nothing, it writes nothing outside standard output, and it needs
# `jq` and `sha256sum`.

set -u

probe=
session=
root=.
produced_paths=

while [ $# -gt 0 ]; do
    case $1 in
        --probe) probe=${2:-}; shift 2 ;;
        --session) session=${2:-}; shift 2 ;;
        --root) root=${2:-}; shift 2 ;;
        --produced) produced_paths="$produced_paths${2:-}
"; shift 2 ;;
        --answer) answer=${2:-}; shift 2 ;;
        *) echo "probe-transform: unknown argument \`$1\`" >&2; exit 2 ;;
    esac
done
answer=${answer:-}

[ -n "$probe" ] && [ -n "$session" ] || {
    echo "usage: probe-transform.sh --probe <id> --session <name> [--root <dir>] [--produced <path>]… [--answer <text>]" >&2
    exit 2
}
command -v jq >/dev/null 2>&1 || {
    echo "probe-transform: \`jq\` is not on the path." >&2
    exit 3
}
command -v sha256sum >/dev/null 2>&1 || {
    echo "probe-transform: \`sha256sum\` is not on the path." >&2
    exit 3
}

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

cat > "$scratch/log.jsonl"

# The observation guard. A harness stream opens with a `system` line of subtype
# `init`. Without one, this input is not a session log that a driver watched,
# and the honest event is no event at all.
watched=$(jq -s -r '[.[] | select(.type == "system" and .subtype == "init")] | length' \
    < "$scratch/log.jsonl" 2>/dev/null) || watched=0
if [ "${watched:-0}" -lt 1 ]; then
    echo "probe-transform: this stream carries no \`system\`/\`init\` line, so nothing watched the session." >&2
    echo "probe-transform: refusing to write \`calls: []\`, which would claim an observation that was not made." >&2
    exit 4
fi

# step_filter_blocks and step_map_call.
#
# Every line, whatever its top-level type, contributes its `.message.content`
# blocks and nothing else. A block survives only where its own `.type` is
# `tool_use` or `tool_result`; the two `select` clauses below are the entire
# admission rule, and nothing downstream reads a block by its position.
#
# A call is keyed by its `id` so that its `tool_result` can be matched to it,
# and the key is dropped before anything is written. `argument` is the path
# where the input names one, because that is the form the contract's own
# fixture writes, and the compact input otherwise.
step_filter_blocks_and_map_call() {
    jq -c '
        def blocks: (.message.content // []) | if type == "array" then .[] else empty end;
        def path_of($input):
            ($input.file_path // $input.path // $input.notebook_path // null);
        blocks
        | select(type == "object")
        | select(.type == "tool_use" or .type == "tool_result")
        | if .type == "tool_use" then
              { kind: "call", id: (.id // ""), tool: (.name // ""),
                path: path_of(.input // {}),
                argument: (path_of(.input // {}) // ((.input // {}) | tojson)) }
          else
              { kind: "result", id: (.tool_use_id // "") }
          end
    ' < "$scratch/log.jsonl"
}

step_filter_blocks_and_map_call > "$scratch/blocks.jsonl"

# step_derive_result.
#
# The digest is of the document as the corpus holds it, in the form the plan
# prints, and never of the bytes the call returned. A call that named no path,
# or named one outside the corpus, derives no result and writes an empty one:
# the contract carries no digest for a thing this repository does not hold.
step_derive_result() {
    file=$1
    case $file in
        /*) abs=$file ;;
        *) abs=$root/$file ;;
    esac
    [ -f "$abs" ] || { printf ''; return; }
    printf 'sha256:%s' "$(sha256sum < "$abs" | cut -d' ' -f1)"
}

# step_derive_cites: every identifier of this corpus that appears in the
# artifact. The shape is the one `.headwater/` schemes mint, `HW-XX-NNNN`.
step_derive_cites() {
    file=$1
    case $file in
        /*) abs=$file ;;
        *) abs=$root/$file ;;
    esac
    [ -f "$abs" ] || return 0
    grep -oE 'HW-[A-Z]+-[0-9]{4}' "$abs" 2>/dev/null | sort -u
}

# step_derive_findings: every rule that reported over the artifact. It asks the
# engine, because the set of rules is the taxonomy's and never this script's.
step_derive_findings() {
    file=$1
    engine=$root/engine/target/dev-release/headwater
    [ -x "$engine" ] || engine=$root/engine/target/release/headwater
    [ -x "$engine" ] || return 0
    "$engine" check --root "$root" --format json "$file" 2>/dev/null |
        jq -r '.. | objects | .rule? // empty' 2>/dev/null | sort -u
}

# step_derive_produced: one entry per artifact the driver was told the session
# produced. The transform never guesses this from the log, because a written
# file is a fact about the filesystem and the log holds only the request.
step_derive_produced() {
    printf '  produced:'
    if [ -z "$produced_paths" ]; then
        printf ' []\n'
        return
    fi
    printf '\n'
    printf '%s' "$produced_paths" | while IFS= read -r file; do
        [ -n "$file" ] || continue
        printf '    - path: %s\n' "$file"
        printf '      result: %s\n' "$(step_derive_result "$file")"
        cites=$(step_derive_cites "$file")
        if [ -z "$cites" ]; then
            printf '      cites: []\n'
        else
            printf '      cites:\n'
            printf '%s\n' "$cites" | while IFS= read -r id; do
                printf '        - %s\n' "$id"
            done
        fi
        findings=$(step_derive_findings "$file")
        if [ -z "$findings" ]; then
            printf '      findings: []\n'
        else
            printf '      findings:\n'
            printf '%s\n' "$findings" | while IFS= read -r rule; do
                printf '        - %s\n' "$rule"
            done
        fi
    done
}

# The event. `calls` is always written, because the guard above already
# established that this session was watched.
printf -- '- probe: %s\n' "$probe"
printf '  session: %s\n' "$session"

calls=$(jq -c 'select(.kind == "call")' < "$scratch/blocks.jsonl")
if [ -z "$calls" ]; then
    printf '  calls: []\n'
else
    printf '  calls:\n'
    printf '%s\n' "$calls" | while IFS= read -r call; do
        tool=$(printf '%s' "$call" | jq -r '.tool')
        argument=$(printf '%s' "$call" | jq -r '.argument')
        file=$(printf '%s' "$call" | jq -r '.path // ""')
        printf '    - tool: %s\n' "$(printf '%s' "$tool" | jq -R .)"
        printf '      argument: %s\n' "$(printf '%s' "$argument" | jq -R .)"
        if [ -n "$file" ]; then
            printf '      result: %s\n' "$(step_derive_result "$file")"
        else
            printf '      result: ""\n'
        fi
    done
fi

step_derive_produced

if [ -z "$answer" ]; then
    printf '  answer: null\n'
else
    printf '  answer: %s\n' "$(printf '%s' "$answer" | jq -R .)"
fi
