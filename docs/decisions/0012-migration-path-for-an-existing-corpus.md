---
id: HW-DR-0012
title: Q12 — Migration path for an existing corpus
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: Adoption is a migration from no taxonomy, and `headwater infer` computes the adoption payload.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-EVAL-first-contact
---

# Q12 — Migration path for an existing corpus

## Context

One [evaluation](../evaluations/first-contact.md) settles this with [Q11](0011-license-and-distribution-posture.md) and [Q16](0016-public-presence.md). The between-majors half closed earlier. What stayed open was first contact: a corpus that was never valid. The migration state appears not to cover it, because that state is defined against a known-good starting point.

**The premise of the open half is nearly right, and one field is the whole of it.** [Spec 7](../spec/07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states) records a migration state with a from-version, a to-version, an owner, an expiry, and the open task list. Only the from-version refers to the prior state. The `(document, rule)` grain, the owner, the expiry, and the counted-visible-never-blocking posture are all defined against the new schema. So the from-version becomes optional, and nothing else changes.

## Decision

Adoption is a migration from no taxonomy ([spec 7](../spec/07-distribution-and-federation.md#first-contact-adoption-is-a-migration-from-no-taxonomy)). Before `headwater init` a corpus is governed by nothing, so every document is trivially valid. The findings that the proposed taxonomy raises over the existing tree are therefore the migration payload of that taxonomy's first version. A publisher computes a payload from the diff between two majors. On first contact, `infer` computes it from the diff between nothing and one.

## Consequences

That is one mechanism where the open-question entry expected two. It also gives an on-ramp four properties that no grandfathering file has. A pair grain, an owner, an expiry, and a line in the coverage report.

**`headwater infer` emits three artifacts, and all three come from one read of the tree.** The overlay, which [Q3](0003-how-much-of-the-default-taxonomy-ships-in-the-box.md) settled. The report of what does not fit, which the entry asked for. And the **adoption payload**, which is what closes first contact. `infer` cannot make a corpus valid by weakening the base, and that is structural rather than a rule to enforce. Q3 made bundles add-only, and an add-only overlay has no operation that removes a base rule.

**`--since <ref>` as a gate is refused, and this is the largest correction to the leaning.** Three arguments, and the third is that the word is taken. A flag that decides which findings count makes two runs over one tree disagree. [Spec 6](../spec/06-engine-architecture.md#implementation-constraints) forbids that, and [Q6](0006-where-the-corpus-graph-lives-at-rest.md) already refused the same shape for the cache. It also turns an unchecked document into an unreported one, against the coverage obligations of [spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for). And `headwater check --changed-only` already exists, as the performance scope that pays for a 200 ms hook. What the adopter wanted is the payload. A green build on day one, and each item of debt carries a name, an expiry, and a line in coverage.

**No threshold ever converts an accounting into a silence.** RuboCop excludes offending files one at a time until a limit, and past that limit it disables the rule entirely. The default limit is 15 files. First contact is the one moment at which every rule exceeds any such limit. A threshold of that shape would thus switch off most of the rule set and report the result as green ([spec 11 §S.5](../spec/11-adjacent-work.md#s5-grandfathering-has-a-scale-at-which-it-lies)). An adoption payload declares no such threshold. The honest cost is that the payload is large on a large corpus, and it sits in the lock, which is committed and reviewed.

**Every run reports how many pairs remain.** ESLint fails a run that carries a suppression which no longer matches, and that is the property `.ste-lint-baseline.json` lacked. Headwater does not need the failure, because a pair that passes is a task that closes and the expiry forces the conversation. It needs the number, beside coverage, so that a payload which is not shrinking is visible long before its expiry.

**The observed shape of gradual adoption is a declaration in the tree, and we already have it.** Sorbet reads a strictness level from a sigil at the top of each file. The default for an unmarked file is the level at which almost nothing fires. Two properties do the work. The level is committed and diffable, so no invocation changes a verdict, and the untouched file stays in the denominator. Headwater reached both from the other direction. The regimes are declared per kind and per shelf, and Q3 already declared `controlled: none` on the base package for this reason.
