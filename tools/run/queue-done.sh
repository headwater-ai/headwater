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
#     queue-done: #<PR> closed
#     queue-done: #<PR> not queued: open, in no queue, and never removed from one
#
# The last line means the merge call did not take, so waiting longer will
# not change it.
#
# Ejected means the pull request is open and in no queue. The reason is the
# last `RemovedFromMergeQueueEvent` on its timeline, which names the check
# that failed on the group or the conflict the group met. An ejected pull
# request stays out: nothing here enqueues it again, because that is a new
# ruling and the parent's.
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

query='query($owner:String!,$name:String!,$n:Int!){repository(owner:$owner,name:$name){pullRequest(number:$n){state mergeCommit{oid} mergeQueueEntry{state} autoMergeRequest{enabledAt} timelineItems(itemTypes:[REMOVED_FROM_MERGE_QUEUE_EVENT],last:1){nodes{... on RemovedFromMergeQueueEvent{reason}}}}}}'

# One line of TSV: state, merge commit, queue entry state, auto-merge,
# last removal reason. A missing value is `-`, so no field is ever empty
# and a split never shifts a column.
row=$(gh api graphql -F owner='{owner}' -F name='{repo}' -F n="$pr" -f query="$query" \
    --jq '.data.repository.pullRequest | [.state, (.mergeCommit.oid // "-"), (.mergeQueueEntry.state // "-"), (if .autoMergeRequest then "auto" else "-" end), ((.timelineItems.nodes[0].reason // "-") | gsub("[\t\n]"; " "))] | @tsv') || {
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

state=$1 oid=$2 entry=$3 auto=$4 reason=$5

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

if [ "$entry" != "-" ]; then
    echo "queue-done: #$pr is in the queue, $entry" >&2
    exit 1
fi
if [ "$auto" != "-" ]; then
    echo "queue-done: #$pr waits on its own checks before the queue takes it" >&2
    exit 1
fi

if [ "$reason" = "-" ]; then
    echo "queue-done: #$pr not queued: open, in no queue, and never removed from one"
    exit 0
fi
echo "queue-done: #$pr ejected: $reason"
exit 0
