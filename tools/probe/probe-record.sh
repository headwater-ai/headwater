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
#         --workspace /tmp/scratch-copy [--raw raw.jsonl] [--produced docs/x.md]… \
#         [--baseline /tmp/tree-it-was-copied-from] [--max-turns 80]
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
# session is prose. It writes a value only where the final non-empty line of
# the final message, trimmed, is one answer the probe declares, and it writes
# that word and nothing else. A session whose last line is a sentence gave no
# answer in the closed set, and `null` is the true record of it. An answer of
# two words is compared as a set of words, so their order is not read (#1384).
#
# Until #980 the rule read the whole final message. The pilot of 2026-09-28
# found 7 of 27 sessions that wrote one sentence of justification and then the
# word on a line of its own, and the rule recorded each one as no answer. The
# arms differed in how often they did it, so the rule graded the format and
# moved the rates unevenly. The owner ruled on #980 the same day that a
# closed-set word alone on the final line is the answer. The transcript still
# holds no prose, because only the word is written.
#
# The remaining three identity members — `tier`, `arm` and `at` — are stated by
# the caller and the clock, and `--tier`/`--arm` default to what the plan says.
#
# It writes only what `--raw` and standard output name, it needs `jq` and the
# `claude` harness, and it spends real money.
#
# ## Exit status
#
# Every code below is returned by one kind of path and no other, so a caller
# and a fixture can branch on it. The harness's own status never leaves this
# script: any nonzero one is 10, and the status it had is printed on stderr.
# The one exception is a session the turn cap stopped, which is recorded and
# exits 0 (#1384).
# The refusals 2, 4, 6, 8 and 9 run before any check of this host and before
# any harness call, so they spend nothing. When both 8 and 9 apply, 8 answers,
# because the instrument guard runs first.
#
#   0   the transcript was written, or a read-only mode printed its members
#   1   no temporary file for the plan
#   2   a usage error: a missing or unknown argument, task file, workspace,
#       baseline or log, or a workspace that could not be compared with its
#       baseline
#   3   a tool is missing: `jq`, the engine, or the `claude` harness
#   4   the workspace carries a `.git` file or directory
#   5   `headwater probe plan` failed
#   6   the workspace is inside the corpus the plan was taken over
#   7   the harness subshell could not enter the workspace
#   8   the workspace still holds the instrument that `ablate.sh --instrument`
#       names, or that list could not be read
#   9   a document under the workspace's `docs/` names the probe by its
#       identifier or its slug, a file of the workspace names one of the
#       probe's answer keys or a document of this checkout that names the
#       probe, or no `grep` can confirm that none does
#       (`tools/probe/seal.sh` is the remedy)
#   10  the harness exited nonzero for a reason other than the turn cap, and
#       its status is on stderr
#   11  the plan refuses the run, or it does not select the probe. It runs
#       before any harness call, so it spends nothing
#
# ## The plan's refusal is this script's refusal
#
# `headwater probe plan` exits 0 when it refuses a run, on purpose: a probe
# never gates, so no exit status of the engine carries a fact about a run, and a
# caller reads the text. Until #980 this script read the six identity members
# out of that text and never the refusal beside them, so a run over the
# ceiling, or over a probe the tier's absent arm cannot measure, recorded as if
# it had planned. The ceiling in `.headwater/probe.yml` held only where a
# person ran the plan first. Now this script reads the heading the plan prints
# above a refusal and stops, and it stops too when the plan does not select the
# probe it was asked to record.
#
# `--category`, `--exclude` and `--repetitions` reach the plan unchanged, so the
# selection digest a narrowed run records is the digest of the run it planned.
# `--max-turns` reaches the harness, so a batch can hold every session of both
# arms to one cap. With no `--max-turns`, the cap is the one the tier declares
# and the plan prints, and a tier that declares none runs with no cap.
# `--oracle-tree` reaches the transform (see there). `--baseline` names the tree
# the workspace was copied from, and every file the session wrote or changed is
# passed to the transform as produced (see `step_diff_baseline`).

set -u

