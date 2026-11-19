#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# Fixtures for `tools/ci/beside.sh`, which runs one suite of a CI job beside
# the steps that follow and gives its verdict to a later step.
#
#     sh tools/ci/beside-fixtures.sh
#
# A `wait` step is the only reader of a suite that `start` put in the
# background. So each case here is a way that step could go green without a
# passing suite, provoked on purpose: a suite that failed, a name that nothing
# started, a suite whose shell was killed, and a second start under one name.
# Two more cases hold what the tool is for. `start` returns while its command
# still runs, and two commands that each wait for the other both end, which
# they can do only when they run at the same time.
#
# It needs `sh`, `mktemp`, `kill` and `sleep`, and writes only under a
# temporary directory.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool=$root/tools/ci/beside.sh

passed=0
failed=0
pass() { printf 'ok   %s\n' "$1"; passed=$((passed + 1)); }
fail() { printf 'FAIL %s\n  %s\n' "$1" "$2"; failed=$((failed + 1)); }
same() {
    if [ "$2" = "$3" ]; then pass "$1"; else fail "$1" "wanted '$2', got '$3'"; fi
}
contains() {
    case $3 in
        *"$2"*) pass "$1" ;;
        *) fail "$1" "wanted a line with '$2', got: $3" ;;
    esac
}

scratch=$(mktemp -d "${TMPDIR:-/tmp}/beside-fixtures.XXXXXX") || exit 1
trap 'rm -rf "$scratch"' EXIT

# Each case gets its own state directory, so no case reads the names of
# another.
beside() {
    case_dir=$1
    shift
    HW_BESIDE_DIR=$scratch/$case_dir sh "$tool" "$@"
}

# 1. A command that passes: `wait` exits 0 and prints what the command printed
#    on both streams.
beside pass start green sh -c 'echo to-stdout; echo to-stderr >&2' > /dev/null
out=$(beside pass wait green 2>&1)
same "a command that exits 0 gives a wait that exits 0" 0 $?
contains "the wait prints the standard output of the command" "to-stdout" "$out"
contains "the wait prints the standard error of the command" "to-stderr" "$out"
contains "the wait prints the exit status and the seconds" "'green' exited 0 after" "$out"

# 2. A command that fails: `wait` exits with the same status, and still prints
#    the log, because the log is where the failure is named.
beside fail start red sh -c 'echo the-finding; exit 7' > /dev/null
out=$(beside fail wait red 2>&1)
same "a command that exits 7 gives a wait that exits 7" 7 $?
contains "the wait of a failed command prints its log" "the-finding" "$out"

# 3. A name that nothing started is not a pass.
out=$(beside missing wait never-started 2>&1)
same "a wait on a name that nothing started exits 2" 2 $?
contains "and says that nothing was started" "nothing named 'never-started' was started" "$out"

# 4. A second start under one name starts nothing.
beside twice start one sh -c 'exit 0' > /dev/null
out=$(beside twice start one sh -c 'echo second > "$0"' "$scratch/twice-ran" 2>&1)
same "a second start under one name exits 2" 2 $?
contains "and says the name is taken" "'one' was already started" "$out"
beside twice wait one > /dev/null 2>&1
sleep 1
if [ -e "$scratch/twice-ran" ]; then
    fail "the second start ran no command" "the refused command ran"
else
    pass "the second start ran no command"
fi

# 5. `start` returns while its command still runs. The command waits for a
#    file that this script writes only after `start` has returned.
beside early start gate sh -c 'until [ -e "$0" ]; do sleep 1; done; echo released' "$scratch/early-go" > /dev/null
same "start returns while its command still runs" 0 $?
if [ -e "$scratch/early/gate.status" ]; then
    fail "the command is still running after start returned" "it already left a status"
else
    pass "the command is still running after start returned"
fi
: > "$scratch/early-go"
out=$(beside early wait gate 2>&1)
same "the wait then reads its exit status" 0 $?
contains "and its output" "released" "$out"

# 6. Two commands that each wait for the other. Each one writes its own file
#    and then waits for the file of the other, so both end only when both run
#    at the same time. In sequence the first would wait forever, and its loop
#    gives up after 30 seconds with status 9.
meet='
mine=$1; theirs=$2
: > "$mine"
n=0
until [ -e "$theirs" ]; do
    n=$((n + 1)); [ "$n" -gt 30 ] && exit 9
    sleep 1
done
exit 0'
beside pair start left sh -c "$meet" meet "$scratch/pair-left" "$scratch/pair-right" > /dev/null
beside pair start right sh -c "$meet" meet "$scratch/pair-right" "$scratch/pair-left" > /dev/null
beside pair wait left > /dev/null 2>&1
same "the first of two commands that wait for each other ends with 0" 0 $?
beside pair wait right > /dev/null 2>&1
same "and so does the second" 0 $?

# 7. A command whose shell is killed leaves no status, and `wait` reads that
#    as a failure and never as a pass.
beside killed start victim sh -c 'sleep 5' > /dev/null
kill -9 "$(cat "$scratch/killed/victim.pid")"
out=$(beside killed wait victim 2>&1)
same "a command whose shell was killed gives a wait that exits 1" 1 $?
contains "and says that it left no exit status" "left no exit status" "$out"

# 8. A name with a character that would escape the state directory.
out=$(beside name start ../up true 2>&1)
same "a name with a slash exits 2" 2 $?

# 9. With no directory private to the job, the tool refuses.
out=$(env -u HW_BESIDE_DIR -u RUNNER_TEMP sh "$tool" start any true 2>&1)
same "with neither HW_BESIDE_DIR nor RUNNER_TEMP set, start exits 2" 2 $?
contains "and says which variables it reads" "neither HW_BESIDE_DIR nor RUNNER_TEMP" "$out"

echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
