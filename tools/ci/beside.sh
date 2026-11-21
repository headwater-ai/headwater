#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# Run one suite of a CI job beside the steps that follow, and read its verdict
# in a later step of the same job.
#
#     sh tools/ci/beside.sh start <name> <command> [<argument>...]
#     sh tools/ci/beside.sh wait <name>
#
# Why it exists. The `headwater` job of `.github/workflows/ci.yml` gates every
# merge, and it ran each of its suites one after another on a runner with idle
# cores. Five of them took 566 of its 1,164 seconds (medians over 55
# self-hosted runs, 2026-11-02 to 2026-11-19), and none of the five writes into
# the checkout. `start` puts one of them in the background, and `wait` is the
# step that carries its name and its verdict. The suite, its strictness and its
# job stay the same. Only the clock time moves.
#
# `start` returns at once. The command runs with standard input closed, and
# its two output streams go to one log file, so the step that started it can
# end while it runs. A name is used once in a job: a second `start` under a
# name exits 2 and starts nothing.
#
# `wait` blocks until the command ends, prints its log, prints one line with
# the exit status and the seconds the command ran, and exits with the status of
# the command. Three cases never read as a pass. A name that nothing started
# exits 2. A command that ended and left no status, because something killed
# the shell that held it, exits 1. A command that is still running keeps the
# step waiting, as a step that ran it in the foreground would.
#
# The state is three files for each name under `$HW_BESIDE_DIR`, or under
# `$RUNNER_TEMP/hw-beside` when that variable is unset. With neither variable
# set the script exits 2, because a default under `/tmp` would let two jobs on
# one host read the verdict of each other.
#
# What may run under it: a command that writes nothing into the checkout, that
# reads no file a step between its `start` and its `wait` writes, and that
# holds no deadline a busy runner would break. The comment on each `start` in
# `ci.yml` says why its command passes those three conditions.
#
# `tools/ci/beside-fixtures.sh` holds each sentence above.

set -u

usage() {
    echo "usage: sh tools/ci/beside.sh start <name> <command> [<argument>...]" >&2
    echo "       sh tools/ci/beside.sh wait <name>" >&2
    exit 2
}

[ $# -ge 2 ] || usage
verb=$1
name=$2
shift 2

case $name in
    '' | *[!a-z0-9-]*)
        echo "beside: the name '$name' has a character outside a-z, 0-9 and '-'" >&2
        exit 2
        ;;
esac

if [ -n "${HW_BESIDE_DIR:-}" ]; then
    dir=$HW_BESIDE_DIR
elif [ -n "${RUNNER_TEMP:-}" ]; then
    dir=$RUNNER_TEMP/hw-beside
else
    echo "beside: neither HW_BESIDE_DIR nor RUNNER_TEMP is set, so there is no directory private to this job" >&2
    exit 2
fi

case $verb in
    start)
        [ $# -ge 1 ] || usage
        mkdir -p "$dir" || exit 2
        # `set -C` makes the redirection fail when the file exists, so two
        # starts under one name cannot both pass this line.
        if ! (set -C && : > "$dir/$name.pid") 2>/dev/null; then
            echo "beside: '$name' was already started in this job" >&2
            exit 2
        fi
        (
            trap '' HUP
            began=$(date +%s)
            status=0
            "$@" > "$dir/$name.log" 2>&1 || status=$?
            echo $(($(date +%s) - began)) > "$dir/$name.seconds"
            # Written under another name and renamed, so `wait` never reads a
            # status file that is half written.
            echo "$status" > "$dir/$name.status.part"
            mv "$dir/$name.status.part" "$dir/$name.status"
        ) < /dev/null > /dev/null 2>&1 &
        echo $! > "$dir/$name.pid"
        echo "beside: started '$name' as process $!: $*"
        ;;
    wait)
        [ $# -eq 0 ] || usage
        if [ ! -s "$dir/$name.pid" ]; then
            echo "beside: nothing named '$name' was started in this job, so there is no verdict to read" >&2
            exit 2
        fi
        pid=$(cat "$dir/$name.pid")
        while [ ! -f "$dir/$name.status" ]; do
            if ! kill -0 "$pid" 2>/dev/null; then
                # The shell can end between the two tests above, so look once
                # more before calling the status missing.
                sleep 1
                [ -f "$dir/$name.status" ] && break
                [ -f "$dir/$name.log" ] && cat "$dir/$name.log"
                echo "beside: '$name' ended and left no exit status, so this step fails" >&2
                exit 1
            fi
            sleep 1
        done
        status=$(cat "$dir/$name.status")
        [ -f "$dir/$name.log" ] && cat "$dir/$name.log"
        echo "beside: '$name' exited $status after $(cat "$dir/$name.seconds" 2>/dev/null || echo '?') seconds"
        case $status in
            '' | *[!0-9]*)
                echo "beside: the status of '$name' is not a number, so this step fails" >&2
                exit 1
                ;;
        esac
        exit "$status"
        ;;
    *)
        usage
        ;;
esac
