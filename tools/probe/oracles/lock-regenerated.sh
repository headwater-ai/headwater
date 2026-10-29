#!/bin/sh
# Generic oracle: the taxonomy lock is what a resolve writes (#1874). Row 8 of
# the grid in HW-DR-0112, "change taxonomy or config".
#
#     sh tools/probe/oracles/lock-regenerated.sh [--check-clean]
#
# It passes when `headwater taxonomy resolve --check` exits 0: the committed
# `.headwater/taxonomy.lock` is what a run over the taxonomy sources
# produces, so a session that edited the overlay or a package also resolved.
# It writes nothing. A probe of row 8 passes `--check-clean`, because the grid
# grades that row on both conditions.

set -u
. "${0%/*}/lib.sh"
[ $# -eq 0 ] || hw_oracle_usage "lock-regenerated.sh [--check-clean]"
hw_oracle_engine
hw_oracle_tmp
status=0
"$hw_engine" taxonomy resolve --check --root . > "$hw_oracle_scratch/resolve.out" 2> "$hw_oracle_scratch/resolve.err" || status=$?
[ "$status" -eq 0 ] || hw_oracle_fail "\`headwater taxonomy resolve --check\` exits $status, so the lock is not what the sources resolve to"
hw_oracle_pass_clean "\`headwater taxonomy resolve --check\` exits 0"
