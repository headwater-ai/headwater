#!/bin/sh
# What holds the commit gate and the producer under it.
#
# Spec 12 refuses a check that ships with no failing fixture, and the reason
# carries to a gate: a gate that has refused nothing is a gate nobody has seen
# work. Every refusal below is provoked on purpose, and so is every pass, because
# a gate that refuses everything is as useless as one that refuses nothing.
#
# Run it from anywhere:
#     sh .githooks/fixtures.sh
#
# # It runs over this corpus and not over a fixture tree
#
# `lifecycle.transition.not_permitted` reads the lifecycle regime a kind binds,
# and this repository binds two. A fixture taxonomy would test the rule, which
# `engine/crates/check/tests/` already does. What has never been tested is the
# path from a commit of *this* repository to that rule, so the tree below is
# this one: every tracked and untracked file, copied into a scratch repository
# whose one commit is the corpus as it stands. The prior version a case reads is
# then a real `HEAD` of a real git repository.
#
# The built engine is copied in rather than found, because the hook reads
# `engine/target/release/headwater` under the root git reports, and the scratch
# repository has its own root. `engine/target/` is ignored there, as it is here.
#
# It needs a built engine and it says so and stops when there is none. It writes
# only under a temporary directory, and it removes it on an interrupt.

set -u

root=$(cd "$(dirname "$0")/.." && pwd)
engine="$root/engine/target/release/headwater"

if [ ! -x "$engine" ]; then
    echo "no built engine, so nothing here can run."
    echo "  cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml"
    exit 1
fi

scratch=$(mktemp -d) || exit 1
# The producer writes outside the repository it describes, which is what the
# hook does with `mktemp -d`. A holding directory inside the tree would be an
# untracked file, which this producer correctly names as one the change adds.
hold=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch" "$hold"' EXIT HUP INT TERM

git -C "$root" ls-files -co --exclude-standard -z \
    | tar -C "$root" --null -T - -cf - \
    | tar -C "$scratch" -xf - || exit 1
mkdir -p "$scratch/engine/target/release"
cp "$engine" "$scratch/engine/target/release/headwater"
(
    cd "$scratch" || exit 1
    git init -q
    git add -A
    git -c user.name=fixtures -c user.email=fixtures@invalid commit -qm "the corpus as it stands" --no-verify
) || exit 1

passed=0
failed=0

# The two documents these cases move, named once. The first opens at `draft`,
# which admits `current` and `deprecated` and not `discharged`. The second
# stands at `discharged`, which the `obligation` regime gives no exit at all.
draft="docs/obligations/0126-every-asserted-document-carries-the-freshness-date-that-spec-3-says-it-cannot.md"
terminal="docs/obligations/0117-a-cached-verdict-about-an-anchor-survives-the-change-that-falsifies-it.md"

reset() {
    git -C "$scratch" reset -q --hard HEAD
    git -C "$scratch" clean -qfd
}

move() {
    file=$scratch/$1
    from=$2
    to=$3
    sed -i "s/^status: $from\$/status: $to/" "$file"
    grep -q "^status: $to\$" "$file" || {
        printf 'FAIL setup: %s did not move from %s to %s\n' "$1" "$from" "$to"
        exit 1
    }
}

# Run the commit gate in the scratch repository, the way git runs it.
gate() {
    (cd "$scratch" && sh .githooks/pre-commit 2>&1)
}

# Run the producer alone, and print the manifest it wrote.
produce() {
    rm -rf "$hold/out" "$hold/why"
    (cd "$scratch" && sh .githooks/change-manifest "$1" "$hold/out" >/dev/null 2>"$hold/why")
}

# The text match is over the words and not over the line breaks.
#
# `headwater check` lays its report out at 80 columns, so a finding message this
# engine composed as one sentence reaches a reader over two or three lines, and
# an expectation written here as one string would span a fold point. Both sides
# have every whitespace run squeezed to one space before they are compared, so a
# case states what the gate says and never where the fill broke it. The failure
# message prints the output as it arrived.
judge() {
    name=$1 want_status=$2 got_status=$3 want_text=$4 got_text=$5
    if [ "$got_status" -ne "$want_status" ]; then
        printf 'FAIL %s\n  expected exit %s, got %s:\n%s\n' "$name" "$want_status" "$got_status" "$got_text"
        failed=$((failed + 1))
        return
    fi
    flat_want=$(printf '%s' "$want_text" | tr -s '[:space:]' ' ')
    flat_got=$(printf '%s' "$got_text" | tr -s '[:space:]' ' ')
    case $want_text in
        "") ;;
        *)
            case $flat_got in
                *"$flat_want"*) ;;
                *)
                    printf 'FAIL %s\n  expected output to hold: %s\n  got:\n%s\n' "$name" "$want_text" "$got_text"
                    failed=$((failed + 1))
                    return
                    ;;
            esac
            ;;
    esac
    printf 'ok   %s\n' "$name"
    passed=$((passed + 1))
}

