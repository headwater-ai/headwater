#!/bin/sh
# What the three harness hooks share, and nothing else.
#
# A hook is a place where a harness hands control to this engine and takes it
# back. The contract is in spec 5. This file holds the two things all three
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
