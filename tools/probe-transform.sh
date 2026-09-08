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
#
# The mapping is **total over what the filter admits**, and the third arm is a
# `halt_error` rather than a fall-through. That is what gives the allowlist a
# consequence a test can see: weaken it to a denylist and a `redacted_thinking`
# block, or any tag a later harness adds, reaches the mapping and stops the run
# with the tag named. A fall-through `else` made the allowlist unobservable,
# because an unadmitted block was silently rewritten as a result and dropped a
# second time downstream, and the suite passed 21 of 21 against a denylist.
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
          elif .type == "tool_result" then
              { kind: "result", id: (.tool_use_id // "") }
          else
              ("probe-transform: a block tagged `\(.type)` reached the mapping. The filter above is an allowlist and this tag is not on it." | halt_error(5))
          end
    ' < "$scratch/log.jsonl"
}

step_filter_blocks_and_map_call > "$scratch/blocks.jsonl" || exit $?

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
# `headwater check` takes **no positional path**. It reports over the whole
# corpus, so the run is one run and the artifact is selected out of
# `.findings[]` by its `path`. Measured 2026-09-08: the first draft of this
# function passed the path positionally, which exits 1 with `unexpected
# argument`, and it sent stderr to `/dev/null` and let the empty pipe become
# the answer. That returned `findings: []` for every artifact, permanently.
#
# An empty `findings` list is a claim that no rule reported over the artifact.
# So a failure here refuses rather than writes one, on the same reasoning as
# the observation guard above: a derivation that could not run is not a
# derivation that found nothing.
step_derive_findings() {
    file=$1
    engine=$root/engine/target/dev-release/headwater
    [ -x "$engine" ] || engine=$root/engine/target/release/headwater
    [ -x "$engine" ] || {
        echo "probe-transform: no engine, so \`findings\` on $file cannot be derived." >&2
        echo "probe-transform: refusing to write an empty list, which claims that no rule reported." >&2
        exit 5
    }
    if [ ! -f "$scratch/check.json" ]; then
        "$engine" check --root "$root" --format json > "$scratch/check.json" 2>"$scratch/check.err" || {
            echo "probe-transform: \`headwater check --format json\` failed:" >&2
            tail -3 "$scratch/check.err" >&2
            exit 5
        }
    fi
    jq -r --arg path "$file" '.findings[] | select(.path == $path) | .rule' \
        < "$scratch/check.json" | sort -u
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
    # Read from a redirect and not from a pipe. A `… | while` body runs in a
    # subshell, so the `exit 5` a refused derivation raises would end that
    # subshell and leave this script writing the rest of an event it had
    # already refused.
    printf '%s' "$produced_paths" > "$scratch/produced.txt"
    while IFS= read -r file; do
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
        # A command substitution is a subshell, so the refusal has to be read
        # off the status rather than left to `exit` inside it.
        findings=$(step_derive_findings "$file") || exit $?
        if [ -z "$findings" ]; then
            printf '      findings: []\n'
        else
            printf '      findings:\n'
            printf '%s\n' "$findings" | while IFS= read -r rule; do
                printf '        - %s\n' "$rule"
            done
        fi
    done < "$scratch/produced.txt"
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
        # `jq -c` on a string value is that string in JSON, on one line, with
        # every control byte escaped, and a YAML double-quoted scalar accepts
        # JSON's escapes. `printf … | jq -R .` is not that: it reads its input
        # by lines and emits one JSON string per line, so a `file_path` or a
        # command carrying a newline put raw model-written bytes at column 0 of
        # the event and broke the block. The `tojson` arm of the mapping was
        # already safe and the path arm was not, which is the shape of the
        # defect: one of two branches encoded, the other formatted.
        tool=$(printf '%s' "$call" | jq -c '.tool')
        argument=$(printf '%s' "$call" | jq -c '.argument')
        file=$(printf '%s' "$call" | jq -r '.path // ""')
        printf '    - tool: %s\n' "$tool"
        printf '      argument: %s\n' "$argument"
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
