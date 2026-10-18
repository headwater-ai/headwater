# The screening round of the additive campaign (#1775): the 15 screening
# lines of `tools/probe/additive-campaign.spec`, and no line of another round.
#
#     sh tools/probe/campaign.sh --dry-run --spec tools/probe/additive-screening.spec
#
# The owner approved the screening round alone, at $700 over every attempt
# of the round (HW-DR-0111). The batch refuses a line of a round whose
# `approved_cents` under `rounds:` in `.headwater/probe.yml` is 0, with 5,
# before it builds a tree or asks the harness anything. So the paid
# invocation names this file, which holds no line that the batch refuses.
#
# At the declared 50 cents a session:
#
#     round screening: 1008 sessions, $504.00
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
additive scenario-7 screening
additive scenario-8 screening
additive scenario-9 screening
additive scenario-10 screening
additive scenario-11 screening
additive scenario-12 screening
additive scenario-13 screening HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded
additive scenario-13 screening HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor HW-PROBE-a-session-follows-two-citations-from-a-record-the-task-does-not-name
