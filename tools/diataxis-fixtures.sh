#!/bin/sh
# What holds the `diataxis-site` entry of the canonical taxonomy library: its
# resolution, its constructors, its worked shape corpus, and the external
# corpus that admission criterion 4 asks for.
#
# Run it from anywhere:
#     sh tools/diataxis-fixtures.sh
#
# # THE DEFECT THIS SUITE EXISTS FOR
#
# The entry shipped with a five-row case table in `fixtures/README.md` and
# nothing that ran it. That table is prose, `docs/taxonomies/**` is excluded
# from this repository's census, and so a defect inside the fixture corpus
# reaches no gate at all. Measured on 2026-09-11 against `58f46a8d`:
# `engine/crates/check/fixtures/corpus.checks` records 0 findings under this
# entry, and the census records its 15 paths as excluded. A planted defect had
# in fact been committed into the worked corpus — `get-started.md` carried
# `kind: reference` — so a strict run over that corpus exited 1 and the corpus
# never demonstrated the clean state it is for. Nothing reported it for two
# days. The document itself still typed `tutorial`: a shelf decides the kind,
# and the field is a conflicting restatement rather than a retyping.
#
# So the plant lives here now and never in the tree. Case group 2 removes it
# from the corpus and case group 3 injects it, which is the only arrangement
# where both the clean corpus and the refusal are held by something that runs.
#
# # WHAT CRITERION 4 ASKS, AND WHAT IT DOES NOT ASK
#
# Criterion 4 of `docs/taxonomies/README.md` asks for "at least one external
# real or realistic corpus, typed by the entry and recorded with its source,
# revision, paths, and run". It asks for a recorded run. It does not ask for a
# clean one, and the run is not clean: the four pinned sources are real
# documentation from two projects that never met this taxonomy, and the kinds'
# section contracts refuse all four of them. Case group 4 records that as a
# measurement rather than repairing it. A fixture corpus edited until it passes
# is a fixture corpus that has stopped being external.
#
# The one property that is asserted rather than recorded is byte identity: the
# assembled document below its front-matter block is the pinned file, byte for
# byte. A body touched to make a section contract pass would void the whole
# measurement, and case group 4 is what notices.
#
# # WHAT EACH JUDGE READS
#
# No list of kinds, shelves, source paths or counts is written in this file.
# Every population is enumerated out of a declaration:
#
#   THE CONCRETE KINDS come from the `add:` block of the entry's `bundle.yml` —
#   the keys under `kinds.` — which is what the resolver itself reads.
#
#   THE MODE MAP comes from the source table of `fixtures/README.md`: one row
#   per kind, naming the pinned file and the path the assembled document takes.
#   Case 4.0 asserts SET EQUALITY between that table's kinds and the bundle's
#   kinds, in both directions, so a kind added with no pinned source reddens
#   and a row for a kind the bundle dropped reddens too.
#
#   THE IDENTIFIER of each assembled document is built from the `pattern` of
#   `identifier_schemes.<kind>_id` in the same `bundle.yml`, not written here.
#
# # THE POPULATION GUARDS, WHICH ARE NOT COUNTS
#
# Each group opens with a floor of zero. A judge whose population came back
# empty reports green for the wrong reason. No expected number of kinds, rows
# or findings is written down here, so none can go stale. The finding
# denominators of case group 4 are printed and never asserted, for the same
# reason: a warning count is a property of somebody else's prose.
#
# # WHAT IT NEEDS, AND WHAT IT WRITES
#
# `awk`, `sed`, `cmp` and a built engine. Every scratch tree is made under
# `mktemp -d`, the directory goes on an interrupt, and nothing inside this
# checkout is written.

set -u

root=$(cd "$(dirname "$0")/.." && pwd)
entry=docs/taxonomies/diataxis-site
bundle="$root/$entry/bundle.yml"
fixtures="$root/$entry/fixtures"

engine=""
if [ -x "$root/engine/target/release/headwater" ]; then
    engine="$root/engine/target/release/headwater"
