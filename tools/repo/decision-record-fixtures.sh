#!/bin/sh
# What holds the external corpus of the `decision-record` entry: the 10 ADRs of
# `UKGovernmentBEIS/inspect_evals`, vendored at a pin, assembled into a corpus
# root at each run, and typed by the entry.
#
# Run it from anywhere:
#     sh tools/repo/decision-record-fixtures.sh
#
# # THE DEFECT THIS SUITE EXISTS FOR
#
# The whole value of an external corpus is that its prose was not written to
# pass. A run that is green because somebody quietly repaired a heading, a
# status line or a link in the vendored copy has measured this repository
# against itself again, which is the condition #488 was filed about. Two cases
# catch it and each holds only half.
#
#   THE DIGEST CASE seals every vendored file against the SHA-256 the source
#   table of `fixtures/README.md` records. An edit to a vendored file reddens
#   this suite rather than quietly re-measuring the denominators.
#
#   THE BYTE-IDENTITY CASE strips the front matter this runner wrote and
#   compares what is left against the pinned file with `cmp`. It cannot fail on
#   the CONTENT of a source, because the runner assembles out of the same file
#   it compares back to: it holds the stripper and nothing else.
#
# Neither case reaches the upstream commit, and nothing in this repository
# does. A digest recorded beside the file it was computed from is a seal
# against local drift and not a proof of provenance.
#
# # WHAT CRITERION 4 ASKS, AND WHAT IT DOES NOT ASK
#
# Criterion 4 of `docs/taxonomies/README.md` asks for "at least one external
# real or realistic corpus, typed by the entry and recorded with its source,
# revision, paths, and run". It asks for a recorded run. It does not ask for a
# clean one, and the run is not clean. The denominators are printed and
# asserted by nothing, because a finding count is a property of somebody else's
# prose and a suite that asserted it would redden the day a rule of the engine
# widened or the day the pin moved.
#
# ONE FINDING CLASS IS ASSERTED, and it is the one that is a claim about the
# tradition rather than about one team's prose. `section.required.missing` must
# be reported for no document here. The entry's doctrine says that Nygard's
# three sections are a convention the tradition carries, so a real ADR log
# ought to satisfy a contract written from it. A run that started reporting a
# missing section would be evidence against the entry, and that is worth a red
# suite. `diataxis-site` refused all 4 of its 4 external documents on this same
# rule, which is why the contrast is worth holding.
#
# # WHAT EACH JUDGE READS
#
# No file name, kind name, shelf path, identifier, date or expected count is
# written in this file. Every population is enumerated out of a declaration:
#
#   THE SOURCE ROWS come from the source table of `fixtures/README.md`, between
#   its `### Source, revision and paths` heading and the next `###`: one row
#   per pinned file, naming the kind, the shelf, the source, the assembled path
#   and the digest.
#
#   THE STATUS MAP comes from the table under `### The status map` on the same
#   page: the prose the corpus writes, and the state the front matter carries.
#   A `## Status` value that matches no row fails the run.
#
#   THE PIN AND ITS DATE come from the metadata table under the same first
#   heading, so a re-pin moves one page and the run follows it.
#
#   THE DECLARED KINDS AND THE DECLARED LIFECYCLE STATES come from the
#   `.headwater/taxonomy.lock` that the scratch root resolves, which is what
#   the engine itself reads.
#
#   THE IDENTIFIER of each assembled document is built from the `pattern` of
#   `identifier_schemes.decision_id` in that lock, with the sequence read off
#   the file name.
#
#   `status_since` comes from each document's own `## Date` heading.
#
# ONE VALUE IS A CONSTANT THIS RUNNER WRITES, AND IT IS A FINDING. `summary` is
# required on every `governed_document` and the tradition supplies nothing that
# derives one. `fixtures/README.md` carries that as a result rather than as a
# defect in their writing.
#
# # THE POPULATION GUARDS, WHICH ARE NOT COUNTS
#
# Each group opens with a floor of zero. A judge whose population came back
# empty reports green for the wrong reason. No expected number of rows, files
# or findings is written down here, so none can go stale.
#
# # WHAT IT NEEDS, AND WHAT IT WRITES
#
# `awk`, `sed`, `cmp`, `sha256sum` and a built engine. Every scratch tree is
# made under `mktemp -d`, the directory goes on an interrupt, and nothing
# inside this checkout is written.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
entry=docs/taxonomies/decision-record
fixtures="$root/$entry/fixtures"
readme="$fixtures/README.md"
vendored="$fixtures/sources/inspect-evals"

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
    echo "  Build one:" >&2
    echo "  cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked" >&2
    exit 1
