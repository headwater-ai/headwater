---
id: HW-PROBE-a-session-follows-a-citation-from-one-record-to-the-next
status: current
status_since: 2026-10-03
summary: The answer is in no single record. One record names the ruling it rests on, and a second record states that a third replaced that ruling.
last_verified: 2026-10-03
probe_category: sufficiency
expectation: answered
oracle: "none"
title: "A session follows a citation from one record to the next"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-OBL-0014
---

# A session follows a citation from one record to the next

## Task

Decision record HW-DR-0061 of this repository rests on one earlier decision record, which it names in its summary. That earlier record is not in force today. Which decision record is in force in its place?

Answer on one line with the identifier of that record, in the form `HW-DR-NNNN`, and with nothing else. Your whole final message is that identifier.

## Expectation

The terminal answer is `HW-DR-0097`, the one expected value of the closed set below.

```yaml
answers: [HW-DR-0039, HW-DR-0078, HW-DR-0097]
expected: [HW-DR-0097]
```

**This probe has the multi-document shape (#1472).** The answer comes from two documents, and from no one of them alone:

1. [HW-DR-0061](../decisions/0061-q61-how-a-recorded-terminal-demonstration-is-held-against-a-run.md) names the record it rests on. Its summary says that a recorded terminal demonstration is a figure under HW-DR-0039. It does not name HW-DR-0097.
2. [HW-DR-0039](../decisions/0039-q39-how-a-figure-reaches-a-hand-built-page.md) is `superseded`, and its `superseded_by` edge names HW-DR-0097. It does not name HW-DR-0061. [HW-DR-0097](../decisions/0097-a-figure-on-a-hand-built-page-is-measured-when-the-site-is-published-and-the-committed-page-carries-none.md) states the same edge from the other end, with `supersedes: HW-DR-0039`, and it does not name HW-DR-0061 either.

So a session must read the citation in the first record and then the state of the record it cites.

**The two wrong values are the values of one document.** A session that reads HW-DR-0061 alone answers `HW-DR-0039`, the record it names. Or it answers `HW-DR-0078`, the record that replaced HW-DR-0061 itself, which its own `superseded_by` edge names. Both are values of the closed set, so the grade finds either one wrong.

**An index holds the pieces and does not join them.** The generated index `docs/decisions/README.md` lists HW-DR-0061, HW-DR-0097 and HW-DR-0039 on three adjacent lines. It marks HW-DR-0061 and HW-DR-0039 as superseded, and the summary of HW-DR-0061 there names HW-DR-0039. It does not state which record replaced HW-DR-0039. A session that answers from the index alone infers the join from the order of the lines. This is a weaker route than the edge, and the grade cannot tell the two apart.

[HW-OBL-0014](../obligations/0014-no-probe-tests-whether-an-agent-reaches-the-adjudication-from.md) records that no probe tests whether an agent reaches a ruling from a document that does not hold it. This probe narrows that record and does not close it.
