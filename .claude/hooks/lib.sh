#!/bin/sh
# What the four harness hooks share, and nothing else.
#
# A hook is a place where a harness hands control to this engine and takes it
# back. The contract is in spec 5. This file holds the two things all four
# hooks need and no rule of any kind: where the engine is, and how to read one
# field off the JSON object the harness puts on standard input.
#
# Every function here fails open. A hook that cannot find the engine, or cannot
# read its input, returns nothing and lets the harness proceed. The commit hook
# and the CI job are downstream of all three, so a hook that failed closed would
# stop work that the position below it already holds.
#
# Nothing here runs an interpreter, and no position of these hooks depends on
# one. `sh` and the built engine are the whole of what a session needs.
# HW-DR-0055 is the ruling, and HW-OBL-0146 is what it discharged: the review
# position could not read its re-entry guard on a machine with no `python3`, so
# it stopped the same turn twice with no way out. The one dependency left is the
# engine itself, which every position already required for the verb it calls.

# The repository these hooks answer for. The harness sets `CLAUDE_PROJECT_DIR`,
# and a fixture run sets `HEADWATER_HOOK_ROOT` instead so that a test can point
# a hook at a corpus that is not this one.
hw_root=${HEADWATER_HOOK_ROOT:-${CLAUDE_PROJECT_DIR:-$PWD}}

# How many entries of one `governs` pattern the edit-time advisory lists
# before it prints the pattern and its count alone. The owner ruled on
# 2026-09-25, on #1093: "list up to about 20 files, then print the pattern and
# the count, so the editor sees the size without a long list to read."
hw_governs_listed_at_most=20

# The built engine, or empty when there is none.
#
# Two profiles build one. `release` is what CI builds and what a release
# artifact ships. `dev-release` is the same optimization level without the
# `lto = true` and `codegen-units = 1` link, and that link is single threaded:
# measured on an eight core host, a one crate relink costs 153 seconds of CPU
# under `release` and 25 under `dev-release`. This function read the first path
# and no other, so every worktree that wanted a hook or the commit gate bought
# the expensive link for a binary each position runs for a fifth of a second.
#
# Both count, and the newer one answers. A rule that preferred `release` by name
# would let a stale one shadow a `dev-release` binary built minutes later, which
# is a hook reading a change through an engine that predates it. That defect is
# quieter than the cost this removes, so the rule is the file system's answer
# rather than a ranking of the two profiles.
#
# An exact tie goes to `release`, because `-nt` is false on equality and
# `release` is assigned first. Nothing rests on that: two builds a second apart
# carry different times on any file system this runs on, and a tie means the two
# binaries are equally current. It is stated because a reader who has to know
# which one ran should not have to derive it from an operator.
#
# A hook that has already found its engine can pin it in `hw_engine_pin`, and
# every read below then uses that binary whatever `hw_root` names afterwards.
# `intent.sh` does, because it corrects `hw_root` to the payload's `cwd` after
# the engine is found, and a worktree with no build of its own then made every
# later read fail and the hook exit before its shadow-log line (#917).
hw_engine() {
    if [ -n "${hw_engine_pin:-}" ] && [ -x "$hw_engine_pin" ]; then
        printf '%s' "$hw_engine_pin"
        return 0
    fi
    _release="$hw_root/engine/target/release/headwater"
    _dev="$hw_root/engine/target/dev-release/headwater"
    engine=
    [ -x "$_release" ] && engine=$_release
    if [ -x "$_dev" ] && { [ -z "$engine" ] || [ "$_dev" -nt "$engine" ]; }; then
        engine=$_dev
    fi
    [ -n "$engine" ] || return 1
    printf '%s' "$engine"
}

