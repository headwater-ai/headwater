#!/bin/sh
# What holds `tools/repo/fix-fixed-point.sh`, the CI step that keeps this tree
# a fixed point of `headwater check --fix`.
#
# Run it from anywhere:
#     sh tools/repo/fix-fixed-point-fixtures.sh
#
# It needs `git` and nothing else. A stub stands in for the engine: it writes
# only when it is called as `check --fix`, and what it writes is chosen by
# `STUB_WRITE`. Each case builds its own repository under a temporary
# directory, so nothing here writes under this checkout.
#
# The case that matters most is the dirty one. A tracked file that is already
# modified before the run, and that `--fix` then modifies again, reads ` M` in
# `git status --porcelain` on both sides. The first version of the tool
# compared that listing and passed this case with "wrote nothing"
# (verification of PR #1245, 2026-09-27).

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/repo/fix-fixed-point.sh"

[ -f "$tool" ] || {
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

# The stub engine. It refuses to write unless it was asked for `check --fix`,
# so a tool that dropped `--fix` passes no writing case below.
stub="$scratch/headwater"
cat > "$stub" <<'STUB'
#!/bin/sh
[ "${1:-}" = check ] || exit 3
fix=no
for argument in "$@"; do
    [ "$argument" = --fix ] && fix=yes
done
[ "$fix" = yes ] || exit 0
case ${STUB_WRITE:-none} in
    none) ;;
    clean) printf 'fixed\n' >> docs/clean.md; echo 'headwater: fixed docs/clean.md (1 patch)' >&2 ;;
    dirty) printf 'fixed\n' >> docs/dirty.md; echo 'headwater: fixed docs/dirty.md (1 patch)' >&2 ;;
    new) mkdir -p .headwater/ids && printf 'claim\n' > .headwater/ids/X-1 ;;
    report) printf 'rewritten\n' >> report.txt ;;
    refuse) echo 'headwater: refused docs/clean.md' >&2; exit 1 ;;
esac
exit 0
STUB
chmod +x "$stub"

# A repository with one clean tracked file, one tracked file already modified,
# and one untracked report file, which is the shape CI hands the step.
repository() {
    dir="$scratch/$1"
    mkdir -p "$dir/docs"
    git -C "$dir" init -q
    printf 'clean\n' > "$dir/docs/clean.md"
    printf 'dirty\n' > "$dir/docs/dirty.md"
    git -C "$dir" add docs
    git -C "$dir" -c user.name=f -c user.email=f@f commit -q -m base
    printf 'edited before the run\n' >> "$dir/docs/dirty.md"
    printf 'a report\n' > "$dir/report.txt"
    printf '%s\n' "$dir"
}

# case <name> <STUB_WRITE> <expected exit> <text the output must carry>
case_() {
    dir=$(repository "$1")
    out=$(cd "$dir" && STUB_WRITE=$2 sh "$tool" "$stub" --now 2026-09-27 2>&1)
    status=$?
    if [ "$status" -ne "$3" ]; then
        fail "$1" "exit $status, expected $3: $out"
    elif ! printf '%s\n' "$out" | grep -qF -- "$4"; then
        fail "$1" "no '$4' in: $out"
    else
        pass "$1"
    fi
}

echo 'a run that writes nothing passes, over a tree that is already dirty'
case_ 'nothing written' none 0 'wrote nothing'

echo 'every write fails the step, and the output names it'
case_ 'a clean tracked file written' clean 1 'docs/clean.md'
case_ 'a tracked file written that was already modified' dirty 1 'docs/dirty.md'
case_ 'a new untracked file created' new 1 '.headwater/ids/X-1'
case_ 'an untracked file that was there before written' report 1 'report.txt'
case_ 'a refused patch' refuse 1 'exited 1'

echo 'the snapshot leaves the real index alone'
dir=$(repository index)
before=$(git -C "$dir" status --porcelain)
(cd "$dir" && STUB_WRITE=none sh "$tool" "$stub" > /dev/null 2>&1)
after=$(git -C "$dir" status --porcelain)
if [ "$before" = "$after" ]; then
    pass 'git status reads the same after a run'
else
    fail 'git status reads the same after a run' "before: $before / after: $after"
fi

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
