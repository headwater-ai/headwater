#!/bin/sh
# Generic oracle: the tree is clean under `headwater check --strict` (#1874).
#
#     sh tools/probe/oracles/check-clean.sh
#
# It passes when the strict check exits 0, which is when the tree carries no
# error. An advisory finding does not fail it: nothing-else-broken.sh is the
# oracle that reads advisories. Rows 3, 4, 5, 7, 8 and 9 of the grid in
# HW-DR-0112 name this condition, and the other scripts add it after their
# own when `--check-clean` is their first argument.

set -u
. "${0%/*}/lib.sh"
[ $# -eq 0 ] || hw_oracle_usage "check-clean.sh takes no argument"
status=0
hw_oracle_check_strict || status=$?
[ "$status" -eq 0 ] || hw_oracle_fail "\`headwater check --strict\` exits $status"
hw_oracle_pass "\`headwater check --strict\` exits 0"
