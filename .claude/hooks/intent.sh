#!/bin/sh
# The intent-time hook: `headwater route`, on the task before the agent reads a
# file. Registered on `UserPromptSubmit`, which takes no matcher, so it sees
# every prompt, including the ones no person wrote. It answers only the ones a
# person did: the envelope test below is the whole of that distinction.
#
# What it passes to the engine: the prompt, as text. Nothing else. The engine
# reads the checkout itself.
#
# What it hands back: what `route` wrote, verbatim, on standard output, which
# this harness adds to the agent's context. It hands back nothing at all when
# the route offered no pointer, because spec 5 makes silence a result and a
# wrong pointer costs more than a missing one.
#
# What a refusal means here: there is none. `route` exits 0 whether or not it
# offers a pointer, and this hook never blocks a prompt. An intent-time hook
# that refused would refuse the work rather than the mistake.
#
# It fails open and silently. Every other position in this repository prints one
# line when it cannot run, and this one does not, because it runs on every
# prompt and a line per prompt is a cost the reader pays forever. The commit
# hook says out loud when the engine is missing, once, at the moment it matters.
#
# # What it reads, and what it used to read
#
# `route --json` writes one document, and this hook decides on its `pointers`
# member: written on every run, and written empty where the route had nothing to
# offer. Until #321 there was no machine form of this verb at all, so the
# decision was a grep for an em-dash separator over the rendered report, with
# an em dash inside a document summary standing in for "the corpus has
# something for this". A summary written without one, or a line of prose that
# happened to carry one, moved the decision. That is clig.dev's "human-readable
# output breaking machine-readable output", and it was live in this repository.
#
# The same document carries the rendered report as `text`, so what reaches the
# agent is still the engine's own rendering, byte for byte. This hook composes
# no line of its own: a pointer line assembled here would be a second copy of
# `Pointer::render`, and a terminal and an agent that read different text about
# one document are reading about different corpora as far as either can tell.
#
# # The shadow-mode routing log
#
# [HW-DR-0064](../../docs/decisions/0064-q64-whether-intent-time-routing-gains-an-offline-embedding-path-in-shadow-mode.md)
# rules that this hook, and nothing else, keeps a private record of what the
# deterministic route decided, so a later comparison against a second, meaning
# based lookup has something to compare against. [#819] carries the build in
# four steps; this file carries step 1 alone, the half that needs no model and
# no vector: the task, the route document as `route` wrote it, and whether the
# pointers it offered reached the agent.
#
# `hw_shadow_log`, below, is the whole of it. It runs once `route` has already
# answered, so a corpus this cannot log for is a corpus this hook still routes
# for exactly as before it existed: the function only ever adds a write to a
# file nobody reads back here, and never a line of output or a changed exit
# status. It fails open the same way every read in this file already does,
# and silently, because a write error surfacing on either stream would corrupt
# what the harness injects, and a non-zero exit would block the prompt.
#
# The file lives at `<git common dir>/headwater-shadow-log/<session>.jsonl`,
# beside the marks `touch.sh` and `review.sh` already keep under the common
# dir rather than under the tree: every worktree of one clone answers the same
# path for it, `repo-cleanup` removing one worktree never touches it, and
# `headwater generate` and `headwater check` never read it because it sits
# outside the corpus entirely. One file per harness session is what keeps a
# POSIX append atomic: a session submits its prompts in order, so a file per
# session has exactly one writer, and the ten worktrees this host may run at
# once never interleave a line because each one holds a session of its own.
#
# A session with no `session_id` on its payload logs nothing at all, the same
# posture `touch.sh` already takes: nobody has yet named a file to hold a line
# for. Every case at the top of this suite predates the shadow log and carries
# no `session_id`, so none of them gains a write; the log-writing cases live
# in their own block, further down, with a session id of their own.
#
# The one gotcha worth stating here because it is easy to get wrong twice: a
# shell that fails to open a file for `>>` writes its complaint to whatever
# stderr already is at that point, and a `2>/dev/null` placed after the `>>`
# on the same line does not catch it — the two are set up in the order
# written, and the redirection that failed already reported before the one
# meant to silence it took effect. Wrapping the write in a brace group and
# putting the redirect on the group is what actually silences it, and the
# decisive fixture in `fixtures.sh` is what would catch a return to the
# broken form.

. "$(dirname "$0")/lib.sh"

