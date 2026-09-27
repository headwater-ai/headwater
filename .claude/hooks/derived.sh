#!/bin/sh
# The conflict-time hook for a derived artifact (#1053).
#
#   PostToolUse Bash   one advisory, or nothing. A merge, a rebase, a
#                      cherry-pick, a revert or a pull that stops on a conflict
#                      is a `Bash` call, so the session hears of a derived
#                      artifact in conflict on the call that made the conflict,
#                      whether or not it edits the file later. The advisory
#                      names each unmerged path that a producer writes, the row
#                      and the treatment of its record, and the command that
#                      rebuilds it on the merged tree.
#
# The edit moment is `write.sh`, on `PreToolUse` for `Write|Edit`. It gives the
# same account of the same path from the same reader, `hw_derived_report` in
# `lib.sh`, which reads `headwater derived`. That is an existing verb, and no
# hook introduces a verb (spec 5, "The hook contract").
#
# Cost: most `Bash` calls meet no merge. The state test comes first and calls
# no engine: one `git rev-parse --git-dir` and five `test -e` on the files git
# leaves in this worktree's own git dir. The engine runs only inside a stopped
# merge that has an unmerged path. Spec 16 C5 records both costs as measured.
#
# Once per state, not once per call. The advisory speaks once for one session,
# one `HEAD` and one set of unmerged members, and every later `Bash` call of
# the same merge is silent. The stamp is a file under
# `<git common dir>/headwater-session/<session id>/`, the directory
# `touch.sh` uses. A payload with no session id speaks on every call.
#
# Claude Code only. Codex and Copilot bind `write.sh` for the edit moment, and
# the name of their shell tool on a `PostToolUse` payload is not measured, so
# neither binds this position. Spec 16 C5 records that.
#
# It fails open, silently: no engine, no git, no payload it can read, and the
# call has already happened.

. "$(dirname "$0")/lib.sh"

input=$(cat)
event=$(hw_field "$input" hook_event_name) || exit 0
[ "$event" = PostToolUse ] || exit 0
hw_root=$(hw_resolve_root "$input")

state=$(hw_merge_state) || exit 0
unmerged=$(hw_unmerged_paths)
[ -n "$unmerged" ] || exit 0

members=
report=
# One path per line, and a path may hold a space.
set -f
IFS='
'
for rel in $unmerged; do
    derived=$(hw_derived_report "$rel") || continue
    heading=$(printf '%s\n' "$derived" | sed -n 1p)
    rebuild=$(printf '%s\n' "$derived" | sed -n 2p)
    members="$members $rel"
    if [ -n "$rebuild" ]; then
        report="$report
  \`$rel\`: $heading. Rebuild it with \`$rebuild\`."
    else
        report="$report
  \`$rel\`: $heading."
    fi
done
unset IFS
set +f
[ -n "$members" ] || exit 0

session=$(hw_field "$input" session_id) || session=
if [ -n "$session" ] && common=$(hw_common_dir); then
    head=$(git -C "$hw_root" rev-parse -q --verify HEAD 2>/dev/null) || head=none
    key=$(printf '%s %s%s' "$state" "$head" "$members" | cksum | sed 's/ .*//')
    dir="$common/headwater-session/$session"
    [ -e "$dir/derived-$key" ] && exit 0
    mkdir -p "$dir" 2>/dev/null && : > "$dir/derived-$key" 2>/dev/null
fi

advisory="Headwater: this $state stopped with a derived artifact in conflict. A producer writes each path below, so resolve the sources first, then run its command on the merged tree and stage what it writes. Do not resolve these by hand.
$report

This is advisory. Nothing here blocks the $state."
quoted=$(hw_quote "$advisory") || exit 0
printf '{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":%s}}\n' "$quoted"
exit 0
