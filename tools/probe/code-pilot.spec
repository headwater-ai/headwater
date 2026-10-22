# The pilot of the code set (#1835): each probe's rate in plain code and in
# plain code with the documents, before the code round depends on it.
#
#     sh tools/probe/campaign.sh --dry-run --spec tools/probe/code-pilot.spec
#
# The `code-pilot` round under `rounds:` in `.headwater/probe.yml` declares
# its count, 18 repetitions of each of the five probes, so 90 sessions in
# each scenario and 180 in all. The owner approved $130 for it on 2026-10-22
# (#1777). A probe outside 15 to 70 percent in scenario 0 is reworded and
# re-run in scenario 0 alone, at 18 sessions, under the same approval: a
# line of its own, `additive scenario-0 code-pilot` followed by the other
# four probes it leaves out. This spec names no probe, so it is not a fold.
additive scenario-0 code-pilot
additive scenario-1 code-pilot
