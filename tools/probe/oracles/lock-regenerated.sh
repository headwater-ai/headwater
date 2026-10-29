#!/bin/sh
# Generic oracle: the taxonomy lock is what a resolve writes (#1874). Row 8 of
# the grid in HW-DR-0112, "change taxonomy or config".
#
#     sh tools/probe/oracles/lock-regenerated.sh [--check-clean] [<file> <literal>]...
#
# It passes when `headwater taxonomy resolve --check` exits 0: the committed
# `.headwater/taxonomy.lock` is what a run over the taxonomy sources
# produces, so a session that edited the overlay or a package also resolved.
# It writes nothing. A probe of row 8 passes `--check-clean`, because the grid
# grades that row on both conditions.
#
# On a tree that nobody changed the lock already matches its sources, so the
# bare form passes the pin, and a probe cannot fail a session that did
# nothing (#1874.3). So a probe names what its change must hold: each
# `<file> <literal>` pair fails the oracle unless <file> holds <literal> as a
# whole name, as `hw_oracle_names_whole` in `lib.sh` reads one. The pairs come
# from a held file through `--args`, because a literal states the answer.
# A literal that only a comment holds does not count, because the engine
# reads no comment: `hw_oracle_uncommented` in `lib.sh` says what a comment
# is in a YAML file and in a Markdown file.
# With `--check-clean` and at least one pair, the strict errors counted are
# those on the files the pairs name.

set -u
. "${0%/*}/lib.sh"
[ $(($# % 2)) -eq 0 ] || hw_oracle_usage "lock-regenerated.sh [--check-clean] [<file> <literal>]..."
hw_oracle_engine
hw_oracle_tmp
held=""
while [ $# -gt 0 ]; do
    [ -f "$1" ] || hw_oracle_usage "no file at $1, which a literal is read from"
    # A comment is not a declaration, so a literal that only a comment holds
    # does not count (#1874.3).
    hw_oracle_uncommented "$1" "$hw_oracle_scratch/literal-source"
    hw_oracle_names_whole "$2" "$hw_oracle_scratch/literal-source" || hw_oracle_fail "$1 does not hold \`$2\` as a whole name outside a comment"
    case " $hw_oracle_subjects " in
        *" $1 "*) ;;
        *) hw_oracle_subjects="${hw_oracle_subjects:+$hw_oracle_subjects }$1" ;;
    esac
    held="${held:+$held, }\`$2\` in $1"
    shift 2
done
status=0
"$hw_engine" taxonomy resolve --check --root . > "$hw_oracle_scratch/resolve.out" 2> "$hw_oracle_scratch/resolve.err" || status=$?
[ "$status" -eq 0 ] || hw_oracle_fail "\`headwater taxonomy resolve --check\` exits $status, so the lock is not what the sources resolve to"
hw_oracle_pass_clean "\`headwater taxonomy resolve --check\` exits 0${held:+, and it holds $held}"
