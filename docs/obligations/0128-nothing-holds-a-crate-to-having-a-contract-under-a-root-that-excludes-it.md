---
id: HW-OBL-0128
status: draft
status_since: 2026-08-16
summary: "Q29 keeps the engine outside the corpus root, so a contract may name a crate and nothing names a crate that has no contract. A participation expectation runs between documents, and an anchor target is no node with a kind."
last_verified: 2026-08-16
title: "Nothing holds a crate to having a contract under a root that excludes it"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0029
---

# Nothing holds a crate to having a contract under a root that excludes it

## Context

[Q29](../spec/09-decisions.md#q29--whether-a-corpus-root-may-contain-code-and-what-an-interface-contract-may-reach) keeps the corpus root at `docs`, and it names the half of [HW-EVAL-specifying-the-engine](../evaluations/specifying-the-engine.md) finding 2 that the ruling leaves open. A document under the root reaches a crate by a `governs` edge onto a `code_path`. Nothing reaches back.

**The mechanism that would state the requirement runs between two documents.** A windowed participation expectation on a relation reports a document that stands at one end of no instance. `SourceTree::resolve` in `engine/crates/graph/src/anchors.rs` binds an anchor by `Path::exists`. So the far end of a `governs` edge is a string that resolved, and never a node with a kind. An expectation over `governs` therefore reports a contract that governs nothing, and it cannot report a crate that no contract governs.

**The absence is visible in the current graph and no run says so.** `headwater check` reports 25 distinct `code_path` anchor targets on the tree this record ships on. Ten sit under `engine/`, and eight of the ten name a path inside a crate. Those eight reach five of the 22 crates. So 17 crates stand at the far end of no edge at all, and the report that counts the ten says nothing about the 17.

## Obligation

The corpus owes a mechanism that names an undescribed subject, rather than only a description with no subject. Under Q29 that mechanism reads a path outside the corpus root, because the subject is outside it.

**What would discharge this.** A projection whose rows are derived from a path outside the corpus root, and whose empty cell is the finding. [Spec 6](../spec/06-engine-architecture.md#projections) states that a projection interpolates a path, a facet value and a declared identity, and nothing else. All three come from the graph, so no projection today reads a directory. A projection over the crate tree and the contracts that name it would write a row for every crate. A crate with no contract would be a blank cell that `generate --check` holds in a committed file.

**What would not discharge this.** A check over the contracts alone. Every such check reads the documents that exist, and the defect here is a document that does not. [Spec 12](../spec/12-check-layer.md#two-phases-and-why-the-order-matters) already states the general form of that answer. The census exists to fix the denominator before a check runs, and no census row covers a crate.

**A rule that reads the bytes of an anchor target would also close this, and it costs more.** It would make an anchor a subject rather than a name, which is the second reopening condition Q29 records. That reading exists in this tree today as a test rather than as a rule. `engine/crates/check/src/fragment.rs` holds a `comment_links` module that walks the comments of every source file of the engine.

**The cheapest candidate is in the tree already, and it is declined rather than unnoticed.** The `comment_links` module above needs no taxonomy change, no specification change and no ruling. It walks the workspace, it holds what it finds against the corpus, and it fails `cargo test` in continuous integration. A sibling of it could enumerate the 22 crates, or the verbs of the command surface, and fail when one carries no contract. That would hold this engine today.

It is declined for one reason, and the reason is what this record is about. A suite test produces no finding, reaches no obligation, and changes no verdict of `headwater check`. It holds the engine of this repository and it holds no adopter's corpus, because an adopter runs the binary rather than the suite. So it would answer the question for the one reader who is this repository, and leave every reader outside it exactly where they are. That is the split the value rule already names. A mechanism on the wrong side of it is a reason to record the gap rather than to close it.

**What that changes for the projection.** [#257](https://github.com/headwater-ai/headwater/issues/257) does not have to argue that no mechanism exists. It has to argue that the mechanism must ship in the binary, and the paragraph above is the argument.

## Discharge

Nothing here is discharged. Q29 is the ruling that produced this record, and the record is the thing Q29 declines to settle in prose.

**What would move it, in the order the corpus would meet it.** [#254](https://github.com/headwater-ai/headwater/issues/254) declares the `interface_contract` kind, which gives the far end of the relation a name. [#257](https://github.com/headwater-ai/headwater/issues/257) asks for a generated verb index, and the shape of that index decides this record. An index derived from the contracts alone lists what exists and closes nothing here. An index derived from the crate tree and the contracts together writes the empty cell, and it discharges this.

**What this record cannot state.** Whether a projection may read outside the corpus root is a question nobody has asked, and this record does not answer it. [#130](https://github.com/headwater-ai/headwater/issues/130) refused a template on a projection declaration, on the ground that a taxonomy source sits outside the root and no rule reads it. That refusal is about prose written outside the root, and a derived row is not prose. The two are near enough that a ruling should say so before a projection reads a directory.
