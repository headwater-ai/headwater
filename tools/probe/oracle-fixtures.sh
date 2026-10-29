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
# The old record names a successor whose identifier the real one is a prefix
# of, so a substring match would read it as the link (#1874.3).
tree=$(fresh successor-prefix) && sed -i '/^  superseded_by:/,/^  [a-z]/s/^    - HW-DR-0112$/    - HW-DR-01120/' "$tree/$old"
grep -q '^    - HW-DR-01120$' "$tree/$old" || echo "oracle-fixtures: the superseded_by line of $old was not rewritten, so the next case reads nothing." >&2
expect 1 "successor-typed fails an old record that names an identifier the successor's is a prefix of" "$tree" successor-typed.sh HW-DR-0111
tree=$(fresh successor-supersedes-prefix) && sed -i '/^  supersedes:/,/^  [a-z]/s/^    - HW-DR-0111$/    - HW-DR-01110/' "$tree/$new"
grep -q '^    - HW-DR-011[1]0$' "$tree/$new" || echo "oracle-fixtures: the supersedes line of $new was not rewritten, so the next case reads nothing." >&2
expect 1 "successor-typed fails a document that supersedes an identifier the old record's is a prefix of" "$tree" successor-typed.sh HW-DR-0111
# A sealed tree holds errors elsewhere, so `--check-clean` reads the two
# documents the script names: the old record and the successor it found.
expect 0 "successor-typed with --check-clean passes an error in a document it does not name" "$work/link-error-elsewhere" successor-typed.sh --check-clean HW-DR-0111
expect 1 "successor-typed with --check-clean fails an error in the old record" "$elsewhere" successor-typed.sh --check-clean HW-DR-0111
expect 1 "successor-typed with --check-clean fails an error in the successor" "$work/check-spoiled" successor-typed.sh --check-clean HW-DR-0111

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
# `--check-clean` reads the moved file and every file that links it by its
# file name, wherever the link resolves (#1874.3). The repaired tree above
# left the relative links inside the moved file as they were, so four of them
# resolve to nothing now, and the strict check reports them on the moved file.
expect 1 "moved with --check-clean fails a move that left the links inside the moved file broken" "$repaired" moved.sh --check-clean "$new" "$moved"
clean=$(fresh moved-clean) && rm -rf "$clean" && cp -a "$repaired" "$clean" && sed -i 's#](\.\./#](../../#g; s#](0111-#](../0111-#g' "$clean/$moved"
expect 0 "moved with --check-clean passes a move with every link repaired, inside the file too" "$clean" moved.sh --check-clean "$new" "$moved"
tree=$(fresh moved-error-elsewhere) && rm -rf "$tree" && cp -a "$clean" "$tree" && spoil "$tree" docs/decisions/0001-implementation-language.md
expect 0 "moved with --check-clean passes an error in a file that does not link the moved file" "$tree" moved.sh --check-clean "$new" "$moved"
tree=$(fresh moved-error-moved) && rm -rf "$tree" && cp -a "$clean" "$tree" && spoil "$tree" "$moved"
expect 1 "moved with --check-clean fails an error in the moved file" "$tree" moved.sh --check-clean "$new" "$moved"
# A sealed tree leaves errors in a file that links the moved file, about its
# other links, so on such a file only an error that names the moved file's
# name counts.
tree=$(fresh moved-error-linker) && rm -rf "$tree" && cp -a "$clean" "$tree" && spoil "$tree" docs/subsystems/measurement.md
expect 0 "moved with --check-clean passes an error that does not name the moved file in a file that links it" "$tree" moved.sh --check-clean "$new" "$moved"
printf '\nSee [the record](../decisions/elsewhere/%s).\n' "${new##*/}" >> "$tree/docs/subsystems/measurement.md"
expect 1 "moved with --check-clean fails an error that names the moved file in a file that links it" "$tree" moved.sh --check-clean "$new" "$moved"
# A link rewritten to a path where nothing stands names the file and resolves
# to neither path: `moved.sh` alone passes it, and the strict check does not.
tree=$(fresh moved-misrepaired) && rm -rf "$tree" && cp -a "$clean" "$tree" && sed -i "s#\.\./decisions/moved/${new##*/}#../decisions/elsewhere/${new##*/}#g" "$tree/docs/subsystems/measurement.md"
expect 0 "moved passes a link repaired to a path where nothing stands" "$tree" moved.sh "$new" "$moved"
expect 1 "moved with --check-clean fails the same link" "$tree" moved.sh --check-clean "$new" "$moved"

