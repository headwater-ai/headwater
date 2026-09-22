#!/bin/sh
# What holds the obligation register: [spec 13](../../docs/spec/13-open-obligations.md)
# names, in one of its five lists, every obligation record on
# `docs/obligations/` that has not been paid, and no others.
#
# Run it from anywhere:
#     sh tools/repo/obligation-register-fixtures.sh
#
# # THE DEFECT THIS SUITE EXISTS FOR
#
# Measured on 2026-09-17 against `docs/obligations/*.md` (excluding
# `README.md`, which carries no `id`) and the `- [HW-OBL-NNNN](...)` bullet
# lines of `docs/spec/13-open-obligations.md`: the shelf held 198 records, 178
# `current` and 20 `discharged`. The file carried 181 bullet lines. Eight of
# them named a record that had already discharged — the rule the file states
# for itself is that a discharged record "stays on the shelf and leaves the
# list" — and five records standing at `current` had no bullet anywhere in the
# file. Three of the eight discharged-but-listed records had discharged in the
# five days since the file's own paragraph 2 last named the discharged set,
# which is the file's own defect recurring against itself.
#
# Nothing in `headwater check --strict` or `generate --check` reads this file
# against the shelf, because the file classifies to `obligation_register` and
# no rule of this engine compares a document's prose against a set of other
# documents' front matter. [HW-OBL-0142](../../docs/obligations/0142-the-obligation-register-states-its-own-size-by-hand-and-it-went-stale-three-times-in-two-days.md)
# is the record of that gap for the file's hand-typed counts; this suite is
# the fixture its Discharge clause asks for, over list membership rather than
# over a count.
#
# # WHAT EACH JUDGE READS
#
# THE CURRENT RECORDS are the `id` of every `docs/obligations/*.md` file,
# excluding `README.md`, whose front-matter `status` is not `discharged`. Read
# from the front matter alone, between the first pair of `---` lines, so a
# document that discusses a status word in its body is never misread.
#
# THE BULLETED RECORDS are the identifier named by every line of
# `docs/spec/13-open-obligations.md` that opens with `- [HW-OBL-` — the
# bullet-list form the file already uses under each of its five headings. A
# bare mention in running prose, such as the file's own paragraph 2 naming a
# discharged record by number, or a sentence in the sanctioned
# "is discharged and it is not in the list below" form, does not open a line
# with `- [`, and neither does the discharged-record annotation the file used
# to carry inline on a bullet. Both are deliberately outside this population,
# because a fixture that read either form would pass a discharged record still
# wearing the bullet shape, which is the defect this suite exists to catch.
#
# # THE POPULATION GUARDS, WHICH ARE NOT COUNTS
#
# Each case group opens with a floor of zero. A judge whose population came
# back empty reports green for the wrong reason, and a shelf or a file that
# stopped parsing would otherwise look clean. No case below writes down an
# expected number of records or of bullets, so none of them can go stale the
# way the prose two paragraphs above once did.
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
# The prose of a discharge sentence, or whether one exists at all for a given
# discharged record. Spec 13 states that a record "stays on the shelf and
# leaves the list", and not that every departure is narrated; some already are
# not. This suite holds only the two set memberships, in both directions.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
obligations="$root/docs/obligations"
register="$root/docs/spec/13-open-obligations.md"

for required in "$obligations" "$register"; do
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
# below asserts how many records or how many bullets there are.
more_than() {
    if [ "$3" -gt "$2" ] 2>/dev/null; then
        pass "$1 ($3)"
    else
        fail "$1" "expected more than $2, got \`$3\`"
    fi
}

# front_matter FILE — the lines between the first pair of `---` delimiters,
# written to stdout. A file with no closing delimiter yields nothing, which
# reads as a record with no id and no status rather than as a crash.
front_matter() {
    awk '
        /^---[ \t]*$/ { n++; if (n == 2) exit; next }
        n == 1 { print }
    ' "$1"
}

# current_ids DIR — one `id` per line, sorted under `LC_ALL=C`, for every
# `*.md` under DIR except `README.md` whose front-matter `status` is not
# `discharged`. Pinned because `membership_judge` compares this population
# with `comm`, which collates bytewise regardless of locale (#828).
current_ids() {
    for ci_f in "$1"/*.md; do
        [ -f "$ci_f" ] || continue
        [ "$(basename "$ci_f")" = "README.md" ] && continue
        ci_fm=$(front_matter "$ci_f")
        ci_status=$(printf '%s\n' "$ci_fm" | sed -n 's/^status:[ \t]*//p' | head -n1)
        [ "$ci_status" = "discharged" ] && continue
        ci_id=$(printf '%s\n' "$ci_fm" | sed -n 's/^id:[ \t]*//p' | head -n1)
        [ -n "$ci_id" ] && echo "$ci_id"
    done | LC_ALL=C sort -u
}