probe=
session=
task_file=
model=
workspace=
raw=
tier=
arm=
category=
excludes=
repetitions=
max_turns=
identity_only=0
provider_only=0
answer_only=0
answers=
transform_args=
baseline=

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
        --category) category=${2:-}; shift 2 ;;
        --exclude) excludes="$excludes --exclude ${2:-}"; shift 2 ;;
        --repetitions) repetitions=${2:-}; shift 2 ;;
        --max-turns) max_turns=${2:-}; shift 2 ;;
        --oracle-tree) transform_args="$transform_args --oracle-tree ${2:-}"; shift 2 ;;
        --identity-only) identity_only=1; shift ;;
        --provider-only) provider_only=1; raw=${2:-}; shift 2 ;;
        --answer-only) answer_only=1; raw=${2:-}; shift 2 ;;
        --answers) answers=${2:-}; shift 2 ;;
        --produced) transform_args="$transform_args --produced ${2:-}"; shift 2 ;;
        --baseline) baseline=${2:-}; shift 2 ;;
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
    if [ -n "$baseline" ] && [ ! -d "$baseline" ]; then
        echo "probe-record: no baseline directory at $baseline" >&2
        exit 2
    fi
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
    # The instrument: the probe shelves every arm removes, because a probe
    # document states the answer it expects. A workspace that still holds one
    # hands the session its own answer key.
    # Fail closed: an instrument this script cannot read is a workspace it
    # cannot clear, and a loop over the output of a failed command runs zero
    # times and lets the session start.
    instrument=$(sh "$root/tools/probe/ablate.sh" --instrument) || {
        echo "probe-record: the instrument of the probe declaration could not be read, so this workspace cannot be cleared of it." >&2
        exit 8
    }
    for path in $instrument; do
        if [ -e "$here/$path" ]; then
            echo "probe-record: the workspace at $here still holds \`$path\`, which every arm removes." >&2
            echo "probe-record: prepare it with \`sh tools/probe/ablate.sh --present <workspace>\` or \`sh tools/probe/ablate.sh <tier> <workspace>\`." >&2
            exit 8
        fi
    done
    # The answer key outside the instrument. A document under `docs/` that
    # names the probe is a record about it, and each such record on this shelf
    # states the probe's expected value, a recorded answer or its target
    # (#1229). `tools/probe/seal.sh` removes those records, and this guard
    # refuses a workspace that was not sealed. It reads `docs/` alone: a file
    # outside it names a probe by path or title, as a derived fold or the
    # overlay does, and states no answer. It runs before any harness call, so
    # the case that asserts it spends nothing, and it exits 9.
    #
    # The slug is the file name the probe has on this checkout's shelf. A
    # record links the probe by that path and not by the identifier, so both
    # are searched for. A host with no `grep` cannot confirm the tree is
    # clean, and that is a refusal rather than a pass.
    command -v grep >/dev/null 2>&1 || {
        echo "probe-record: \`grep\` is not on the path, so nothing can confirm the workspace holds no answer key." >&2
        exit 9
    }
    slug=""
    shelf_file=$(grep -rlx -- "id: $probe" "$root/docs/probes" 2>/dev/null) || shelf_file=""
    case "$shelf_file" in
        *.md)
            slug=${shelf_file##*/}
            slug=${slug%.md}
            ;;
    esac
    if [ -n "$slug" ]; then
        key=$(grep -rlF -e "$probe" -e "$slug" -- "$here/docs" 2>/dev/null) || key=""
    else
        key=$(grep -rlF -e "$probe" -- "$here/docs" 2>/dev/null) || key=""
    fi
    if [ -n "$key" ]; then
        first=${key%%
*}
        echo "probe-record: the workspace names the probe it would run: $first" >&2
        echo "probe-record: a session that reads that file reads its own answer key. Run \`sh tools/probe/seal.sh $here $probe\` first." >&2
        exit 9
    fi
    # The answer keys `.headwater/probe.yml` declares for the probe (#980): a
    # document an earlier session wrote in answer to this task. Any file of the
    # workspace that still names one hands the session its task already done.
    keys=$(sh "$root/tools/probe/seal.sh" --keys "$probe") || {
        echo "probe-record: the answer keys of $probe could not be read, so this workspace cannot be cleared of them." >&2
        exit 9
    }
    for key in $(printf '%s\n' "$keys" | awk '{ n = split($2, p, "/"); s = p[n]; sub(/\.md$/, "", s); print $1; print s }'); do
        left=$(grep -rlIF -e "$key" -- "$here" 2>/dev/null | head -1) || left=""
        if [ -n "$left" ]; then
            echo "probe-record: the workspace still names the answer key $key: $left" >&2
            echo "probe-record: run \`sh tools/probe/seal.sh $here $probe\` first." >&2
            exit 9
        fi
    done
    # The named documents (#1384). The seal deletes each document under
    # `docs/` that names the probe, and every line that names that document
    # by its identifier or its `<slug>.md` (#1293). A workspace whose record is
    # gone but whose register still links it passed the two checks above,
    # because the line names the record and not the probe. `seal.sh --named`
    # prints the names the seal strips, read from this checkout, and this
    # guard matches them with the seal's own edge, so an identifier inside a
    # longer one does not refuse a sealed tree.
    named=$(sh "$root/tools/probe/seal.sh" --named "$probe") || {
        echo "probe-record: the documents that name $probe could not be read, so this workspace cannot be cleared of them." >&2
        exit 9
    }
    if [ -n "$named" ]; then
        edge='[^A-Za-z0-9_-]'
        # `awk` and `grep` alone, like the checks above, so the guard runs on
        # the smallest `PATH` the fixtures give it. Every character that is
        # not a letter, a digit, `-` or `_` is escaped as a bracket.
        pattern=$(printf '%s\n' "$named" | awk -v e="$edge" 'BEGIN { ORS = "" } {
                name = $1
                md = (name ~ /\.md$/)
                out = ""
                for (i = 1; i <= length(name); i++) {
                    c = substr(name, i, 1)
                    if (c ~ /[A-Za-z0-9_-]/) out = out c
                    else out = out "[" c "]"
                }
                if (NR > 1) print "|"
                if (md) print "(^|" e ")" out
                else print "(^|" e ")" out "(" e "|$)"
            }')
        left=$(grep -rlIE -e "$pattern" -- "$here" 2>/dev/null | awk 'NR == 1') || left=""
        if [ -n "$left" ]; then
            echo "probe-record: the workspace still names a document that names $probe: $left" >&2
            echo "probe-record: run \`sh tools/probe/seal.sh $here $probe\` first." >&2
            exit 9
        fi
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
#
# An answer of more than one word is compared as a set of words (#1384): the
# line and the declared answer are split on space, folded, and sorted, and the
# two lists must be equal. The status pilot of 2026-09-29 ended an absent-arm
# session with `HW-DR-0052 current` for the declared `current HW-DR-0052`, and
# the whole-line rule recorded it as no answer. The words are the same two
# facts, so the order is format and not content. The rule still takes no
# substring and no extra word: `0052 current HW-DR-0052` is three words and is
# no answer. The value written is the declared form.
words() {
    printf '%s\n' "$1" | tr '[:upper:]' '[:lower:]' | tr -s ' \t' '\n\n' | awk 'NF' | sort | tr '\n' ' '
}

step_derive_answer() {
    [ -n "$answers" ] || return 0
    # The final non-empty line of the final message (#980). A message that is
    # one word is its own final line, so the whole-message reading is the case
    # of one line and not a second rule.
    said=$(jq -s -r '([.[] | select(.type == "result")] | last | .result // "")' < "$raw" \
        | tr -d '\r' | awk 'NF { last = $0 } END { print last }' \
        | sed 's/^[[:space:]]*//; s/[[:space:]]*$//; s/\.$//')
    [ -n "$said" ] || return 0
    folded=$(words "$said")
    # `printf '%s\n'` and never `printf '%s'`: a set of one answer carries no
    # comma, so the unterminated form gives `read` a line with no newline, and
    # `while read` stops before the body on that. The fixture below caught it
    # dropping the last answer of every set, which for a three-answer probe is
    # a value the recorder would have written as `null` forever.
    printf '%s\n' "$answers" | tr ',' '\n' | while read -r one; do
        one=$(printf '%s' "$one" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')
        [ -n "$one" ] || continue
        if [ "$folded" = "$(words "$one")" ]; then
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
    # shellcheck disable=SC2086
    "$engine" probe plan --root "$root" ${tier:+--tier "$tier"} \
        ${category:+--category "$category"} $excludes \
        ${repetitions:+--repetitions "$repetitions"} > "$1" 2>"$1.err"
}

plan=$(mktemp) || exit 1
trap 'rm -f "$plan" "$plan.err"' EXIT HUP INT TERM
step_copy_plan_identity "$plan" || {
    echo "probe-record: \`headwater probe plan\` failed:" >&2
    cat "$plan.err" >&2
    exit 5
}
if grep -q '^## This run does not start' "$plan"; then
    echo "probe-record: \`headwater probe plan\` refuses this run:" >&2
    sed -n '/^## This run does not start/,$p' "$plan" | sed '1,2d' >&2
    exit 11
fi
if [ "$identity_only" = 0 ] && [ -n "$probe" ] && ! grep -q "^- $probe (" "$plan"; then
    echo "probe-record: the plan does not select $probe, so a session of it records nothing this run planned." >&2
    exit 11
fi

member() { sed -n "s/^${1}: *//p" "$plan" | head -1; }

# The turn cap (#1384). A tier declares one in `.headwater/probe.yml` and the
# plan prints it in its cost section. `--max-turns` overrides it, and a tier
# that declares none runs a session with no cap, as before.
if [ -z "$max_turns" ]; then
    max_turns=$(sed -n 's/.*stops at a turn cap of \([0-9][0-9]*\).*/\1/p' "$plan" | head -1)
fi

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
# `hw_engine` looks for the binary under the session's own project directory.
# A workspace built by hand is a corpus copy with no `engine/target` in it, so
# the hook exits before it routes, and every recording before #980 says "not
# live". `tools/probe/campaign.sh` copies the built binary into every workspace,
# on the owner's ruling on #980 (2026-09-28). There the hook is live in a
# present-arm session, because the hook is part of the governance the campaign
# measures, and the absent arm has no `.claude/` to run it from.
probe_log=${HEADWATER_PROBE_LOG_DIR:-$(mktemp -d "${TMPDIR:-/tmp}/headwater-probe-shadow.XXXXXX")}
HEADWATER_PROBE_SESSION=$session
HEADWATER_SHADOW_LOG_DIR=$probe_log
export HEADWATER_PROBE_SESSION HEADWATER_SHADOW_LOG_DIR

# The channel. Standard output of the harness process, read by this script.
# Never a file under `~/.claude/projects/`. The session runs in the workspace
# the guard above cleared, so nothing it writes moves the `tree` digest the
# plan fixed; the subshell keeps that `cd` out of this script's own state.
task=$(cat "$task_file")
# The subshell cannot return 7 for its own `cd` and let a harness return 7
# too, so it leaves a marker instead and the status below is the harness's.
rm -f "$raw.nocd"
(
    cd "$here" || { : > "$raw.nocd"; exit 1; }
    claude -p --output-format stream-json --verbose \
        ${model:+--model "$model"} \
        ${max_turns:+--max-turns "$max_turns"} \
        "$task"
) > "$raw" 2>"$raw.err"
status=$?
if [ -e "$raw.nocd" ]; then
    rm -f "$raw.nocd"
    echo "probe-record: the session could not enter the workspace at $here." >&2
    exit 7
fi
# A harness status is the harness's own choice and may equal any code above,
# so every nonzero one leaves as 10, with the status it had on stderr.
#
# One nonzero status is an observation and not a failure (#1384). A session
# that `--max-turns` stopped ends its stream with a `result` line of subtype
# `error_max_turns`, and the harness exits 1. Until #1384 this script exited
# 10 for it, so the campaign of 2026-09-28 dropped 4 of 540 sessions, all on
# the `patched` probe, and a resumed batch drew each one again. A draw that
# can be repeated until it finishes under the cap is not a draw. So a capped
# session records what the log holds: its calls, what it wrote, and no answer,
# because its `result` line carries no text. The transcript says it was capped.
capped=0
if [ "$status" != 0 ]; then
    stopped=$(jq -s -r '([.[] | select(.type == "result")] | last | .subtype // "")' < "$raw" 2>/dev/null) || stopped=""
    if [ "$stopped" = error_max_turns ]; then
        capped=1
    else
        echo "probe-record: the harness exited $status." >&2
        tail -5 "$raw.err" >&2
        exit 10
    fi
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
if [ "$capped" = 1 ]; then
    printf 'The session stopped at the turn cap of %s.\n\n' "${max_turns:-the harness}"
fi

answers=$(declared_answers "$probe")
answer=
[ "$capped" = 1 ] || answer=$(step_derive_answer)

# step_diff_baseline (#1384). A write made through `Bash` names no path in the
# log, so the transform cannot seed `produced` from it. On 2026-09-28, 22 of 30
# present-arm sessions of the `cited` probe and 26 of 30 absent-arm sessions
# recorded `produced: []`, and the grader reads only `produced`. With
# `--baseline <dir>`, the tree the workspace was copied from, every regular
# file that is new or changed after the session is passed to the transform as
# `--produced`. A file under `.claude/worktrees/<name>/` is compared with the
# file at the same path under the baseline's root, so a worktree the session
# made counts the files it changed and not the checkout it copied. The engine's
# build and cache directories and the probe log directory are passed over.
#
# A file is unchanged when its size and its time match the baseline's, since a
# `cp -a` keeps both, and otherwise when its content digest matches. It needs
# GNU `find` for `-printf`. A path that holds a tab or a newline is passed over.
step_diff_baseline() {
    list() {
        (cd "$1" && find . \( -path ./engine/target -o -path ./.headwater/cache \
            -o -path './.claude/worktrees/*/engine/target' -o -path './.claude/worktrees/*/.headwater/cache' \) \
            -prune -o -type f -printf '%P\t%s\t%T@\n')
    }
    list "$baseline" > "$diffdir/base.tsv" || return 1
    list "$here" > "$diffdir/here.tsv" || return 1
    log_rel=
    case "$probe_log" in
        "$here"/*) log_rel=${probe_log#"$here"/}/ ;;
    esac
    awk -F '\t' -v log_rel="$log_rel" '
        NR == FNR { size[$1] = $2; time[$1] = $3; next }
        NF != 3 { next }
        log_rel != "" && index($1, log_rel) == 1 { next }
        {
            key = $1
            sub(/^\.claude\/worktrees\/[^\/]+\//, "", key)
            if (!(key in size) || size[key] != $2) { print "new\t" $1; next }
            if (key == $1 && time[key] == $3) next
            print "check\t" $1 "\t" key
        }
    ' "$diffdir/base.tsv" "$diffdir/here.tsv" > "$diffdir/candidates.tsv"
    awk -F '\t' '$1 == "new" { print $2 }' "$diffdir/candidates.tsv" > "$diffdir/produced"
    awk -F '\t' '$1 == "check" { print $2 }' "$diffdir/candidates.tsv" > "$diffdir/here.list"
    awk -F '\t' '$1 == "check" { print $3 }' "$diffdir/candidates.tsv" > "$diffdir/base.list"
    if [ -s "$diffdir/here.list" ]; then
        (cd "$here" && tr '\n' '\0' < "$diffdir/here.list" | xargs -0 sha256sum) > "$diffdir/here.sum" || return 1
        (cd "$baseline" && tr '\n' '\0' < "$diffdir/base.list" | xargs -0 sha256sum) > "$diffdir/base.sum" || return 1
        # `sha256sum` prints in the order it was given, so line N of each sum
        # is the pair on line N of the lists. A digest is the first 64
        # characters of its line, and the path is taken from the list.
        awk 'NR == FNR { h[FNR] = substr($0, 1, 64); next }
             substr($0, 1, 64) != h[FNR] { print FNR }' "$diffdir/here.sum" "$diffdir/base.sum" \
            > "$diffdir/differ"
        awk 'NR == FNR { want[$1]; next } FNR in want' "$diffdir/differ" "$diffdir/here.list" \
            >> "$diffdir/produced"
    fi
    sort -u "$diffdir/produced"
}

set --
if [ -n "$baseline" ]; then
    baseline=$(cd "$baseline" && pwd)
    diffdir=$(mktemp -d) || exit 1
    trap 'rm -f "$plan" "$plan.err"; rm -rf "$diffdir"' EXIT HUP INT TERM
    step_diff_baseline > "$diffdir/produced.sorted" || {
        echo "probe-record: the workspace could not be compared with the baseline at $baseline." >&2
        exit 2
    }
    while IFS= read -r path; do
        [ -n "$path" ] && set -- "$@" --produced "$path"
    done < "$diffdir/produced.sorted"
fi

# The transform reads the same stream this script wrote, on its standard input.
# It is told the workspace because every artifact the session wrote is in that
# copy and not in the corpus, and `produced` is read from there (#911).
# shellcheck disable=SC2086
sh "$root/tools/probe/probe-transform.sh" --probe "$probe" --session "$session" \
    --root "$root" --workspace "$here" $transform_args "$@" ${answer:+--answer "$answer"} < "$raw"
