# The additive campaign of milestone 0.07 (#1774): seven scenarios, each
# plain code with layers added, in three rounds.
#
#     sh tools/probe/campaign.sh --dry-run --spec tools/probe/additive-campaign.spec
#
# The plan is HW-DR-0112, and #1872 cut this spec to its seven scenarios by
# dropping the lines of scenarios 7 to 13. #1875 rewrites it whole. Each
# line is `additive scenario-<n> <round>`, and
# the round names the set it runs under `rounds:` in `.headwater/probe.yml`.
# The screening round and each tuning try run the tuning set, and the final
# round runs the held-out set, so no held-out result exists before the final
# round. `the_additive_spec_plans_each_scenario_in_each_round` in
# `engine/crates/probe/tests/corpus.rs` holds that.
#
# The dry run computes each round's repetitions from the round's two rates,
# with the one formula it states, over the probes of the round's set, five
# in the tuning set and four in the held-out set: 14 for the screening round
# and 55 for the final round. A tuning line instead runs the counts the owner
# ruled, rounded up over the five probes: 5 for a screened variant and 29
# for a confirmation, by the stage in `HEADWATER_TUNING_STAGE` (#1838).
# A line that leaves a probe out runs the others at the round's count, so a
# probe on a line of its own runs as many sessions as a pooled one.
#
# **What it costs.** At the declared 50 cents a session:
#
#     round screening: 490 sessions, $245.00
#     round tuning: 10920 sessions, $5460.00 (4 rounds at most)
#     round final: 1540 sessions, $770.00
#     total: 12950 sessions, $6475.00
#
# At the 35.0 cents a session the campaign of 2026-10-03 realized, the total
# is $4,532.50. A tuning line is priced as up to 4 rounds of 390 sessions
# each: 4 variants of 25, a confirmation of 145 and a best so far of 145
# (#1838). That sum bounds every tuning line, and only the skills, the hook
# and MCP are tuned, under a ceiling of $1,290.00 for the tries and of
# $220.00 for the bases: the first base and the first best so far of each
# layer.
#
# **The owner approved the screening and tuning rounds, and not the final
# round.** On 2026-10-18 he approved $700 over every attempt of the screening
# round (#1775). On 2026-10-19 he approved $1,512.00 over every attempt and
# every layer of the tuning round (#1777): one layer at a time, with each
# layer's plan accepted before any spend on it. On 2026-10-22 he ruled that
# only the skills, the hook and the MCP layer are tuned, each against
# scenario 3, and that the documents, the engine and `CLAUDE.md` are frozen
# (#1833). HW-DR-0111 records the three rulings. The final round is not
# approved, so a batch of this spec is refused with 5 before it builds a tree.
# The screening round runs from `tools/probe/additive-screening.spec`, which
# holds its lines alone, and a tuning try runs from a spec of its own while
# `rounds.tuning.open` in `.headwater/probe.yml` opens that layer (#1819).

# The screening round.
additive scenario-0 screening
additive scenario-1 screening
additive scenario-2 screening
additive scenario-3 screening
additive scenario-4 screening
additive scenario-5 screening
additive scenario-6 screening

# The tuning round.
additive scenario-0 tuning
additive scenario-1 tuning
additive scenario-2 tuning
additive scenario-3 tuning
additive scenario-4 tuning
additive scenario-5 tuning
additive scenario-6 tuning

# The final round.
additive scenario-0 final
additive scenario-1 final
additive scenario-2 final
additive scenario-3 final
additive scenario-4 final
additive scenario-5 final
additive scenario-6 final
