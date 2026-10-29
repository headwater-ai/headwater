#!/bin/sh
# What this repository adds to the hooks that `headwater init --harness`
# writes, and nothing an adopter receives.
#
# The six scripts beside this file are the shipped bytes (#1871): this
# repository runs the copy it ships, and `headwater init --harness --check`
# holds them. The shipped `lib.sh` reads this file last, when it exists, so
# that this repository keeps three behaviors the shipped layer drops without an
# edit to a shipped script:
#
# - `hw_engine` finds the engine this checkout built, under `engine/target/`,
#   because no `headwater` is on the `PATH` of a session here.
# - `hw_local_intent` writes the shadow-mode routing log after each route,
#   which HW-DR-0064 keeps outside the adopter population.
# - `hw_local_review` runs `.githooks/pre-commit` at the end of a turn in place
#   of the shipped check, with the exemption of a build-order parent.
#
# `.claude/hooks/fixtures.sh` holds all three, through the shipped scripts.

# The built engine, or empty when there is none.
#
# Two profiles build one, `release` and `dev-release`, and the newer one
# answers. `tools/repo/resolve-engine.sh` is the one home of that rule and says
# why on itself, and this function delegates to it. Before #990 this function
# carried its own copy of the comparison, one of thirteen such copies.
#
# The resolver is found beside this file, at
# `$(dirname "$0")/../../tools/repo/resolve-engine.sh`, because every hook
# sources this file as `$(dirname "$0")/lib.sh`. It is not found under
# `$hw_root` first, because `hw_root` names the corpus a hook answers for, and a
# fixture points it at a scratch tree that has no `tools/` at all. `$hw_root` is
# the fallback for a caller that sources this file from somewhere else. A `.`
# of a missing file exits a non-interactive POSIX shell, and this file fails
# open, so each source is guarded: with no resolver, `hw_engine` returns 1 and
# prints nothing.
#
# The function still sets the global `engine` to what it printed, or to empty,
# as the copy it replaced did.
#
# A hook that has already found its engine can pin it in `hw_engine_pin`, and
# every read below then uses that binary whatever `hw_root` names afterwards.
# `intent.sh` does, because it corrects `hw_root` to the payload's `cwd` after
# the engine is found, and a worktree with no build of its own then made every
# later read fail and the hook exit before its shadow-log line (#917).
for _hw_resolver in "$hw_hooks/../../tools/repo/resolve-engine.sh" "$hw_root/tools/repo/resolve-engine.sh"; do
    if [ -r "$_hw_resolver" ]; then
        . "$_hw_resolver"
        break
    fi
done
unset _hw_resolver

hw_engine() {
    if [ -n "${hw_engine_pin:-}" ] && [ -x "$hw_engine_pin" ]; then
        printf '%s' "$hw_engine_pin"
        return 0
    fi
    engine=
    command -v hw_resolve_engine_bin >/dev/null 2>&1 || return 1
    engine=$(hw_resolve_engine_bin "$hw_root") || {
        engine=
        return 1
    }
    printf '%s' "$engine"
}