# --- lock-regenerated.sh --------------------------------------------------

expect 0 "lock-regenerated passes a lock the sources resolve to" "$base" lock-regenerated.sh
expect 0 "lock-regenerated with --check-clean passes a clean tree" "$base" lock-regenerated.sh --check-clean
tree=$(fresh lock-stale) && sed -i 's/reason: it stands in for the goal$/reason: it stands in for the goal it names/' "$tree/.headwater/overlay.yml"
cmp -s "$base/.headwater/overlay.yml" "$tree/.headwater/overlay.yml" && echo "oracle-fixtures: the overlay edit changed nothing, so the next case reads nothing." >&2
expect 1 "lock-regenerated fails an overlay edit with no resolve" "$tree" lock-regenerated.sh
"$engine" taxonomy resolve --root "$tree" > /dev/null 2>&1
expect 0 "lock-regenerated passes the same edit once resolved" "$tree" lock-regenerated.sh
# On a tree whose lock matches its sources the bare form passes, so a probe
# names what the change must hold: `<file> <literal>` pairs, each read as a
# whole name (#1874.3). `--check-clean` then reads the files it names.
overlay=.headwater/overlay.yml
expect 1 "lock-regenerated fails a file that does not hold the literal" "$base" lock-regenerated.sh "$overlay" 'term: game-changer'
expect 2 "lock-regenerated refuses a file with no literal" "$base" lock-regenerated.sh "$overlay"
expect 2 "lock-regenerated refuses a file that does not exist" "$base" lock-regenerated.sh .headwater/no-such-file 'term: game-changer'
retired=$(fresh lock-retired) && awk '{ print } $0 == "        reason: it stands in for the failure the thing survives" { print "      - term: game-changer"; print "        reason: it stands in for the change the thing makes" }' "$base/$overlay" > "$retired/$overlay"
cmp -s "$base/$overlay" "$retired/$overlay" && echo "oracle-fixtures: the retired term was not added, so the next cases read nothing." >&2
expect 1 "lock-regenerated fails the literal with no resolve" "$retired" lock-regenerated.sh "$overlay" 'term: game-changer'
"$engine" taxonomy resolve --root "$retired" > /dev/null 2>&1
expect 0 "lock-regenerated passes the literal once resolved" "$retired" lock-regenerated.sh "$overlay" 'term: game-changer'
expect 1 "lock-regenerated fails when one of two literals is absent" "$retired" lock-regenerated.sh "$overlay" 'term: game-changer' "$overlay" 'replacement: use'
expect 1 "lock-regenerated fails a literal that the file holds only as the start of a longer name" "$retired" lock-regenerated.sh "$overlay" 'term: game'
# A literal that only a YAML comment holds declares nothing, so it fails,
# both on a comment line and after a value. Each tree is resolved, so the
# lock is not why it fails (verify of #1874.3).
tree=$(fresh lock-commented) && awk '{ print } $0 == "        reason: it stands in for the failure the thing survives" { print "      # - term: game-changer" }' "$base/$overlay" > "$tree/$overlay"
grep -q '^      # - term: game-changer$' "$tree/$overlay" || echo "oracle-fixtures: the comment line was not added, so the next case reads nothing." >&2
"$engine" taxonomy resolve --root "$tree" > /dev/null 2>&1
expect 1 "lock-regenerated fails a literal that only a comment line holds" "$tree" lock-regenerated.sh "$overlay" 'term: game-changer'
tree=$(fresh lock-column-comment) && cp "$base/$overlay" "$tree/$overlay" && printf '# term: game-changer\n' >> "$tree/$overlay"
"$engine" taxonomy resolve --root "$tree" > /dev/null 2>&1
expect 1 "lock-regenerated fails a literal that only a comment in the first column holds" "$tree" lock-regenerated.sh "$overlay" 'term: game-changer'
tree=$(fresh lock-trailing-comment) && sed -i 's/^        reason: it stands in for the failure the thing survives$/& # term: game-changer/' "$tree/$overlay"
grep -q ' # term: game-changer$' "$tree/$overlay" || echo "oracle-fixtures: the trailing comment was not added, so the next case reads nothing." >&2
"$engine" taxonomy resolve --root "$tree" > /dev/null 2>&1
expect 1 "lock-regenerated fails a literal that only a comment after a value holds" "$tree" lock-regenerated.sh "$overlay" 'term: game-changer'
# A finding that a directive suppresses is not an error a strict run fails
# on, so a subject that carries one passes `--check-clean`. This how-to
# holds suppressed `surface.local_path.instructed` errors (verify of #1874.3).
suppressed=$("$engine" check --json --root "$base" 2> /dev/null | jq '[.findings[] | select(.path == "docs/how-to/publish-your-corpus-as-a-site.md" and .severity == "error" and .escape != "none")] | length')
if [ "${suppressed:-0}" -eq 0 ]; then
    echo "FAIL docs/how-to/publish-your-corpus-as-a-site.md holds no suppressed error, so the next case reads nothing. Name another file that does."
    failures=$((failures + 1))
