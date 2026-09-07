#!/bin/sh
# What holds the command an outsider runs: the container recipes in
# `engine/README.md`, and the Rust floor the whole repository states.
#
# This suite follows the shape of `tools/readme-fixtures.sh`, which #611 wrote
# for the root `README.md`: `pass`/`fail`/`same`/`more_than`, a population
# guard on every judge, a `mktemp -d` scratch with a trap, and a provoked arm
# for each judge. It is a second suite rather than a group inside that one,
# because that file's subject is hardcoded to the root page, its header states
# that nothing else in this repository reads that page, and every judge it owns
# is link-shaped. Nothing there reads a code block.
#
# Run it from anywhere:
#     sh tools/engine-readme-fixtures.sh
#
# # THE CASE THIS SUITE EXISTS FOR, WHICH IS A DEAD COMMAND AND NOT A DEAD LINK
#
# On 2026-09-07 both container recipes on that page exited 101 before compiling
# a line:
#
#     error: rustc 1.85.1 is not supported by the following package:
#       ordered-float@5.5.0 requires rustc 1.90
#
# `ordered-float 5.5.0` entered `engine/Cargo.lock` transitively under `saphyr`
# on 2026-08-26. So the command this repository tells a stranger to run had been
# dead for twelve days and a hundred and eleven commits, and it was dead on the
# day the repository went public. A change on 2026-09-06 edited both of those
# very commands, to add `--locked`, without running either one.
#
# Nothing could have noticed. The image tag is a string in a Markdown file, the
# floor it pins was written in prose, and no crate of this workspace declared
# `rust-version`, so a routine `cargo update` could raise the real floor above
# the pinned image with every gate in this repository staying green. The
# obligation record `docs/obligations/0148-…` said exactly that in writing, on
# the day it became true, and nothing read it.
#
# So case group 1 is the judge this suite is for: the maximum `rust-version`
# declared across `cargo metadata --locked` must be at or below the `rust:<tag>`
# image the recipes pin. It reads the resolved dependency graph rather than the
# manifests, so a transitive floor counts. It needs no Docker, no cold build and
# no network beyond a registry index cargo has already fetched, which is the
# cost bar the issue set: the container itself is not run in CI and this suite
# does not run it either.
#
# # THE FOUR OTHER THINGS THOSE RECIPES SAY, WHICH NOTHING ELSE READS
#
# `tools/build-declaration-fixtures.sh` case 6 already reads this file, one
# occurrence at a time, for `--locked` on every cargo invocation in an image
# line, and case 9 provokes it. That is one of the ways these recipes break.
# This suite holds the four the page explains at length and nothing reads:
#
#   The SECOND recipe runs `--user "$(id -u):$(id -g)"` and the first does not.
#   Three tests set a path unwritable and require the engine to refuse it; root
#   writes through a `0444` mode, so under root those three report a defect the
#   engine does not have. Root in the first recipe is not an oversight either:
#   `rustup component add` writes into a `RUSTUP_HOME` no arbitrary user owns.
#
#   Both mount `"$(git rev-parse --show-toplevel)"`. The earlier form mounted
#   `$PWD/..`, which is the right tree only from `engine/`; from the repository
#   root it failed with `mkdir /w/engine: read-only file system`, and from a
#   worktree it mounted the tree above the worktree.
#
#   They are TWO commands in TWO blocks with two exit statuses, and a pipe
#   joining them would report the wrong one. CommonMark merges two indented
#   blocks separated by blank lines alone into a single `<pre>`, so the prose
#   standing between them is what keeps them two on the rendered page.
#
#   Neither carries `-D warnings`. `[workspace.lints.clippy]` denies a lint the
#   pinned clippy does not know, so the flag turns `unknown lint` into an error
#   in every crate there.
#
# # THE FLOOR IS STATED IN NINE FILES, AND CASE GROUP 6 HOLDS THEM TOGETHER
#
# The version number is not written once. It is in the image tag, in the
# workspace manifest, in `engine/clippy.toml`'s `msrv`, and in prose on six more
# pages. Moving one and not the rest is how the last drift happened, so the
# population is ENUMERATED out of the tree by pattern rather than listed here: a
# `rust:<v>-slim` tag, a `<v> or later` floor sentence, an `msrv` key and a
# `rust-version` key. A file added later that states the floor in one of those
# shapes joins the population with no edit to this script.
#
# `*.html` is in that glob because `site/` is the hand-built public half under
# HW-DR-0037 and nothing regenerates it. `site/tutorial/index.html` told an
# outside reader the floor was 1.85 for a day after this repository went public,
# and a glob of Markdown and manifests could not see it. Nine tracked HTML files
# are read and exactly one of them states a floor.
#
# Two exclusions, each stated rather than assumed. `engine/crates/*/fixtures/`
# holds copies of pages under test by the engine's own suites and is not prose
# anybody reads. This script is excluded from its own population, because the
# provoked arms below write version strings that are meant to be wrong.
#
# # WHAT THIS SUITE DOES NOT HOLD
#
#   It does not run the container. Nothing in CI does, which the issue ruled
#   deliberate: the run is minutes of cold compile for a floor the judge above
#   establishes in milliseconds. So a recipe that is well-formed, correctly
#   pinned and still broken for a reason no static reading can see stays green
#   here. Run the two commands by hand after changing them.
#
#   It does not read the prose of `engine/README.md` for the language rules.
#   That file sits outside the corpus root, nothing under `docs/` types it, and
#   a second copy of a rule living in a script is what this repository refuses
#   everywhere else.
#
#   It judges no cargo flag. `tools/build-declaration-fixtures.sh` owns
#   `--locked` across every site in this repository, including these recipes,
#   and a second reading of it here would be that same second copy.
#
# # WHAT IT NEEDS, AND WHAT IT WRITES
#
# `git`, `awk`, `sort -V` and `cargo`. It builds no engine and runs none, and it
# opens no socket. `cargo metadata --locked` resolves against the committed lock
# and the registry index; a checkout that has never fetched the index needs one
# network round trip for it, the same one `cargo build` already needs. Every
# scratch file is made under `mktemp -d`, the directory goes on an interrupt,
# and nothing inside this checkout is written.

