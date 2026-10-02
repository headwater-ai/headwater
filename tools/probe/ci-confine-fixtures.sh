#!/bin/sh
# What holds `tools/probe/ci-confine.sh`, the gate CI runs before the
# recorder's fixtures (#1467).
#
# The gate failed once already in the one way that turned `main` red: it
# called `sudo` on a runner that has none (PR #1546). The other failure it
# exists to prevent is silent: a runner that cannot confine a session, where
# the cases neither run nor say so. So each case below runs the gate on a
# `PATH` built from this host's tools with `bwrap` and `sudo` taken out, adds
# stubs for the two, and states the exit status, the line the gate must
# print, and what must not happen: the suite running where it must not, or a
# command that is not there being called. The suite itself is a stub that
# prints a marker and exits with the status a case asks for.
#
#     sh tools/probe/ci-confine-fixtures.sh
#
# It needs `sh` and the usual tools, and writes only under a temporary
# directory.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
gate="$root/tools/probe/ci-confine.sh"

w=$(mktemp -d) || exit 1
trap 'rm -rf "$w"' EXIT HUP INT TERM

passed=0
failed=0

# The host's tools, with `bwrap` and `sudo` taken out.
mkdir -p "$w/base"
for dir in /usr/local/bin /usr/bin /bin /usr/local/sbin /usr/sbin /sbin; do
    [ -d "$dir" ] || continue
    for tool in "$dir"/*; do
        name=${tool##*/}
        case $name in bwrap|sudo) continue ;; esac
        [ -e "$w/base/$name" ] || [ -L "$w/base/$name" ] || ln -s "$tool" "$w/base/$name" 2>/dev/null
    done
done

printf '#!/bin/sh\necho SUITE-RAN\nexit "${SUITE_EXIT:-0}"\n' > "$w/suite.sh"

# A `bwrap` that confines, one that cannot make a namespace, and one that
# cannot until the AppArmor restriction is lifted (the flag file).
mkdir -p "$w/good" "$w/bad" "$w/flag" "$w/sudo" "$w/installed"
printf '#!/bin/sh\nexit 0\n' > "$w/good/bwrap"
printf '#!/bin/sh\necho "bwrap: setting up uid map: Permission denied" >&2\nexit 1\n' > "$w/bad/bwrap"
cat > "$w/flag/bwrap" <<EOF
#!/bin/sh
[ -e "$w/userns-allowed" ] && exit 0
echo "bwrap: No permissions to create new namespace" >&2
exit 1
EOF
# A `sudo` that logs every call. `apt-get update` exits APT_UPDATE_EXIT,
# `apt-get install` puts a confining `bwrap` on the path when APT_INSTALL is
# `ok`, the flag `bwrap` when it is `flag` (ubuntu-latest: the package
# installs, and AppArmor still refuses the namespace), and fails otherwise.
# `sysctl` lifts the restriction when SYSCTL_LIFTS is `yes`.
cat > "$w/sudo/sudo" <<EOF
#!/bin/sh
echo "\$*" >> "$w/sudo.log"
case "\$*" in
    *"apt-get update"*) exit "\${APT_UPDATE_EXIT:-0}" ;;
    *"apt-get install"*)
        case "\${APT_INSTALL:-ok}" in
            ok) printf '#!/bin/sh\nexit 0\n' > "$w/installed/bwrap" ;;
            flag) cp "$w/flag/bwrap" "$w/installed/bwrap" ;;
            *) exit 100 ;;
        esac
        chmod +x "$w/installed/bwrap" ;;
    *sysctl*) [ "\${SYSCTL_LIFTS:-no}" = yes ] && : > "$w/userns-allowed" ;;
esac
exit 0
EOF
chmod +x "$w"/good/bwrap "$w"/bad/bwrap "$w"/flag/bwrap "$w"/sudo/sudo