# Append one line to a log file in one `write()`, so a second writer of the
# same file can land before the line or after it and never inside it (#927).
# A `write()` to a file opened for appending moves to the end and writes in
# one step, and Linux holds the file for the whole call. `printf '%s\n'` is
# two calls in dash, the line and then the newline, and bash writes in 4 KiB
# pieces, so neither shell's `printf` is used for the append. `dd` with one
# block the size of the whole record reads it in one `read()` and writes it
# in one `write()`, in every shell, because POSIX requires `dd` to write each
# input block as one output block when `bs=` is given.
#
# `conv=notrunc` is required, and not a default spelled out. The `dd` of
# uutils coreutils 0.8.0, which is `/usr/bin/dd` on this repository's host,
# calls `ftruncate` on its standard output after the write, at the offset it
# wrote to. A line another writer appended in between is cut off, and the
# file can then hold zero bytes where that line was (measured 2026-09-29: 12
# hooks at once left 3 lines). With `conv=notrunc` it makes the one `write()`
# and nothing else.
#
# A file that does not end in a newline holds a line whose writer died inside
# its write. The record then starts with a newline, so the torn line stays one
# bad line and this line stays whole. The torn line is kept as it is, because
# nothing here rewrites what a file already holds.
#
# The check of the last byte and the write are one step under an exclusive
# `flock` on the log file, held until the write returns. Without the lock the
# check races another writer's `write()`: a read can see the file grow before
# that write ends, so the last byte is not yet its newline, and this writer
# adds a newline the file does not need. The result is an empty line, which a
# strict JSONL reader refuses and `wc -l` counts (the verify of PR #1390
# measured 2 of 40 rounds with 40KB lines and 1 of 100 with 2KB lines). A
# `write()` that holds the lock ends before the next writer reads, so the next
# writer reads a complete line. The wait is at most 2 seconds. After that, or
# on a host with no `flock`, the line is written without the lock, because a
# line with a rare empty line before it is worth more than no line.
#
# The record is staged in a file beside the log, named `.<pid>.part`, so a
# reader that counts `*.jsonl` files never counts it. Any failure returns 1
# and writes nothing on either stream: the brace groups carry the redirect
# for the reason `intent.sh`'s header gives.
hw_append_line() {
    if command -v flock >/dev/null 2>&1; then
        { flock -w 2 9 || :; _hw_append_now "$1" "$2"; } 2>/dev/null 9>>"$1" || return 1
        return 0
    fi
    _hw_append_now "$1" "$2"
}

# The check and the write of `hw_append_line`, with no lock of their own.
_hw_append_now() {
    _hal_file=$1
    _hal_record=$2
    if [ -s "$_hal_file" ] && [ -n "$({ tail -c 1 "$_hal_file"; } 2>/dev/null)" ]; then
        _hal_record="
$_hal_record"
    fi
    _hal_part="$_hal_file.$$.part"
    if ! { printf '%s\n' "$_hal_record" > "$_hal_part"; } 2>/dev/null; then
        rm -f "$_hal_part" 2>/dev/null
        return 1
    fi
    _hal_size=$(wc -c < "$_hal_part" 2>/dev/null) || _hal_size=0
    _hal_size=$((_hal_size))
    _hal_status=1
    if [ "$_hal_size" -gt 0 ] && { dd if="$_hal_part" bs="$_hal_size" count=1 conv=notrunc >> "$_hal_file"; } 2>/dev/null; then
        _hal_status=0
    fi
    rm -f "$_hal_part" 2>/dev/null
    return "$_hal_status"
}

