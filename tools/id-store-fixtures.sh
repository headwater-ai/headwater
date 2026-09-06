#!/bin/sh
# What holds the identifier claim store at `.headwater/ids/`.
#
# The store answers one question — is this number taken — and it answers it
# through git rather than through a rule. Two branches that claim one value add
# the same path with different bytes, and git refuses the merge with both
# claimants named. Two branches that claim different values touch two paths and
# merge clean. Neither outcome is written anywhere in this repository, so this
# suite is the only thing that holds them.
#
# Spec 12 refuses a check that ships with no failing fixture, and the reason
# carries to a mechanism: a mechanism nobody has seen refuse anything is a
# mechanism nobody has seen work. Every refusal below is provoked on purpose,
# and so is every clean merge, because a store that conflicted on every pair
# would be as useless as one that conflicted on none.
#
# Run it from anywhere:
#     sh tools/id-store-fixtures.sh
#
# # THE ONE CASE THAT LOOKS LIKE A PASS AND IS THE WHOLE DESIGN
#
# Case 5 merges two branches that both add the same claim path holding zero
# bytes, and it merges CLEAN. Git compares blobs before it selects a merge
# strategy, and two identical empty blobs read as the same change. So the
# claimant path written inside a claim file is not documentation: it is the
# only thing that makes the two sides differ, and therefore the only thing that
# makes the add/add conflict fire at all. Case 9 is the consequence — a claim
# file of zero bytes anywhere in the shipped store fails this suite, so a change
# that tidies the store into empty markers goes red instead of silently
# disarming every case above it.
#
# # WHAT IT NEEDS, AND WHAT IT WRITES
#
# `git` and nothing else. It does not build the engine and it does not run one,
# because the two properties it holds belong to git and to `.gitattributes`.
# Every scratch repository is made under `mktemp -d`, the directory goes on an
# interrupt, and nothing inside this checkout is written.

set -u

root=$(cd "$(dirname "$0")/.." && pwd)
store=.headwater/ids

if ! command -v git >/dev/null 2>&1; then
    echo "no \`git\` on the path, and every case here is a git merge." >&2
    exit 1
fi

if [ ! -d "$root/$store" ]; then
    echo "no claim store at \`$store\`, so the cases over the real one cannot" >&2
    echo "  run. This suite states that and stops rather than reporting a row" >&2
    echo "  of passes over a directory that is not there." >&2
    exit 1
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0

# ok NAME CONDITION-DESCRIPTION  (called with $? already decided by the caller)
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

git_q() {
    git "$@" >/dev/null 2>&1
}

# claim REPO SCHEME VALUE CLAIMANT — write one claim file, as the engine does
claim() {
    mkdir -p "$1/$store/$2"
    printf '%s\n' "$4" >"$1/$store/$2/$3"
}

# empty_claim REPO SCHEME VALUE — the shape this suite exists to refuse
empty_claim() {
    mkdir -p "$1/$store/$2"
    : >"$1/$store/$2/$3"
}

# commit REPO MESSAGE
commit() {
    git_q -C "$1" add -A
    git_q -C "$1" -c user.name=fixture -c user.email=fixture@example.invalid \
        -c commit.gpgsign=false commit -m "$2"
}

# base REPO — a repository holding one already-spent claim on `main`
base() {
    mkdir -p "$1"
    git_q -C "$1" init -q
    git_q -C "$1" symbolic-ref HEAD refs/heads/main
    claim "$1" decision_id HW-DR-0048 docs/decisions/0048-base.md
    commit "$1" "the store as it stands"
}

# branch REPO NAME — a branch off main
branch() {
    git_q -C "$1" checkout -q -b "$2" main
}

# merge REPO BRANCH — merge BRANCH into the checked-out branch, echo the status
merge() {
    git -C "$1" -c user.name=fixture -c user.email=fixture@example.invalid \
        -c commit.gpgsign=false merge --no-edit "$2" >"$scratch/out" 2>&1
    echo $?
}

# ids REPO — every decision claim the tree holds, in order, space separated
ids() {
    ( cd "$1/$store/decision_id" 2>/dev/null && ls | sort | tr '\n' ' ' | sed 's/ $//' )
}

echo "two branches claiming one value"