# run NAME KIND EXTRA-PATH WANT-STATUS WANT-LINE SUITE(ran|not) [VAR=value...]
# Every case runs as under CI (`GITHUB_ACTIONS=true`) unless it sets
# `GITHUB_ACTIONS=` itself. A case that sets NO_SUDO=yes also fails when
# `sudo` received any call. The marker `GITHUB_ACTIONS=unset` runs the case
# with the variable absent from the environment, which a contributor's shell
# is, and which `GITHUB_ACTIONS=` is not: `env` cannot unset a variable after
# it has set one, so the marker selects `env -u` in place of the assignment.
run() {
    name=$1 kind=$2 extra=$3 want=$4 line=$5 ran=$6
    shift 6
    unset_ga= n=$#
    while [ "$n" -gt 0 ]; do
        arg=$1; shift; n=$((n - 1))
        if [ "$arg" = GITHUB_ACTIONS=unset ]; then unset_ga=yes; else set -- "$@" "$arg"; fi
    done
    rm -f "$w/sudo.log" "$w/installed/bwrap" "$w/userns-allowed"
    if [ -n "$unset_ga" ]; then
        out=$(env -u GITHUB_ACTIONS NO_SUDO= "$@" RUNNER_KIND="$kind" RUNNER_NAME=fixture-runner HW_RECORDER_SUITE="$w/suite.sh" \
            PATH="$extra$w/installed:$w/base" sh "$gate" 2>&1)
    else
        out=$(env GITHUB_ACTIONS=true NO_SUDO= "$@" RUNNER_KIND="$kind" RUNNER_NAME=fixture-runner HW_RECORDER_SUITE="$w/suite.sh" \
            PATH="$extra$w/installed:$w/base" sh "$gate" 2>&1)
    fi
    status=$?
    why=
    [ "$status" = "$want" ] || why="exit $status, wanted $want"
    printf '%s\n' "$out" | grep -qF -- "$line" || why="${why:+$why; }no line holding \`$line\`"
    if [ "$ran" = ran ]; then
        printf '%s\n' "$out" | grep -q SUITE-RAN || why="${why:+$why; }the suite did not run"
    else
        ! printf '%s\n' "$out" | grep -q SUITE-RAN || why="${why:+$why; }the suite ran"
    fi
    ! printf '%s\n' "$out" | grep -q 'not found' || why="${why:+$why; }a command that is not there was called"
    case " $* " in
        *" NO_SUDO=yes "*) [ ! -e "$w/sudo.log" ] || why="${why:+$why; }sudo was called: $(tr '\n' ';' < "$w/sudo.log")" ;;
    esac
    if [ -z "$why" ]; then
        printf 'ok   %s\n' "$name"; passed=$((passed + 1))
    else
        printf 'FAIL %s\n  %s\n' "$name" "$why"
        printf '%s\n' "$out" | sed 's/^/  | /'
        failed=$((failed + 1))
    fi
}

skip="Recorder fixtures skipped::The self-hosted runner fixture-runner has"
cannot="Recorder fixtures cannot run::"

# The runner of #1546's red main: no `bwrap`, no `sudo`.
run "a self-hosted runner with no bwrap and no sudo skips with one line" self-hosted "" 0 "$skip no \`bwrap\` and no \`sudo\` to install it" not
run "and the line names the hosted job that runs the cases" self-hosted "" 0 "\`Recorder confinement (hosted)\` of this run runs them" not
run "a hosted runner with no bwrap and no sudo fails" github-hosted "" 1 "$cannot" not
run "a runner whose kind is unknown and cannot confine fails, not skips" "" "" 1 "$cannot" not
run "a self-hosted runner that confines runs the cases" self-hosted "$w/good:" 0 SUITE-RAN ran
run "and a failing suite fails the step there" self-hosted "$w/good:" 2 SUITE-RAN ran SUITE_EXIT=2
run "a hosted runner that confines runs the cases, and their failure is the step's" github-hosted "$w/good:" 1 SUITE-RAN ran SUITE_EXIT=1
run "a suite with no jq is a warning" github-hosted "$w/good:" 0 "No jq on this runner" ran SUITE_EXIT=3
run "a self-hosted bwrap with no namespace skips and says what bwrap said" self-hosted "$w/bad:" 0 "cannot create a user namespace (bwrap: setting up uid map: Permission denied)" not
run "a hosted bwrap with no namespace fails after sudo cannot lift it" github-hosted "$w/bad:$w/sudo:" 1 "$cannot" not
run "a namespace that sudo sysctl allows runs the cases" github-hosted "$w/flag:$w/sudo:" 0 SUITE-RAN ran SYSCTL_LIFTS=yes
run "on a self-hosted runner too" self-hosted "$w/flag:$w/sudo:" 0 SUITE-RAN ran SYSCTL_LIFTS=yes
run "a hosted runner with sudo and no bwrap installs it and runs the cases" github-hosted "$w/sudo:" 0 SUITE-RAN ran
run "an apt-get update that fails does not stop the install" github-hosted "$w/sudo:" 0 SUITE-RAN ran APT_UPDATE_EXIT=100
run "an install that fails is named as the reason, not a missing sudo" self-hosted "$w/sudo:" 0 "$skip no \`bwrap\`, and \`sudo apt-get install bubblewrap\` failed" not APT_INSTALL=failed
run "a runner of a kind that is neither hosted nor self-hosted fails, not skips" other "" 1 "$cannot" not
run "a runner that installs a bwrap AppArmor still refuses lifts it and runs the cases" github-hosted "$w/sudo:" 0 SUITE-RAN ran APT_INSTALL=flag SYSCTL_LIFTS=yes

