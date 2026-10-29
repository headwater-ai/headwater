#!/bin/sh
# What the Headwater hooks share, and no rule of any kind: where the binary
# is, how to read one field of the JSON object the harness puts on standard
# input, and the reads of `headwater route`, `explain` and `derived` that more
# than one hook makes.
#
# Every function here fails open. A hook that cannot find the binary, or
# cannot read its input, prints nothing and lets the harness go on. The commit
# and the pull request still hold whatever the hook did not see.
#
# No interpreter is needed. The hooks call `sh`, `git`, `awk`, `sed`, `cat`
# and the `headwater` binary, and the binary is what reads JSON.
#
# A repository can extend the hooks without an edit to them, because an edit
# makes `headwater init --harness` refuse the file. When `lib.local.sh` sits
# beside this file, it is read last. It can define `hw_engine` to find a
# binary that is not on the `PATH`, `hw_local_intent` to run after each route,
# and `hw_local_review` to replace the review at the end of a turn.

# The directory that holds the hooks. Every hook reads this file as
# `${0%/*}/lib.sh`, so `$0` names the hook.
case $0 in
    */*) hw_hooks=${0%/*} ;;
    *) hw_hooks=. ;;
esac

# The repository these hooks answer for. The harness sets
# `CLAUDE_PROJECT_DIR`, and a test sets `HEADWATER_HOOK_ROOT` to point a hook
# at another corpus.
hw_root=${HEADWATER_HOOK_ROOT:-${CLAUDE_PROJECT_DIR:-$PWD}}

# How many entries of one `governs` pattern the edit-time advisory lists
# before it prints the pattern and its count alone.
hw_governs_listed_at_most=20

# The `headwater` binary, or nothing and a non-zero status when there is none.
#
# A hook that has already found its binary can pin it in `hw_engine_pin`, and
# every read below then uses that binary.
hw_engine() {
    if [ -n "${hw_engine_pin:-}" ] && [ -x "$hw_engine_pin" ]; then
        printf '%s' "$hw_engine_pin"
        return 0
    fi
    engine=$(command -v headwater 2>/dev/null) && [ -n "$engine" ] || {
        engine=
        return 1
    }
    printf '%s' "$engine"
}

# One top-level or one nested field of the JSON object on standard input, as a
# bare string. `hw_field "$input" tool_input file_path` reads `.tool_input
# .file_path`. It prints nothing when the field is absent or the input will
# not parse, and the caller reads that as "the harness said nothing".
hw_field() {
    _json=$1
    shift
    _engine=$(hw_engine) || return 1
    printf '%s' "$_json" | "$_engine" json field "$@" 2>/dev/null
}

# The repository, read from the `cwd` member of the payload.
#
# `CLAUDE_PROJECT_DIR` names the checkout a session started in and never
# moves, and `cwd` follows the session into a worktree or a subdirectory. So
# the top of the work tree that holds `cwd` is the repository, where that tree
# holds a taxonomy lock. Read every payload field that is not about the
# checkout before this call.
#
# `HEADWATER_HOOK_ROOT` always wins. Any failure returns `hw_root` unchanged.
hw_resolve_root() {
    if [ -n "${HEADWATER_HOOK_ROOT:-}" ]; then
        printf '%s' "$hw_root"
        return 0
    fi
    _cwd=$(hw_field "$1" cwd) && [ -n "$_cwd" ] || {
        printf '%s' "$hw_root"
        return 0
    }
    _top=$(git -C "$_cwd" rev-parse --show-toplevel 2>/dev/null) && [ -n "$_top" ] && [ -f "$_top/.headwater/taxonomy.lock" ] || {
        printf '%s' "$hw_root"
        return 0
    }
    printf '%s' "$_top"
}

# How many elements the array or the object at that path holds. An empty
# array and an absent member are different facts, and `hw_field` gives the
# same nothing for both.
hw_count() {
    _json=$1
    shift
    _engine=$(hw_engine) || return 1
    printf '%s' "$_json" | "$_engine" json count "$@" 2>/dev/null
}

# A JSON string literal, so a hook can put a path or a report into the object
# it writes back without breaking it.
hw_quote() {
    _engine=$(hw_engine) || return 1
    printf '%s' "$1" | "$_engine" json quote 2>/dev/null
}

# The git common dir of `hw_root`, absolute. Every worktree of one clone
# answers the same path, so state that a session leaves for a later hook goes
# there. Empty, with a non-zero status, on any failure.
hw_common_dir() {
    _common=$(git -C "$hw_root" rev-parse --git-common-dir 2>/dev/null) || return 1
    [ -n "$_common" ] || return 1
    case $_common in
        /*) ;;
        *) _common="$hw_root/$_common" ;;
    esac
    printf '%s' "$_common"
}

# The first path an `apply_patch` call names. Codex passes the patch text
# under `tool_input.command` and no `file_path`, and the header lines of the
# patch name the files: `*** Add File: <path>`, `*** Update File: <path>` and
# `*** Delete File: <path>`. Only the first path reaches the hooks, and the
# commit holds the rest.
hw_patch_path() {
    _patch=$(hw_field "$1" tool_input command) || return 1
    _first=$(printf '%s\n' "$_patch" |
        sed -n -e 's/^\*\*\* Add File: *//p' \
            -e 's/^\*\*\* Update File: *//p' \
            -e 's/^\*\*\* Delete File: *//p' |
        sed -n -e '1s/[[:space:]]*$//p')
    [ -n "$_first" ] || return 1
    printf '%s' "$_first"
}

