---
id: HW-EVAL-warrant-and-adjudication
status: current
status_since: 2026-08-10
last_verified: 2026-08-10
summary: The Q15, Q19 and Q18 evidence, which is what stands behind a document, who vouched for it, and how a disagreement is settled.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: evaluate+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cited_by:
    - HW-REG-decisions
    - HW-SPEC-conceptual-model
    - HW-SPEC-taxonomy-model
    - HW-SPEC-authoring-and-lifecycle
    - HW-SPEC-assurance-model
    - HW-SPEC-ai-integration
    - HW-SPEC-engine-architecture
    - HW-SPEC-distribution-and-federation
    - HW-SPEC-adjacent-work
    - HW-SPEC-check-layer
    - HW-SPEC-glossary
---

# Warrant — what stands behind a document, and who vouched for it

This evaluation closes three entries at once: [Q15](../spec/09-open-questions.md#q15--a-synthesized-content-tier) (a synthesized content tier), [Q19](../spec/09-open-questions.md#q19--inbound-integration-an-external-system-of-record) (inbound integration with an external system of record), and [Q18](../spec/09-open-questions.md#q18--recording-adjudicated-disagreements) (recording adjudicated disagreements).

The house pattern is one evaluation per question. The [graph group](graph-export-and-federation.md) departed from it because three entries were one question at three radii. The [serving-boundary group](the-serving-boundary.md) departed from it because three entries described one boundary from three sides. This group departs from it for a third reason, and the reason is the first finding below.

## Why the three did not close separately

The three entries already say that they belong together, and each one says it in a sentence that names the wrong thing.

Q15 says that Q19's imported text "*is* regenerable, against a pinned upstream snapshot, so it is not synthesized", and adds that "the boundary that this question draws is a verification method, not an author". Q15 then spends the rest of its text on the author. Q19 says that imported text "fails the Q15 definition of synthesized" and is "not a projection in the spec-6 sense either", so it asks for a fourth tier. Q18 says that it "belongs beside Q15's provenance questions", because "an adjudication without a named adjudicator is a rank with extra steps".

Each entry reaches for the same missing concept and none of them names it. The concept is this: **what licenses a reader to move from "this document says X" to "the corpus asserts X"?** That license is what the three entries are arguing about, and the specification has never had a word for it.

The word is **warrant**, and it is borrowed rather than invented. Toulmin's model of argument separates the claim, the data, and the **warrant** that licenses the step from one to the other. A corpus is in the same position. A document is data. What the corpus asserts is a claim. The warrant is the mechanism that connects them, and today the specification has exactly two of those mechanisms and no name for the category.

Once the category has a name, all three entries resolve as instances of it.

- **Q15** asks which warrants exist, and what content with the weakest one may do.
- **Q19** asks where imported text sits, and the answer is a warrant value that the standards already name.
- **Q18** asks where an adjudication lives, and the answer follows from what warrant an adjudication needs.

Three documents would have derived the same category three times and drifted on the third.

## What the specification already fixed

Fifteen rulings constrain this evaluation, and it may not revisit any of them.

**Acceptance is a human act, and the record names the human.** `accepted_by` carries it, and an agent may draft ([spec 3](../spec/03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed)). [Spec 5](../spec/05-ai-integration.md#what-we-do-not-do) forbids an agent-authored document merged without review.

**One source of truth per fact, and many paths to it** ([principle 2](../spec/00-vision-and-scope.md#design-principles)).

**Derived artifacts are regenerable, and the system checks them against regeneration** ([principle 3](../spec/00-vision-and-scope.md#design-principles)). A projection is generated, checked, and declared ([spec 1](../spec/01-conceptual-model.md#projections)).

**Explicit incompleteness** ([principle 5](../spec/00-vision-and-scope.md#design-principles)), and its operational form: visible incompleteness beats apparent completeness ([spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)).

**Every rule earns its place** ([principle 6](../spec/00-vision-and-scope.md#design-principles)). A facet with one value everywhere, on which no check varies, fails that test ([spec 2](../spec/02-taxonomy-model.md#language-is-declared-not-assumed)).

**Degrade toward the cheaper error, and say which one that is** ([principle 7](../spec/00-vision-and-scope.md#design-principles)).

**A control walks the promotion path when both of its error classes are recoverable** ([spec 4](../spec/04-assurance-model.md#where-promotion-does-not-apply)).

**An instance attribute takes a facet's value space and is never a reference.** An edge that must point at a node is a request to make the edge a node, and Q18 owns that change ([Q4](../spec/09-open-questions.md#q4--relation-storage), [evaluation](relation-storage.md)).

**A facet value is never a reference either**, and the reason is that a second ungoverned edge mechanism wins under deadline pressure ([spec 1](../spec/01-conceptual-model.md#facet)).

**Reading precedence is derived, never declared** ([spec 2](../spec/02-taxonomy-model.md#reading-precedence-is-derived)). The nucleus governs, the successor governs, and a governance source governs. Everything else carries no reading order.

**Evidence has three honest states**, and `evidenced` obliges a pointer that resolves to an external auditable artifact ([spec 3](../spec/03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two)).

**A coherence obligation becomes a cohesion obligation the moment the judgment that it needs is recorded as data** ([spec 4](../spec/04-assurance-model.md#declaration-moves-the-boundary)). The authority rank was cut because a scalar pre-answers a judgment ([spec 2](../spec/02-taxonomy-model.md#disagreement-is-adjudicated-not-ranked)).

**An anchor resolver reads repository content or a committed snapshot, and never a live service** ([spec 2](../spec/02-taxonomy-model.md#behavior-at-the-limits)). Exactly one resolver owns each anchor kind.

**The export is where a corpus decides what leaves it, and the filter is default-deny over classes** ([spec 6](../spec/06-engine-architecture.md#an-export-profile-carries-a-filter)). Every emitter declares a loss set, and every export run emits a projection census.

**The scaffolder is a correctness root**, because it manufactures edges that nobody reviews individually ([spec 12](../spec/12-check-layer.md#the-correctness-roots)).

Those fifteen settle more of the three questions than any of the three entries noticed. Two of them settle a question outright, and the entry that asked it cited neither.

## Prior art, and what practitioners shipped

[Principle 10](../spec/00-vision-and-scope.md#design-principles) asks for the research literature and at least one observed industry application. Eleven sources bear on this group. They are grouped by which of the three questions each one reaches.

### What a provenance standard records, and what it does not

**W3C PROV supplies three of the four warrant values and cannot express the fourth.** PROV models an **entity**, an **activity**, and an **agent**, and it relates them with `prov:wasGeneratedBy`, `prov:wasDerivedFrom`, and `prov:wasAttributedTo`. Its derivation subtypes are `prov:Revision`, `prov:Quotation`, and `prov:PrimarySource`. `prov:Quotation` is defined as the repeat of part or all of an entity by somebody who may not be its original author, which is Q19's imported requirement text under a standard name.

What PROV has no vocabulary for is endorsement. A PROV record says what happened to an entity and who was involved. It never says that a party stands behind the result. Headwater's `accepted_by` is precisely that addition, and [spec 2](../spec/02-taxonomy-model.md#lineage-aligns-with-prov) claims PROV alignment without recording that the alignment stops one field short.

That is the strongest single result in this section, because it contradicts an assumption that the three entries share. Each entry treats "record the provenance" as the whole answer. A provenance record is a record of derivation. A warrant is a record of who vouched, and the two are different fields with different failure modes.

PROV supplies one further point for Q18. A PROV record is itself an entity, so it may carry provenance of its own. An adjudication that names an adjudicator wants exactly that recursion, and the cheapest thing that already has it is a document.

**SPDX makes "nobody asserted anything" a value rather than an absence.** A license field in SPDX may hold `NONE` or `NOASSERTION`, and the two mean different things. `NONE` means that the document states there is no license. `NOASSERTION` means that no party made a claim either way. SPDX 3.0 goes further and puts `creationInfo` on every element, with no exception.

That settles a design question that would otherwise have been a coin toss. An unwarranted document carries a positive mark, and it is never the absence of a field. A standard with a very large installed base reached that conclusion for the same reason, which is that an absent field and an unknown value are indistinguishable to a consumer.

**C2PA measures what happens to a mark in transit.** Content Credentials bind a signed manifest to an asset, with assertions about capture, editing, and the use of generative tools. The specification's own threat model names manifest removal as a live case, and the ecosystem's answer is a durable binding through watermarking and fingerprinting. The reason is blunt: metadata that travels beside content gets stripped by ordinary tooling that never intended to strip it.

That sharpens the constraint that [Q17](../spec/09-open-questions.md#q17--governed-access-and-the-solution-layer) handed this group. Q15 proposed that an emitter which cannot carry the mark declares that in its loss set. A loss-set entry tells a consumer who reads it that the distinction is gone. It does nothing at all for the consumer who does not read it, and that consumer receives unwarranted content that looks vouched. C2PA's second contribution is the signer. A manifest names an accountable party and validates against a trust list, so a credential with no named party is decoration. That is Q18's own test, generalized past adjudication.

### What the largest maintained corpus does about content nobody checked

**Wikipedia runs a warrant model, and it has run one for two decades.** The core rule is verifiability: content must be attributable to a reliable published source, and the burden sits with the editor who adds the material. The companion rule forbids original research. The slogan that the community used for years was "verifiability, not truth", which is the same separation that this evaluation needs. The project does not ask whether a sentence is true. It asks what stands behind it.

Two details transfer directly.

**The response to machine-generated content was a mark, and the mark alone did not hold.** The community first tagged suspected model output with a template. In 2025 it added a speedy-deletion criterion for unreviewed model output, aimed at hallucinated citations and unedited chatbot artifacts. The trigger of that criterion is worth reading closely. It is not that a model wrote the text. It is that nobody reviewed it. The largest observed case of this problem separates the author from the warrant, and enforces on the warrant.

That is the reframing of Q15, confirmed by the case that Q15 did not cite.

**And the contradiction, which is the more useful half.** Most Wikipedia prose carries no inline citation. The policy demands attributability rather than attribution, and the encyclopedia is useful anyway. So a corpus full of unwarranted content is not worthless. It is useful in proportion to how cheaply a reader can check it, and the inline `citation needed` mark is what keeps that cost visible at the point of reading. That contradicts any ruling that would forbid unwarranted content, and it supports Q15's leaning to admit it and mark it.

One thing that Wikipedia does and this design declines: the `citation needed` mark sits inside a paragraph, at sub-document grain. [Q17](../spec/09-open-questions.md#q17--governed-access-and-the-solution-layer) already refused a filter that reaches inside a body, and the reasoning transfers. A warrant is per document.

**TrustGraph and Serena are the two observed cases that the entries already supply, and the survey confirms both.** [Spec 11 §J](../spec/11-adjacent-work.md#j-trustgraph--the-same-pitch-the-opposite-mechanism) records per-fact receipts of source, ingestion timestamp, and extraction method. Read against PROV, a receipt is a derivation record and not an endorsement record, so it supplies the *shape* of the provenance block and none of the warrant. It also works at per-fact grain, which is the grain that the paragraph above declines. [Spec 11 §L.5](../spec/11-adjacent-work.md#l5-onboarding-ships-the-synthesized-tier-and-marks-nothing) records the failure with no mark at all, in a tool with a very large installed base.

**Karpathy's LLM Wiki names its own control and the control is the wrong class.** [Spec 11 §F.2](../spec/11-adjacent-work.md#f2-karpathys-llm-wiki) records ingest, query, and lint as the operations. Lint is a cohesion mechanism. It finds orphans, undefined concepts, and contradictions between pages. It establishes nothing about whether any page is true, so the pattern that raised Q15 ships no warrant at all. That is a confirmation rather than a criticism: the entry was right that the tier is real and right that nothing in the pattern keeps it non-canonical.

### What requirements practice did about a store that somebody else owns

**Baselining is the pinned snapshot, named by the industry that invented the problem.** Requirements-management practice freezes an identified set of requirements as a baseline, and traceability is evaluated against the baseline rather than against the live set. [Spec 11 §K](../spec/11-adjacent-work.md#k-modern-requirements--the-first-candidate-where-the-arrow-reverses) records that the tool in question mints baselines as work items. So the pin that [Q9](../spec/09-open-questions.md#q9--multi-repository-corpora) generalized into one pattern with three instances is, for this instance, the upstream's own native concept.

**Suspect links are the drift mechanism, and they are better than what Q19 proposed.** DOORS-family tooling flags every trace link into a requirement as *suspect* when that requirement changes, and the flag clears only when a person confirms the link. Q19 proposed a change proposal against the snapshot, which is correct and too coarse. A snapshot advance that changes one requirement should not raise one proposal about the snapshot. It should raise a finding on each edge that pointed into the changed requirement, because that is where the person who can act will look. [Spec 4](../spec/04-assurance-model.md#absence-is-a-finding-class-of-its-own) already states the report-at-the-origin rule for participation expectations, and this is the same rule.

**ReqIF is a real OMG standard and it is the adopter's asset rather than the engine's.** ReqIF exists so that two requirements tools can exchange a requirement set without either one owning the format. That value accrues to an organization that wants to leave its current tool. It does not accrue to Headwater, which reads identities and text and writes nothing back. So ReqIF is a resolver's choice, and the engine that privileges it would be privileging a vendor's escape hatch in the core.

**Debian settles the redistribution question by segregating the archive.** Software whose terms Debian cannot pass on lives in a separate archive area, and a default installation carries only the free one. An administrator who wants the rest enables it explicitly, and the act is visible in a configuration file. Machine-readable per-file copyright records sit beside that, and embedded code copies must be declared.

That is the shape of the answer to Q19's fifth open point. Content whose terms the corpus does not control travels only by an explicit act that a reviewer sees. Headwater needs no new mechanism for it, because the [export filter is already default-deny over classes](../spec/06-engine-architecture.md#an-export-profile-carries-a-filter).

### The one source that argues against Q18's ruling, and then for it

**Legal citators put the treatment signal on the citing relationship, which is the edge.** Shepard's and KeyCite attach a treatment to the pair of cases: a later opinion overrules, distinguishes, criticizes, or follows an earlier one. That is an edge attribute, shipped for over a century, in the domain that has thought hardest about how a later judgment binds an earlier one. Read alone, it is a direct argument for Q18's leaning and against the ruling below.

Two further facts reverse it.

**The signal is an index over a document, and never the authority.** The treatment flag is derived by an editorial process from a published opinion. The opinion is the record, the flag is a lookup surface, and no practitioner treats the flag as the holding. That maps onto a distinction that this specification already makes everywhere. The opinion is the document. The flag is a projection.

**Two citators over the same case law disagree at a rate that researchers measured and published.** Comparisons of the two major services report substantial divergence in how they treat the same decisions. A derived scalar signal, produced by a third party who did not make the judgment, is not reliable enough to be the record. That is Q18's own sentence about the authority rank, confirmed by the largest deployed instance of the pattern.

**Retraction notices land on the same design, in a field with a century of practice.** A retraction in scholarly publishing is a separate, citable object with its own identifier, and it points at the work that it retracts. The retracted work is marked and kept, not deleted. Crossref carries the relationship as metadata between two registered items, and CrossMark exists so that a reader holding a copy can ask the publisher whether that copy is still current.

That last mechanism is worth taking whole. CrossMark solves the stripped-mark problem of C2PA in the opposite direction. It does not try to make a flag survive the copy. It puts a resolvable pointer to the authority in the copy, so that a reader can always go back and ask. A transcribed document should carry its pin for the same reason.

## The decision

### Every document carries a warrant, and there are four

A **warrant** is the mechanism by which the corpus can defend that a document is what it claims to be. Every document has exactly one, drawn from a closed set, recorded in the provenance block, and never inferred from an absent field.

| Warrant | What stands behind the content | What fails loudly when it is wrong |
|---|---|---|
| `accepted` | A named human accepted it | Nothing mechanical. A human re-reads, and `last_verified` records that they did |
| `regenerated` | It is a function of inputs inside the repository | `generate --check` on every run |
| `transcribed` | It is a byte-faithful copy of a pinned external snapshot | `generate --check` against the pin, plus the drift comparison when the pin advances |
| `asserted` | Nothing | Nothing |

Four observations make this a ruling rather than a table.

**The first three are three different mechanisms, and they are not ranked against one another.** A projection is not weaker than an accepted document. It restates one, and it inherits that document's standing through the derivation family that [spec 2](../spec/02-taxonomy-model.md#nuclearity) already declares. The only ordering that the design needs is that `asserted` sits below all three, and that ordering is not a matter of taste. It is the difference between a defect that some mechanism finds and a defect that no mechanism finds.

**The fourth value is the whole question, and Q15 asked it about the wrong field.** Q15 asks whether Headwater admits content that an agent synthesized. [Spec 3](../spec/03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed) already admits that, in full, and has since the first draft. `agency: agent` plus `drafted_by: claude-opus-5` plus a human in `accepted_by` is the ordinary agent-assisted document, and [spec 5](../spec/05-ai-integration.md) puts the machinery for it in the first release. So the agency field was never the boundary.

What Q15 actually names is content that **nobody accepted**, at a volume where acceptance does not scale. Serena's onboarding writes a directory of memories. TrustGraph extracts thousands of facts. The pitch of the tier is exactly that a human will not read them all. That is a claim about the warrant and not about the author, and the word "synthesized" pointed at the author. This is why the entry could not place Q19's imported text, which has a different author *and* a different source *and* the same question about who vouched.

**The answer is yes, admit it, and the enforcement moves from the author to the warrant.** Wikipedia reached the same placement under far heavier load. Its speedy-deletion criterion for model output fires on the absence of review rather than on the presence of a model, and its `citation needed` mark is what makes an unwarranted claim cheap to spot. Headwater admits `asserted` content, marks it positively, and constrains what it may do.

**A `warrant` is engine-significant, so the provenance block is the engine's.** Four values produce four different sets of required fields, four staleness behaviors, one edge rule, and one export rule. [Spec 3](../spec/03-authoring-and-lifecycle.md#front-matter-is-the-contract) currently lists provenance among the things that are "a taxonomy choice", and that sentence is now wrong. A taxonomy chooses which facets a kind carries. It does not choose whether `accepted_by` exists, because `accepted_by` is what enforces the human-acceptance boundary, and it does not choose the warrant vocabulary for the same reason.

That placement also answers the objection that a careful reader raises next. [Spec 2](../spec/02-taxonomy-model.md#language-is-declared-not-assumed) refuses a facet that holds one value everywhere and that no check varies on. A simple corpus has `accepted` documents and `regenerated` projections and nothing else, so is the warrant such a facet? No, and for the stated reason: every check below varies on it. The value that a document carries decides which fields it must have, whether staleness applies, which edges it may sit on, and whether it leaves in an export. A field on which four rules turn is not a field with one value.

### What `asserted` content may not do, derived rather than listed

Q15's leaning proposed that asserted content is "never a valid target for a `governs` or `verifies` relation". The list is close to right and it is a list, which means that the next relation somebody adds is not covered. Two rules replace it, and both come from machinery that exists.

**No edge may let unwarranted content govern the reading of warranted content.** [Reading precedence is already derived](../spec/02-taxonomy-model.md#reading-precedence-is-derived) from nuclearity, from succession, and from the governance family. Where the governing end is `asserted` and the other end is not, the edge is a finding. That covers an asserted standard that constrains an accepted specification, an asserted successor that displaces an accepted decision, and an asserted nucleus with an accepted satellite. It covers each of them for the same stated reason, and it needs no per-relation list.

The rule is silent about anchors, and that silence is correct. [Spec 2](../spec/02-taxonomy-model.md#behavior-at-the-limits) states that a relation ending on an external anchor carries no reading precedence. So an asserted document may declare `governs` against a code path, and it should. That edge is what gives synthesized onboarding notes write-time impact detection, which is the only mechanism that will ever tell anyone that they went stale.

**An asserted document does not discharge an evidence obligation.** The evidence family carries no reading precedence, so the first rule does not reach it, and that is the case Q15 cared about most. [Spec 3](../spec/03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two) already supplies the rule under a different name. `evidenced` means that an external, auditable artifact supports the claim. An asserted document is neither external nor auditable, so a pointer to one leaves the basis at `reconstructed` or `unevidenced`. Spec 3's own sentence is the argument: a fabricated *why* is worse than an admitted absence, because somebody will cite it.

**Staleness does not apply, and the remedy is never a date.** `last_verified` is the date on which a human confirmed that a document agrees with reality. An asserted document has no such date and cannot acquire one without becoming `accepted`. So it carries no freshness value, and it is never reported as stale. What it carries instead is its production date and the sources that produced it, which is TrustGraph's receipt with nothing added. Its decay is reported as drift: the sources changed after the date on which the content was asserted. The remedy is to regenerate the content or to delete it. [Spec 3](../spec/03-authoring-and-lifecycle.md#freshness-and-staleness) already warns that a block on staleness teaches authors to bump a date, and an asserted document has no date to bump.

### Promotion is one human, one document, one diff

Q15 asks whether human acceptance promotes asserted content to authored, or whether it stays second-class permanently.

Promotion is acceptance, and acceptance is the act that the whole model rests on. A person reads the document, sets the warrant to `accepted`, and puts their name in `accepted_by`. Nothing new is needed and nothing new should be built.

**The failure mode is bulk, and no mechanism can detect it.** A script that stamps `accepted_by` across forty synthesized memories produces bytes that are identical to forty real acceptances. This design cannot tell them apart and it does not pretend to. What it can do is make the rate visible, which is the move that the escape-hatch concentration and the suppression inventory both already make. `taxonomy audit` reports promotions per change, and a change that promotes forty documents at once is a finding about the review rather than about the documents. The posture is advisory, permanently, for the reason that the [focus-shift ratio](../spec/04-assurance-model.md#measuring-coherence-where-we-can-continuity-across-links) is advisory permanently: a threshold would only teach people to promote in batches of nine.

**Promotion does not rewrite history.** `drafted_by` and `agency` stay as they were. An accepted document that an agent drafted is the normal case in this design, and the corpus can still answer "which parts of this are agent-drafted?" after any number of promotions.

### Q19 — the axis is the pin, and imported text is a `transcribed` projection

Q19's five open points close, and one of them dissolves.

**The tier question was the same question as Q15's, and it has one answer.** Imported requirement text is not a fourth tier and not a qualifier on `generated`. It is `transcribed`, which is a warrant value that PROV already names `prov:Quotation`. The boundary between it and asserted content is exactly the one that Q15 identified and did not follow through: **is the content a function of a pinned input that the repository holds?** If it is, `generate --check` proves the equality on every run and the warrant is `transcribed`. If somebody edits, summarizes, reformats with judgment, or merges it with local prose, then it is no longer a function of the pin, no mechanism proves anything, and the warrant is `asserted`.

So regenerability was never a disqualifier from a tier. It *is* the tier axis. Q15's sentence "that text is regenerable, so it is not synthesized" had the fact right and the conclusion backwards.

**Does imported prose enter the corpus at all? Yes, as a projection, and reference-first survives with a better reason.** A transcribed document is generated, held to regeneration, and marked as generated like every other projection. A hand edit to it is the same finding that a hand edit to a shelf index is. Truth stays upstream, which is [principle 2](../spec/00-vision-and-scope.md#design-principles) satisfied rather than strained.

The leaning said that references ship first because imported text "imports a maintenance obligation". The sharper reason is a cost difference. Anchors and edges need only the anchor machinery, which exists and is declared. A transcription needs a resolver that reads text, a projection kind that writes it, and a `generate --check` comparison over the result. The first is free and the second is work, so the first ships first. No adopter has asked for the second.

**Snapshot format and home. The format question dissolves and the home question closes.** [Spec 2](../spec/02-taxonomy-model.md#behavior-at-the-limits) already rules that exactly one resolver owns each anchor kind, and that a resolver reads repository content or a committed snapshot. So what a snapshot looks like is a property of the resolver, and the specification privileges neither ReqIF nor a vendor's API export. That is the same ruling that keeps a forge out of the core, applied to a second kind of upstream.

What does not dissolve is the set of properties that the snapshot owes, because a committed file is a compatibility surface whatever its syntax.

- It is **committed inside the governed repository**. Check time is a function of repository content ([spec 0](../spec/00-vision-and-scope.md#non-negotiables)), and a reviewer needs to see in a diff what an upstream change moved.
- It carries its **fetch time and the upstream identity and revision of every requirement in it**. Without a revision, the drift comparison below cannot say which requirements changed.
- It is a **pin**, so [spec 7](../spec/07-distribution-and-federation.md#upstream-awareness)'s one-pattern-three-instances covers it with no new machinery.

The size question stays open, and it is the same question that [Q9](../spec/09-open-questions.md#q9--multi-repository-corpora) left open about a vendored source export. A requirement identity set is small enough that the answer here is obvious today. A snapshot with the full text of ten thousand requirements is not, and no adopter has one yet.

**What is an imported edge worth? Exactly what any generated edge is worth, and the leaning's answer is wrong.** The leaning said that imported edges "start advisory and walk the same evidence-driven promotion path as every other control". That misplaces the instrument, and the mistake is worth naming because it is easy to repeat.

[Principle 4](../spec/00-vision-and-scope.md#design-principles) promotes a **rule** against evidence about that rule's false-positive rate. An importer is not a rule. It is a producer of graph facts, and it has no false-positive rate to measure. A wrong imported edge produces a *correct* check result over a *wrong* graph, and no advisory posture on any check ever finds it. The instrument that finds it is a fixture set over the importer, which is exactly what [spec 12](../spec/12-check-layer.md#the-correctness-roots) already demands of the scaffolder, and for the identical stated reason: it manufactures edges at a scale that nobody reviews individually.

So an importer joins the correctness roots, and imported edges carry full weight. Two consequences follow.

- **An imported edge satisfies a participation expectation.** Expectations are detective and never blocking ([spec 4](../spec/04-assurance-model.md#absence-is-a-finding-class-of-its-own)), so nothing gates on this. To refuse would make the absence-finding class report a false positive for every requirement that the corpus genuinely traces to.
- **An imported edge supports `evidenced`.** The obligation on `evidenced` is that the pointer resolves to an external auditable artifact. A work item in a requirements tool is the clearest example of one that this specification has. Resolution runs offline against the committed snapshot and is deterministic. Q19's worry that "a system that nobody here governs discharges obligations" reads the state wrongly: `evidenced` asserts that an external artifact exists and is reachable, and it has never asserted that Headwater governs it.

**Whether imported text may leave again. The default is that it does not, and the mechanism landed one group earlier.** [Q17](../spec/09-open-questions.md#q17--governed-access-and-the-solution-layer) made the export filter **default-deny over classes**. A transcribed document is a class. So imported text stays inside the repository unless a profile names it, and naming it is a line in a taxonomy that a reviewer reads.

What this evaluation adds is not machinery but a statement, and the statement belongs beside the other non-claims. Headwater checks nothing about a license. It does not read an upstream's terms, it cannot tell whether an adopter may redistribute a requirement, and a profile that carries transcribed content is a redistribution decision that the adopter makes. Debian's archive split is the shape of the honest answer: segregate by default, carry by an explicit act, and record the terms beside the content.

**Drift is reported on the edge, not on the snapshot.** When a scheduled comparison advances the pin, the requirements whose revision changed are known. Every `traces_to` edge into one of them is a finding until a person re-verifies it. That is the suspect-link mechanism of requirements practice, and it is also [spec 4](../spec/04-assurance-model.md#absence-is-a-finding-class-of-its-own)'s report-at-the-origin rule. A change proposal against the whole snapshot names a file. A finding on an edge names the document whose author can act.

### Q18 — the adjudication is a document, and the edge already exists

Q4 handed this entry one question to decide: does the edge become a node?

**No. The edge does not become a node, because an adjudication that carries what Q18 wants is already a document.**

Read what the entry asks for. A named adjudicator. A date. Enough scope to say that one source wins *here* and not everywhere. Q21 adds a reason, and says that a judgment with no recorded reason is the authority rank again. A thing with an author, a date, a scope, and a reason, which a reader must find and which a later judgment may replace, is a document. The corpus has documents. It gives them shelves, kinds, lifecycles, voice regimes, identifiers, `accepted_by`, and supersession.

An edge promoted to a node would be a second and weaker version of all of that: an object with an author and a date and some prose, held by no shelf, typed by no kind, governed by no lifecycle, and bound by no `accepted_by`. [Spec 1](../spec/01-conceptual-model.md#facet) removed reference-valued facets on exactly that argument, and [Q4](../spec/09-open-questions.md#q4--relation-storage) cut the annotated prose link on it too. The cheaper and weaker mechanism wins under pressure, and then the record is in the weaker one.

**The edge that carries it is `overrides`, and the default vocabulary has shipped it all along.** [Spec 2](../spec/02-taxonomy-model.md#the-decision-relation-vocabulary) lists `overrides` in the Kruchten set, family `succession`, with the claim "displaces a prior decision's effect and does not retire it". That is an adjudication, stated in the specification, defined before the question was asked. The loser is kept and its effect is displaced, which is what a human adjudicating a live conflict actually does.

Everything that Q18 wants then follows from rulings that exist.

- **The adjudicator is named and resolved.** The adjudicating document carries `accepted_by`, its warrant is `accepted`, and the engine enforces both. No attribute holds an unresolved name.
- **Readers and agents inherit it without a new mechanism.** `overrides` is in the succession family, and [reading precedence](../spec/02-taxonomy-model.md#reading-precedence-is-derived) says that the successor governs. Routing already offers a successor before the document that it superseded ([spec 5](../spec/05-ai-integration.md#intent-time-routing)). An agent that meets both sides of a settled conflict is told which one governs, by machinery built for a different reason.
- **Scope comes from the endpoints and the prose.** "A specification outranks a standard about its own component and is silent about anything else" is one adjudicating document with one `overrides` edge. The scalar authority rank could not express that, which was the third reason for cutting it.
- **The adjudication can itself be wrong, and then it is superseded.** A document has a lifecycle. An attribute pair does not.

**One thing is missing today and it is small.** A reader who arrives at the losing document must learn that it was overridden. `supersedes` declares `inverse: superseded_by` with `reciprocal: required`, so `check --fix` writes the back-link mechanically. `overrides` needs the same declaration, and the base package supplies it. Without it, the adjudication is reachable from one end only.

**What the entry's three options got wrong, in one sentence each.** A resolution on the `conflicts_with` edge cannot name a resolvable adjudicator, which is the entry's own test. A correction or succession of the loser destroys the record, because the loser was not wrong and [spec 3](../spec/03-authoring-and-lifecycle.md#lifecycle) reserves correction for a document that was wrong about the present. A scoped-precedence declaration regrows the authority rank with more syntax, which the entry already suspected.

**And the strongest source argues the other way before it argues this way.** Legal citators put treatment on the citing relationship, which is the edge, and they have done so for over a century. Two facts turn that around. The flag is derived from a published opinion by an editorial process, so the opinion is the record and the flag is a projection over it. And two citators over the same case law disagree at a measured rate, which is what a derived scalar produced by a party who did not make the judgment is worth. Retraction practice lands where this ruling lands: a separate citable object with its own identifier, pointing at a work that stays in place.

**A generated view over `overrides` edges is legitimate and is a projection.** An adopter who wants a citator-style flag beside each decision declares one. It is generated, checked against regeneration, and canonical for nothing, which is the standing that a citator flag has in practice and never had in the entry's proposal.

### The mark has to survive an export, and a loss set is not enough

[Q17](../spec/09-open-questions.md#q17--governed-access-and-the-solution-layer) handed this group one constraint. Q15's answer to it was that an emitter whose target cannot carry the mark declares that in its loss set.

That is right and it is one step short. A loss-set entry informs the consumer who reads the loss set. The consumer who does not read it receives content that carries no warrant and looks exactly like content that carries one. That is Serena's failure delivered by post, and C2PA's own threat model says that stripped marks are the ordinary case rather than the exotic one.

So the rule is stronger. **An emitter that cannot carry the warrant does not carry `asserted` or `transcribed` content.** It withholds it, at the profile's declared tombstone grain, with the emitter's inability to mark as the declared reason.

Three things make this the right shape rather than an over-reaction.

- It is [principle 7](../spec/00-vision-and-scope.md#design-principles) as the serving-boundary group restated it. Degrade toward the cheaper error. Shipping unmarked unwarranted content is unrecoverable, because a reader acts on it. Withholding is visible and a consumer asks.
- It reuses the withholding machinery whole. A withholding is a census reason, and a tombstone reports it. Nothing new is built.
- It costs nothing today. The native graph export carries the warrant as a node property with no loss, and the first release ships only `json` and `jsonschema` ([spec 6](../spec/06-engine-architecture.md#cli)).

The `transcribed` case takes one addition, and CrossMark is where it comes from. A transcribed document that does leave carries its **pin identity** in the exported node, so a consumer holding a copy can go back and ask the authority. A mark that says "this came from somewhere else" is worth much less than a mark that says where.

## What stays open

**Whether anything is ever promoted.** The instrument is named below. If asserted content accumulates and nothing moves to `accepted`, the tier is a dumping ground, and the honest response is to say so rather than to make the mark louder.

**Whether a transcription projection ships at all.** No adopter has asked for imported requirement text, and reference-first is what ships. The trigger is an adopter who needs the corpus to be self-contained offline.

**The size of a committed snapshot.** This is the same open question that [Q9](../spec/09-open-questions.md#q9--multi-repository-corpora) holds about a vendored source export, and one real import is the evidence that closes both.

**Whether an adjudication is ever partial.** One decision may govern axis A while another governs axis B. Today that is two `overrides` edges, or one document whose prose carries the split and whose edges do not. Whether the corpus needs the split expressed mechanically is unargued, and no corpus shows the need.

**Whether the `asserted` edge rule needs a converse.** The rule stops unwarranted content from governing warranted content. It says nothing about a warranted document that cites an asserted one in a plain association, and nothing needs it to today.

**The `$`-reference grammar** stays where [Q2](../spec/09-open-questions.md#q2--schema-format) left it. A filter predicate over warrant values is one more candidate consumer of it.

## What this predicts, and how to measure it

[Principle 11](../spec/00-vision-and-scope.md#design-principles) forbids an inherited claim of efficacy. Five claims here are testable, and every one is unmeasured today.

| Claim | Instrument | Status |
|---|---|---|
| A visible warrant stops a reader treating asserted content as vouched | a probe whose only answer sits in an `asserted` document, graded on whether the transcript reports the warrant ([spec 5](../spec/05-ai-integration.md#measuring-whether-any-of-this-works)) | unmeasured |
| Asserted content is promoted rather than accumulated | the promotion rate per quarter in `taxonomy audit`, against the count of asserted documents | half measured. The denominator runs, and `taxonomy audit` reports the count under its warrant reading. The numerator is `warrant.promoted`, a change-scoped check that declares `needs_prior` |
| An adjudicating document reaches an agent that meets the losing document first | a probe over a settled conflicting pair, graded on which document the transcript cites | unmeasured |
| Imported edges do not decay faster than scaffolded ones | staleness by `created_by` in `taxonomy audit`, comparing `import` against `scaffold` | unmeasured, and no importer exists |
| Admitting the asserted tier does not lower routing precision | routing precision and abandonment ([spec 5](../spec/05-ai-integration.md#scent-is-the-thing-being-engineered)), split by warrant | unmeasured |

The second claim is the one to watch, because it is the whole argument for admitting the tier. Q15's case is that a refusal to model asserted content means that it arrives unmarked. If it arrives marked and then never moves, the corpus has bought a label and nothing else.

## Consequences for the specification

Twenty-four changes follow, and all are applied.

| Where | Change |
|---|---|
| [Spec 1](../spec/01-conceptual-model.md#warrant) | Warrant: the four values, the closed set, and the rule that absence is an error rather than a default |
| [Spec 1](../spec/01-conceptual-model.md#warrant) | No edge may let unwarranted content govern the reading of warranted content, and anchors are outside the rule |
| [Spec 1](../spec/01-conceptual-model.md#the-authored-form) | The relation example drops the adjudication attributes, which Q18 replaced with a document |
| [Spec 2](../spec/02-taxonomy-model.md#shape) | The worked taxonomy gains a transcription projection with its pin |
| [Spec 2](../spec/02-taxonomy-model.md#the-thirteen-declarations) | The count stays at thirteen. A snapshot pin is an anchor resolver, and a transcription is a projection |
| [Spec 2](../spec/02-taxonomy-model.md#the-decision-relation-vocabulary) | `overrides` is the adjudication edge, and it declares an inverse with required reciprocity |
| [Spec 2](../spec/02-taxonomy-model.md#instance-attributes-and-which-end-owns-each-one) | The attribute example loses the adjudication pair, and Q18's ruling is recorded where Q4 pointed at it |
| [Spec 2](../spec/02-taxonomy-model.md#behavior-at-the-limits) | The warrant edge rule, stated where the generated Graph checks live |
| [Spec 2](../spec/02-taxonomy-model.md#disagreement-is-adjudicated-not-ranked) | The positive half: an adjudication is a decision, and `overrides` carries it |
| [Spec 2](../spec/02-taxonomy-model.md#lineage-aligns-with-prov) | PROV supplies derivation and not endorsement, and `prov:Quotation` names the transcribed value |
| [Spec 2](../spec/02-taxonomy-model.md#the-meta-schema) | Warrant integrity: a transcription names a declared pin, and a pin names one resolver |
| [Spec 3](../spec/03-authoring-and-lifecycle.md#front-matter-is-the-contract) | The provenance block is the engine's, and it is no longer listed as a taxonomy choice |
| [Spec 3](../spec/03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed) | `warrant` joins the provenance block, with the required fields for each value |
| [Spec 3](../spec/03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed) | Promotion is one human, one document, one diff, and the bulk case is visible rather than detectable |
| [Spec 3](../spec/03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two) | An asserted document does not support `evidenced`, and an imported pointer does |
| [Spec 3](../spec/03-authoring-and-lifecycle.md#freshness-and-staleness) | Asserted content carries no freshness value, and its decay is drift with no date to bump |
| [Spec 3](../spec/03-authoring-and-lifecycle.md#lifecycle) | Correction, succession, and now adjudication: the third case that the pair did not cover |
| [Spec 4](../spec/04-assurance-model.md#declaration-moves-the-boundary) | The sweep's best output is a `conflicts_with` edge, and its resolution is an adjudicating document |
| [Spec 4](../spec/04-assurance-model.md#where-promotion-does-not-apply) | Promotion measures a rule. A producer of graph facts has no false-positive rate, and its instrument is a fixture set |
| [Spec 5](../spec/05-ai-integration.md#intent-time-routing) | A pointer to asserted content reports its warrant, which is a third kind of silence |
| [Spec 5](../spec/05-ai-integration.md#generated-artifacts-cite-what-licensed-them) | Where a declared adjudication exists, the agent cites it instead of flagging the conflict again |
| [Spec 5](../spec/05-ai-integration.md#the-stop-rules) | A fifth stop rule: an agent never writes `accepted_by` |
| [Spec 6](../spec/06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped) | An emitter that cannot carry the warrant withholds the content, and a transcription carries its pin |
| [Spec 6](../spec/06-engine-architecture.md#what-a-filtered-export-claims-and-what-it-does-not) | A sixth non-claim: Headwater checks nothing about a license |

Two further changes sit outside that table. [Spec 7](../spec/07-distribution-and-federation.md#upstream-awareness) reports snapshot drift on each affected edge rather than on the snapshot. [Spec 12](../spec/12-check-layer.md#the-correctness-roots) gains the importer as a correctness root, on the scaffolder's terms.

[Spec 11 §P](../spec/11-adjacent-work.md#p--provenance-endorsement-and-the-record-of-a-judgment) records the sources above, with what each one confirms, sharpens, or contradicts. Sections [§B.2](../spec/11-adjacent-work.md#b2-source-authority-ordering), [§F.2](../spec/11-adjacent-work.md#f2-karpathys-llm-wiki), [§J](../spec/11-adjacent-work.md#j-trustgraph--the-same-pitch-the-opposite-mechanism), [§K](../spec/11-adjacent-work.md#k-modern-requirements--the-first-candidate-where-the-arrow-reverses) and [§L.5](../spec/11-adjacent-work.md#l5-onboarding-ships-the-synthesized-tier-and-marks-nothing) move from recorded to decided.

The [glossary](../spec/glossary.md) gains **warrant**, **asserted content**, **transcription**, **snapshot pin** and **adjudication**. Its **provenance**, **projection**, **export**, **freshness** and **`created_by`** entries are corrected, Toulmin joins the table of borrowed terms, and two pairs join the table of distinctions.

In [spec 9](../spec/09-open-questions.md), Q15, Q19 and Q18 are rewritten as closed entries. Two other entries carry a stale reference that this ruling corrects. [Q4](../spec/09-open-questions.md#q4--relation-storage) said that Q18 owns the edge-as-node change, and now records that Q18 declined it. [Q20](../spec/09-open-questions.md#q20--where-scent-lives) keeps its cue as the one live instance attribute in the specification.
