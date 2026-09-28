#!/bin/sh
# What holds that a figure on a hand-built page is measured when the site is
# published, and never committed (#1273).
#
# Each case runs against a scratch copy of this tree, with the engine this
# checkout built, and reads the exit status and what the run left behind.
# `wrangler` is never reached: `npx` is a stub on `PATH` that writes its
# arguments to a log, so "the stub was called once" and "the log is empty"
# are the two outcomes a case reads. Three steps of `tools/site/deploy-site.sh`
# are also stubs in the scratch copy, and each one has fixtures of its own:
# `mkdocs build` (a module on `PYTHONPATH`), `tools/site/assemble-site.sh`
# (copies `site/` into the assembled directory; the CI step "The two halves of
# the site compose into one served directory" holds the real one) and
# `tools/site/fetch-apt.sh` (`tools/site/fetch-apt-fixtures.sh`). What is real
# is the order the deploy script runs them in, `refresh-figures.sh`,
# `check-site-figures.sh` and the engine.
#
# The two cases the issue exists for are the first and the third: a pull
# request that adds a document moves no byte under `site/`, and a deploy that
# meets a figure it cannot measure publishes nothing.
#
# Needs `python3`, `tar` and a built engine of either profile. It writes only
# under a temporary directory.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
engine=
for candidate in "$root/engine/target/release/headwater" "$root/engine/target/dev-release/headwater"; do
    if [ -x "$candidate" ] && { [ -z "$engine" ] || [ "$candidate" -nt "$engine" ]; }; then
        engine=$candidate
    fi
done
if [ -z "$engine" ]; then
    echo "figures-fixtures.sh: no built engine under $root/engine/target" >&2
    echo "  build it: cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked" >&2
    exit 1
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
tree="$scratch/tree"
log="$scratch/wrangler.log"
mkdir -p "$tree" "$scratch/bin" "$scratch/py/mkdocs"

# The tree, less what no case reads. `tar` keeps each file's time, so the
# engine copied below stays newer than the sources beside it, as it is here.
tar -C "$root" \
    --exclude=./.git --exclude=./engine/target --exclude=./.claude/worktrees \
    --exclude=./.headwater/site-build --exclude=./.headwater/site-deploy \
    --exclude=./node_modules \
    -cf - . | tar -C "$tree" -xf -
mkdir -p "$tree/engine/target/dev-release"
cp -p "$engine" "$tree/engine/target/dev-release/headwater"

cat > "$scratch/bin/npx" <<STUB
#!/bin/sh
printf '%s\n' "\$*" >> "$log"
STUB
chmod +x "$scratch/bin/npx"
: > "$scratch/py/mkdocs/__init__.py"
printf 'print("the planted mkdocs: nothing rendered")\n' > "$scratch/py/mkdocs/__main__.py"
cat > "$tree/tools/site/assemble-site.sh" <<'STUB'
#!/bin/sh
set -eu
rm -rf .headwater/site-deploy
mkdir -p .headwater/site-deploy
cp -R site/. .headwater/site-deploy/
STUB
cat > "$tree/tools/site/fetch-apt.sh" <<'STUB'
#!/bin/sh
set -eu
mkdir -p "$1/apt"
echo "the planted fetch-apt: $1/apt"
STUB
cp -R "$tree/site" "$scratch/site.pristine"

pass=0
fail=0
judge() {
    # judge <name> <expected-status> <actual-status> <needle> <haystack>
    if [ "$2" = "$3" ] && { [ -z "$4" ] || printf '%s' "$5" | grep -qF -- "$4"; }; then
        pass=$((pass + 1))
        echo "ok   $1"
    else
        fail=$((fail + 1))
        echo "FAIL $1 (expected $2, got $3; wanted '$4')"
        printf '%s\n' "$5" | sed 's/^/     /'
    fi
}
reset() {
    rm -rf "$tree/site" "$tree/.headwater/site-deploy"
    cp -R "$scratch/site.pristine" "$tree/site"
    : > "$log"
}
calls() {
    wc -l < "$log" | tr -d ' '
}
figures() {
    (cd "$tree" && sh tools/site/refresh-figures.sh "$@" 2>&1)
}
deploy() {
    (cd "$tree" && PATH="$scratch/bin:$PATH" PYTHONPATH="$scratch/py" sh tools/site/deploy-site.sh 2>&1)
}
seen() {
    printf '%s\n' "$1" | sed -n 's/^census\.seen  *\([0-9][0-9]*\) .*/\1/p'
}

# 0. The committed pages carry no measured figure, so --check passes here
#    before anything is planted. A red here means the tree itself is wrong.
reset
out=$(figures --check); status=$?
judge 'the committed pages carry every figure blank' 0 "$status" 'checked under site' "$out"

# 1. A pull request that adds a governed document. The measurement sees it
#    (census.seen moves by one), --check still passes, and no byte under
#    site/ moves.
reset
before=$(seen "$(figures --print)")
sed 's/HW-DR-0039/HW-DR-9039/g' "$tree/docs/decisions/0039-q39-how-a-figure-reaches-a-hand-built-page.md" \
    > "$tree/docs/decisions/9039-a-fixture-decision.md"
