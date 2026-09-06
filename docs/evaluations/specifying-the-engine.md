---
id: HW-EVAL-specifying-the-engine
status: current
status_since: 2026-08-16
summary: "`docs/spec/` is a design-spec series and nothing types the engine as a thing under test, and three traditions each supply one part of what would."
last_verified: 2026-08-16
title: "Specifying the engine"
provenance:
  warrant: accepted
  accepted_by: j.baxter
  agency: mixed
  drafted_by: claude-opus-5
  activity: evaluate+draft
  evidence_basis: evidenced
---

# Specifying the engine

This evaluation answers a question that nobody had filed: the corpus types its own prose, and it types nothing about the engine that reads it. It is a survey of three specification traditions, and a mapping of each one onto the declarations this taxonomy already has. It declares nothing. No kind, facet, shelf, relation, regime or projection in this repository moves because of it.

Two claims prompted it, and both were checked against the engine rather than against memory.

## What the corpus types today, measured

`.headwater/corpus.json` sets the corpus root to `docs` and to nothing else. `headwater check` reports 233 files under it: 190 typed, 3 generated, 36 excluded as package content under `docs/taxonomies/**`, 2 not a document, and **2 untyped**. The two untyped files are `docs/doctrine/maturity-model.md` and `docs/w3id/README.md`, each reported as "no shelf pattern claims this path".

So the first claim is nearly right and wrong in two small ways. Two documents under `docs/` answer to no kind. And the typing is not the work of one sample taxonomy but of four sources. The base package `headwater/standard 2.0.0` supplies `decision`, and the [design-spec entry](../taxonomies/design-spec/doctrine.md) supplies `design_spec`, `decision_register`, `obligation_register`, `evaluation`, `review_prompt` and `review_record`. The [decision-record entry](../taxonomies/decision-record/doctrine.md) supplies `obligation_record`, and `.headwater/overlay.yml` declares `probe`, `probe_transcript` and `probe_result` locally, which no entry ships. The base's own `specification` kind and its `docs/specifications/**` shelf stand empty, which is finding 2 of the design-spec doctrine arriving on schedule.

The second claim holds, and it is sharper than it was put. Three separate things are missing, and they are missing for different reasons.

**No functional specification of the command surface.** The engine is 22 crates and a binary whose verbs include `check`, `new`, `route`, `explain`, `generate`, `query`, `import`, `conformance`, `sweep plan|report`, `probe plan|record|grade|stale`, `taxonomy validate|resolve|audit|vendor` and `capture`. Their exit-status contracts exist, and they exist as prose scattered across five specification parts. Spec 4 covers probe results and sweeps, spec 5 covers the two streams and the status, and spec 7 covers `--level`. Spec 12 covers the sweep's three zero exits and two non-zero ones, and spec 15 covers `probe stale`. No document states, for one verb, its inputs, its flags, its exit codes, the split between its two streams, and the errors it can return.

**No technical specification of the crates.** `engine/README.md` is the only crate map, it runs 293 lines, and it sits outside the corpus root. Nothing types it, no rule reads it, and the graph reaches it only as a `code_path` anchor. A reader who wants the contract of `headwater-resolve` reads Rust.