set -u

root=$(cd "$(dirname "$0")/.." && pwd)
readme="$root/engine/README.md"
manifest="$root/engine/Cargo.toml"
clippy_toml="$root/engine/clippy.toml"

for tool in git awk cargo sort comm; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "no \`$tool\` on the path, and every case below needs it. This suite" >&2
        echo "  stops rather than reporting a row of passes over a set it could" >&2
        echo "  not read." >&2
        exit 1
    fi
done

for f in "$readme" "$manifest" "$clippy_toml"; do
    if [ ! -f "$f" ]; then
        echo "no \`$f\`, so every case here has nothing to judge. This suite" >&2
        echo "  stops rather than reporting a row of passes over a file that is" >&2
        echo "  not there." >&2
        exit 1
    fi
done

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

# more_than NAME FLOOR ACTUAL — a population that came back empty is a judge
# that measured nothing, and every group below carries one of these.
more_than() {
    if [ "$3" -gt "$2" ] 2>/dev/null; then
        pass "$1 ($3)"
    else
        fail "$1" "expected more than $2, got \`$3\`"
    fi
}

# norm VERSION — `1.85`, `1.85.0` and `1.85.1` are three spellings of two
# different things, and `sort -V` orders the first two as unequal. Every version
# below is padded to three components before it is compared to another.
norm() {
    echo "$1" | awk -F. '{ printf "%d.%d.%d\n", $1, ($2 == "" ? 0 : $2), ($3 == "" ? 0 : $3) }'
}

# ver_ge A B — true when A is at or above B.
ver_ge() {
    a=$(norm "$1")
    b=$(norm "$2")
    [ "$a" = "$b" ] && return 0
    [ "$(printf '%s\n%s\n' "$a" "$b" | sort -V | head -1)" = "$b" ]
}

