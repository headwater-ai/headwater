#!/bin/sh
# What holds `tools/cite/check-citations.py`.
#
# The checker answers one question about each citation comment in a code tree —
# is this still true — and it answers it through the shipped binary. So the
# cases below are not unit tests of a regular expression. Each one plants a
# fixture file at a path this repository really governs, or really does not,
# and runs the checker against a real corpus.
#
# Run it from anywhere:
#     sh tools/cite/check-citations-fixtures.sh
#
# # THE ONE CASE THE WHOLE TOOL IS FOR
#
# Case 3 plants a citation of spec 5 at `engine/crates/query/src/mcp.rs`. The
# identifier resolves and the file is governed, so both halves a naive checker
# looks at are clean; what is wrong is the pair, because the document that
# governs that file is `docs/interfaces/headwater-mcp.md` and not spec 5. That
# is a citation that was true and went stale, which is what issue #500 was
# filed about. Case 6 is the assertion that matters beside it: the stale class
# and the invented class are reported under different rule identifiers, so a
# reader can tell a typo from a citation that has aged.
#
# Case 4 is the second load-bearing one. A citation of an asserted document is
# reported and the exit status stays 0, because a checker whose advisory class
# fails a build is a checker an adopter turns off.
#
# # WHY THERE IS A SCRATCH CORPUS
#
# A governing set is a property of a path, and the only paths this repository
# governs are the ones its documents name. A fixture file checked in under
# `tools/cite/fixtures/` is governed by nothing, so every case over it would
# collapse into one. The suite therefore copies the parts of this repository
# the engine reads — the consumer declaration, the lock, the overlay and the
# corpus root — into a temporary directory, and plants each fixture at the
# governed path it was written for. Nothing inside this checkout is written.
#
# # WHAT IT NEEDS
#
# `python3` and a built `headwater`. It builds nothing. If the binary is
# missing it says so and exits 1 rather than reporting a row of passes.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/cite/check-citations.py"
fixtures="$root/tools/cite/fixtures"

if ! command -v python3 >/dev/null 2>&1; then
    echo "no \`python3\` on the path, and the checker is written in it." >&2
    exit 1
fi

engine=
for profile in release dev-release; do
    candidate="$root/engine/target/$profile/headwater"
    if [ -x "$candidate" ]; then
        if [ -z "$engine" ] || [ "$candidate" -nt "$engine" ]; then
            engine=$candidate
        fi
    fi
done
if [ -z "$engine" ]; then
    echo "no engine at engine/target/{release,dev-release}/headwater, and every" >&2
    echo "  case here drives one. Build it:" >&2
    echo "        cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked" >&2
    exit 1
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

corpus="$scratch/corpus"
mkdir -p "$corpus/.headwater"
cp "$root/.headwater/taxonomy.yml" "$corpus/.headwater/"
cp "$root/.headwater/taxonomy.lock" "$corpus/.headwater/"
cp "$root/.headwater/overlay.yml" "$corpus/.headwater/"
cp -R "$root/docs" "$corpus/docs"

# Each fixture, at the path it was written for. A `governs` edge is matched
# against the path it named rather than against a file on disk, so planting a
# file here is what makes the citation in it carry that path's governing set.
plant() {
    mkdir -p "$corpus/$(dirname "$2")"
    cp "$fixtures/$1" "$corpus/$2"
}

plant governed.sh .claude/hooks/write.sh
plant asserted.sh .claude/hooks/lib.sh
plant stale.rs engine/crates/query/src/mcp.rs
plant invented.py tools/cite/invented.py
plant orphan.py tools/cite/orphan.py
plant shapes.txt tools/cite/shapes.txt

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

same() {
    if [ "$2" = "$3" ]; then
        pass "$1"
    else
        fail "$1" "expected \`$2\`, got \`$3\`"
    fi
}

holds() {
    if grep -qF "$2" "$3"; then
        pass "$1"
    else
        fail "$1" "no line of $3 holds \`$2\`"
    fi
}

absent() {
    if grep -qF "$2" "$3"; then
        fail "$1" "$3 holds \`$2\` and should not"
    else
        pass "$1"
    fi
}

# run FORMAT PATH... — the checker over the scratch corpus. Writes
# `$scratch/out` and `$scratch/err`, and echoes the exit status.
run() {
    format=$1
    shift
    targets=""
    for one in "$@"; do
        targets="$targets $corpus/$one"
    done
    # shellcheck disable=SC2086
    python3 "$tool" --root "$corpus" --engine "$engine" --format "$format" \
        $targets >"$scratch/out" 2>"$scratch/err"
    echo $?
}

# rules — every rule identifier the last sarif run reported, in order
rules() {
    python3 -c 'import json,sys
d=json.load(open(sys.argv[1]))
print(" ".join(r["ruleId"] for r in d["runs"][0]["results"]))' "$scratch/out"
}

field() {
    python3 -c 'import json,sys
d=json.load(open(sys.argv[1]))
print(eval(sys.argv[2], {"d": d}))' "$scratch/out" "$2"
}

echo "one citation at a time, each against the corpus that licensed it"

# 1. A citation that is true. Spec 5 declares a `governs` edge onto exactly
#    this path, and its warrant is `accepted`, so no class fires at all.
status=$(run text .claude/hooks/write.sh)
same "a true citation reports nothing" 0 "$status"
holds "  and the report says so in words" "every citation resolves" "$scratch/out"
holds "  and counts one citation" "1 citations, 0 unresolved, 0 stale, 0 asserted" "$scratch/out"

