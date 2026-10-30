---
id: HW-PROBE-a-session-says-where-a-check-gets-the-version-that-stood-before-a-change
status: deprecated
status_since: 2026-10-30
summary: "Deprecated on 2026-10-30 because no set of the additive campaign draws it, and the four probes of row 1 of HW-DR-0112 measure the action in its place (#1874)."
last_verified: 2026-10-22
probe_category: navigability
expectation: answered
action: ask_or_locate
moments: [intent]
oracle: "none"
title: "A session says where a check gets the version that stood before a change"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  examines:
    - HW-SPEC-check-layer
  traces_to:
    - HW-OBL-0223
    - HW-OBL-0004
    - HW-SPEC-check-layer
---

# A session says where a check gets the version that stood before a change

## Task

You are extending this engine so that `lifecycle.transition.not_permitted` can judge a status change, which needs the version of the document that stood before the change. In this engine, how does a check obtain that prior version? Answer with one word and nothing else: `git` if the check runs a version control command itself, `history` if the engine walks the repository history through a library, or `manifest` if a caller hands the run a file that names the prior bytes.

Your whole final message is that one word.

## Target

[Spec 12, *Temporal inputs: the clock and the prior version*](../spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version).

## Key

```yaml
answers: [git, history, manifest]
expected: [manifest]
```

**Where plain code carries it.** `engine/crates/check/src/change.rs:1-10`: "It walks no history, it runs no version control tool, and it opens no file that the caller did not name." `engine/crates/vcs/src/lib.rs:1-20`: "The one crate of this workspace that runs a version control command", and the check-evaluation path "runs no version control command". `engine/crates/cli/src/lib.rs` carries `--change <manifest>` on `check`. The distractor is real and in plain code: `headwater change` and the `vcs` crate run version control commands. A session that finds the `vcs` crate first and stops gives the wrong answer.

**Which documents help, and why.** [Spec 12](../spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version) states the rule and its boundary in one section, with the reason. [HW-DR-0072](../decisions/0072-the-binary-is-the-only-interface-an-adopter-must-run-and-every-integration-point-outside-it-is-declared.md) rules which verb may run version control plumbing and why. [The `headwater change` contract](../interfaces/headwater-change.md) says that the verb is a producer outside the check loop.

**Expected rates, from proposal 2 of #1835.** The guess floor is 33 percent. Scenario 0: 45 to 60 percent. The right answer is in the rustdoc of two modules, and the wrong one is in the rustdoc of a third, so the rate is the share that reads `change.rs` or the `vcs` header rather than stopping at the first place that runs a command. Scenario 1: 80 to 90 percent.

**Leak.** The comments state the rule in prose and name spec 12. None of them holds the expected word beside a marker that names this probe. The expected word is also the name of the flag's operand, which is the one place plain code states it as a word, and that is the fair case.
