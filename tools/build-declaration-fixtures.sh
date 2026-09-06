#!/bin/sh
# What holds the two declarations the build of this engine makes about itself:
# the `--locked` flag on the cargo runs that must not rewrite the lock, and the
# Apache-2.0 license every crate manifest inherits.
#
# Neither is behavior. Both are declarations that are green the day they land
# and green forever, so nothing about a passing CI run tells you either one is
# still there. That is what this suite is for, and it is the reason it exists
# separately from any check rule: no rule of this engine reads a `Cargo.toml`
# or a workflow file, and none should.
#
# Run it from anywhere:
#     sh tools/build-declaration-fixtures.sh
#
# # WHAT EACH HALF WOULD LOSE, AND WHICH HALF NEEDS THIS SUITE
#
# The license half is self-guarding in one direction and not in the other.
# Delete `license` from `[workspace.package]` in `engine/Cargo.toml` and every
# cargo command fails before it compiles anything:
#
#     error inheriting `license` from workspace root manifest's
#     `workspace.package.license`
#
# So the removal direction is unrepresentable and needs no fixture. The
# direction that stays silent is a 24th crate whose manifest forgets
# `license.workspace = true`: it reports `license: null` to `cargo metadata`,
# every other crate still reports Apache-2.0, and nothing anywhere says so.
# Case group 1 enumerates the crate directories rather than listing them, so a
# crate added later has to answer for itself.
#
# The `--locked` half has no self-guard at all in either direction. A flag
# dropped from one CI step, or a fourteenth copy of the install command landing
# without it, changes no exit status on a lock that is in sync — which is every
# day until the day it matters. Case groups 2 and 3 enumerate the invocation
# sites rather than listing them, for the same reason.
#
# # THE BOUNDARY, WHICH IS DELIBERATE AND IS NOT COVERAGE MISSING
#
# `--locked` is required where a lock rewrite would be both silent and
# unwanted: the cargo steps of `.github/workflows/ci.yml`, the two container
# commands of `engine/README.md` that reproduce CI's floor, and every verbatim
# copy of the install command a stranger or an agent is handed. It is NOT
# required on a maintainer's own loop — the four commands at the top of
# `engine/README.md` and the eleven `HEADWATER_BLESS=1` commands at the end of
# it — because there a rewritten lock is the intended result of adding a
# dependency, and `--locked` would refuse the very thing the maintainer meant.
#
# `spike/` is a second cargo workspace with its own committed `spike/Cargo.lock`
# and it is out of scope here, deliberately: no workflow runs `spike/build.sh`
# and nothing builds that workspace, so its lock is a second ungated committed
# lock and this suite leaves it that way rather than pretending otherwise.
#
# `cargo fmt` is exempt everywhere. It rejects the flag outright with
# `error: unexpected argument '--locked' found`, because it reads no manifest
# and resolves nothing. So does `cargo --version`. The judge below reads the
# subcommand and exempts exactly those two shapes, rather than carrying a list
# of blessed lines that would go stale.
#
# # WHAT IT NEEDS, AND WHAT IT WRITES
#
# `git`, `awk` and `cargo`. Case group 1's last case runs
# `cargo metadata --no-deps --locked --format-version 1`, which resolves
# nothing beyond the workspace and compiles nothing; it carries `--locked`
# itself, so this suite cannot rewrite the lock it is checking. Every scratch
# file is made under `mktemp -d`, the directory goes on an interrupt, and
# nothing inside this checkout is written.

set -u

root=$(cd "$(dirname "$0")/.." && pwd)

if ! command -v git >/dev/null 2>&1; then
    echo "no \`git\` on the path, and every site below is enumerated from the" >&2
    echo "  tracked tree. This suite stops rather than reporting a row of" >&2
    echo "  passes over a set it could not read." >&2
    exit 1
fi
if ! command -v cargo >/dev/null 2>&1; then
    echo "no \`cargo\` on the path. The license cases below read what cargo" >&2
    echo "  reports, which is the whole point of declaring the field, so this" >&2
    echo "  suite stops rather than skipping them." >&2
    exit 1
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0