# ## The shadow-mode routing log
#
# [HW-DR-0064](../../docs/decisions/0064-q64-whether-intent-time-routing-gains-an-offline-embedding-path-in-shadow-mode.md)
# rules that this hook, and nothing else, keeps a private record of what the
# deterministic route decided, so a later comparison against a second, meaning
# based lookup has something to compare against. [#819] carries the build in
# four steps; this file carries step 1 alone, the half that needs no model and
# no vector: the task, the route document as `route` wrote it, and whether the
# pointers it offered reached the agent.
#
# `hw_shadow_log`, below, is the whole of it. It runs once the engine is found,
# the input parses and the prompt is not an envelope, on every exit after that
# point: a `route` that fails, or a route document with no `pointers` count,
# writes a line with `"route":null` and a `"skip"` member naming the reason
# (`route_failed`, `no_pointers_count`), and a routed prompt writes
# `"skip":null`. Until #917 those two exits were silent and wrote nothing, so a
# person prompt they ended could not be told from one the hook never saw. The
# exits before that point stay unlogged: with no engine the input cannot be
# read (HW-DR-0055), and HW-DR-0064 puts that checkout outside the population.
# So a corpus this cannot log for is a corpus this hook still routes
# for exactly as before it existed: the function only ever adds a write to a
# file nobody reads back here, and never a line of output or a changed exit
# status. It fails open the same way every read in this file already does,
# and silently, because a write error surfacing on either stream would corrupt
# what the harness injects, and exit 2 would block the prompt. Exit 2 is the
# one status that blocks a `UserPromptSubmit` hook in Claude Code, and any
# other non-zero status is shown as an error (HW-OBL-0203).
#
# The file lives at `<git common dir>/headwater-shadow-log/<session>.jsonl`,
# beside the marks `touch.sh` and `review.sh` already keep under the common
# dir rather than under the tree: every worktree of one clone answers the same
# path for it, `repo-cleanup` removing one worktree never touches it, and
# `headwater generate` and `headwater check` never read it because it sits
# outside the corpus entirely. One file per harness session keeps the ten
# worktrees this host may run at once out of each other's files. It does not
# give a file one writer: session 17deb7c5 wrote two lines for each of its
# prompts within one second, from two runs of this hook (#927). So each line
# goes to the file in one `write()`, which a second writer cannot split, and
# `hw_append_line` in `lib.sh` is where that is done and why `printf` is not.
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
#
# The shipped `intent.sh` calls `hw_local_intent` once on every exit after the
# route is asked, with `input`, `engine`, `task`, `route`, `skip`, `injected`
# and `hw_root` set, and discards both of its streams.

# One line, appended to the file the header above names. Every value it
# cannot get — no session id, no common git dir, no writable directory, no
# quotable string — ends this function at that point and leaves nothing
# written, the same fail-open posture as `hw_field` and `hw_count`. It
# never touches standard output or standard error, and it never sets an exit
# status the caller reads, because the caller has already decided both by the
# time this runs. Defined ahead of the input it reads, because a shell script
# executes top to bottom and a call to a function defined after the point
# that calls it finds nothing there yet — `foo: not found`, on standard
# error, which is exactly the leak this whole file exists to not have.
hw_local_intent() {
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
    # The brace group matters for the same reason as the write at the bottom:
    # a skipped route can reach here from a root with no lock, and a `<` that
    # fails to open reports before a `2>/dev/null` after it takes effect.
    _lock=$({ "$engine" json field lock digest < "$hw_root/.headwater/taxonomy.lock"; } 2>/dev/null) || _lock=
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
    # A skipped prompt carries no route document, and `null` says so rather
    # than a quoted empty string a reader could take for one.
    if [ -n "${skip:-}" ]; then
        _route_q=null
        _skip_q=$(hw_quote "$skip") || return 0
    else
        _route_q=$(hw_quote "$route") || return 0
        _skip_q=null
    fi
    _version_q=$(hw_quote "$_version") || _version_q='""'
    _lock_q=$(hw_quote "$_lock") || _lock_q='""'

    _line=$(printf '{"at":"%s","session":%s,"prompt_id":%s,"probe_session":%s,"corpus_root":%s,"engine_version":%s,"lock_digest":%s,"task":%s,"injected":%s,"skip":%s,"route":%s%s}' \
        "$_at" "$_session_q" "$_prompt_id_q" "$_probe_session_q" "$_root_q" "$_version_q" "$_lock_q" "$_task_q" "$injected" "$_skip_q" "$_route_q" "$_embedding") || return 0

    # One `write()` for the line and its newline, through `hw_append_line` in
    # `lib.sh`, which says why `printf` is not one (#927). Its redirects sit
    # on brace groups for the reason the header above gives: a bare
    # `... >> "$_file" 2>/dev/null` still leaks "cannot create" to the real
    # standard error when the `>>` itself is what fails, because a shell sets
    # redirections up in the order written.
    hw_append_line "$_file" "$_line"
    return 0
}

