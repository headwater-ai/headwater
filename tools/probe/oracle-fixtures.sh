#!/bin/sh
# Fixtures for the generic oracle scripts under tools/probe/oracles/ (#1874).
#
#     sh tools/probe/oracle-fixtures.sh
#
# Each oracle runs over at least one tree it must pass and one tree it must
# fail, and a case holds the exact exit status and that standard output is
# one line that opens with the oracle's name. So an oracle that passes both
# trees fails here, and so does one that fails both. That is the failure the
# #1835 pilot found: probes that passed every session or failed every session.
#
# The trees are built at run time, not committed. The base is a copy of every
# file `git ls-files` names in this checkout, so it is this repository at its
# own commit, and each case copies the base and changes it the way a session
# would. A committed corpus would go stale against the engine with every
# taxonomy change, and this one cannot. The tree oracles read the documents
# HW-DR-0111 and HW-DR-0112 because the first supersedes the second's
# predecessor and both are settled records, so a later change rarely moves
# them. `test-passes.sh` runs against a stub cargo, because a real build per
# case would take minutes, and `no-hook-warning.sh` against logs this script
# writes.
#
# It needs a built engine, `sh`, `git`, `tar`, `python3`, `comm` and `sed`.
# Exit 0 when every case holds, 1 when one does not, and 8 when no engine is
# built.

set -u

