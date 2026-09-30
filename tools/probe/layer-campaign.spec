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
# probe keep their cue in the present arm (`cued:` in `.headwater/probe.yml`),
# so they run on a line of their own and are never pooled into the rate of
# the other two. Discovery runs without the authoring-skill probe, which no
# arm without `.claude/` can satisfy, and at the powered repetitions the dry
# run prints.
#
# Nobody runs this spec until the owner rules on one ceiling for the whole
# plan (#1472) and the sessions are confined (#1467).

# Sufficiency, less the two cued probes.
campaign      present      sufficiency HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted
campaign      absent       sufficiency HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted
campaign      no-hook      sufficiency HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted
campaign      no-skills    sufficiency HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted
campaign      no-claude-md sufficiency HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted
campaign      mcp          sufficiency HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted
documentation absent       sufficiency HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted

# The two cued probes, on lines of their own.
campaign      present      sufficiency HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks
campaign      absent       sufficiency HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks
campaign      no-hook      sufficiency HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks
campaign      no-skills    sufficiency HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks
campaign      no-claude-md sufficiency HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks
campaign      mcp          sufficiency HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks
documentation absent       sufficiency HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks

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
