---
id: HW-DR-0106
status: current
status_since: 2026-10-03
summary: "When a rule moves to a new home, a document that still governs changes its link to the home and keeps its claim. An evaluation, a review or a probe run stays as written, and its date or commit marks its moment"
last_verified: 2026-10-03
title: "A governing document repoints a link to a rule that moved, and a record of a moment stays as written"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: evidenced
---

# A governing document repoints a link to a rule that moved, and a record of a moment stays as written

## Context

A rule sometimes moves from one document to another. In #1572, the trim of [spec 6](../spec/06-engine-architecture.md) moved the export rules to [spec 7](../spec/07-distribution-and-federation.md#an-export-is-a-projection-and-it-declares-what-it-dropped). It moved the audit readings to [the `headwater taxonomy` contract](../interfaces/headwater-taxonomy.md#description). It moved what a run states beside its findings to [Checks and cache](../subsystems/checks-and-cache.md#what-a-run-states-beside-its-findings). Seven sections of spec 6 now state only a pointer to the new home. On 2026-10-03, 221 links outside `docs/reviews/` and `docs/probe-results/` still named spec 6. Of these, 96 were in evaluations and probe runs, and 125 were in 64 documents that still govern.

Each link credits spec 6 for a sentence. Where the rule moved, the credit is now one hop from the text that states it. The question was which of the two classes of document to change. An evaluation records what was true when someone measured it, and a change to it rewrites history. A decision or an obligation that is still in force tells a reader where the rule is today.

The owner ruled twice on 2026-10-03, in the decisions file of run `20261002-1233`:

> OWNER (2026-10-03) #1572-dated: the owner said 'review and repoint the decision if it hasn't been superceded, leave the evaluation as it was (assuming it notes the git sha it was evaluated at). This seems like it should be a general rule unless I am missing something?' The adjudicator for slice 4 checks the decision is not superseded and repoints it, and checks the evaluation names the commit it was evaluated at before leaving it. Whether this is a general rule is for the product owner to answer from the corpus at its next pass.

> OWNER (2026-10-03) #dated-records: the owner chose 'Yes, short decision record': a document that still governs (an accepted decision nothing replaced, an obligation, a spec, a guide) has a link to a moved rule changed to the new home with no change to what it claims; a record of a moment (an evaluation, a review, a probe run) stays as written, and its date stands for the moment when it names no commit. It is written as a short decision record, and the last slice of the spec 6 trim (#1572 slice 4b) applies it.

The second ruling replaces the commit condition of the first. An evaluation that names no commit stays as written, and its date marks its moment.

## Decision

**A document that still governs changes a link to a moved rule to the new home of the rule, and it changes no claim.** These documents govern:

- a decision at `current` that nothing supersedes,
- an obligation at any status, which includes a discharged one,
- a specification part, a subsystem spec and an interface contract,
- a guide (a how-to or a tutorial), the glossary and a README.

**A record of a moment stays as written.** These are records of a moment: an evaluation on either shelf, a review, a probe run, a probe result, and a superseded decision. Where the record names a commit, the commit marks its moment. Where it names none, its date marks its moment.

**A sentence that reports what another document said is history, and it stays.** A governing document that writes "spec 6 said" in the past tense describes an earlier text, and that earlier text did say it.

**A new link keeps the claim of its sentence.** Where the sentence quotes words, the quotation marks stay only if the new home holds those words. Where the home states the rule in other words, the author keeps the claim and the link, and the pull request lists the case. Nothing is dropped without a record.

**A link to a section that still states the credited rule stays.** A pointer section of spec 6 still places each rule in the engine. A sentence that credits spec 6 for that placement keeps its link.

## Consequences

- The governing links to the seven pointer sections of spec 6 now go to the homes. `engine/crates/cli/tests/spec_six_inbound_links.rs` fails on a governing document that links one of the seven again. It skips the records of a moment by path, and it skips a document at `status: superseded`. Its allow table holds the links whose sentence credits spec 6 for the placement that the pointer section still states. A second table holds the 34 links that the evaluations keep to the seven sections, so a repoint inside an evaluation also fails.
- The test cannot read a bare link to spec 6, or a link to a section that still states something. A reader checks those, against the rule above.
- An evaluation, a review or a probe run is never edited to follow a moved rule. A reader who follows its link to an older home reads a pointer there, and the pointer names the new home.
- "Names its commit" is not yet a fact that a check can read. [#1652](https://github.com/headwater-ai/headwater/issues/1652) carries that work, and it carries this rule into the shipped package, because an adopter meets the same choice when a rule moves.
- [HW-DR-0078](0078-a-recorded-terminal-demonstration-may-show-a-frozen-number-behind-a-recorded-on-date-marker.md) is the nearest earlier ruling. It lets a recording keep a frozen number when a recorded-on date stands beside it. This record applies the same idea to a link. A date or a commit marks a moment, and the record of that moment does not follow the corpus.