case $0 in
    */*) invoked_from=${0%/*} ;;
    *) invoked_from=. ;;
esac
checkout=$(cd "$invoked_from/../.." && pwd -P)
oracles=$checkout/tools/probe/oracles

engine=$checkout/engine/target/dev-release/headwater
[ -x "$engine" ] || engine=$checkout/engine/target/release/headwater
[ -x "$engine" ] || {
    echo "oracle-fixtures: no engine under $checkout/engine/target. Build it first: cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked" >&2
    exit 8
}
HW_ORACLE_ENGINE=$engine
export HW_ORACLE_ENGINE

work=$(mktemp -d "${TMPDIR:-/tmp}/oracle-fixtures.XXXXXX") || exit 1
base=$work/base
mkdir -p "$base"
git -C "$checkout" ls-files -z > "$work/files" || exit 1
tar -C "$checkout" --null -T "$work/files" -cf - | tar -C "$base" -xf - || {
    echo "oracle-fixtures: the base tree could not be copied." >&2
    exit 1
}

new=docs/decisions/0112-the-additive-campaign-measures-eleven-actions-by-five-layers-in-seven-scenarios.md
old=docs/decisions/0111-the-additive-campaign-adds-six-layers-to-plain-code-in-fourteen-scenarios-and-registers-one-comparison-each.md
for path in "$new" "$old"; do
    [ -f "$base/$path" ] || {
        echo "oracle-fixtures: $path is gone, so the tree cases read nothing. Name another settled pair here." >&2
        exit 1
    }
done

failures=0
cases=0

# fresh <name>: a copy of the base, printed.
fresh() {
    cp -a "$base" "$work/$1"
    printf '%s\n' "$work/$1"
}

# expect <status> <label> <tree> <script> [<argument>...]
expect() {
    want=$1 label=$2 case_tree=$3 script=$4
    shift 4
    cases=$((cases + 1))
    got=0
    (cd "$case_tree" && sh "$oracles/$script" "$@") > "$work/out" 2> "$work/err" < /dev/null || got=$?
    lines=$(wc -l < "$work/out" | tr -d ' ')
    name=${script%.sh}
    if [ "$got" -ne "$want" ]; then
        echo "FAIL $label: $script exits $got, and the case wants $want: $(cat "$work/out")"
        sed 's/^/    stderr: /' "$work/err" | sed -n '1,5p'
        failures=$((failures + 1))
    elif [ "$lines" -ne 1 ] || ! grep -q "^$name: " "$work/out"; then
        echo "FAIL $label: $script prints $lines line(s) on standard output, and the form is one line that opens with \`$name: \`"
        failures=$((failures + 1))
    else
        echo "ok   $label: $(cat "$work/out")"
    fi
}

# A hard error the strict check fails on: a British spelling in a governed
# record (CLAUDE.md, "American spelling").
spoil() {
    printf '\nThe organisation of this record is unchanged.\n' >> "$1/${2:-$new}"
}

# --- check-clean.sh -------------------------------------------------------

expect 0 "check-clean passes this repository" "$base" check-clean.sh
tree=$(fresh check-spoiled) && spoil "$tree"
expect 1 "check-clean fails a tree with an error" "$tree" check-clean.sh
expect 2 "check-clean refuses an argument" "$base" check-clean.sh extra

# --- typed-at.sh ----------------------------------------------------------

title='eleven actions by five layers'
expect 0 "typed-at passes a decision under its shelf that cites its ruling" "$base" typed-at.sh docs/decisions decision HW-PD-0026 "$title"
expect 0 "typed-at with --check-clean passes a clean tree" "$base" typed-at.sh --check-clean docs/decisions/ decision HW-PD-0026 "$title"
expect 1 "typed-at with --check-clean fails a tree with an error" "$tree" typed-at.sh --check-clean docs/decisions decision HW-PD-0026 "$title"
# A sealed tree carries errors the seal made in documents it did not remove,
# so a document oracle reads the strict errors of the document it names.
elsewhere=$(fresh check-elsewhere) && spoil "$elsewhere" "$old"
expect 1 "check-clean fails an error in any document" "$elsewhere" check-clean.sh
expect 0 "typed-at with --check-clean passes an error in a document it does not name" "$elsewhere" typed-at.sh --check-clean docs/decisions decision HW-PD-0026 "$title"
expect 1 "typed-at fails the wrong kind" "$base" typed-at.sh docs/decisions obligation_record HW-PD-0026 "$title"
expect 1 "typed-at fails the wrong shelf" "$base" typed-at.sh docs/obligations decision HW-PD-0026 "$title"
expect 1 "typed-at fails a title nothing matches" "$base" typed-at.sh docs/decisions decision HW-PD-0026 'no document is titled this'
tree=$(fresh typed-uncited) && sed -i 's/HW-PD-0026/HW-PD-9999/g; s/0026-we-pivot/9999-we-pivot/g' "$tree/$new"
expect 1 "typed-at fails a document that does not cite the ruling" "$tree" typed-at.sh docs/decisions decision HW-PD-0026 "$title"
tree=$(fresh typed-elsewhere) && mkdir -p "$tree/docs/notes" && mv "$tree/$new" "$tree/docs/notes/"
expect 1 "typed-at fails the document moved off its shelf" "$tree" typed-at.sh docs/decisions decision HW-PD-0026 "$title"
tree=$(fresh typed-prefix) && sed -i 's/HW-PD-0026/HW-PD-00260/g' "$tree/$new"
expect 1 "typed-at fails a citation of an identifier that the ruling is a prefix of" "$tree" typed-at.sh docs/decisions decision HW-PD-0026 "$title"
# A period ends a whole name only when no letter, digit, `_` or `-` follows
# it: `HW-PD-0026.md` and `HW-PD-0026.5` are other names (verify of #1874).
tree=$(fresh typed-suffix) && sed -i 's/HW-PD-0026/HW-PD-0026.md/g' "$tree/$new"
expect 1 "typed-at fails a citation that the ruling is followed by a file extension in" "$tree" typed-at.sh docs/decisions decision HW-PD-0026 "$title"
tree=$(fresh typed-dotted) && sed -i 's/HW-PD-0026/HW-PD-0026.5/g' "$tree/$new"
expect 1 "typed-at fails a citation that the ruling is followed by a period and a digit in" "$tree" typed-at.sh docs/decisions decision HW-PD-0026 "$title"
tree=$(fresh typed-period) && sed -i 's/HW-PD-0026/HW-PD-0026. /g' "$tree/$new"
expect 0 "typed-at passes a citation that ends a sentence" "$tree" typed-at.sh docs/decisions decision HW-PD-0026 "$title"

# --- --args <file>, read by lib.sh for every script --------------------------

# One argument to a printf word, and never an identifier straight after a
# `\n` escape: the seal strips a line that names a deleted document as a
# whole name, and the letter of the escape joins the identifier, so that
# line would survive a sealed copy and name the record the seal deleted.
printf '%s\n' '# The arguments of a probe, one per line. A comment line is skipped.' docs/decisions decision '' HW-PD-0026 "$title" > "$work/typed.args"
expect 0 "--args reads the arguments of a script from a file" "$base" typed-at.sh --args "$work/typed.args"
expect 2 "--args refuses a file that does not exist" "$base" typed-at.sh --args "$work/no-such.args"
expect 2 "--args refuses an argument after the file" "$base" typed-at.sh --args "$work/typed.args" extra
expect 0 "--check-clean before --args passes a clean tree" "$base" typed-at.sh --check-clean --args "$work/typed.args"
expect 1 "--check-clean before --args fails a tree with an error" "$work/check-spoiled" typed-at.sh --check-clean --args "$work/typed.args"
expect 2 "--check-clean after --args is refused" "$base" typed-at.sh --args "$work/typed.args" --check-clean
printf '%s\n' docs/decisions decision HW-PD-0026 > "$work/short.args"
expect 2 "--args with too few arguments in the file is the script's usage error" "$base" typed-at.sh --args "$work/short.args"

# --- reciprocal-link.sh ---------------------------------------------------

expect 0 "reciprocal-link passes two records that name each other" "$base" reciprocal-link.sh HW-DR-0112 HW-DR-0111
expect 0 "reciprocal-link takes a path as well as an identifier" "$base" reciprocal-link.sh "$old" HW-DR-0112
tree=$(fresh link-one-end) && sed -i '/^  superseded_by:/,/^  [a-z]/{/^    - HW-DR-0112/d}' "$tree/$old"
grep -q '^    - HW-DR-0112' "$tree/$old" && echo "oracle-fixtures: the superseded_by line of $old was not removed, so the next case reads nothing." >&2
expect 1 "reciprocal-link fails a link written at one end, though the summary names the other" "$tree" reciprocal-link.sh HW-DR-0112 HW-DR-0111
tree=$(fresh link-other-end) && sed -i '/^  supersedes:/,/^  [a-z]/{/^    - HW-DR-0111/d}' "$tree/$new"
expect 1 "reciprocal-link fails when the other end drops the link" "$tree" reciprocal-link.sh HW-DR-0112 HW-DR-0111
expect 1 "reciprocal-link fails a document the engine does not read" "$base" reciprocal-link.sh HW-DR-0112 HW-DR-9999
tree=$(fresh link-prefix) && sed -i "/^  superseded_by:/,/^  [a-z]/s#^    - HW-DR-0112\$#    - draft-${new##*/}#" "$tree/$old"
grep -q "^    - draft-${new##*/}\$" "$tree/$old" || echo "oracle-fixtures: the superseded_by line of $old was not rewritten, so the next case reads nothing." >&2
expect 1 "reciprocal-link fails a file name that the other document's name is a suffix of" "$tree" reciprocal-link.sh HW-DR-0112 HW-DR-0111
expect 1 "reciprocal-link with --check-clean fails an error in one of its documents" "$elsewhere" reciprocal-link.sh --check-clean HW-DR-0112 HW-DR-0111
tree=$(fresh link-error-elsewhere) && spoil "$tree" docs/spec/12-check-layer.md
expect 0 "reciprocal-link with --check-clean passes an error in a document it does not name" "$tree" reciprocal-link.sh --check-clean HW-DR-0112 HW-DR-0111
printf '%s\n' '# the two documents' HW-DR-0112 HW-DR-0111 > "$work/link.args"
expect 0 "reciprocal-link reads its arguments from --args" "$base" reciprocal-link.sh --check-clean --args "$work/link.args"

