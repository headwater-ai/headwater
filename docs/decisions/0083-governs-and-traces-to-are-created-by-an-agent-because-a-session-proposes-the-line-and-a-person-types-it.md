---
id: HW-DR-0083
status: current
status_since: 2026-09-26
summary: "The base package declares `created_by: agent` on `governs` and `traces_to`, because no verb writes either edge and a person types the line a session proposes"
last_verified: 2026-09-26
title: "Governs and traces_to are created by an agent, because a session proposes the line and a person types it"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: evidenced
relations:
  governs:
    - taxonomy-source/headwater-standard/taxonomy.yml
  traces_to:
    - HW-OBL-0105
    - HW-SPEC-taxonomy-model
    - HW-EVAL-default-taxonomy-first-run
---

# Governs and traces_to are created by an agent, because a session proposes the line and a person types it

## Context

`created_by` names the actor that a taxonomy expects to pay for an edge, from the closed set that [spec 2](../spec/02-taxonomy-model.md#who-creates-each-edge) states. Until this change the base package gave `governs` and `traces_to` the value `hook`. [HW-OBL-0105](../obligations/0105-nothing-plays-the-hook-role-that-two-relations-name.md) records that no verb writes either edge, and that every such edge in this corpus is hand entry.

On 2026-09-26 no verb of the engine writes either edge. `headwater new --relates` refuses a relation whose `created_by` is not `scaffold`. `headwater check --fix` writes a reciprocal half and a `verified_revision`, and it proposes no edge. The write-time hook that [#953](https://github.com/headwater-ai/headwater/issues/953) builds prints the front-matter lines of a `governs` edge and writes nothing.

The owner filed [#955](https://github.com/headwater-ai/headwater/issues/955), and its first Done-when clause is the ruling: "The base package declares `created_by: agent` on `governs` and `traces_to`". The issue also states why: "the proposal reaches a person and the person types the line." The actor section of [the governs evaluation](../evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md#the-actor-created_by-agent-and-the-proposal-reaches-a-person) carries the design. [HW-DR-0074](0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md) rules on denotation and says that it does not rule on the actor.

## Decision

The base package declares `created_by: agent` on `governs` and on `traces_to`. A session proposes the edge and a person accepts it by typing the line.

The value `hook` claimed a mechanical creator that does not exist. Which document governs which file is a judgment, and the [stop rules](../spec/05-ai-integration.md#the-stop-rules) forbid an agent that invents structure. So a hook that reads a change can show a candidate line, and the line reaches the file only through a person.

The change is a minor release of the package, 4.7.0. No check reads `hook` or `agent`, so no finding changes. `headwater taxonomy audit` moves both relations from the `hook` row to the `agent` row, so a consumer's audit report changes. That is why the release is not a patch.

## Consequences

Four of the five base relations have no mechanical creator: `governs`, `traces_to`, `constrains` and `conflicts_with`. Only `supersedes` has one, which is the scaffold. No base relation is `created_by: author`, so the rule that spec 2 states still holds.

Over this corpus on 2026-09-26 the audit moves 533 of 762 declared edge halves from `hook` to `agent`. The capture of each relation does not change, because the reading keys on the relation and not on its actor.

The skills fixture that holds the refusal of `headwater new --relates traces_to` now expects `created_by: agent` in the message. The refusal itself does not change.

This ruling covers the base package only. The bundle relations `discharges` and `cites_evidence` still declare `created_by: hook`, and nothing writes either of them.

A verb that writes these edges from a change, with no person between the proposal and the file, reopens this question. That verb would make `hook` true again.