after=$(seen "$(figures --print)")
judge 'the added document moves the measurement by one' 0 0 "census.seen $((before + 1))" "census.seen $after"
out=$(figures --check); status=$?
judge 'and --check passes with the added document' 0 "$status" 'checked under site' "$out"
diff -r "$scratch/site.pristine" "$tree/site" > "$scratch/site.diff" 2>&1; status=$?
judge 'and no byte under site/ moved' 0 "$status" '' "$(cat "$scratch/site.diff")"

# 2. The deploy fills every figure in the assembled directory, touches no
#    page under site/, and calls wrangler once.
out=$(deploy); status=$?
judge 'the deploy exits 0' 0 "$status" 'deploy-site.sh: done' "$out"
judge 'and it calls wrangler once' 1 "$(calls)" '' ''
judge 'and it calls wrangler deploy at the pinned version' 0 0 'wrangler@' "$(cat "$log")"
out=$(sh "$tree/tools/site/check-site-figures.sh" "$tree/.headwater/site-deploy" 2>&1); status=$?
judge 'and every figure in the assembled directory holds a value' 0 "$status" 'none blank' "$out"
filled=$(grep -o 'data-figure="census.seen"[^>]*>[0-9]*<' "$tree/.headwater/site-deploy/index.html" | head -1)
judge 'and the landing page states the measured census' 0 0 ">$after<" "$filled"
diff -r "$scratch/site.pristine" "$tree/site" > "$scratch/site.diff" 2>&1; status=$?
judge 'and site/ is still byte for byte what was committed' 0 "$status" '' "$(cat "$scratch/site.diff")"
rm -f "$tree/docs/decisions/9039-a-fixture-decision.md"

# 3. A key nothing measures. The deploy stops before wrangler.
reset
sed -i 's|data-figure="census.seen"|data-figure="nosuch.key"|' "$tree/site/index.html"
out=$(deploy); status=$?
judge 'a page carrying an unknown key stops the deploy' 1 "$status" 'unknown figure key nosuch.key' "$out"
judge 'and wrangler is never called' 0 "$(calls)" '' ''

#    A key off the shape the pattern reads is not passed over as no marker.
reset
sed -i 's|data-figure="census.seen"|data-figure="no.such.key"|' "$tree/site/index.html"
out=$(deploy); status=$?
judge 'a marker the pattern cannot read stops the deploy' 1 "$status" 'that this cannot fill' "$out"
judge 'and wrangler is never called' 0 "$(calls)" '' ''

# 4. A measured key that no page carries any more.
reset
find "$tree/site" -name '*.html' -exec sed -i 's|data-figure="verbs.groups"|data-gone="verbs.groups"|g' {} +
out=$(deploy); status=$?
judge 'a measured figure on no page stops the deploy' 1 "$status" 'measured but on no page: verbs.groups' "$out"
judge 'and wrangler is never called' 0 "$(calls)" '' ''

# 5. A value written into a committed marker is refused, and --blank is the
#    remedy it names.
reset
sed -i 's|data-figure="census.seen"></span>|data-figure="census.seen">635</span>|' "$tree/site/index.html"
out=$(figures --check); status=$?
judge 'a committed figure is refused' 1 "$status" 'measured figure' "$out"
judge 'and the refusal names the remedy' 1 "$status" 'refresh-figures.sh --blank' "$out"
out=$(cd "$tree" && sh tools/site/refresh-figures.sh --blank 2>&1); status=$?
judge 'and --blank empties it' 0 "$status" 'blanked the figures on 1 page' "$out"
out=$(figures --check); status=$?
judge 'after which --check passes' 0 "$status" 'checked under site' "$out"

# 6. The net under a Workers Build left connected: an assembled directory with
#    a blank figure is refused.
reset
mkdir -p "$scratch/blank"
cp -R "$tree/site/." "$scratch/blank/"
out=$(sh "$tree/tools/site/check-site-figures.sh" "$scratch/blank" 2>&1); status=$?
judge 'an assembled directory with a blank figure is refused' 1 "$status" 'are blank, so this directory is not deployed' "$out"

# 7. An engine older than the sources measures nothing, in any mode, and the
#    deploy stops there too.
reset
touch "$tree/engine/crates/cli/src/main.rs"
out=$(figures --check); status=$?
judge 'an engine behind its sources cannot measure' 3 "$status" 'cannot tell whether a figure is stale' "$out"
out=$(deploy); status=$?
judge 'and the deploy stops before wrangler' 3 "$status" '' "$out"
judge 'and wrangler is never called' 0 "$(calls)" '' ''
touch -r "$engine" "$tree/engine/crates/cli/src/main.rs"
touch "$tree/engine/target/dev-release/headwater"

# 8. No mode writes a measured figure into site/.
out=$(figures); status=$?
judge 'a bare run names the modes and writes nothing' 2 "$status" 'name a mode' "$out"

echo
echo "$pass passed, $fail failed"
[ "$fail" -eq 0 ]