# One line, appended to the file the header above names. Every value it
# cannot get — no session id, no common git dir, no writable directory, no
# quotable string — ends this function at that point and leaves nothing
# written, the same fail-open posture as `hw_field` and `hw_count` below. It
# never touches standard output or standard error, and it never sets an exit
# status the caller reads, because the caller has already decided both by the
# time this runs. Defined ahead of the input it reads, because a shell script
# executes top to bottom and a call to a function defined after the point
# that calls it finds nothing there yet — `foo: not found`, on standard
# error, which is exactly the leak this whole file exists to not have.
hw_shadow_log() {
    _session=$(hw_field "$input" session_id) || return 0
    [ -n "$_session" ] || return 0

    _dir=${HEADWATER_SHADOW_LOG_DIR:-}
    if [ -z "$_dir" ]; then
        _common=$(hw_common_dir) || return 0
        _dir="$_common/headwater-shadow-log"
    fi
    mkdir -p "$_dir" 2>/dev/null || return 0
    _file="$_dir/$_session.jsonl"

    # The taxonomy lock's own digest, read off the file this repository
    # already commits rather than asked of the engine to recompute — spec
    # 15's own identity block reads the same field the same way. The corpus
    # tree digest that goes beside it in that block has no such shortcut — it
    # is a census walk over every classified document, and step 2's
    # `neighbors` verb is where the issue's own build order puts that call,
    # so this line carries no `tree_digest` member until then.
    _lock=$("$engine" json field lock digest < "$hw_root/.headwater/taxonomy.lock" 2>/dev/null) || _lock=
    _version=$("$engine" -V 2>/dev/null) || _version=
    _at=$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null) || _at=

    # The prompt id is the join key to the harness transcript, and the only
    # way a count can tell a typed prompt from one the harness submitted on a
    # schedule. The payload carries no such fact, and the transcript line that
    # does is written after this hook runs (#917). An absent id is an empty
    # string, so the member is present on every line.
    _prompt_id=$(hw_field "$input" prompt_id) || _prompt_id=

    # Step 3 of #819: the recorder's own name for the session it drives, which
    # `tools/probe/probe-record.sh` exports before it starts the harness. A
    # person's session carries none, so the member is empty on every line a
    # person's prompt writes, and a count reads a non-empty value as a prompt
    # that a recorder submitted. That recorder also names a log directory of
    # its own, so the lines this names never reach the collection HW-DR-0064
    # counts.
    _probe_session=${HEADWATER_PROBE_SESSION:-}

    # Step 2 of #819: the embedding path's ranking for the same text, from
    # `headwater neighbors`, which also supplies the tree digest. The model
    # files are shared by every worktree under the common dir, where
    # `tools/embed/fetch-model.sh` puts them. A run that cannot load the pinned
    # model writes `"neighbors":null`, so a line written after this step says
    # the ranking was missing rather than never asked for.
    _models=${HEADWATER_MODEL_DIR:-}
    if [ -z "$_models" ]; then
        _models=$(hw_common_dir) && _models="$_models/headwater-models" || _models=
    fi
    _neighbors=
    if [ -n "$_models" ]; then
        _neighbors=$("$engine" neighbors --root "$hw_root" --model "$_models" --json "$task" 2>/dev/null) || _neighbors=
    fi
    _embedding=',"neighbors":null'
    if [ -n "$_neighbors" ]; then
        _tree=$(printf '%s' "$_neighbors" | "$engine" json field tree_digest 2>/dev/null) || _tree=
        _model=$(printf '%s' "$_neighbors" | "$engine" json field model_digest 2>/dev/null) || _model=
        _tree_q=$(hw_quote "$_tree") || _tree_q='""'
        _model_q=$(hw_quote "$_model") || _model_q='""'
        _neighbors_q=$(hw_quote "$_neighbors") || _neighbors_q=
        if [ -n "$_neighbors_q" ]; then
            _embedding=$(printf ',"tree_digest":%s,"model_digest":%s,"neighbors":%s' "$_tree_q" "$_model_q" "$_neighbors_q")
        fi
    fi

    _session_q=$(hw_quote "$_session") || return 0
    _prompt_id_q=$(hw_quote "$_prompt_id") || _prompt_id_q='""'
    _probe_session_q=$(hw_quote "$_probe_session") || _probe_session_q='""'
    _root_q=$(hw_quote "$hw_root") || return 0
    _task_q=$(hw_quote "$task") || return 0
    _route_q=$(hw_quote "$route") || return 0
    _version_q=$(hw_quote "$_version") || _version_q='""'
    _lock_q=$(hw_quote "$_lock") || _lock_q='""'

    _line=$(printf '{"at":"%s","session":%s,"prompt_id":%s,"probe_session":%s,"corpus_root":%s,"engine_version":%s,"lock_digest":%s,"task":%s,"injected":%s,"route":%s%s}' \
        "$_at" "$_session_q" "$_prompt_id_q" "$_probe_session_q" "$_root_q" "$_version_q" "$_lock_q" "$_task_q" "$injected" "$_route_q" "$_embedding") || return 0

    # The brace group is what keeps this silent, and not a stylistic choice: a
    # bare `printf ... >> "$_file" 2>/dev/null` still leaks "cannot create" to
    # the real standard error when the `>>` itself is what fails, because a
    # shell sets redirections up in the order written and the first one had
    # already reported before the second took effect. Putting the redirect on
    # the group instead silences the open failure along with everything it
    # wraps.
    { printf '%s\n' "$_line" >> "$_file"; } 2>/dev/null
    return 0
}

