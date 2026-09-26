---
id: HW-DR-0084
status: current
status_since: 2026-09-26
summary: "A live document with any relation to a draft is a warning, unless the relation writes a state onto its target"
last_verified: 2026-09-26
title: "A live document that rests on a draft one is reported over every relation, because a draft leaves no record to cite"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0065
    - HW-DR-0052
    - HW-SPEC-authoring-and-lifecycle
---

# A live document that rests on a draft one is reported over every relation, because a draft leaves no record to cite

## Context

[#569](https://github.com/headwater-ai/headwater/issues/569) reported two gaps. No rule reported a live document that rests on a draft one. A shelf of obligation records also stayed at `draft` after the merge that activated them, and nothing reported that.

The owner ruled on both questions on 2026-09-26, in [a comment on the issue](https://github.com/headwater-ai/headwater/issues/569#issuecomment-5844324444). The words of the first ruling are: "A current document that cites a draft one: not silent. It is reported as a warning, which is what 0.4's accepted statement says an adopter gets." The words of the second ruling are: "The state an open obligation stands at: `current`."

`lifecycle.dependency.on_terminal` is the sibling rule. It reads only a relation that carries `lifecycle_sensitive`. [HW-DR-0065](0065-a-relation-declares-lifecycle-sensitive-for-itself-and-a-core-requirement-demands-it-of-a-family.md) rules that the published base marks no family beyond `succession`. In the base, `supersedes` is the only marked relation, and it writes a state onto its target. So a rule that read the same filter would find no pair in any adopter of the base, and the 0.4 statement would be false.

## Decision

`lifecycle.dependency.on_initial` reports a live source that declares a relation to a target at a state with the role `initial`. It reads every relation whose two ends are documents. It does not read `lifecycle_sensitive`.

HW-DR-0065 leaves `traces_to` unmarked for a reason. A live document can cite a retired one correctly, as a citation of the record that the retired document left. A draft leaves no record. The guidance that its state declares is that nothing may rely on it. So no citation of a draft is correct in the sense that HW-DR-0065 protects, and the rule has no reason to read the flag.

One relation is exempt: a relation that declares `on_target.set_state`. An edge of it is a statement about its target, and it is not a reliance on the text of the target. The exemption applies at every target state, because a successor that supersedes a draft closes that draft.

The finding is a warning and never an error. This also applies to a current specification that cites a draft one. The repair is to promote the target, to point the source at a document that stands, or to move the source back to draft. Only the author can choose among the three. `CT-LIFE-5` is `permanently_advisory` for this reason.

An open obligation record stands at `current`, as the second ruling states and as [HW-DR-0052](0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md) rules for every document. The thirteen records at `draft` moved to `current` in the same change.

## Consequences

An adopter of the base gets the finding with no overlay change, over every relation that the adopter declares. The base package declares `OB-LIFE-5` and `CT-LIFE-5` at 4.9.0.

A target that stands at a state its own regime does not name is skipped. `lifecycle.state.not_admitted` reports it once. A target with no state is also skipped, and an edge to an anchor forms no pair.

The rule does not find a draft that no live document cites. The thirteen obligation records had no incoming edge from a live document, so this rule would not have reported them. What files a record at `draft` in a run is a separate gap.
