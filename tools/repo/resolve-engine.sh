#!/bin/sh
# The one home for "which built engine answers": prefer the newer of
# `engine/target/release/headwater` and `engine/target/dev-release/headwater`
# under a given root, and report only when neither exists.
#
# #647 found this same four-line check written out nine times, independently,
# across `.claude/` and `tools/`, and three more consumers that carried only
# the `release` half and so reported "no engine" against a worktree built the
# way the build order tells a session to build it. This file is the copy a
# new or fixed consumer sources instead of writing a tenth, so the check has
# one place to be right and nine fewer places to quietly disagree.
#
# `release` is what CI builds and what a release artifact ships; `dev-release`
# is the same optimization level without the `lto = true` and
# `codegen-units = 1` link, which is single threaded and costs minutes for a
# binary a hook or a fixture suite runs for a fifth of a second. Either counts,
# and the newer file answers, so a `dev-release`-only worktree is found rather
# than silently skipped. An exact tie goes to `release`, because a POSIX `-nt`
# test is false on equality and this checks `release` first; nothing rests on
# that, since two builds a second apart carry different mtimes on any file
# system this runs on.
#
# `.githooks/pre-commit`, `.githooks/fixtures.sh` and the sibling scripts
# beside them under `.githooks/` are the one exception, and they do not source
# this file. Each carries its own copy of the same four lines, and each says
# why on itself: the git-hook half of this repository must not depend on a
# file outside `.githooks/`, because git runs a hook from whatever tree
# `core.hooksPath` names and a clone that has fetched a hook body but not this
# file would run a hook it cannot resolve. `tools/repo/engine-resolver-fixtures.sh`
# holds that exception by name rather than letting the guard it carries fault
# them for the same duplication it exists to end elsewhere.
#
# Usage, from a caller that already knows its own root:
#
#     . "$root/tools/repo/resolve-engine.sh"
#     if engine=$(hw_resolve_engine_bin "$root"); then
#         : # $engine is the executable to run
#     else
#         hw_resolve_engine_missing_message "$root" >&2
#         exit 1
#     fi
#
# The function takes the root as an argument rather than reading it off an
# environment variable, on purpose: sourcing this file sets no variable and
# leaves no state a child process could inherit. That is what lets
# `.claude/hooks/fixtures-live.sh` source it even though it must not source
# `.claude/hooks/lib.sh` — that file's `hw_engine` reads `HEADWATER_HOOK_ROOT`,
# and every harness process `fixtures-live.sh` drives is a child that would
# inherit it and read the real repository in place of the scratch clone it was
# handed.
hw_resolve_engine_bin() {
    _hw_resolve_root=$1
    _hw_resolve_release="$_hw_resolve_root/engine/target/release/headwater"
    _hw_resolve_dev="$_hw_resolve_root/engine/target/dev-release/headwater"
    _hw_resolve_engine=
    [ -x "$_hw_resolve_release" ] && _hw_resolve_engine=$_hw_resolve_release
    if [ -x "$_hw_resolve_dev" ] && { [ -z "$_hw_resolve_engine" ] || [ "$_hw_resolve_dev" -nt "$_hw_resolve_engine" ]; }; then
        _hw_resolve_engine=$_hw_resolve_dev
    fi
    [ -n "$_hw_resolve_engine" ] || return 1
    printf '%s' "$_hw_resolve_engine"
}

# The message every fixed consumer prints when neither profile exists. It
# names both paths this function looked in, which is the distinction the
# issue asked for: a message that named one path could not tell a reader
# apart from a message that had simply not looked at the other one.
hw_resolve_engine_missing_message() {
    _hw_resolve_root=$1
    printf 'no engine at %s/engine/target/release/headwater or %s/engine/target/dev-release/headwater\n' \
        "$_hw_resolve_root" "$_hw_resolve_root"
    printf '  build one:\n'
    printf '        cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked\n'
    printf '  that is the cheap profile. Build --release when you want the shipped artifact:\n'
    printf '        cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml --locked\n'
}