**No kind for a requirement or for an acceptance criterion.** [Spec 3](../spec/03-authoring-and-lifecycle.md#identifiers) reserves stable identifiers for five things: "decisions, requirements, acceptance criteria, controls, and obligations". Three of the five have kinds — `decision`, the control block of the taxonomy, and `obligation_record`. Two have nothing at all. The specification names them and the schema cannot express them.

The distinction between the three matters, because a different tradition answers each one.

## Requirements: 29148, EARS, Volere

**What the tradition fixes.** [ISO/IEC/IEEE 29148:2018](https://www.iso.org/standard/72089.html) is the current requirements-engineering standard, and the successor to IEEE 830. It names four specification documents: a business requirements specification, a stakeholder requirements specification, a system requirements specification, and a software requirements specification. It adds an operational-concept document beside them. Its contribution is not the document outlines. It is the treatment of one requirement as an identified object with declared attributes. Those attributes are an identifier, an owner, a priority, a source, a rationale, and a status. It adds a **verification method** drawn from a closed set of Test, Demonstration, Inspection and Analysis.

It then states nine characteristics of a well-formed individual requirement: necessary, appropriate, unambiguous, complete, singular, feasible, verifiable, correct, and conforming. It also states five characteristics of a requirement **set**: complete, consistent, feasible, comprehensible, and able to be validated. The split between the two lists is the part worth copying. A set can be inconsistent while every member of it is well formed. That is the same shape as this repository's split between [cohesion and coherence](../spec/04-assurance-model.md#cohesion-and-coherence-are-different-obligations).

[EARS](https://alistairmavin.com/ears/) is the syntax half. Alistair Mavin and colleagues at Rolls-Royce derived it from airworthiness regulations for a jet-engine control system and published it at RE'09 in 2009. It is five patterns and a composite:

| Pattern | Template |
|---|---|
| Ubiquitous | The `<system>` shall `<response>` |
| State driven | **While** `<precondition>`, the `<system>` shall `<response>` |
| Event driven | **When** `<trigger>`, the `<system>` shall `<response>` |
| Optional feature | **Where** `<feature is included>`, the `<system>` shall `<response>` |
| Unwanted behavior | **If** `<trigger>`, **then** the `<system>` shall `<response>` |
| Complex | While `<precondition>`, when `<trigger>`, the `<system>` shall `<response>` |

The [Volere](https://www.volere.org/) shell of James and Suzanne Robertson supplies the third piece: the **fit criterion**, a quantification of the requirement written beside it. That quantification is what makes a subjective statement testable. A requirement whose fit criterion cannot be written is a requirement that nothing can verify. Volere's discipline is to discover that at authoring time rather than at acceptance time.

**What it would type here.** `requirement` and `acceptance_criterion` as kinds, because spec 3 already reserves identifiers for both and because a relation has to name them. The relation is the one 29148 exists for: a requirement is `verified_by` an acceptance criterion, a probe, a check rule, or a test. The reciprocal half says what each verifying artifact answers for. This corpus has three of those four verifying artifact types already, and `probe` is a kind. A check rule is a name the control block reaches through `check:`, and a test is a `code_path` anchor.

EARS maps onto a declaration this taxonomy already has and does not use for this purpose. `regimes.language` carries `controlled` and `profile`, and `.headwater/overlay.yml` sets `controlled: ASD-STE100` for the house prose. `controlled: EARS` sits in exactly that slot, on the `requirement` kind alone. The five patterns are lexically decidable in a way that most language rules are not. A sentence that opens with While, When, Where or If, and carries one `shall`, either matches a pattern or does not. The remediation for a miss names the pattern it should have used. That clears the [fixability](../spec/12-check-layer.md#fixability) bar for an error rather than an advisory finding, which almost nothing in the current language regime does.

Volere's fit criterion is a required section on `acceptance_criterion`, and 29148's verification method is a facet with the closed set the standard already fixes.

**Where it stops.** The nine characteristics are an authoring bar and not a schema. Seven of them — necessary, appropriate, complete, feasible, correct, and both halves of unambiguous — are semantic judgments, and [spec 1](../spec/01-conceptual-model.md) forswears reasoning about judgments like these. A facet that asked an author to self-certify "this requirement is necessary" would be a field that every document sets to true. Only `conforming` is checkable, and it is checkable precisely because EARS turns it into a lexical question. Declaring the other eight would repeat the mistake the design-spec doctrine already named: ten kinds for ten sections, buying no check.

**What has to be reconciled before any of this is declared.** [Q19](../decisions/0019-inbound-integration-an-external-system-of-record.md) rules that imported requirement text is `transcribed` against a committed pin, held to regeneration. That truth stays upstream in an external system of record. That ruling is about an **adopter** whose requirements live in a requirements tool. This engine's own requirements have no upstream and never will, so they would be `asserted` native documents rather than transcribed projections. The two cases are different and the specification has only decided one of them. A `requirement` kind declared without saying which case it serves would put two warrants on one kind and make the Q19 mechanism unreadable.

## Interface contracts: man-pages, Design by Contract, RFC 2119

**What the tradition fixes.** This is the branch that answers the functional-specification gap directly, and it is the cheapest of the three.

[man-pages(7)](https://man7.org/linux/man-pages/man7/man-pages.7.html) fixes a section order for a command-line program, starting with NAME, LIBRARY, SYNOPSIS, CONFIGURATION and DESCRIPTION. The order continues through OPTIONS, PARAMETERS, EXIT STATUS, RETURN VALUE, ERRORS, ENVIRONMENT, FILES, ATTRIBUTES and VERSIONS. It closes with STANDARDS, HISTORY, NOTES, CAVEATS, BUGS, EXAMPLES, AUTHORS, REPORTING BUGS, COPYRIGHT and SEE ALSO. Four are mandatory: NAME, SYNOPSIS, DESCRIPTION and SEE ALSO. EXIT STATUS is expected of anything in manual sections 1 and 8, which is where a program lives. There is no DIAGNOSTICS heading in that list. DIAGNOSTICS is the BSD mdoc convention rather than the Linux one. A section contract that required it would be citing the wrong tradition.

Meyer's [Design by Contract](https://se.inf.ethz.ch/~meyer/publications/computer/contract.pdf) supplies the semantics that a section order does not. A routine declares a **precondition** the caller must satisfy, and a **postcondition** the routine guarantees in return. An **invariant** holds across every routine of the class. The obligation of one party is the benefit of the other, which is the property that makes the contract a specification rather than a description.

For a verb of this engine the three read cleanly. The precondition of `headwater check` is that `.headwater/taxonomy.lock` exists and that `taxonomy resolve` wrote it. That is a condition the engine already enforces and that no document states as a contract. The postcondition is the pair of stream contracts and the exit status. The invariant is the set of properties that hold across every verb. Each takes `--root`, and each writes its artifact to standard output and its account to standard error. No verb writes into the corpus except the ones declared to.

[RFC 2119](https://www.rfc-editor.org/info/rfc2119) and [RFC 8174](https://www.rfc-editor.org/info/rfc8174), together BCP 14, supply the normative vocabulary. The vocabulary is MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, OPTIONAL. 8174's one contribution is the rule that they carry normative force **only** in all capitals. A document invokes them by carrying a fixed boilerplate sentence near its start.

**What it would type here.** One kind, `interface_contract`, and one document per verb. Its reader intent is `behavior`, the same as `design_spec`. The split between them is not purpose but audience and grain. A design spec argues a model to a reader deciding whether to adopt it. An interface contract answers a reader who is about to call the thing. That is a weaker warrant for a separate kind than the doctrine's rule normally accepts. The strong warrant is elsewhere: a relation must name it. An `interface_contract` declares `governs` onto the `code_path` of the crate that implements the verb. The write-time hook already names governing documents on an edit, and it would then name the contract when somebody edits that crate. That is the mechanism this repository has and does not point at its own engine.

The man-page order becomes a `sections: {require: [...]}` contract, which is a live rule here — `section.required.missing` already runs against 154 instances. A defensible subset for a verb of this engine is Synopsis, Description, Options, Exit status, Environment, Files and See also. A Preconditions heading, added from Meyer, belongs too, because the man tradition has no slot for it.

BCP 14 becomes a member on the language regime beside `controlled` and `profile`. It is the one place in this survey where a declaration would need a new member rather than a new value. What a check would read is narrow and worth having. A lower-case "must" in a document whose regime declares BCP 14 is a finding with a mechanical fix. A document that uses a keyword without carrying the boilerplate is a second finding.

**Where it stops.** The remaining seventeen man-page headings are template guidance. Nothing in this model references "the CAVEATS section", no endpoint names it and no expectation windows it.

## Architecture description: 42010, arc42, C4, Views and Beyond

**What the tradition fixes.** [ISO/IEC/IEEE 42010:2022](https://www.iso.org/standard/74393.html) is already cited in the design-spec doctrine, and it is the standard that makes a **viewpoint** an object with a declared form. Its conceptual model separates the entity of interest, the stakeholders, and their concerns. It also separates the architecture viewpoint that fixes how one class of concern is addressed, and the view that applies a viewpoint. It separates the model kinds a viewpoint governs, and the **correspondence rules** that state what must hold between two views. A viewpoint declares three things: the stakeholders it serves, the concerns it frames, and the modeling conventions it uses.

[arc42](https://arc42.org/overview/) is the practitioner form: twelve sections in fixed order. They run from Introduction and Goals through Constraints, Context and Scope, Solution Strategy, and the Building Block, Runtime and Deployment views. Then come Crosscutting Concepts, Architectural Decisions, Quality Requirements, Risks and Technical Debt, and Glossary. The [C4 model](https://c4model.com/), which Simon Brown developed between 2006 and 2011, is four nested levels of abstraction: context, container, component, and code. It is deliberately notation-independent. Clements and colleagues' *Documenting Software Architectures: Views and Beyond* is the book-length treatment of the same discipline, and the design-spec doctrine already cites it.

**What it would type here.** Less than the other two, and that is the honest reading. Spec 6 is already an architecture description in this tradition. What 42010 would add is the declaration that spec 6 is one view among several. That distinction matters only when there is a second view to be inconsistent with. Today there is not.

The part with real value is the correspondence rule, because it is the only checkable object in the standard. A correspondence rule between the crate table in `engine/README.md` and the crates that exist on disk is a rule an engine can decide. It would catch the drift that a hand-maintained table always develops. That is the same argument that produced the `shelf_index` projection, and it arrives at the same answer: the table should be generated rather than checked.

**Where it stops.** The twelve sections of arc42 and the four levels of C4 are template guidance under the doctrine's rule. Neither should become a kind or a facet. C4 in particular is a diagramming discipline, and this corpus commits nothing to diagrams.

## The mapping, in one table

| What the tradition supplies | What it would be here | Why that form |
|---|---|---|
| 29148 requirement as an identified object | kind `requirement` | Spec 3 reserves the identifier, and `verified_by` must name it |
| 29148 acceptance criterion | kind `acceptance_criterion` | Same, and it is the far end of `verified_by` |
| 29148 verification method | facet, closed set Test, Demonstration, Inspection, Analysis | A check reads it and nothing links to it |
| 29148 nine characteristics | nothing | Seven are semantic, and spec 1 forswears reasoning about prose |
| 29148 five set characteristics | nothing new | This is the cohesion and coherence split spec 4 already draws |
| EARS five patterns | `controlled: EARS` on the `requirement` kind | It is a controlled language, and `regimes.language` is where one goes |
| Volere fit criterion | required section on `acceptance_criterion` | Prose structure, and the section contract is a live rule |
| Volere originator, priority, source | facets, or nothing | Each earns its place only where a report reads it |
| man-pages(7) section order | `sections: {require: [...]}` on `interface_contract` | Prose structure that this model already checks |
| Design by Contract | kind `interface_contract`, one per verb | `governs` must reach the crate that implements it |
| BCP 14 keywords | a new member on `regimes.language` | It is a vocabulary rule over a body, like `retired_terms` |
| 42010 viewpoint and view | nothing yet | One view cannot disagree with itself |
| 42010 correspondence rule | a projection, not a check | The crate table should be generated |
| arc42 twelve sections, C4 four levels | template guidance | Nothing in the model references a section |
| 29148 traceability matrix | a projection | See below |

## The output side, which is the part this engine is unusually ready for

29148's classic deliverable is a traceability matrix, and every requirements tool in the field builds one. This engine already builds the graph it would be derived from, and it already has the emitter that would write it. `projections` in `.headwater/overlay.yml` declares `shelf_index` and `shelf_sections`, both held to regeneration by `generate --check`.

A `traceability_matrix` projection over `requirement × verifying artifact` is the same shape. It needs the `verified_by` relation and nothing else. Its value is the cell that is empty. A requirement that no acceptance criterion, probe or rule verifies is the finding that 29148 exists to surface. Here it would be visible in a generated file that a reviewer reads in a diff, rather than in a report somebody has to run.

Two more projections fall out of the same graph at no extra declaration. A verb index over `interface_contract`, which is the missing functional specification as a table of contents. And the crate table of `engine/README.md` is generated from what is on disk, rather than kept aligned by hand. That is the correspondence rule above, arriving as a projection.

## What this evaluation found that nobody had filed

**1. Spec 3 names two identified artifacts that the schema cannot express.** Requirements and acceptance criteria are two of the five things the specification reserves identifiers for. Neither has a kind in the base package, in either entry, or in the local overlay. This is not a gap in a tradition this repository has not adopted. It is an internal inconsistency between spec 3 and every taxonomy source shipped beside it.

**2. The engine is the one artifact in this repository that governs nothing and is governed by nothing.** The corpus root is `docs`, so 22 crates and 293 lines of `engine/README.md` sit outside every rule. The `governs` relation reaches a `code_path`, and the anchor resolver already binds one. 20 code-path anchors are bound in the current graph, so the mechanism exists and points almost nowhere near the engine's own interface.

**3. The obligation and control block of spec 4 is a requirements apparatus pointed at the corpus.** An obligation is an identified invariant with a statement, a rationale, a severity and a disposition. A control declares the mechanism that discharges it and the posture it acts at. That is 29148's requirement-plus-verification-method with different words, and the register that projects it is a traceability matrix. Nothing in this repository says so, and a reader who knows 29148 would recognize spec 4 faster if it did. The corollary is the interesting half: adopting 29148 for the engine would not import a second apparatus. It would point the existing one at a second subject.

**4. EARS is the strongest candidate for an error-severity language rule this repository has found.** The current language regime carries six prose rules, and four of them are advisory because the remedy is a rewrite. An EARS pattern miss names the pattern the sentence should have used, which is a mechanical remediation under the spec 12 fixability bar. That is worth knowing independently of whether a `requirement` kind is ever declared.

**5. Two documents under `docs/` are typed by nothing, and neither is recorded anywhere.** `docs/doctrine/maturity-model.md` and `docs/w3id/README.md` are reported as untyped on every run, and no obligation record, decision or open item names them. The census reports them and the register does not, which is the reporting asymmetry spec 4 warns about in a different context.

**6. A `requirement` kind collides with Q19 unless the ruling is read narrowly.** Q19 decided where an adopter's imported requirement text lives. It did not decide whether a corpus may hold native requirements of its own, and the two carry different warrants. The ruling has to be read or extended before anything is declared. A bundle that declared a `requirement` kind without addressing it would leave two readings live.

## What this evaluation does not do

It declares nothing, and it is not a bundle doctrine. It proposes no `add` to `.headwater/overlay.yml`, no kind, no facet, no shelf, no relation, no regime member and no projection. Every mapping above is a candidate with a stated reason and an estimated cost. Each one would go through the [headwater-taxonomy](../../.claude/skills/headwater-taxonomy/SKILL.md) route with a failing fixture before it shipped.

It also carries the warrant `accepted`. An agent drafted it, the external citations are real and checkable, and a human accepted the reading of them.

## Sources

- [ISO/IEC/IEEE 29148:2018](https://www.iso.org/standard/72089.html) — requirements engineering
- [ISO/IEC/IEEE 42010:2022](https://www.iso.org/standard/74393.html) — architecture description
- [EARS, the official guide](https://alistairmavin.com/ears/) — Mavin and colleagues, Rolls-Royce, first published at RE'09 in 2009
- [Easy Approach to Requirements Syntax](https://ccy05327.github.io/SDD/08-PDF/Easy%20Approach%20to%20Requirements%20Syntax%20(EARS).pdf) — Mavin and Wilkinson, the original paper
- [Volere](https://www.volere.org/) — the Robertsons' requirements shell and the fit criterion
- [man-pages(7)](https://man7.org/linux/man-pages/man7/man-pages.7.html) — the Linux manual page section order
- [Applying Design by Contract](https://se.inf.ethz.ch/~meyer/publications/computer/contract.pdf) — Meyer, IEEE Computer, 1992
- [RFC 2119](https://www.rfc-editor.org/info/rfc2119) and [RFC 8174](https://www.rfc-editor.org/info/rfc8174) — BCP 14
- [arc42](https://arc42.org/overview/) — the twelve-section architecture template
- [C4 model](https://c4model.com/) — Brown, four levels of abstraction
