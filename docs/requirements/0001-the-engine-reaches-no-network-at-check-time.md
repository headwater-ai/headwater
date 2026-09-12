---
id: HW-REQ-0001
status: current
status_since: 2026-09-06
summary: "The engine opens no network connection while it reads a corpus, so a check runs offline and a resolution result stays reproducible."
last_verified: 2026-08-25
title: "The engine reaches no network at check time"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  verified_by:
    - HW-AC-0001
  governs:
    - engine/crates/graph/src/anchors.rs
    - engine/Cargo.lock
---

# The engine reaches no network at check time

## Context

Three documents state this property and no document holds it. Spec 2 states the rule and gives both reasons, which `engine/crates/graph/src/anchors.rs` quotes: check time stays offline, and a resolution result stays reproducible. `docs/tutorials/your-first-governed-corpus.md` tells a new reader that nothing in this engine fetches a package over a network. CLAUDE.md states that no crate of this engine opens a socket. Each of those is a sentence inside a larger document. A reader who asks whether the property is met, and by what, has nowhere to look.

The property is worth stating on its own for two reasons beyond the bookkeeping. A check that reaches a network has a verdict that depends on a host the corpus does not name. Two runs over one commit can then disagree, which is the reproducibility half of the rule. A gate that reaches a network fails when the network fails, and `.githooks/pre-commit` runs this engine on every commit in this repository.

## Requirement

While the engine reads a corpus, it opens no network connection.

The scope is every verb and every crate of the workspace under `engine/`. It covers the direct source of the workspace and every dependency the lock file pins. A dependency that opens a socket opens it in this process, so the pinned set is part of the claim rather than context around it. The requirement covers the whole of a run and not the check phase alone. The title names check time, because that is the moment spec 2 states the rule for, and it is the moment a gate depends on.

Three things sit outside the scope. `headwater probe` reaches a model, which spec 5 declares and CLAUDE.md repeats. It is a separate verb, and no gate and no CI job runs it. `headwater sweep` reaches a model in the same way, through the agent that runs it rather than through a socket this engine opens. A harness that runs the engine may reach a network for its own reasons, and this requirement says nothing about the harness.

## Verification

[HW-AC-0001](../acceptance-criteria/0001-no-source-file-of-the-engine-names-a-network-api-and-no-locked-dependency-provides-one.md) verifies this requirement, and its method is inspection. Nothing runs, because the property is a fact about the source and about the lock file rather than a behavior of a process. Two facts settle it, and the criterion states both.

No check rule holds this requirement. A rule identifier is an anchor now, and `verified_by` reaches one: the anchor kind is `check_rule` and the requirement block of `.headwater/overlay.yml` declares it. What is absent here is the rule and not the route. No rule of the 32 this engine ships reads a Rust source file or a lock file for a network API, which is what this requirement is about, and [HW-AC-0001](../acceptance-criteria/0001-no-source-file-of-the-engine-names-a-network-api-and-no-locked-dependency-provides-one.md) says the same thing from the other end.
