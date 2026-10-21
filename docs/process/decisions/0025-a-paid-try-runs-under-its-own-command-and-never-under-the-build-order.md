---
id: HW-PD-0025
status: current
status_since: 2026-10-21
summary: "A paid try of the tuning round runs under `/next-experiment` in a session of its own. The build order never queues an issue labeled `paid-experiment`. The first try went through the build order and took 14 agents and 632 turns."
last_verified: 2026-10-21
title: "A paid try runs under its own command and never under the build order"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - to: .claude/commands/next-experiment.md
      verified_revision: sha256:c53c9b1ff4500832ee4187e5f4825fcf79ca3b436e47d0c72288e5ac72470977
    - to: tools/probe/try.sh
      verified_revision: sha256:bde5e71b4164281184820d4459566cece26d2140e86631c0827e6bce87df5593
    - to: tools/probe/additive-launch.env
      verified_revision: sha256:b7bdd819610d49e23ca67556db5be04c29293570663489d1e966d27cc0e03b07
  traces_to:
    - HW-DR-0111
    - HW-PD-0022
---

# A paid try runs under its own command and never under the build order

## Context

The tuning round of the additive campaign spends money on probe sessions, one try at a time ([HW-DR-0111](../../decisions/0111-the-additive-campaign-adds-six-layers-to-plain-code-in-fourteen-scenarios-and-registers-one-comparison-each.md)). The owner ruled that a person reads each result before the next try (OWNER #1777, 2026-10-19). So one try is one launch, one wait and one record, and then a stop.

The first try, try 1 of the documents layer under #1777, went through the build order in run `20261018-0144`. `/next-run` queued #1777, and the loop for one issue ([HW-PD-0022](0022-the-verify-and-rework-loop-for-one-issue-runs-below-the-parent.md)) took the try as an issue to build and verify. `tools/run/run-census.sh` reads the agents of that chain from the subagent transcripts of the run's parent session `dd2c5bac-42af-45b6-8420-8f21d3af5d7e`:

| part of the chain | agents | turns | tool calls | cache reads |
|---|---|---|---|---|
| the change that the try measured, PR #1816 | 6 | 416 | 488 | 43,485,999 |
| the try and its record, PR #1817 | 7 | 201 | 234 | 19,084,605 |
| the product owner that posted the approval | 1 | 15 | 18 | 396,756 |
| all | 14 | 632 | 740 | 62,967,360 |

The 7 agents of PR #1817 are two loop agents, two builders, a maintainer, a verifier and an integrator. One builder alone took 115 of the 201 turns. The census prints no dollar total for an agent. It also has no price for the model of four of the 14 agents. So this record states no dollar figure for the chain. The parent's own turns for #1777 are not in the table.

The chain made two decisions: launch the try, and do not keep the change. The tuned version solved 32 of 72 tasks against 39 of 72 for the base. `tools/probe/try.sh` now carries each step of a try as a verb: it plans, launches, waits, reads the canary and records the result. A builder and a verifier add no check that the script does not already make.

## Decision

A paid try runs under `.claude/commands/next-experiment.md`, in a session of its own, and never inside `/next-run`. The command names only verbs that the header of `tools/probe/try.sh` states. It names no stage of the loop that builds an issue: no adjudicate, loop, build, verify or integrate agent. The session runs one try, opens one pull request for its record, reports to the owner in plain terms and stops.

`headwater-product-owner` labels an issue `paid-experiment` when its next step spends money on probe sessions. `hw-queue` never queues such an issue. It names each one on an `EXPERIMENT:` line of its report, so the parent sees the work and no slot takes it.

Case 17 of `.claude/agents/fixtures.sh` holds the three parts. It reads the verbs of the script from its header, and not from a copied list.

## Consequences

A try costs the turns of one session and the money of its sessions, and no loop agent, builder or verifier. The keep rule and the base invalidators stay in `tools/probe/try.sh`, where the owner's rulings put them, and the command cites them.

A verb that the script adds reaches the command only when the header states it, because case 17 refuses any other verb. A new kind of paid work, other than a try of the tuning round, needs its own command or an amendment of this record. The label alone does not say how to run it.

The record of a try still goes through a pull request and the merge queue, and CI checks it. The session does not merge it.

This record reopens if a try needs a check that a verifier can make and the script cannot. It also reopens if the owner rules that a try may run unattended.