fi
if [ -x "$root/engine/target/dev-release/headwater" ] &&
    { [ -z "$engine" ] || [ "$root/engine/target/dev-release/headwater" -nt "$engine" ]; }; then
    engine="$root/engine/target/dev-release/headwater"
fi
if [ -z "$engine" ]; then
    echo "no engine at engine/target/{release,dev-release}/headwater, and every" >&2
    echo "  case below runs one. This suite states that and stops rather than" >&2
    echo "  reporting a row of passes over a binary that is not there." >&2
    echo "  Build one: cargo build --profile dev-release -p headwater-cli \\" >&2
    echo "      --manifest-path engine/Cargo.toml --locked" >&2
    exit 1
fi

for needed in "$bundle" "$fixtures/README.md" "$fixtures/corpus" "$fixtures/sources"; do
    if [ ! -e "$needed" ]; then
        echo "missing $needed, so the cases over the entry cannot run." >&2
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
    [ $# -gt 1 ] && echo "          $2"
}

judge() { # name, expected, actual
    if [ "$2" = "$3" ]; then
        pass "$1"
    else
        fail "$1" "expected [$2], read [$3]"
    fi
}

# --- the populations, read out of the declarations -------------------------

# Every key under `kinds.` in the entry's add block.
bundle_kinds() {
    awk '
        /^add:/ { in_add = 1; next }
        /^[a-z_]+:/ { in_add = 0 }
        in_add && /^  kinds\./ {
            k = $0
            sub(/^  kinds\./, "", k)
            sub(/:.*/, "", k)
            print k
        }
    ' "$bundle" | sort -u
}