# discharged_ids DIR — the complement of current_ids: the `id` of every record
# under DIR whose `status` is `discharged`. Sorted under `LC_ALL=C`, matching
# `current_ids`, because both feed a `comm` comparison (#828).
discharged_ids() {
    for di_f in "$1"/*.md; do
        [ -f "$di_f" ] || continue
        [ "$(basename "$di_f")" = "README.md" ] && continue
        di_fm=$(front_matter "$di_f")
        di_status=$(printf '%s\n' "$di_fm" | sed -n 's/^status:[ \t]*//p' | head -n1)
        [ "$di_status" = "discharged" ] || continue
        di_id=$(printf '%s\n' "$di_fm" | sed -n 's/^id:[ \t]*//p' | head -n1)
        [ -n "$di_id" ] && echo "$di_id"
    done | LC_ALL=C sort -u
}

# bulleted_ids FILE — the identifier named by every line of FILE that opens
# with `- [HW-OBL-`, one per line, sorted under `LC_ALL=C` by the caller
# (never here: case group 1b reads the raw, unsorted count too). A duplicate
# bullet for one identifier appears twice in the raw scan and once deduped,
# which is why case group 1 below also checks the raw count against the
# deduplicated one.
bulleted_ids() {
    grep -oE '^- \[HW-OBL-[0-9]+\]' "$1" | sed -E 's/^- \[(HW-OBL-[0-9]+)\]/\1/'
}

# membership_judge CURRENT-FILE BULLETED-FILE — one line per member of the
# symmetric difference between the two populations, naming which side it is
# missing from. Empty output is set equality in both directions. `LC_ALL=C`
# on both `comm` calls, matching the `LC_ALL=C sort` that built each input:
# `comm` collates bytewise regardless of locale, and a mismatch is a silent
# wrong answer (#828).
membership_judge() {
    LC_ALL=C comm -23 "$1" "$2" | sed 's/^/a current record with no bullet line: /'
    LC_ALL=C comm -13 "$1" "$2" | sed 's/^/a bulleted line naming no current record: /'
}

# ---------------------------------------------------------------------------

# From here on, standard error is captured rather than printed: a `sort` or a
# `comm` this suite runs that writes a diagnostic (for instance `comm`'s "not
# in sorted order" warning, the exact shape a collation mismatch produces and
# that nothing used to read, #828) fails the case at the bottom of this file
# instead of leaving a line on the console nobody reads. fd 3 holds the real
# standard error so it can be restored before the summary.
exec 3>&2 2>"$scratch/stderr"

current_ids "$obligations" >"$scratch/current"
discharged_ids "$obligations" >"$scratch/discharged"
bulleted_ids "$register" | LC_ALL=C sort >"$scratch/bulleted-sorted"
bulleted_ids "$register" | LC_ALL=C sort -u >"$scratch/bulleted"

echo "the shelf's current records, and the file's bulleted lines"

# 1a. Both populations, and the discharged population, are non-empty. A judge
#     whose input came back empty passes everything below it for the wrong
#     reason.
more_than "the shelf holds records that stand at current" 0 \
    "$(wc -l <"$scratch/current" | tr -d ' ')"
more_than "the shelf holds records that stand at discharged" 0 \
    "$(wc -l <"$scratch/discharged" | tr -d ' ')"
more_than "the file carries bulleted lines" 0 \
    "$(wc -l <"$scratch/bulleted" | tr -d ' ')"

# 1b. No identifier is bulleted twice. Case group 1c below reads the
# deduplicated set, so a duplicate would otherwise pass silently.
same "no identifier carries more than one bullet line" \
    "$(wc -l <"$scratch/bulleted-sorted" | tr -d ' ')" \
    "$(wc -l <"$scratch/bulleted" | tr -d ' ')"

# 1c. Set equality, in both directions. Case group A of the decisive fixture:
#     every current record has exactly one bullet line, and no bullet line
#     names a record that is not current.
same "every current record has a bullet line, and every bulleted line names a current record" \
    "" "$(membership_judge "$scratch/current" "$scratch/bulleted" | tr '\n' '|')"

# 1d. No discharged record is bulleted. This restates 1c's second half over
#     the discharged population directly, rather than through set complement,
#     so a discharged record that is also somehow missing from current_ids
#     (an id that appears on no shelf file's front matter) cannot slip past
#     both checks unnoticed.
same "no discharged record has a bullet line" \
    "" "$(LC_ALL=C comm -12 "$scratch/discharged" "$scratch/bulleted" |
          sed 's/^/a discharged record still bulleted: /' | tr '\n' '|')"

echo
echo "the judges, provoked over scratch trees"

mkdir -p "$scratch/obl"
printf -- '---\nid: HW-OBL-0001\nstatus: current\n---\n\n# One\n' >"$scratch/obl/0001-one.md"
printf -- '---\nid: HW-OBL-0002\nstatus: current\n---\n\n# Two\n' >"$scratch/obl/0002-two.md"
printf -- '---\nid: HW-OBL-0003\nstatus: discharged\n---\n\n# Three\n\nstatus: current is a word in this body, not in front matter.\n' \
    >"$scratch/obl/0003-three.md"
