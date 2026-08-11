# 13 — Open obligations

Every decision in [9 — The decision register](09-decisions.md) is closed, and each one left work behind. This file gathers that work in one place. Nothing here blocks a decision. Each item waits on an engine that nobody has built, on a corpus that nobody has adopted, or on a measurement that nobody has run.

This is where two principles are discharged for the project as a whole. [Principle 5](00-vision-and-scope.md#design-principles) requires explicit incompleteness, so a gap is named here rather than left for a reader to find. [Principle 11](00-vision-and-scope.md#design-principles) requires that efficacy is measured and never inherited. So a claim about what Headwater achieves is published as unmeasured until an instrument reports on it.

Four classes cover all of it. The first three are the classes that the register's own index named. The fourth holds the rest, so that no item falls between the classes.

## Unmeasured claims

Seventeen decisions carry a claim that no run supports. Each one names its instrument, and no instrument has run, because the engine does not exist. Each entry below keeps the claim and its instrument together, because a claim published without its instrument is not what principle 11 asks for.

- **[Q4 — Relation storage](09-decisions.md#q4--relation-storage).** The promotion fix should raise author-attributable edges without a rise in hand entry, and the assisted fraction plus the audit report are the instruments.
- **[Q5 — Voice checking depth](09-decisions.md#q5--voice-checking-depth).** [Spec 8](08-design-departures.md) calls declarative voice "mechanically detectable at useful precision". The instrument is a run of the three categories over this corpus, with an adjudicated sample of at least 50 findings each. The measurement that closed Q5 covered three ASD-STE100 structural rules, used as proxies. Nobody has measured `future_intent`, `change_narration` or `phased_rollout`, which are the categories that the declarative regime forbids, because no implementation of them exists.
- **[Q6 — Where the corpus graph lives at rest](09-decisions.md#q6--where-the-corpus-graph-lives-at-rest).** The rebuild-and-cache design is measured at spike scale, on generated documents. A real corpus and a real harvesting tier are the instruments, and neither exists yet.
- **[Q7 — Scope of the MCP surface](09-decisions.md#q7--scope-of-the-mcp-surface).** Working-tree write tools should raise the assisted fraction ([spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric)), and that metric is the instrument.
- **[Q8 — Probe cost and cadence](09-decisions.md#q8--probe-cost-and-cadence).** A closed-set expectation should make a probe verdict reproducible under grading, and `generate --check` over a result document is the instrument.
- **[Q9 — Multi-repository corpora](09-decisions.md#q9--multi-repository-corpora).** Harvest should keep a solution-tier route query inside the same 100 ms budget, and route latency at that tier is the instrument.
- **[Q11 — License and distribution posture](09-decisions.md#q11--license-and-distribution-posture).** Permissive terms should remove a step before a trial. The instrument is a count of adopters who report the terms as the reason that they stopped. That count needs a public channel, which does not exist yet.
- **[Q12 — Migration path for an existing corpus](09-decisions.md#q12--migration-path-for-an-existing-corpus).** An adoption payload should shrink. The instrument is its remaining pair count over time, against the fraction of payloads that reach zero before the expiry. This repository ran the pattern in miniature. `.ste-lint-baseline.json` grandfathered 65 violations when the check landed, and it holds 14 today. That number is weaker than it looks. Most of the fall came from prose that a later commit rewrote for other reasons, and from a defect in the checker. The baseline also has neither an owner nor an expiry. It shows that a text-keyed ratchet does not run backwards. It shows nothing about whether anyone works a debt list.
- **[Q13 — LinkML and SHACL as substrate](09-decisions.md#q13--linkml-and-shacl-as-substrate).** Emitted JSON Schema should lower the rate of invalid front matter that reaches a check. The instrument is the coverage report's finding rate for Shape-origin rules. If that rate does not move, the staging order is wrong and SHACL has as good a claim to the first slot.
- **[Q14 — Discovery surface](09-decisions.md#q14--discovery-surface).** A descriptor should let a cold agent reach a governing document that it otherwise misses, and the Discovery and Navigability probe categories are the instrument.
- **[Q15 — A synthesized content tier](09-decisions.md#q15--a-synthesized-content-tier).** Asserted content should move to `accepted` rather than accumulate, and the promotion rate against the asserted count is the instrument. If it never moves, the tier is a dumping ground and the honest response is to say so.
- **[Q16 — Public presence](09-decisions.md#q16--public-presence).** A generated site should let a reader answer "is this for me?" without reading the specification. The Discovery probe category is the instrument for the machine half. The human half has no instrument at all, which is worth a statement rather than a silence.
- **[Q17 — Governed access and the solution layer](09-decisions.md#q17--governed-access-and-the-solution-layer).** A `counted` tombstone should stop an agent reporting absence with confidence, and a probe over a withheld answer is the instrument.
- **[Q18 — Recording adjudicated disagreements](09-decisions.md#q18--recording-adjudicated-disagreements).** An agent that meets the losing document first should reach the adjudication, and a probe over a settled pair is the instrument.
- **[Q19 — Inbound integration](09-decisions.md#q19--inbound-integration-an-external-system-of-record).** Imported edges should not decay faster than scaffolded ones, and staleness by `created_by` in `taxonomy audit` is the instrument.
- **[Q20 — Where scent lives](09-decisions.md#q20--where-scent-lives).** A cue should raise traversal precision over the summary fallback, and a paired campaign over one corpus is the instrument. If it does not, the cue is authoring cost with no scent gain. The honest response is then to remove it rather than to make it longer.
- **[Q21 — Terminological succession](09-decisions.md#q21--terminological-succession-and-validity-under-merge).** Publishing the read set should let a gate skip a full re-run on most merges. The instrument is the fraction of merges whose read set the other side never touched.

## What waits on a first adopter

Nine items, and each one is data or a deferred component. So the first real adopter is the evidence, rather than a further argument.

- **The bundle set** ([Q3](09-decisions.md#q3--how-much-of-the-default-taxonomy-ships-in-the-box)). It is a guess about how adopters cluster, and no adopter exists yet. It is data in a package, so the first real adopter revises it at the cost of a release.
- **A second export profile** ([Q17](09-decisions.md#q17--governed-access-and-the-solution-layer)). Whether any corpus ever needs one. The first release ships one profile, unfiltered, and a real adopter with a real second audience is what builds the rest.
- **Vendoring or reference at a solution corpus** ([Q9](09-decisions.md#q9--multi-repository-corpora)). Whether a solution corpus vendors each source export or references it. A vendored copy keeps checks offline and grows the repository, and a reference does the reverse. The size of one real harvest is the evidence that closes it, and no such tier exists yet.
- **A named consumer for emitters 3 through 6** ([Q13](09-decisions.md#q13--linkml-and-shacl-as-substrate)). None exists, and that is the trigger rather than an oversight. If none appears, four emitters are never written and nothing upstream changes.
- **The transcription projection** ([Q19](09-decisions.md#q19--inbound-integration-an-external-system-of-record)). Whether it ships at all, because no adopter has asked for imported text.
- **Cue volume** ([Q20](09-decisions.md#q20--where-scent-lives)). Whether any corpus authors enough cues to grade, because no adopter exists and this repository has not tried.
- **The retired-term lexicon** ([Q21](09-decisions.md#q21--terminological-succession-and-validity-under-merge)). Whether any corpus other than this one needs one at all.
- **Probe kind placement** ([Q8](09-decisions.md#q8--probe-cost-and-cadence)). Whether the probe kind ships in the base package or in a bundle, which is a [Q3](09-decisions.md#q3--how-much-of-the-default-taxonomy-ships-in-the-box) clustering question.
- **The operational shape of a hosted server** ([Q7](09-decisions.md#q7--scope-of-the-mcp-surface) and [Q17](09-decisions.md#q17--governed-access-and-the-solution-layer)). Who runs it, and how it is deployed. Both decisions hold the question, and neither answers it. [Q11](09-decisions.md#q11--license-and-distribution-posture) records that this surface and the probe harness are the only two places in the specification where a commercial tier could sit.

## Design work that nothing blocks

**The `$`-reference sublanguage** ([Q2](09-decisions.md#q2--schema-format)). It has three uses and no grammar, and it needs one definition before the meta-schema ships. No adopter and no measurement gates this one.

## What else each decision left open

The three classes above do not exhaust the eighteen decisions that carry open work. The rest sits here, under the decision that produced each item.

- **[Q4 — Relation storage](09-decisions.md#q4--relation-storage).** The friction signal survives in a narrower form. If authors still declare the same link twice at a rate the fix does not absorb, this ruling is wrong and the annotation question returns.
- **[Q8 — Probe cost and cadence](09-decisions.md#q8--probe-cost-and-cadence).** How a probe reaches a harvesting tier, which has no tier to try it on. The retention policy for transcripts, whose size nobody has measured. Whether any adopter ever pays for a campaign is a business-model question, and [Q11](09-decisions.md#q11--license-and-distribution-posture) places it rather than answers it.
- **[Q9 — Multi-repository corpora](09-decisions.md#q9--multi-repository-corpora).** Whether a harvesting tier owes conformance rules of its own, because `headwater conformance` evaluates one repository.
- **[Q11 — License and distribution posture](09-decisions.md#q11--license-and-distribution-posture).** Where a commercial tier could sit, which that entry holds rather than answers. The trademark position is also deliberately thin, and a registration or a transfer would change it.
- **[Q15 — A synthesized content tier](09-decisions.md#q15--a-synthesized-content-tier).** Whether anything is ever promoted.
- **[Q16 — Public presence](09-decisions.md#q16--public-presence).** Whether the four documentation modes of Diátaxis are the right kind set for a documentation-site bundle. That is a [Q3](09-decisions.md#q3--how-much-of-the-default-taxonomy-ships-in-the-box) clustering question, and it is data in a package.
- **[Q17 — Governed access and the solution layer](09-decisions.md#q17--governed-access-and-the-solution-layer).** Whether a withheld anchor needs a class beside its count.
- **[Q18 — Recording adjudicated disagreements](09-decisions.md#q18--recording-adjudicated-disagreements).** Whether an adjudication is ever partial, with one document governing one axis and another governing a second. Today that is two edges, or one document whose prose carries the split. No corpus shows the need.
- **[Q19 — Inbound integration](09-decisions.md#q19--inbound-integration-an-external-system-of-record).** The size of a committed snapshot with full requirement text, which is the same question that [Q9](09-decisions.md#q9--multi-repository-corpora) holds about a vendored source export.
- **[Q21 — Terminological succession](09-decisions.md#q21--terminological-succession-and-validity-under-merge).** Whether the read set of a real corpus is small enough that publishing it is free.

## A human maintains this list by hand

The [first-run walkthrough](../evaluations/default-taxonomy-first-run.md#five-first-runs) found the defect that this file demonstrates. Each item above is really a decision document with a state, a date, edges to other items, and citations to its evidence. One file cannot carry that, so a human keeps the four lists aligned by hand. [Principle 2](00-vision-and-scope.md#design-principles) calls that a defect, and this file inherits the register index's position as the most-edited artifact in the repository.

The remedy is ordinary corpus work rather than a design change. An engine that governs this corpus derives these lists from the entries themselves. Until then the defect is stated here, which is the treatment that [principle 5](00-vision-and-scope.md#design-principles) prescribes for a known gap.