# One top-level or one nested field of the JSON object on standard input, as a
# bare string. `hw_field "$input" tool_input file_path` reads `.tool_input
# .file_path`. It prints nothing when the field is absent or the input will not
# parse, and the caller treats that as "the harness said nothing".
#
# The engine reads it. It parses JSON already, because JSON is a subset of the
# YAML 1.2 core schema its loader implements, and `headwater json` is the verb
# that hands one member of it back.
# [HW-DR-0055](../../docs/decisions/0055-a-hook-reads-a-wire-format-through-the-engine-and-not-through-an-interpreter.md)
# is the ruling, and it states why an interpreter, a `jq`, and an input form on
# a corpus verb were each refused. A host with no built engine reads no field,
# which is the same silence a host with no `python3` produced until then, and
# every caller below treats it that way on purpose.
hw_field() {
    _json=$1
    shift
    _engine=$(hw_engine) || return 1
    printf '%s' "$_json" | "$_engine" json field "$@" 2>/dev/null
}

# The corpus-relative correction of `hw_root`, read from the payload rather
# than from an environment variable, once a hook has captured `$input` and can
# call `hw_field` at all.
#
# `CLAUDE_PROJECT_DIR` names the checkout a session started in and never
# moves: the harness's own docs put it plainly — "`${CLAUDE_PROJECT_DIR}`
# stays put... `cwd` follows Claude" (https://code.claude.com/docs/en/worktrees.md,
# "Hook paths don't follow the worktree"). Every session in this repository
# runs from a worktree, by the rule at the top of `CLAUDE.md`, so the line
# above this one hands every such hook a `hw_root` that names the main
# checkout regardless of which worktree is actually being checked. The
# payload's own `cwd` member names the worktree correctly, and this is the one
# place that reads it.
#
# The chicken-and-egg is real and this is the answer to it. Reading `cwd`
# takes `hw_field`, `hw_field` takes an engine, and an engine is found through
# `hw_root`. Locating a binary to answer `headwater json field` is not a
# corpus-relative act, so it does not matter which checkout's binary answers
# it. So a caller reads `cwd` through whatever engine the unmodified `hw_root`
# already resolves to, and only the value this function returns — never the
# read that produced it — is corpus-relative. Every JSON-only read a hook
# still needs (a session id, an event name, a prompt) is best taken before
# calling this, through the pre-correction root, so a worktree with no engine
# of its own loses only the corpus-relative answers and not the plain parse of
# its own input.
#
# `HEADWATER_HOOK_ROOT` keeps winning regardless. A fixture sets it to point a
# hook at a corpus that is not this one on purpose, and a session's own `cwd`
# is not grounds to override what a test asked for.
#
# Empty input, no `cwd` member, no engine to read it with, a `cwd` in no work
# tree, a work tree with no taxonomy lock: this returns
# `hw_root` unchanged, the same fail-open answer every function in this file
# gives. That is the wrong checkout in exactly the case this function exists
# to fix, and the checkout every caller already had before that case was
# noticed — a caller loses nothing it did not already lack.
hw_resolve_root() {
    if [ -n "$HEADWATER_HOOK_ROOT" ]; then
        printf '%s' "$hw_root"
        return 0
    fi
    _cwd=$(hw_field "$1" cwd) && [ -n "$_cwd" ] || {
        printf '%s' "$hw_root"
        return 0
    }
    # The harness moves `cwd` whenever a `cd` persists, so a session doing
    # engine work hands a hook `engine/` or deeper, where no
    # `.headwater/taxonomy.lock` sits and every corpus verb fails (#917). The
    # top of the work tree that holds `cwd` is the repository.
    #
    # A `cwd` outside every work tree, or a work tree with no taxonomy lock,
    # names no corpus, so this returns `hw_root` for it. The parent session of
    # every `/next-run` works from a run directory under the git common dir,
    # where `--show-toplevel` fails. Until #917 this returned that directory as
    # it came, `route` found no lock there, and the hook exited with no output
    # and no shadow-log line.
    _top=$(git -C "$_cwd" rev-parse --show-toplevel 2>/dev/null) && [ -n "$_top" ] && [ -f "$_top/.headwater/taxonomy.lock" ] || {
        printf '%s' "$hw_root"
        return 0
    }
    printf '%s' "$_top"
}

# How many elements the array or the object at that path holds.
#
# An empty array and an absent member give a caller the same nothing back
# through `hw_field`, and they are different facts about the message. The
# routing report is read on exactly that difference.
hw_count() {
    _json=$1
    shift
    _engine=$(hw_engine) || return 1
    printf '%s' "$_json" | "$_engine" json count "$@" 2>/dev/null
}

