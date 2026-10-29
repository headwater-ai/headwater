# The screening round of the additive campaign (#1775): the 7 screening
# lines of `tools/probe/additive-campaign.spec`, and no line of another round.
#
#     sh tools/probe/campaign.sh --dry-run --spec tools/probe/additive-screening.spec
#
# The owner approved the screening round at $700 over every attempt
# of the round (HW-DR-0111). The batch refuses a line of a round whose
# `approved_cents` under `rounds:` in `.headwater/probe.yml` is 0, with 5,
# before it builds a tree or asks the harness anything. So the paid
# invocation names this file, which holds no line that the batch refuses.
#
# At the declared 50 cents a session:
#
#     round screening: 490 sessions, $245.00
#
# Keep these lines the same as the screening lines of
# `tools/probe/additive-campaign.spec`. The fixtures in
# `tools/probe/probe-record-fixtures.sh` compare the two.

additive scenario-0 screening
additive scenario-1 screening
additive scenario-2 screening
additive scenario-3 screening
additive scenario-4 screening
additive scenario-5 screening
additive scenario-6 screening
