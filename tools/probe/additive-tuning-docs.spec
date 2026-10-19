# One tuning try of the documents layer (#1777): the scenario-1 tuning line
# of `tools/probe/additive-campaign.spec`, and no line of another round.
# Scenario 1 adds the documents layer to scenario 0, so the try measures that
# layer against base 0.
#
#     sh tools/probe/campaign.sh --dry-run --spec tools/probe/additive-tuning-docs.spec
#
# The owner approved the tuning round on 2026-10-19, at $1,512.00 over every
# attempt and every layer, one layer at a time, with the plan of each layer
# accepted before any spend on it (HW-DR-0111). The final round is not
# approved, so the batch refuses its lines with 5. This file holds no such
# line, so the paid invocation names it.
#
# At the declared 50 cents a session:
#
#     line 1  additive scenario-1 tuning: 4 probes x 18 repetitions (round tuning, ceil(70 / 4)) = 72 sessions, $36.00 a try
#
# Keep this line the same as the scenario-1 tuning line of
# `tools/probe/additive-campaign.spec`. The fixtures in
# `tools/probe/probe-record-fixtures.sh` compare the two.

additive scenario-1 tuning
