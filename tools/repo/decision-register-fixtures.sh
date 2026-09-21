#!/bin/sh
# What holds the decision register: [docs/spec/09-decisions.md](../../docs/spec/09-decisions.md)
# names no decision record that the shelf does not carry, and it no longer
# claims to supersede the generated register that lists every record.
#
# Run it from anywhere:
#     sh tools/repo/decision-register-fixtures.sh
#
# # THE DEFECT THIS SUITE EXISTS FOR
#
# Measured on 2026-09-21 against `docs/decisions/*.md` (excluding
# `README.md`) and the `[HW-DR-NNNN]` citations of
# `docs/spec/09-decisions.md`: the shelf held 80 records and the hand-authored
# register named 37 of them. Its front matter still declared
# `supersedes: HW-REG-open-questions`, an edge that claims a completeness the
# file did not have, against the generated register at
# `docs/spec/09-open-questions.md`, which `headwater generate` writes with one
# heading for every one of the 80 and `generate --check` holds whole or not at
# all. Two readers relied on the hand register's last heading to pick the next
# decision number on 2026-09-11 and both minted `HW-DR-0044`, a number the
# shelf already carried; a complete register would have shown the shelf past
# that number.
#
# [HW-DR-0081](../../docs/decisions/0081-the-hand-authored-decision-register-indexes-a-subset-of-the-shelf-and-stops-asserting-it-supersedes-the-complete-one.md)
# rules that the hand register stays a curated selection rather than a
# complete index, and that it stops claiming otherwise. This suite is the
# check that ruling asks for: it never requires every shelf record to have a
# heading here, because that is not what the file is for, but it does require
# that every record the file cites still exists, and that the file never again
# declares the `supersedes` edge that misled the two readers above.
#
# Nothing in `headwater check --strict` or `generate --check` reads this file
# against the shelf or against its own `supersedes` edge, because no rule of
# this engine compares a document's prose or its front matter against a
# ruling stated in another document.
#
# # WHAT EACH JUDGE READS
#
# THE SHELF RECORDS are the `id` of every `docs/decisions/*.md` file,
# excluding `README.md`, read from the front matter alone, between the first
# pair of `---` lines, so a document that discusses an identifier in its body
# is never misread.
#
# THE NAMED RECORDS are every identifier cited in the bracket form
# `[HW-DR-NNNN]` anywhere in `docs/spec/09-decisions.md`, deduplicated. That is
# the form every citation in the file already takes, in running prose and not
# only on a line of its own, because a hand-authored register argues each
# decision in prose rather than bulleting it.
#
# THE SUPERSEDES CLAIM is whether `docs/spec/09-decisions.md`'s front matter
# declares `HW-REG-open-questions` under `supersedes`.
#
# # THE POPULATION GUARD, WHICH IS NOT A COUNT
#
# The case group over named records opens with a floor of zero. A judge whose
# population came back empty reports green for the wrong reason, and a file
# that stopped parsing would otherwise look clean. No case below writes down
# an expected number of shelf records or of named records, so none of them can
# go stale the way this file's own claim once did.
#
# # WHAT IT NEEDS, AND WHAT IT WRITES
#
# `awk`, `sed` and `comm`. It builds no engine and runs none, which is why it
# can be a required step that costs nothing but this file and the shelf.
# Every scratch tree is made under `mktemp -d`, the directory goes on an
# interrupt, and nothing inside this checkout is written.
#
# # WHAT THIS SUITE DOES NOT HOLD
#
# Whether every shelf record has a heading here. [HW-DR-0081] rules that the
# file is a deliberate subset, so the 43 records it does not name are not a
# defect this suite reports. It also does not hold the Q-number of a heading
# against the number of the record it cites: the Q prefix already lapsed on
# new records before this suite existed, and that convention is a separate
# question HW-DR-0081 does not rule on.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
decisions="$root/docs/decisions"
register="$root/docs/spec/09-decisions.md"

for required in "$decisions" "$register"; do
    if [ ! -e "$required" ]; then
        echo "no \`${required#$root/}\`, so every case here has nothing to judge." >&2
        echo "  This suite stops rather than reporting a row of passes over a" >&2
        echo "  population it could not read." >&2
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
# that measured nothing. This is a floor and never an expected value: no case
# below asserts how many records the shelf or the register carries.
more_than() {
    if [ "$3" -gt "$2" ] 2>/dev/null; then
        pass "$1 ($3)"
    else
        fail "$1" "expected more than $2, got \`$3\`"
    fi
}

# front_matter FILE — the lines between the first pair of `---` delimiters,
# written to stdout. A file with no closing delimiter yields nothing, which
# reads as a document with no id and no relations rather than as a crash.
front_matter() {
    awk '
        /^---[ \t]*$/ { n++; if (n == 2) exit; next }
        n == 1 { print }
    ' "$1"
}

# shelf_ids DIR — one `id` per line, sorted, for every `*.md` under DIR except
# `README.md`.
shelf_ids() {
    for si_f in "$1"/*.md; do
        [ -f "$si_f" ] || continue
        [ "$(basename "$si_f")" = "README.md" ] && continue
        si_id=$(front_matter "$si_f" | sed -n 's/^id:[ \t]*//p' | head -n1)
        [ -n "$si_id" ] && echo "$si_id"
    done | sort -u
}

