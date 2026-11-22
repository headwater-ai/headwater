# SPDX-License-Identifier: Apache-2.0
#
# The case helpers of `tools/probe/probe-record-fixtures.sh` and
# `tools/probe/campaign-fixtures.sh`, sourced by both, so the two suites
# count and report a case one way. Each suite sets `passed=0` and `failed=0`
# before it sources this file, and prints the two counts at its end.

pass() { printf 'ok   %s\n' "$1"; passed=$((passed + 1)); }
fail() { printf 'FAIL %s\n  %s\n' "$1" "$2"; failed=$((failed + 1)); }
# Each helper takes exactly three arguments. Two calls joined on one line pass
# the second as extra arguments, and the second case would never run, so an
# extra argument is a failure of its own (verify of #1467).
arity() {
    [ "$2" = 3 ] && return 0
    fail "$1" "the case helper received $2 arguments, not 3: two cases joined on one line?"
    return 1
}
same() {
    arity "$1" $# || return 0
    if [ "$2" = "$3" ]; then pass "$1"; else fail "$1" "expected \`$2\`, got \`$3\`"; fi
}
absent() {
    # $1 name, $2 needle, $3 file
    arity "$1" $# || return 0
    if grep -qF -- "$2" "$3"; then
        fail "$1" "the output holds \`$2\`, which came from a block the filter must drop"
    else
        pass "$1"
    fi
}
present() {
    arity "$1" $# || return 0
    if grep -qF -- "$2" "$3"; then pass "$1"; else fail "$1" "the output does not hold \`$2\`"; fi
}
