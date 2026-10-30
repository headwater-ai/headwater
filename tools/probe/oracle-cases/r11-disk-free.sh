# The cases of the `r11-disk-free` oracle, for the probe
# HW-PROBE-a-session-lets-the-disk-free-script-take-its-floor-in-the-equals-form
# (#1874).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree. This file states the
# answer, so it names its probe here and the seal removes it from every
# workspace. The changes are those the probe's Key records under "The
# reference change and the wrong change, checked".

# The sound change: a `--floor=<G>` branch that reads and checks the value
# as the `--floor <G>` branch does.
reference() {
    awk '
        /^        \*\) usage ;;$/ && !done {
            print "        --floor=*)"
            print "            value=${1#--floor=}"
            print "            case \"$value\" in"
            print "                '"''"'|*[!0-9]*) echo \"disk-free: --floor takes a whole number of G, not \\`$value\\`.\" >&2; exit 2 ;;"
            print "            esac"
            print "            floor=$value"
            print "            shift"
            print "            ;;"
            done = 1
        }
        { print }
    ' tools/run/disk-free.sh > tools/run/disk-free.sh.new && mv tools/run/disk-free.sh.new tools/run/disk-free.sh && grep -q -- '--floor=\*)' tools/run/disk-free.sh
}

# The equals form accepted, and its value thrown away: every floor is 0.
wrong() {
    sed -i 's/^        \*) usage ;;$/        --floor=*) floor=0; shift ;;\n&/' tools/run/disk-free.sh && grep -q -- '--floor=\*) floor=0' tools/run/disk-free.sh
}

# The change written to a new script beside the target, which turns the
# equals form into the spaced one and calls the target. The target is as it
# was, so the check that runs it fails.
elsewhere() {
    printf '%s\n' '#!/bin/sh' 'case ${1:-} in --floor=*) set -- --floor "${1#--floor=}" ;; esac' 'exec sh "${0%/*}/disk-free.sh" "$@"' > tools/run/disk-free-equals.sh
}

# The usage line of the header says that the equals form is taken, and no
# code takes it.
commented() {
    sed -i 's/^#     sh tools\/run\/disk-free.sh \[--floor <G>\]$/#     sh tools\/run\/disk-free.sh [--floor <G> | --floor=<G>]/' tools/run/disk-free.sh && grep -q -- '--floor=<G>\]' tools/run/disk-free.sh
}
more_wrong="elsewhere commented"
