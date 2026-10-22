# The code round of the additive campaign (#1835): the code set in plain code
# and in plain code with the documents, so that scenario 1 against scenario 0
# reads as whether the documents help an agent write code.
#
#     sh tools/probe/campaign.sh --dry-run --spec tools/probe/code-campaign.spec
#
# HW-DR-0111 registers the comparison. The `code` round under `rounds:` in
# `.headwater/probe.yml` is sized as the final round is, so the dry run
# computes 44 repetitions of each of the five probes, 220 sessions in each
# scenario and 440 in all. The round is approved for nothing until the owner
# rules on its spend with #1778, so the batch refuses both lines with 5.
# `tools/probe/code-pilot.spec` is the pilot that measures each probe's rate
# first. This spec names no probe, so it is not a fold.
additive scenario-0 code
additive scenario-1 code