# A JSON string literal, so a hook can put a path or a report into the object it
# writes back without breaking it.
hw_quote() {
    _engine=$(hw_engine) || return 1
    printf '%s' "$1" | "$_engine" json quote 2>/dev/null
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
# The record is staged in a file beside the log, named `.<pid>.part`, so a
# reader that counts `*.jsonl` files never counts it. Any failure returns 1
# and writes nothing on either stream: the brace groups carry the redirect
# for the reason `intent.sh`'s header gives.
hw_append_line() {
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

# The one path an `apply_patch` tool call names, for a harness that hands this
# hook a unified-diff-style command instead of the `file_path` field the other
# two harnesses pass. Codex's own edit tool is `apply_patch`, and its
# `tool_input` carries the patch text under `command` rather than a path
# (spec 16's C4 row: "exit 2 with the reason on standard error", the same
# document, names no `file_path`). A patch's own header lines say what it
# touches: `*** Add File: <path>`, `*** Update File: <path>`, `*** Delete
# File: <path>`. This reads the first one and stops.
#
# Only the first path a patch names reaches the check below. A patch that
# creates several files in one call holds this hook to the first of them, and
# the commit gate holds the rest, the same as it holds every write a `Bash`
# call makes that no matcher here ever sees.
# The git common dir of `hw_root`, absolute. Every worktree of one clone
# answers the same path, which is where state that has to reach every
# worktree of a clone belongs — `tools/run/run-dir.sh` puts `headwater-run`
# there for the same reason, and `.claude/hooks/touch.sh` and `review.sh` put
# `headwater-session` there.
#
# Empty on any failure: no git, no common dir, or a relative answer this
# cannot resolve against `hw_root`. A caller that gets nothing back skips
# whatever it meant to read or write there, the same as every other fail-open
# read in this file.
hw_common_dir() {
    _common=$(git -C "$hw_root" rev-parse --git-common-dir 2>/dev/null) || return 1
    [ -n "$_common" ] || return 1
    case $_common in
        /*) ;;
        *) _common="$hw_root/$_common" ;;
    esac
    printf '%s' "$_common"
}

hw_patch_path() {
    _patch=$(hw_field "$1" tool_input command) || return 1
    _first=$(printf '%s\n' "$_patch" |
        sed -n -e 's/^\*\*\* Add File: *//p' \
            -e 's/^\*\*\* Update File: *//p' \
            -e 's/^\*\*\* Delete File: *//p' |
        head -n 1 |
        sed -e 's/[[:space:]]*$//')
    [ -n "$_first" ] || return 1
    printf '%s' "$_first"
}

# The pointers of one `headwater route --json` document whose evidence is `by`
# (`anchor` or `terms`), one line each, as `path (name) — summary`, or nothing
# and a non-zero status when none is. Every value is a member of the JSON the
# engine wrote, read through `headwater json`, so no line of rendered text is
# picked apart. The em dash here is the hook's own separator. The rendered
# report folds a pointer across lines, and a grep for the dash in it missed a
# pointer whose fold put the summary on the next line (HW-OBL-0149, #953).
hw_route_pointers() {
    _json=$1 _by=$2
    _total=$(hw_count "$_json" pointers) || return 1
    _lines= _at=0
    while [ "$_at" -lt "$_total" ]; do
        _evidence=$(hw_field "$_json" pointers "$_at" evidence by) || _evidence=
        if [ "$_evidence" = "$_by" ]; then
            _path=$(hw_field "$_json" pointers "$_at" path) || _path=
            _name=$(hw_field "$_json" pointers "$_at" name) || _name=
            _summary=$(hw_field "$_json" pointers "$_at" summary) || _summary=
            _lines="$_lines
  $_path ($_name) — $_summary"
        fi
        _at=$((_at + 1))
    done
    [ -n "$_lines" ] || return 1
    printf '%s\n' "${_lines#?}"
}

# The pointer lines of every document that declares it governs one path, or
# nothing and a non-zero status when none does. The read position and the
# pre-edit position both say the same set, and this is the one place that asks
# for it, so the two cannot drift into two answers (#953).
#
# It asks `headwater route --json` for the path, the verb that ships, and keeps
# the pointers whose evidence is `anchor`: a `governs` edge admits the path. A
# pointer the route reached by terms is not the set this answers for.
hw_governing_pointers() {
    _engine=$(hw_engine) || return 1
    _route=$("$_engine" route --json --root "$hw_root" "$1" 2>/dev/null) || return 1
    hw_route_pointers "$_route" anchor
}

# One line naming every governing edge of one path that went suspect, or
# nothing and a non-zero status when none did. An edge is suspect when it
# records a `verified_revision` and the revision its target has now differs
# (#953). The engine decides that with the comparison `headwater check` makes,
# and writes it as the `suspect` member of an anchored pointer's evidence, so
# this reads that member and compares nothing itself.
hw_suspect_edges() {
    _engine=$(hw_engine) || return 1
    _route=$("$_engine" route --json --root "$hw_root" "$1" 2>/dev/null) || return 1
    _total=$(hw_count "$_route" pointers) || return 1
    _edges= _n=0 _at=0
    while [ "$_at" -lt "$_total" ]; do
        _count=$(hw_count "$_route" pointers "$_at" evidence suspect) || _count=0
        _path=$(hw_field "$_route" pointers "$_at" path) || _path=
        _e=0
        while [ "$_e" -lt "$_count" ]; do
            _target=$(hw_field "$_route" pointers "$_at" evidence suspect "$_e" target) || _target=
            _verified=$(hw_field "$_route" pointers "$_at" evidence suspect "$_e" verified) || _verified=
            _current=$(hw_field "$_route" pointers "$_at" evidence suspect "$_e" current) || _current=
            _edges="$_edges, the edge of $_path onto $_target (verified at $_verified, now at $_current)"
            _n=$((_n + 1))
            _e=$((_e + 1))
        done
        _at=$((_at + 1))
    done
    [ "$_n" -gt 0 ] || return 1
    case $_n in 1) _noun='edge' _it='it' ;; *) _noun='edges' _it='each one' ;; esac
    printf 'Headwater: this edit left %s governing %s suspect: %s. `headwater check` reports %s under relation.target.suspect.\n' \
        "$_n" "$_noun" "${_edges#, }" "$_it"
}

