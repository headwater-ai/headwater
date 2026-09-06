---
id: HW-REG-decisions
status: current
status_since: 2026-08-11
last_verified: 2026-08-13
summary: An index of the twenty-one design decisions and the ones that the build raised, where the record of each one lives, and the evidence that closed it.
doc_type: decision_register
sequence: 9
title: "The decision register"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  supersedes:
    - HW-REG-open-questions
  cites_evidence:
    - HW-EVAL-adjacent-work
    - HW-EVAL-default-taxonomy-first-run
    - HW-EVAL-first-contact
    - HW-EVAL-graph-export-and-federation
    - HW-EVAL-language-choice
    - HW-EVAL-language-spike-results
    - HW-EVAL-linkml-worked-example
    - HW-EVAL-relation-storage
    - HW-EVAL-schema-format-walkthrough
    - HW-EVAL-shacl-worked-example
    - HW-EVAL-the-measurement-layer
    - HW-EVAL-the-serving-boundary
    - HW-EVAL-warrant-and-adjudication
    - HW-EVAL-what-a-check-can-know
---

# 9 — The decision register

This file began as a list of decisions that the design phase deferred, with the options and a leaning for each one. Every entry is now argued, and all twenty-one are closed. The last one to close was [Q11](#q11--license-and-distribution-posture). No argument from the design could close it, because a license states what the owner intends for the project. The owner ratified it on 2026-08-11.

**This file is now an index, and it stopped being the place a decision lives.** Each decision is a document on [the decisions shelf](../decisions/README.md), of the base `decision` kind, and it carries the argument that settled it. Every one of the twenty-one is `current`, with a `status_since` of 2026-08-11. The evidence for each closure lives beside it in [`docs/evaluations/`](../evaluations/).

**The entries below them came from the build rather than from the design phase, and this file no longer counts them.** A question that a build raises belongs on this list, because the alternative is a ruling that lives only in a module comment. The evidence for one of these is a measurement in the engine rather than an evaluation, and each section names the measurement it rests on. A count here was a second copy of what the headings already say, and it was wrong for two entries before anybody read it.

A decision was a heading in a register. Nothing could carry its state, its dates, its acceptance or its edges. A reader who cited one cited a position in a file. Now a citation names a document with an identifier, and a check reads the document that a citation reaches.

**The headings below stay because the corpus cites them.** More than two hundred citations name a decision by the anchor of its heading here. An evaluation is a point-in-time record that this project does not rewrite. So each heading keeps the text that it carried, and each section says which document now holds the argument.

**[9 — Open questions](09-open-questions.md) is the same instrument one step earlier, and a projection writes that one.** All twenty-one of the design-phase questions closed, and this corpus cites the anchors of that file 133 times, across the twenty-one anchors `#q1` to `#q21`. The projection writes thirty-five sections there now, and no citation reaches the fourteen below Q21. An evaluation is a point-in-time record, and this project does not rewrite one when a later decision changes something. So those anchors have to keep resolving. [Spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for) forbids a silent pass, and [Q17](#q17--governed-access-and-the-solution-layer) rules that a filtered view announces its filtering. A deleted file breaks every one of those citations and says nothing about it.

What the decisions leave open is in [13 — Open obligations](13-open-obligations.md). That file is an index of the same shape as this one. Each of its 125 items is a document on [the obligations shelf](../obligations/), and each one names the decision or the specification that produced it.

A human maintains the list below by hand, and [spec 13](13-open-obligations.md#a-human-maintains-this-list-by-hand) records why. Two of the three things that blocked a projection here are gone. `shelf_sections` writes one heading for each document on a shelf, and a projection declares the identity of the document it writes. A generated file at this path would keep the 15 edge halves that name `HW-REG-decisions` as their target. `headwater explain docs/spec/09-decisions.md` reports that count as the incoming halves, and it reports 14 outgoing halves that this file declares. The two counts read different directions, and the identity block on a projection holds only the incoming one. What holds this file back is its own content. The paragraphs above the list have no derivation. One of the 28 anchors that this corpus cites here is a subsection of Q9 rather than a document, and it is `#the-aggregator-authors-its-own-facts`.

## Q1 — Implementation language

The language is **Rust**, with a WebAssembly build of the same crate for editor and browser embedding. The record is [HW-DR-0001](../decisions/0001-implementation-language.md).

## Q2 — Schema format

YAML 1.2 is the concrete syntax, and Headwater owns the schema language, the reference sublanguage, the overlay language and the meta-schema. The record is [HW-DR-0002](../decisions/0002-schema-format.md).

## Q3 — How much of the default taxonomy ships in the box

The base package is minimal and derived from the core, and optional content ships as add-only bundles. The record is [HW-DR-0003](../decisions/0003-how-much-of-the-default-taxonomy-ships-in-the-box.md).

## Q4 — Relation storage

Front matter is authoritative, and a relation instance is an object rather than a pointer. The record is [HW-DR-0004](../decisions/0004-relation-storage.md).

## Q5 — Voice checking depth

Voice checking stays lexical, with a curated pattern set, a per-category posture and a reasoned escape hatch. The record is [HW-DR-0005](../decisions/0005-voice-checking-depth.md).

## Q6 — Where the corpus graph lives at rest

The graph never rests. Every run rebuilds it, and no derived artifact is canonical for anything. The record is [HW-DR-0006](../decisions/0006-where-the-corpus-graph-lives-at-rest.md).

## Q7 — Scope of the MCP surface

Three classes of tool, and a landed write never ships. The record is [HW-DR-0007](../decisions/0007-scope-of-the-mcp-surface.md).

## Q8 — Probe cost and cadence

A probe is a document with a declared expectation, and cadence follows the purpose of the run. The record is [HW-DR-0008](../decisions/0008-probe-cost-and-cadence.md).

## Q9 — Multi-repository corpora

A repository holds one or more corpora, the tier above harvests pinned exports, and no merged graph exists. The record is [HW-DR-0009](../decisions/0009-multi-repository-corpora.md).

### The aggregator authors its own facts

The federation layer is a corpus at a higher altitude. It authors the facts that live between repositories, and it reads exports for everything else. This heading stays because [HW-EVAL-adjacent-work](../evaluations/adjacent-work.md) cites it twice. The text is now [a subsection of the record](../decisions/0009-multi-repository-corpora.md#the-aggregator-authors-its-own-facts).

## Q10 — Naming

The name is **Headwater**, capitalized in prose and lower case as an identifier. The record is [HW-DR-0010](../decisions/0010-naming.md).

## Q11 — License and distribution posture

**Apache-2.0** for the engine, the library, the base package and the bundles, ratified by the owner on 2026-08-11. The record is [HW-DR-0011](../decisions/0011-license-and-distribution-posture.md).

## Q12 — Migration path for an existing corpus

Adoption is a migration from no taxonomy, and `headwater infer` computes the adoption payload. The record is [HW-DR-0012](../decisions/0012-migration-path-for-an-existing-corpus.md).

## Q13 — LinkML and SHACL as substrate

Headwater owns the language, emitters never chain, and LinkML is the last of six siblings. The record is [HW-DR-0013](../decisions/0013-linkml-and-shacl-as-substrate.md).

## Q14 — Discovery surface

The corpus descriptor is a generated projection at `.headwater/corpus.json`, and registration went to Q16. The record is [HW-DR-0014](../decisions/0014-discovery-surface.md).

## Q15 — A synthesized content tier

Every document carries a warrant from a closed set of four, and `asserted` content is admitted with limits. The record is [HW-DR-0015](../decisions/0015-a-synthesized-content-tier.md).

## Q16 — Public presence

Registration needs a channel with an obliged reader, a directory of corpora is refused, and the site is a projection of this corpus. The record is [HW-DR-0016](../decisions/0016-public-presence.md).

## Q17 — Governed access and the solution layer

The serving boundary is the export step of each publishing corpus. A profile filters for an audience, and Headwater has no principals. The record is [HW-DR-0017](../decisions/0017-governed-access-and-the-solution-layer.md).

## Q18 — Recording adjudicated disagreements

The edge does not become a node. An adjudication is a decision document that carries `overrides`. The record is [HW-DR-0018](../decisions/0018-recording-adjudicated-disagreements.md).

## Q19 — Inbound integration: an external system of record

Imported content is `transcribed` against a committed pin, and an imported edge carries full weight from the first release. The record is [HW-DR-0019](../decisions/0019-inbound-integration-an-external-system-of-record.md).

## Q20 — Where scent lives

An optional source-owned cue sits on a relation instance, graded against the alternatives in view at the point of decision. The record is [HW-DR-0020](../decisions/0020-where-scent-lives.md).

## Q21 — Terminological succession, and validity under merge

A retired-term lexicon sits in the language regime, and a verdict reports the read set that a merge may void. The record is [HW-DR-0021](../decisions/0021-terminological-succession-and-validity-under-merge.md).

## Q22 — The integrity posture of a published package

A package digest checks a fetched artifact against a pin that a person committed, and it is never a signature. The in-house SHA-256 is held against a second implementation rather than replaced by a dependency. The record is [HW-DR-0022](../decisions/0022-q22-the-integrity-posture-of-a-published-package.md).

**This question arrived after the twenty-one above closed, and it did not come from the design phase.** [#77](https://github.com/headwater-ai/headwater/issues/77) built the publishing half of [spec 7](07-distribution-and-federation.md#publishing) and found a choice that would otherwise have been made by reflex. The engine already carried a SHA-256, and reaching for it would have settled what a fetched artifact is checked against without anybody stating it.

## Q23 — The engine lint floor

A published Rust guideline set is not installed as a skill, because a skill carries no rule of its own. Twenty-two lints are declared once in the workspace manifest, and each was chosen by measuring the corpus rather than by adopting a list. The record is [HW-DR-0023](../decisions/0023-the-engine-lint-floor.md).

**This question also arrived from outside the design phase, and it was asked as a question about a skill.** Two agent-facing Rust guideline sets were offered for installation. [Spec 5](05-ai-integration.md#how-a-skill-reaches-an-agent-and-what-nothing-does) already refuses that shape, so the answer was available. What the question found is that the rule the guidance points at had nowhere to live. The workspace manifest declared no lint, and the continuous integration job ran the default set alone.

## Q24 — Readability, and what a sweep can be asked about

Readability is not a class of the assisted sweep, and it gets no verb. Every class of a sweep names two things that do not fit, and a readability finding names one. A source file is not a slice member either. The record is [HW-DR-0024](../decisions/0024-q24-readability-and-what-a-sweep-can-be-asked-about.md).

**This question arrived from the build, and it was asked about a body of prose that no rule reads.** [#191](https://github.com/headwater-ai/headwater/issues/191) measured the engine's comment prose at about 179,000 words, which is more than the governed prose under `docs/spec/`. Every prose rule here was run over it. It found zero contractions and zero British spellings, so the two rules that block already pass by habit. What makes the prose hard to read is a habit that no lexical rule reaches, and the question is which mechanism performs the pass.

## Q25 — Where the namespace goes in an identifier, and who declares it

The namespace goes first, so every scheme renders `{namespace}-<TYPE>-<local part>`. A published source declares no namespace, and the corpus that adopts it declares one in its overlay. The record is [HW-DR-0025](../decisions/0025-q25-where-the-namespace-goes-in-an-identifier-and-who-declares-it.md).

**This question came from the build, and it was asked about the two schemes that ship.** [#207](https://github.com/headwater-ai/headwater/issues/207) found `namespace: repo` on `decision_id` and on `obligation_record_id`, which are the only two schemes that a published source declares. `repo` is a word every adopter also writes, so two adopters who take the base unchanged both mint the same identifier. A package has no value it can honestly write there, so it writes none. The order was settled in the same change, because a rendered identifier travels into a ticket and cannot be recalled from one.

## Q26 — Whether terminality belongs to a state, or to a state and a regime

Terminality stays a property of a state alone. A regime that ends one state where another continues past it is refused, and the remedy is a second state value. The record is [HW-DR-0026](../decisions/0026-q26-whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime.md).

**This question came from the build, and the change that made the narrowing is what raised it.** [#241](https://github.com/headwater-ai/headwater/issues/241) made the two readings of terminal name one set, and [#242](https://github.com/headwater-ai/headwater/issues/242) says what that costs. The reason both of those gave was wrong. They state that a per-regime reading has no regime at the call site, and both readers of terminality hold one. A per-regime reading was built here, and it turns a deferral into a pass. No published tradition asks for one word to end one lifecycle and to continue another.

## Q27 — Whether a decision record is governed prose

A decision record is governed prose, and the overlay binds the house language regime to `decision`. The shelf carries a prose problem rather than a quotation problem, and the binding costs no gate work. The record is [HW-DR-0027](../decisions/0027-q27-whether-a-decision-record-is-governed-prose.md).

**This question came from the build, and it was asked about the shelf that answers this register.** [#205](https://github.com/headwater-ai/headwater/issues/205) found that the regime bound four kinds, that this register was one of them, and that the 26 documents it indexes were not. A decision record quotes prior art, and [HW-OBL-0086](../obligations/0086-an-inline-quotation-reaches-every-lexical-rule-as-this-authors.md) holds an inline quotation against its author. So the issue asked for the findings to be characterized before the ruling was taken. Collapsing every inline quotation on the shelf moved the count by one finding in 64. Every finding is advisory, and 60 of the 64 fall on the five most recent records.

## Q28 — Whether an evaluation is governed prose

An evaluation is governed prose, and the overlay binds the house language regime to `evaluation`. The shelf carries 432 advisory findings and no error, and a binding does not clean a shelf. The record is [HW-DR-0028](../decisions/0028-q28-whether-an-evaluation-is-governed-prose.md).

**This question came from the build, and it was the arm Q27 left open.** [#247](https://github.com/headwater-ai/headwater/issues/247) counted the 432 and asked what they are. 422 name a sentence past the word limit, 10 name a semicolon, and 408 of the 432 fall on a paragraph. Collapsing all 159 inline quotations moved the count to 417. The distribution that Q27 read as an era of authorship does not hold here. Every evaluation is agent-drafted and 14 of the 15 are accepted by a human. Across six groups of this corpus, what separates a clean one from a dirty one is whether a rule was reading the prose.

## Q29 — Whether a corpus root may contain code, and what an interface contract may reach

A path is corpus content when a file arriving there with no front matter is a defect, and no code directory answers that test. The corpus root stays `docs`, and an interface contract lives inside it and reaches a crate by a `governs` edge that binds on existence alone. The record is [HW-DR-0029](../decisions/0029-q29-whether-a-corpus-root-may-contain-code-and-what-an-interface-contract-may-reach.md).

**This question came from the build, and it had to be settled before a kind was declared.** [#253](https://github.com/headwater-ai/headwater/issues/253) asks where a document that describes a verb of this engine lives. `git ls-files engine` reports 608 files and 186 Markdown files, and 185 of the 186 are inputs to the suite rather than prose a person reads. The exclusion this corpus already declares on `docs/taxonomies/**` gives the reason in one sentence. A fixture corpus is a corpus that another root is meant to walk. What the ruling leaves open is that nothing holds a crate to having a contract, and [HW-OBL-0128](../obligations/0128-nothing-holds-a-crate-to-having-a-contract-under-a-root-that-excludes-it.md) carries it.

## Q30 — Whether what an obligation waits on is a state or a property, and how many values it takes

What an obligation waits on is a present-tense state and not a property fixed at filing, and the closed set holds four values. A required `waiting_on` facet on `obligation_record` takes `ruling`, `build`, `measurement` or `adopter`, and the decision-record bundle declares it. The record is [HW-DR-0030](../decisions/0030-q30-whether-what-an-obligation-waits-on-is-a-state-or-a-property-and-how-many-values-it-takes.md).

**This question came from the board rather than from the build, and the specification argued against it in writing.** The obligations shelf holds 129 records and no record says what it is waiting on. So the owner reads the shelf with `grep -il ruling`, at a precision of 0.50 and a recall of 0.36. [Spec 13](13-open-obligations.md#a-human-maintains-this-list-by-hand) refuses a facet over this shelf because a four-class partition makes one class a residual. No value of these four is a residual, and the pair that objection rests on takes one value here. 38 of the 39 open records that wait on a ruling are followed by a build. A two-valued property would be wrong about 38 records.

## Q31 — Whether this repository becomes public, and when

This repository is public. The owner set 2026-09-08 and opened it on 2026-09-06, and this record stops each run from raising the schedule. The record is [HW-DR-0031](../decisions/0031-q31-whether-this-repository-becomes-public-and-when.md).

**This question came from the owner, and the cost of leaving it unwritten was a repeated argument.** The build order carried it on a list of standing escalations as undecided, so each run met it and left it where it was. 22 open obligation records wait for a corpus that another organization keeps, and nobody outside can bring one while the repository is unreadable.

## Q32 — Which test the self-audit label states

The `self-audit` label states the reader test. Its description reads *No reader outside this repository. How it was found is not the test.* The record is [HW-DR-0032](../decisions/0032-q32-which-test-the-self-audit-label-states.md).

**This question came from the board, and the label decided whether work waits.** The earlier description named a test of provenance and a test of the reader in one sentence. The two partition the board differently. Almost every finding here arrives from running the engine over this repository, so a test of provenance separates nothing.

## Q40 — Whether extends, bundle, requires and an overlay's taxonomy key are a mechanism or a label

All four keys of the family are a label rather than a mechanism today. `extends` sits at a taxonomy source's root, and `taxonomy`, `bundle`, `extends` and `requires` sit at an overlay's root. The specification and the meta-schema now say so beside each declaration. The record is [HW-DR-0040](../decisions/0040-q40-whether-extends-bundle-requires-and-an-overlay-s-taxonomy-key-are-a-mechanism-or-a-label.md).

**This question came from [#295](https://github.com/headwater-ai/headwater/issues/295), and one bar applied once settled all five sites at once.** A key earns a reader only where giving it one is a comparison against a value the resolver already holds, with no new fetch and no new lock content. Three of the five sites have a plausible cheap mechanism in principle, and each fails the bar for a distinct, evidenced reason. A taxonomy source's `extends` needs a resolver capability this repository has not built. It also collides with the still-open lock-seam ruling [HW-OBL-0082](../obligations/0082-the-lock-is-half-generated-and-half-authored-and-nothing.md). An overlay's `extends` needs a design decision this piece does not make, because every bundle this repository ships already carries a value that has moved apart from its actual base. An overlay's `requires` documents an invariant that referential integrity already enforces on an unrelated path. The other two sites, `bundle` and an overlay's `taxonomy`, fail on evidence: nothing in this repository exercises either one.

## Q41 — Whether Vale becomes a declared regime backend

Vale does not become a declared regime backend, and its findings are not translated into this engine's `Finding` shape. It stays where [spec 00](00-vision-and-scope.md#what-we-do-not-build) already puts it, composable alongside. The record is [HW-DR-0041](../decisions/0041-q41-whether-vale-becomes-a-declared-regime-backend.md).

**This question came from [#438](https://github.com/headwater-ai/headwater/issues/438), and [Q24](#q24--readability-and-what-a-sweep-can-be-asked-about) set the bar a reopening argument had to clear.** A profile the taxonomy declares is read by a check. So the argument had to say why the check layer is the wrong place a second time. Four structural mismatches meet that bar. The plugin interface [spec 12](12-check-layer.md#the-plugin-interface) already declared does not fit an external process like Vale. The closed control registry cannot hold Vale's open-ended, adopter-configured rule set, because [`Finding.rule`](../../engine/crates/check/src/finding.rs) is a compile-time constant. Vale's own raw-text scoping model reintroduces the segmentation false-positive class [Q5](#q5--voice-checking-depth) already measured and [`sentences.rs`](../../engine/crates/doc/src/sentences.rs) already fixed. Determinism has no answer today for an external tool's own versioned inputs.

## Q42 — What "one screen" means for the first help screen

"One screen" is retired as a claim about a terminal and replaced by a claim about content. The first screen carries one line per entry, and the ceiling that follows from it is 60 lines. The record is [HW-DR-0042](../decisions/0042-q42-what-one-screen-means-for-the-first-help-screen.md).

**This question came from [#342](https://github.com/headwater-ai/headwater/issues/342), and no layout at 80 columns could ever have met the reading that was retired.** One line per verb costs 24 lines for eighteen verbs, and the headings and separators cost eleven more. So 35 lines are spent before one flag or one example is printed. The lever is what the screen says, and 25 of its 76 lines were the whole description of five global flags. `GLOBALS` in [`engine/crates/cli/src/lib.rs`](../../engine/crates/cli/src/lib.rs) now declares a one-line summary beside each description, and the screen prints the summaries. The descriptions stay on all 32 verb pages, where [HW-DR-0033](../decisions/0033-q33-whether-the-command-line-is-derived-and-who-a-flag-belongs-to.md) already puts them. The measured screen is 56 lines.

## Q43 — Whether a refusal under `--json` is a JSON document

A refusal is an English sentence on standard error, and it is never a JSON document. `--json` names the shape of an artifact, and it moves no stream and no grammar. The record is [HW-DR-0043](../decisions/0043-q43-whether-a-refusal-under-json-is-a-json-document.md).

**This question came from [#346](https://github.com/headwater-ai/headwater/issues/346), and the alternative was refused because it cannot keep its own promise.** Ten command lines name a JSON target and then refuse, and every one of them writes zero bytes to standard output. The benefit claimed for a JSON refusal is one grammar for one consumer. `clap` refuses a conflicting pair of flags before the verb is entered, so a consumer would meet an English sentence on some command lines anyway. Three other grounds agree with that one. The split between an artifact and an account is a rule that [spec 6](06-engine-architecture.md) already states for `--fix`. The refusal architecture is uniform across 83 call sites of three helpers. And `docs/interfaces/headwater-explain.md` already carried the house wording for one verb. Eight interface contracts state the rule for their own verb, and a refusal of `export` names the spelling of the target that the caller typed.
