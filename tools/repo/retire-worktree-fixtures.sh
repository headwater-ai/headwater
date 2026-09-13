#!/bin/sh
# What holds `tools/repo/retire-worktree.sh`, the sweep that retires a finished tree.
#
# The tool deletes directories and branches, so what has to be held is mostly
# what it refuses. Each of its five guards is provoked below, and so is the
# content check on both sides, because a sweep that retired everything would be
# `rm -rf` with a longer name and a sweep that retired nothing would be a
# comment. The two arms that matter most are the squash-merge arm, where
# ancestry says unmerged and the pull request says merged, and the in-use arm,
# where a live process is the only thing standing between the sweep and a
# directory somebody is working in.
#
# Run it from anywhere:
#     sh tools/repo/retire-worktree-fixtures.sh
#
# It needs `git` and nothing else. It builds its own repository under a
# temporary directory, it stands in for `gh` through
# `HEADWATER_RETIRE_PR_STATE`, and it writes nothing under this checkout.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/repo/retire-worktree.sh"

# A tool this suite cannot execute makes every case below read an empty
# report, and an empty report fails all 37 assertions below.
# That reads as a broken tool rather than as a path this file got wrong,
# which is what it was when `tools/` was grouped by subject under the suite.
[ -x "$tool" ] || {
    printf 'no tool at %s, so no case below can run.\n' "$tool" >&2
    exit 1
}

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0

pass() {
    passed=$((passed + 1))
    printf '  ok   %s\n' "$1"
}

fail() {
    failed=$((failed + 1))
    printf '  FAIL %s\n       %s\n' "$1" "$2"
}

same() {
    name=$1 want=$2 got=$3
    if [ "$want" = "$got" ]; then
        pass "$name"
    else
        fail "$name" "expected \`$want\`, got \`$got\`"
    fi
}

# One line of the report for a path or a branch, without the leading verb.
verdict() {
    printf '%s\n' "$report" | awk -v needle="$1" '
        index($0, needle) { print $1; exit }
    '
}

# --- a repository with a remote, because the sweep reads `origin/main` --------

upstream="$scratch/upstream.git"
clone="$scratch/clone"
git init --quiet --bare --initial-branch=main "$upstream"
git clone --quiet "$upstream" "$clone" 2>/dev/null
cd "$clone" || exit 1
git config user.email fixture@example.invalid
git config user.name 'Retirement Fixture'
git config commit.gpgsign false
echo 'one' >file.txt
git add file.txt
git commit --quiet -m 'the first commit'
git push --quiet -u origin main 2>/dev/null

# The squash-merge shape: a branch writes two commits, `main` gains one new
# commit carrying the same content under a different identifier, and no commit
# of the branch is an ancestor of `main` afterwards. This is the normal case in
# the repository the tool serves, and the case ancestry alone gets wrong.
git checkout --quiet -b squashed
echo 'two' >>file.txt
git commit --quiet -am 'branch work, first half'
echo 'three' >>file.txt
git commit --quiet -am 'branch work, second half'
squashed_head=$(git rev-parse HEAD)
git checkout --quiet main
git merge --quiet --squash squashed >/dev/null 2>&1
git commit --quiet -m 'the squashed result (#1)'
git push --quiet origin main 2>/dev/null

printf '# the shape the tool exists for\n'
git merge-base --is-ancestor squashed origin/main 2>/dev/null && ancestry=merged || ancestry=unmerged
same 'ancestry calls the squash-merged branch unmerged' 'unmerged' "$ancestry"

# --- the pull request record `gh` would return --------------------------------

state="$scratch/pr-state"
printf 'squashed\tMERGED\t%s\n' "$squashed_head" >"$state"
printf 'open-work\tOPEN\t0000000000000000000000000000000000000000\n' >>"$state"
export HEADWATER_RETIRE_PR_STATE="$state"

# --- the trees ----------------------------------------------------------------

trees="$scratch/trees"
mkdir -p "$trees"

git worktree add --quiet "$trees/merged" squashed 2>/dev/null

git worktree add --quiet "$trees/dirty" -b dirty-work origin/main 2>/dev/null
echo 'uncommitted' >"$trees/dirty/scratch.txt"

git worktree add --quiet "$trees/locked" -b locked-work origin/main 2>/dev/null
git worktree lock "$trees/locked" 2>/dev/null