fi
expect 0 "lock-regenerated with --check-clean passes a file whose only errors a directive suppresses" "$base" lock-regenerated.sh --check-clean docs/how-to/publish-your-corpus-as-a-site.md 'id: HW-HOW-publish-your-corpus-as-a-site'
spoil "$retired" docs/spec/12-check-layer.md
expect 0 "lock-regenerated with --check-clean passes an error in a file it does not name" "$retired" lock-regenerated.sh --check-clean "$overlay" 'term: game-changer'
expect 1 "lock-regenerated with --check-clean and no file fails the same error" "$retired" lock-regenerated.sh --check-clean

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
# A baseline with `# scope: <path>` lines reads the findings on those files
# alone, and `--check-clean` reads the strict errors on them (#1874.3). A
# committed baseline of the whole tree moves with every finding any change
# adds anywhere, so the baseline of a probe holds the file it repairs.
printf '%s\n' '# the baseline of a probe, scoped to the file it repairs' "# scope: $new" > "$work/scoped"
expect 0 "nothing-else-broken --record keeps the comments and the scope of a baseline" "$base" nothing-else-broken.sh --record "$work/scoped"
if ! grep -q "^# scope: $new\$" "$work/scoped" || awk -F '\t' -v path="$new" '!/^#/ && $2 != path { bad = 1 } END { exit !bad }' "$work/scoped"; then
    echo "FAIL nothing-else-broken --record keeps the scope line and records the scoped file alone"
    failures=$((failures + 1))
fi
expect 1 "nothing-else-broken with a scope fails a new finding in the scoped file" "$work/broken-advisory" nothing-else-broken.sh "$work/scoped"
tree=$(fresh broken-elsewhere)
printf '\nThis sentence was added by a session that repaired one finding and in doing so wrote one long sentence that runs well past the twenty-five words the house style allows for it.\n' >> "$tree/$old"
expect 1 "nothing-else-broken with no scope fails a new finding in another file" "$tree" nothing-else-broken.sh "$work/baseline"
expect 0 "nothing-else-broken with a scope passes a new finding in another file" "$tree" nothing-else-broken.sh "$work/scoped"
expect 0 "nothing-else-broken with a scope and --check-clean passes an error in another file" "$elsewhere" nothing-else-broken.sh --check-clean "$work/scoped"
awk '!/^#/ && !gone { gone = 1; next } { print }' "$work/scoped" > "$work/scoped-less"
expect 1 "nothing-else-broken with a scope fails the pin against a baseline less one of its findings" "$base" nothing-else-broken.sh "$work/scoped-less"
# A `# keep: <path> <literal>` line fails a session that deletes the text the
# repair keeps, which clears its finding and raises nothing (verify of
# #1874.3).
keep_literal='by the third slice of #1874'
grep -q -F -- "$keep_literal" "$base/$new" || echo "oracle-fixtures: $new does not hold \`$keep_literal\`, so the keep cases read nothing." >&2
{ cat "$work/scoped"; printf '# keep: %s %s\n' "$new" "$keep_literal"; } > "$work/scoped-keep"
expect 0 "nothing-else-broken passes the pin when it keeps the text a keep line names" "$base" nothing-else-broken.sh --check-clean "$work/scoped-keep"
tree=$(fresh keep-deleted) && grep -v -F -- "$keep_literal" "$base/$new" > "$tree/$new"
expect 0 "nothing-else-broken with no keep line passes a tree that deleted the paragraph" "$tree" nothing-else-broken.sh "$work/scoped"
expect 1 "nothing-else-broken with a keep line fails the same tree" "$tree" nothing-else-broken.sh "$work/scoped-keep"
{ cat "$work/scoped"; printf '# keep: %s\n' "$new"; } > "$work/scoped-keep-bad"
expect 2 "nothing-else-broken refuses a keep line with no literal" "$base" nothing-else-broken.sh "$work/scoped-keep-bad"

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
