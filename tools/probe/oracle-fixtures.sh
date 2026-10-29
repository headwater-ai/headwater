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

# The meaning of each `(relation, inbound)` pair of `headwater explain
# --json`, fixed here because the oracles read it (#1874.3). An edge that a
# document declares under `supersedes` or under its inverse `superseded_by`
# is printed on that document as `supersedes`, outbound, with the target as
# written. On the far document it is `superseded_by`, inbound, with the
# declaring document's path. So the relation string does not say which key
# was written, and successor-typed.sh tells the two ends apart by status. A
# key that names nothing is printed as one outbound edge to the target `~`.
# A case here fails when the engine prints these pairs another way.
explain_pairs() {
    "$engine" explain --json --root "$1" "$2" 2> /dev/null | jq -c '[.related[] | select(.relation == "supersedes" or .relation == "superseded_by") | [.relation, .inbound, (.targets | map(sub("^docs/decisions/(?<n>[0-9]+)-.*"; "path:\(.n)")))]] | sort_by(.)'
}
pairs_case() {
    cases=$((cases + 1))
    got=$(explain_pairs "$2" "$3")
    if [ "$got" = "$4" ]; then
        echo "ok   $1: $got"
    else
        echo "FAIL $1: explain --json prints $got, and the contract is $4"
        failures=$((failures + 1))
    fi
}
pairs_case "explain prints the superseded_by that HW-DR-0111 declares as supersedes, outbound, and the far end as superseded_by, inbound" "$base" HW-DR-0111 '[["superseded_by",true,["path:0112"]],["supersedes",false,["HW-DR-0112"]]]'
pairs_case "explain prints the supersedes that HW-DR-0112 declares as supersedes, outbound, and the far end as superseded_by, inbound" "$base" HW-DR-0112 '[["superseded_by",true,["path:0111"]],["supersedes",false,["HW-DR-0111"]]]'
pairs_case "explain prints no outbound supersedes to HW-DR-0112 on HW-DR-0111 once its superseded_by names nothing, and the empty key as the target ~" "$work/link-one-end" HW-DR-0111 '[["superseded_by",true,["path:0112"]],["supersedes",false,["~"]]]'
pairs_case "explain prints no outbound supersedes to HW-DR-0111 on HW-DR-0112 once its supersedes names nothing, and the empty key as the target ~" "$work/link-other-end" HW-DR-0112 '[["superseded_by",true,["path:0111"]],["supersedes",false,["~"]]]'

# The shape is versioned, and the scripts read version 1. An engine that
# prints another major version stops the grade with a usage error, so a
# change of shape never passes a session. A minor version reads as 1.
explain_stub() {
    printf '#!/bin/sh\nif [ "$1" = explain ]; then\n    "%s" "$@" | jq -c %s\n    exit 0\nfi\nexec "%s" "$@"\n' "$engine" "'.version = \"$1\"'" "$engine" > "$work/engine-$1.sh"
    chmod +x "$work/engine-$1.sh"
    printf '%s\n' "$work/engine-$1.sh"
}
HW_ORACLE_ENGINE=$(explain_stub 2.0)
expect 2 "successor-typed refuses an explain --json of major version 2" "$base" successor-typed.sh HW-DR-0111
HW_ORACLE_ENGINE=$(explain_stub 1.9)
expect 0 "successor-typed reads an explain --json of version 1.9" "$base" successor-typed.sh HW-DR-0111
HW_ORACLE_ENGINE=$engine

