---
id: HW-OBL-0143
status: current
status_since: 2026-09-06
last_verified: 2026-10-01
title: "A specification sentence closes a set with a count the source exceeds, and 115 of 127 candidates are unchecked"
summary: "Three sentences that closed a set with a wrong count were found by accident, and 115 more candidates were never checked."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
waiting_on: measurement
---

# A specification sentence closes a set with a count the source exceeds, and 115 of 127 candidates are unchecked

## Context

Pull request #262, commit 7772252, fixed three sentences of one defect class. A specification sentence closes a set with a count, with "both", or with "either", and the source it describes has one more member. The three sentences were in `docs/spec/12-check-layer.md`, `docs/spec/01-conceptual-model.md`, and `docs/spec/02-taxonomy-model.md`. The find came from writing an interface_contract, because a contract forces an exhaustive enumeration where a specification wrote a summary. A grep of `docs/spec/` for a sentence closing a set with a number word, "both", or "either" returns 1109 candidates. Restricting to a definite determiner or a pronoun subject gives 497. Restricting to a noun a source can enumerate — exits, verbs, crates, rules, kinds, facets, relations, shelves, formats, streams, severities, scopes, emitters, arms — gives 127.

## Obligation

Twelve of the 127 candidates are checked, and nine passed while three failed. The three failures are the sentences #262 already corrected. A further 115 candidates remain unchecked against the source each one describes. No rule reads a count in prose against a source, and none could without knowing which source a sentence describes. The gap is not a cost the check layer has priced. It is a class the check layer's own subject matter excludes.

**Two more instances were found on 2026-09-30, and [#1487](https://github.com/headwater-ai/headwater/issues/1487) fixed both.** Spec 12 named six barriers where the read set prints seven, because it left out `link.fragment.unresolved`. Spec 15 gave the `tier` key two values where `Tier::ALL` in `engine/crates/probe/src/lib.rs` has three, because it left out `documentation`. Neither sentence was in the population of 127. The first names no number word, and the second is a table row.

**The population was re-derived on 2026-10-01, and it is 90 sentences.** The original greps were described and not recorded, so this filter is new, and its counts do not compare with 1109, 497 and 127. The unit is one sentence of running prose in `docs/spec/*.md`. The filter skips fenced code and table rows. A sentence ends at a period, a question mark or an exclamation mark, followed by a space. Three nested filters apply:

1. The sentence holds a number word from `two` to `twelve`, or `both`, or `either`. 1207 sentences match.
2. That word follows `the`, `these`, `those`, `its`, `their`, `all`, `they` or `it`. 343 sentences match.
3. The sentence also names a noun that a source can enumerate, in the singular or the plural. The nouns are exit, exit status, verb, crate, rule, kind, facet, relation, shelf, format, stream, severity, scope, emitter, arm, barrier, tier and key. 90 sentences match.

None of the 90 is checked against its source yet. The filter misses a table row and a list with no number word, as the two instances above show. So a sweep of the 90 does not close the class.
## Discharge

Discharge requires every one of the 127 candidates checked against its source, or the population re-derived with a stated filter and checked again. A sentence found wrong needs correction, or its count replaced by a reference to the verb or file that produces it. Spec 6 states that a count copied into prose is a claim no run re-derives. This waits on the sweep running to completion. No rule traces a sentence to the source it describes, and none has been designed.
