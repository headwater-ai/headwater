---
id: OBL-repo-0125
status: current
status_since: 2026-08-15
summary: "Spec 3 closes the warrant vocabulary at four values, nine documents here state a fifth, no check reads a warrant, and the query surface serves an unknown value as a vouched one."
last_verified: 2026-08-15
title: "Nine documents state a warrant the closed set does not hold, and no check reads one"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - SPEC-HW-authoring-and-lifecycle
---

# Nine documents state a warrant the closed set does not hold, and no check reads one

## Context

[Spec 3](../spec/03-authoring-and-lifecycle.md#the-warrant-and-what-each-value-requires) states that the warrant "is required, its value set is closed, and an absent value is a finding rather than a default". The closed set holds `accepted`, `regenerated`, `transcribed` and `asserted`. Each value requires a different part of the provenance block, and spec 3 says the engine checks the pairing.

The warrant reading of `taxonomy audit` walks that closed set in full, and its first run over this repository found a fifth value. Nine documents state `warrant: proposed`. Eight are obligation records numbered 0115 through 0122, and the ninth is [DR-repo-0022](../decisions/0022-q22-the-integrity-posture-of-a-published-package.md). Every one of them was drafted by an agent, and every one also names an acceptor.

`proposed` appears in no package, in no overlay, in no skill and in no part of this specification. An agent coined it, and nine documents carry it.

**No check of this engine reads a warrant.** A search of the check crate for the word returns nothing. So the pairing that spec 3 says the engine checks is checked nowhere. An absent value is a finding nowhere. A value outside the closed set reached no report at all until the warrant reading landed. That reading is advisory and it gates nothing, so such a value is now visible and still unblocked.

**A reader is served an unknown warrant as a vouched one, and that is the part with teeth.** `headwater_query` sets the flag that makes a pointer state its warrant out loud by testing the value against `asserted` alone. Every other value, known or not, leaves the flag clear. [Spec 5](../spec/05-ai-integration.md) makes a pointer to an `asserted` document state that warrant beside it. `docs/decisions/README.md` is the proof of what that costs here. It lists [DR-repo-0022](../decisions/0022-q22-the-integrity-posture-of-a-published-package.md) in the same form it lists an accepted decision, with no caveat, because `proposed` is not `asserted`. So an agent that reads that index sees a document nobody accepted, presented as one that somebody did.

The predicate fails open, which is the direction this project rules against everywhere else. A one-line change would set the flag for any value outside the closed set. It is not made here. The message beside the flag reads "asserted, and no human has accepted it", and that sentence is false about a `proposed` document. The flag and its message are one decision, and the ruling below decides which way it goes.

## Obligation

Two things are owed, and they are separable.

**The engine owes a check of the warrant.** Spec 3 already states the rule, the value space and the pairing table. So the declaration a check would read exists, and nothing reads it. Whether that value space belongs in a taxonomy or stays with the engine is the prior question, because spec 3 says no taxonomy declares it.

**The corpus owes a warrant for the nine documents.** Each states an acceptor, so `accepted` is the value the block already fits. An agent may not choose it. [OBL-repo-0108](0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md) records that an agent writing the acceptance stamp is the act under review. An agent that resolved these nine by picking `accepted` would perform that act nine more times.

**The query surface owes a reading of a value it does not know.** Three answers are open. It treats an unknown value as `asserted` and says so in a message that fits. It treats one as a defect of the document and refuses to serve a pointer at all. Or it keeps the current behavior, and a check that reads the warrant is what stops an unknown value ever reaching a corpus. The third answer is the one that pairs with the first obligation above.

## Discharge

`headwater taxonomy audit` reports the count on every run, under the warrant reading, apart from the four rows of the closed set. That is the instrument, and it is the whole of what runs today. Nothing reports the third finding, because a pointer that says nothing is what a reader sees.

The first half closes when a check reads a warrant and a fixture fails without it. The second half closes when a human sets the value on the nine documents. A ruling that `proposed` belongs in the closed set closes it too, and that ruling states what the value requires. The third half closes with a ruling on what a pointer says about a warrant it does not know.
