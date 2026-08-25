---
id: HW-DR-0035
status: draft
status_since: 2026-08-25
summary: "The `requirement` and `acceptance_criterion` kinds serve the authored population alone. A transcribed requirement is a marked file that answers no contract, so a separate kind with an empty contract carries the imported case."
last_verified: 2026-08-25
title: "Q35 — Whether one requirement kind holds an imported requirement and an authored one"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-authoring-and-lifecycle
    - HW-SPEC-engine-architecture
---

# Q35 — Whether one requirement kind holds an imported requirement and an authored one

## Context

**[Q19](0019-inbound-integration-an-external-system-of-record.md) ruled on one branch and named the other.** It gives the test in one sentence: "is the content a function of a pinned input that the repository holds?" Where the answer is yes, the content is `transcribed` against a committed pin. Where a person edits, summarizes or merges it with local prose, Q19 states that "no mechanism proves anything and the warrant is `asserted`". The subject of Q19 throughout is an adopter's external system of record. This repository holds no such system. So every requirement it writes about its own engine falls on the branch Q19 names and leaves open.

**[#398](https://github.com/headwater-ai/headwater/issues/398) asks for the kind and cannot declare it.** It asks for `requirement` and `acceptance_criterion`, a `verified_by` relation, a required fit-criterion section and a `verification_method` facet. Its third clause asks a builder to state which case the kind serves. No record states it, so the clause asks a builder to write the ruling while building against it. [#252](https://github.com/headwater-ai/headwater/issues/252) named this decision as queued and blocking, and closed without filing it.

**The warrant is a declaration on one branch and a derived reading on the other.** [Spec 3](../spec/03-authoring-and-lifecycle.md#the-warrant-and-what-each-value-requires) closes the warrant at four values, and `transcribed` requires the generated-file marker. [Spec 6](../spec/06-engine-architecture.md#taxonomy-validate-versus-taxonomy-audit) states what follows. The engine reads `regenerated` and `transcribed` "off the generated-file marker rather than out of a declaration". So two rows of the warrant reading stand at zero by construction. A document that states `warrant: transcribed` in its front matter therefore states a value that no reading of this engine takes.

**A marked file answers no contract, and no run reports the gap.** [Spec 3](../spec/03-authoring-and-lifecycle.md#templates-and-scaffolding) rules that "a projection carries a generated-file marker, and no check reads the file". [Spec 6](../spec/06-engine-architecture.md#projections) gives the two reasons and gives the census an outcome for such a file. So a required facet is absent and unreported on a marked file. A required section is absent and unreported there too. A kind whose contract demands either one demands it of two populations and measures it on one.

**This corpus holds a worked example of that mismatch, and it is an illustration rather than a proof.** `kinds.decision_register` requires the `doc_type` and `sequence` facets. `docs/spec/09-decisions.md` is authored and carries both. `docs/spec/09-open-questions.md` is the `shelf_sections` projection under the same kind, and it carries `doc_type` and no `sequence`. A run of `headwater check` over this repository reports the second file as generated in the census and reports no finding against it. This is what a declaration reads like when it requires something that half of its own population does not answer. It does not show that one kind is unsafe. The repair it asks for sits at the emitter or at the check layer, and [#409](https://github.com/headwater-ai/headwater/issues/409) holds it.

## Decision

**`requirement` and `acceptance_criterion` serve the authored population, and a separate kind carries the imported one.** A document of either kind is authored, every check reads it, and its author repairs it where it stands. A transcribed requirement is a projection of a pinned snapshot, and the kind it declares is neither of these two.

**A natively authored requirement takes `accepted` or `asserted`, and a human decides which.** [Spec 3](../spec/03-authoring-and-lifecycle.md#the-warrant-and-what-each-value-requires) names four warrants. Two of them require the generated-file marker, and two do not. `accepted` and `asserted` are the two that a document states about itself. [Q34](0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md) rules that the merge onto `main` is what makes `accepted` true. So the warrant of a requirement of this kind is a fact about its review and never a fact about the kind.

**The test is the contract, and not the provenance.** A kind serves the authored population when its contract is not empty. Such a contract requires a facet, a section or a lifecycle state that the document states in its own bytes. A kind serves the transcribed population when its contract is empty, because a marked file satisfies nothing more and no check reads it. `kinds.probe_result` is the worked example this engine already ships. It requires no facet and no section, and `kinds.probe_transcript` beside it requires two of each.

**A contract conditioned on the marker is the alternative this record refuses, and the schema language is the reason.** The warrant is not the only way to hold one kind over two populations. The engine reads the generated-file marker reliably, and it already acts on that reading when it exempts a marked file from every check. So one kind could in principle require a facet of its authored members and require nothing of its marked ones. The taxonomy language cannot say that. `facets` takes `require`, `optional` and `forbid`, and `sections` takes `require` and `optional`. Each of those is a flat list of names, and none of them takes a condition. The language does carry one conditional member, which is `when` on a relation expectation, and it keys on a facet value. The marker is not a facet, and the thirteen declared facets do not include it. Conditioning a contract on the marker would therefore need two members that the meta-schema does not have. Two kinds is how this language states the split today.

**The imported kind belongs to the adopter, and this repository declares none.** [Q19](0019-inbound-integration-an-external-system-of-record.md) put the snapshot, the resolver and the snapshot format on the adopter's side. The kind that a transcription projection names in its identity block follows them there. This repository has no upstream system of record, so it declares no such kind and holds no such document.

**The two populations meet in the graph and not in the kind.** Q19 rules that an imported edge "is worth what any generated edge is worth". `verified_by` therefore names both kinds at its `from` end, and an edge crosses between the two populations with no qualifier. An adopter reads one shelf and one traceability matrix over both.

**A requirement that changes population changes identifier, by succession.** A team that retires an upstream and takes ownership of the text writes an authored requirement and supersedes the transcribed one. `supersedes` is the mechanism the base package already declares, and this ruling adds none.

## Consequences

**An author-owned language rule bound to `requirement` reaches the whole population of that kind.** This answers the second half of the question, and it follows from the ruling above rather than from a new mechanism. The kind holds authored documents alone, so a language regime bound to it reaches every document of it. The EARS rule that #398 defers therefore has a coherent subject on the day it lands.

**No language rule reaches transcribed text, whatever kind carries it.** No check reads a marked file, so a regime bound to the imported kind reads nothing at all. Imported requirement text is the text most likely to fail a controlled-language rule, and it is text that an author may not repair in place. Conformance of imported text is therefore a property of the upstream and of the importer. Q19 already put the instrument there, because "an importer joins the correctness roots" and its fixture set is where a wrong transcription is caught.

**The split checks nothing new, and a new check is not what it buys.** No rule reads a marked file under one kind, and no rule reads one under two. The exemption is the same either way. What changes is where the exemption is written down. Under two kinds the kind name states which population a document is in. A reader, a query and a count then partition by the kind they already read. Under one kind that partition is a fact that every reader downstream has to remember and apply by hand. The second gain is honesty in the declaration. A kind that requires a facet of a population it cannot reach claims something it does not enforce. A kind with an empty contract claims nothing it cannot keep.

**The cost of the reading this record refuses is one register split into two kinds.** ISO/IEC/IEEE 29148 keeps one requirement register whatever the provenance of an entry. An adopter arriving from a requirements tool meets two kinds where practice gave them one, and pays for that in three places. A count by kind sums two numbers. `verified_by` names two kinds at one end and stays in step with both. A requirement that migrates from imported to authored takes a new identifier. A citation of the old identifier then resolves through a supersession edge rather than directly.

**The split earns that cost because it is not a split by provenance.** It is the split between a file that an author writes and a file that an emitter writes. That line already runs the length of this engine. [Spec 3](../spec/03-authoring-and-lifecycle.md#templates-and-scaffolding) draws it between a scaffolded document and a projection, and calls regeneration "the whole of the difference". [Spec 6](../spec/06-engine-architecture.md#projections) gives a marked file a census outcome of its own and exempts it from every check. Requirements practice has no projection, so it has no reason to draw the line. This system has one, and a kind that ignored the line would be the only declaration in this taxonomy that does.

**This ruling does not make a declared warrant checkable, and it removes the reason to want that here.** [HW-OBL-0125](../obligations/0125-nine-documents-state-a-warrant-the-closed-set-does-not-hold-and-no-check-reads-one.md) records that no check reads a warrant against the closed set. [Spec 13](../spec/13-open-obligations.md#what-the-first-typing-of-this-corpus-found) records that `facet.value.not_permitted` "reads one value list for the whole taxonomy", and [#222](https://github.com/headwater-ai/headwater/issues/222) holds the ruling on a per-kind value set. Under this ruling neither requirement kind needs one. The engine derives the warrant of an imported requirement from the marker. The authored kind holds no imported document to admit a wrong value on.

**One kind in this corpus serves both populations today, and its repair is not this ruling.** `decision_register` carries an authored register and a generated one, and the generated one is missing a facet that the kind requires. No run reports that. [#409](https://github.com/headwater-ai/headwater/issues/409) holds it. The fix it asks for is a rule that reports the mismatch, or an emitter that writes the facet. Neither of those is a split. So the example shows the shape of the problem and argues for no particular remedy.

**What reopens this.** A member that narrowed a facet value set per kind would let one kind carry two warrants. It would refuse the wrong combination on each. #222 holds that ruling. Even with that member, a required section and a required facet stay unreadable on a marked file. The argument from the contract above therefore stands unchanged. Two other changes would reopen this ruling. The first is a facet contract or a section contract that takes a condition on the generated-file marker. The schema language refuses that design today, and the argument above does not refuse it. The second is a check that reads a generated file.
