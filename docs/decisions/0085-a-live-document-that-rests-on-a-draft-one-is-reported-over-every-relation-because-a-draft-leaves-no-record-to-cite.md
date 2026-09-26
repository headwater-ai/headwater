---
id: HW-DR-0085
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

`lifecycle.dependency.on_initial` reports a live document that writes a relation to a document at a state with the role `initial`. It reads every relation whose two ends are documents. It does not read `lifecycle_sensitive`.

The rule reads each edge in the direction that its author wrote it. The document that writes the line is the document that cites. So a live document that writes the inverse name onto a draft gets a warning. The relation declares the draft as its source, and that does not change the finding. A draft that writes the inverse name onto a live document gets no warning.

HW-DR-0065 leaves `traces_to` unmarked for a reason. A live document can cite a retired one correctly, as a citation of the record that the retired document left. A draft leaves no record. The guidance that its state declares is that nothing may rely on it. So no citation of a draft is correct in the sense that HW-DR-0065 protects, and the rule has no reason to read the flag.

One relation is exempt: a relation that declares `on_target.set_state`. An edge of it states that its target is replaced, and it is not a reliance on the text of the target. The exemption applies at every target state, as the parent ruled. It has a silent case. A live document that supersedes a draft which still stands at `draft` gets no warning. The base regimes let a draft move only to `current` or `deprecated`. So a draft cannot move to `superseded`, and no verb writes that state onto a target. The draft stays at `draft`, and this rule does not report the pair.

Two relations of the base are read although neither is a reliance of the writer on the document named. `constrains` runs from the decision that constrains to the decision it constrains, so a settled decision that constrains a new draft is the usual shape. `conflicts_with` declares that it is invalid when both ends are `current`. So a current decision that conflicts with a draft is the only valid shape of that relation with a live end. The rule warns on both on purpose. The ruling is every relation, and the engine exempts a relation only by a declaration that it can read. A live document that constrains a draft, or conflicts with one, names a text that can still change. A declaration on the relation that says it is not a reliance would exempt it, and no such declaration exists yet.

The finding is a warning and never an error. This also applies to a current specification that cites a draft one. The repair is to promote the target, to point the source at a document that stands, or to move the source back to draft. Only the author can choose among the three. `CT-LIFE-5` is `permanently_advisory` for this reason.

An open obligation record stands at `current`, as the second ruling states and as [HW-DR-0052](0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md) rules for every document. The thirteen records at `draft` moved to `current` in the same change.

## Consequences

An adopter of the base gets the finding with no overlay change, over every relation that the adopter declares. The base package declares `OB-LIFE-5` and `CT-LIFE-5` at 4.9.0.

A target that stands at a state its own regime does not name is skipped. `lifecycle.state.not_admitted` reports it once. A target with no state is also skipped, and an edge to an anchor forms no pair.

`headwater new` writes a new document at `draft`. For a relation that it creates, it also writes the reciprocal half into the live document at the far end. That live document then writes a line to a draft, and it gets a warning until the author promotes the new document. The base `supersedes` is exempt, so this does not occur for a successor. It does occur for `verified_by` in this repository, and for every relation that the scaffolder creates and that writes no state. The repair is the one that [HW-DR-0052](0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md) names: promote the new document before you propose it.

The rule does not find a draft that no live document cites. What files a record at `draft` in a run is a separate gap.
