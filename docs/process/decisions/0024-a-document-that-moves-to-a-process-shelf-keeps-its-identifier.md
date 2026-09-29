---
id: HW-PD-0024
status: current
status_since: 2026-09-29
summary: "A specification, an evaluation or an obligation record that moves from a product shelf to a process shelf keeps its identifier. Each process kind binds the scheme of the product kind it mirrors, and the move rewrites every relative link to the file."
last_verified: 2026-09-29
title: "A document that moves to a process shelf keeps its identifier"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0054
    - HW-OBL-0031
    - HW-OBL-0139
---

# A document that moves to a process shelf keeps its identifier

## Context

Two process shelves existed before this record, one for decisions and one for explanations. Both hold documents that were minted there under a process scheme, `HW-PD-` and `HW-PEXP-`. No document had moved from a product shelf to a process shelf. Issue #1286 moves fifteen: the specification of the build order, three evaluations of the build order and of CI, and eleven obligation records about the run tooling. Other documents, commit messages and issues cite each of them by identifier. `HW-SPEC-orchestration-architecture` and `HW-OBL-0221` are two examples.

The question was whether a moved document keeps its identifier or takes a new one under a process scheme, such as `HW-POBL-`. Four facts on `main` at `19db1e68` settle it.

1. The corpus already binds one scheme to two kinds. `register_id` is bound to `decision_register` and to `obligation_register`, and `review_id` is bound to `review_prompt` and to `review_record`, in `.headwater/overlay.yml`.
2. The reconcile-first allocator reads every identifier on the tree and in the claim store, and not the identifiers of one shelf. `engine/crates/scaffold/src/lib.rs` chains the typed index, the untyped index and the claims of the scheme. So the next `HW-OBL-` that either shelf mints counts past a number that the other shelf holds.
3. The overlay already rules that an identifier must survive a rename, because a slug that a path derives dies at the first rename.
4. A new identifier needs a supersession edge from each new record to its old one. That edge keeps the old file on the product shelf, which is the thing the move removes.

## Decision

A document that moves from a product shelf to a process shelf keeps its identifier. The process kind that receives it binds the identifier scheme of the product kind that it mirrors. `process_spec` binds `spec_id`, `process_evaluation` binds `evaluation_id`, and `process_obligation` binds `obligation_record_id`.

Each process kind mirrors its product kind clause for clause, and it is not an `is_a` of the product kind. This is the shape that `process_decision` and `process_explanation` already have. With `is_a`, every rule that the taxonomy scopes to the product kind would also read the process records, and that would undo the split. `process_spec` does not require `doc_type` or `sequence`, because both facets place a document in the numbered series and a process specification is not a part of it.

The move rewrites every relative link to a moved file in the same change. A claim file under `.headwater/ids/` for a moved record gets the new path, as a rename did before (#1178). Recorded transcripts under `docs/probe-runs/` and the readings in `.headwater/capture-cost.jsonl` keep the old paths, because they are records of a past run and not links.

## Consequences

A citation by identifier in a commit, an issue or another document resolves after the move without change, and `headwater explain HW-OBL-0221` finds the record on its new shelf.

A mirror costs one change for each relation that names the product kind as an endpoint. The corpus used one: the specification of the build order cited the evaluation of the build order through `cites_evidence`. An overlay cannot add an endpoint to a relation that a package declares, because the two list operations do not commute ([HW-OBL-0031](../../obligations/0031-list-extension-in-an-add-only-overlay.md)). So that edge is now the base `traces_to`, which reaches any governed document and is in the same evidence family. The evaluation no longer declares the `cited_by` half.

One scheme that two kinds on two shelves share is a new instance of what [HW-OBL-0139](../../obligations/0139-an-identifier-scheme-glues-three-parts-into-one-string-so-nothing-can-decide-that-two-schemes-are-disjoint.md) records. The literal segment `OBL` no longer tells a reader which register holds a record. The path does, and `headwater explain` does.

`tools/repo/obligation-register-fixtures.sh` reads the process register too. It fails when spec 13 lists a process record, and it names that record as a process obligation. A current process record is not owed a line in spec 13.

This record reopens when the process has a reader who is not this repository, which is the condition in the overlay for a move of the process to its own corpus. A move to another repository changes the namespace of every identifier, and a decision of its own settles that.
