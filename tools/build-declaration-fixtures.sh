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
# direction that stays silent is a crate whose manifest forgets
# `license.workspace = true`: it reports `license: null` to `cargo metadata`,
# every other crate still reports Apache-2.0, and nothing anywhere says so.
# Case group 1 enumerates the crate directories rather than listing them, so a
# crate added later has to answer for itself.
#
# The `--locked` half has no self-guard at all in either direction. A flag
# dropped from one CI step, or a further copy of the install command landing
# without it, changes no exit status on a lock that is in sync — which is every
# day until the day it matters. Case groups 2 and 3 enumerate the invocation
# sites rather than listing them, for the same reason.
#
# # HOW EACH POPULATION IS READ, WHICH IS WHERE THE EASY MISTAKE IS
#
# `.github/workflows/ci.yml` is read by parsing it, not by matching a line
# shape. Every `run:` value is scanned as shell, single-line and block scalar
# alike, so `cd engine && cargo tree`, `RUSTFLAGS=… cargo build` and
# `bash -c "cargo test"` are all found. The first cut of this suite anchored a
# regular expression at the start of the line and silently saw none of those
# three: it could not tell "no other cargo step exists" from "a cargo step
# exists in a shape I do not parse", and the wrong reading returned green.
# Case 8b plants those shapes, and a block scalar, so the parse is held
# rather than asserted.
#
# The install command is read per OCCURRENCE and not per line. Filtering lines
# — `grep -n <cmd> | grep -v <cmd> --locked` — drops a whole line when any
# occurrence on it carries the flag, and this repository forbids hard-wrapped
# Markdown, so its paragraphs are single long lines and a paragraph contrasting
# the flagged form with the unflagged one would hide the unflagged one at exit
# 0. Each occurrence is judged on the text that follows it, up to the next
# shell operator, quote or backtick, so `-j2` or any other flag between the
# manifest path and `--locked` is read correctly rather than reported as a
# defect. A copy split over a `\` line continuation is reported as unflagged,
# which is loud and wrong in the safe direction.
#
# # THE BOUNDARY, WHICH IS DELIBERATE AND IS NOT COVERAGE MISSING
#
# `--locked` is required where a lock rewrite would be both silent and
# unwanted: the cargo steps of `.github/workflows/ci.yml`, the two container
# commands of `engine/README.md` that reproduce CI's floor, and every copy of
# the release build of the CLI that a stranger or an agent is handed. It is NOT
# required on a maintainer's own loop — the `cargo run` and `cargo test`
# commands at the top and the end of `engine/README.md`, and the
# `cargo test --workspace` line two skills carry — because there a rewritten
# lock is the intended result of adding a dependency, and `--locked` would
# refuse the very thing the maintainer meant. That is why case 7's population
# is `cargo build` naming `-p headwater-cli` and not every cargo command that
# names the engine.
#
# Three cargo shapes in this tree are out of scope, each for its own reason,
# written here rather than left to be inferred from their absence:
#
#   The Q1 language spike is a second cargo workspace with its own committed
#   `Cargo.lock` and its own `build.sh`. No workflow runs that script and
#   nothing builds that workspace, so its lock is a second ungated committed
#   lock and this suite leaves it that way rather than pretending otherwise.
#   It is named here by what it is rather than by where it lives, because it
#   moved once already: #591 took it from `spike/` to `tools/language-spike/`
#   while this branch was open. It names no `-p headwater-cli`, so the move
#   changes no population read below — measured on `origin/main`, not assumed.
#
#   `engine/.cargo/config.toml` declares four aliases, two of which expand to a
#   release build of the CLI. An alias is invoked as `cargo release-cli`, is a
#   maintainer convenience, and answers to the maintainer's-loop half of the
#   boundary above.
#
#   `site/index.html` advertises `cargo install headwater-cli` in the landing
#   page's animated terminal. That is not a lock question at all: the route
#   does not exist, because `publish = false` and nothing is on crates.io. The
#   page carries its own comment naming what gates it, and #526 tracks the
#   install path. Nothing here judges it.
#
# `cargo fmt` and `cargo --version` are exempt everywhere, for two different
# reasons that are worth keeping apart. `cargo fmt --locked` REJECTS the flag —
# `error: unexpected argument '--locked' found` — because fmt reads no manifest
# and resolves nothing. `cargo --version --locked` ACCEPTS it and exits 0,
# ignoring it, for the same underlying reason. So neither can rewrite a lock,
# and only one of them would fail if the flag were added. The judge reads the
# subcommand and exempts those two shapes, rather than carrying a list of
# blessed lines that would go stale.
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

