---
id: HW-OBL-0167
status: current
status_since: 2026-09-06
summary: "The ruleset on the default branch requires zero approving reviews, which is the posture HW-DR-0034 named as the thing that erases its line. The corpus owes a ruling."
last_verified: 2026-09-06
title: "The default branch requires no approving review, which is the condition HW-DR-0034 named as reopening it"
waiting_on: ruling
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0034
---

# The default branch requires no approving review, which is the condition HW-DR-0034 named as reopening it

## Context

[HW-DR-0034](../decisions/0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md) rules that acceptance is the merge onto `main`, and that a pull request is the review the merge records. Its last paragraph names what reopens it. A branch-protection posture that lets a document reach `main` with no review erases the line the ruling draws. That paragraph also records that nothing in this engine enforces the posture, so the ruling rests on the process this repository runs.

**The posture is now declared, and it asks for no review.** This repository carries one active ruleset over the default branch, named `Protect main`. `gh api repos/headwater-ai/headwater/rulesets/20627391` reports a `pull_request` rule that carries `required_approving_review_count: 0`. It reports a `required_status_checks` rule naming `Engine tests`, `headwater check (advisory)` and `Workers Builds: headwater`. Measured on 2026-09-06.

**A pull request is required and an approval is not.** So a document reaches `main` through a request that three machines checked and no person read. The three checks read what a rule reads, and [HW-OBL-0030](0030-the-provenance-block-belongs-to-the-engine-and-nothing-states.md) records that no rule of this engine reads a field of a provenance block. So the stamp that HW-DR-0034 gives its meaning to is the one value nothing on that path examines.

**The condition is met and the ruling is not wrong.** Zero required approvals is a defensible setting on a repository with one maintainer. A repository with one maintainer cannot ask a second person for an approval. This record states that the named condition fired. It does not state that the setting is a defect.

## Obligation

**The corpus owes a ruling on what the met condition costs.** HW-DR-0034 named this posture in advance and said what it would erase. It did not say what a reader does once the posture is in force. So the corpus holds a ruling with a fired condition and no successor.

**Three answers are open and this record proposes none.** The owner may raise the required count to one. The owner may hold it at zero and record why a solo repository draws the line somewhere else. The owner may rewrite the reopening paragraph, because the condition it names reads a setting rather than the act it cares about.

**The shape this record carries is the shape the ruling permits for the case.** HW-DR-0034 admits an agent stamp where a named human reads the request before the merge, and it admits `warrant: asserted` with no stamp where nobody will. An autonomous run that merges its own request is the second case, so this record carries `asserted` and no acceptor.

## Discharge

**This record discharges when the owner rules on the posture.** The ruling either moves the branch-protection setting or states why zero approvals holds the line HW-DR-0034 draws.

**What does not discharge this.** A change to the ruleset that no record states, because the next reader learns nothing from a setting. A reading of the ruleset by an agent, which is what produced this record rather than what settles it.

**What this record does not carry.** Whether the decision of HW-DR-0034 is right, which stands and which this record reads rather than reopens. The documents that hold the acceptance stamp back, which the ruling already reaches and asks nobody to rewrite. Every other clause of the ruleset, which no ruling of this corpus reads.