# 1. The case the store exists for. Both branches add the same path with
#    different bytes, so git cannot take either side.
repo="$scratch/collide"
base "$repo"
branch "$repo" a
claim "$repo" decision_id HW-DR-0049 docs/decisions/0049-from-a.md
commit "$repo" "branch a claims 0049"
branch "$repo" b
git_q -C "$repo" reset -q --hard main
claim "$repo" decision_id HW-DR-0049 docs/decisions/0049-from-b.md
commit "$repo" "branch b claims 0049"
git_q -C "$repo" checkout -q a
status=$(merge "$repo" b)
if [ "$status" = 0 ]; then
    fail "one value claimed twice refuses the merge" "the merge exited 0"
else
    pass "one value claimed twice refuses the merge"
fi

unmerged=$(git -C "$repo" ls-files -u -- "$store/decision_id/HW-DR-0049" | wc -l | tr -d ' ')
same "  and the claim path is the unmerged one, at two stages" 2 "$unmerged"

ours=$(git -C "$repo" cat-file blob ":2:$store/decision_id/HW-DR-0049" 2>/dev/null)
theirs=$(git -C "$repo" cat-file blob ":3:$store/decision_id/HW-DR-0049" 2>/dev/null)
same "  and stage 2 names the claimant on this branch" docs/decisions/0049-from-a.md "$ours"
same "  and stage 3 names the claimant on the other" docs/decisions/0049-from-b.md "$theirs"

# 2. The conflict is on the claim and not on the documents. Neither branch here
#    wrote a document at all, so nothing but the store could have refused this.
files=$(git -C "$repo" ls-files -u | awk '{print $4}' | sort -u | tr '\n' ' ' | sed 's/ $//')
same "  and nothing else in the tree is consulted" "$store/decision_id/HW-DR-0049" "$files"

echo "two branches claiming different values"

# 3. Zero false positives inside one scheme. This is the case the append-only
#    ledger form gets wrong: two lines appended at the same end of one file land
#    in one hunk and conflict although nothing collided.
repo="$scratch/disjoint"
base "$repo"
branch "$repo" a
claim "$repo" decision_id HW-DR-0049 docs/decisions/0049-from-a.md
commit "$repo" "branch a claims 0049"
branch "$repo" b
git_q -C "$repo" reset -q --hard main
claim "$repo" decision_id HW-DR-0050 docs/decisions/0050-from-b.md
commit "$repo" "branch b claims 0050"
git_q -C "$repo" checkout -q a
status=$(merge "$repo" b)
same "different values in one scheme merge clean" 0 "$status"
same "  and the union holds both" "HW-DR-0048 HW-DR-0049 HW-DR-0050" "$(ids "$repo")"

# 4. Two schemes are two directories, so they cannot interact at all.
repo="$scratch/schemes"
base "$repo"
branch "$repo" a
claim "$repo" decision_id HW-DR-0049 docs/decisions/0049-from-a.md
commit "$repo" "branch a claims a decision"
branch "$repo" b
git_q -C "$repo" reset -q --hard main
claim "$repo" obligation_record_id HW-OBL-0163 docs/obligations/0163-from-b.md
commit "$repo" "branch b claims an obligation"
git_q -C "$repo" checkout -q a
status=$(merge "$repo" b)
same "two schemes merge clean" 0 "$status"
obl=$([ -f "$repo/$store/obligation_record_id/HW-OBL-0163" ] && echo yes || echo no)
same "  and the other scheme's claim survives" yes "$obl"

echo "the long-lived branch, which the watermark form got wrong"

# 5. `main` advances past a number a branch still holds, and never takes it.
#    A watermark says 52 on both sides and pushes the branch toward a renumber
#    of a number nobody contended. The store says 0049 is free, because no file
#    claims it.
repo="$scratch/stale"
base "$repo"
branch "$repo" held
claim "$repo" decision_id HW-DR-0049 docs/decisions/0049-held.md
commit "$repo" "the branch claims 0049 and then sits"
git_q -C "$repo" checkout -q main
claim "$repo" decision_id HW-DR-0050 docs/decisions/0050-later.md
claim "$repo" decision_id HW-DR-0051 docs/decisions/0051-later.md
claim "$repo" decision_id HW-DR-0052 docs/decisions/0052-later.md
commit "$repo" "main advances without taking 0049"
status=$(merge "$repo" held)
same "a branch holding a number main skipped merges clean" 0 "$status"
same "  and no number is renumbered" \
    "HW-DR-0048 HW-DR-0049 HW-DR-0050 HW-DR-0051 HW-DR-0052" "$(ids "$repo")"