# The install command, assembled from three pieces so that no line of this file
# holds `cargo build`, and no line holds the whole command either. That is not
# fastidiousness. This file is a tracked file and the judges below read the
# tracked tree, so a copy written here whole would be a copy of the install
# command with no `--locked` on it, and case 7 would report its own source as
# the offender. The first CI run of this suite did exactly that.
#
# It did not fail locally, and the reason is worth knowing: `git grep` reads
# tracked files only, and this file was still untracked when the suite was
# first run over it. A new file that a suite reads through git is invisible to
# that suite until it is committed, so a local green over an uncommitted
# addition is not a reading of the population the suite will see.
install_verb='cargo'
install_head='build --release -p headwater-cli'
install_key='-p headwater-cli'
install_tail='--manifest-path engine/Cargo.toml'
install_cmd="$install_verb $install_head $install_tail"

# cargo_offenders FILE MODE SELECTOR
#
# Print one line per cargo invocation that must carry `--locked` and does not,
# then `TOTAL EXEMPT FLAGGED` on the last line, always — so a parse that
# stopped finding anything reads as zero flagged rather than as zero missing.
#
# MODE `workflow` parses the file as a GitHub workflow and scans the value of
# every `run:` key, both the single-line form and the block scalar. A YAML
# comment, a `name:` line and a shell comment inside a run block are not
# commands and are never scanned. MODE `lines` scans every line matching
# SELECTOR, which is what a Markdown file needs.
#
# An invocation runs from `cargo ` to the next shell operator or closing quote,
# so a line joining two of them with `&&` is read as two.
cargo_offenders() {
    awk -v mode="$2" -v sel="$3" '
        function classify(inv,   cmd, w) {
            split(inv, w, /[ \t]+/)
            cmd = w[1]
            # `fmt` rejects the flag; `--version` accepts and ignores it.
            # Neither reads a manifest, so neither can rewrite a lock.
            if (cmd == "fmt" || substr(cmd, 1, 1) == "-") { exempt++; return }
            if (inv ~ /(^|[ \t])--locked([^A-Za-z0-9_-]|$)/) { flagged++; return }
            print "cargo " inv
        }
        function scan(text,   rest, inv) {
            rest = text
            while (match(rest, /cargo [^ \t]/)) {
                rest = substr(rest, RSTART + 6)
                inv = rest
                if (match(inv, /(&&|\|\||;|")/)) inv = substr(inv, 1, RSTART - 1)
                sub(/[ \t]+$/, "", inv)
                total++
                classify(inv)
            }
        }
        mode == "lines" { if ($0 ~ sel) scan($0); next }
        /^[ \t]*#/ { next }
        {
            if (inblock) {
                if ($0 ~ /^[ \t]*$/) next
                if (match($0, /[^ \t]/) - 1 >= blockind) { scan($0); next }
                inblock = 0
            }
            p = index($0, "run:")
            if (p > 0 && substr($0, 1, p - 1) ~ /^[ \t-]*$/) {
                rest = substr($0, p + 4)
                sub(/^[ \t]+/, "", rest)
                if (rest ~ /^[|>][-+0-9]*$/) { inblock = 1; blockind = p; next }
                scan(rest)
            }
        }
        END { print total + 0, exempt + 0, flagged + 0 }
    ' "$1"
}

counts() { cargo_offenders "$1" "$2" "$3" | tail -1; }
offenders() { cargo_offenders "$1" "$2" "$3" | sed '$d'; }

# install_judge DIR
#
# Every `cargo build` in DIR's tracked files that names `-p headwater-cli`,
# judged one occurrence at a time. Prints `path:line: cargo build …` for each
# occurrence missing `--locked`, then `TOTAL FLAGGED` on the last line.
install_judge() {
    ( cd "$1" && git grep -n -F -- "$install_key" 2>/dev/null ) |
        awk -v verb="$install_verb build" -v key="$install_key" '
            {
                pfx = (match($0, /^[^:]*:[0-9]+:/)) ? substr($0, 1, RLENGTH) : ""
                rest = $0
                while ((p = index(rest, verb)) > 0) {
                    rest = substr(rest, p + length(verb))
                    inv = rest
                    if (match(inv, /(&&|\|\||;|"|`)/)) inv = substr(inv, 1, RSTART - 1)
                    if (index(inv, key) == 0) continue
                    total++
                    if (inv ~ /(^|[ \t])--locked([^A-Za-z0-9_-]|$)/) flagged++
                    else print pfx " " verb inv
                }
            }
            END { print total + 0, flagged + 0 }
        '
}

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

# 4b. The sentence the manifest comment makes about the sources, held rather
#     than asserted. `engine/Cargo.toml` says the license restates the SPDX
#     header every source file under `crates/` carries, and a comment claiming
#     a property of 219 files is the shape that goes false in silence. A file
#     under a `fixtures/` directory is exempt, because a fixture whose purpose
#     is to not be a document correctly carries no header — that is the one
#     file the issue names, and the exemption is stated as a rule rather than
#     as that path, so the next such fixture needs no edit here.
sources=0
exempt_rs=0
no_header=""
for file in $(cd "$root" && git ls-files 'engine/crates/**/*.rs'); do
    case "$file" in
        */fixtures/*) exempt_rs=$((exempt_rs + 1)); continue ;;
    esac
    sources=$((sources + 1))
    head -1 "$root/$file" | grep -q '^// SPDX-License-Identifier: Apache-2.0$' ||
        no_header="$no_header $file"
done
same "every source file under \`crates/\` opens with the SPDX header" "" "$no_header"
if [ "$sources" -gt 0 ]; then
    pass "  over every tracked \`.rs\` that is not a fixture ($sources read, $exempt_rs exempt)"
else
    fail "  over every tracked \`.rs\` that is not a fixture" \
        "no source file was read, so the case above measured nothing"
fi

echo "the flag on every cargo run that must not rewrite the lock"

# 5. The workflow, parsed rather than pattern-matched. Every `run:` value is
#    read as shell, so a step added below the ones this change edited has to
#    carry the flag whatever shape it is written in.
set -- $(counts "$root/.github/workflows/ci.yml" workflow '')
ci_total=$1; ci_exempt=$2; ci_flagged=$3
bad=$(offenders "$root/.github/workflows/ci.yml" workflow '')
same "every cargo step in the workflow that resolves a manifest carries \`--locked\`" "" "$bad"
if [ "$ci_flagged" -ge 3 ] && [ "$ci_total" -gt "$ci_exempt" ]; then
    pass "  over every \`run:\` value in the file ($ci_total invocations, $ci_exempt exempt, $ci_flagged flagged)"
else
    fail "  over every \`run:\` value in the file" \
        "$ci_total invocations, $ci_exempt exempt, $ci_flagged flagged — too few to have parsed the file"
fi

# 6. The two container commands, which exist to reproduce CI's floor and so
#    answer to CI's rule. Asked of the image line rather than of the file,
#    because the maintainer's loop in that README is deliberately exempt. The
#    flagged count is asserted, not just the offender count, so an image tag
#    change that stopped the selector matching fails here instead of reading as
#    nothing to report.
set -- $(counts "$root/engine/README.md" lines 'rust:1[.][0-9]+')
rd_total=$1; rd_exempt=$2; rd_flagged=$3
bad=$(offenders "$root/engine/README.md" lines 'rust:1[.][0-9]+')
same "both container commands carry it" "" "$bad"
same "  and there are two of them, beside the one \`fmt\` that cannot" 2 "$rd_flagged"
same "  which is every cargo run on an image line" 3 "$rd_total"
same "  and the exempt one is \`fmt\`" 1 "$rd_exempt"

# 7. The release build of the CLI, wherever a stranger or an agent is handed a
#    copy of it. These live in files that no single reader owns, and the
#    failure this refuses is a further copy landing unflagged: a README that
#    says `--locked` beside a dozen copies that do not is worse than none of
#    them saying it, because it reads as a guarantee.
#
#    The offenders are named rather than counted. A case that reports
#    `expected 15, got 14` is a case whose reader has to go and find out which
#    one, and the first CI run of this suite made somebody do that.
set -- $(install_judge "$root" | tail -1)
inst_total=$1; inst_flagged=$2
bad=$(install_judge "$root" | sed '$d')
if [ -z "$bad" ] && [ "$inst_total" = "$inst_flagged" ]; then
    pass "every copy of the release build of the CLI carries it"
else
    fail "every copy of the release build of the CLI carries it" \
        "$inst_total occurrences, $inst_flagged flagged. Unflagged:
$(printf '%s\n' "$bad" | sed 's/^/            /')"
fi
if [ "$inst_total" -gt 0 ]; then
    pass "  over every occurrence in the tracked tree ($inst_total read)"
else
    fail "  over every occurrence in the tracked tree" \
        "no occurrence was found, so the case above compared zero with zero"
fi

echo "the judges, provoked on purpose"

# 8a. Spec 12 refuses a check that ships with no failing fixture, and the reason
#     carries to a suite: a judge nobody has seen refuse anything is a judge
#     nobody has seen work. Every judge above returns the empty string on this
#     tree, and the empty string is also what a broken judge returns.
mkdir -p "$scratch/wf"
sed 's/^\( *run: cargo test\) --locked$/\1/' \
    "$root/.github/workflows/ci.yml" >"$scratch/wf/ci.yml"
if cmp -s "$root/.github/workflows/ci.yml" "$scratch/wf/ci.yml"; then
    fail "the workflow judge names a step whose flag was removed" \
        "the planted edit changed nothing, so the case below measured nothing"
else
    bad=$(offenders "$scratch/wf/ci.yml" workflow '')
    same "the workflow judge names a step whose flag was removed" \
        "cargo test" "$bad"
fi

# 8b. The three shapes the first cut of this suite could not see. Each hides
#     `cargo` behind something on the same `run:` line — a directory change, an
#     environment assignment, a nested shell — and each was reported as nothing
#     at all rather than as an offender. A regular expression anchored at the
#     start of the line cannot tell those apart from their absence, which is
#     why the workflow half parses `run:` values instead.
{
    cat "$root/.github/workflows/ci.yml"
    printf '      - name: A step that changes directory first\n'
    printf '        run: cd engine && cargo tree --workspace\n'
    printf '      - name: A step with an environment assignment first\n'
    printf '        run: RUSTFLAGS=-Awarnings cargo doc --no-deps\n'
    printf '      - name: A step inside a nested shell\n'
    printf '        run: bash -c "cargo test"\n'
    printf '      - name: A step whose command is a block scalar\n'
    printf '        run: |\n'
    printf '          cd engine\n'
    printf '          cargo clippy --all-targets\n'
} >"$scratch/wf/shapes.yml"
bad=$(offenders "$scratch/wf/shapes.yml" workflow '' | sort | tr '\n' '|')
same "the workflow judge sees a cargo run behind a \`cd\`, an assignment, a nested shell and a block scalar" \
    "cargo clippy --all-targets|cargo doc --no-deps|cargo test|cargo tree --workspace|" \
    "$bad"

# 9. And the same for the container line, where two invocations share one line
#    and only the second one is the subject.
mkdir -p "$scratch/rd"
sed 's/^\(.*rust:1\.[0-9]*-slim cargo test\) --locked$/\1/' \
    "$root/engine/README.md" >"$scratch/rd/README.md"
bad=$(offenders "$scratch/rd/README.md" lines 'rust:1[.][0-9]+')
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

# 10b. And the SPDX judge, over one file that carries the header and one that
#      does not, because `head -1 | grep -q` is exactly the shape that passes
#      everything when the pattern is wrong.
mkdir -p "$scratch/src"
printf '// SPDX-License-Identifier: Apache-2.0\nfn a() {}\n' >"$scratch/src/has.rs"
printf 'fn b() {}\n' >"$scratch/src/lacks.rs"
named=""
for file in "$scratch"/src/*.rs; do
    head -1 "$file" | grep -q '^// SPDX-License-Identifier: Apache-2.0$' ||
        named="$named $(basename "$file")"
done
same "the SPDX judge names a source file with no header" " lacks.rs" "$named"

# 11. The install judge, over a scratch repository holding one flagged copy, one
#     unflagged one, a line carrying BOTH, and a flagged copy with `-j2` in the
#     middle. The line carrying both is the case a line filter gets wrong at
#     exit 0, and this repository's own no-hard-wrapping rule makes that line
#     shape ordinary rather than contrived. The `-j2` copy is the false
#     positive an adjacency test produces, and this run's own build guidance
#     puts `-j2` on every cargo invocation.
#
#     It reads through git, so the planted copies have to be committed to be
#     seen at all — the same reason the defect this suite first caught was
#     invisible to the local run that preceded it.
repo="$scratch/copies"
mkdir -p "$repo"
git -C "$repo" init -q
git -C "$repo" symbolic-ref HEAD refs/heads/main
printf 'build it: %s --locked\n' "$install_cmd" >"$repo/good.md"
printf 'build it: %s\n' "$install_cmd" >"$repo/bad.md"
printf 'Write `%s --locked`, never `%s`, because the flag is the point.\n' \
    "$install_cmd" "$install_cmd" >"$repo/both.md"
printf 'build it: %s -j2 --locked\n' "$install_cmd" >"$repo/jobs.md"
git -C "$repo" add -A >/dev/null 2>&1
git -C "$repo" -c user.name=fixture -c user.email=fixture@example.invalid \
    -c commit.gpgsign=false commit -q -m "one of each" >/dev/null 2>&1
bad=$(install_judge "$repo" | sed '$d' | sed 's/  */ /g' | sort | tr '\n' '|')
same "the install judge names the unflagged copy and not the flagged one" \
    "bad.md:1: $install_cmd|both.md:1: $install_cmd|" \
    "$bad"
set -- $(install_judge "$repo" | tail -1)
same "  and counts occurrences, so two on one line are two" 5 "$1"
same "  and reads \`-j2\` between the manifest and the flag as flagged" 3 "$2"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
