#!/bin/sh
# What a build owes before it opens its pull request, in one call and in the
# order `.claude/agents/hw-build.md` states: merge `origin/main`, rebuild the
# engine, regenerate, re-bless and run the workspace suite, the format gate,
# the commit gate, and the fixture suites CI runs over the tree.
#
#     sh tools/run/pre-pr.sh <issue|none>     in the builder's own worktree, on its branch
#
# The sequence was eight commands in the definition, and each builder typed
# them in its own order and skipped the one it forgot: seven of ten vetoes in
# one run were a Format, Lint or unblessed-fixture failure, and the one
# workspace-wide run a build owes was the step most often left to the
# verifier. The order never varies, so it is a script, and a step cannot be
# skipped. Clippy is not here: `tools/engine/clippy.sh` runs CI's pinned
# toolchain once per build, and the definition says when.
#
# It stops at the first failed step, prints the block with that step named
# and the log it wrote, and exits 1; the builder repairs and runs it again.
# The fixture suites run as a set, and the block names every one that failed.
# Most take seconds; `tools/probe/probe-record-fixtures.sh` takes minutes and
# needs `bwrap`, and it is here because PR #1894 printed `PRE-PR: ok` and
# failed 9 of its cases in CI. The two oracle suites are here because #1902
# printed `PRE-PR: ok` and went red in CI on two `oracle-fixtures.sh` cases. Exit 0 when every step passed. Exit 2 on a
# refused argument, on the main checkout, and on `main` itself.
#
#     PRE-PR: ok | failed at <step>
#     MERGED: origin/main at <sha>, <n> commits brought in | conflict on <files>
#     BUILT: <engine>
#     GENERATED: ok
#     SUITE: pass | fail
#     FMT: ok | fail
#     CHECK: ok | fail
#     FIXTURES: ok | failed: <names>
#     MOVED: <file>          one per line: what the regenerate and the bless changed, for the builder to read and commit
#     LOGS: <dir>            one log per step
#
# The logs go under `$CLAUDE_JOB_DIR/tmp/issue-<N>/pre-pr/`, so the tree is
# left with nothing but the files the regenerate moved; `HEADWATER_SCRATCH`
# overrides that root, and a shell outside the harness gets a temporary
# directory. Every cargo command goes through `tools/hw-cargo`, as the run
# policy requires, and the engine is resolved as every other consumer
# resolves it, by `tools/repo/resolve-engine.sh`. For a fixture:
# `HEADWATER_ENGINE` names the engine, `HW_CARGO` the wrapper and
# `HW_PRE_PR_FIXTURES` the fixture list.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
cargo=${HW_CARGO:-$root/tools/hw-cargo}
fixtures=${HW_PRE_PR_FIXTURES:-".claude/agents/fixtures.sh .claude/skills/fixtures.sh .claude/hooks/fixtures.sh tools/repo/developing-fixtures.sh tools/repo/engine-resolver-fixtures.sh tools/repo/collation-fixtures.sh tools/engine/build-declaration-fixtures.sh tools/probe/probe-record-fixtures.sh tools/probe/oracle-fixtures.sh tools/probe/probe-oracle-fixtures.sh"}

usage() {
    sed -n '/^#     sh tools\/run\/pre-pr.sh/p' "$0" | sed 's/^# *//' >&2
    exit 2
}

[ $# -eq 1 ] || usage
case $1 in
    none) tag=run-lessons ;;
    '' | *[!0-9]*) echo "pre-pr: \`$1\` is not an issue number or \`none\`." >&2; exit 2 ;;
    *) tag="issue-$1" ;;
esac

tree=$(pwd -P)
gitdir=$(git rev-parse --absolute-git-dir 2>/dev/null) || { echo "pre-pr: not inside a git worktree." >&2; exit 2; }
common=$(cd "$(git rev-parse --git-common-dir)" 2>/dev/null && pwd -P)
gitdir=$(cd "$gitdir" && pwd -P)
if [ "$gitdir" = "$common" ]; then
    echo "pre-pr: this is the main checkout, and a build never runs here. Run it in the worktree the harness made for you." >&2
    exit 2
fi
branch=$(git symbolic-ref --short -q HEAD)
if [ -z "$branch" ] || [ "$branch" = main ]; then
    echo "pre-pr: the tree is on \`${branch:-a detached HEAD}\`, and a build runs on its own branch." >&2
    exit 2