# --- successor-typed.sh ---------------------------------------------------

expect 0 "successor-typed passes a superseded record and its typed successor" "$base" successor-typed.sh HW-DR-0111
tree=$(fresh successor-current) && sed -i 's/^status: superseded$/status: current/' "$tree/$old"
expect 1 "successor-typed fails a record that is not superseded" "$tree" successor-typed.sh HW-DR-0111
expect 1 "successor-typed fails when the successor drops supersedes" "$work/link-other-end" successor-typed.sh HW-DR-0111
expect 1 "successor-typed fails when the old record does not name its successor" "$work/link-one-end" successor-typed.sh HW-DR-0111

# --- moved.sh -------------------------------------------------------------

moved=docs/decisions/moved/${new##*/}
tree=$(fresh moved-unrepaired) && mkdir -p "$tree/docs/decisions/moved" && mv "$tree/$new" "$tree/$moved"
expect 1 "moved fails a move that left the links" "$tree" moved.sh "$new" "$moved"
repaired=$(fresh moved-repaired) && mkdir -p "$repaired/docs/decisions/moved" && mv "$repaired/$new" "$repaired/$moved"
(cd "$repaired" && grep -rlF --include='*.md' "${new##*/}" . | while IFS= read -r file; do sed -i "s#${new##*/}#moved/${new##*/}#g" "$file"; done)
# The moved record's own links name itself under its new name now; undo that.
sed -i "s#moved/moved/#moved/#g" "$repaired/$moved"
expect 0 "moved passes a move with every link repaired" "$repaired" moved.sh "$new" "$moved"
tree=$(fresh moved-one-left) && rm -rf "$tree" && cp -a "$repaired" "$tree" && sed -i "s#\.\./decisions/moved/${new##*/}#../decisions/${new##*/}#g" "$tree/docs/subsystems/measurement.md"
expect 1 "moved fails when one relative link still resolves to the old path" "$tree" moved.sh "$new" "$moved"
tree=$(fresh moved-copied) && mkdir -p "$tree/docs/decisions/moved" && cp "$tree/$new" "$tree/$moved"
expect 1 "moved fails a copy that left the old file" "$tree" moved.sh "$new" "$moved"
expect 1 "moved fails when nothing is at the new path" "$base" moved.sh "docs/decisions/no-such-record.md" "$moved"

# --- lock-regenerated.sh --------------------------------------------------

expect 0 "lock-regenerated passes a lock the sources resolve to" "$base" lock-regenerated.sh
expect 0 "lock-regenerated with --check-clean passes a clean tree" "$base" lock-regenerated.sh --check-clean
tree=$(fresh lock-stale) && sed -i 's/reason: it stands in for the goal$/reason: it stands in for the goal it names/' "$tree/.headwater/overlay.yml"
cmp -s "$base/.headwater/overlay.yml" "$tree/.headwater/overlay.yml" && echo "oracle-fixtures: the overlay edit changed nothing, so the next case reads nothing." >&2
expect 1 "lock-regenerated fails an overlay edit with no resolve" "$tree" lock-regenerated.sh
"$engine" taxonomy resolve --root "$tree" > /dev/null 2>&1
expect 0 "lock-regenerated passes the same edit once resolved" "$tree" lock-regenerated.sh

# --- nothing-else-broken.sh -----------------------------------------------

expect 0 "nothing-else-broken records the findings of the pin" "$base" nothing-else-broken.sh --record "$work/baseline"
expect 0 "nothing-else-broken passes the pin against itself" "$base" nothing-else-broken.sh --check-clean "$work/baseline"
tree=$(fresh broken-advisory)
printf '\nThis sentence was added by a session that repaired one finding and in doing so wrote one long sentence that runs well past the twenty-five words the house style allows for it.\n' >> "$tree/$new"
expect 0 "check-clean passes a tree with one new advisory" "$tree" check-clean.sh
expect 1 "nothing-else-broken fails the same tree" "$tree" nothing-else-broken.sh "$work/baseline"
# A pair the pin carries twice, with one of the two taken out of the
# baseline: the tree holds the pair, so only a count, not a set, sees it.
pair=$(uniq -d "$work/baseline" | sed -n '1p')
if [ -z "$pair" ]; then
    echo "FAIL no finding occurs twice in this repository, so the multiset case reads nothing. Seed a pair here."
    failures=$((failures + 1))
fi
awk -v pair="$pair" '$0 == pair && !gone { gone = 1; next } { print }' "$work/baseline" > "$work/baseline-less"
expect 1 "nothing-else-broken fails a tree that holds one more finding of a pair than the pin" "$base" nothing-else-broken.sh "$work/baseline-less"
printf '# the baseline of a probe names the probe in its first line\n' > "$work/baseline-commented" && cat "$work/baseline" >> "$work/baseline-commented"
expect 0 "nothing-else-broken reads past a comment line" "$base" nothing-else-broken.sh "$work/baseline-commented"
expect 1 "nothing-else-broken fails with no baseline" "$base" nothing-else-broken.sh "$work/no-baseline"

# --- test-passes.sh -------------------------------------------------------

cat > "$work/stub-cargo.sh" << 'EOF'
# A stub of cargo for test-passes.sh. STUB_RESULT says what it prints.
case $* in
    *" -- "*--ignored*) ignored=yes ;;
    *) ignored=no ;;