# move_items <file> <key-to-drop> <key-to-add> [flow]: move the items of one
# relation of <file> to another, as a session that rewrites the text of an
# edge does. With `flow`, the items go into a comment after an empty flow
# list.
move_items() {
    python3 -I - "$@" << 'PY'
import sys

path, drop, add = sys.argv[1:4]
flow = sys.argv[4:] == ["flow"]
lines = open(path, encoding="utf-8").read().split("\n")
end = lines.index("---", 1)
front, rest = lines[1:end], lines[end:]
items, kept, inside = [], [], False
for line in front:
    if line == "  " + drop + ":":
        inside = True
        continue
    if inside and line.startswith("    - "):
        items.append(line[6:])
        continue
    inside = False
    kept.append(line)
if not items:
    sys.exit("no item under " + drop + " in " + path)
if flow:
    at = kept.index("relations:") + 1
    kept[at:at] = ["  " + add + ": [] # " + " ".join(items)]
elif "  " + add + ":" in kept:
    at = kept.index("  " + add + ":") + 1
    kept[at:at] = ["    - " + item for item in items]
else:
    at = kept.index("relations:") + 1
    kept[at:at] = ["  " + add + ":"] + ["    - " + item for item in items]
open(path, "w", encoding="utf-8").write("\n".join(["---"] + kept + rest))
PY
}
# The successor's list written as an empty flow list whose comment names the
# old record, and the old record naming its successor under traces_to: a
# reader of lines finds both ends, and the engine parses neither (round 4 of
# #1874.3, parent-slice-3-stop.md, mutants 2 and 3).
tree=$(fresh successor-flow-comment) && move_items "$tree/$new" supersedes supersedes flow
grep -q '^  supersedes: \[\] # HW-DR-0111$' "$tree/$new" || echo "oracle-fixtures: the supersedes list of $new was not rewritten, so the next case reads nothing." >&2
expect 1 "successor-typed fails a successor whose supersedes is an empty flow list that a comment names the old record in" "$tree" successor-typed.sh HW-DR-0111
tree=$(fresh successor-traces) && move_items "$tree/$old" superseded_by traces_to
expect 1 "successor-typed fails an old record that names its successor under traces_to" "$tree" successor-typed.sh HW-DR-0111
tree=$(fresh successor-both-text) && move_items "$tree/$new" supersedes supersedes flow && move_items "$tree/$old" superseded_by traces_to
expect 1 "successor-typed with --check-clean fails both ends written as text and neither as an edge" "$tree" successor-typed.sh --check-clean HW-DR-0111
# Status is read as the top-level key of the front matter. A comment, or the
# same words under another key, is no status.
tree=$(fresh successor-status-comment) && sed -i 's/^status: superseded$/status: current\n# status: superseded/' "$tree/$old"
grep -q '^# status: superseded$' "$tree/$old" || echo "oracle-fixtures: the status of $old was not commented, so the next case reads nothing." >&2
expect 1 "successor-typed fails a status that only a comment holds" "$tree" successor-typed.sh HW-DR-0111
# A record that supersedes another is printed with an outbound `supersedes`
# to it, as its successor is, so the record it superseded reads as its
# successor unless the successor's own status is read: HW-DR-0112 marked
# superseded has no successor, and HW-DR-0111 is not one.
tree=$(fresh successor-predecessor) && sed -i 's/^status: current$/status: superseded/' "$tree/$new"
grep -q '^status: superseded$' "$tree/$new" || echo "oracle-fixtures: the status of $new was not set, so the next case reads nothing." >&2
expect 1 "successor-typed fails a record whose only succession edge is to the record it superseded" "$tree" successor-typed.sh HW-DR-0112

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
# to neither path. The regex of `moved.sh` reads no stale link there, and the
# engine's link rule reads a link that names the file name and resolves to
# nothing, so `moved.sh` fails it, with `--check-clean` and without (round 4
# of #1874.3).
tree=$(fresh moved-misrepaired) && rm -rf "$tree" && cp -a "$clean" "$tree" && sed -i "s#\.\./decisions/moved/${new##*/}#../decisions/elsewhere/${new##*/}#g" "$tree/docs/subsystems/measurement.md"
expect 1 "moved fails a link repaired to a path where nothing stands" "$tree" moved.sh "$new" "$moved"
expect 1 "moved with --check-clean fails the same link" "$tree" moved.sh --check-clean "$new" "$moved"
# A link that the link regex of moved.sh does not read, a reference-style
# link, still resolves to the old path in a file that links nothing else of
# the move. The engine's link rule reads that form, so moved.sh fails it on
# the engine's `link.path.unresolved` finding (round 4 of #1874.3).
tree=$(fresh moved-reference-link) && rm -rf "$tree" && cp -a "$clean" "$tree" && printf '\nThe [moved record][moved-old] is named here by a reference.\n\n[moved-old]: ../decisions/%s\n' "${new##*/}" >> "$tree/docs/spec/12-check-layer.md"
expect 1 "moved fails a reference-style link that still resolves to the old path" "$tree" moved.sh "$new" "$moved"
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
# names what the change must hold, and the engine's own retired-term rule
# reads it in a scratch root: `retires <carrier> <term> [<replacement>]`
# and `binds <path> <term>` (round 4 of #1874.3). No line of the overlay is
# matched, so a term that only a comment or a quoted reason holds retires
# nothing.
overlay=.headwater/overlay.yml
carrier=docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md
[ -f "$base/$carrier" ] || echo "oracle-fixtures: $carrier is gone, so the retires cases read nothing. Name another decision here." >&2
anchor='        reason: it stands in for the failure the thing survives'
grep -q -x -F -- "$anchor" "$base/$overlay" || echo "oracle-fixtures: the overlay holds no line \`$anchor\`, so the retires cases read nothing." >&2
expect 1 "lock-regenerated fails a term that no entry retires" "$base" lock-regenerated.sh retires "$carrier" game-changer
expect 0 "lock-regenerated passes a term that the pin already retires" "$base" lock-regenerated.sh retires "$carrier" load-bearing
expect 2 "lock-regenerated refuses a retires with no term" "$base" lock-regenerated.sh retires "$carrier"
expect 2 "lock-regenerated refuses a carrier that does not exist" "$base" lock-regenerated.sh retires docs/decisions/no-such-record.md game-changer
expect 2 "lock-regenerated refuses the line form of round 3, a file and a literal" "$base" lock-regenerated.sh "$overlay" 'term: game-changer'
retired=$(fresh lock-retired) && awk -v a="$anchor" '{ print } $0 == a { print "      - term: game-changer"; print "        reason: it stands in for the change the thing makes" }' "$base/$overlay" > "$retired/$overlay"
cmp -s "$base/$overlay" "$retired/$overlay" && echo "oracle-fixtures: the retired term was not added, so the next cases read nothing." >&2
expect 1 "lock-regenerated fails a retired term with no resolve" "$retired" lock-regenerated.sh retires "$carrier" game-changer
"$engine" taxonomy resolve --root "$retired" > /dev/null 2>&1
expect 0 "lock-regenerated passes a retired term once resolved" "$retired" lock-regenerated.sh retires "$carrier" game-changer
expect 1 "lock-regenerated fails when one of two terms is not retired" "$retired" lock-regenerated.sh retires "$carrier" game-changer retires "$carrier" cutting-edge
expect 1 "lock-regenerated fails a term whose entry has no replacement when the probe names one" "$retired" lock-regenerated.sh retires "$carrier" game-changer change
expect 1 "lock-regenerated fails a term that only starts a retired one" "$retired" lock-regenerated.sh retires "$carrier" game
# The finding is read for the term it names and not for a substring of its
# message: a carrier that writes the retired `game-changer` raises a finding
# whose message holds `game`, and `game` is still not retired.
tree=$(fresh lock-substring) && rm -rf "$tree" && cp -a "$retired" "$tree" && printf '\nThe change was a game-changer for the team.\n' >> "$tree/$carrier"
expect 1 "lock-regenerated fails a term that only the finding on a longer retired term holds" "$tree" lock-regenerated.sh retires "$carrier" game
# The parent's mutant 5 of round 3: no entry for the term, and `term:
# game-changer` inside the quoted reason of another entry, resolved.
tree=$(fresh lock-quoted) && sed "s/^$anchor\$/        reason: \"it stands in for the failure the thing survives, term: game-changer\"/" "$base/$overlay" > "$tree/$overlay"
grep -q 'survives, term: game-changer"$' "$tree/$overlay" || echo "oracle-fixtures: the quoted reason was not written, so the next case reads nothing." >&2
"$engine" taxonomy resolve --root "$tree" > /dev/null 2>&1
expect 1 "lock-regenerated fails a term that only the quoted reason of another entry holds" "$tree" lock-regenerated.sh retires "$carrier" game-changer
# A comment declares nothing, on a line of its own or after a value.
tree=$(fresh lock-commented) && awk -v a="$anchor" '{ print } $0 == a { print "      # - term: game-changer" }' "$base/$overlay" > "$tree/$overlay"
"$engine" taxonomy resolve --root "$tree" > /dev/null 2>&1
expect 1 "lock-regenerated fails a term that only a comment line holds" "$tree" lock-regenerated.sh retires "$carrier" game-changer
tree=$(fresh lock-trailing-comment) && sed -i "s/^$anchor\$/& # - term: game-changer/" "$tree/$overlay"
"$engine" taxonomy resolve --root "$tree" > /dev/null 2>&1
expect 1 "lock-regenerated fails a term that only a comment after a value holds" "$tree" lock-regenerated.sh retires "$carrier" game-changer
# A replacement is read on the term's own entry: the finding's remediation
# is `write <replacement>`. Two literals in two entries are not one entry.
tree=$(fresh lock-replacement) && awk -v a="$anchor" '{ print } $0 == a { print "      - term: utilize"; print "        reason: it stands in for use"; print "        replacement: use" }' "$base/$overlay" > "$tree/$overlay"
"$engine" taxonomy resolve --root "$tree" > /dev/null 2>&1
expect 0 "lock-regenerated passes a term retired with its replacement" "$tree" lock-regenerated.sh retires "$carrier" utilize use
expect 1 "lock-regenerated fails another replacement than the entry's" "$tree" lock-regenerated.sh retires "$carrier" utilize employ
tree=$(fresh lock-split) && awk -v a="$anchor" '{ print } $0 == a { print "        replacement: use"; print "      - term: utilize"; print "        reason: it stands in for use" }' "$base/$overlay" > "$tree/$overlay"
"$engine" taxonomy resolve --root "$tree" > /dev/null 2>&1
expect 1 "lock-regenerated fails a term with no replacement beside a replacement on another entry" "$tree" lock-regenerated.sh retires "$carrier" utilize use
# `binds <path> <term>`: the house rules reach <path>, which is how
# `outside_root` binds a file outside the document roots.
expect 1 "lock-regenerated fails a file that no list binds" "$base" lock-regenerated.sh binds TRADEMARKS.md load-bearing
bound=$(fresh lock-bound) && awk '{ print } $0 == "      - .github/ISSUE_TEMPLATE/issue.md" { print "      - TRADEMARKS.md" }' "$base/$overlay" > "$bound/$overlay"
"$engine" taxonomy resolve --root "$bound" > /dev/null 2>&1
expect 0 "lock-regenerated passes a file that outside_root binds, once resolved" "$bound" lock-regenerated.sh binds TRADEMARKS.md load-bearing
tree=$(fresh lock-bound-comment) && awk '{ print } $0 == "      - .github/ISSUE_TEMPLATE/issue.md" { print "      # - TRADEMARKS.md" }' "$base/$overlay" > "$tree/$overlay"
"$engine" taxonomy resolve --root "$tree" > /dev/null 2>&1
expect 1 "lock-regenerated fails a file that only a comment in outside_root names" "$tree" lock-regenerated.sh binds TRADEMARKS.md load-bearing
expect 1 "lock-regenerated fails a bound file that is gone" "$bound" lock-regenerated.sh binds NO-SUCH-FILE.md load-bearing
# A finding that a directive suppresses is not an error a strict run fails
# on, so a subject that carries one passes `--check-clean`. This how-to
# holds suppressed `surface.local_path.instructed` errors (verify of #1874.3).
suppressed=$("$engine" check --json --root "$base" 2> /dev/null | jq '[.findings[] | select(.path == "docs/how-to/publish-your-corpus-as-a-site.md" and .severity == "error" and .escape != "none")] | length')
if [ "${suppressed:-0}" -eq 0 ]; then
    echo "FAIL docs/how-to/publish-your-corpus-as-a-site.md holds no suppressed error, so the next case reads nothing. Name another file that does."
    failures=$((failures + 1))
