# The cases of the `r11-asset-limit` oracle, for the probe
# HW-PROBE-a-session-lets-the-asset-size-check-of-the-site-take-its-limit-in-mebibytes-as-a-second-argument
# (#1874).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree. This file states the
# answer, so it names its probe here and the seal removes it from every
# workspace. The changes are those the probe's Key records under "The
# reference change and the wrong change, checked".

r11_target=tools/site/check-site-asset-size.sh

# Admit a second argument, and set the limit from it after the variable.
# <multiplier> is the size of the unit in bytes.
second_argument() {
    sed -i 's/^if \[ \$# -ne 1 \]; then$/if [ $# -ne 1 ] \&\& [ $# -ne 2 ]; then/' "$r11_target" || return 1
    awk -v unit="$1" '
        { print }
        /^limit_bytes=/ && !done {
            print "if [ $# -eq 2 ]; then"
            print "    case \"$2\" in"
            print "        '"''"'|*[!0-9]*)"
            print "            echo \"the asset limit argument must be a whole number of MiB.\" >&2"
            print "            exit 2"
            print "            ;;"
            print "    esac"
            print "    limit_bytes=$(($2 * " unit "))"
            print "fi"
            done = 1
        }
    ' "$r11_target" > "$r11_target.new" && mv "$r11_target.new" "$r11_target" && grep -q '^if \[ \$# -eq 2 \]; then$' "$r11_target"
}

# The sound change: the argument in MiB, 1048576 bytes.
reference() {
    second_argument 1048576
}

# The argument read as a count of bytes, as the variable is.
wrong() {
    second_argument 1
}

# The argument read in MB of 1000000 bytes, so a file of exactly 2 MiB is
# over a limit of 2.
decimal() {
    second_argument 1000000
}

# The change written to a new script beside the target, which turns the
# argument into the variable and calls the target. The target is as it was.
elsewhere() {
    printf '%s\n' '#!/bin/sh' '[ $# -eq 2 ] && export HEADWATER_SITE_ASSET_LIMIT_BYTES=$(($2 * 1048576))' 'exec sh "${0%/*}/check-site-asset-size.sh" "$1"' > tools/site/check-site-asset-size-mib.sh
}

# The usage line names the second argument, and no code reads it.
commented() {
    sed -i 's/^    echo "usage: tools\/site\/check-site-asset-size.sh SITE-ROOT" >&2$/    echo "usage: tools\/site\/check-site-asset-size.sh SITE-ROOT [LIMIT-MIB]" >\&2/' "$r11_target" && grep -q 'LIMIT-MIB' "$r11_target"
}
more_wrong="decimal elsewhere commented"
