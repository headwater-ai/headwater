# The campaign that separates the Headwater layer from the documents (#1472).
#
#     sh tools/probe/campaign.sh --dry-run --spec tools/probe/layer-campaign.spec
#
# Six arms of the campaign tier, each against the one present tree: the whole
# layer removed (`absent`), one component of it removed (`no-hook`,
# `no-skills`, `no-claude-md`) or added (`mcp`). The documentation tier's
# absent arm keeps the documents' own comparison (HW-OBL-0223), against the
# campaign's absent arm and present arm of the same batch.
#
# Sufficiency is two lines per arm. The status probe and the accepted-event
# probe keep their leak string in the present arm (`leaks_kept:` in `.headwater/probe.yml`),
# so they run on a line of their own and are never pooled into the rate of
# the other two. Discovery runs without the authoring-skill probe, which no
# arm without `.claude/` can satisfy, and at the powered repetitions the dry
# run prints.
#
# A line names the probes it leaves out. So each sufficiency probe that is not
# leak-kept is named on the leak-kept lines, or it would share a line with the
# two leak-kept probes and the dry run would refuse the spec. The two harder
# sufficiency probes of #1472 are named there for that reason.
#
# Nobody runs this spec until the owner rules on one ceiling for the whole
# plan (#1472) and the sessions are confined (#1467).
#
# ## What the plan costs, and the cuts the owner has not ruled
#
# The owner asked on #1472 (RULING #1472-a, 2026-10-01) for a check of
# inefficiencies before any spend. Every figure below is from
# `campaign.sh --dry-run` of this spec, at 50 cents a session. NO CUT BELOW IS
# APPLIED: every line of this spec still runs, and only the owner's ruling on
# the $2,142.00 plan can remove one.
#
# The plan is 4284 sessions, $2,142.00. By tier and category:
#
#     campaign discovery                  2124 sessions  $1,062.00  49.6%
#     campaign navigability                900 sessions    $450.00  21.0%
#     campaign sufficiency                 720 sessions    $360.00  16.8%
#     campaign sufficiency (leak-kept)     360 sessions    $180.00   8.4%
#     documentation sufficiency            120 sessions     $60.00   2.8%
#     documentation sufficiency (leak-k.)   60 sessions     $30.00   1.4%
#
# Candidate 1, discovery powered in two arms only: saves 1056 sessions,
# $528.00. Discovery runs 3 probes x 118 repetitions = 354 sessions in each of
# six arms. The 118 powers 15% against 8% (`power:` in .headwater/probe.yml),
# the present arm against the absent arm. A component arm differs from the
# present arm by a part of those 7 points, so 354 sessions do not power that
# comparison either. The cut runs present and absent at 354 (708 sessions,
# $354.00), and the four component arms at the tier's 30 repetitions (4 x 90
# = 360 sessions, $180.00). It gives up a powered discovery reading for each
# component, which the plan does not give today.
#
# Candidate 2, the documentation absent arm (lines 7 and 14): saves 180
# sessions, $90.00. HW-OBL-0223 is discharged at +57.5 points (Newcombe +47.1
# to +66.0). The cut gives up a second measurement of that comparison in the
# same batch as the component arms. Its paragraph "The plan of #1472 keeps
# this comparison" must be amended with the cut.
#
# Candidate 3, the leak-kept lines (lines 8 to 14): saves up to 420 sessions,
# $210.00. In the 2026-09-30 batch the present arm answered the status probe
# 30 of 30 times with no tool call, because the always-loaded text holds the
# answer. The absent arm made a call in 30 of 30 and answered in 29. What each
# arm's line still tells: `absent` and `no-claude-md` tell whether the answer
# is reached without the always-loaded text. `no-hook` and `no-skills` keep
# that text, so they are expected to repeat the present arm. `mcp` adds a
# tool and keeps the text, so it is expected to repeat it too. Cutting the
# three expected repeats (`no-hook`, `no-skills`, `mcp`) saves 180 sessions,
# $90.00, and gives up a check of that expectation. Cutting the documentation
# line too is in candidate 2.
#
# Candidate 4, the cost of a session and the turn cap: no inefficiency found.
# The 2026-09-30 batch recorded 658 sessions for 29975 cents, 45.6 cents a
# session against the declared 50. By line it was 26.6 to 61.4 cents: the
# present arm's discovery line was 61.4 cents a session (60 sessions, 3686
# cents), so the present discovery line here can cost about $217 and not the
# $177 the dry run prints. 2 of 658 sessions stopped at the 80-turn cap. The
# record keeps tool calls and not turns: the median session made 10 calls,
# and 62 of 658 made more than 40. So a lower cap is not shown to change no
# outcome, and this note does not propose one.
#
# Candidates 1 to 3 together save 1056 + 180 + 180 = 1416 sessions, $708.00,
# and leave 2868 sessions, $1,434.00.

# Sufficiency, less the two leak-kept probes.
campaign      present      sufficiency HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted
campaign      absent       sufficiency HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted
campaign      no-hook      sufficiency HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted
campaign      no-skills    sufficiency HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted
campaign      no-claude-md sufficiency HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted
campaign      mcp          sufficiency HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted
documentation absent       sufficiency HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted

# The two leak-kept probes, on lines of their own.
campaign      present      sufficiency HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks HW-PROBE-a-session-follows-a-citation-from-one-record-to-the-next HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged
campaign      absent       sufficiency HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks HW-PROBE-a-session-follows-a-citation-from-one-record-to-the-next HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged
campaign      no-hook      sufficiency HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks HW-PROBE-a-session-follows-a-citation-from-one-record-to-the-next HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged
campaign      no-skills    sufficiency HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks HW-PROBE-a-session-follows-a-citation-from-one-record-to-the-next HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged
campaign      no-claude-md sufficiency HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks HW-PROBE-a-session-follows-a-citation-from-one-record-to-the-next HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged
campaign      mcp          sufficiency HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks HW-PROBE-a-session-follows-a-citation-from-one-record-to-the-next HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged
documentation absent       sufficiency HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks HW-PROBE-a-session-follows-a-citation-from-one-record-to-the-next HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged

# Navigability.
campaign      present      navigability
campaign      absent       navigability
campaign      no-hook      navigability
campaign      no-skills    navigability
campaign      no-claude-md navigability
campaign      mcp          navigability

# Discovery, less the authoring-skill probe, at the powered repetitions.
campaign      present      discovery HW-PROBE-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document
campaign      absent       discovery HW-PROBE-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document
campaign      no-hook      discovery HW-PROBE-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document
campaign      no-skills    discovery HW-PROBE-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document
campaign      no-claude-md discovery HW-PROBE-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document
campaign      mcp          discovery HW-PROBE-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document