# named_ids FILE — the identifier named by every `[HW-DR-NNNN]` citation of
# FILE, deduplicated and sorted. A citation inside running prose counts the
# same as one on a line of its own.
named_ids() {
    grep -oE '\[HW-DR-[0-9]+\]' "$1" | sed -E 's/^\[(HW-DR-[0-9]+)\]/\1/' | sort -u
}

# supersedes_open_questions FILE — prints `yes` if FILE's front matter
# declares `HW-REG-open-questions` under `supersedes`, and prints nothing
# otherwise.
supersedes_open_questions() {
    front_matter "$1" | awk '
        /^[ \t]*supersedes:[ \t]*$/ { insup = 1; next }
        insup && /^[ \t]*-/ { if ($0 ~ /HW-REG-open-questions/) found = 1; next }
        { insup = 0 }
        END { if (found) print "yes" }
    '
}

# missing_from_shelf SHELF-FILE NAMED-FILE — one line per named record that
# the shelf does not carry. Empty output means every named record still
# exists.
missing_from_shelf() {
    comm -23 "$2" "$1"
}

# ---------------------------------------------------------------------------

shelf_ids "$decisions" >"$scratch/shelf"
named_ids "$register" >"$scratch/named"

echo "the shelf's records, and the register's citations"

# 1a. Both populations are non-empty.
more_than "the shelf holds decision records" 0 \
    "$(wc -l <"$scratch/shelf" | tr -d ' ')"
more_than "the register cites decision records" 0 \
    "$(wc -l <"$scratch/named" | tr -d ' ')"

# 1b. Every record the register cites still exists on the shelf. This is the
#     one direction the ruling asks for: the register may name fewer records
#     than the shelf holds, but never a record the shelf does not.
same "every record the register cites still exists on the shelf" \
    "" "$(missing_from_shelf "$scratch/shelf" "$scratch/named" | tr '\n' '|')"

# 1c. The register's front matter carries no `supersedes` edge toward the
#     generated register. That edge is the state that misled the two readers
#     of 2026-09-11, and HW-DR-0081 rules it dropped.
same "the register does not claim to supersede the generated one" \
    "" "$(supersedes_open_questions "$register")"

echo
echo "the judges, provoked over scratch trees"

mkdir -p "$scratch/dec"
printf -- '---\nid: HW-DR-0001\nstatus: current\n---\n\n# One\n' >"$scratch/dec/0001-one.md"
printf -- '---\nid: HW-DR-0002\nstatus: current\n---\n\n# Two\n' >"$scratch/dec/0002-two.md"
printf -- 'not a record, and carries no front matter at all\n' >"$scratch/dec/README.md"

scratch_shelf=$(shelf_ids "$scratch/dec")
same "the scratch shelf's own front matter is read, not its body" \
    "HW-DR-0001
HW-DR-0002" "$scratch_shelf"
printf '%s\n' "$scratch_shelf" >"$scratch/s-shelf"

printf '# 9 — The decision register\n\n## Q1\n\nThe record is [HW-DR-0001](../decisions/0001-one.md).\n\n## Q2\n\nThe record is [HW-DR-0002](../decisions/0002-two.md).\n' \
    >"$scratch/reg-clean.md"
named_ids "$scratch/reg-clean.md" >"$scratch/s-named-clean"
same "the clean scratch pair names no record the shelf lacks" \
    "" "$(missing_from_shelf "$scratch/s-shelf" "$scratch/s-named-clean" | tr '\n' '|')"

# 2a. Cite a record the scratch shelf does not carry — the defect this suite
#     exists for, over the direction it holds.
printf '# 9 — The decision register\n\n## Q1\n\nThe record is [HW-DR-0001](../decisions/0001-one.md).\n\n## Q9\n\nThe record is [HW-DR-0009](../decisions/0009-nine.md).\n' \
    >"$scratch/reg-dangling.md"
named_ids "$scratch/reg-dangling.md" >"$scratch/s-named-dangling"
same "citing a record the shelf does not carry reddens the membership judge" \
    "HW-DR-0009|" \
    "$(missing_from_shelf "$scratch/s-shelf" "$scratch/s-named-dangling" | tr '\n' '|')"

# 2b. A register with no `supersedes` edge at all passes the claim check.
printf -- '---\nid: HW-REG-decisions\n---\n\n# 9 — The decision register\n' \
    >"$scratch/reg-no-edge.md"
same "a register with no supersedes edge passes the claim check" \
    "" "$(supersedes_open_questions "$scratch/reg-no-edge.md")"

# 2c. A register that declares the `supersedes` edge reddens the claim check
#     — the exact shape this file carried before HW-DR-0081.
printf -- '---\nid: HW-REG-decisions\nrelations:\n  supersedes:\n    - HW-REG-open-questions\n---\n\n# 9 — The decision register\n' \
    >"$scratch/reg-superseding.md"
same "declaring the supersedes edge reddens the claim check" \
    "yes" "$(supersedes_open_questions "$scratch/reg-superseding.md")"

# 2d. A `supersedes` edge naming a different target does not trip the claim
#     check, because the claim is specifically about the generated register
#     this file used to overclaim against.
printf -- '---\nid: HW-REG-decisions\nrelations:\n  supersedes:\n    - HW-REG-something-else\n---\n\n# 9 — The decision register\n' \
    >"$scratch/reg-other-edge.md"
same "a supersedes edge toward a different target does not trip the claim check" \
    "" "$(supersedes_open_questions "$scratch/reg-other-edge.md")"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
