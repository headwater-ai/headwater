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
hw_engine() {
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
# Empty input, no `cwd` member, no engine to read it with: this returns
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
    printf '%s' "$_cwd"
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
