---
id: HW-DR-0070
status: current
status_since: 2026-09-17
summary: "The purposes a task matches take turns at the route budget, and the kinds of one purpose take turns inside it. Each pointer states the task terms that reached it and its rank in the order by score. No pointer carries a score or a confidence."
last_verified: 2026-09-17
title: "The matched purposes take turns at a route budget, and each pointer states what reached it"
relations:
  governs:
    - engine/crates/query/src/route.rs
    - engine/crates/query/src/json.rs
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# The matched purposes take turns at a route budget, and each pointer states what reached it

## Context

[Issue #915](https://github.com/headwater-ai/headwater/issues/915) asks for two rulings on `headwater route`. This record rules on the first. [HW-DR-0071](0071-a-route-offers-a-document-and-never-a-heading-inside-it.md) rules on the second.

[Spec 5](../spec/05-ai-integration.md#intent-time-routing) said that lexical ranking orders results within the matched purpose. `engine/crates/query/src/route.rs` implemented that sentence as one total order, with the purpose score first. So the purpose with the highest score took every slot of the budget while it had candidates.

[The evaluation of the pointer probe](../evaluations/the-pointer-probe-grades-a-session-against-the-router-s-own-first-pick-so-a-correct-session-that-declines-a-wrong-pointer-fails.md) records the cost. The task "Decide which shelf a new document belongs on, and where its identifier comes from." scores `obligation` 7 and `behavior` 3. On the tree of #914, the first 12 of 220 pointers were all obligation records, and the default five held one kind. A comment on the issue adds a second task, from the cold-agent probe. That task scores six purposes, with `behavior` fifth, and `docs/spec/12-check-layer.md` ranked 127th of 343. It was still the first design spec in that route.

The pointer also carried no evidence. Its JSON members were `id`, `kind`, `name`, `path`, `purpose`, `summary` and `unwarranted`. No member said which task terms reached the document or where it stood in the order. An agent that read five obligation records had no way to see that they shared only the words "new", "document" and "identifier" with its task.

## Decision

**The matched purposes take turns at the budget.** The order inside one purpose does not change. It is the purpose score, then the term score, then the path. The budget takes the candidates in turns. At each turn, every matched purpose that has a candidate offers its best remaining one, in the order of the purpose scores. So the first pointer is still the best candidate of the best purpose.

**Inside one purpose, the kinds that serve it take turns the same way.** The kinds go in the order of their best candidate. So one kind takes every slot of a purpose only where no other kind of that purpose has a candidate.

**How many slots one kind may take follows from the turns, and no separate cap exists.** A kind takes a second slot only after each matched purpose with a candidate has offered one. Each other kind of its own purpose must also have offered one. A task that matches one purpose of one kind keeps the order by score.

A cap for each kind was measured and rejected. The second task matches `requirement`, `rationale`, `evidence`, `procedure`, `behavior` and `obligation`, and three kinds serve `rationale` alone. With one slot for each kind, the default five fill with the kinds of the first two purposes and never reach `design_spec`. With turns for each purpose, `docs/spec/12-check-layer.md` is the fifth pointer.

**Each pointer carries its evidence.** In `--json`, each pointer gains an `evidence` object with a `by` token:

- `by: anchor` means that the task named a path that the document governs, and `anchors` lists those paths. Nothing was ranked.
- `by: terms` means that a term of the task reached the document. `terms` lists the distinctive terms that reached it, in task order. `rank` is its place in the order by score before the turns, counted from 1, and `of` is the length of that order.

The text report prints the same evidence on one dim line under each pointer. That line carries no em dash, because `.claude/hooks/write.sh` selects pointer lines by that character.

**No pointer member is a score, and no member names or implies a confidence.** The scores are sums of term overlap, weighted by how many documents a term rules out. They are integers and not probabilities. A member named `confidence`, or a raw score beside the pointer, would claim a calibration that does not exist. A rank and the terms that produced it are facts that an agent can compare against its own task.

## Consequences

Spec 5's intent-time routing section states the amended rule. The sentence about lexical ranking now says that the matched purposes take turns at the budget.

On this tree the first task offers `HW-OBL-0107`, `docs/spec/12-check-layer.md`, `docs/evaluations/graph-export-and-federation.md`, `docs/how-to/relocate-a-shelf.md` and `HW-DR-0062`. That is five kinds under five purposes. Spec 12 does not answer that task. The documents that answer it are still out of reach, and HW-DR-0071 records why this record does not reach them.

**Diversity is not precision, and this record does not claim it is.** The change removes the case where one purpose hides every other purpose. It does not make a weak pointer strong. The evidence line is there so that an agent can see a weak pointer, for example one reached only by "where", "comes" and "from".

The recorded route of the query fixture tree moves. The task "why is throttling applied at the edge" now offers a `behavior` document second, between two `rationale` documents. The MCP transcript fixture moves with it. `engine/crates/query/tests/reads.rs` holds the property over this repository's own corpus. The first task offers more than one kind and more than one purpose within the default budget. Each pointer carries evidence. That test fails on the tree of #914.

`.claude/hooks/intent.sh` reads `pointers` and `text` and composes no line of its own, so it needs no change. The text it injects gains one evidence line for each pointer. The shadow-mode log of [HW-DR-0064](0064-q64-whether-intent-time-routing-gains-an-offline-embedding-path-in-shadow-mode.md) records the route document as the verb wrote it. So a line written after this change carries the evidence, and a comparison across the change has to compare lines on the engine version.