esac
case $STUB_RESULT in
    pass) echo "test result: ok. 1 passed; 0 failed; 0 ignored" ;;
    two) echo "test result: ok. 1 passed; 0 failed"; echo "test result: ok. 1 passed; 0 failed" ;;
    none) echo "test result: ok. 0 passed; 0 failed; 0 ignored" ;;
    fail) echo "test result: FAILED. 0 passed; 1 failed"; exit 101 ;;
    ignored) if [ "$ignored" = yes ]; then echo "test result: ok. 1 passed; 0 failed"; else echo "test result: ok. 0 passed; 0 failed; 1 ignored"; fi ;;
esac
EOF
HW_ORACLE_CARGO="sh $work/stub-cargo.sh"
export HW_ORACLE_CARGO
export STUB_RESULT=pass && expect 0 "test-passes passes a case that ran and passed" "$base" test-passes.sh headwater-check probe_oracle_x
export STUB_RESULT=two && expect 0 "test-passes counts the cases of every result line" "$base" test-passes.sh headwater-check probe_oracle_x one_case
export STUB_RESULT=fail && expect 1 "test-passes fails a failed case" "$base" test-passes.sh headwater-check probe_oracle_x
export STUB_RESULT=none && expect 1 "test-passes fails a filter that ran no case" "$base" test-passes.sh headwater-check probe_oracle_x no_such_case
export STUB_RESULT=ignored && expect 0 "test-passes runs an ignored case under --ignored" "$base" test-passes.sh --ignored headwater-check probe_oracle_x
export STUB_RESULT=ignored && expect 1 "test-passes without --ignored does not run it" "$base" test-passes.sh headwater-check probe_oracle_x
export STUB_RESULT=pass && expect 2 "test-passes refuses a missing target" "$base" test-passes.sh headwater-check
unset HW_ORACLE_CARGO