# The review at the end of a turn, as this repository runs it: the commit
# gate, `.githooks/pre-commit`, and nothing else. A second script that ran
# `headwater check --strict` with a second opinion about which findings block
# would be two live paths over one question. So what blocks a commit here is
# what blocks a turn, by construction rather than by agreement. The shipped
# `review.sh` has already read `stop_hook_active` and ended a turn it blocked
# once, and it exits with the status this returns.
hw_local_review() {
    # The parent of a build-order run edits nothing on this tree and owns no
    # checkout: the integrator it dispatches owns the main one, and reads this same
    # gate at its own commits. Run against the parent, this position read the
    # integrator's half-finished rebuild as red nine times in one run and woke the
    # parent each time at the price of its whole context (20 million input tokens
    # over nine turns, run 20260911-1331). `tools/run/run-dir.sh start` records the
    # parent's session under the run directory, and a session named there ends its
    # turn without the gate. A session id is never reused, so a marker a dead run
    # left behind names nobody, and a run started outside the harness writes none.
    session=$(hw_field "$input" session_id)

    # From here on, every use of `hw_root` is corpus-relative: the git common dir
    # below, and the gate path and `cd` after it. The two reads above it are not —
    # `stop_hook_active` and `session_id` are read off the payload, not off a
    # checkout — so they run first, through whichever engine the pre-correction
    # root already resolves to, and lose nothing on a worktree with no engine of
    # its own.
    hw_root=$(hw_resolve_root "$input")

    if [ -n "$session" ]; then
        common=$(hw_common_dir) || common=
        if [ -n "$common" ]; then
            # One line per parent session: a parent that `tools/run/supervise.sh`
            # restarts appends its own with `run-dir.sh session`, and each of them
            # is exempt, not only the first.
            for marker in "$common"/headwater-run/*/parent.session; do
                [ -f "$marker" ] || continue
                while IFS= read -r prefix || [ -n "$prefix" ]; do
                    [ -n "$prefix" ] || continue
                    case $session in "$prefix"*) return 0 ;; esac
                done < "$marker"
            done

            # A session that never called a tool `.claude/hooks/touch.sh` marks
            # could not have introduced whatever this tree already fails. Gating
            # its Stop anyway reports somebody else's unfinished work: every
            # session that has not fixed `core.hooksPath` back to a relative
            # value after `EnterWorktree` shares the main checkout's tree with
            # whatever else is running there, and a session that only reads,
            # searches or talks inherits every failure already sitting on it.
            # That is not a hypothetical — a session with zero mutating tool
            # calls was stopped by a stale `site/` figure and a stale
            # `taxonomy.lock` hash that a different, unrelated branch's edits had
            # left on the checkout it ran against.
            #
            # `touch.sh` marks `Write`, `Edit` and Codex's `apply_patch`, and not
            # `Bash`, so a session whose only mutation ran through a shell command
            # is not exempted here and still gates normally — the safe direction,
            # since the alternative is skipping a session that did change the
            # tree. A session this exempts wrongly is not lost: the commit hook
            # and CI hold the same tree at the next real gate, which is the whole
            # reason this position is allowed to be wrong in the permissive
            # direction rather than the refusing one. The file above states it:
            # "this position only makes the finding arrive sooner."
            [ -e "$common/headwater-session/$session/touched" ] || return 0
        fi
    fi

    gate="$hw_root/.githooks/pre-commit"
    [ -x "$gate" ] || return 0

    report=$(cd "$hw_root" && "$gate" 2>&1)
    status=$?
    [ "$status" -eq 0 ] && return 0

    message="The commit gate, run at the end of this turn rather than at the next commit.
$report"

    if [ -n "${COPILOT_CLI:-}" ]; then
        quoted=$(hw_quote "$message") || return 0
        printf '{"decision":"block","reason":%s}\n' "$quoted"
        return 0
    fi

    printf '%s\n' "$message" >&2
    return 2
}