printf '# the gate, over this corpus\n'

reset
move "$draft" draft discharged
out=$(gate); status=$?
judge 'a movement the regime does not admit is refused at the commit' 1 "$status" \
    'lifecycle.transition.not_permitted (OB-LIFE-1): HW-OBL-0126 moved from `draft` to `discharged`' "$out"

# The instrument, and it is the whole reason the case above means anything. The
# same tree, the same engine, and no manifest: the rule reports a skip and the
# gate exits 0. So the refusal above comes from the producer and from nothing
# else in the tree.
out=$(cd "$scratch" && ./engine/target/release/headwater check --strict 2>&1); status=$?
judge 'the same tree with no change described is not refused, and says why' 0 "$status" \
    '' "$out"
out=$(cd "$scratch" && ./engine/target/release/headwater check 2>/dev/null); status=$?
judge 'and the instance reports the reason rather than a pass' 0 "$status" \
    'change-scoped-only' "$out"

reset
move "$draft" draft current
out=$(gate); status=$?
judge 'the same document moved to a state the regime admits passes' 0 "$status" '' "$out"

reset
move "$terminal" discharged current
out=$(gate); status=$?
judge 'a movement out of a terminal state is refused' 1 "$status" \
    'lifecycle.transition.not_permitted' "$out"

reset
git -C "$scratch" mv "$draft" "docs/obligations/0126-renamed.md" >/dev/null 2>&1
move "docs/obligations/0126-renamed.md" draft discharged
out=$(gate); status=$?
judge 'a renamed document is read against the version at the path it left' 1 "$status" \
    'moved from `draft` to `discharged`' "$out"

printf '# the producer, and the arms with nothing in them\n'

reset
produce HEAD
out=$(cd "$scratch" && ./engine/target/release/headwater check --change "$hold/out/manifest" 2>/dev/null); status=$?
judge 'a tree that moved nothing is a change that names nothing' 0 "$status" \
    'scoped to a change: 0 documents named, 0 added, 0 with a prior version' "$out"

reset
printf '\n// one line nothing governs\n' >> "$scratch/engine/crates/check/src/change.rs"
produce HEAD
out=$(cd "$scratch" && ./engine/target/release/headwater check --change "$hold/out/manifest" 2>/dev/null); status=$?
judge 'a change that carries no document of the corpus names what it carried' 0 "$status" \
    'engine/crates/check/src/change.rs' "$out"

reset
printf -- '---\nid: X\n---\n' > "$scratch/docs/obligations/9999-untracked.md"
produce HEAD
out=$(grep -c '^added	docs/obligations/9999-untracked.md$' "$hold/out/manifest")
judge 'a file the index does not hold is named as one the change adds' 0 $? "1" "$out"

reset
git -C "$scratch" rm -q "$terminal"
produce HEAD
out=$(cd "$scratch" && ./engine/target/release/headwater check --change "$hold/out/manifest" 2>/dev/null); status=$?
judge 'a document the change deleted is named, and the report says no row holds it' 0 "$status" \
    "$terminal" "$out"

printf '# a document that left, and the three departures that are not one\n'

# The gate rather than the verb, because this is the moment the rule exists
# for. `$terminal` stands at `discharged`, which the `obligation` regime gives
# no exit, and that regime declares `retain_terminal: true`.
reset
git -C "$scratch" rm -q "$terminal"
out=$(gate); status=$?
judge 'deleting a document at a terminal state of a retaining regime is refused' 1 "$status" \
    'lifecycle.deletion.not_permitted (OB-LIFE-4): HW-OBL-0117 stood at `discharged`' "$out"