# The engine's account of one path that the governed scope admits and that no
# document governs, or nothing and a non-zero status for any other path (#953).
# It is the block `headwater route` writes under the line naming the path, and
# the pointer lines the same route reached by terms, where there are any. The
# engine composes every line, the front-matter lines included, so the hook
# holds no scope pattern and no relation name. The block is read from the
# report the JSON carries as `text`, by the engine's own sentence with the path
# in it. The pointers are read from the JSON members, never by an em dash.
hw_ungoverned_in_scope() {
    _engine=$(hw_engine) || return 1
    _route=$("$_engine" route --json --root "$hw_root" "$1" 2>/dev/null) || return 1
    _text=$(hw_field "$_route" text) || return 1
    _block=$(printf '%s\n' "$_text" | awk -v head="  $1 is in the governed scope, and nothing governs it" '
        $0 == head { on = 1; print; next }
        on && /^    / { print; next }
        { on = 0 }')
    [ -n "$_block" ] || return 1
    printf '%s\n' "$_block"
    _reached=$(hw_route_pointers "$_route" terms) || return 0
    printf '\nThe route reached these documents by the terms of the path, and none of them declares the edge:\n%s\n' "$_reached"
}

# The reverse of the set above: what one document governs, and which documents
# declare an edge onto it. It prints two lists, each under a heading line that
# carries its count, or nothing and a non-zero status when both lists are
# empty. A path that is not a document gets nothing, because `explain` refuses
# a path outside every corpus root and a file that does not exist yet (#1008).
#
# Every fact comes from `headwater explain --json` through `headwater json`,
# one element of `related` at a time, and no line of rendered text is read.
# An outbound entry counts only where its relation is `governs`, and every
# inbound entry counts, whatever its relation. `related` of a heavily cited
# specification part holds twenty entries or more, and the list names each.
#
# A `governs` pattern is named by the entries it matched, which the engine
# writes as `reach.members[i].paths`, so the hook never matches a glob itself
# (HW-DR-0074 gives a path one matcher). A member whose only entry is its own
# pattern is a literal and prints as one line. Any other member prints the
# pattern and its count, then each entry under it, up to
# `hw_governs_listed_at_most`. Past that bound it prints the pattern and the
# count and no entry. The heading count is the number of distinct entries the
# document's patterns reach, a pattern past the bound counting every entry it
# matched and an entry two edges reach counting once, so the count is the
# size the editor has to check and not the number of lines. An edge with no
# `reach`, an anchor that binds nothing or a document target, prints its
# `targets` as written.
hw_governed_by_document() {
    _engine=$(hw_engine) || return 1
    _explain=$("$_engine" explain --json --root "$hw_root" "$1" 2>/dev/null) || return 1
    _total=$(hw_count "$_explain" related) || return 1
    _governs= _governs_n=0 _summed=0 _cites= _cites_n=0 _at=0
    while [ "$_at" -lt "$_total" ]; do
        _relation=$(hw_field "$_explain" related "$_at" relation) || _relation=
        _inbound=$(hw_field "$_explain" related "$_at" inbound) || _inbound=
        _members=0
        if [ "$_inbound:$_relation" = false:governs ]; then
            _members=$(hw_count "$_explain" related "$_at" reach members) || _members=0
        fi
        if [ "$_members" -gt 0 ]; then
            _m=0
            while [ "$_m" -lt "$_members" ]; do
                _pattern=$(hw_field "$_explain" related "$_at" reach members "$_m" pattern) || _pattern=
                _matched=$(hw_field "$_explain" related "$_at" reach members "$_m" matched) || _matched=0
                _m=$((_m + 1))
                [ -n "$_pattern" ] || continue
                _first=
                if [ "$_matched" -eq 1 ]; then
                    _first=$(hw_field "$_explain" related "$_at" reach members "$((_m - 1))" paths 0) || _first=
                fi
                _files=files
                [ "$_matched" -eq 1 ] && _files=file
                if [ "$_matched" -eq 1 ] && [ "$_first" = "$_pattern" ]; then
                    _governs="$_governs
  $_pattern"
                elif [ "$_matched" -le "$hw_governs_listed_at_most" ]; then
                    _governs="$_governs
  $_pattern ($_matched $_files):"
                    _p=0
                    while [ "$_p" -lt "$_matched" ]; do
                        _path=$(hw_field "$_explain" related "$_at" reach members "$((_m - 1))" paths "$_p") || _path=
                        _p=$((_p + 1))
                        [ -n "$_path" ] || continue
                        _governs="$_governs
    $_path"
                    done
                else
                    _governs="$_governs
  $_pattern ($_matched $_files, not listed past $hw_governs_listed_at_most)"
                fi
                _summed=$((_summed + _matched))
            done
            _at=$((_at + 1))
            continue
        fi
        # `targets` holds one member per target as written. `target` joins
        # them with `, `, and a member may hold a comma itself (#1092).
        _count=$(hw_count "$_explain" related "$_at" targets) || _count=0
        _t=0
        while [ "$_t" -lt "$_count" ]; do
            _target=$(hw_field "$_explain" related "$_at" targets "$_t") || _target=
            _t=$((_t + 1))
            [ -n "$_target" ] || continue
            case $_inbound:$_relation in
                false:governs)
                    _governs="$_governs
  $_target"
                    _governs_n=$((_governs_n + 1)) ;;
                true:*)
                    _cites="$_cites
  $_target ($_relation)"
                    _cites_n=$((_cites_n + 1)) ;;
            esac
        done
        _at=$((_at + 1))
    done
    # Two edges can reach one entry, a literal page and a glob over its
    # directory, so the entries a pattern reached are counted as the engine's
    # union, `governed_entries`, and never as the sum of `matched` (#1093).
    # The sum stands in only where an older engine writes no union.
    if [ "$_summed" -gt 0 ]; then
        _union=$(hw_field "$_explain" governed_entries) || _union=$_summed
        _governs_n=$((_governs_n + _union))
    fi
    [ "$_governs_n" -gt 0 ] || [ "$_cites_n" -gt 0 ] || return 1
    if [ "$_governs_n" -gt 0 ]; then
        printf 'It governs these code paths (%s):%s\n' "$_governs_n" "$_governs"
        [ "$_cites_n" -gt 0 ] && printf '\n'
    fi
    if [ "$_cites_n" -gt 0 ]; then
        printf 'These documents declare an edge onto it (%s):%s\n' "$_cites_n" "$_cites"
    fi
    return 0
}