# 2. An identifier nothing carries.
status=$(run text tools/cite/invented.py)
same "an invented identifier fails the run" 1 "$status"
holds "  under the unresolved rule" "citation.identifier.unresolved" "$scratch/out"
holds "  naming the identifier it could not find" \
    "no document of this corpus carries \`HW-DR-9999\`" "$scratch/out"

# 3. THE DECISIVE CASE. The identifier resolves, the file is governed, and the
#    document that governs it is not the one cited.
status=$(run text engine/crates/query/src/mcp.rs)
same "a citation that went stale fails the run" 1 "$status"
holds "  under the stale rule" "citation.governance.stale" "$scratch/out"
holds "  saying which document does govern the file" \
    "\`docs/interfaces/headwater-mcp.md\` does" "$scratch/out"
absent "  and not under the unresolved rule, because the identifier is real" \
    "citation.identifier.unresolved" "$scratch/out"
holds "  and the report states the limit of the class it just reported" \
    "HW-OBL-0104" "$scratch/out"

# 4. THE SECOND DECISIVE CASE. An asserted document is reported and the exit
#    status does not move, which is what makes the class usable.
status=$(run text .claude/hooks/lib.sh)
same "an asserted citation leaves the exit status at 0" 0 "$status"
holds "  and is still reported" "citation.warrant.asserted" "$scratch/out"
holds "  and says the exit status did not move" \
    "advisory and did not move the exit status" "$scratch/out"
absent "  and the stale limit is not printed where nothing stale was found" \
    "HW-OBL-0104" "$scratch/out"

# 5. The other half of the stale class: a governing set that is empty rather
#    than wrong. A reader who is told which document governs the file can move
#    the citation; a reader whose file nothing governs cannot, so the two say
#    different things.
status=$(run text tools/cite/orphan.py)
same "a citation in an ungoverned file fails the run" 1 "$status"
holds "  under the same stale rule" "citation.governance.stale" "$scratch/out"
holds "  and says the set is empty rather than wrong" \
    "no document governs this file at all" "$scratch/out"

echo
echo "the classes stay apart, and the whole tree agrees with its parts"

# 6. The assertion the tool exists for. An invented identifier and a stale one
#    are two findings under two rules, in one run, and a reader can tell them
#    apart without reading the prose.
status=$(run sarif tools/cite/invented.py engine/crates/query/src/mcp.rs)
same "the two blocking classes fail one run together" 1 "$status"
same "  and are reported under two different rules" \
    "citation.governance.stale citation.identifier.unresolved" "$(rules)"

# 7. Every fixture at once. Four findings over six planted files, and the one
#    true citation contributes none.
status=$(run sarif .claude/hooks/write.sh .claude/hooks/lib.sh \
    engine/crates/query/src/mcp.rs tools/cite/invented.py tools/cite/orphan.py)
same "the whole fixture tree fails the run" 1 "$status"
same "  with one finding per defective file and none for the true one" 4 \
    "$(field . 'len(d["runs"][0]["results"])')"
same "  over five citations" 5 \
    "$(field . 'd["runs"][0]["properties"]["headwater"]["citations"]')"

echo
echo "what counts as a citation"

# 8. The lexer, held on its own so that no finding class stands between the
#    line and the count. Three comment markers and a trailing comment are
#    citations; a string literal, a bare line, an unhyphenated word and a
#    citation with no path are not.
status=$(run sarif tools/cite/shapes.txt)
same "four of the eight shapes are citations" 4 \
    "$(field . 'd["runs"][0]["properties"]["headwater"]["citations"]')"
same "  and the run fails, because nothing governs where they sit" 1 "$status"

echo
echo "the format a forge annotates a diff with"

# 9. SARIF, as a forge ingests it. The three rules are declared whether or not
#    a run found one, because a rule a consumer cannot look up is a finding
#    with no explanation.
status=$(run sarif .claude/hooks/lib.sh)
same "an advisory-only sarif run still exits 0" 0 "$status"
same "  the document states the version a forge reads" 2.1.0 \
    "$(field . 'd["version"]')"
same "  all three rules are declared" 3 \
    "$(field . 'len(d["runs"][0]["tool"]["driver"]["rules"])')"
same "  the advisory rule carries sarif note rather than error" note \
    "$(field . 'd["runs"][0]["results"][0]["level"]')"
same "  and says in its own properties that it blocks nothing" False \
    "$(field . 'd["runs"][0]["results"][0]["properties"]["headwater"]["blocking"]')"
same "  and the rule a reader looks up carries the reason it is advisory" True \
    "$(field . '"turns off" in d["runs"][0]["tool"]["driver"]["rules"][2]["help"]["text"]')"

echo
echo "what the checker refuses rather than guesses"

# 10. A missing engine is a refusal and not a clean run. A checker that
#     reported zero findings because it could not reach the corpus is the
#     silent-success shape this repository keeps meeting.
python3 "$tool" --root "$corpus" --engine "$scratch/no-such-engine" \
    "$corpus/tools/cite/invented.py" >"$scratch/out" 2>"$scratch/err"
same "a missing engine exits 2 rather than 0" 2 "$?"
holds "  and says so on standard error" "check-citations:" "$scratch/err"

# 11. And a tree with no citation in it costs no engine at all, which is why
#     the bogus binary above is not reached here.
mkdir -p "$corpus/tools/cite/quiet"
printf 'value = 1\n' >"$corpus/tools/cite/quiet/plain.py"
python3 "$tool" --root "$corpus" --engine "$scratch/no-such-engine" \
    "$corpus/tools/cite/quiet" >"$scratch/out" 2>"$scratch/err"
same "a tree with no citation exits 0 without starting an engine" 0 "$?"
holds "  and says it found none" "0 citations" "$scratch/out"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
