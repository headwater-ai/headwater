---
id: HW-PROBE-a-session-says-what-a-refusal-writes-on-standard-output-under-json
status: current
status_since: 2026-10-22
summary: "The deliberate case where plain code states the answer in a macro and a test name, against the prior of a general agent that a JSON flag writes an error object, so the documents must state it where a search lands first."
last_verified: 2026-10-22
probe_category: navigability
expectation: answered
action: ask_or_locate
moments: [intent]
oracle: "none"
title: "A session says what a refusal writes on standard output under JSON"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  examines:
    - HW-DR-0043
  traces_to:
    - HW-OBL-0223
    - HW-OBL-0004
    - HW-DR-0043
---

# A session says what a refusal writes on standard output under JSON

## Task

You are adding a refusal to `headwater explain`: when the identifier the caller names has no document in the corpus, the verb refuses. A caller passed `--json`. What does the verb write on standard output for that refusal? Answer with one word and nothing else: `document` if it writes a JSON document that states the refusal, or `nothing` if it writes no bytes on standard output.

Your whole final message is that one word.

## Target

[HW-DR-0043](../decisions/0043-q43-whether-a-refusal-under-json-is-a-json-document.md).

## Key

```yaml
answers: [document, nothing]
expected: [nothing]
```

**Where plain code carries it.** `engine/crates/cli/src/lib.rs:95-121`: the macro `a_refusal_is_not_an_artifact`, whose text is "A run that refuses writes nothing here: the account is one English sentence on standard error and the status is 1", and the comment above it that separates a refusal from a non-zero exit. `engine/crates/cli/tests/json.rs:405`, `a_refusal_writes_no_document_and_accounts_for_itself_on_the_other_stream`, asserts that standard output is empty for every refusal of every verb. The test name states the answer in prose, so a session that searches the tests for a refusal finds it.

**This is a deliberate case of the code set: plain code states the answer, and the documents state it better.** The macro and the test name state the rule, so a session in scenario 0 can find it. The documents state it with its reason, for every verb, where a search for the flag lands first. So this probe measures whether the documents make a stated convention reachable, against the prior of a general agent, which is the other answer because most tools write an error object under a JSON flag.

**Which documents help, and why.** HW-DR-0043 is the ruling with its reason: `--json` names the shape of an artifact, and it moves no stream and no grammar. Every interface contract states it for its verb, [the `headwater explain` contract](../interfaces/headwater-explain.md) among them. [Spec 6](../spec/06-engine-architecture.md) says that a refusal is never a document in the target a caller named.

**Expected rates, from proposal 2 of #1835.** The guess floor is 50 percent. Scenario 0: 50 to 65 percent. Scenario 1: 80 to 90 percent. The difference is in range, and the power at 18 sessions is poor. Proposal 2 found it adequate for 25 points at 55, and the code round runs 44.

**Leak.** The macro and the test name state the rule in prose. The two closed-set words are ordinary words, and no line of plain code pairs one with a name of this probe. That is the fair shape.
