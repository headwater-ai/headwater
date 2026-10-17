# The additive campaign of milestone 0.07 (#1774): fourteen scenarios, each
# plain code with layers added, in three rounds.
#
#     sh tools/probe/campaign.sh --dry-run --spec tools/probe/additive-campaign.spec
#
# The plan is HW-DR-0111. Each line is `additive scenario-<n> <round>`, and
# the round names the set it runs under `rounds:` in `.headwater/probe.yml`.
# The screening round and each tuning try run the tuning set, and the final
# round runs the held-out set, so no held-out result exists before the final
# round. `the_additive_spec_plans_each_scenario_in_each_round` in
# `engine/crates/probe/tests/corpus.rs` holds that.
#
# The dry run computes each round's repetitions from the round's two rates,
# with the one formula it states, over the four probes of the round's set:
# 18 for the screening round and each tuning try, and 55 for the final round.
# A line that leaves a probe out runs the others at the round's count, so a
# probe on a line of its own runs as many sessions as a pooled one.
#
# **Scenario 13 holds the internal harness.** Its skill bodies state the
# answer of three probes: `hw-corpus` names the governs-line probe's ruling;
# `headwater-engine`, `hw-run-policy` and `hw-verification-bar` name the
# ruling-cited probe's; and `ste-editor` names the adjudication probe's. The
# dry run prints each such hit, and this header names no answer, because the
# spec is a fold under `folds:` and a fold states none. The leak
# check reads each skill body in a scenario tree and prints such a hit as
# `harness`, and the dry run refuses a scenario-13 line that pools a probe
# with a `harness` hit with another probe. So scenario 13 runs the
# governs-line probe and the ruling-cited probe each on a line of its own in
# the final round, and the adjudication probe on a line of its own in the
# screening and tuning rounds. A line leaves out, by identifier, the probes of
# the set it does not run.
#
# **What it costs.** At the declared 50 cents a session:
#
#     round screening: 1008 sessions, $504.00
#     round tuning: 3024 sessions, $1512.00 (3 tries at most)
#     round final: 3080 sessions, $1540.00
#     total: 7112 sessions, $3556.00
#
# At the 35.0 cents a session the campaign of 2026-10-03 realized, the total
# is $2,489.20.
#
# **Nobody runs this spec before the owner's ruling on that total.** The
# ruling was deferred in run 20261017-0919 (owner-question-mid-run), and the
# additive tier's ceiling is zero until it is given, so the plan refuses every
# line on the ceiling. The spec plans the four probes of #1718 and the #1709
# successor, which await the same approval of spend.

# The screening round.
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

# The tuning round.
additive scenario-0 tuning
additive scenario-1 tuning
additive scenario-2 tuning
additive scenario-3 tuning
additive scenario-4 tuning
additive scenario-5 tuning
additive scenario-6 tuning
additive scenario-7 tuning
additive scenario-8 tuning
additive scenario-9 tuning
additive scenario-10 tuning
additive scenario-11 tuning
additive scenario-12 tuning
additive scenario-13 tuning HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded
additive scenario-13 tuning HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor HW-PROBE-a-session-follows-two-citations-from-a-record-the-task-does-not-name

# The final round.
additive scenario-0 final
additive scenario-1 final
additive scenario-2 final
additive scenario-3 final
additive scenario-4 final
additive scenario-5 final
additive scenario-6 final
additive scenario-7 final
additive scenario-8 final
additive scenario-9 final
additive scenario-10 final
additive scenario-11 final
additive scenario-12 final
additive scenario-13 final HW-PROBE-the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens HW-PROBE-a-session-says-which-date-a-corrected-status-carries HW-PROBE-a-session-says-whether-a-command-or-a-hand-writes-a-governs-line
additive scenario-13 final HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on HW-PROBE-the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens HW-PROBE-a-session-says-which-date-a-corrected-status-carries
additive scenario-13 final HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on HW-PROBE-a-session-says-whether-a-command-or-a-hand-writes-a-governs-line
