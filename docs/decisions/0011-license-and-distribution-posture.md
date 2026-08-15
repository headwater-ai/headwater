---
id: HW-DR-0011
title: Q11 — License and distribution posture
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: Apache-2.0 for the engine, the library, the base package and the bundles, ratified by the owner on 2026-08-11.
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

# Q11 — License and distribution posture

## Context

No argument from the design closed this one, and that is why it closed last. A license states what the owner intends for the project, and a register of design decisions cannot make such a statement. What the design could do was narrow the field and prepare the decision. One [evaluation](../evaluations/first-contact.md) did that with [Q12](0012-migration-path-for-an-existing-corpus.md) and [Q16](0016-public-presence.md), because the three describe one path that one outsider walks.

What the evaluation does instead is narrow the field, and it narrows it much further than the open-question entry assumed. Eight rulings already made refuse two of the four postures outright, and a third for the library.

| Posture | State | What decides it |
|---|---|---|
| Permissive | Available | Nothing refuses it |
| Weak copyleft | Available | Nothing refuses it, and it taxes the embedding requirement |
| Strong or network copyleft | Refused for the library | [Spec 6](../spec/06-engine-architecture.md#library) requires in-process embedding, and [Q1](0001-implementation-language.md) chose the language for that requirement alone. Obligations that reach a linked work reach software that the adopter owns |
| Source-available | Refused | [Principle 9](../spec/00-vision-and-scope.md#design-principles) builds integrations first, and [Q13](0013-linkml-and-shacl-as-substrate.md) makes an external consumer the trigger for four emitters. A restriction on a competing offering makes every such consumer ask a lawyer about itself |
| Internal-only | Refused | Emitters 3 through 6 then have no trigger and never ship. [Principle 8](../spec/00-vision-and-scope.md#design-principles) cannot show its public half. [Q17](0017-governed-access-and-the-solution-layer.md)'s disclosure process has nobody to disclose to |

**One half of this question the specification decides on its own, and it decides against the framing.** The entry treats the terms of the base package as a related preference. It is a constraint. [Q3](0003-how-much-of-the-default-taxonomy-ships-in-the-box.md) ships a base plus add-only bundles, and [Q2](0002-schema-format.md) makes an overlay a patch whose resolution contains base content. So an adopter's resolved taxonomy and lock contain the base. A copyleft or share-alike term on the base therefore propagates into an artifact that [spec 0](../spec/00-vision-and-scope.md#who-this-is-for) promises is the adopter's own. **The base package and the bundles may impose nothing on a derived taxonomy** ([spec 7](../spec/07-distribution-and-federation.md#publishing)).

## Decision

The owner ratified this on 2026-08-11, and the terms are **Apache-2.0** for the engine, the library, the base package and the bundles.

It satisfies the embedding requirement. It grants a patent license that MIT does not, and it reserves the trademark that [Q10](0010-naming.md) fixed as the `https://w3id.org/headwater/` namespace. [Q17](0017-governed-access-and-the-solution-layer.md) is the reason the patent clause matters, because this tool emits artifacts into other people's compliance pipelines. Creative Commons Attribution for the doctrine prose, on the boundary that `contents.doctrine` already draws. The Developer Certificate of Origin for contributions, and no contributor license agreement. State that reason positively. The one power a contributor agreement adds is the power to change the terms later without asking, and the recommendation is not to want it.

## Consequences

**What ratification produced.** The open-question entry named six artifacts and said that it stayed pending until they existed. All six exist.

1. [`LICENSE`](../../LICENSE) carries the Apache-2.0 text without modification. Source files carry `SPDX-License-Identifier: Apache-2.0` on the first line that permits a comment, and [`CONTRIBUTING.md`](../../CONTRIBUTING.md) states that convention. The identifier is the whole header, because a per-file copyright line goes stale and the commit history does not.
2. [`NOTICE`](../../NOTICE) carries the project-level statement. [`CONTRIBUTING.md`](../../CONTRIBUTING.md) requires a `Signed-off-by` line under the Developer Certificate of Origin, and states the reason for no contributor agreement positively.
3. [`SECURITY.md`](../../SECURITY.md) carries the coordinated-disclosure process that [Q17](0017-governed-access-and-the-solution-layer.md) obliges. It reports that no engine has shipped, so no released artifact can carry a defect yet. It also states what is out of scope by design, which is a reader who can clone a repository and then reads it.
4. [`docs/LICENSE`](../LICENSE) puts the prose under Creative Commons Attribution 4.0. It keeps the embedded code samples under Apache-2.0, so that copying an example carries no attribution obligation.
5. [`TRADEMARKS.md`](../../TRADEMARKS.md) states the position, and the position is that there is no registered mark. Apache-2.0 section 6 grants no trademark rights, and that reservation was one of the two reasons to prefer these terms to MIT.
6. The date is 2026-08-11. A term with no date cannot later be shown to have changed, and a visible change is the whole value of the commitment.

**What the specification could not decide, kept on the record.** Every constraint above rules an option out. Not one of them rules exactly one option in. Between MIT and Apache-2.0 the design is indifferent except for the patent and trademark clauses. Between Apache-2.0 and a weak copyleft it is nearly indifferent, and the choice turns on whether the owner wants engine improvements to return. The owner chose the permissive side of that question, and no argument in the register made the choice.

**Where money could sit, because the question arrives with this one.** [Q7](0007-scope-of-the-mcp-surface.md) and [Q17](0017-governed-access-and-the-solution-layer.md) both leave open what a hosted server is operationally, and [Q8](0008-probe-cost-and-cadence.md) prices a campaign run at real money. Those two surfaces are the only places in this specification where a commercial tier could sit, and neither is specified. The closest observed analog draws its line there: [Vale](../spec/00-vision-and-scope.md#what-we-do-not-build) is MIT, and its author sells a hosted authoring layer beside it ([spec 11 §S.3](../spec/11-adjacent-work.md#s3-where-the-closest-analog-draws-the-commercial-line)).

**The link to [Q7](0007-scope-of-the-mcp-surface.md) that this entry used to name is gone.** Q7 closed on the ruling that a landed write never ships, and no license term changes that.