fi

for needed in "$readme" "$vendored/adr" "$vendored/LICENSE"; do
    if [ ! -e "$needed" ]; then
        echo "missing $needed, so the cases over the external corpus cannot run." >&2
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

# The source table: kind, shelf, source, assembled path, digest. Only the rows
# between the first `###` heading of the external section and the next one.
source_rows() {
    awk -F'|' '
        /^### Source, revision and paths/ { here = 1; next }
        /^### / { here = 0 }
        here && /^\|/ {
            k = $2; sh = $3; s = $4; d = $5; g = $6
            gsub(/[ `]/, "", k); gsub(/[ `]/, "", sh)
            gsub(/[ `]/, "", s); gsub(/[ `]/, "", d); gsub(/[ `]/, "", g)
            if (s ~ /^sources\//) { print k "\t" sh "\t" s "\t" d "\t" g }
        }
    ' "$readme"
}

# The status map: the prose the corpus writes, and the state it maps to.
status_rows() {
    awk -F'|' '
        /^### The status map/ { here = 1; next }
        /^### / { here = 0 }
        here && /^\| `/ {
            a = $2; b = $3
            gsub(/[ `]/, "", a); gsub(/[ `]/, "", b)
            if (a != "" && b != "") { print a "\t" b }
        }
    ' "$readme"
}

# The pin and the date it was taken, out of the metadata table.
pin_field() { # 1 = the word the row opens with
    awk -F'|' -v want="$1" '
        /^### Source, revision and paths/ { here = 1; next }
        /^### / { here = 0 }
        here && /^\|/ {
            a = $2; gsub(/^ +| +$/, "", a)
            if (a == want) { b = $3; gsub(/^ +| +$/, "", b); print b; exit }
        }
    ' "$readme"
}

# Every key under `resolved.kinds` of a written lock.
lock_kinds() {
    awk '
        /^  kinds:/ { here = 1; next }
        /^  [a-z_]+:/ { here = 0 }
        here && /^    [a-z_]+:/ { k = $0; sub(/^    /, "", k); sub(/:.*/, "", k); print k }
    ' "$1" | sort -u
}

# Every value of `resolved.vocabularies.lifecycle_state` of a written lock.
lock_states() {
    awk '
        /^    lifecycle_state:/ { here = 1; next }
        here && /^    [a-z_]+:/ { here = 0 }
        here && /value:/ {
            v = $0
            sub(/.*value: */, "", v)
            sub(/[,}].*/, "", v)
            gsub(/[" ]/, "", v)
            if (v != "") print v
        }
    ' "$1" | sort -u
}

# The `pattern` of an identifier scheme in a written lock.
lock_pattern() { # 1 = lock, 2 = scheme
    awk -v want="$2" '
        /^  identifier_schemes:/ { here = 1; next }
        /^  [a-z_]+:/ { here = 0 }
        here && $0 ~ "^    " want ":" { hit = 1; next }
        here && /^    [a-z_]+:/ { hit = 0 }
        hit && /pattern:/ {
            p = $0; sub(/^[^:]*: */, "", p); gsub(/["]/, "", p); gsub(/ *$/, "", p)
            print p; exit
        }
    ' "$1"
}

# The bundles a root has to select to carry this entry: the entry itself, and
# whatever its `requires:` line names. Criterion 6 makes the first entry to
# claim an address its owner, so a dependency is a name this bundle already
# writes down and never a list this runner keeps.
bundle_selection() {
    deps=$(sed -n 's/^requires: *\[\(.*\)\] *$/\1/p' "$root/$entry/bundle.yml" | head -n 1)
    if [ -n "$deps" ]; then
        printf '%s, decision-record' "$deps"
    else
        printf 'decision-record'
    fi
}

# A scratch root that selects the entry. $1 = destination.
make_root() {
    at=$1
    mkdir -p "$at/packages" "$at/.headwater" "$at/docs"
    cp -r "$root/packages/headwater-standard" "$at/packages/"
    {
        echo 'taxonomy:'
        echo '  package: headwater/standard'
        sed -n 's/^  version: /  version: /p' "$root/.headwater/taxonomy.yml" | head -n 1
        sed -n 's/^  digest: /  digest: /p' "$root/.headwater/taxonomy.yml" | head -n 1
        echo "  bundles: [$(bundle_selection)]"
        echo '  overlay: .headwater/overlay.yml'
        echo
        echo 'corpus:'
        echo '  root: docs'
    } > "$at/.headwater/taxonomy.yml"
    # The overlay supplies exactly the namespaces the selection is missing, and
    # it is derived rather than listed: an overlay that declares a namespace on
    # a scheme the selection does not carry is itself a refusal.
    echo 'add: {}' > "$at/.headwater/overlay.yml"
    (cd "$at" && "$engine" taxonomy validate) > "$scratch/ns.out" 2> "$scratch/ns.err"
    sed -n 's/.*`identifier_schemes\.\([a-z_]*\)`: identifier integrity: carries no namespace.*/\1/p' \
        "$scratch/ns.out" "$scratch/ns.err" | sort -u > "$scratch/ns.list"
    {
        echo 'add:'
        while read -r scheme; do
            [ -n "$scheme" ] || continue
            echo "  identifier_schemes.${scheme}.namespace: IE"
        done < "$scratch/ns.list"
    } > "$at/.headwater/overlay.yml"
}

# The `## Status` value of a source file: the first non-empty line under it.
status_of() {
    awk '/^## Status/ { seen = 1; next }
         seen && NF { print; exit }' "$1"
}

# The `## Date` value of a source file, by the same reading.
date_of() {
    awk '/^## Date/ { seen = 1; next }
         seen && NF { print; exit }' "$1"
}

# The `# ` title of a source file, with the tradition's own identifier prefix
# removed where it carries one.
title_of() {
    awk '/^# / { t = substr($0, 3); sub(/^ADR-[0-9]+: */, "", t); print t; exit }' "$1"
}

rows=$(source_rows)
row_floor=$(printf '%s\n' "$rows" | grep -c . || true)
smap=$(status_rows)
smap_floor=$(printf '%s\n' "$smap" | grep -c . || true)
files=$(find "$vendored/adr" -name '*.md' | sed "s|^$fixtures/||" | sort)
file_floor=$(printf '%s\n' "$files" | grep -c . || true)

echo "decision-record external fixtures, against $engine"
echo

echo "population guards"
if [ "$row_floor" -eq 0 ]; then
    fail "the source table carries at least one row" "none parsed out of $entry/fixtures/README.md"
else
    pass "the source table carries at least one row ($row_floor read)"
fi
if [ "$smap_floor" -eq 0 ]; then
    fail "the status map carries at least one row" "none parsed out of $entry/fixtures/README.md"
else
    pass "the status map carries at least one row ($smap_floor read)"
fi
if [ "$file_floor" -eq 0 ]; then
    fail "the vendored directory holds at least one file" "none under $entry/fixtures/sources/inspect-evals/adr"
else
    pass "the vendored directory holds at least one file ($file_floor read)"
fi
[ "$row_floor" -eq 0 ] && exit 1
[ "$smap_floor" -eq 0 ] && exit 1
[ "$file_floor" -eq 0 ] && exit 1

echo
echo "case group 1 — the source table covers the vendored files, in both directions"
printf '%s\n' "$rows" | cut -f3 | sort -u > "$scratch/table-files"
printf '%s\n' "$files" > "$scratch/tree-files"
judge "every vendored file has a row in the source table" "" \
    "$(comm -13 "$scratch/table-files" "$scratch/tree-files" | tr '\n' ' ' | sed 's/ *$//')"
judge "every row of the source table names a vendored file that is there" "" \
    "$(comm -23 "$scratch/table-files" "$scratch/tree-files" | tr '\n' ' ' | sed 's/ *$//')"

echo
echo "case group 2 — the pinned bytes are the bytes the table seals"
# This is the half of the pinning that can fail on the content of a source.
# An edit to a vendored file moves the recorded denominators without moving any
# other assertion here, and this case is what stops that being silent.
printf '%s\n' "$rows" | while IFS="$(printf '\t')" read -r k sh src dest digest; do
    [ -n "$src" ] || continue
    if [ -z "$digest" ]; then
        echo "  FAIL  $src: the source table records no digest"
        echo fail >> "$scratch/digests"
        continue
    fi
    read_digest=$(sha256sum "$fixtures/$src" | cut -d' ' -f1)
    if [ "$read_digest" = "$digest" ]; then
        echo "  ok    $src is the file the table seals"
        echo ok >> "$scratch/digests"
    else
        echo "  FAIL  $src is not the file the table seals"
        echo "          recorded [$digest], read [$read_digest]"
        echo fail >> "$scratch/digests"
    fi
done
digests_ok=$(grep -c '^ok$' "$scratch/digests" 2>/dev/null | head -n 1)
digests_bad=$(grep -c '^fail$' "$scratch/digests" 2>/dev/null | head -n 1)
passed=$((passed + ${digests_ok:-0}))
failed=$((failed + ${digests_bad:-0}))

license_digest=$(pin_field 'SHA-256 of `sources/inspect-evals/LICENSE`' | tr -d '` ')
read_license=$(sha256sum "$vendored/LICENSE" | cut -d' ' -f1)
judge "the vendored LICENSE is the file the metadata table seals" \
    "$license_digest" "$read_license"

echo
echo "case group 3 — the corpus resolves, and the table names kinds it declares"
corpus="$scratch/corpus"
make_root "$corpus"
(cd "$corpus" && "$engine" taxonomy resolve) > "$scratch/resolve.out" 2> "$scratch/resolve.err"
code=$?
if [ "$code" -eq 0 ]; then
    pass "a root that selects $(bundle_selection) resolves"
else
    fail "a root that selects $(bundle_selection) resolves" \
        "exit $code: $(sed -n '2p' "$scratch/resolve.err")"
fi
lock="$corpus/.headwater/taxonomy.lock"
if [ ! -f "$lock" ]; then
    fail "the run wrote a lock, which is a validated taxonomy" "no $lock"
    echo
    echo "$passed passed, $failed failed"
    exit 1
fi
pass "the run wrote a lock, which is a validated taxonomy"
lock_kinds "$lock" > "$scratch/declared-kinds"
lock_states "$lock" > "$scratch/declared-states"
printf '%s\n' "$rows" | cut -f1 | sort -u > "$scratch/table-kinds"
judge "every kind the source table names is a kind the taxonomy declares" "" \
    "$(comm -23 "$scratch/table-kinds" "$scratch/declared-kinds" | tr '\n' ' ' | sed 's/ *$//')"
printf '%s\n' "$smap" | cut -f2 | sort -u > "$scratch/mapped-states"
judge "every state the status map targets is a declared lifecycle state" "" \
    "$(comm -23 "$scratch/mapped-states" "$scratch/declared-states" | tr '\n' ' ' | sed 's/ *$//')"

echo
echo "case group 4 — the status map reaches every document, and what it drops"
: > "$scratch/statuses"
printf '%s\n' "$rows" | while IFS="$(printf '\t')" read -r k sh src dest digest; do
    [ -n "$src" ] || continue
    raw=$(status_of "$fixtures/$src")
    : > "$scratch/hits"
    printf '%s\n' "$smap" | while IFS="$(printf '\t')" read -r prose state; do
        [ -n "$prose" ] || continue
        case "$raw" in
            "$prose"*) printf '%s\t%s\n' "$prose" "$state" >> "$scratch/hits" ;;
        esac
    done
    if [ -s "$scratch/hits" ]; then
        # the longest matching prefix wins
        hit=$(awk -F'\t' '{ if (length($1) > n) { n = length($1); s = $2; p = $1 } } END { print p "\t" s }' "$scratch/hits")
        prose=$(printf '%s' "$hit" | cut -f1)
        state=$(printf '%s' "$hit" | cut -f2)
        lossy=no
        [ "$raw" = "$prose" ] || lossy=yes
        printf '%s\t%s\t%s\t%s\n' "$src" "$state" "$lossy" "$raw" >> "$scratch/statuses"
    else
        printf '%s\t\tUNMAPPED\t%s\n' "$src" "$raw" >> "$scratch/statuses"
    fi
done
unmapped=$(awk -F'\t' '$3 == "UNMAPPED" { print $1 }' "$scratch/statuses" | tr '\n' ' ' | sed 's/ *$//')
judge "every status value in the corpus matches a row of the status map" "" "$unmapped"
mapped=$(grep -c . "$scratch/statuses" 2>/dev/null | head -n 1)
lossy_n=$(awk -F'\t' '$3 == "yes"' "$scratch/statuses" | grep -c . || true)
echo "  recorded, not asserted: ${lossy_n:-0} of ${mapped:-0} statuses lost a clause to the map"
awk -F'\t' '$3 == "yes" { printf "    %s  [%s] -> %s\n", $1, $4, $2 }' "$scratch/statuses"

echo
echo "case group 5 — the corpus, assembled with front matter and no other edit"
pattern=$(lock_pattern "$lock" decision_id)
namespace=$(sed -n 's/^  identifier_schemes\.decision_id\.namespace: *//p' "$corpus/.headwater/overlay.yml")
verified=$(pin_field Pin | sed -n 's/.*committed \([0-9][0-9-]*\)T.*/\1/p')
if [ -z "$pattern" ]; then
    fail "the identifier pattern and the pin date come from declarations" "no decision_id pattern in $lock"
elif [ -z "$namespace" ]; then
    fail "the identifier pattern and the pin date come from declarations" "no namespace in the scratch overlay"
elif [ -z "$verified" ]; then
    fail "the identifier pattern and the pin date come from declarations" "no ISO date in the Pin row"
else
    pass "the identifier pattern and the pin date come from declarations ($pattern, $verified)"
fi
rm -rf "$corpus/docs"
printf '%s\n' "$rows" | while IFS="$(printf '\t')" read -r k sh src dest digest; do
    [ -n "$src" ] || continue
    seq=$(basename "$src" | sed 's/^\([0-9]*\).*/\1/')
    id=$(printf '%s' "$pattern" | sed -e "s/{namespace}/$namespace/" -e "s/{seq:04d}/$seq/")
    state=$(awk -F'\t' -v s="$src" '$1 == s { print $2; exit }' "$scratch/statuses")
    since=$(date_of "$fixtures/$src")
    title=$(title_of "$fixtures/$src" | sed 's/\\/\\\\/g; s/"/\\"/g')
    mkdir -p "$(dirname "$corpus/$dest")"
    # An adopter that mints an identifier claims it, so the assembler does
    # too. Without this the run reports one `identifier.claim.missing` per
    # document, which is a defect of the assembly and not of their prose.
    mkdir -p "$corpus/.headwater/ids/decision_id"
    printf '%s\n' "$dest" > "$corpus/.headwater/ids/decision_id/$id"
    {
        printf -- '---\n'
        printf 'id: %s\n' "$id"
        printf 'title: "%s"\n' "$title"
        printf 'status: %s\n' "$state"
        printf 'status_since: %s\n' "$since"
        printf 'last_verified: %s\n' "$verified"
        printf 'summary: A pinned external decision record, assembled with front matter and no other edit.\n'
        printf -- '---\n\n'
        cat "$fixtures/$src"
    } > "$corpus/$dest"
done

# Byte identity, which is the assertion the whole group rests on.
printf '%s\n' "$rows" | while IFS="$(printf '\t')" read -r k sh src dest digest; do
    [ -n "$src" ] || continue
    if [ ! -f "$corpus/$dest" ]; then
        echo "  FAIL  $dest was not assembled"
        echo fail >> "$scratch/bytes"
        continue
    fi
    awk 'NR == 1 && $0 == "---" { infm = 1; next }
         infm && $0 == "---" { infm = 0; skipblank = 1; next }
         infm { next }
         skipblank { skipblank = 0; if ($0 == "") next }
         { print }' "$corpus/$dest" > "$scratch/body"
    if cmp -s "$scratch/body" "$fixtures/$src"; then
        echo "  ok    $dest below its front matter is $src, byte for byte"
        echo ok >> "$scratch/bytes"
    else
        echo "  FAIL  $dest below its front matter is not $src"
        echo fail >> "$scratch/bytes"
    fi
done
bytes_ok=$(grep -c '^ok$' "$scratch/bytes" 2>/dev/null | head -n 1)
bytes_bad=$(grep -c '^fail$' "$scratch/bytes" 2>/dev/null | head -n 1)
passed=$((passed + ${bytes_ok:-0}))
failed=$((failed + ${bytes_bad:-0}))

echo
echo "case group 6 — the corpus types, and the section contract holds"
(cd "$corpus" && "$engine" check --no-cache) > "$scratch/check.out" 2> "$scratch/check.err"
extcode=$?
docs=$(sed -n 's/^ *\([0-9][0-9]*\) files under the corpus root$/\1/p' "$scratch/check.out" | head -n 1)
typed=$(sed -n 's/^ *\([0-9][0-9]*\) typed$/\1/p' "$scratch/check.out" | head -n 1)
if [ "${docs:-0}" -eq 0 ]; then
    fail "the assembled corpus holds at least one document" "the check read 0 files under the corpus root"
else
    pass "the assembled corpus holds at least one document (${docs} read)"
fi
judge "every document of the external corpus is typed" "${docs:-0}" "${typed:-0}"
onekind=$(printf '%s\n' "$rows" | cut -f1 | sort -u | head -n 1)
atkind=$(sed -n "s/^ *\([0-9][0-9]*\) typed $onekind$/\1/p" "$scratch/check.out" | head -n 1)
judge "every one of them types at $onekind" "${typed:-0}" "${atkind:-0}"
# Two readings, and the second is what stops a silent skip reading as a pass.
# The report's finding lines name a rule with its obligation in parentheses;
# its check-instance block says how many times the rule ran.
missing=$(grep -c 'section.required.missing (OB-' "$scratch/check.out" || true)
ran=$(sed -n 's/^ *\([0-9][0-9]*\) instances of section.required.missing$/\1/p' "$scratch/check.out" | head -n 1)
judge "no document of this corpus is missing a required section" "0" "${missing:-0}"
judge "and the section rule ran over every document of it" "${docs:-0}" "${ran:-0}"

(cd "$corpus" && "$engine" check --strict --no-cache) > "$scratch/strict.out" 2> "$scratch/strict.err"
strictcode=$?

echo
echo "the run record of criterion 4, recorded and not asserted"
echo "  engine:        $("$engine" --version)"
echo "  taxonomy:      $(sed -n 's/^  version: *//p' "$lock" | head -n 1)"
findings=$(sed -n 's/^ *\([0-9][0-9]*\) findings$/\1/p' "$scratch/check.out" | head -n 1)
errors=$(sed -n 's/^ *\([0-9][0-9]*\) ✗ error$/\1/p' "$scratch/check.out" | head -n 1)
warns=$(sed -n 's/^ *\([0-9][0-9]*\) ▲ warn$/\1/p' "$scratch/check.out" | head -n 1)
echo "  documents:     ${docs:-0}, of which ${typed:-0} typed"
echo "  findings:      ${findings:-0}, of which ${errors:-0} error and ${warns:-0} warn"
echo "  check exit:    $extcode plain, $strictcode strict"
echo "  by rule, over the ${findings:-0}:"
grep -oE '^ +[a-z._]+ \(OB-' "$scratch/check.out" |
    sed 's/ (OB-$//' | sed 's/^ *//' | sort | uniq -c |
    while read -r n rule; do echo "    $n  $rule"; done

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ] || exit 1