# The engine's account of one path that a producer writes, or nothing and a
# non-zero status when the path is not one (#1053). It prints two lines. The
# first is the heading `headwater derived` writes over the path: the row, the
# treatment and the reason, as `<row> — takes `<treatment>`, because <why>`.
# The second is the command that rebuilds the path, and it is empty where no
# producer rebuilds it.
#
# The report is read from `headwater derived`, an existing verb that takes no
# path and has no `--json`, so this reads the rendered text over the whole tree
# and picks out the one path. It reads only the section under the line `N paths
# hold a record whose shape decides its merge`, and it matches a path line
# exactly: `    <path>` or `    <path> — rerun with `<cmd>``. That is the
# precedent `hw_ungoverned_in_scope` sets for `route` text. The engine pushes
# one `format!` per line there and writes no color with no terminal, so no
# line is folded. HW-OBL-0149 and #1058 are the caution: a reader of rendered
# text breaks when the engine folds a line, and a fixture holds the shape.
#
# The exit status of the verb is ignored. It exits 1 on a disagreement and
# outside a git repository, and the report it prints is valid in both cases.
hw_derived_report() {
    _engine=$(hw_engine) || return 1
    _derived=$("$_engine" derived --root "$hw_root" 2>/dev/null)
    [ -n "$_derived" ] || return 1
    _found=$(printf '%s\n' "$_derived" | awk -v path="$1" '
        /^[0-9]+ paths hold a record whose shape decides its merge$/ { on = 1; next }
        !on { next }
        /^  [^ ].* — takes `/ { head = substr($0, 3); next }
        /^    / {
            line = substr($0, 5)
            if (line == path) { print head; print ""; exit }
            prefix = path " — rerun with `"
            if (index(line, prefix) == 1 && substr(line, length(line)) == "`") {
                cmd = substr(line, length(prefix) + 1)
                print head
                print substr(cmd, 1, length(cmd) - 1)
                exit
            }
            next
        }
        /^[^ ]/ { head = "" }')
    [ -n "$_found" ] || return 1
    printf '%s\n' "$_found"
}

