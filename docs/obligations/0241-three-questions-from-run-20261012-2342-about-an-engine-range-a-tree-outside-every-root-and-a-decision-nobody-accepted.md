---
id: HW-OBL-0241
status: current
status_since: 2026-10-13
summary: "Run 20261012-2342 raised three questions that no record answers: which decision names the engine range a corpus declares, whether integrations/ stays outside every corpus root, and who accepts HW-DR-0109."
last_verified: 2026-10-13
title: "Three questions from run 20261012-2342 about an engine range, a tree outside every root and a decision nobody accepted"
waiting_on: ruling
provenance:
  warrant: asserted
---

# Three questions from run 20261012-2342 about an engine range, a tree outside every root and a decision nobody accepted

## Context

Run `20261012-2342` added the `engine` key of a corpus (#1600, slice 1), fixed the version read of the coverage dashboard (#1435) and changed how `headwater route` ranks a title query (#1470). Its builders, maintainers and verifiers found three cases that no rule settles. No build closes them until a person rules, so the product owner ruled each one RECORD with `waiting_on: ruling`, two at the fifth-merge pass and one at the final pass, both on 2026-10-13. This is one of two spec 13 records for that run. Each item was checked on `56febd34`. The product owner read the `engine` key of `.headwater/taxonomy.yml`, the paragraph of spec 7 that opens "The `engine` key states the engines that this corpus asks for", the `corpora` list of `.headwater/corpus.json`, and the `provenance` block of HW-DR-0109.

## Obligation

- Spec 7 states the `engine` key of a corpus and the refusal of `headwater taxonomy resolve` when the corpus range and the `requires_engine` of a package have no version in common. No decision record names the key or the refusal. HW-DR-0060 and HW-DR-0088 rule on the engine version and on `requires_engine`. The question is whether the corpus range owes a decision record beside them, or whether spec 7 is enough.
- `integrations/` is in no corpus root, so `headwater check` reads nothing under `integrations/dashboard/`, which reads the graph export of the engine. The question is whether that is by design. If it is, a sentence should say so. If it is not, the taxonomy should give the tree a root or a code path scope.
- [HW-DR-0109](../decisions/0109-a-task-that-repeats-the-declared-name-of-a-document-names-that-document.md) is at `status: current` with `warrant: asserted` and no `accepted_by`. During the run, the parent accepted the case that a task which mentions a verb name, such as "headwater check", names the interface page of that verb. The record still waits on a person to accept it. Reopen that case if a measured route shows a verb-name mention burying the document the task needed.

## Discharge

Each item discharges alone, when its ruling is recorded where the next reader meets it: a decision record or a sentence in spec 7 for the first, a sentence in the taxonomy or a new root for the second, and `accepted_by` on HW-DR-0109 for the third. The record discharges when every item has.
