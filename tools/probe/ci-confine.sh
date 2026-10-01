#!/bin/sh
# Decides whether this CI runner can run the recorder's fixtures, and runs them
# when it can (#1467).
#
#     RUNNER_KIND=<runner.environment> sh tools/probe/ci-confine.sh
#
# `tools/probe/probe-record.sh` confines every session to its workspace under
# `bwrap` and refuses a session it cannot confine, so the cases of
# `tools/probe/probe-record-fixtures.sh` need `bwrap` and a user namespace.
# This script gets them where it can: under CI (`GITHUB_ACTIONS=true`) and
# with `sudo`, it installs `bubblewrap` and lifts the AppArmor restriction on
# unprivileged user namespaces. It never calls `sudo` where there is none, and
# never outside CI, because on a contributor's machine both are host-wide
# changes nobody asked for (round 2 verify of #1550).
#
# Then it asks `bwrap` for a namespace and acts on the answer:
#
# - It can confine: it runs the suite, and the suite's status is the exit
#   status. Status 3 is the suite's "no `jq`", a warning and 0.
# - It cannot, on a self-hosted runner: one warning that names the runner,
#   what it lacks, and the hosted job that runs the cases instead. Exit 0.
# - It cannot, on any other runner, an unknown one included: one error, exit 1.
#
# The decision reads what the runner can do, not its name, so a self-hosted
# runner that can confine runs the cases. The self-hosted runner's container
# cannot: measured 2026-10-01 on `mediaserver3-slot1`, it has no `bwrap` and
# no `sudo`, and `unshare --user true` exits 1 with "Operation not
# permitted", because Docker's default seccomp profile refuses the namespace.
#
# `HW_RECORDER_SUITE` names the suite to run, for
# `tools/probe/ci-confine-fixtures.sh`, which holds every branch above.
# Exit status: 0 ran green or skipped, 1 cannot confine where it must, or the
# suite's own status.

root=$(cd "$(dirname "$0")/../.." && pwd)
suite=${HW_RECORDER_SUITE:-$root/tools/probe/probe-record-fixtures.sh}
runner="${RUNNER_KIND:-unknown} runner ${RUNNER_NAME:-unnamed}"

has() { command -v "$1" >/dev/null 2>&1; }
confines() { bwrap --unshare-user --ro-bind / / true 2>"$1"; }
may_sudo() { [ "${GITHUB_ACTIONS:-}" = true ] && has sudo; }

said=$(mktemp) || exit 1
install=
if ! has bwrap && may_sudo; then
    # `apt-get update` exits nonzero when any one source fails, even one this
    # step never reads, so only the install decides.
    sudo apt-get update -q || true
    sudo apt-get install -y -q bubblewrap || install=failed
fi
if has bwrap && ! confines "$said" && may_sudo; then
    sudo sysctl -w kernel.apparmor_restrict_unprivileged_userns=0
fi

missing=
if ! has bwrap; then
    if [ "$install" = failed ]; then
        missing="no \`bwrap\`, and \`sudo apt-get install bubblewrap\` failed"
    elif [ "${GITHUB_ACTIONS:-}" = true ]; then
        missing="no \`bwrap\` and no \`sudo\` to install it"
    else
        missing="no \`bwrap\`, which this script installs only under CI"
    fi
elif ! confines "$said"; then
    missing="a \`bwrap\` that cannot create a user namespace ($(tail -1 "$said"))"
    [ "${GITHUB_ACTIONS:-}" = true ] \
        || missing="$missing, and outside CI this script does not lift the AppArmor restriction"
fi
rm -f "$said"

if [ -n "$missing" ]; then
    if [ "${RUNNER_KIND:-}" = self-hosted ]; then
        echo "::warning title=Recorder fixtures skipped::The $runner has $missing, so none of these cases ran here. The job \`Recorder confinement (hosted)\` of this run runs them on ubuntu-latest."
        exit 0
    fi
    echo "::error title=Recorder fixtures cannot run::The $runner has $missing, and the recorder refuses a session it cannot confine (#1467)."
    exit 1
fi

status=0
sh "$suite" || status=$?
if [ "$status" = 3 ]; then
    echo "::warning title=Recorder fixtures skipped::No jq on this runner, so none of these cases ran."
    exit 0
fi
exit "$status"