git worktree add --quiet "$trees/unmerged" -b unmerged-work origin/main 2>/dev/null
cd "$trees/unmerged" || exit 1
echo 'nobody else has this' >>file.txt
git commit --quiet -am 'work that reached no merge'
cd "$clone" || exit 1

git worktree add --quiet "$trees/open" -b open-work origin/main 2>/dev/null
cd "$trees/open" || exit 1
echo 'in flight' >>file.txt
git commit --quiet -am 'work under review'
cd "$clone" || exit 1

# The in-use tree carries a commit and a merged pull request of its own, so
# that what clears it once its sitter exits is the arm the tool exists for and
# not an emptiness. A tree sitting on `origin/main` with nothing committed to it
# is the fresh case below, and the sweep now keeps it.
git worktree add --quiet "$trees/in-use" -b in-use-work origin/main 2>/dev/null
cd "$trees/in-use" || exit 1
echo 'work an in-use tree carried' >>file.txt
git commit --quiet -am 'work that a pull request merged'
printf 'in-use-work\tMERGED\t%s\n' "$(git rev-parse HEAD)" >>"$state"
cd "$clone" || exit 1

# A tree made the way every build agent makes one, and not yet committed to. By
# ancestry it is indistinguishable from a tree whose work landed: it sits at the
# tip of `origin/main` from the second it exists. The sweep deleted such a tree,
# and its branch with it, while the agent that made it was still working.
git worktree add --quiet "$trees/fresh" -b fresh-work origin/main 2>/dev/null

# A branch nobody checked out, sitting exactly on `origin/main`. This is what a
# harness leaves when it makes a worktree for a session that commits nothing,
# and six of them were found in one real sweep.
git branch placeholder origin/main

# A branch nobody checked out that holds work no merge carried.
git branch survivor squashed
cd "$clone" || exit 1
git checkout --quiet -b survivor-tip survivor 2>/dev/null
echo 'written after the merge' >>file.txt
git commit --quiet -am 'a commit written after the merge'
git branch --quiet -f survivor HEAD
git checkout --quiet main
git branch --quiet -D survivor-tip

# --- the report arm: it must retire nothing -----------------------------------

printf '\n# the default is a report, because the action is destructive\n'
before_trees=$(git worktree list | wc -l | tr -d ' ')
report=$(sh "$tool" 2>&1)
after_trees=$(git worktree list | wc -l | tr -d ' ')
same 'a run with no flag removes no worktree' "$before_trees" "$after_trees"
same '  and says plainly that it retired nothing' 1 \
    "$(printf '%s\n' "$report" | grep -c '^RETIRED: nothing')"
same '  and still names what it would retire' 1 \
    "$(printf '%s\n' "$report" | grep -c "^WOULD    retire $trees/merged")"

# --- the guards ---------------------------------------------------------------

printf '\n# each guard, on the report the same run produced\n'
same 'the squash-merged tree is cleared by its pull request' 'WOULD' "$(verdict "$trees/merged")"
same '  and the reason names the merged head rather than ancestry' 1 \
    "$(printf '%s\n' "$report" | grep -F "$trees/merged " \
        | grep -c 'inside the merged head of its pull request')"
same 'a tree with an uncommitted change is kept' 'KEPT' "$(verdict "$trees/dirty")"
same 'a locked tree is kept' 'KEPT' "$(verdict "$trees/locked")"
same '  and the lock is given as the reason' 1 \
    "$(printf '%s\n' "$report" | grep -c 'locked, which is a deliberate hold')"
same 'a tree holding unmerged work is kept' 'KEPT' "$(verdict "$trees/unmerged")"
same '  and the report says how many commits it holds' 1 \
    "$(printf '%s\n' "$report" | grep -c 'unmerged, holding 1 commit')"
same 'a tree whose pull request is open is kept' 'KEPT' "$(verdict "$trees/open")"
same 'a tree holding no commit origin/main lacks is kept' 'KEPT' "$(verdict "$trees/fresh")"
same '  and the reason names the emptiness rather than ancestry' 1 \
    "$(printf '%s\n' "$report" | grep -F "$trees/fresh " \
        | grep -c 'holds no commit origin/main lacks, so it is a tree in flight rather than a tree whose work landed')"
same '  and no line of the report proposes retiring it' 0 \
    "$(printf '%s\n' "$report" | grep -F "$trees/fresh " | grep -c '^WOULD')"
