#!/bin/sh
# What holds `tools/repo/new-worktree.sh`: it fetches before it adds, either
# `git worktree add` shape passes through unchanged, the engine builds
# through `tools/hw-cargo` inside the new tree, and a tree with no
# `engine/Cargo.toml` is refused rather than silently left unbuilt.
#
# Run it from anywhere:
#     sh tools/repo/new-worktree-fixtures.sh
#
# It needs no toolchain: every scratch repo carries a fake `tools/hw-cargo`
# that logs its own arguments and plants an empty binary, so this suite holds
# the sequence the tool runs, not what building an engine produces. Every
# repo is a real `git init` under `mktemp -d`, and nothing inside this
# checkout is written.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/repo/new-worktree.sh"

if [ ! -f "$tool" ]; then
    echo "no script at \`tools/repo/new-worktree.sh\`." >&2
    exit 1
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

origin="$scratch/origin"
mkdir -p "$origin/tools" "$origin/engine"
git init -q "$origin"
git -C "$origin" symbolic-ref HEAD refs/heads/main
git -C "$origin" config user.email t@t
git -C "$origin" config user.name t

log="$scratch/hw-cargo.log"
cat > "$origin/tools/hw-cargo" <<'EOF'
#!/bin/sh
printf 'hw-cargo: %s\n' "$*" >> "$LOG"
mkdir -p engine/target/dev-release
: > engine/target/dev-release/headwater
chmod +x engine/target/dev-release/headwater
EOF
chmod +x "$origin/tools/hw-cargo"
: > "$origin/engine/Cargo.toml"
git -C "$origin" add -A
git -C "$origin" commit -q -m seed
git -C "$origin" branch second-branch

clone="$scratch/clone"
git clone -q "$origin" "$clone" 2>/dev/null

passed=0
failed=0

echo "-b <branch> <start-point> makes a new branch, fetches, and builds"
: > "$log"
wt1="$scratch/wt1"
(cd "$clone" && LOG="$log" sh "$tool" "$wt1" -b test-branch origin/main) > "$scratch/out" 2>"$scratch/err"
status=$?
if [ "$status" -eq 0 ] && [ -x "$wt1/engine/target/dev-release/headwater" ] \
    && grep -q 'build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked' "$log"; then
    passed=$((passed + 1))
    echo "  ok    new branch made, engine built through tools/hw-cargo"
else
    failed=$((failed + 1))
    echo "  FAIL  new branch made, engine built (exit $status)"
    echo "        log: $(cat "$log" 2>/dev/null)"
    echo "        err: $(cat "$scratch/err")"
fi

echo "an existing branch, with no -b, passes through unchanged"
: > "$log"
wt2="$scratch/wt2"
(cd "$clone" && LOG="$log" sh "$tool" "$wt2" second-branch) > "$scratch/out" 2>"$scratch/err"
status=$?
if [ "$status" -eq 0 ] && [ -x "$wt2/engine/target/dev-release/headwater" ]; then
    passed=$((passed + 1))
    echo "  ok    an existing branch is added and built the same way"
else
    failed=$((failed + 1))
    echo "  FAIL  an existing branch is added and built (exit $status)"
    echo "        err: $(cat "$scratch/err")"
fi

echo "a tree with no engine/Cargo.toml is refused rather than left unbuilt"
bare="$scratch/bare-origin"
mkdir -p "$bare"
git init -q "$bare"
git -C "$bare" symbolic-ref HEAD refs/heads/main
git -C "$bare" config user.email t@t
git -C "$bare" config user.name t
: > "$bare/README.md"
git -C "$bare" add -A
git -C "$bare" commit -q -m seed
bareclone="$scratch/bareclone"
git clone -q "$bare" "$bareclone" 2>/dev/null
wt3="$scratch/wt3"
(cd "$bareclone" && sh "$tool" "$wt3" -b no-engine origin/main) > "$scratch/out" 2>"$scratch/err"
status=$?
if [ "$status" -ne 0 ] && grep -q 'no engine/Cargo.toml' "$scratch/err"; then
    passed=$((passed + 1))
    echo "  ok    refused, and says why"
else
    failed=$((failed + 1))
    echo "  FAIL  refused with the reason (exit $status, err: $(cat "$scratch/err"))"
fi

echo "too few arguments is a usage refusal"
sh "$tool" "$scratch/wt4" > "$scratch/out" 2>"$scratch/err"
status=$?
if [ "$status" -eq 2 ]; then
    passed=$((passed + 1))
    echo "  ok    a missing worktree-add argument is refused"
else
    failed=$((failed + 1))
    echo "  FAIL  a missing worktree-add argument is refused (exit $status)"
fi

echo "$passed passed; $failed failed"
[ "$failed" -eq 0 ]