fi
expect 0 "lock-regenerated with --check-clean passes a bound file whose only errors a directive suppresses" "$base" lock-regenerated.sh --check-clean binds docs/how-to/publish-your-corpus-as-a-site.md load-bearing
spoil "$retired" docs/spec/12-check-layer.md
expect 0 "lock-regenerated with --check-clean passes an error in a file it does not name" "$retired" lock-regenerated.sh --check-clean retires "$carrier" game-changer
expect 1 "lock-regenerated with --check-clean and no form fails the same error" "$retired" lock-regenerated.sh --check-clean
tree=$(fresh lock-overlay-error) && rm -rf "$tree" && cp -a "$retired" "$tree" && spoil "$tree" "$carrier"
expect 0 "lock-regenerated with --check-clean passes an error in the carrier, which the session does not change" "$tree" lock-regenerated.sh --check-clean retires "$carrier" game-changer

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
# A session that deletes the edge or the clause a finding is on clears the
# finding and raises nothing, so a baseline names the structure the repair
# keeps, and the engine reads it through `headwater explain --json`: `#
# edge: <path> <relation> <target>` and `# summary: <path> <literal>`. No
# line of the file is matched, so text that is not the structure keeps
# nothing: a comment, a body line, another key (round 4 of #1874.3).
edge_front='- .headwater/probe.yml'
grep -q -x -F -- "    $edge_front" "$base/$new" || echo "oracle-fixtures: $new does not govern .headwater/probe.yml in its front matter, so the edge cases read nothing." >&2
{ cat "$work/scoped"; printf '# edge: %s governs .headwater/probe.yml\n' "$new"; } > "$work/scoped-edge"
expect 0 "nothing-else-broken passes the pin when it keeps the edge an edge line names" "$base" nothing-else-broken.sh --check-clean "$work/scoped-edge"
HW_ORACLE_ENGINE=$(explain_stub 2.0)
expect 2 "nothing-else-broken refuses an edge line read from an explain --json of major version 2" "$base" nothing-else-broken.sh "$work/scoped-edge"
HW_ORACLE_ENGINE=$engine
tree=$(fresh edge-deleted) && grep -v -x -F -- "    $edge_front" "$base/$new" > "$tree/$new"
expect 0 "nothing-else-broken with no edge line passes a tree that deleted the edge" "$tree" nothing-else-broken.sh "$work/scoped"
expect 1 "nothing-else-broken with an edge line fails the same tree" "$tree" nothing-else-broken.sh "$work/scoped-edge"
tree=$(fresh edge-body-line) && grep -v -x -F -- "    $edge_front" "$base/$new" > "$tree/$new" && printf '\nthe record is governs: .headwater/probe.yml in the notes.\n' >> "$tree/$new"
expect 1 "nothing-else-broken with an edge line fails the edge deleted and its text written in the body" "$tree" nothing-else-broken.sh "$work/scoped-edge"
tree=$(fresh edge-commented) && sed "s|^    $edge_front\$|    # $edge_front|" "$base/$new" > "$tree/$new"
grep -q -x -F -- "    # $edge_front" "$tree/$new" || echo "oracle-fixtures: the edge was not commented, so the next case reads nothing." >&2
expect 1 "nothing-else-broken with an edge line fails an edge that only a comment holds" "$tree" nothing-else-broken.sh "$work/scoped-edge"
tree=$(fresh edge-other-key) && grep -v -x -F -- "    $edge_front" "$base/$new" | sed "s|^  traces_to:\$|&\n    $edge_front|" > "$tree/$new"
grep -q -x -F -- "    $edge_front" "$tree/$new" || echo "oracle-fixtures: the edge was not moved under traces_to, so the next case reads nothing." >&2
expect 1 "nothing-else-broken with an edge line fails the edge moved under another relation" "$tree" nothing-else-broken.sh "$work/scoped-edge"
{ cat "$work/scoped"; printf '# edge: %s governs\n' "$new"; } > "$work/scoped-edge-bad"
expect 2 "nothing-else-broken refuses an edge line with no target" "$base" nothing-else-broken.sh "$work/scoped-edge-bad"
# The far end of an edge is not a declaration of the path. HW-DR-0112
# supersedes HW-DR-0111, so explain lists `superseded_by` on HW-DR-0111 as an
# inbound edge from HW-DR-0112, and an edge line reads outbound edges alone.
{ cat "$work/scoped"; printf '# edge: %s superseded_by %s\n' "$old" "$new"; } > "$work/scoped-edge-inbound"
expect 1 "nothing-else-broken with an edge line fails an edge the path does not declare and its far end does" "$base" nothing-else-broken.sh "$work/scoped-edge-inbound"
summary_clause='Phase B reports held-out numbers per action.'
grep -q -F -- "$summary_clause" "$base/$new" || echo "oracle-fixtures: the summary of $new does not hold \`$summary_clause\`, so the summary cases read nothing." >&2
{ cat "$work/scoped"; printf '# summary: %s %s\n' "$new" "$summary_clause"; } > "$work/scoped-summary"
expect 0 "nothing-else-broken passes the pin when the summary holds what a summary line names" "$base" nothing-else-broken.sh --check-clean "$work/scoped-summary"
tree=$(fresh summary-body-line) && sed "s| $summary_clause||" "$base/$new" > "$tree/$new" && printf '\n%s\n' "$summary_clause" >> "$tree/$new"
expect 1 "nothing-else-broken with a summary line fails the clause moved out of the summary into the body" "$tree" nothing-else-broken.sh "$work/scoped-summary"
tree=$(fresh summary-commented) && awk -v c=" $summary_clause" '/^summary: / { print "# " $0; sub(c, ""); print; next } { print }' "$base/$new" > "$tree/$new"
grep -q '^# summary: ' "$tree/$new" || echo "oracle-fixtures: the summary was not commented, so the next case reads nothing." >&2
expect 1 "nothing-else-broken with a summary line fails a summary that only a comment holds, above a shorter one" "$tree" nothing-else-broken.sh "$work/scoped-summary"
tree=$(fresh summary-other-key) && sed "s| $summary_clause||; s|^title: \"\\(.*\\)\"\$|title: \"\\1. $summary_clause\"|" "$base/$new" > "$tree/$new"
grep -q "^title: .*$summary_clause" "$tree/$new" || echo "oracle-fixtures: the clause was not moved into the title, so the next case reads nothing." >&2
expect 1 "nothing-else-broken with a summary line fails the clause moved into another key" "$tree" nothing-else-broken.sh "$work/scoped-summary"
{ cat "$work/scoped"; printf '# summary: %s\n' "$new"; } > "$work/scoped-summary-bad"
expect 2 "nothing-else-broken refuses a summary line with no literal" "$base" nothing-else-broken.sh "$work/scoped-summary-bad"
{ cat "$work/scoped"; printf '# keep: %s %s\n' "$new" "$summary_clause"; } > "$work/scoped-keep"
expect 2 "nothing-else-broken refuses the keep line of round 3, which matched lines of a file" "$base" nothing-else-broken.sh "$work/scoped-keep"

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