# The pointers of one `headwater route --json` document whose evidence is
# `by` (`anchor` or `terms`), one line each, as `path (name) — summary`, or
# nothing and a non-zero status when none is. Every value is a member of the
# JSON, so no line of rendered text is picked apart.
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
# nothing and a non-zero status when none does. The read hook and the write
# hook both ask this one function.
hw_governing_pointers() {
    _engine=$(hw_engine) || return 1
    _route=$("$_engine" route --json --root "$hw_root" "$1" 2>/dev/null) || return 1
    hw_route_pointers "$_route" anchor
}

# One line naming every governing edge of one path that went suspect, or
# nothing and a non-zero status when none did. An edge is suspect when it
# records a `verified_revision` and its target has another revision now. The
# binary decides that, with the comparison `headwater check` makes.
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

# The account of one path that the governed scope admits and that no document
# governs, or nothing and a non-zero status for any other path. It is the
# block `headwater route` writes under the line naming the path, and the
# pointers the same route reached by terms, where there are any. The binary
# writes every line, so the hook holds no scope pattern and no relation name.
hw_ungoverned_in_scope() {
    _engine=$(hw_engine) || return 1
    _route=$("$_engine" route --json --root "$hw_root" "$1" 2>/dev/null) || return 1
    _text=$(hw_field "$_route" text) || return 1
    # The route was asked about one path, so the one head line is the block,
    # whatever path it names: an edit through a symlink is named by the file
    # the link reaches.
    _block=$(printf '%s\n' "$_text" | awk '
        /^  [^ ].* is in the governed scope, and nothing governs it$/ { on = 1; print; next }
        on && /^    / { print; next }
        { on = 0 }')
    [ -n "$_block" ] || return 1
    printf '%s\n' "$_block"
    _reached=$(hw_route_pointers "$_route" terms) || return 0
    printf '\nThe route reached these documents by the terms of the path, and none of them declares the edge:\n%s\n' "$_reached"
}

# What one document governs, and which documents declare an edge onto it, as
# two lists, each under a heading that carries its count. Nothing, and a
# non-zero status, when both are empty or the path is not a document.
#
# Every fact comes from `headwater explain --json`. An outbound entry counts
# where its relation is `governs`, and every inbound entry counts. A pattern
# is named by the entries it matched, which the binary writes as
# `reach.members[i].paths`, so the hook matches no glob itself. A pattern
# past `hw_governs_listed_at_most` entries prints its count and no entry. The
# heading count is the number of distinct entries, which the binary writes as
# `governed_entries`.
hw_governed_by_document() {
    _engine=$(hw_engine) || return 1
    _explain=$("$_engine" explain --json --paths-at-most "$hw_governs_listed_at_most" --root "$hw_root" "$1" 2>/dev/null) || return 1
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
        # `targets` holds one member per target as written, and a member may
        # hold a comma itself.
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
    # Two edges can reach one entry, so the count is the union the binary
    # writes, and the sum stands in only where a binary writes no union.
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

# The account of one path that a producer writes, or nothing and a non-zero
# status when the path is not one. It prints two lines. The first is the
# heading `headwater derived` writes over the path: the row, the treatment and
# the reason. The second is the command that rebuilds the path, and it is
# empty where no producer rebuilds it.
#
# `headwater derived` takes no path and has no `--json`, so this reads the
# section under the line `N paths hold a record whose shape decides its
# merge` and matches a path line exactly. The exit status of the verb is
# ignored, because the report is valid when it exits 1.
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
# cherry-pick or a revert. It prints the name of the operation, or nothing and
# a non-zero status. Git keeps that state for each worktree, so this reads the
# git dir of this worktree and not the common dir.
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

# The paths git reports as unmerged in `hw_root`, one per line.
hw_unmerged_paths() {
    git -C "$hw_root" diff --name-only --diff-filter=U 2>/dev/null
}

# The file of the paths one session wrote, under the git common dir. The
# touch hook appends to it, and the review hook reads it.
hw_touched_file() {
    _common=$(hw_common_dir) || return 1
    printf '%s' "$_common/headwater-session/$1/touched"
}

# The extension of this repository, read last so that it can replace any
# function above.
if [ -r "$hw_hooks/lib.local.sh" ]; then
    . "$hw_hooks/lib.local.sh"
fi
# installed by headwater init --harness, digest sha256:ec40adeb7bb8303e3180ffecba36580ee099092a73f2f9700563413d7ec10c6a