same 'a branch sitting on origin/main is retirable' 'WOULD' "$(verdict placeholder)"
same 'a branch holding work no merge carried is kept' 'KEPT' "$(verdict survivor)"
same 'main is never a candidate' 0 \
    "$(printf '%s\n' "$report" | awk '$2 == "main" || $3 == "main"' | wc -l | tr -d ' ')"

# --- the in-use guard ---------------------------------------------------------
#
# This is the guard with a cost behind it. A removed worktree that a process
# still sits in keeps the directory pinned on disk, and the next `cargo test`
# there exits 101 with a `NotFound` naming the binary under test, which reads as
# a defect in the code rather than as a directory somebody deleted.

printf '\n# the in-use guard, with a real process inside the tree\n'
if [ -d /proc/1 ]; then
    cd "$trees/in-use" || exit 1
    # Every descriptor is closed, because a background process that inherits the
    # pipe of a `$(...)` below keeps that substitution waiting for it to exit.
    sleep 30 >/dev/null 2>&1 </dev/null &
    sitter=$!
    cd "$clone" || exit 1
    report=$(sh "$tool" 2>&1)
    same 'a tree a live process sits in is kept' 'KEPT' "$(verdict "$trees/in-use")"
    same '  and the live process is given as the reason' 1 \
        "$(printf '%s\n' "$report" | grep -c 'a live process has its working directory there')"
    kill "$sitter" 2>/dev/null
    wait "$sitter" 2>/dev/null
    report=$(sh "$tool" 2>&1)
    same '  and once it exits the tree is retirable again' 'WOULD' "$(verdict "$trees/in-use")"
else
    printf '  skip no /proc, so the in-use guard cannot be provoked here\n'
fi

# --- the retiring arm ---------------------------------------------------------

printf '\n# --retire acts, and only on what the report cleared\n'
report=$(sh "$tool" --retire 2>&1)
same 'the merged tree is gone' 0 "$(test -d "$trees/merged" && echo 1 || echo 0)"
same '  and it is reported as retired' 'RETIRED' "$(verdict "$trees/merged")"
same '  and the reason is still its merged pull request, not ancestry' 1 \
    "$(printf '%s\n' "$report" | grep -F "$trees/merged " \
        | grep -c 'inside the merged head of its pull request')"
same 'the dirty tree survives' 1 "$(test -d "$trees/dirty" && echo 1 || echo 0)"
same 'the locked tree survives' 1 "$(test -d "$trees/locked" && echo 1 || echo 0)"
same 'the unmerged tree survives' 1 "$(test -d "$trees/unmerged" && echo 1 || echo 0)"
same 'the open pull request tree survives' 1 "$(test -d "$trees/open" && echo 1 || echo 0)"
same 'the fresh tree survives' 1 "$(test -d "$trees/fresh" && echo 1 || echo 0)"
same '  and its branch survives with it' 1 \
    "$(git show-ref --verify --quiet refs/heads/fresh-work && echo 1 || echo 0)"
same 'the placeholder branch is gone' 0 \
    "$(git show-ref --verify --quiet refs/heads/placeholder && echo 1 || echo 0)"
same 'the survivor branch is still here' 1 \
    "$(git show-ref --verify --quiet refs/heads/survivor && echo 1 || echo 0)"
same 'the branch of the retired tree went with it' 0 \
    "$(git show-ref --verify --quiet refs/heads/squashed && echo 1 || echo 0)"
same 'main is still here' 1 \
    "$(git show-ref --verify --quiet refs/heads/main && echo 1 || echo 0)"

printf '\n# running it twice changes nothing more\n'
second=$(sh "$tool" --retire 2>&1)
same 'the second run retires no worktree' 1 \
    "$(printf '%s\n' "$second" | grep -c '^RETIRED: 0 worktree')"

# --- the sweep never removes the tree it runs in ------------------------------

printf '\n# the sweep never removes the tree it is running in\n'
git worktree add --quiet "$trees/self" squashed-again 2>/dev/null \
    || git worktree add --quiet -b squashed-again "$trees/self" "$squashed_head" 2>/dev/null
printf 'squashed-again\tMERGED\t%s\n' "$squashed_head" >>"$state"
cd "$trees/self" || exit 1
report=$(sh "$tool" --retire 2>&1)
cd "$clone" || exit 1
same 'a tree that would otherwise be retired is kept when it is the caller' 1 \
    "$(test -d "$trees/self" && echo 1 || echo 0)"
same '  and the reason says the sweep is inside it' 1 \
    "$(printf '%s\n' "$report" | grep -c 'the sweep is running inside it')"

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