same "  and the held claim keeps its claimant" docs/decisions/0049-held.md \
    "$(cat "$repo/$store/decision_id/HW-DR-0049")"

echo "the empty claim file, which is why a claim carries a claimant"

# 6. Both sides add one path holding zero bytes. Git compares blobs before it
#    selects a strategy, so two identical empty blobs are the same change and
#    the merge is clean and silent. This is case 1 with the mechanism removed.
repo="$scratch/emptied"
base "$repo"
branch "$repo" a
empty_claim "$repo" decision_id HW-DR-0049
commit "$repo" "branch a claims 0049 with nothing in it"
branch "$repo" b
git_q -C "$repo" reset -q --hard main
empty_claim "$repo" decision_id HW-DR-0049
commit "$repo" "branch b claims 0049 with nothing in it"
git_q -C "$repo" checkout -q a
status=$(merge "$repo" b)
same "two empty claims on one value merge clean, and say nothing" 0 "$status"
size=$(wc -c <"$repo/$store/decision_id/HW-DR-0049" | tr -d ' ')
same "  and the merged claim names nobody" 0 "$size"

echo "the attribute that would restore that silence over the real store"

# 7. `merge=union` on this path concatenates two claimants into one file and
#    exits 0. The claim file is the only record of who minted the identifier,
#    and nothing in this repository ever modifies one, so a union merge loses a
#    fact that no later run can reconstruct. `.gitattributes` must set no merge
#    attribute here. Asked of the attribute rather than of the file, because
#    `.gitattributes` carries many lines and a grep of it would redden on an
#    unrelated one.
attr=$(cd "$root" && git check-attr merge -- "$store/decision_id/HW-DR-0001")
same "no merge attribute is set on a claim file" \
    "$store/decision_id/HW-DR-0001: merge: unspecified" "$attr"
attr=$(cd "$root" && git check-attr merge -- "$store/obligation_record_id/HW-OBL-0001")
same "  nor on another scheme's" \
    "$store/obligation_record_id/HW-OBL-0001: merge: unspecified" "$attr"
attr=$(cd "$root" && git check-attr merge -- "$store")
same "  nor on the directory itself" "$store: merge: unspecified" "$attr"

# 8. And the outcome the attribute would produce, measured rather than assumed,
#    so that the case above states a consequence and not a preference.
repo="$scratch/union"
base "$repo"
printf '%s\n' "$store/** merge=union" >"$repo/.gitattributes"
commit "$repo" "the attribute this suite refuses"
branch "$repo" a
claim "$repo" decision_id HW-DR-0049 docs/decisions/0049-from-a.md
commit "$repo" "branch a claims 0049"
branch "$repo" b
git_q -C "$repo" reset -q --hard main
claim "$repo" decision_id HW-DR-0049 docs/decisions/0049-from-b.md
commit "$repo" "branch b claims 0049"
git_q -C "$repo" checkout -q a
status=$(merge "$repo" b)
same "under \`merge=union\` the same collision merges clean" 0 "$status"
lines=$(wc -l <"$repo/$store/decision_id/HW-DR-0049" | tr -d ' ')
same "  and one claim file then names two claimants" 2 "$lines"

echo "the store this repository ships"

# 9. Every claim in the real store carries exactly one non-empty line. An empty
#    file disarms case 1, and two lines is what case 8 leaves behind, so this
#    one case refuses both of the ways the mechanism can be lost.
bad_empty=0
bad_lines=0
claims=0
for file in $(cd "$root/$store" && find . -type f | sort); do
    claims=$((claims + 1))
    path="$root/$store/${file#./}"
    if [ ! -s "$path" ]; then
        bad_empty=$((bad_empty + 1))
        [ "$bad_empty" -le 3 ] && echo "          empty claim: ${file#./}"
    elif [ "$(wc -l <"$path" | tr -d ' ')" != 1 ]; then
        bad_lines=$((bad_lines + 1))
        [ "$bad_lines" -le 3 ] && echo "          not one line: ${file#./}"
    fi
done
same "no claim in the shipped store is empty" 0 "$bad_empty"
same "no claim in the shipped store holds more than one line" 0 "$bad_lines"
if [ "$claims" -gt 0 ]; then
    pass "the store is not empty ($claims claims read)"
else
    fail "the store is not empty" "no claim file was read, so nothing above was measured"
fi

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