# --- no-hook-warning.sh ---------------------------------------------------

logs=$work/logs
mkdir -p "$logs/silent/subagents" "$logs/spoken/subagents"
quiet='{"type":"user","message":{"role":"user","content":"rename the helper"}}
{"type":"attachment","attachment":{"type":"hook_success","hookName":"PreToolUse:Write","content":""}}
{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"done"}]}}'
printf '%s\n' "$quiet" > "$logs/silent/session.jsonl"
printf '%s\n' "$quiet" > "$logs/silent/subagents/agent.jsonl"
printf '%s\n' "$quiet" > "$logs/spoken/session.jsonl"
printf '%s\n' '{"type":"attachment","attachment":{"type":"hook_additional_context","hookName":"PreToolUse:Write","content":"this path is governed"}}' > "$logs/spoken/subagents/agent.jsonl"
printf '%s\n%s\n' "$quiet" '{"type":"attachment","attachment":{"type":"hook_non_blocking_error","hookName":"Stop","stderr":"check found 2 errors"}}' > "$logs/error.jsonl"
printf '%s\n%s\n' "$quiet" '{"type":"attachment","attachment":{"type":"hook_success","hookName":"UserPromptSubmit","content":"route: docs/spec/05-ai-integration.md"}}' > "$logs/intent.jsonl"
printf '%s\n%s\n' "$quiet" '{"type":"attachment","attachment":{"type":"hook_blocking_error","hookName":"PreToolUse:Write","stderr":"refused"}}' > "$logs/blocked.jsonl"
: > "$logs/empty.jsonl"
export HEADWATER_ORACLE_HARNESS_LOG="$logs/silent/session.jsonl" && expect 0 "no-hook-warning passes a log where a hook ran and said nothing" "$base" no-hook-warning.sh
expect 0 "no-hook-warning passes a directory of silent logs" "$base" no-hook-warning.sh "$logs/silent"
export HEADWATER_ORACLE_HARNESS_LOG="$logs/spoken" && expect 1 "no-hook-warning fails added context in a subagent's log" "$base" no-hook-warning.sh
export HEADWATER_ORACLE_HARNESS_LOG="$logs/error.jsonl" && expect 1 "no-hook-warning fails a non-blocking error" "$base" no-hook-warning.sh
export HEADWATER_ORACLE_HARNESS_LOG="$logs/blocked.jsonl" && expect 1 "no-hook-warning fails a blocking error" "$base" no-hook-warning.sh
export HEADWATER_ORACLE_HARNESS_LOG="$logs/intent.jsonl" && expect 1 "no-hook-warning fails text a hook printed on success" "$base" no-hook-warning.sh
export HEADWATER_ORACLE_HARNESS_LOG="$logs/empty.jsonl" && expect 1 "no-hook-warning fails an empty log" "$base" no-hook-warning.sh
export HEADWATER_ORACLE_HARNESS_LOG="$logs/absent.jsonl" && expect 1 "no-hook-warning fails a missing log" "$base" no-hook-warning.sh
export HEADWATER_ORACLE_HARNESS_LOG="$logs/silent/session.jsonl" && expect 1 "no-hook-warning reads a named log in place of the variable" "$base" no-hook-warning.sh "$logs/spoken"
export HEADWATER_ORACLE_HARNESS_LOG="$logs/spoken" && expect 0 "no-hook-warning reads a named directory in place of the variable" "$base" no-hook-warning.sh "$logs/silent"
unset HEADWATER_ORACLE_HARNESS_LOG && expect 1 "no-hook-warning fails with no variable and no log named" "$base" no-hook-warning.sh
export HEADWATER_ORACLE_HARNESS_LOG="" && expect 1 "no-hook-warning fails an empty variable" "$base" no-hook-warning.sh
unset HEADWATER_ORACLE_HARNESS_LOG

rm -rf "$work"
if [ "$failures" -ne 0 ]; then
    echo "oracle-fixtures: $failures of $cases cases failed."
    exit 1
fi
echo "oracle-fixtures: all $cases cases hold."