# A contributor's machine, which DEVELOPING.md tells to run this script: no
# CI, so `sudo` is never called, whatever the host has.
run "outside CI, a refused namespace gets no sudo call and fails" "" "$w/flag:$w/sudo:" 1 "outside CI this script does not lift the AppArmor restriction" not GITHUB_ACTIONS= SYSCTL_LIFTS=yes NO_SUDO=yes
run "outside CI, a missing bwrap gets no sudo call and fails" "" "$w/sudo:" 1 "no \`bwrap\`, which this script installs only under CI" not GITHUB_ACTIONS= NO_SUDO=yes
run "outside CI, a bwrap that confines runs the cases with no sudo call" "" "$w/good:$w/sudo:" 0 SUITE-RAN ran GITHUB_ACTIONS= NO_SUDO=yes
run "and GITHUB_ACTIONS must be true, not merely set" "" "$w/flag:$w/sudo:" 1 "$cannot" not GITHUB_ACTIONS=false SYSCTL_LIFTS=yes NO_SUDO=yes
# A contributor's shell has no `GITHUB_ACTIONS` at all, set or empty, so a gate
# that read an absent variable as CI would call `sudo` there (#1467, clause 8).
run "outside CI with GITHUB_ACTIONS unset, a refused namespace gets no sudo call and fails" "" "$w/flag:$w/sudo:" 1 "outside CI this script does not lift the AppArmor restriction" not GITHUB_ACTIONS=unset SYSCTL_LIFTS=yes NO_SUDO=yes
run "outside CI with GITHUB_ACTIONS unset, a missing bwrap gets no sudo call and fails" "" "$w/sudo:" 1 "no \`bwrap\`, which this script installs only under CI" not GITHUB_ACTIONS=unset NO_SUDO=yes

# The calls `sudo` received: `sysctl` only where bwrap could not make a
# namespace, and nothing where `bwrap` already confines.
rm -f "$w/sudo.log"
PATH="$w/good:$w/sudo:$w/base" GITHUB_ACTIONS=true RUNNER_KIND=github-hosted HW_RECORDER_SUITE="$w/suite.sh" sh "$gate" >/dev/null 2>&1
if [ -e "$w/sudo.log" ]; then
    printf 'FAIL a runner whose bwrap confines gets no sudo call\n  it got: %s\n' "$(cat "$w/sudo.log")"; failed=$((failed + 1))
else
    printf 'ok   a runner whose bwrap confines gets no sudo call\n'; passed=$((passed + 1))
fi
rm -f "$w/sudo.log" "$w/userns-allowed"
PATH="$w/flag:$w/sudo:$w/base" GITHUB_ACTIONS=true RUNNER_KIND=github-hosted SYSCTL_LIFTS=yes HW_RECORDER_SUITE="$w/suite.sh" sh "$gate" >/dev/null 2>&1
if grep -qx 'sysctl -w kernel.apparmor_restrict_unprivileged_userns=0' "$w/sudo.log" 2>/dev/null; then
    printf 'ok   a refused namespace gets exactly the sysctl that lifts it\n'; passed=$((passed + 1))
else
    printf 'FAIL a refused namespace gets exactly the sysctl that lifts it\n  sudo got: %s\n' "$(cat "$w/sudo.log" 2>/dev/null)"; failed=$((failed + 1))
fi

printf '%d passed, %d failed\n' "$passed" "$failed"
[ "$failed" = 0 ]