# The source table of `fixtures/README.md`: kind, pinned source, assembled path.
# A row is a table line whose first cell is a backticked kind name and whose
# third cell names a file under `sources/`.
mode_rows() {
    awk -F'|' '
        /^\|/ {
            k = $2; s = $4; d = $5
            gsub(/[ `]/, "", k); gsub(/[ `]/, "", s); gsub(/[ `]/, "", d)
            if (s ~ /^sources\//) { print k "\t" s "\t" d }
        }
    ' "$fixtures/README.md"
}

# The `pattern` of `identifier_schemes.<scheme>` in the entry's add block.
scheme_pattern() {
    awk -v want="$1" '
        /^add:/ { in_add = 1; next }
        /^[a-z_]+:/ { in_add = 0 }
        in_add && $0 ~ "^  identifier_schemes\\." want ":" { hit = 1; next }
        in_add && /^  [a-z_]/ { hit = 0 }
        hit && /pattern:/ {
            p = $0
            sub(/^[^:]*: */, "", p)
            gsub(/"/, "", p)
            print p
            exit
        }
    ' "$bundle"
}

# A scratch root that selects the entry. $1 = destination, $2 = bundle list.
make_root() {
    at=$1
    bundles=$2
    mkdir -p "$at/packages" "$at/.headwater" "$at/docs"
    cp -r "$root/packages/headwater-standard" "$at/packages/"
    {
        echo 'taxonomy:'
        echo '  package: headwater/standard'
        sed -n 's/^  version: /  version: /p' "$root/.headwater/taxonomy.yml" | head -n 1
        sed -n 's/^  digest: /  digest: /p' "$root/.headwater/taxonomy.yml" | head -n 1
        echo "  bundles: [$bundles]"
        echo '  overlay: .headwater/overlay.yml'
        echo
        echo 'corpus:'
        echo '  root: docs'
    } > "$at/.headwater/taxonomy.yml"
    # The overlay supplies exactly the namespaces the selection is missing, and
    # it is derived rather than listed: an overlay that declares a namespace on
    # a scheme the selection does not carry is itself a refusal, so a written
    # list would redden every time the bundle set changed.
    echo 'add: {}' > "$at/.headwater/overlay.yml"
    (cd "$at" && "$engine" taxonomy validate) > "$scratch/ns.out" 2> "$scratch/ns.err"
    sed -n 's/.*`identifier_schemes\.\([a-z_]*\)`: identifier integrity: carries no namespace.*/\1/p' \
        "$scratch/ns.out" "$scratch/ns.err" | sort -u > "$scratch/ns.list"
    {
        echo 'add:'
        while read -r scheme; do
            [ -n "$scheme" ] || continue
            echo "  identifier_schemes.${scheme}.namespace: DX"
        done < "$scratch/ns.list"
    } > "$at/.headwater/overlay.yml"
}

kinds=$(bundle_kinds)
kind_floor=$(printf '%s\n' "$kinds" | grep -c . || true)
rows=$(mode_rows)
row_floor=$(printf '%s\n' "$rows" | grep -c . || true)

echo "diataxis-site fixtures, against $engine"
echo

echo "population guards"
if [ "$kind_floor" -eq 0 ]; then
    fail "the bundle declares at least one concrete kind" "none parsed out of $entry/bundle.yml"
else
    pass "the bundle declares at least one concrete kind ($kind_floor read)"
fi
if [ "$row_floor" -eq 0 ]; then
    fail "the source table carries at least one row" "none parsed out of $entry/fixtures/README.md"
else
    pass "the source table carries at least one row ($row_floor read)"
fi
[ "$kind_floor" -eq 0 ] && exit 1
[ "$row_floor" -eq 0 ] && exit 1

echo
echo "case group 1 — the source table covers the kinds, in both directions"
printf '%s\n' "$rows" | cut -f1 | sort -u > "$scratch/table-kinds"
printf '%s\n' "$kinds" > "$scratch/bundle-kinds"
judge "every kind the bundle declares has a pinned source" "" \
    "$(comm -23 "$scratch/bundle-kinds" "$scratch/table-kinds" | tr '\n' ' ' | sed 's/ *$//')"
judge "every row of the source table names a kind the bundle declares" "" \
    "$(comm -13 "$scratch/bundle-kinds" "$scratch/table-kinds" | tr '\n' ' ' | sed 's/ *$//')"

echo
echo "case group 2 — resolution and the constructors (criterion 7)"

isolated="$scratch/isolated"
make_root "$isolated" "diataxis-site"
(cd "$isolated" && "$engine" taxonomy validate) > "$scratch/validate.out" 2> "$scratch/validate.err"
judge "isolated selection validates" 0 "$?"
(cd "$isolated" && "$engine" taxonomy resolve) > "$scratch/resolve.out" 2> "$scratch/resolve.err"
judge "isolated selection resolves" 0 "$?"
if [ -f "$isolated/.headwater/taxonomy.lock" ]; then
    pass "the isolated run wrote a lock, which is a validated taxonomy"
else
    fail "the isolated run wrote a lock" "no .headwater/taxonomy.lock"
fi

composed="$scratch/composed"
make_root "$composed" "design-spec, evidence-and-obligation, decision-record, diataxis-site"
(cd "$composed" && "$engine" taxonomy resolve) > "$scratch/composed.out" 2> "$scratch/composed.err"
judge "selection beside three library entries resolves" 0 "$?"

for k in $kinds; do
    (cd "$isolated" && "$engine" new "$k" --title "A worked $k" --summary "One sentence about a worked $k.") \
        > "$scratch/new-$k.out" 2> "$scratch/new-$k.err"
    judge "headwater new $k succeeds" 0 "$?"
done

echo
echo "case group 3 — the overlay collision, and what it is not"
collided="$scratch/collided"
make_root "$collided" "diataxis-site"
cp "$root/.headwater/overlay.yml" "$collided/.headwater/overlay.yml"
(cd "$collided" && "$engine" taxonomy resolve) > "$scratch/collide.out" 2> "$scratch/collide.err"
code=$?
if [ "$code" -eq 0 ]; then
    fail "an overlay that reaches the entry's addresses refuses" "resolve exited 0"
else
    pass "an overlay that reaches the entry's addresses refuses (exit $code)"
fi
first=$(sed -n 's/.*both reach `\([^`]*\)`.*/\1/p' "$scratch/collide.err" | head -n 1)
judge "the first leaf it names is the purpose both sides declare" \
    "purposes.procedure.intent" "$first"

# Criterion 6 binds an entry against every other entry, and not against an
# adopter overlay. The collision above is with this repository's own overlay,
# which is why it is not an admission failure. The next case is the criterion.
selfaddr=$(awk '
    /^add:/ { in_add = 1; next }
    /^[a-z_]+:/ { in_add = 0 }
    in_add && /^  [a-z_]/ { a = $0; sub(/^  /, "", a); sub(/:.*/, "", a); print a }
' "$bundle" | sort -u)
printf '%s\n' "$selfaddr" > "$scratch/self-addr"
overlap=""
for other in "$root"/docs/taxonomies/*/bundle.yml; do
    name=$(basename "$(dirname "$other")")
    [ "$name" = "diataxis-site" ] && continue
    o=$(awk '
        /^add:/ { in_add = 1; next }
        /^[a-z_]+:/ { in_add = 0 }
        in_add && /^  [a-z_]/ { a = $0; sub(/^  /, "", a); sub(/:.*/, "", a); print a }
    ' "$other" | sort -u)
    printf '%s\n' "$o" > "$scratch/other-addr"
    shared=$(comm -12 "$scratch/self-addr" "$scratch/other-addr" | tr '\n' ' ')
    [ -n "$(printf '%s' "$shared" | tr -d ' ')" ] && overlap="$overlap$name:$shared "
done
judge "criterion 6: the address set is disjoint from every other entry" "" \
    "$(printf '%s' "$overlap" | sed 's/ *$//')"
judge "criterion 3: the entry writes no override and no remove" "" \
    "$(grep -cE '^(override|remove):' "$bundle" | sed 's/^0$//')"

echo
echo "case group 4 — the worked shape corpus is clean, and the plant is here"
shape="$scratch/shape"
make_root "$shape" "diataxis-site"
rm -rf "$shape/docs"
cp -r "$fixtures/corpus/docs" "$shape/docs"
(cd "$shape" && "$engine" taxonomy resolve) > "$scratch/shape-resolve.out" 2>&1
(cd "$shape" && "$engine" check --strict --no-cache) > "$scratch/shape.out" 2> "$scratch/shape.err"
judge "the committed worked corpus passes a strict run" 0 "$?"
for k in $kinds; do
    n=$(sed -n "s/^ *\([0-9][0-9]*\) typed $k$/\1/p" "$scratch/shape.out" | head -n 1)
    judge "the committed corpus holds a document of kind $k" 1 "${n:-0}"
done

planted="$scratch/planted"
make_root "$planted" "diataxis-site"
rm -rf "$planted/docs"
cp -r "$fixtures/corpus/docs" "$planted/docs"
(cd "$planted" && "$engine" taxonomy resolve) > "$scratch/planted-resolve.out" 2>&1
victim=$(find "$planted/docs" -name '*.md' | head -n 1)
for f in "$planted"/docs/tutorials/*.md; do
    [ -f "$f" ] && victim=$f
done
sed -i '0,/^---$/!{0,/^---$/s/^---$/kind: reference\n---/}' "$victim"
(cd "$planted" && "$engine" check --strict --no-cache) > "$scratch/planted.out" 2> "$scratch/planted.err"
code=$?
if [ "$code" -eq 0 ]; then
    fail "the planted kind restatement fails a strict run" "check --strict exited 0"
else
    pass "the planted kind restatement fails a strict run (exit $code)"
fi
if grep -q 'shelf.placement_is_primary' "$scratch/planted.out" "$scratch/planted.err"; then
    pass "the plant reports shelf.placement_is_primary"
else
    fail "the plant reports shelf.placement_is_primary" "no such rule in the report"
fi

echo
echo "case group 5 — the external corpus of criterion 4"
external="$scratch/external"
make_root "$external" "diataxis-site"
rm -rf "$external/docs"
mkdir -p "$external/docs"
printf '%s\n' "$rows" | while IFS="$(printf '\t')" read -r k src dest; do
    [ -n "$k" ] || continue
    pattern=$(scheme_pattern "${k}_id")
    slug=$(basename "$dest" .md)
    id=$(printf '%s' "$pattern" | sed -e 's/{namespace}/DX/' -e "s/{slug}/$slug/")
    mkdir -p "$(dirname "$external/$dest")"
    {
        printf -- '---\n'
        printf 'id: %s\n' "$id"
        printf 'title: %s\n' "$slug"
        printf 'status: current\n'
        printf 'status_since: 2026-09-11\n'
        printf 'last_verified: 2026-09-11\n'
        printf 'summary: A pinned external document, assembled with front matter and no other edit.\n'
        printf -- '---\n\n'
        cat "$fixtures/$src"
    } > "$external/$dest"
done

# Byte identity, which is the assertion the whole group rests on: strip the
# assembled document's first front-matter block and the blank line after it,
# and compare what is left against the pinned file.
printf '%s\n' "$rows" | while IFS="$(printf '\t')" read -r k src dest; do
    [ -n "$k" ] || continue
    awk 'NR == 1 && $0 == "---" { infm = 1; next }
         infm && $0 == "---" { infm = 0; skipblank = 1; next }
         infm { next }
         skipblank { skipblank = 0; if ($0 == "") next }
         { print }' "$external/$dest" > "$scratch/body-$k"
    if cmp -s "$scratch/body-$k" "$fixtures/$src"; then
        echo "  ok    $k: the body below the front matter is $src, byte for byte"
        echo ok >> "$scratch/bytes"
    else
        echo "  FAIL  $k: the body below the front matter is not $src"
        echo fail >> "$scratch/bytes"
    fi
done
bytes_ok=$(grep -c '^ok$' "$scratch/bytes" 2>/dev/null | head -n 1)
bytes_bad=$(grep -c '^fail$' "$scratch/bytes" 2>/dev/null | head -n 1)
bytes_ok=${bytes_ok:-0}
bytes_bad=${bytes_bad:-0}
passed=$((passed + bytes_ok))
failed=$((failed + bytes_bad))

(cd "$external" && "$engine" taxonomy resolve) > "$scratch/ext-resolve.out" 2>&1
(cd "$external" && "$engine" check --no-cache) > "$scratch/external.out" 2> "$scratch/external.err"
extcode=$?
for k in $kinds; do
    n=$(sed -n "s/^ *\([0-9][0-9]*\) typed $k$/\1/p" "$scratch/external.out" | head -n 1)
    judge "the external corpus types a document at kind $k" 1 "${n:-0}"
done

docs=$(sed -n 's/^ *\([0-9][0-9]*\) files under the corpus root$/\1/p' "$scratch/external.out" | head -n 1)
typed=$(sed -n 's/^ *\([0-9][0-9]*\) typed$/\1/p' "$scratch/external.out" | head -n 1)
judge "every document of the external corpus is typed" "$docs" "$typed"

echo
echo "the run record of criterion 4, recorded and not asserted"
echo "  engine:        $("$engine" --version)"
findings=$(sed -n 's/^ *\([0-9][0-9]*\) findings$/\1/p' "$scratch/external.out" | head -n 1)
# Read the report's own tally rather than counting glyph lines: the read-set
# block at the foot of a report carries lines that end the same way.
errors=$(sed -n 's/^ *\([0-9][0-9]*\) ✗ error$/\1/p' "$scratch/external.out" | head -n 1)
warns=$(sed -n 's/^ *\([0-9][0-9]*\) ▲ warn$/\1/p' "$scratch/external.out" | head -n 1)
(cd "$external" && "$engine" check --strict --no-cache) > "$scratch/ext-strict.out" 2> "$scratch/ext-strict.err"
strictcode=$?
echo "  documents:     ${docs:-0}, of which ${typed:-0} typed"
echo "  findings:      ${findings:-0}, of which ${errors:-0} error and ${warns:-0} warn"
echo "  check exit:    $extcode plain, $strictcode strict"
echo "  by rule, over the ${findings:-0}:"
grep -oE '^ +[a-z._]+ \(OB-' "$scratch/external.out" |
    sed 's/ (OB-$//' | sed 's/^ *//' | sort | uniq -c |
    while read -r n rule; do echo "    $n  $rule"; done

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ] || exit 1