fi

if [ -n "${HEADWATER_SCRATCH:-}" ]; then
    logs="$HEADWATER_SCRATCH/$tag/pre-pr"
elif [ -n "${CLAUDE_JOB_DIR:-}" ]; then
    logs="$CLAUDE_JOB_DIR/tmp/$tag/pre-pr"
else
    logs=$(mktemp -d) || exit 1
fi
mkdir -p "$logs" || exit 1

merged='not reached' built='not reached' generated='not reached' suite='not reached' fmt='not reached' check='not reached' fixt='not reached'

# finish <ok|failed at step>: the block, then the exit.
finish() {
    printf 'PRE-PR: %s\n' "$1"
    printf 'MERGED: %s\nBUILT: %s\nGENERATED: %s\nSUITE: %s\nFMT: %s\nCHECK: %s\nFIXTURES: %s\n' \
        "$merged" "$built" "$generated" "$suite" "$fmt" "$check" "$fixt"
    git status --porcelain 2>/dev/null | sed 's/^...//' | while IFS= read -r file; do
        printf 'MOVED: %s\n' "$file"
    done
    printf 'LOGS: %s\n' "$logs"
    case $1 in ok) exit 0 ;; *) exit 1 ;; esac
}

# --- merge -------------------------------------------------------------------
git fetch -q origin > "$logs/fetch.log" 2>&1 || { merged="fetch failed ($logs/fetch.log)"; finish 'failed at fetch'; }
before=$(git rev-parse HEAD)
if ! git merge --no-edit origin/main > "$logs/merge.log" 2>&1; then
    conflicts=$(git diff --name-only --diff-filter=U 2>/dev/null | tr '\n' ' ')
    git merge --abort > /dev/null 2>&1
    merged="conflict on ${conflicts:-see $logs/merge.log}"
    finish 'failed at merge'
fi
brought=$(git rev-list --count "$before..origin/main" 2>/dev/null || echo '?')
merged="origin/main at $(git rev-parse --short origin/main), $brought commits brought in"

# --- build -------------------------------------------------------------------
if ! sh "$cargo" build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked > "$logs/build.log" 2>&1; then
    built="fail ($logs/build.log)"
    finish 'failed at build'
fi
if [ -n "${HEADWATER_ENGINE:-}" ]; then
    engine=$HEADWATER_ENGINE
else
    . "$root/tools/repo/resolve-engine.sh"
    if ! engine=$(hw_resolve_engine_bin "$tree"); then
        built="built, and no engine resolves under $tree/engine/target"
        finish 'failed at build'
    fi
fi
built=$engine

# --- generate ----------------------------------------------------------------
if ! "$engine" generate --root "$tree" > "$logs/generate.log" 2>&1; then
    generated="fail ($logs/generate.log)"
    finish 'failed at generate'
fi
generated=ok

# --- the suite, blessing the recorded fixtures it moves ----------------------
if ! HEADWATER_BLESS=1 sh "$cargo" test --workspace --no-fail-fast --manifest-path engine/Cargo.toml > "$logs/suite.log" 2>&1; then
    suite="fail ($logs/suite.log)"
    finish 'failed at suite'
fi
suite=pass

# --- format ------------------------------------------------------------------
if ! sh "$cargo" fmt --manifest-path engine/Cargo.toml --all -- --check > "$logs/fmt.log" 2>&1; then
    fmt="fail ($logs/fmt.log)"
    finish 'failed at fmt'
fi
fmt=ok

# --- the commit gate: what pre-push and pre-commit refuse ---------------------
if ! "$engine" generate --check --root "$tree" > "$logs/generate-check.log" 2>&1; then
    check="fail, generate --check ($logs/generate-check.log)"
    finish 'failed at check'
fi
if ! "$engine" check --strict --root "$tree" > "$logs/check.log" 2>&1; then
    check="fail, check --strict ($logs/check.log)"
    finish 'failed at check'
fi
check=ok

# --- the fixture suites CI runs over the tree ---------------------------------
failed=''
for f in $fixtures; do
    [ -f "$tree/$f" ] || continue
    name=$(printf '%s' "$f" | tr '/' '-')
    sh "$tree/$f" > "$logs/$name.log" 2>&1 || failed="$failed $f"
done
if [ -n "$failed" ]; then
    fixt="failed:$failed"
    finish 'failed at fixtures'
fi
fixt=ok

finish ok