pass() {
    passed=$((passed + 1))
    echo "  ok    $1"
}

fail() {
    failed=$((failed + 1))
    echo "  FAIL  $1"
    echo "          $2"
}

# same NAME EXPECTED ACTUAL
same() {
    if [ "$2" = "$3" ]; then
        pass "$1"
    else
        fail "$1" "expected \`$2\`, got \`$3\`"
    fi
}

# The install command, assembled from two halves so that this file never holds
# a verbatim copy of it. That is not fastidiousness: this file is a tracked file
# and `git grep` reads it, so a whole copy written here would be a copy of the
# install command with no `--locked` on it, and case 7 would report its own
# source as the offender. The first CI run of this suite did exactly that.
#
# It did not fail locally, and the reason is worth knowing: `git grep` reads
# tracked files only, and this file was still untracked when the suite was first
# run over it. A new file that a suite reads through git is invisible to that
# suite until it is committed, so a local green over an uncommitted addition is
# not a reading of the population the suite will see.
install_head='cargo build --release -p headwater-cli'
install_tail='--manifest-path engine/Cargo.toml'
install_cmd="$install_head $install_tail"

# unflagged FILE SELECTOR
#
# Print one line per cargo invocation in FILE, on a line matching SELECTOR,
# that must carry `--locked` and does not. An invocation runs from `cargo ` to
# the next shell operator or closing quote, so a line joining two of them with
# `&&` is read as two. The subcommand decides whether the flag is required:
# `fmt` and any `--flag` shape (`cargo --version`) are exempt because cargo
# refuses the flag on them.
#
# Prints `TOTAL EXEMPT FLAGGED` on the last line, always, so a selector that
# stopped matching anything reads as zero flagged rather than as zero missing.
unflagged() {
    awk -v sel="$2" '
        function classify(inv,   cmd, w) {
            split(inv, w, /[ \t]+/)
            cmd = w[1]
            if (cmd == "fmt" || substr(cmd, 1, 1) == "-") { exempt++; return }
            if (inv ~ /(^|[ \t])--locked([ \t]|$)/) { flagged++; return }
            print "cargo " inv
        }
        $0 ~ sel {
            rest = $0
            while (match(rest, /cargo [^ \t]/)) {
                rest = substr(rest, RSTART + 6)
                inv = rest
                if (match(inv, /(&&|\|\||;|")/)) inv = substr(inv, 1, RSTART - 1)
                sub(/[ \t]+$/, "", inv)
                total++
                classify(inv)
            }
        }
        END { print total + 0, exempt + 0, flagged + 0 }
    ' "$1"
}

# report FILE SELECTOR — the counts line of `unflagged`
counts() { unflagged "$1" "$2" | tail -1; }
# offenders FILE SELECTOR — every invocation missing the flag, one per line
offenders() { unflagged "$1" "$2" | sed '$d'; }

echo "the license every crate inherits"

# 1. The inheritance source. Without this line the workspace does not load at
#    all, so this case is about the value rather than about the presence.
declared=$(awk '/^\[workspace\.package\]/ {in_block = 1; next}
                /^\[/ {in_block = 0}
                in_block && /^license *=/ {print $3}' "$root/engine/Cargo.toml")
same "\`[workspace.package]\` declares the license" '"Apache-2.0"' "$declared"

# 2. Every crate opts in. Enumerated from the directories, not listed, so a
#    crate added later is in this population the moment it exists.
crates=0
missing=""
for dir in "$root"/engine/crates/*/; do
    crates=$((crates + 1))
    if ! grep -q '^license\.workspace = true$' "$dir/Cargo.toml"; then
        missing="$missing $(basename "$dir")"
    fi
done
same "every crate manifest inherits it" "" "$missing"
if [ "$crates" -gt 0 ]; then
    pass "  over every crate directory there is ($crates read)"
else
    fail "  over every crate directory there is" \
        "no crate directory was read, so the case above measured nothing"
fi

# 3. And the workspace membership is that same set, so case 2's population is
#    the set cargo actually builds rather than a directory listing beside it.
members=$(awk '/^members = \[/ {in_list = 1; next}
               /^\]/ {in_list = 0}
               in_list {gsub(/[",]/, ""); gsub(/^ +| +$/, ""); sub(/^crates\//, "")
                        if ($0 != "") print}' "$root/engine/Cargo.toml" | sort | tr '\n' ' ')
dirs=$(for d in "$root"/engine/crates/*/; do basename "$d"; done | sort | tr '\n' ' ')
same "  and the workspace members are exactly those directories" "$dirs" "$members"

# 4. What cargo reports, which is the reading the declaration exists for. A
#    manifest key nothing reads back is the shape this whole suite refuses.
cargo metadata --no-deps --locked --format-version 1 \
    --manifest-path "$root/engine/Cargo.toml" >"$scratch/metadata.json" 2>"$scratch/metadata.err"
status=$?
if [ "$status" -ne 0 ]; then
    fail "cargo reports the license for every workspace member" \
        "\`cargo metadata --no-deps --locked\` exited $status: $(head -1 "$scratch/metadata.err")"
else
    apache=$(grep -o '"license":"Apache-2.0"' "$scratch/metadata.json" | wc -l | tr -d ' ')
    null=$(grep -o '"license":null' "$scratch/metadata.json" | wc -l | tr -d ' ')
    same "cargo reports the license for every workspace member" "$crates" "$apache"
    same "  and reports it for no member as null" 0 "$null"
fi

echo "the flag on every cargo run that must not rewrite the lock"

# 5. The workflow. Every invocation is found by reading the file, so a step
#    added below the ones this change edited has to carry the flag too.
set -- $(counts "$root/.github/workflows/ci.yml" '^ *(run: )?cargo ')
ci_total=$1; ci_exempt=$2; ci_flagged=$3
bad=$(offenders "$root/.github/workflows/ci.yml" '^ *(run: )?cargo ')
same "every cargo step in the workflow carries \`--locked\` or cannot" "" "$bad"
if [ "$ci_flagged" -ge 3 ]; then
    pass "  and at least the three the issue's bar names do ($ci_total invocations, $ci_exempt exempt, $ci_flagged flagged)"
else
    fail "  and at least the three the issue's bar names do" \
        "$ci_total invocations, $ci_exempt exempt, only $ci_flagged flagged"
fi

# 6. The two container commands, which exist to reproduce CI's floor and so
#    answer to CI's rule. Asked of the image line rather than of the file,
#    because the loop at the top of that README is deliberately exempt — see
#    the boundary in this file's header. The flagged count is asserted, not
#    just the offender count, so an image tag change that stopped the selector
#    matching fails here instead of reading as nothing to report.
set -- $(counts "$root/engine/README.md" 'rust:1[.][0-9]+')
rd_total=$1; rd_exempt=$2; rd_flagged=$3
bad=$(offenders "$root/engine/README.md" 'rust:1[.][0-9]+')
same "both container commands carry it" "" "$bad"
same "  and there are two of them, beside the one \`fmt\` that cannot" 2 "$rd_flagged"
same "  which is every cargo run on an image line" 3 "$rd_total"
same "  and the exempt one is \`fmt\`" 1 "$rd_exempt"

# 7. The install command a stranger and every agent is handed. Fourteen
#    verbatim copies today across thirteen files, none of which a single reader
#    owns, and the failure this refuses is the fifteenth landing unflagged: a
#    README that says `--locked` beside thirteen copies that do not is worse
#    than none of them saying it, because it reads as a guarantee.
#
#    The offenders are named rather than counted. A case that reports
#    `expected 15, got 14` is a case whose reader has to go and find out which
#    one, and the first CI run of this suite made somebody do that.
#    Written as a function taking a directory, so case 11 can drive the same
#    code over a scratch repository holding a planted unflagged copy.
install_offenders() {
    ( cd "$1" && git grep -n -F "$install_cmd" 2>/dev/null |
        grep -v -F "$install_cmd --locked" )
}
install_copies() {
    ( cd "$1" && git grep -c -F "$2" 2>/dev/null |
        awk -F: '{n += $2} END {print n + 0}' )
}

copies=$(install_copies "$root" "$install_cmd")
flagged=$(install_copies "$root" "$install_cmd --locked")
bad=$(install_offenders "$root")
if [ "$copies" = "$flagged" ] && [ -z "$bad" ]; then
    pass "every copy of the install command carries it"
else
    fail "every copy of the install command carries it" \
        "$copies copies, $flagged flagged. Unflagged:
          $(echo "$bad" | sed 's/^/          /')"
fi
if [ "$copies" -gt 0 ]; then
    pass "  over every copy in the tracked tree ($copies read)"
else
    fail "  over every copy in the tracked tree" \
        "no copy was found, so the case above compared zero with zero"
fi

echo "the judges, provoked on purpose"

# 8. Spec 12 refuses a check that ships with no failing fixture, and the reason
#    carries to a suite: a judge nobody has seen refuse anything is a judge
#    nobody has seen work. Both judges above return the empty string on this
#    tree, and the empty string is also what a broken judge returns.
mkdir -p "$scratch/wf"
sed 's/^\( *run: cargo test\) --locked$/\1/' \
    "$root/.github/workflows/ci.yml" >"$scratch/wf/ci.yml"
if cmp -s "$root/.github/workflows/ci.yml" "$scratch/wf/ci.yml"; then
    fail "the workflow judge names a step whose flag was removed" \
        "the planted edit changed nothing, so the case below measured nothing"
else
    bad=$(offenders "$scratch/wf/ci.yml" '^ *(run: )?cargo ')
    same "the workflow judge names a step whose flag was removed" \
        "cargo test" "$bad"
fi

# 9. And the same for the container line, where two invocations share one line
#    and only the second one is the subject.
mkdir -p "$scratch/rd"
sed 's/^\(.*rust:1\.[0-9]*-slim cargo test\) --locked$/\1/' \
    "$root/engine/README.md" >"$scratch/rd/README.md"
bad=$(offenders "$scratch/rd/README.md" 'rust:1[.][0-9]+')
same "the container judge names the test command whose flag was removed" \
    "cargo test" "$bad"

# 10. The license judge, over a crate that forgot the member line. This is the
#     one direction the compiler does not cover, so it is the one case here
#     with nothing else behind it.
mkdir -p "$scratch/crates/remembered" "$scratch/crates/forgot"
printf '[package]\nname = "a"\nlicense.workspace = true\n' \
    >"$scratch/crates/remembered/Cargo.toml"
printf '[package]\nname = "b"\n' >"$scratch/crates/forgot/Cargo.toml"
named=""
for dir in "$scratch"/crates/*/; do
    grep -q '^license\.workspace = true$' "$dir/Cargo.toml" || named="$named $(basename "$dir")"
done
same "the license judge names a crate that forgot to inherit" " forgot" "$named"

# 11. The install-command judge, over a scratch repository holding one flagged
#     copy and one unflagged one. This is the case that failed on this suite's
#     first CI run, and it failed against this suite's own source: a tracked
#     file holding a whole copy of the install command is a copy of it, and
#     `git grep` does not care which file it lives in. It reads through git, so
#     the planted copies have to be committed to be seen at all — which is the
#     same reason the defect was invisible to the local run that preceded it.
repo="$scratch/copies"
mkdir -p "$repo"
git -C "$repo" init -q
git -C "$repo" symbolic-ref HEAD refs/heads/main
printf 'build it: %s --locked\n' "$install_cmd" >"$repo/good.md"
printf 'build it: %s\n' "$install_cmd" >"$repo/bad.md"
git -C "$repo" add -A >/dev/null 2>&1
git -C "$repo" -c user.name=fixture -c user.email=fixture@example.invalid \
    -c commit.gpgsign=false commit -q -m "one of each" >/dev/null 2>&1
same "the install judge names the unflagged copy and not the flagged one" \
    "bad.md:1:build it: $install_cmd" "$(install_offenders "$repo")"
same "  and counts both copies" 2 "$(install_copies "$repo" "$install_cmd")"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
