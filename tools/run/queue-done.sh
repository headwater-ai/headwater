#!/bin/sh
# The condition an integrator waits on after it hands a pull request to the
# merge queue: has the queue finished with it, and how.
#
#     sh tools/run/queue-done.sh <PR>
#     sh tools/run/wait-for.sh 'sh tools/run/queue-done.sh <PR>'
#
# It exits 1 while the pull request is still waiting: in the queue, or with
# auto-merge set so that the queue takes it once its own checks pass. It
# exits 0 once the queue is done with it, and prints one line that says how:
#
#     queue-done: #<PR> merged <sha>
#     queue-done: #<PR> ejected: <the reason GitHub recorded>
#     queue-done: #<PR> unmergeable: the queue will eject it with merge_conflict
#     queue-done: #<PR> closed
#     queue-done: #<PR> not queued: open, in no queue, and never removed from one
#
# The last line means the merge call did not take, so waiting longer will
# not change it.
#
# The queue state is read from the timeline: the last `AddedToMergeQueueEvent`
# or `RemovedFromMergeQueueEvent`, whichever came later. Four incidents set
# the rules that read it.
#
# Ejected means the pull request is open, in no queue, and its last queue
# event is a removal. The reason is that removal's, which names the check
# that failed on the group or the conflict the group met. A removal whose
# reason is `merged` is never an ejection: the queue records it about one
# second before the merge, and in that second #1353 printed `ejected:
# merged`. It exits 1 until the merge is recorded.
#
# A last event that is an add, with no entry in the queue, means the entry
# has cleared and the removal is not on the timeline yet. It exits 1. In
# that window #1412 printed `not queued`, because the script then read
# removals alone and could not tell a pull request never queued from one
# whose removal it could not see yet. `not queued` now needs no queue event
# at all.
#
# Unmergeable means the pull request is still in the queue and conflicts
# with the group ahead of it. The queue ejects it with `merge_conflict`
# about nine minutes later, which #1328 waited through three times. The
# answer is final at once, so it exits 0 then. An integrator reads it as an
# ejection with reason `merge_conflict (unmergeable)`.
#
# An ejected or unmergeable pull request stays out: nothing here enqueues it
# again, because that is a new ruling and the parent's.
#
# Before the `Protect main` ruleset carries a `merge_queue` rule, `gh pr
# merge --squash` merges at once, and this reports `merged` on its first
# call. So the one integrator procedure holds on both sides of that change.
#
# An error reaching GitHub exits 1 with the error on stderr, so a wait
# re-asks rather than reads a failed call as an ejection. It prints nothing
# on stdout until the answer is final, which is what `wait-for.sh` relays.

set -u

pr=${1:-}
case $pr in
    ''|*[!0-9]*)
        echo "usage: sh tools/run/queue-done.sh <PR number>" >&2
        exit 64
        ;;
esac

query='query($owner:String!,$name:String!,$n:Int!){repository(owner:$owner,name:$name){pullRequest(number:$n){state mergeCommit{oid} mergeQueueEntry{state} autoMergeRequest{enabledAt} timelineItems(itemTypes:[ADDED_TO_MERGE_QUEUE_EVENT,REMOVED_FROM_MERGE_QUEUE_EVENT],last:1){nodes{__typename ... on RemovedFromMergeQueueEvent{reason}}}}}}'

# One line of TSV: state, merge commit, queue entry state, auto-merge, and
# the last queue event on the timeline, packed into one field: `added`,
# `removed:<reason>`, or `-` when there is none. A missing value is `-`, so
# no field is ever empty and a split never shifts a column.
row=$(gh api graphql -F owner='{owner}' -F name='{repo}' -F n="$pr" -f query="$query" \
    --jq '.data.repository.pullRequest | [.state, (.mergeCommit.oid // "-"), (.mergeQueueEntry.state // "-"), (if .autoMergeRequest then "auto" else "-" end), (.timelineItems.nodes[-1] | if . == null then "-" elif .__typename == "AddedToMergeQueueEvent" then "added" elif .__typename == "RemovedFromMergeQueueEvent" then "removed:" + ((.reason // "-") | gsub("[\t\n]"; " ")) else .__typename end)] | @tsv') || {
    echo "queue-done: #$pr: the query failed; asking again next time" >&2
    exit 1
}

tab=$(printf '\t')
old_ifs=$IFS
IFS=$tab
# shellcheck disable=SC2086
set -- $row
IFS=$old_ifs

if [ "$#" -ne 5 ]; then
    echo "queue-done: #$pr: the answer read '$row', not five fields" >&2
    exit 1
fi

state=$1 oid=$2 entry=$3 auto=$4 event=$5

case $state in
    MERGED)
        echo "queue-done: #$pr merged $oid"
        exit 0
        ;;
    CLOSED)
        echo "queue-done: #$pr closed"
        exit 0
        ;;
    OPEN) ;;
    *)
        echo "queue-done: #$pr: state '$state' is not one this script knows" >&2
        exit 1
        ;;
esac

# An unmergeable entry is still in the queue, but it conflicts with the
# group ahead of it, and the queue removes it with `merge_conflict` about
# nine minutes later (#1328). The answer is final now, so say it now.
if [ "$entry" = "UNMERGEABLE" ]; then
    echo "queue-done: #$pr unmergeable: the queue will eject it with merge_conflict"
    exit 0
fi
if [ "$entry" != "-" ]; then
    echo "queue-done: #$pr is in the queue, $entry" >&2
    exit 1
fi
if [ "$auto" != "-" ]; then
    echo "queue-done: #$pr waits on its own checks before the queue takes it" >&2
    exit 1
fi

case $event in
    -)
        echo "queue-done: #$pr not queued: open, in no queue, and never removed from one"
        exit 0
        ;;
    added)
        # The entry has cleared and the removal event is not on the
        # timeline yet (#1412). Never `not queued`: it was queued.
        echo "queue-done: #$pr left the queue, and its removal is not visible yet" >&2
        exit 1
        ;;
    removed:*)
        reason=${event#removed:}
        ;;
    *)
        echo "queue-done: #$pr: the last queue event read '$event', not one this script knows" >&2
        exit 1
        ;;
esac

# The queue removes a pull request with reason `merged` one second before
# the merge is recorded (#1353). That removal is never an ejection.
case $(printf '%s' "$reason" | tr '[:upper:]' '[:lower:]') in
    merged)
        echo "queue-done: #$pr left the queue as merged; the merge is not recorded yet" >&2
        exit 1
        ;;
esac
echo "queue-done: #$pr ejected: $reason"
exit 0
