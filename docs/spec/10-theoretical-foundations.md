---
id: SPEC-HW-theoretical-foundations
status: current
status_since: 2026-08-01
last_verified: 2026-08-11
summary: Where the research literature confirms, sharpens, or contradicts the design, and what changed in the specification as a result.
doc_type: design_spec
sequence: 10
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: [claude-fable-5, claude-opus-5]
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cites_evidence:
    - EVAL-HW-schema-format-walkthrough
    - EVAL-HW-the-measurement-layer
    - EVAL-HW-what-a-check-can-know
---

# 10 — Theoretical foundations

The design so far was derived from practice. This document tests it against the research literature — mainly discourse linguistics, knowledge organization, and architecture-knowledge management. It records where the theory **confirms**, **sharpens**, or **contradicts** what specs 0–9 say.

Each entry states what the theory claims, what it gives us, and what changes as a result. Entries that change nothing still have value: they tell us which decisions are defensible, not only ours.

> **Status.** All twenty changes were applied to the specification — the five structural ones first, then the additive remainder. The table in [§G](#g-summary-of-changes-this-document-proposes) records where each landed. This document is now a record of *why* the design is shaped as it is, not a list of pending work.

---

## A. Coherence

### A.1 Cohesion is not coherence — and we have been conflating them

Halliday and Hasan (*Cohesion in English*, 1976) draw the distinction that the whole design rests on. **Cohesion** is the set of surface ties that bind a text — reference, substitution, ellipsis, conjunction, lexical repetition. **Coherence** is the reader's experience that the text hangs together. Cohesion is a property of the artifact. Coherence is a property of the encounter. Cohesion is necessary, nowhere near sufficient, and — importantly — *mechanically checkable* in a way that coherence is not.

Everything that spec 4 currently checks is cohesion: links resolve, relations are reciprocal, identifiers bind, vocabularies are respected. A corpus can pass all of it and stay incoherent — the documents can each be well-formed and collectively fail to add up.

> **Change:** name the distinction explicitly in the assurance model. Do not imply that a clean check run means a coherent corpus. Cohesion checks are deterministic and blocking. Coherence assessment is sampled and semantic, and it belongs to the audit and probe layers. Two obligations, two mechanisms. Do not pretend that one covers the other.

### A.2 Nuclearity — some relations have an end the other depends on

Rhetorical Structure Theory (Mann & Thompson, 1988) analyzes text as a hierarchy of spans joined by coherence relations. It makes a distinction that we lack. Relations are either **nucleus–satellite** (hypotactic — the satellite supports the nucleus, and its removal does not destroy the point) or **multinuclear** (paratactic — the spans are co-equal). RST's ~23 relations are less important than that structural asymmetry.

Our relations are currently flat: `supersedes`, `governs`, `derives_from`, `cites` all look alike to the engine. But they are not alike. A derived AI rule is a *satellite* of the standard that it points at. Delete it, and you lose nothing that the nucleus does not still carry. A specification is a *nucleus*. Its contract sidecar is a satellite. Two specifications that describe halves of one seam are *multinuclear*.

> **Change:** add `nuclearity` to the relation declaration. It pays for itself immediately. Satellite documents inherit lifecycle from their nucleus (a satellite of a superseded document is itself stale, automatically). Pruning for context budgets drops satellites before nuclei. Orphan detection distinguishes "unlinked nucleus" (a real problem) from "unlinked satellite" (a bug in generation).

### A.3 Coherence relations cluster into a few families

Hobbs (1985) and Kehler (*Coherence, Reference, and the Theory of Grammar*, 2002) both argue that the large relation inventories collapse to a small number of families. Kehler's families are **Resemblance**, **Cause–Effect**, and **Contiguity**. The practical lesson from decades of RST annotation is that large flat relation sets produce poor inter-annotator agreement. Humans cannot reliably choose between forty labels.

An open relation vocabulary in a taxonomy will grow out of control in the same way, and adopters will apply it inconsistently.

> **Change:** relation types declare a **family** from a small fixed set (`succession`, `derivation`, `governance`, `evidence`, `composition`, `association`). Families carry default checking semantics, so a new relation type inherits sensible behavior. The schema validator can also flag a taxonomy that grew twelve near-synonymous relations in one family.

### A.4 Intentional structure — the missing dimension

Grosz and Sidner ("Attention, Intentions, and the Structure of Discourse", *Computational Linguistics*, 1986) decompose discourse into three components that interact: **linguistic structure** (the utterances), **intentional structure** (the purposes behind segments, and the dominance and satisfaction-precedence relations between them), and **attentional state** (what is in focus now).

In corpus terms: linguistic structure is the file tree that we model well. The intentional structure is *purpose and the subordination of purposes*, which we model only weakly (a shelf implies purpose, a `type:` facet refines it). Attentional state is exactly the agent context-window problem of spec 5.

The gap is dominance. We record that document A supersedes B, but not that A's *purpose* subordinates B's — that B exists in service of A. That is the relation that a reader most needs to decide what to read. It is also the one that an agent needs to decide what to load.

> **Change:** treat purpose as an explicit declaration — every kind declares the reader intent that it serves. Reading precedence between linked documents is derived from nuclearity, succession, and the governance family. (As first applied, this was a per-relation `dominance` declaration. The core-concepts review found it redundant wherever those two already determined it, and unused elsewhere, so it is now derived, not declared. The Q2 walkthrough later found governance unanswered rather than unused, and added the third clause.) This is the theoretical justification for the routing layer in spec 5: routing is a search over intentional structure, not over text. It also reframes context budgeting as attentional-state management, which is a better-posed problem than "fit under N tokens".

### A.5 Local coherence: continuity of focus

Centering Theory (Grosz, Joshi & Weinstein, *Computational Linguistics*, 1995) models local coherence as continuity of the entity in focus across adjacent utterances. It ranks transition types by the inference cost that they impose.

The corpus analog is cheap and useful. A link between two documents with no shared subject — no common component, domain, identifier, or anchor — is a **focus shift**. A shelf full of them is a corpus that a reader cannot traverse without a re-orientation at every hop.

> **Change:** an advisory *transition-continuity* check. For each relation edge, compute shared facets and anchors. Report edges with no continuity. Advisory forever — some shifts are legitimate. But the *distribution* is a genuine corpus-health metric. It is our first metric that measures coherence rather than cohesion.

### A.6 Hypertext already learned this

Thüring, Hannemann and Haake ("Hypermedia and Cognition: Designing for Comprehension", *CACM* 38(8), 1995) studied comprehension in networked documents and identified two forces. Coherence is the positive influence. **Cognitive overhead** is the negative one — the cost to decide where to go, and to hold your place while you go there. Their design principles: make relations explicit and typed, preserve context across transitions, and supply overview maps.

This is the closest prior art to what we build, and it validates three choices. The choices are typed relations over bare links, generated overview projections, and pointers-with-summaries rather than content in the routing layer. (Each pointer is a navigation decision, and every navigation decision costs.)

> **Change:** none — but use cognitive overhead as an explicit design budget. Every added navigation hop needs a justification, and "the taxonomy is elegant" is not one.

---

## B. Relationships and classification

### B.1 Kruchten's decision-relationship ontology

Kruchten ("An Ontology of Architectural Design Decisions in Software-Intensive Systems", 2004) lists relationships between design decisions: *constrains, forbids, enables, subsumes, conflicts with, overrides, comprises, is bound to, is an alternative to, is related to, traces to, does not comply with.* <!-- ste-lint: allow sentence-length # Kruchten's relation vocabulary, quoted entire -->

Our decision relations are `supersedes` / `superseded_by` / `refines` — the temporal axis only. Kruchten's set is mostly *logical*: `conflicts with` and `constrains` say things about simultaneously live decisions that succession cannot express. A corpus that holds two current decisions that conflict is incoherent in a way that no reciprocity check will ever detect.

> **Change:** ship Kruchten's set (or a defensible subset) as the default taxonomy's decision-relation vocabulary. Do not invent our own. `conflicts with` and `constrains` in particular buy real checks: a conflict between two `current` decisions is a finding. A decision constrained by a superseded one needs review.

### B.2 Faceted classification — the discipline behind facets

Ranganathan's facet analysis and its later formalization (Vickery, 1960) is the grounding for spec 2's facet model. The transferable rules: facets should be **orthogonal** (a document's value on one facet must not determine its value on another), and each facet needs a stated **principle of division**. Ranganathan's canons give real acceptance tests — *differentiation* (the facet must actually separate documents), *relevance* (to the purpose of the scheme), *ascertainability* (the author can determine the value without ambiguity), and *permanence* (the value must not change for incidental reasons).

> **Change:** add these as schema-validation checks. Orthogonality is measurable over an existing corpus — if two facets' values are near-perfectly correlated, one is redundant and the validator should say so. Ascertainability becomes a requirement: every enum facet value must document how to choose it. This turns "is this a good facet?" from taste into a test.

### B.3 OntoClean — why kind and state must stay separate

Guarino and Welty's OntoClean validates taxonomies with meta-properties: **rigidity** (does the property hold for an instance in every possible world?), **identity**, **unity**, and **dependence**. Its central rule is that an anti-rigid class cannot subsume a rigid one.

Apply it to us: *specification* is rigid — a document cannot cease to be a specification and stay the same document. *Draft* is anti-rigid — a document passes through it. Thus lifecycle state must never be modeled as a kind. Spec 3 already separates them. OntoClean explains **why**, which matters the next time that someone proposes a `docs/drafts/` shelf, or a kind called "deprecated standard".

> **Change:** none to the model — but use the meta-properties as the taxonomy-review vocabulary, and add a validator check that flags kinds whose names match state-facet values. Cheap, and it catches the single most common taxonomy mistake.

### B.4 SKOS and ISO 25964 — the federation answer already exists

SKOS (W3C, 2009) and ISO 25964 standardize exactly what spec 2 defines ad hoc: concept schemes, and hierarchical (broader/narrower), associative (related), and equivalence relations. Critically, they also standardize **mapping relations between schemes** (`exactMatch`, `closeMatch`, `broadMatch`, `narrowMatch`).

That mapping vocabulary is the unsolved part of Q9 (multi-repository corpora). Two divisions with different taxonomies do not need a merged taxonomy. They need declared mappings between their concept schemes, which is a solved problem with a standard.

> **Change:** model cross-taxonomy federation as SKOS-style mapping relations rather than as a merge. Consider whether to emit the resolved taxonomy as SKOS/RDF alongside the native lock. That gives free interoperability with existing KOS tooling, and a sanity check that our model is expressible in a standard one.

### B.5 Traceability information models — our idea has a name and a literature

A **Traceability Information Model** declares permitted artifact types, link types, and their directions, and it drives the tracing tools. That is exactly "taxonomy as configuration". Ramesh and Jarke ("Toward reference models for requirements traceability", *IEEE TSE*, 2001) derived reference models empirically from practice. One finding: organizations cluster into low-end and high-end traceability users, which is exactly our "small team vs regulated platform" split in spec 2.

The Grand Challenge of Traceability work (Gotel, Cleland-Huang et al., 2012; revisited 2017) carries the harder lesson: traceability fails not on modeling but on **creation and maintenance cost**. Manually created links decay because the person who pays the cost is not the person who receives the benefit.

> **Change:** use the reference-model framing and — more importantly — treat link creation cost as an explicit design constraint. Every relation in the default taxonomy must be derivable from work that the author already does (a commit, a template field, a scaffolded document). If it is not, it will not survive contact with a deadline. Add "what creates this edge, and who pays?" to the review checklist for relation declarations.

### B.6 Provenance has a standard model

W3C PROV (entities, activities, agents: `wasDerivedFrom`, `wasRevisionOf`, `wasGeneratedBy`) already models what our lineage relations approximate. That includes the agent dimension, which matters now that both humans and LLMs draft documents.

> **Change:** align lineage relation semantics with PROV, and record the generating agent on derived artifacts. "Which of these documents were agent-drafted, and did a human accept them?" becomes a query rather than an archaeology exercise.

---

## C. Documents as social objects

### C.1 Genre theory grounds "kind" — and warns us

Yates and Orlikowski (*Academy of Management Review*, 1992; *ASQ*, 1994; *Journal of Business Communication*, 2002) define **genres** as socially recognized types of communicative action, characterized by shared *purpose* and *form*. They define a **genre repertoire** as the set that a community actually enacts. They define **genre systems** as sequences of interrelated genres that structure work.

Three consequences:

1. Our "kind" is a genre. Purpose plus form is exactly the kind declaration — theoretically solid, and it confirms that a kind without a stated purpose is incoherent by construction.
2. Genres are **emergent and social**, not imposed. A taxonomy handed down and frozen will be worked around. This is the strongest theoretical argument for overlays and versioned evolution — and against a single blessed taxonomy.
3. **Genre systems are sequences**, and we do not model sequence at all. Proposal → decision → specification → evidence is a genre system. So is incident → postmortem → standard change.

> **Change:** add optional **sequence expectations** to the taxonomy — declared chains of kinds where one is expected to follow another. This yields the detective control that we currently lack. It finds decisions with no downstream specification, incidents with no postmortem, and proposals that were accepted and then never implemented. That is drift that the cohesion checks cannot see, and it is the failure mode that people complain about most. (As first applied, these were a separate top-level declaration. The core-concepts review showed that every declared chain was a single windowed hop, and folded them into [relation participation](02-taxonomy-model.md#participation-expectations). That fold also forced the window-origin definition that the mechanism lacked.)

### C.2 Boundary objects — publisher and consumer

Star and Griesemer (*Social Studies of Science*, 1989) define **boundary objects** as artifacts "plastic enough to adapt to local needs… yet robust enough to maintain a common identity across sites", weakly structured in common use and strongly structured in local use. <!-- ste-lint: allow sentence-length # verbatim definition, not ours to split -->

That is the taxonomy package, stated better than spec 7 states it. But it carries a requirement that we do not yet meet: for identity to hold across sites, something must be *invariant*. Our overlay algebra currently lets a consumer override or remove almost anything, which means that two consumers of "the same" taxonomy may share nothing.

> **Change:** the taxonomy package declares a **core** — the kinds, relations, and facets that overlays may extend but never remove or redefine. Conformance checks the core, not the whole. This gives the publisher a real answer to "are they still using our method?" and gives the consumer a bounded, legible customization surface.

### C.3 The design-rationale capture bottleneck

The design-rationale tradition — IBIS (Kunz & Rittel, 1970), gIBIS (Conklin & Begeman, 1988), QOC (MacLean et al., 1991), Toulmin's argument structure (1958) — produced rich models and almost no sustained adoption. Grudin's analysis of why is the durable result: **capture costs the author and benefits someone else, later.** Every system that ignored that asymmetry died.

Our decision records inherit the entire problem, and our evidence stop rule *increases* author cost in exchange for corpus quality. That trade may well be right, but it is the trade that historically kills these systems.

> **Change:** treat author cost as a tracked metric, not an afterthought. This is the strongest argument in the whole document for the agent-assisted authoring path of spec 5. An agent can draft the record from the evidence already in the commit, the ticket, and the conversation. That shifts the cost off the author — the first genuinely new answer to Grudin's objection in thirty years. If we cannot show that shift, we should expect the same adoption curve as gIBIS.

---

## D. Evolution and variability

### D.1 Ontology evolution is not schema evolution

Noy and Klein ("Ontology Evolution: Not the Same as Schema Evolution", *KAIS*, 2004) show that ontology change differs from database schema change. There is no clean separation between evolution and versioning, and **compatibility is multi-dimensional**. Instance-data preservation, consequence preservation, and consistency preservation are distinct properties, and a change can preserve one while it breaks another.

Spec 2 originally carried a version table (minor for additive, major for anything else) — exactly the naive schema-evolution model that this paper argues against. The addition of an optional facet is "additive". Yet it can change which documents a projection includes, or make a previously valid corpus fail a completeness check.

> **Change:** replace the coarse semver table with declared **compatibility dimensions**, evaluated against a real corpus. `headwater taxonomy diff` should report per dimension: does every existing document still classify to the same kind (identity preservation)? does every check that passed still pass (consequence preservation)? do projections produce the same outputs? The version number becomes a *consequence* of the measured impact, not a publisher's guess about it.

### D.2 Overlays are a variability problem with prior art

Research on software product lines — feature models (Kang et al., 1990), staged configuration (Czarnecki et al.), and delta-oriented programming (Schaefer et al.) — fully worked the "base plus modifications" ground. Delta modeling in particular formalizes exactly our add/override/remove operations. That includes application-order constraints, and the conditions under which a delta set is confluent.

> **Change:** borrow the confluence requirement explicitly. Spec 2 says that overlays that conflict are an error. The delta literature says how to *decide* that statically. Use the check rather than derive it again, and state that overlay application must be confluent as a schema-validation property.

---

## E. Cognition and retrieval

### E.1 Information foraging — routing has a theory

Pirolli and Card (*Psychological Review*, 1999) model information seeking as foraging. Readers follow **information scent** — proximal cues that predict distal value — and abandon a patch when the scent weakens. Scent quality, not corpus quality, determines whether anything is found.

This is the theoretical justification for spec 5's pointers-with-summaries design, and it relocates where the effort should go. A perfect document with a vague summary is invisible.

The computational descendants sharpen it further, and they decide where a cue lives. SNIF-ACT and the Bloodhound line model a link choice as a utility computed over **the links available at the current position**. They model patch leaving as that utility falling below what another patch offers. Scent is therefore never an absolute property of a target. It is a comparison over the options at the point of decision, which means that every scent measure owes a comparison set. For a summary, that set is the documents a reader is choosing between. For a cue on a relation, it is the other links that the same document offers.

> **Change:** treat summary quality as an assurance concern in its own right rather than a front-matter formality. Frame the routing confidence gate in scent terms. The engine stays silent when scent is weak, because a cue that misleads costs more than an absent one. Probe categories map directly onto foraging outcomes. (As first applied, spec 5 called the `summary` facet the entire scent surface. [Q20](09-decisions.md#q20--where-scent-lives) later found that the theory places a second cue on the referring edge, and that each measure states the set it compares against.)

### E.2 Cognitive dimensions — how to evaluate the schema language

Green and Petre's **cognitive dimensions of notations** framework (1996) is the standard instrument for the evaluation of a notation. The dimensions include viscosity (cost of change), hidden dependencies, premature commitment, role-expressiveness, error-proneness, abstraction gradient, and others.

Q2 (schema format) is currently an argument about YAML versus CUE. It should be an evaluation.

> **Change:** settle Q2 with a cognitive-dimensions walkthrough over concrete authoring scenarios (add a kind, rename a shelf, split a facet). Hidden dependencies and viscosity are the dimensions that will decide it. Note that our overlay design *deliberately trades* viscosity for hidden dependencies. That is exactly the trade-off that the framework exists to make visible.

### E.3 Minimalism

Carroll's minimalist doctrine (*The Nurnberg Funnel*, 1990) says: cut everything that does not support action, and start from the reader's task. It is the research behind spec 0's "every rule earns its place", and behind the size budgets of spec 5. We cite it exactly because that principle is the one that people push back on.

---

## F. Empirical software engineering

### F.1 Ground the obligations in observed defects

Aghajani et al. ("Software Documentation Issues Unveiled", ICSE 2019; "Software Documentation: The Practitioners' Perspective", ICSE 2020) derived a taxonomy of documentation defects from mining and from practitioner surveys. The taxonomy spans information content (incorrect, incomplete, outdated), presentation, and process.

Our obligation set in spec 4 was invented from experience. There is no reason for that when an empirically derived defect taxonomy exists.

> **Change:** derive the default obligation set from a published defect taxonomy, and record the mapping. Every obligation then answers "which observed defect class does this prevent?" That is a better filter than intuition, and a real answer to "why are you making me do this?".

### F.2 Stale documentation is still used

Lethbridge, Singer and Forward (*IEEE Software*, 2003) found that engineers rely on documentation that is known to be out of date. They use it as an imperfect but valuable guide.

> **Change:** none — this is direct empirical support for spec 3's decision that staleness is detective and never blocking. A gate on freshness would remove artifacts that are demonstrably still useful.

### F.3 Rational reconstruction is legitimate

Parnas and Clements ("A Rational Design Process: How and Why to Fake It", *IEEE TSE*, 1986) argue that the real process is never rational. To document it *as if* it were rational is both honest and valuable, provided that the reconstruction is labeled.

This sits in productive tension with our evidence stop rule. Reconstruction of rationale after the fact is legitimate. *Fabrication* is not. The line between them is whether the reconstruction is grounded in auditable evidence and marked as reconstructed.

> **Change:** the stop rule stays, but a **reconstructed** provenance value is added alongside evidence-backed and gap. That is the honest third option, and its absence currently pushes authors toward one of the other two.

### F.4 Agent context files drift — measured, not assumed

Recent work on agent context files ("Agent READMEs: An Empirical Study of Context Files for Agentic Coding", 2025, and the 2026 studies of rule taxonomies in AI IDEs) finds that these files grow over time, are inconsistently maintained, and drift from the codebase that they describe. That is the same decay curve as any other documentation, on a shorter timescale.

> **Change:** none — this is empirical support for the two things that spec 5 already insists on. Rule files are *generated projections* checked in CI rather than hand-maintained prose, and they are size-budgeted. The literature says that hand-written agent context is an asset that decays. Our design already refuses to hand-write it.

### F.5 Structured retrieval versus embeddings

Current GraphRAG and KG-RAG work reports that graph-structured retrieval outperforms pure vector similarity on multi-hop and relational queries. Vector retrieval stays stronger for fuzzy lexical matching. Hybrid designs are the practical default, at the cost of two indexes to keep in step.

> **Change:** none — this supports spec 5's "graph first, embeddings at most a fallback". Note the literature's operational warning in Q6: a second index is a second thing to keep fresh. Our corpus is small enough that the graph should win outright.

### F.6 Write skew names the anomaly, and read sets detect it

Two changes that are each valid against the merge base can produce an invalid corpus. That failure has a name outside this project, and the name comes from concurrency control rather than from version control.

Berenson, Bernstein, Gray, Melton, O'Neil and O'Neil ("A Critique of ANSI SQL Isolation Levels", SIGMOD 1995) named **write skew**. Two transactions read overlapping data, write disjoint data, and each one preserves an invariant that the pair violates. Snapshot isolation permits the anomaly, because it detects a write-write conflict and nothing else. That is exactly git. A branch is a transaction, the merge base is its snapshot, and a textual conflict is a write-write conflict at line grain.

Fekete, Liarokapis, O'Neil, O'Neil and Shasha ("Making Snapshot Isolation Serializable", *TODS* 2005) found the condition. The anomaly needs a cycle with two consecutive read-write antidependencies, which makes it analyzable rather than mysterious. Cahill, Röhm and Fekete ("Serializable Isolation for Snapshot Databases", SIGMOD 2008) turned that into a run-time mechanism, and PostgreSQL ships it as its serializable level. It works by tracking what each transaction **read**, and it accepts a false abort as the price.

The transfer is precise and it is favorable. Detection of this anomaly needs a read set. A database adds read tracking to get one, and git has none at all. Headwater already computes a read set per check instance, because scope enforcement makes the cache key a hash of exactly the in-scope inputs. The ingredient is a byproduct of a decision taken for caching.

> **Change:** state that validity is not preserved under merge, as a property of a verdict rather than as a caution ([spec 4](04-assurance-model.md#a-verdict-is-about-one-state-of-the-corpus)). Report the read set with every run, and treat a merge as an ordinary change ([spec 12](12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict)). Fail toward re-running, as serializable snapshot isolation fails toward aborting.

---

## G. Summary of changes this document proposes

| # | Change | Spec affected | Source | Status |
|---|---|---|---|---|
| 1 | Separate cohesion (checkable) from coherence (sampled) as distinct obligation classes | 4 | Halliday & Hasan | **applied** |
| 2 | Add `nuclearity` to relation declarations | 2 | RST | **applied** |
| 3 | Group relation types into a small fixed set of families | 2 | Hobbs, Kehler | **applied** |
| 4 | Declare purpose explicitly. Derive reading precedence from nuclearity, succession, and governance | 2, 5 | Grosz & Sidner | **applied, revised** |
| 5 | Advisory transition-continuity check over relation edges | 4 | Centering Theory | **applied** |
| 6 | Adopt Kruchten's decision-relation vocabulary, incl. `conflicts with` | 2 | Kruchten | **applied** |
| 7 | Facet acceptance tests: orthogonality, ascertainability, permanence | 2, 6 | Ranganathan, Vickery | **applied** |
| 8 | Validator check: no kind may name a lifecycle state | 2, 6 | OntoClean | **applied** |
| 9 | Federate via SKOS-style mapping relations, with optional SKOS export | 7, Q9 | SKOS / ISO 25964 | **applied** |
| 10 | Every relation declares what creates it and who pays | 2 | Traceability grand challenge | **applied** |
| 11 | Align lineage with PROV. Record generating agent | 2, 3 | W3C PROV | **applied** |
| 12 | Windowed participation expectations (genre systems) | 2, 4 | Yates & Orlikowski | **applied, revised** |
| 13 | Taxonomy packages declare an immutable core | 7 | Star & Griesemer | **applied** |
| 14 | Track author capture cost as a metric | 3, 5 | Grudin | **applied** |
| 15 | Replace semver rules with measured compatibility dimensions | 2, 7 | Noy & Klein | **applied** |
| 16 | Require confluence of overlay application, statically checked | 2 | Delta modeling | **applied** |
| 17 | Treat `summary` as the scent surface. Frame the routing gate in scent terms | 5 | Pirolli & Card | **applied** |
| 18 | Settle the schema-format question by cognitive-dimensions walkthrough | Q2 | Green & Petre | **applied** |
| 19 | Derive default obligations from an empirical defect taxonomy | 4 | Aghajani et al. | **applied** |
| 20 | Add `reconstructed` as a third provenance value | 3 | Parnas & Clements | **applied** |
| 21 | State that validity is not preserved under merge. Report the read set | 4, 12 | Berenson et al., Cahill et al. | **applied** |

All are applied. The first twenty came from one sweep of the literature, and change 21 arrived later with [Q21](09-decisions.md#q21--terminological-succession-and-validity-under-merge). The five structural changes (2, 4, 12, 13, 15) landed first, because they altered the schema itself. Change 3 came with them, since change 4's precedence semantics needed families to exist first. The remaining fourteen were additive and landed against the schema as it then stood.

Where they ended up:

| Spec | What these changes added |
|---|---|
| [1 — Conceptual model](01-conceptual-model.md) | Purpose on kinds, family and nuclearity on relations, windowed participation expectations |
| [2 — Taxonomy model](02-taxonomy-model.md) | The five structural changes, plus the decision-relation vocabulary, `created_by`, PROV alignment, facet acceptance tests, the rigidity rule, overlay confluence, and cross-taxonomy mappings |
| [3 — Authoring](03-authoring-and-lifecycle.md) | Three-state evidence basis, recorded provenance with `accepted_by`, and the assisted-fraction metric |
| [4 — Assurance](04-assurance-model.md) | The cohesion/coherence split, defect-derived obligations, transition continuity, absence findings, cost-aware adaptive reporting, and what a verdict is about |
| [5 — AI integration](05-ai-integration.md) | Purpose-first routing, satellite-first pruning, and scent measurement |
| [6 — Engine](06-engine-architecture.md) | The `validate` / `audit` split and the enlarged check inventory |
| [7 — Distribution](07-distribution-and-federation.md) | The invariant core, measured compatibility, and federation by mapping |
| [9 — The decision register](09-decisions.md) | A decision procedure for Q2, and the cross-taxonomy half of Q9 closed |
| [12 — Check layer](12-check-layer.md) | The read set of a run, and the merge treated as an ordinary change |

### What the theory did not settle

The application of every change does not mean that the design is finished. Three things that the literature sharpened but could not decide, all still open in [spec 13](13-open-obligations.md):

- **Whether the assisted fraction actually rises.** The strongest claim in the design — that agent-assisted authoring answers the capture-cost objection that killed every prior rationale system — is now falsifiable, measured, and untested. Nothing here proves it.
- **Whether the taxonomy language survives contact with authors.** The [walkthrough](../evaluations/schema-format-walkthrough.md) has run, and it kept the format that the rest of the spec is written in. What it could not do is meet a real author. It scored notations against scenarios, and a scenario is not an adopter.
- **Whether coherence measurement is worth its noise.** Transition continuity is a proxy, defensible in theory, unvalidated in practice. If its distribution is stable across healthy and unhealthy corpora alike, it measures nothing and should be cut.

## H. Theory considered and set aside

- **Diátaxis** (tutorial / how-to / reference / explanation). It is elegant and widely adopted, but it classifies by *reader mode* for end-user documentation, and our corpus is organized by *authority and lifecycle*. Values statements, registers, and decision records do not map onto the four modes without distortion. It is available as an optional facet for adopters who want it, but it is not the spine.
- **Formal argumentation frameworks** (Dung, 1995). This is a rigorous account of attack and defeat between arguments, and it genuinely applies to decisions that conflict. But it demands a formalization cost that no author will pay. Revisit only if `conflicts with` edges become common enough to need automated resolution.
- **Full OWL/description-logic semantics.** This is reasoning power that we do not need, at a cost in authoring difficulty and validation time that we cannot afford. SKOS is deliberately the weaker, cheaper standard, and it is the right one here.
- **Speech act theory** for normative language. It is an attractive framing for MUST/SHOULD/MAY, but RFC 2119 already gives us the operational subset, and the deeper theory adds no checks.

## References

Discourse and coherence — Halliday & Hasan, *Cohesion in English* (1976) · [Mann & Thompson, *Rhetorical Structure Theory* (1988)](https://www.semanticscholar.org/paper/Rhetorical-Structure-Theory:-Toward-a-functional-of-Mann-Thompson/af5100605a3b6bfd0adf9a30e69a47d1b98340ba) · Hobbs, *On the Coherence and Structure of Discourse* (1985) · Kehler, *Coherence, Reference, and the Theory of Grammar* (2002) · Grosz & Sidner, *Attention, Intentions, and the Structure of Discourse* (1986) · Grosz, Joshi & Weinstein, *Centering* (1995) · [Thüring, Hannemann & Haake, *Hypermedia and Cognition* (1995)](https://dl.acm.org/doi/10.1145/208344.208348)

Knowledge organization — [Kruchten, *An Ontology of Architectural Design Decisions* (2004)](https://philippe.kruchten.com/wp-content/uploads/2009/07/kruchten-2004-design-decisions.pdf) · Ranganathan, facet analysis · Vickery, *Faceted Classification* (1960) · [Guarino & Welty, *An Overview of OntoClean*](https://www.loa.istc.cnr.it/old/Papers/GuarinoWeltyOntoCleanv3.pdf) · SKOS (W3C, 2009) · ISO 25964 · W3C PROV-O

Traceability — Gotel & Finkelstein (1994) · Ramesh & Jarke, *Reference models for requirements traceability* (2001) · [Gotel, Cleland-Huang et al., *The Grand Challenge of Traceability*](https://arxiv.org/abs/1710.03129)

Social and organizational — [Yates & Orlikowski, *Genres of Organizational Communication* (1992)](https://www.semanticscholar.org/paper/Genres-of-Organizational-Communication:-A-Approach-Yates-Orlikowski/bbe0a59e50ae8d4cb25124eaa157db807988ea9e) · [Orlikowski & Yates, *Genre Repertoire* (1994)](https://www.semanticscholar.org/paper/Genre-Repertoire:-The-Structuring-of-Communicative-Orlikowski-Yates/d6de1ae4f0cadbf088894a04b38aecaf704ee787) · [Yates & Orlikowski, *Genre Systems* (2002)](https://journals.sagepub.com/doi/10.1177/002194360203900102) · [Star & Griesemer, *Institutional Ecology, 'Translations' and Boundary Objects* (1989)](https://journals.sagepub.com/doi/10.1177/030631289019003001)

Design rationale — Kunz & Rittel, *IBIS* (1970) · Toulmin, *The Uses of Argument* (1958) · Conklin & Begeman, *gIBIS* (1988) · MacLean et al., *QOC* (1991) · Grudin, *Evaluating opportunities for design capture* (1996)

Evolution and variability — [Noy & Klein, *Ontology Evolution: Not the Same as Schema Evolution* (2004)](https://link.springer.com/content/pdf/10.1007/s10115-003-0137-2.pdf) · Kang et al., *FODA* (1990) · Schaefer et al., delta-oriented programming

Cognition — Pirolli & Card, *Information Foraging* (1999) · Green & Petre, *Cognitive Dimensions* (1996) · Carroll, *The Nurnberg Funnel* (1990)

Concurrency control — Berenson, Bernstein, Gray, Melton & O'Neil, *A Critique of ANSI SQL Isolation Levels* (SIGMOD 1995) · Fekete, Liarokapis, O'Neil, O'Neil & Shasha, *Making Snapshot Isolation Serializable* (*ACM TODS* 2005) · Cahill, Röhm & Fekete, *Serializable Isolation for Snapshot Databases* (SIGMOD 2008)

Empirical software engineering — [Aghajani et al., *Software Documentation Issues Unveiled* (ICSE 2019)](https://2019.icse-conferences.org/details/icse-2019-Technical-Papers/49/Software-Documentation-Issues-Unveiled) · Aghajani et al., *Software Documentation: The Practitioners' Perspective* (ICSE 2020) · Lethbridge, Singer & Forward (2003) · Parnas & Clements, *A Rational Design Process* (1986) · [*Agent READMEs: An Empirical Study of Context Files for Agentic Coding* (2025)](https://arxiv.org/pdf/2511.12884) · [*Rule Taxonomy and Evolution in AI IDEs* (2026)](https://arxiv.org/pdf/2606.12231)
