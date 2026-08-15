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

judge() {
    name=$1 want_status=$2 got_status=$3 want_text=$4 got_text=$5
    if [ "$got_status" -ne "$want_status" ]; then
        printf 'FAIL %s\n  expected exit %s, got %s:\n%s\n' "$name" "$want_status" "$got_status" "$got_text"
        failed=$((failed + 1))
        return
    fi
    case $want_text in
        "") ;;
        *)
            case $got_text in
                *"$want_text"*) ;;
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

reset
printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
