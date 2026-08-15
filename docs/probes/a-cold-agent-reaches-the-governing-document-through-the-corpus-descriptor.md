---
id: HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor
status: draft
status_since: 2026-08-14
summary: The descriptor names five entry points, and this asks whether a session with no other pointer opens the one that governs the task.
last_verified: 2026-08-14
probe_category: discovery
expectation: opened
oracle: "none"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft
  evidence_basis: unevidenced
relations:
  examines:
    - HW-SPEC-check-layer
  traces_to:
    - HW-OBL-0010
---

# A cold agent reaches the governing document through the corpus descriptor

## Task

Add a rule to this engine that reports a document whose summary is empty. Say where the rule is declared, where it is implemented, and what its severity is.

The session opens with `.headwater/corpus.json` and with no other pointer into the corpus. That is the state [Q14](../spec/09-decisions.md#q14--discovery-surface) claims the descriptor serves: a cold agent that has read one generated file and nothing else.

## Expectation

`opened` over `HW-SPEC-check-layer`. The transcript shows that the session read the document that states what a rule is, where it is declared and what makes it an error rather than advisory.

The task is written against behavior and not against a phrasing. It names no document, no shelf and no file path, so a session that reaches the check layer reached it through the descriptor or through the graph, and a session that answers from the task statement alone has not.

**This probe measures the descriptor and never the answer.** Whether the rule the session proposes is a good rule is a judgment, and a question that needs a judgment is not a probe. The predicate is over the read set, which is a fact about the transcript.

[HW-OBL-0010](../obligations/0010-the-corpus-descriptor-exists-and-no-probe-has-run-against.md) is what this discharges when it runs. It has not run: no transcript exists in this repository, and the recorder that would write one is not here.
