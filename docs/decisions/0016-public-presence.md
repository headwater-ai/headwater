---
id: HW-DR-0016
title: Q16 — Public presence
status: current
status_since: 2026-08-11
last_verified: 2026-08-17
summary: Registration needs a channel with an obliged reader, a directory of corpora is refused, and the site is a projection of this corpus.
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

# Q16 — Public presence

## Context

One [evaluation](../evaluations/first-contact.md) settles this with [Q11](0011-license-and-distribution-posture.md) and [Q12](0012-migration-path-for-an-existing-corpus.md). The three describe one path. An outsider hears the name, decides whether it is worth an hour, and checks the terms. Then they point the tool at a corpus that nobody wrote to any of this.

**Registration has a definition, and the definition is why no file can perform it.** [Q14](0014-discovery-surface.md) refused registration because no file inside a corpus makes that corpus findable, which is a symptom. The reason is that **registration is an act of publication, and a publication needs a channel whose reader is already obliged to read it.** `llms.txt` fails at a larger radius for the same reason. About 137,000 domains publish one, 97% of the valid files went unread for a month, and no provider is obliged to read one ([HW-EVAL-adjacent-work §O.3](../evaluations/adjacent-work.md#o3-llmstxt-is-the-measured-failure-of-a-descriptor-with-no-obliged-reader)).

## Decision

Registration closes with no new machinery, because two obliged channels already exist. A taxonomy package goes to a registry that a resolver must read to install it ([spec 7](../spec/07-distribution-and-federation.md#publishing)), and a resolver is an obliged reader by construction. A rendered page already carries a link relation to the served descriptor ([spec 7](../spec/07-distribution-and-federation.md#arriving-at-a-corpus-cold)). Registration is the publisher's own act in a channel that exists, and Headwater supplies only the payload, which [Q14](0014-discovery-surface.md) already settled.

**A registry or directory of Headwater corpora is refused, not deferred** ([spec 0](../spec/00-vision-and-scope.md#what-we-do-not-build)). [Q9](0009-multi-repository-corpora.md) refused query fan-out, and Q14 refused a reserved path at the root of an origin. A central directory is both refusals at the largest radius, and it adds one that neither has. It would be the single piece of Headwater infrastructure that must stay online for discovery to work. The [non-negotiables](../spec/00-vision-and-scope.md#non-negotiables) of this system include an offline run with the same result as CI. The cost of the refusal is real: nobody can enumerate Headwater corpora. An organization that wants its own enumerated builds a solution corpus and pins them, which is enumeration where somebody owns the list.

**The site is a projection of this corpus, and no generator is built.** [Spec 0](../spec/00-vision-and-scope.md#what-we-do-not-build) already refuses to build a renderer and names the alternative, and [spec 0](../spec/00-vision-and-scope.md#what-we-build) item 3 already lists site navigation among the projections. So Headwater emits the navigation and the content, and a third-party static-site generator renders them. Nothing here is built before the engine exists, and when it does the site is an emitter target and a projection. Both mechanisms have owners already.

## Consequences

**The sitemap ran, and it measures the specification rather than the marketing.** [LeanCTX](../evaluations/adjacent-work.md#i5-the-presentation-is-the-lesson) is the standard to match, and it is useful because it is one developer's project rather than a large company. Its facets are questions this specification should already answer.

| Facet | What answers it today | Verdict |
|---|---|---|
| How it works, architecture | [Spec 6](../spec/06-engine-architecture.md), [spec 1](../spec/01-conceptual-model.md), [spec 12](../spec/12-check-layer.md) | Answered |
| Benchmarks, metrics | Nothing. Every efficacy claim is marked unmeasured | Empty, and it stays empty |
| Comparisons | [HW-EVAL-adjacent-work](../evaluations/adjacent-work.md), [spec 8](../spec/08-design-departures.md), and spec 0's table of what we do not build | Answered, and the strongest row |
| Use cases | The five adopters of the [first-run walkthrough](../evaluations/default-taxonomy-first-run.md#five-first-runs) | Answered, from an evaluation |
| Compatibility, integrations | [Q13](0013-linkml-and-shacl-as-substrate.md)'s six emitters, of which two ship | Answered, and the answer is two |
| Docs, getting started | [Spec 3](../spec/03-authoring-and-lifecycle.md), the [interview](../spec/07-distribution-and-federation.md#the-interview) and the [tutorial](../tutorials/your-first-governed-corpus.md) | Answered as a document, and the page that would carry it waits on the site |
| Pricing, enterprise, consulting | [Q11](0011-license-and-distribution-posture.md) | Partial. The terms are Apache-2.0, and where a commercial tier sits is still open |
| Compliance, audits, self-assessment | [Spec 4](../spec/04-assurance-model.md)'s obligation and gap registers, and [spec 6](../spec/06-engine-architecture.md#what-a-filtered-export-claims-and-what-it-does-not)'s one claim and five non-claims | Answered |
| Changelog, community, open-source posture | [Q11](0011-license-and-distribution-posture.md). There is no changelog and no community | Partial. The posture is stated, and neither artifact exists |
| `llms.txt`, AI-crawler `robots.txt` | Cheap to emit, and measurably unread | Ship it, and count it as nothing |

**One row moved on 2026-08-17, and the reason it carried had expired.** The getting-started row read *no quickstart, because there is nothing to start*. That was true when this entry closed. It stopped being true when M1 and M2 shipped `headwater check`, `headwater taxonomy validate` and `headwater taxonomy resolve`. [The tutorial](../tutorials/your-first-governed-corpus.md) now takes a reader from an empty directory to a passing strict run. The row is answered as a document and no further. This entry rules the site a projection, and no item of the plan builds one.

**What the forcing function found is uncomfortable and correct.** The largest hole in the public story is the one that this project has decided it may not fill. The benchmark row is empty because [principle 11](../spec/00-vision-and-scope.md#design-principles) forbids a number that no run produced, and it stays empty until a campaign runs. The pressure to relax that will arrive exactly when the site does.

**The second finding is quieter.** The comparison row is the strongest asset here, because [HW-EVAL-adjacent-work](../evaluations/adjacent-work.md) already carries the arguments against Headwater. OpenGEO declines the standards stack for a neighboring problem, and TrustGraph ships the opposite mechanism. Vale is the closest analog and is in another language. [§M](../evaluations/adjacent-work.md#m--what-the-survey-shows-as-a-whole-convergence-is-not-evidence) states that the survey licenses no efficacy conclusion at all. A comparison page that carries its own counter-evidence is unusual enough to be the difference, and it is already written.

**"Honest before impressive" becomes a mechanism, because the site is generated.** §I.4 records claims that move between README versions as the thing which made an otherwise strong project harder to trust. An intention does not prevent that. A generated page does. **Every number on the site comes from the evidence register, and a claim with no instrument is generated as unmeasured** ([spec 4](../spec/04-assurance-model.md#the-systems-own-assurance)). A hand-written number on the site is then a finding, in the way that a hand-edited shelf index is. And the self-assessment states that it is self-published, which is §I.4's standard applied to ourselves.

**The leaning on timing survives, with a derived trigger.** The site ships when the engine ships. A site supplies observability, and it cannot supply trialability. The diffusion literature is clear that neither substitutes for the other ([HW-EVAL-adjacent-work §S.6](../evaluations/adjacent-work.md#s6-what-predicts-adoption-and-which-entry-supplies-each-attribute)). So a site that describes a tool nobody can run spends the first impression and offers no next step. The sitemap is drafted above, which is what the leaning asked for.
