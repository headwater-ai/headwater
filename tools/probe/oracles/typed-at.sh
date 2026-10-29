#!/bin/sh
# Generic oracle: a document is typed at the right path and cites its ruling
# (#1874). Row 4 of the grid in HW-DR-0112, "add a document".
#
#     sh tools/probe/oracles/typed-at.sh [--check-clean] <dir> <kind> <ruling> <title-pattern>
#
# It passes when one Markdown file directly under <dir> holds all of these:
#
# 1. a `title:` line in its front matter that matches <title-pattern>, an
#    extended regular expression read without regard to case;
# 2. the kind <kind>, as the `kind` line of `headwater explain` states it;
# 3. the identifier <ruling> as a whole name somewhere in its bytes, in a
#    relation or a link, so that `HW-DR-01120` does not stand for `HW-DR-0112`.
#
# The grading copy holds no history, so nothing here can ask which file is
# new. The title pattern is what selects the document the session wrote. A
# probe chooses a pattern that no document under <dir> matches at the pin,
# and `tools/probe/oracle-fixtures.sh` holds the case where the pattern
# matches a document of the wrong kind or one that cites no ruling.

set -u
. "${0%/*}/lib.sh"
[ $# -eq 4 ] || hw_oracle_usage "typed-at.sh [--check-clean] <dir> <kind> <ruling> <title-pattern>"
dir=${1%/} kind=$2 ruling=$3 pattern=$4
[ -d "$dir" ] || hw_oracle_fail "$dir is not a directory"
hw_oracle_engine

titled=0 seen=""
for file in "$dir"/*.md; do
    [ -f "$file" ] || continue
    hw_oracle_front_matter "$file" | grep -E -i -q "^title:.*($pattern)" || continue
    titled=$((titled + 1))
    typed=$("$hw_engine" explain --root . "$file" 2> /dev/null | sed -n 's/^  kind \([^ ]*\).*/\1/p' | sed -n '1p')
    if [ "$typed" != "$kind" ]; then
        seen="$file is typed ${typed:-as nothing}"
        continue
    fi
    if ! hw_oracle_names_whole "$ruling" "$file"; then
        seen="$file does not cite $ruling"
        continue
    fi
    hw_oracle_pass_clean "$file is typed $kind under $dir and cites $ruling"
done
[ "$titled" -gt 0 ] || hw_oracle_fail "no document under $dir has a title that matches the pattern"
hw_oracle_fail "$seen"
