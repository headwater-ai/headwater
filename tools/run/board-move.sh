#!/bin/sh
# Moves one issue's card on the headwater-ai/headwater project board.
#
# The board's Status field is Projects v2, which is GraphQL under `gh
# project`, not the REST `gh issue` surface the rest of this tree's tools
# use. The project id, the field id and each option's id are opaque strings
# nothing derives; they lived only as prose in the `hw-run-policy` skill,
# copied by hand into a `gh project item-edit` call on every claim a build
# agent made. A single mistyped id there does not error — it edits a
# different field, or fails silently on an id that no longer resolves — and
# the project's own README already says this Status field is the one thing
# here not derivable from the repository, so getting it right matters enough
# to fix in one place rather than retype correctly every time.
#
#     sh tools/run/board-move.sh <issue-number> todo|in-progress|done
#
# The card is found with `gh project item-add`, which returns the existing
# item when the issue is already on the project and adds it when it is not.
# That is one call where there were two ways to fail. The script first read
# `gh project item-list --limit 200`, which stops finding cards once the
# project holds more than 200, and it refused an issue with no card and told
# the caller to file it by hand. The project stopped adding new issues on its
# own after #995, so in run 20260923-0733 all three builders' claims were
# refused that way and each one carried on without a card.

set -u

project_id=PVT_kwDOEsVrw84BgGeU
field_id=PVTSSF_lADOEsVrw84BgGeUzhaUkwg
owner=headwater-ai

usage() {
    sed -n '/^#     sh tools\/run\/board-move.sh/p' "$0" | sed 's/^# *//' >&2
    exit 2
}

option_id() {
    case $1 in
        todo) echo f75ad846 ;;
        in-progress) echo 47fc9ee4 ;;
        done) echo 98236657 ;;
        *) echo "" ;;
    esac
}

[ $# -eq 2 ] || usage
n=$1 status=$2
case $n in '' | *[!0-9]*) echo "board-move: \`$n\` is not an issue number." >&2; exit 2 ;; esac
option=$(option_id "$status")
[ -n "$option" ] || { echo "board-move: status is one of todo, in-progress, done — got \`$status\`." >&2; exit 2; }

item_id=$(gh project item-add 1 --owner "$owner" \
    --url "https://github.com/headwater-ai/headwater/issues/$n" --format json --jq .id) || item_id=

if [ -z "$item_id" ]; then
    echo "board-move: #$n could not be put on the project board; gh gave no item id." >&2
    exit 1
fi

gh project item-edit --id "$item_id" --project-id "$project_id" --field-id "$field_id" --single-select-option-id "$option" >/dev/null
echo "board-move: #$n moved to $status"