input=$(cat)
engine=$(hw_engine) || exit 0

task=$(hw_field "$input" user_input) || task=$(hw_field "$input" prompt) || exit 0
[ -n "$task" ] || exit 0

# Not every prompt was written by a person. This position takes no matcher, and
# the header above says it therefore sees every prompt — written when a prompt
# meant somebody typing. A harness that runs agents in parallel submits a prompt
# each time one of them finishes, and the text of that prompt is the agent's own
# closing report inside an envelope. Routing over it is a category error twice
# over: the report is not a task, and it is already in the reader's context, so
# every pointer the route offers was chosen against text nobody wrote as an
# intent.
#
# Measured on run 22, on 2026-09-06: 39 of this hook's 46 firings were a
# `<task-notification>`, and none of the 39 had a task in it. One of them routed
# over a finished agent's report, emitted 45.4KB, and reported that the budget
# had withheld 279 more pointers.
#
# So the envelope is the signal, and the test is the first characters rather
# than a search: a person may well write the words `task-notification` in a
# question about this repository, and that question deserves its pointers.
case $task in
    '<task-notification>'* | '<cross-session-message'* ) exit 0 ;;
esac

# Corrected from the payload's own `cwd` now that it is readable. `engine`
# above is a binary, found once through the pre-correction root and reused as
# is — locating it is not a corpus-relative act, only what it is asked about
# below is. `--root "$hw_root"` is that corpus-relative ask, so it is the one
# argument this correction has to reach before the call is made.
hw_root=$(hw_resolve_root "$input")

route=$("$engine" route --root "$hw_root" --json "$task" 2>/dev/null) || exit 0

# Two reads of one document, and the same engine that wrote it reads it back.
# `pointers` is written on every run and written empty where the route had
# nothing to offer, so the count is the decision and the emptiness is a result
# rather than a failure. A route that offered something then hands back the
# rendering it wrote. Every other way this can fail — a document that will not
# parse, a member that is not there, no built engine — leaves the substitution
# empty or its status non-zero, and both of those end the hook with the prompt
# untouched.
pointers=$(hw_count "$route" pointers) || exit 0

# `report` and `injected` are worked out here, ahead of the shadow-log write
# below, rather than at the two early exits the rest of this hook used before
# the log existed. The four branches below reach the same output and the same
# exit status the two-exit form gave: an empty `report` is never printed
# either way, and every path here still ends at the one `exit 0` at the
# bottom. What changes is only that `hw_shadow_log` now runs, once, on every
# branch, because a silence is as much a shadow-log line as a hit is.
if [ "$pointers" -gt 0 ] 2>/dev/null; then
    report=$(hw_field "$route" text) || report=
else
    report=
fi

injected=false
if [ "$pointers" -gt 0 ] 2>/dev/null && [ -n "$report" ]; then
    injected=true
fi

hw_shadow_log

if [ "$injected" = true ]; then
    printf 'Headwater routed this task to the documents that govern it, before you open a file.\n'
    printf 'These are pointers, not content. Open the ones that bear on the task.\n\n'
    printf '%s\n' "$report"
fi
exit 0
