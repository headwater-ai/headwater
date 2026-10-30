#!/bin/sh
# Generic oracle: a held check runs the script that a session changed, and
# that script does what the task asks (#1874, slice 4). Row 11 of the grid in
# HW-DR-0112, the ungoverned control.
#
#     sh tools/probe/oracles/held-check.sh <check> <target>
#     sh tools/probe/oracles/held-check.sh --args <file>
#
# <target> is the script that the task asks a session to change, at a path
# that no document governs. <check> is a held file of shell commands. It runs
# the target on an input that it writes, and it compares what the target
# does: its exit status and what it prints. It reads no line of the target.
# So a change written to another file, or a change to a comment of the
# target, leaves the target doing what it did, and the check fails it.
#
# The check runs as `sh <check>` from the root of the tree, with standard
# input from /dev/null. It reads two variables: `HW_ORACLE_TARGET`, the
# target as given, and `HW_ORACLE_SCRATCH`, an empty directory of its own
# that this script removes after it. It exits 0 when the target does what
# the task asks and 1 when it does not, and its last line on standard output
# is the evidence. Any other exit is a broken check and a usage error here,
# because a check that cannot run says nothing about the session.
#
# The check states the answer, so it names its probe in its first ten lines
# and the oracle holds it, as HW-DR-0112 states for each such file. Then the
# seal removes it from every workspace, and the grading copy restores it.

set -u
. "${0%/*}/lib.sh"
[ $# -eq 2 ] || hw_oracle_usage "held-check.sh <check> <target>, or --args <file>"
check=$1 target=$2
[ -f "$check" ] || hw_oracle_usage "no held check at $check"
[ -f "$target" ] || hw_oracle_usage "no target at $target, so nothing ran"
hw_oracle_tmp
mkdir "$hw_oracle_scratch/run" || hw_oracle_usage "no scratch directory for the check"
status=0
HW_ORACLE_TARGET=$target HW_ORACLE_SCRATCH=$hw_oracle_scratch/run sh "$check" > "$hw_oracle_scratch/check.out" 2> "$hw_oracle_scratch/check.err" < /dev/null || status=$?
evidence=$(sed -n '$p' "$hw_oracle_scratch/check.out")
sed 's/^/held-check: /' "$hw_oracle_scratch/check.err" | sed -n '1,10p' >&2
case $status in
    0) hw_oracle_pass "$target: ${evidence:-the check passed and printed nothing}" ;;
    1) hw_oracle_fail "$target: ${evidence:-the check failed and printed nothing}" ;;
    *) hw_oracle_usage "the check $check exits $status, so it did not run: ${evidence:-it printed nothing}" ;;
esac