# The instrument, on the terms the transition case above states. The same tree,
# the same engine, and no manifest: the rule reports a skip and the gate exits
# 0. So the refusal above comes from the producer and from nothing else.
out=$(cd "$scratch" && ./engine/target/release/headwater check --strict 2>&1); status=$?
judge 'the same deletion with no change described is not refused' 0 "$status" '' "$out"

# The same file, moved rather than removed. Git reports a rename as one path
# with a prior version, the census holds a row where it arrived, and the entry
# binds. A rule that read a `prior` line as a departure refuses this.
reset
git -C "$scratch" mv "$terminal" "docs/obligations/0117-renamed.md" >/dev/null 2>&1
out=$(gate); status=$?
judge 'the same document renamed at the same state is not a deletion' 0 "$status" '' "$out"

# A document that never reached a terminal state. `$draft` opens at `draft`,
# which reaches `current` and `deprecated`, so nothing about it is retained.
reset
git -C "$scratch" rm -q "$draft"
out=$(gate); status=$?
judge 'deleting a document that stands at no terminal state is not refused' 0 "$status" '' "$out"

# A file that is no document of this corpus. Its prior version does not parse as
# one, so nothing is held against a regime.
reset
git -C "$scratch" rm -q "engine/crates/check/src/change.rs"
out=$(gate); status=$?
judge 'deleting a file that is no document of this corpus is not refused' 0 "$status" '' "$out"

reset
produce "0000000000000000000000000000000000000000"
status=$?
judge 'a base revision this clone does not hold is refused rather than skipped' 1 "$status" \
    'does not hold' "$(cat "$hold/why")"

reset
touch "$scratch/a$(printf '\t')b.md"
produce HEAD
status=$?
judge 'a path a manifest line cannot carry is refused before it is written' 1 "$status" \
    'holds a tab' "$(cat "$hold/why")"

printf '# the deployed site, which no engine reads\n'

# `site/` is what Cloudflare serves as https://headwater.tools/, with no build
# step, so the bytes a commit removes from it are the bytes that go off the air.
# The clause that refuses them sits above the engine check in the hook and reads
# the index rather than the working tree, so every case below stages its change.
#
# The file the pair of cases moves is `site/_headers`, and the choice is
# measured rather than arbitrary. It is the one file under `site/` that no
# relation of this corpus names: HW-DR-0037 declares `governs` over
# `site/index.html` and `site/ns/index.html` and `traces_to`
# `site/DESIGN-BRIEF.md`, and removing any of those three raises
# `relation.target.unresolved` whatever this clause decides. So `_headers` is
# the only one whose removal the engine has no second opinion about, which is
# what lets the release case below reach exit 0 and mean what it says.
reset
git -C "$scratch" rm -q site/_headers
out=$(gate); status=$?
judge 'a commit that removes a file from the deployed site is refused' 1 "$status" \
    'deletes a file from `site/`' "$out"

# The instrument. Without it the case above only proves the gate refuses
# something about `site/`, rather than a deletion in particular.
reset
printf '\n<!-- a hand edit of the deployed site -->\n' >> "$scratch/site/index.html"
git -C "$scratch" add site/index.html
out=$(gate); status=$?
judge 'a hand edit of a page of the deployed site is not refused' 0 "$status" '' "$out"

reset
printf '<p>a new page</p>\n' > "$scratch/site/about.html"
git -C "$scratch" add site/about.html
out=$(gate); status=$?
judge 'a page added to the deployed site by hand is not refused' 0 "$status" '' "$out"

# An escape hatch nobody has watched work is an escape hatch nobody knows works.
reset
git -C "$scratch" rm -q site/_headers
out=$(cd "$scratch" && HEADWATER_ALLOW_SITE_DELETE=1 sh .githooks/pre-commit 2>&1); status=$?
judge 'the same removal with the named variable set is allowed through' 0 "$status" '' "$out"

# What the variable releases, and what it does not. It lifts this clause and
# nothing else, so a page that a document of this corpus declares `governs`
# over is refused a second time, by a rule the engine already runs. Nobody
# designed that pairing and it is worth a case, because it is the reason the
# case above names `_headers` and not `index.html`.
reset
git -C "$scratch" rm -q site/index.html
out=$(cd "$scratch" && HEADWATER_ALLOW_SITE_DELETE=1 sh .githooks/pre-commit 2>&1); status=$?
judge 'the variable releases this clause and not the rule that reads a governed path' 1 "$status" \
    'relation.target.unresolved' "$out"

reset
printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
