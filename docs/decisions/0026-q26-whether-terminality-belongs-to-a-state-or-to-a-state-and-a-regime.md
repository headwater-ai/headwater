---
id: HW-DR-0026
status: draft
status_since: 2026-08-15
summary: Terminality stays a property of a state alone, because the per-regime reading is available at both call sites and costs the deferral that keeps one defect one finding. No published tradition wants the collision, and two that met it minted a second state name.
last_verified: 2026-08-15
title: "Q26 — Whether terminality belongs to a state, or to a state and a regime"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - engine/crates/resolve/src/rules.rs
    - engine/crates/check/src/lifecycle_state.rs
    - engine/crates/check/src/dependency.rs
---

# Q26 — Whether terminality belongs to a state, or to a state and a regime

## Context

A state vocabulary is one list for a whole taxonomy, and each value may carry a `role`. A `terminal-` role says the value ends a lifecycle. A lifecycle regime is a state machine over part of that vocabulary, and a kind binds one regime. So the role is written once for every regime that reaches the state.

[#241](https://github.com/headwater-ai/headwater/issues/241) found that two shipping rules read terminality from two places and could disagree. `lifecycle.dependency.on_terminal` read the role. `lifecycle.deletion.not_permitted` and `lifecycle.transition.not_permitted` read the machine. A third declaration, `regimes.lifecycle.<name>.terminal`, was read by one escape inside `lifecycle soundness` and by nothing else.

[#244](https://github.com/headwater-ai/headwater/pull/244) removed that third declaration and made `lifecycle soundness` refuse a regime where the role and the machine differ. Over any resolved taxonomy the two readings now name one set. `engine/crates/resolve/fixtures/validate/one-state-two-regimes-disagree/` is the case: two regimes share `superseded`, one gives it no exit and the other gives it one, and the second is refused.

**The cost of that refusal is a narrowing, and it reaches an adopter.** A state cannot end one regime and be a waypoint in another. A corpus whose policy documents end at `retired` and whose hardware records pass through `retired` has to rename one of the two. Nothing had ruled on whether that is right. [#242](https://github.com/headwater-ai/headwater/issues/242) states the problem plainly: the only prose asserting the narrowing was the prose the change wrote about its own change, and a ruling should not be self-certified.

**The reason both of those documents gave for the narrowing is wrong.** #242 states it as a property of the code: "Making the role per-regime would need a regime at every call site that has none." That was measured here and it does not hold. Terminality has two production readers. `crate::retention` reads the machine through `LifecycleRegime::terminal`, and it holds the regime already. `crate::dependency` reads the role through `StateFacet::standing`, and eleven lines below that reading it calls `self.shape.lifecycle_of(target.kind)` for another purpose. The target's own regime is in hand at the only call site the argument was about. A per-regime reading is a one-line change there.

The source end of that rule is different, and a refutation pass measured it. `Shape::lifecycle_of` is called once in `crate::dependency`, for the target's kind alone, so the regime of the citing document is not in hand. That does not rescue the original argument. `lifecycle.dependency.on_terminal` decides terminality at the target, and the citing document's regime is the wrong regime to ask, which `engine/crates/check/tests/terminal_dependency.rs` states in its module comment.

So the narrowing is a choice rather than a constraint of the engine, and it needs an argument that survives that fact.

## Decision

**Terminality is a property of a state, and never of a state and a regime together.** The ruling that #244 wrote about itself stands, and these are the reasons, none of which is the sentence it wrote.

**First, the per-regime reading costs a deferral, and the loss is measured rather than argued.** The reading was built. `crate::dependency` was changed to ask the target's own regime, and the converse arm of `lifecycle soundness` was removed so a disagreeing taxonomy resolves. Against a clean tree of 84 targets and 813 tests, two targets failed. One was the recorded resolution fixtures, which is the refusal going away as intended. The other was `a_target_standing_outside_its_own_machine_is_left_to_the_rule_that_owns_it`.

That test holds a `record` standing at `discharged`, which is a state its own regime never names. Under the role reading the state is terminal, the dependency rule notices, and it declines with a reason that names `lifecycle.state.not_admitted` as the rule that owns the defect. Under the per-regime reading the regime does not name the state, `terminal` is false, and the instance passes. **Three facts fold into two, and the one that disappears is the decline to judge.** A pass states that the rule read both ends and found nothing to report, and here the rule read a state that means nothing in the machine it was read against. One defect stops being one finding, because the account of why the second rule stood down is gone.

**Second, no written-down tradition wants the collision.** Eight were surveyed against primary text: the Python PEP process, the IETF standards process, the W3C Process Document, the Java JEP process, the Rust RFC process, records-management disposition scheduling, controlled-document practice under the quality-management standards, and the documentation schemes DITA, DocBook and Diátaxis. None produces a status name that is normatively an ending for one document type and a waypoint for another. The seventh produces nothing either way, because the quality-management standards enumerate no status names at all and leave the vocabulary to each implementer.

Two of them met the pressure and answered it by minting a second name, which is exactly the remedy this ruling leaves an adopter. PEP 1 gives a completed proposal `Final` and gives a proposal that is never meant to complete `Active`. The W3C Process gives the Recommendation track `Discontinued Draft`, `Obsolete Recommendation`, `Superseded Recommendation` and `Rescinded Recommendation`, and gives the Note track `Note` and `Statement`, so the two tracks never contend for one word. The JEP process shows the third safe shape, where types share a terminal state and differ only in which waypoints they cross, which is what a regime already expresses here.

**The survey has one near miss and it is not normative.** PEP 8016 is an Informational PEP that stands permanently at `Accepted`, while `Accepted` is a waypoint for a Standards Track PEP on its way to `Final`. PEP 1 never states that reading, and the shape is an artifact of PEP 1 naming no ending for an Informational proposal other than `Active`. It is observed drift rather than a documented tradition. Two limits of the survey are stated here rather than hidden: the JEP text read was a draft of the 2.0 process rather than the ratified JEP 1, and ISO 15489, ISO 9001, ISO 13485 and the ITIL texts are paywalled, so the records-management finding rests on the public DoD 5015.02 standard and on secondary quotation.

**Third, the remedy the narrowing forces is a rename, and the engine holds the split rather than leaving it to discipline.** `engine/crates/resolve/fixtures/validate/terminality-by-rename/` is the repair of the refused case beside it. The regime that continues gives the state a second value with the `live` role and its own exits, and the regime that ends keeps the `terminal-retained` value. Every value is reached somewhere, no state is read two ways, and the taxonomy resolves clean. `lifecycle.state.not_admitted` then refuses a document authored at the value its own regime does not name, which is the case `engine/crates/check/fixtures/admitted-state/records/authored-wrong.md` already carries. So an adopter who takes the rename cannot silently write the wrong word.

**Fourth, a vocabulary value carries one sentence of guidance for the whole taxonomy.** `taxonomy validate` requires guidance on every enum value of a facet, and there is one string per value rather than one per regime. A state that ends one machine and continues in another is two meanings under one word, with one sentence to explain both. Two meanings under one word is what a second word is for.

## Consequences

**What an adopter meets, and where the record sends them.** A taxonomy that wants one word to end one lifecycle and to be a waypoint in another is refused by `lifecycle soundness`, at `regimes.lifecycle.<name>.transitions`, with a message that names both readings. The arm that survives is the rename, and `terminality-by-rename/` is the worked shape rather than a sentence. The refused case and its repair sit in adjacent directories on purpose, so a reader who meets the refusal can read the two files against each other.

**What reopens this.** Two conditions, and each one is a thing that can happen.

The first is an adopter who cannot rename. A corpus whose state vocabulary is fixed by an authority outside it does not own the words. A regulator's document-control scheme, or an upstream system of record reached through the inbound path that [Q19](../spec/09-decisions.md#q19--inbound-integration-an-external-system-of-record) settled, can hand a corpus a status list it may not edit. If such a list carries one name that ends one document type and continues another, the remedy this ruling leaves is not available, and the ruling has to be argued again against a real corpus rather than against a survey.

The second is a reader that keeps three facts as three. The measured cost above is the loss of the deferral in `crate::dependency`, and it is a property of one implementation rather than of the idea. A per-regime reading that returns terminal, not terminal, and not a state of this regime as three answers, and that keeps the deferral naming `lifecycle.state.not_admitted`, dissolves the first reason. What would remain is the survey, which is an absence of evidence, and one adopter is enough to move it.

**What this does not settle.** Whether a value set narrows per kind for a facet that is not the state is [#222](https://github.com/headwater-ai/headwater/issues/222). Whether a relation may declare itself lifecycle-sensitive is [#226](https://github.com/headwater-ai/headwater/issues/226). Whether a lock reader should re-run the rule that produced the lock is [#243](https://github.com/headwater-ai/headwater/issues/243), and it bounds this ruling the same way it bounds #244: the identity of the two readings is a property of the resolver's output, and a lock written by an older engine can carry a disagreement into a newer check layer.

**A closed value set is decided here on a guess, and the record says so.** [HW-OBL-0049](../obligations/0049-the-meta-schema-closes-eight-value-sets-that-nothing-states.md) records that the meta-schema closes eight value sets that nothing states, and `vocabulary_value.role` is one of them. This ruling rests on that closure: the role admits `initial`, `live` and `terminal-retained` and no fourth value. The survey above is evidence about that one set and about none of the other seven. It does not discharge the record either, because what discharges it is a statement in the specification of each set, and spec 3 states what makes a state terminal rather than what values the role holds.

**This record is unchecked prose.** The house language regime binds four kinds and `decision` is not one of them, which [#205](https://github.com/headwater-ai/headwater/issues/205) records. No controlled-language rule, no source-form rule and no retired-term rule reads a word of this file.