# Whether `hw_root` is stopped part way through a merge, a rebase, a
# cherry-pick or a revert, which is the state a conflict leaves (#1053). It
# prints the operation's name, or nothing and a non-zero status.
#
# The git dir is the one of this worktree and not `hw_common_dir`: git writes
# `MERGE_HEAD` and the rebase directories per worktree, so another worktree's
# merge is not this one's. The test is one `rev-parse` and five `test -e`
# calls, and it calls no engine, so a caller can make it on every call.
hw_merge_state() {
    _gitdir=$(git -C "$hw_root" rev-parse --git-dir 2>/dev/null) || return 1
    [ -n "$_gitdir" ] || return 1
    case $_gitdir in
        /*) ;;
        *) _gitdir="$hw_root/$_gitdir" ;;
    esac
    if [ -e "$_gitdir/MERGE_HEAD" ]; then printf 'merge'
    elif [ -e "$_gitdir/rebase-merge" ] || [ -e "$_gitdir/rebase-apply" ]; then printf 'rebase'
    elif [ -e "$_gitdir/CHERRY_PICK_HEAD" ]; then printf 'cherry-pick'
    elif [ -e "$_gitdir/REVERT_HEAD" ]; then printf 'revert'
    else return 1
    fi
}

# The paths git reports as unmerged in `hw_root`, one per line, relative to it.
hw_unmerged_paths() {
    git -C "$hw_root" diff --name-only --diff-filter=U 2>/dev/null
}