printf -- 'not a record, and carries no front matter at all\n' >"$scratch/obl/README.md"

printf '# 13 — Open obligations\n\n## A heading\n\n- [HW-OBL-0001](../obligations/0001-one.md) — One\n- [HW-OBL-0002](../obligations/0002-two.md) — Two\n\n[HW-OBL-0003](../obligations/0003-three.md) is discharged and it is not in the list above.\n' \
    >"$scratch/reg-clean.md"

scratch_current=$(current_ids "$scratch/obl")
scratch_discharged=$(discharged_ids "$scratch/obl")

same "the scratch shelf's own front matter is read, not its body" \
    "HW-OBL-0001
HW-OBL-0002" "$scratch_current"
same "the scratch shelf's discharged record is read as discharged" \
    "HW-OBL-0003" "$scratch_discharged"

printf '%s\n' "$scratch_current" >"$scratch/s-current"
bulleted_ids "$scratch/reg-clean.md" | LC_ALL=C sort -u >"$scratch/s-bulleted-clean"
same "the clean scratch pair passes set equality in both directions" \
    "" "$(membership_judge "$scratch/s-current" "$scratch/s-bulleted-clean" | tr '\n' '|')"

# 2a. Drop the bullet for a current record — case group A's own defect, over
#     the direction it exists for. This is the scratch arm that has to redden
#     before the repair below existed, and the run below shows it doing so.
printf '# 13 — Open obligations\n\n## A heading\n\n- [HW-OBL-0002](../obligations/0002-two.md) — Two\n\n[HW-OBL-0003](../obligations/0003-three.md) is discharged and it is not in the list above.\n' \
    >"$scratch/reg-drop.md"
bulleted_ids "$scratch/reg-drop.md" | LC_ALL=C sort -u >"$scratch/s-bulleted-drop"
same "dropping a current record's bullet reddens the membership judge" \
    "a current record with no bullet line: HW-OBL-0001|" \
    "$(membership_judge "$scratch/s-current" "$scratch/s-bulleted-drop" | tr '\n' '|')"

# 2b. Bullet a discharged record — case group B's own defect. The exact shape
#     of the original: an ordinary bullet, with no inline annotation, for a
#     record whose front matter already reads `discharged`.
printf '# 13 — Open obligations\n\n## A heading\n\n- [HW-OBL-0001](../obligations/0001-one.md) — One\n- [HW-OBL-0002](../obligations/0002-two.md) — Two\n- [HW-OBL-0003](../obligations/0003-three.md) — Three\n' \
    >"$scratch/reg-add.md"
bulleted_ids "$scratch/reg-add.md" | LC_ALL=C sort -u >"$scratch/s-bulleted-add"
same "bulleting a discharged record reddens the membership judge" \
    "a bulleted line naming no current record: HW-OBL-0003|" \
    "$(membership_judge "$scratch/s-current" "$scratch/s-bulleted-add" | tr '\n' '|')"
printf '%s\n' "$scratch_discharged" >"$scratch/s-discharged"
same "bulleting a discharged record reddens the direct discharged check" \
    "a discharged record still bulleted: HW-OBL-0003|" \
    "$(LC_ALL=C comm -12 "$scratch/s-discharged" "$scratch/s-bulleted-add" |
       sed 's/^/a discharged record still bulleted: /' | tr '\n' '|')"

# 2c. A bare mention in prose, in the sanctioned "is discharged and it is not
#     in the list above" form, is not a bullet, so it never enters either
#     population. This is what keeps case group A from matching the file's
#     own paragraph 2 or its sanctioned delisting sentences, which was the
#     false-pass this suite's own design rules out.
same "a sanctioned discharge sentence is not read as a bullet" \
    "0" "$(bulleted_ids "$scratch/reg-clean.md" | grep -c 'HW-OBL-0003' || true)"

# 2d. The inline `(**discharged**: ...)` annotation the file used to carry on
#     HW-OBL-0174's bullet does not remove the line from the bulleted
#     population, because the line still opens with `- [HW-OBL-`. This is why
#     Done-when asks for the line to be deleted rather than annotated.
printf '# 13 — Open obligations\n\n## A heading\n\n- [HW-OBL-0003](../obligations/0003-three.md) — Three (**discharged**: paid)\n' \
    >"$scratch/reg-annotated.md"
same "an annotated bullet for a discharged record still counts as bulleted" \
    "HW-OBL-0003" "$(bulleted_ids "$scratch/reg-annotated.md")"

exec 2>&3 3>&-
same "no sort or comm in this run wrote to standard error" \
    "" "$(tr '\n' '|' <"$scratch/stderr")"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