# blocks_of FILE — every line of every INDENTED code block, as
# `index<TAB>line<TAB>text`, with the four leading spaces removed. Blocks are
# numbered from 1 in document order.
#
# An indented block cannot interrupt a paragraph, so a four-space line that
# follows a non-blank line is a lazy continuation and not code — CommonMark says
# so, and this file's tables and lists would otherwise land in the population.
# Fenced blocks are skipped outright: the recipes are indented, and a fence
# carrying an example of one is an example rather than the command.
#
# A BLANK LINE DOES NOT END AN INDENTED BLOCK. That is the rule case group 4
# rests on: two recipes with nothing but blank lines between them render as one
# `<pre>`, so a reader copies them joined and the first exit status is lost. An
# extractor that split on blank lines would report two blocks there and case 4a
# would pass over the exact page it exists to refuse. The block ends at the
# first non-blank line that is not indented by four spaces.
blocks_of() {
    awk '
        /^[ \t]*```/ { fence = 1 - fence; blank = 1; inblock = 0; next }
        fence { next }
        /^[ \t]*$/ { if (!inblock) blank = 1; next }
        /^    / {
            if (!inblock) {
                if (!blank) next
                idx++
                inblock = 1
            }
            print idx "\t" NR "\t" substr($0, 5)
            blank = 0
            next
        }
        { inblock = 0; blank = 0 }
    ' "$1"
}

# commands_of FILE — one LOGICAL command per row, as `blockindex<TAB>text`. A
# line ending in a backslash continues into the next, which is how both recipes
# are written, and a judge that read raw lines would see each recipe as three
# unrelated fragments.
commands_of() {
    blocks_of "$1" | awk -F'\t' '
        {
            text = $3
            if (cur != "" && $1 != idx) { print idx "\t" cur; cur = "" }
            idx = $1
            sub(/^[ \t]+/, "", text)
            cur = (cur == "" ? text : cur " " text)
            if (text ~ /\\$/) { sub(/[ \t]*\\$/, "", cur); next }
            print idx "\t" cur
            cur = ""
        }
        END { if (cur != "") print idx "\t" cur }
    '
}

# docker_runs FILE — the container recipes, as `blockindex<TAB>command`.
docker_runs() {
    commands_of "$1" | awk -F'\t' '$2 ~ /docker run/'
}

# floor_versions — every version number this repository states as its Rust
# floor, as `path:line:version`, enumerated from the tree by shape. Four shapes:
# a `rust:<v>-slim` image tag, a `<v> or later` sentence with `Rust`, `rustc` or
# `toolchain` in front of it, an `msrv` key and a `rust-version` key.
floor_pattern='rust:[0-9]+\.[0-9]+(\.[0-9]+)?-slim|(Rust|rustc|toolchain)[^.]{0,40}[0-9]+\.[0-9]+(\.[0-9]+)? or later|^[ #]*msrv[ ]*=[ ]*"[0-9][0-9.]*"|^[ #]*rust-version[ ]*=[ ]*"[0-9][0-9.]*"'

floor_versions() {
    # $1 is the tree to read. The file list comes from `git ls-files` so that an
    # untracked scratch copy of a page cannot join the population of a real run.
    (
        cd "$1" || exit 1
        files=$(git ls-files -- '*.md' '*.yml' '*.yaml' '*.toml' '*.html' 2>/dev/null |
            grep -v '^docs/reviews/' |
            grep -v '/fixtures/')
        [ -n "$files" ] || exit 0
        # shellcheck disable=SC2086
        grep -onE "$floor_pattern" $files 2>/dev/null | awk -F: '
            {
                where = $1 ":" $2
                text = $0
                sub(/^[^:]*:[0-9]+:/, "", text)
                if (match(text, /[0-9]+\.[0-9]+(\.[0-9]+)?/)) {
                    print where ":" substr(text, RSTART, RLENGTH)
                }
            }'
    )
}

# max_locked_rust_version — the highest `rust-version` any package in the
# resolved graph declares. The values are read with `grep` rather than a JSON
# parser, because the verdict must not need `python3`: `"rust_version"` is a
# package field and every occurrence of it in that document is one, so a reading
# that took a stray match would err toward red rather than toward green.
max_locked_rust_version() {
    cargo metadata --locked --format-version 1 --manifest-path "$1/engine/Cargo.toml" 2>/dev/null |
        grep -o '"rust_version":"[^"]*"' |
        sed 's/.*:"//; s/"$//' |
        while IFS= read -r v; do norm "$v"; done |
        sort -V |
        tail -1
}

# image_tags FILE — the `rust:<tag>-slim` tag of every container recipe, one per
# occurrence.
image_tags() {
    docker_runs "$1" | grep -oE 'rust:[0-9][0-9.]*-slim' | sed 's/^rust://; s/-slim$//'
}

echo "the floor the lock actually requires, against the image the page pins"

# 1a. The population. A selector that stopped matching reports every judge below
#     as passing over nothing, so the count is asserted before anything is
#     judged with it.
runs=$(docker_runs "$readme" | wc -l | tr -d ' ')
same "the page carries two container recipes" "2" "$runs"

blocks=$(blocks_of "$readme" | awk -F'\t' '{ print $1 }' | sort -un | wc -l | tr -d ' ')
more_than "the page carries indented code blocks" "2" "$blocks"

# 1b. Both recipes pin the same image. Two tags that disagree is a page where
#     half the instructions build and half do not, and 1c would then judge only
#     whichever one it read first.
tags=$(image_tags "$readme" | sort -u | tr '\n' ' ' | sed 's/ $//')
tagcount=$(image_tags "$readme" | wc -l | tr -d ' ')
same "both recipes pin one image" "2" "$tagcount"
pin=$tags
same "the two recipes name the same tag" "1" "$(image_tags "$readme" | sort -u | wc -l | tr -d ' ')"

# 1c. THE CASE. The resolved graph's floor against the pinned image.
maxfloor=$(max_locked_rust_version "$root")
if [ -z "$maxfloor" ]; then
    fail "the lock declares a floor this judge can read" \
        "\`cargo metadata --locked\` produced no \`rust_version\` at all, so the judge below would pass over an empty set"
else
    pass "the lock declares a floor this judge can read ($maxfloor)"
    if ver_ge "$pin" "$maxfloor"; then
        pass "the pinned image is at or above the floor the lock requires (rust:$pin, lock needs $maxfloor)"
    else
        declaring=$(cargo metadata --locked --format-version 1 --manifest-path "$root/engine/Cargo.toml" 2>/dev/null |
            grep -c -o '"rust_version":"[^"]*"' || true)
        fail "the pinned image is at or above the floor the lock requires" \
            "the recipes pin rust:$pin and the resolved graph requires $maxfloor, so both container commands exit 101 before compiling a line ($declaring packages in the graph declare a floor; \`cargo build\` inside the image names the one that raised it)"
    fi
fi

# 1d. The judge, provoked, in both directions. A judge nobody has watched refuse
#     anything is a judge nobody has watched work.
mkdir -p "$scratch/arms"
sed "s/rust:$pin-slim/rust:1.0-slim/g" "$readme" >"$scratch/arms/low.md"
lowpin=$(image_tags "$scratch/arms/low.md" | sort -u)
if ver_ge "$lowpin" "$maxfloor"; then
    fail "a pin below the lock's floor is refused" "rust:$lowpin was accepted against a floor of $maxfloor"
else
    pass "a pin below the lock's floor is refused"
fi

sed "s/rust:$pin-slim/rust:99.0-slim/g" "$readme" >"$scratch/arms/high.md"
highpin=$(image_tags "$scratch/arms/high.md" | sort -u)
if ver_ge "$highpin" "$maxfloor"; then
    pass "a pin above the lock's floor is accepted"
else
    fail "a pin above the lock's floor is accepted" "rust:$highpin was refused against a floor of $maxfloor"
fi

# 1e. The comparison itself, on the spellings that made it necessary.
same "1.85 and 1.85.0 are one version" "1.85.0" "$(norm 1.85)"
if ver_ge "1.85" "1.85.0"; then pass "1.85 is not below 1.85.0"; else fail "1.85 is not below 1.85.0" "the padding is not applied"; fi
if ver_ge "1.90" "1.85.0"; then pass "1.90 is above 1.85.0"; else fail "1.90 is above 1.85.0" "the ordering is wrong"; fi
if ver_ge "1.9" "1.85"; then fail "1.9 is below 1.85" "the comparison is lexical, not version-ordered"; else pass "1.9 is below 1.85"; fi

echo "the user each recipe runs as, which is not the same user"

# 2a. Exactly one recipe drops out of root, and it is the one running the tests.
usered=$(docker_runs "$readme" | awk -F'\t' '$2 ~ /--user/ { print $1 }' | tr '\n' ' ' | sed 's/ $//')
same "exactly one recipe passes --user" "1" "$(printf '%s' "$usered" | wc -w | tr -d ' ')"

testblock=$(docker_runs "$readme" | awk -F'\t' '$2 ~ /cargo test/ { print $1 }' | tr '\n' ' ' | sed 's/ $//')
same "the recipe that passes --user is the one that runs the tests" "$testblock" "$usered"

# 2b. The value, and not just the flag. `--user root` is a `--user`.
uservalue=$(docker_runs "$readme" | grep -oE -- '--user "\$\(id -u\):\$\(id -g\)"' | wc -l | tr -d ' ')
same "--user names the invoking user" "1" "$uservalue"

# 2c. The component install stays as root, which is the whole reason for two
#     recipes rather than one.
rustupblock=$(docker_runs "$readme" | awk -F'\t' '$2 ~ /rustup component add/ { print $1 }' | tr '\n' ' ' | sed 's/ $//')
more_than "a recipe installs the components" "0" "$(printf '%s' "$rustupblock" | wc -w | tr -d ' ')"
if [ "$rustupblock" = "$usered" ]; then
    fail "the component install does not run under --user" \
        "\`rustup component add\` writes into a RUSTUP_HOME no arbitrary user owns, so this recipe cannot drop root"
else
    pass "the component install does not run under --user"
fi

# 2d. Provoked. A page that lost `--user` on the test half is the shape that
#     turns three refusal tests green by writing through a 0444 file.
sed 's/--user "\$(id -u):\$(id -g)" //' "$readme" >"$scratch/arms/nouser.md"
same "a test recipe with no --user is named" "0" \
    "$(docker_runs "$scratch/arms/nouser.md" | awk -F'\t' '$2 ~ /--user/' | wc -l | tr -d ' ')"

echo "the tree each recipe mounts"

mounts=$(docker_runs "$readme" | grep -oE -- '-v "\$\(git rev-parse --show-toplevel\)":/w:ro' | wc -l | tr -d ' ')
same "both recipes mount the worktree root, read only" "2" "$mounts"

badmount=$(docker_runs "$readme" | grep -cE -- '-v +"?(\$PWD/\.\.|\.\.)"?:' || true)
same "neither recipe mounts a relative parent" "0" "$badmount"

# 3b. Provoked, on the exact form that was there before and failed from the
#     repository root with `mkdir /w/engine: read-only file system`.
sed 's|"\$(git rev-parse --show-toplevel)"|"$PWD/.."|g' "$readme" >"$scratch/arms/relative.md"
more_than "a relative parent mount is named" "0" \
    "$(docker_runs "$scratch/arms/relative.md" | grep -cE -- '-v +"?(\$PWD/\.\.|\.\.)"?:' || true)"

echo "two commands, two exit statuses, never one pipe"

# 4a. The recipes are in different blocks. Two indented blocks with only blank
#     lines between them are ONE `<pre>` to CommonMark, and a reader who copied
#     the rendered block would run them joined.
runblocks=$(docker_runs "$readme" | awk -F'\t' '{ print $1 }' | sort -u | wc -l | tr -d ' ')
same "the two recipes are two blocks" "2" "$runblocks"

# 4b. Prose stands between them, which is what keeps them two blocks on the
#     rendered page. Read as the line span between the last line of the first
#     recipe's block and the first line of the second's.
gap=$(blocks_of "$readme" | awk -F'\t' -v a="$(docker_runs "$readme" | awk -F'\t' '{ print $1 }' | sort -n | head -1)" \
    -v b="$(docker_runs "$readme" | awk -F'\t' '{ print $1 }' | sort -n | tail -1)" '
    $1 == a { last = $2 }
    $1 == b && first == 0 { first = $2 }
    END { print first - last - 1 }')
more_than "prose stands between the two recipes" "1" "$gap"

# 4c. No pipe inside either recipe. A shell that joins them reports the exit
#     status of the second and hides a failure in the first.
piped=$(docker_runs "$readme" | awk -F'\t' '$2 ~ /\|/ { print $1 }' | wc -l | tr -d ' ')
same "neither recipe carries a pipe" "0" "$piped"

# 4d. Provoked, on both refusals.
printf '%s\n' 'Prose.' '' '    docker run --rm rust:1.85-slim a | docker run --rm rust:1.85-slim b' >"$scratch/arms/piped.md"
more_than "a piped recipe is named" "0" \
    "$(docker_runs "$scratch/arms/piped.md" | awk -F'\t' '$2 ~ /\|/' | wc -l | tr -d ' ')"

printf '%s\n' 'Prose.' '' '    docker run --rm rust:1.85-slim a' '' '    docker run --rm rust:1.85-slim b' >"$scratch/arms/merged.md"
same "two blocks separated by a blank line alone are one block" "1" \
    "$(docker_runs "$scratch/arms/merged.md" | awk -F'\t' '{ print $1 }' | sort -u | wc -l | tr -d ' ')"

echo "the lint flag the container half must not carry"

denied=$(docker_runs "$readme" | grep -c -- '-D warnings' || true)
same "neither recipe carries -D warnings" "0" "$denied"

printf '%s\n' 'Prose.' '' '    docker run --rm rust:1.85-slim cargo clippy -- -D warnings' >"$scratch/arms/denywarn.md"
more_than "a recipe carrying -D warnings is named" "0" \
    "$(docker_runs "$scratch/arms/denywarn.md" | grep -c -- '-D warnings' || true)"

echo "the floor, stated in every file that states it"

# 6a. The population, enumerated from the tree rather than listed here.
sites=$(floor_versions "$root")
sitecount=$(printf '%s\n' "$sites" | grep -c . || true)
more_than "the tree states the floor in several places" "6" "$sitecount"

filecount=$(printf '%s\n' "$sites" | awk -F: '{ print $1 }' | sort -u | grep -c . || true)
more_than "the floor is stated across several files" "5" "$filecount"

# 6b. Every one of them names one version.
declared=$(awk -F'"' '/^rust-version[ ]*=/ { print $2; exit }' "$manifest")
msrv=$(awk -F'"' '/^msrv[ ]*=/ { print $2; exit }' "$clippy_toml")
same "the workspace manifest declares a rust-version" "$pin" "$declared"
same "clippy is told the same floor" "$pin" "$msrv"

disagreeing=$(printf '%s\n' "$sites" | awk -F: -v want="$pin" '
    $3 != "" && $3 != want { print $1 ":" $2 " says " $3 }' | sort | tr '\n' '|')
same "no file states a floor other than the pinned one" "" "$disagreeing"

# 6c. Provoked. A site that moved on its own is the drift this group exists for,
#     and the arm runs the same comparison over a scratch list.
arm=$(printf '%s\n' "engine/README.md:14:$pin" "README.md:11:1.42" |
    awk -F: -v want="$pin" '$3 != want { print $1 ":" $2 " says " $3 }' | tr '\n' '|')
same "a file left behind at another floor is named" "README.md:11 says 1.42|" "$arm"

# 6d. The `rust-version` key is the one that makes the floor machine readable,
#     so a workspace that declares it must also pass it to its members.
members=$(grep -c '^rust-version\.workspace = true' "$root"/engine/crates/*/Cargo.toml | awk -F: '{ s += $2 } END { print s }')
crates=$(ls -d "$root"/engine/crates/*/Cargo.toml | wc -l | tr -d ' ')
same "every crate inherits the declared floor" "$crates" "$members"

echo "every crate has a row, and every row is a crate"

# table_crates FILE — the crate directory of every row of the "What is here"
# table, sorted and one per line. A row's first cell is a code span holding the
# package name; every package of this workspace is `headwater-<dir>`, so the
# prefix comes off and what is left is the directory under `engine/crates/`.
#
# The selector is anchored to the section rather than to the whole page,
# because a code span naming a crate appears in prose all over this file and
# only a table row starts a line with one.
table_crates() {
    awk '
        /^## What is here[ \t]*$/ { inside = 1; next }
        inside && /^## / { inside = 0 }
        inside && /^\| *`headwater-[a-z0-9-]+` *\|/ {
            row = $0
            sub(/^\| *`headwater-/, "", row)
            sub(/`.*$/, "", row)
            print row
        }
    ' "$1" | sort -u
}

# 7a. Both populations, before either is compared to the other. A selector that
#     stopped matching reports an empty table as agreeing with an empty tree.
table_crates "$readme" >"$scratch/table-crates"
ls "$root/engine/crates" | sort -u >"$scratch/tree-crates"

rows=$(grep -c . "$scratch/table-crates" || true)
dirs=$(grep -c . "$scratch/tree-crates" || true)
more_than "the table names several crates" "20" "$rows"
more_than "the workspace holds several crates" "20" "$dirs"

# 7b. The diff, in both directions. A crate with no row is a crate an outside
#     reader cannot find from this page; a row with no crate is a name that
#     sends one to a directory that is not there. #188 found the first at 12 of
#     23, and nothing here could see it.
missing=$(comm -13 "$scratch/table-crates" "$scratch/tree-crates" | tr '\n' ' ' | sed 's/ *$//')
same "every crate in the tree has a row" "" "$missing"

invented=$(comm -23 "$scratch/table-crates" "$scratch/tree-crates" | tr '\n' ' ' | sed 's/ *$//')
same "every row names a crate in the tree" "" "$invented"

same "the table has one row per crate" "$dirs" "$rows"

# 7c. Provoked, on both refusals. The first arm deletes a row and the second
#     adds one for a crate that does not exist, and each arm is judged by the
#     same comparison as the case above it.
sed '/^| *`headwater-hash` *|/d' "$readme" >"$scratch/arms/rowgone.md"
table_crates "$scratch/arms/rowgone.md" >"$scratch/arms/rowgone.list"
same "a crate whose row was deleted is named" "hash" \
    "$(comm -13 "$scratch/arms/rowgone.list" "$scratch/tree-crates" | tr '\n' ' ' | sed 's/ *$//')"

awk '
    { print }
    !added && /^\| *`headwater-[a-z0-9-]+` *\|/ {
        print "| `headwater-nonesuch` | A crate that is not in the tree. |"
        added = 1
    }
' "$readme" >"$scratch/arms/rowextra.md"
table_crates "$scratch/arms/rowextra.md" >"$scratch/arms/rowextra.list"
same "a row naming no crate is named" "nonesuch" \
    "$(comm -23 "$scratch/arms/rowextra.list" "$scratch/tree-crates" | tr '\n' ' ' | sed 's/ *$//')"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
