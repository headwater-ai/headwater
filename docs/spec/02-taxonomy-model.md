---
id: HW-SPEC-taxonomy-model
status: current
status_since: 2026-08-01
last_verified: 2026-08-12
summary: The central design, which declares kinds, facets, relations, regimes, the immutable core, and customization by overlay.
doc_type: design_spec
sequence: 2
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: [claude-fable-5, claude-opus-5]
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cites_evidence:
    - HW-EVAL-default-taxonomy-first-run
    - HW-EVAL-graph-export-and-federation
    - HW-EVAL-relation-storage
    - HW-EVAL-schema-format-walkthrough
    - HW-EVAL-the-serving-boundary
    - HW-EVAL-warrant-and-adjudication
    - HW-EVAL-what-a-check-can-know
---

# 2 — The taxonomy model

**This is the central design of the system.** Everything else is downstream of it.

> **Revision note.** This document contains five structural changes that [theoretical foundations](10-theoretical-foundations.md) proposed. The changes are: nuclearity on relations, windowed participation expectations, an immutable semantic core, versioning by measured compatibility, and purpose as an explicit declaration. Reading precedence is derived from nuclearity, succession, and the governance family. Relation families arrived as a dependency of the precedence change. The [core-concepts review](../reviews/) later cut the per-relation `dominance` declaration, which was redundant wherever nuclearity or succession already determined the order. The review also called it unused elsewhere. The [schema-format walkthrough](../evaluations/schema-format-walkthrough.md) found that governance was unanswered rather than unused, and the derivation now carries a clause for it. That walkthrough made four further changes here. Endpoints are the only declaration of a permitted relation, abstract kinds have semantics, compatibility gained an `addressability` dimension, and the migration payload rewrites overlays. The [first-run walkthrough](../evaluations/default-taxonomy-first-run.md) then made three more, all in what the default taxonomy enables. The base enables `governs`, no base relation is `created_by: author`, and optional content ships as add-only bundles.

## The problem being solved

In the systems that this replaces, the taxonomy exists in at least three places at once. It exists as prose in a standards document, as constants in each linter, and as path globs in the AI instruction files. Those three copies drift. Worse, the third copy is *executable*.

A customer whose documentation culture differs must thus fork the tooling to express the difference. The difference can be different shelf names, an extra document kind, or a lifecycle with an extra state. That customer then cannot take upstream fixes.

The taxonomy must thus be:

- **declarative** — data, not code.
- **complete** — nothing structural known only to the engine.
- **validated** — the schema itself has a schema.
- **composable** — customization by overlay, never by fork.
- **versioned** — a change is a release, with a migration path.
- **explainable** — the engine can justify every classification decision.

## Shape

A taxonomy is one logical document. The engine assembles it from a base package plus zero or more overlays, and it resolves to a single validated object. The form below is YAML, for illustration. [Q2](09-decisions.md#q2--schema-format) made that the binding decision.

```yaml
taxonomy: acme-engineering
version: 3.2.0
extends: headwater/standard@2.1.0        # base package, or null for from-scratch

vocabularies:                          # authoring sugar: named value sets facets reference
  lifecycle_state:
    - {value: draft,      role: initial}
    - {value: current,    role: live}
    - {value: superseded, role: terminal-retained}
    - {value: deprecated, role: terminal-retained}
  audience: [engineer, operator, integrator, auditor]

purposes:                              # reader intents the corpus serves
  rationale:
    intent: explain why a choice was made and what it forecloses
    answers: ["why is it this way", "what was rejected", "what does this constrain"]
  behavior:
    intent: state what the system does, as it is now
    answers: ["what does this component do", "what may I rely on"]
  procedure:
    intent: enable a reader to carry out a task correctly
    answers: ["how do I do this", "what do I do when X happens"]

facets:
  status:
    role: state                        # engine-significant role
    values: $vocabularies.lifecycle_state
    required: true
  status_since:
    role: state_entered                # stamped on transition; the origin windows are measured from
    type: date
    required: true
  last_verified:
    role: freshness
    type: date
    stale_after_days: 180
  summary:
    role: scent                        # what routing and indexes surface
    type: string
    required: true
  owner:
    type: string
    required: false
  audience:
    values: $vocabularies.audience
    required: false
    severity: warn

regimes:
  voice:
    declarative:
      forbid: [future_intent, change_narration, phased_rollout]
    narrative: {}
  lifecycle:
    standard:
      initial: draft
      transitions:
        draft:      [current, deprecated]
        current:    [superseded, deprecated]
        superseded: []
      retain_terminal: true            # a change that deletes one of them is refused
  language:
    default:
      tag: en-US                       # BCP 47: language and variant in one tag
      controlled: ste-house            # none | ste-house | ste-strict
      retired_terms:                   # a term this corpus no longer uses
        - term: reference system
          reason: the design stands on its own, and the attribution added risk
        # replacement is optional; with one, the fix is mechanical
        - term: docgov
          replacement: Headwater
          reason: the name was settled in Q10

relations:
  supersedes:
    family: succession
    from: [decision]
    to:   [decision]
    inverse: superseded_by
    reciprocal: required
    nuclearity: multinuclear           # both ends stand alone
    on_target: {set_state: superseded}
    created_by: scaffold               # who pays for this edge

  derives_from:
    family: derivation
    from: [agent_rule]
    to:   [standard]
    nuclearity: nucleus-satellite
    nucleus: to                        # the standard is the nucleus
    inherits: [status, last_verified]  # the satellite tracks its nucleus
    created_by: generator

  governs:
    family: governance
    from: [standard, specification]
    to:   [code_path]                  # an external anchor kind
    cardinality: many
    created_by: author

  conflicts_with:
    family: association
    from: [decision]
    to:   [decision]
    reciprocal: symmetric
    nuclearity: multinuclear
    invalid_when: {both: {status: current}}   # two live conflicting decisions

  implemented_by:
    family: evidence
    from: [decision]
    to:   [specification]
    created_by: hook                   # proposed at review time from the change

anchors:                               # non-document node types relations may target
  code_path:     {resolver: source-tree}
  ado_work_item: {resolver: ado-snapshot}   # reads a committed pin, never a live service

shelves:
  decisions:
    path: docs/decisions/**
    homogeneous: true
    kind: decision
    group_by: domain                   # sub-directory is a facet value
  specifications:
    path: docs/specifications/**
    homogeneous: true
    kind: specification
    layout: "{domain}/{component}/{aspect}.md"
  governance:
    path: docs/governance/**
    homogeneous: false
    discriminator: doc_type            # facet naming the kind
    kinds: [standard, methodology, runbook, reference, register, values]

kinds:
  decision:
    purpose: rationale                 # required; a kind without one is invalid
    identifier: {scheme: decision_id}
    voice: declarative
    lifecycle: standard
    facets:
      require: [status, status_since, last_verified, domain, summary]
      forbid:  [doc_type]              # placement already states the kind
    sections:
      require: [Context, Decision, Consequences]
      optional: [Alternatives considered, Related]
    relations:                         # what a decision may link to is derived from relation endpoints
      expect:                          # windowed participation: finds what should exist and does not
        - id: decision-realized
          relation: implemented_by
          to_kind: specification
          when: {status: current}
          within: 90d
          since: state_entered         # window origin: the state-entry date facet
          severity: warn
          rationale: a decision nothing implements is either not a decision or not done

identifier_schemes:
  decision_id:
    pattern: "{namespace}-DR-{seq:04d}"  # the namespace opens the identifier
    allocation: reconcile-first        # never reuse; scan before minting
                                       # no namespace here: a package cannot name one

core:                                  # what overlays may never remove or redefine
  requires:
    - facet_role: state
    - facet_role: freshness
    - facet_role: scent
    - purpose: rationale               # some kind must serve it
    - purpose: behavior
    - relation_family: succession
      lifecycle_sensitive: true
    - identifier_scheme: decision_id   # rewrite the pattern at will, never remove the scheme

projections:
  - kind: shelf_index
    for: [decisions, governance]
    output: "{shelf}/README.md"
  - kind: agent_rules
    output: .agent/rules/
  - kind: site_nav
    output: .headwater/nav.yml
  - kind: graph_export                 # a projection that leaves the repository
    profile: partner                   # the audience it serves
    format: json
    output: .headwater/export/partner.json
    filter: {exclude: {confidentiality: [internal, secret]}}
    tombstone: counted                 # counted | sealed
  - kind: transcription                # requirement text copied from a pinned snapshot
    from: {anchor: ado_work_item}      # the resolver that owns the pin
    output: docs/requirements/
```

## The thirteen declarations

| Declaration | Answers |
|---|---|
| `purposes` | What reader intents the corpus serves |
| `facets` | What metadata documents carry, its shape, and how hard it is enforced |
| `regimes` | Reusable rule bundles: voice, lifecycle, and language |
| `relations` | What typed links exist, their family, endpoints, nuclearity, and reciprocity |
| `anchors` | What non-document node types exist, and which resolver owns each |
| `shelves` | How the corpus is partitioned, and what each partition means |
| `kinds` | What each species of document is, what it requires, and what it is expected in time to link to |
| `identifier_schemes` | How stable identifiers are shaped, namespaced, and allocated |
| `core` | What an overlay may never remove or redefine |
| `mappings` | How this taxonomy's concepts correspond to another's |
| `projections` | What derived artifacts are generated, and where they land |
| `obligations` | What invariants the corpus commits to, and how much each one matters |
| `controls` | What discharges each obligation, when it runs, and at what posture |

An earlier draft counted twelve and then added two more in the next sentence. Four of those fourteen are gone deliberately, and what each protected survives without its name.

`sequences` was folded into windowed relation participation on kinds (below). `profiles` are publisher-shipped overlays ([spec 7](07-distribution-and-federation.md#profiles-are-publisher-overlays)). `compatibility` named the engine's fixed measurement dimensions, which no taxonomy could legally vary. A declaration with one legal value is an engine constant. `vocabularies` remains as authoring syntax — a named value set that facets reference. It is validated as part of `facets`, and it is not a concept that anyone must learn first.

The count does not move when an adopter serves a filtered audience. An **export profile** is an entry under `projections`, with a named audience, a filter over facet values, and a tombstone grain. It is not a declaration of its own. The reason is the one that removed `profiles` and `compatibility`: a use of an existing mechanism earns no name of its own ([spec 6](06-engine-architecture.md#an-export-profile-carries-a-filter), [Q17](09-decisions.md#q17--governed-access-and-the-solution-layer)).

The count also does not move when a corpus imports content from a system that it does not govern. The pin is an anchor kind with one resolver, and the imported text is a `transcription` projection over that pin. Neither half is new, and [Q19](09-decisions.md#q19--inbound-integration-an-external-system-of-record) closed on that reading.

Two declarations arrived from the assurance model, and the count went from eleven to thirteen. [Spec 4](04-assurance-model.md#obligations-are-data) writes `obligations` and `controls` as blocks of a taxonomy, and the language had no room for either one. The test that removed `profiles` also applies to these two, and both pass it. An obligation is not a species of document, and it is not a property of one. It is a claim about the corpus, and every other declaration describes documents or the links between them. A facet that carried the claim would attach it to one document at a time. A projection that carried it would make the generated register the source of the binding rather than the view of it, which spec 4 refuses.

The price is the one this section states for any root. To add a root is a meta-schema change, and this change also cost the sublanguage a rule. Both blocks are keyed by the identifier, and every identifier that spec 4 writes holds a hyphen, so [a segment](#the--reference-sublanguage) now admits one. The key is the identifier because an address never reaches inside a list. A bundle that adds a kind also adds the obligations that its rules serve, and each such entry needs an address of its own.

One declaration is here that no earlier draft had: `anchors`. Relation endpoints referenced anchor kinds (`code_path`) that nothing ever declared. Identity, resolver ownership, and referential integrity for anchors all hung on a name that was used but never defined. The count went up because a real corner of the model was missing, which is the one honest reason that it may.

## Purpose is declared, not implied

**Every concrete kind declares the reader intent that it serves**, or inherits it from an [abstract parent](#abstract-kinds). A concrete kind with no purpose fails schema validation.

This makes the genre-theoretic definition operational: a genre is a socially recognized type, defined by a shared *purpose* and *form*. The rest of a kind declaration — sections, facets, voice — is form. Without purpose, a kind is a shape with no reason. The first question that anyone asks about a corpus ("what is this shelf *for*?") then has no answer in the schema.

Purposes are declared once, at the taxonomy level, and kinds reference them. Several kinds may serve one purpose — a decision record and an architecture note can both serve `rationale`. But a kind that serves two unrelated purposes is a signal to split the kind, and the validator says so.

Purpose does real work downstream:

- **Routing** ([spec 5](05-ai-integration.md)) matches a task's intent against declared purposes before it matches text. "Why is it like this?" resolves to `rationale` kinds. "what does it do?" resolves to `behavior` kinds. This is a search over intentional structure, not over prose. It is far cheaper and more precise than lexical ranking alone.
- **`headwater explain`** states a document's purpose alongside its kind. A reader who opens a document thus knows what it is *for* before they read it.
- **The core** (below) is expressed in terms of purposes. This lets an adopter rename everything and still run the same method in a recognisable way.

## Language is declared, not assumed

Every corpus is written in a natural language, in a variant of that language, and sometimes in a controlled profile of that variant. Most systems leave all three implicit. The cost surfaces as style rules that nothing enforces. This project met the cost in its own corpus: the ruling for American spelling lived in an instruction file, and a manual sweep applied it. The schema had no slot to hold the ruling. That is configuration expressed as convention — the failure that this specification exists to remove.

The three axes are different decisions, and a taxonomy must not conflate them:

- **Language and variant** — one BCP 47 tag carries both (`en-US`, `en-GB`, `de-DE`).
- **Controlled profile** — none, or a named profile such as STE house (structural rules) or STE strict (closed dictionary). A profile is a promise about the text, so checks can read it.

Language is the third regime family. The corpus declares one default, and a kind may bind a different profile — runbooks in the strict profile, specifications in the house profile. Checks parameterize on the declaration: the spelling lexicon, the sentence-length limits, and the controlled-vocabulary check all read the regime instead of a hard-coded assumption. A check that cannot state its language assumption is not portable between corpora.

### The language regime carries the terms that the corpus retired

A judgment that a corpus no longer uses a term is a promise about the text, on the same terms as a spelling lexicon. A judgment that lives in prose and a diff is inherited by nobody, and `retired_terms` is where it becomes data ([evaluation](../evaluations/what-a-check-can-know.md)). Each entry carries the term, a required reason, and an optional replacement. `language.retired_term.used` reads them, and this repository declares eighteen entries: the stock phrasing that its own contributing guide used to state in prose.

**The reason is required, because a retirement with no recorded reason is an authority rank with extra steps.** [Q18](09-decisions.md#q18--recording-adjudicated-disagreements) refused that shape for adjudication, and it does not improve here.

**The replacement decides fixability.** With one, the fix is a substitution, which meets the mechanical-and-total bar of [spec 12](12-check-layer.md#fixability), and the check offers a patch. Without one, the finding carries remediation prose and no patch. A retired term is the usual case for the first shape. A retired *framing* is the usual case for the second, and no lexicon repairs it for the author.

**The lexicon belongs to the language regime and never to a voice regime.** A voice regime binds per kind, and the `narrative` value exempts a kind from voice rules entirely. A term that this corpus retired is retired in a proposal as much as in a specification. The declaration count does not move, for the reason that it did not move for an export profile. A use of an existing mechanism earns no name of its own.

**A new entry forces a major version, and the migration state absorbs the existing text.** A retired term makes checks that passed fail, which breaks the `consequence` dimension below. So the release ships a migration payload, and the payload already knows which rules it broke for which documents. Those findings are `migration-pending` at `(document, rule)` grain, with an owner and an expiry ([spec 7](07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states)). No separate grandfathering mechanism is needed, and a mechanism with no owner and no expiry would be worse than this one.

A monolingual corpus declares no `language` facet on documents. Such a facet has one value everywhere, and no check varies on it, so it fails the every-rule-earns-its-place test ([spec 0](00-vision-and-scope.md#design-principles)). A multilingual corpus promotes language to a facet, and the promotion pays for itself at once. Checks select their lexicon per document. The AI surface knows what language it reads and answers in. And a `translation_of` relation becomes declarable, with the standard expectation machinery behind it. When a source document changes, its translations fall stale. That is the ordinary freshness check, not a new mechanism.

The doctrine starter kit declares `en-US` with the house profile as its default. The base package declares `en-US` with `controlled: none`, and the difference between the two is deliberate. A controlled profile that the base turns on meets an adopted corpus with a wall of findings on its first run ([Q3](09-decisions.md#q3--how-much-of-the-default-taxonomy-ships-in-the-box)).

## Relation families and nuclearity

Every relation type carries two properties — one chosen, one usually inherited. Together they turn a flat link set into a structure that the engine can reason about.

### Family

Every relation belongs to exactly one of six families:

| Family | Meaning | Default nuclearity | Lifecycle-sensitive |
|---|---|---|---|
| `succession` | One artifact replaces or revises another | multinuclear | yes |
| `derivation` | One artifact is generated or distilled from another | nucleus–satellite | yes |
| `governance` | One artifact constrains another, or constrains code | — | yes |
| `evidence` | One artifact substantiates a claim in another | — | no |
| `composition` | One artifact is part of another | nucleus–satellite | yes |
| `association` | Related, with no stronger claim | multinuclear | no |

The fixed family set is deliberate. Open relation vocabularies sprawl, and readers apply them inconsistently once the list passes about a dozen entries. Decades of discourse annotation agree on that finding. A family supplies default semantics, so a new relation type inherits sensible checking without a declaration of its own. The validator can also flag a taxonomy that grew five near-synonymous relations inside one family.

### Nuclearity

A relation is **multinuclear** (both ends stand alone) or **nucleus–satellite** (one end supports the other and cannot stand without it). The family supplies the default. A relation type that contradicts its family's default must say so explicitly, and `taxonomy audit` reports every override. When a family is overridden more often than followed, the audit reports the family declaration itself as misassigned. That is a finding with an owner and a remediation (reassign the members, or change the default), not an observation left in a report.

One thing that the family cannot supply is *which end* is the nucleus, because relation direction is the choice of the taxonomy author. An author may spell a derivation `derives_from` or in the opposite direction, and a composition `part_of` or `comprises`. The family is the same in both spellings. Nucleus–satellite relation types thus always name their nucleus end.

This asymmetry pays for itself in four places:

- **Lifecycle inheritance.** A satellite inherits declared facets from its nucleus. A generated rule that derives from a superseded standard is stale the moment the standard is superseded. The engine detects this structurally, with no separate check.
- **Context pruning.** When an agent's context budget binds, the engine drops satellites before nuclei ([spec 5](05-ai-integration.md)). To drop the standard and keep its teaser is exactly backwards, and without nuclearity the engine cannot tell.
- **Orphan detection.** An unlinked nucleus is a real finding: something exists that nothing points at. An unlinked satellite is a *generation* bug — different severity, different owner, different fix. To conflate the two produces noise that is suppressed wholesale.
- **Deletion safety.** To remove a nucleus that still has live satellites is a finding. To remove a satellite is free.

### Reading precedence is derived

When two documents are linked, no declaration states which one governs the reading. Properties already declared entail it. The earlier per-relation `dominance` field was cut when the entailment was noticed:

- **On a nucleus–satellite relation, the nucleus governs.** A satellite supports the other end and cannot stand without it. A document that cannot stand alone cannot have its purpose govern the reading of the document that it depends on.
- **On succession, the successor governs.** That is the whole meaning of the family: the successor is the one that governs behavior now.
- **On governance between two documents, the source governs.** The family means that the source constrains the target. A reader who wants to know what holds reads the constraint first.
- **Everything else carries no reading order.** `conflicts_with` and `is_alternative_to` assert exactly that neither end subordinates the other, and to impose an order would misstate the relation. Evidence is the same case. An artifact that substantiates a claim does not govern the reading of the document that makes the claim.

The four clauses are total over the six families, which the earlier three were not.

**The governance clause is a correction.** The earlier list stopped at succession, so a governance edge between two documents fell to the last clause and carried no reading order at all. That contradicts what the family means. The cut `dominance` field held this one case and nothing else. The core-concepts review removed it as redundant wherever nuclearity or succession decided the order, and unused where they did not. The first half of that reading was right. The second was not, because governance is precisely where nothing else decided the order, and the review read silence as no demand. The clause replaces the field, because the family already states the answer and no taxonomy needs to repeat it.

**Reading precedence is not nuclearity.** A governance relation still declares no nuclearity, and the family table's cell stays blank. Nuclearity asks whether an end stands alone, and a constrained document stands alone perfectly well. Reading precedence asks which end a reader consults first. This family answers the two questions differently, which is why one declaration could never have served both.

The engine uses the derived precedence in three places. It orders routing results, chooses which document a conflict is reported against, and decides reading order in generated indexes. A real corpus may still produce a multinuclear, non-succession relation whose ends genuinely need an order. That outcome is the case to reintroduce a declaration, and that is the time to argue it.

### The decision-relation vocabulary

Succession is the temporal axis, and on its own it is not enough. Two decisions can both be current and contradict each other. One decision can constrain another and not replace it. Neither is expressible with `supersedes` alone, and neither is detectable by any reciprocity check.

The default taxonomy therefore adopts the decision-relation set from Kruchten's ontology of architectural design decisions, each assigned to a family:

| Relation | Family | Claim |
|---|---|---|
| `supersedes` | succession | Replaces a prior decision |
| `overrides` | succession | Displaces a prior decision's effect and does not retire it |
| `constrains` | governance | Narrows what the target may decide |
| `forbids` | governance | Rules out an option in the target's space |
| `enables` | governance | Makes the target decidable — the weak form of `constrains` |
| `does_not_comply_with` | governance | Records a known, deliberate violation |
| `subsumes` | composition | Is wider than, and implies, the target |
| `comprises` | composition | Is made up of the target decisions |
| `conflicts_with` | association | Contradicts the target |
| `is_alternative_to` | association | Was a considered alternative |
| `is_bound_to` | association | Must change together with the target |
| `traces_to` | evidence | Derives from a requirement, driver, or source |

To adopt a published vocabulary rather than invent one is deliberate. This set was in use and under criticism for two decades, and the arguments about where its edges sit already occurred.

**`overrides` is the adjudication edge, and [Q18](09-decisions.md#q18--recording-adjudicated-disagreements) closed on it.** A human who settles a live disagreement writes a decision, and that decision overrides the one whose effect it displaces. The claim in the table above states the semantics exactly: the loser stays, and only its effect goes. So `overrides` declares an inverse and required reciprocity, in the way that `supersedes` does. Without the inverse, a reader who arrives at the losing document learns nothing, and `check --fix` has no back-link to write.

Four checks come with it, none of which succession alone can express:

- two `current` decisions joined by `conflicts_with` — an incoherent corpus state.
- a decision `constrains`-linked to a superseded one — the constraint may be void.
- `forbids` and `enables` edges that assert opposite things about one option.
- a `does_not_comply_with` edge that points at a `current` standard, which is a registered deviation and must carry an owner and an expiry.

A taxonomy may enable any subset. The default enables four: `supersedes`, `conflicts_with`, `constrains`, and `traces_to`. This keeps the live-conflict and void-constraint checks, at the cost of the `forbids`/`enables` and `does_not_comply_with` checks. Those checks belong to the regulated column of the worked example anyway, and they arrive with the regulated platform's overlay as *additions*.

**A fifth default relation comes from outside this vocabulary, and the count above hid the need for it.** Every relation in the table runs between decisions, because that is what an ontology of decisions contains. A default drawn only from this table thus gives a corpus no edge to code. A behavior-serving kind then has no permitted edge at all, and every document of that kind is an orphan finding. The base therefore also enables `governs`, from a governed document to the `code_path` anchor. It carries write-time impact detection ([spec 5](05-ai-integration.md)), and spec 0's promise that a code path resolves to its governing documents rests on it. The [first-run walkthrough](../evaluations/default-taxonomy-first-run.md) found this only when it wrote the base out as YAML. No reading of the table shows it.

An earlier draft enabled all twelve and expected a small team's overlay to remove most of them. That was backwards twice over. Defaults are the learnability surface for exactly the adopter ([spec 0](00-vision-and-scope.md#who-this-is-for)) with no taxonomist on staff. And the design's own sprawl warning — readers apply relation sets inconsistently past about a dozen entries — was aimed at its own default. A first taxonomy experience that consists of `remove:` lines is friction spent to delete things that nobody asked for.

**Defined and enabled are distinct states, and both have semantics.** The *package* defines the full vocabulary — endpoints, family, generated checks, doctrine — as a library of complete, named declarations. A *taxonomy* enables a relation when it carries the declaration in the resolved result. The base pulls in five. An overlay enables another by reference (`add: {relations.forbids: $package.optional.forbids}`), with no restatement, through the same `$`-reference syntax that vocabularies already use. A defined-but-unenabled relation does not exist as far as a corpus is concerned. An edge that names it is an ordinary unknown-relation finding, the relation generates no checks, and it appears in no template. This is what makes the minimal default nearly free for the regulated adopter — to enable the rest is a line per relation, not a redeclaration.

**Past relations, enabling by reference needs a bundle.** One line works for a relation in this vocabulary, because such a relation references only kinds that the base already has. An optional *kind* does not. `kinds.control` needs a purpose, an identifier scheme, facets, a shelf, and the relations that make it more than a shape. One `add` line for it produces five dangling references. Optional content therefore ships as a **bundle**: a named, add-only publisher overlay with a declared dependency closure ([spec 7](07-distribution-and-federation.md#bundles-are-publisher-overlays-in-the-other-direction)). A bundle holds no `override` and no `remove`, so any subset of bundles commutes and resolves under the confluence check that the resolver already runs. [Abstract kinds](#abstract-kinds) are what make that possible. A bundle's new kind names `governed_document` as its parent, and joins the base relations with no endpoint edit. No bundle thus has to override an endpoint list.

### Who creates each edge

**Every relation declares `created_by`**, from a closed set: `author`, `scaffold`, `generator`, `hook`, `agent`, `import`. It is required, and a taxonomy that omits it fails validation.

This exists because of the single most consistent finding in the traceability literature. Trace links decay when their creation costs the author and benefits someone else later. The field forces the question at design time — *what creates this edge, and who pays?* — not after the corpus quietly stops maintenance of the edge.

It is also measurable after the fact. `headwater taxonomy audit` reports edge counts and staleness by creator. A relation declared `created_by: author` but present on 4% of eligible documents visibly receives no maintenance. The remedy is usually to move it to `scaffold` or `generator`, not to exhort authors harder.

The default taxonomy assumes that remedy from the start. **No relation that it enables is `created_by: author`.** That value is reserved for overlay additions which an adopter explicitly chooses.

An earlier statement of this rule named scaffold, generator, and hook as the permitted creators, and the base contradicts it. A scaffold proposes `supersedes`. A hook proposes `traces_to` and `governs` from the change. Nothing mechanical proposes `conflicts_with` or `constrains`, because both come from the coherence sweep, which is an agent. The narrower rule is the one that does the intended work, because the claim under test is that unassisted human capture decays.

The claim that assisted authoring raises edge capture is the strongest and least-tested in the system ([spec 10](10-theoretical-foundations.md#what-the-theory-did-not-settle)). A default that only works if the claim holds is a bet, and a default that survives when the claim fails is a design. Two of the base's five edges depend on that claim. `taxonomy audit` reports edge counts and staleness by creator, so the dependence is measurable rather than assumed.

### Endpoints are the only permission

A relation declares its endpoints, and nothing else declares them.

An earlier draft also listed permitted relations on each kind, under `relations.may`. The two declarations fixed the same set of permitted pairs, from opposite ends, and no rule made them agree. A taxonomy where `supersedes.from` named a kind whose `may` omitted it passed validation, and the relation was unusable in that taxonomy.

That is the failure that [placement is primary](#placement-is-primary-metadata-fills-the-gap) forbids for the discriminator facet. A second statement of one fact in time disagrees with the first. The rule was right and it was not applied here. So `may` is gone. The permitted set for a kind is derived — every relation whose `from` names the kind, plus the inverse of every relation whose `to` names it.

The reading need that `may` served is real, and it survives without a declaration. `headwater explain <path>` prints the permitted relations for a document, and the template for a kind offers them. A declaration that exists to be read is a projection, not a source.

### Instance attributes, and which end owns each one

An edge carries data of its own. [Q20](09-decisions.md#q20--where-scent-lives) puts an optional cue on a reference, and an importer records the upstream revision that it checked an edge against ([Q19](09-decisions.md#q19--inbound-integration-an-external-system-of-record)). Neither fits on a bare pointer, so a relation type declares the attributes that its instances may take.

```yaml
relations:
  cites:
    family: association
    from: [governed_document]
    to:   [governed_document]
    attributes:
      cue: {type: string, owner: source}   # why this reference, from here

  traces_to:
    family: evidence
    from: [specification]
    to:   [ado_work_item]
    created_by: import
    attributes:
      verified_revision: {type: string, owner: edge}   # the pinned revision this edge was checked against
```

Three rules keep the attribute surface from becoming the ungoverned second syntax that [Q4](09-decisions.md#q4--relation-storage) just closed.

**Declared, not free.** An attribute that the relation type does not declare is a finding. The meta-schema owns the attribute declaration, exactly as it owns a facet declaration.

**The value space is a facet's value space.** A free scalar, a date, an enum with a controlled vocabulary, or a list of any of those. Never a reference. [Spec 1](01-conceptual-model.md#facet) removed reference-valued facets because they were a second ungoverned edge mechanism, and a reference-valued attribute would be a third one. A connection is a relation. An edge that needs to point at a node is a request to make the edge a node. That is a change to the model, not a type in this table.

[Q18](09-decisions.md#q18--recording-adjudicated-disagreements) held that request and gave it back. An adjudication needs a named adjudicator, a date, a scope, and a reason, and a thing with all four is a document. So the model change is refused, and the surface stays as this section describes it. An earlier draft showed `adjudicated_by` and `adjudicated_on` on a `conflicts_with` edge, and both are gone.

**One owning end.** `owner` is `source`, `target`, or `edge`. A source-owned attribute on a symmetric relation gives one value per direction, which is what a cue needs. An edge-owned attribute declared at both ends with different values is a finding, and no fix resolves it, because reconciliation is a judgment.

`created_by` stays a property of the relation type and does not move here. It states an intent about who maintains the edges of that type, and [`taxonomy audit`](#who-creates-each-edge) measures the intent against a real corpus. A per-instance value would answer a different question and would leave that measurement with no baseline.

### Lineage aligns with PROV

The `derivation` and `succession` families map onto W3C PROV: `derives_from` to `prov:wasDerivedFrom`, `supersedes` to `prov:wasRevisionOf`, and generated projections to `prov:wasGeneratedBy`. Alignment is deliberate — provenance is a solved modeling problem. A match with a standard costs nothing, and it makes the graph interoperable with tooling that already exists.

It also brings PROV's agent dimension, which now matters: humans, agents, and both together draft documents. See [authoring and lifecycle](03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed).

**The alignment stops one field short, and the specification says so rather than implying more.** PROV records what happened to an entity and who took part. It has no vocabulary for endorsement, so it cannot say that a party stands behind a result. `accepted_by` is that addition, and the [warrant](01-conceptual-model.md#warrant) is what makes the addition visible. PROV does supply one warrant value under a standard name. `prov:Quotation` is the repeat of part or all of an entity by somebody who may not be its original author. That is the `transcribed` value exactly ([spec 11 §P](11-adjacent-work.md#p--provenance-endorsement-and-the-record-of-a-judgment)).

### Behavior at the limits

A real corpus finds the edges of this machinery quickly. Every case below is declared here, not left for an implementation accident to settle. All of them are generated Graph checks ([spec 12](12-check-layer.md)) — they come with the family, not with adopter code.

- **Succession, derivation, and composition are acyclic.** `A supersedes A`, a mutual succession, or a longer cycle leaves a corpus with no live end. A `derives_from` loop means that satellite inheritance never terminates. A `comprises` cycle is nonsense. All three families reject self-reference and cycles. Association may legitimately cycle. Governance and evidence edges are directed, so their cycles are expressible — and **legal**. Two decisions genuinely can constrain each other, and evidence can be mutual. Nothing downstream depends on an order over these families — no inheritance, no live end, no part-of hierarchy — so a cycle breaks no semantics. Acyclicity is confined to the three families where a cycle destroys what the family means, not applied wherever it sounds hygienic. Self-reference stays invalid in every family except association. A document that constrains or evidences itself is a modeling error, not a relationship.
- **Unwarranted content may not govern the reading of warranted content.** [Reading precedence](#reading-precedence-is-derived) says which end of an edge governs. Where that end carries the `asserted` [warrant](01-conceptual-model.md#warrant) and the other end does not, the edge is a finding. One rule covers three cases. An asserted standard constrains an accepted specification. An asserted successor displaces an accepted decision. An asserted nucleus carries an accepted satellite. A list of forbidden relations would miss the next relation that an adopter adds. The rule is silent about an edge that ends on an anchor, because such an edge carries no reading precedence ([Q15](09-decisions.md#q15--a-synthesized-content-tier)).
- **Duplicate edges collapse to one, with a finding.** Two identical declarations of one relation between the same endpoints are a single edge and an advisory finding. The cause is usually a merge artifact, never a stronger claim.
- **Inverses that disagree are a finding, not a choice.** Where both ends author their half, the halves can disagree — B names a successor that is not the document that names B. The engine prefers neither side. It reports the pair, and the corpus is incoherent there until an author resolves it.
- **A satellite with two nuclei inherits nothing contested.** Where the inherited facet values agree, inheritance proceeds. Where they disagree — one nucleus `current`, the other `superseded` — the engine does not silently pick a value. The conflict is a finding against the satellite. A satellite that declares a value that its nucleus also supplies keeps its local value, and the divergence is itself a finding. Silent shadowing is how inherited staleness disappears.
- **Anchors are endpoints without document semantics.** Nuclearity, reading precedence, and lifecycle interaction are defined between documents. A relation that ends on an external anchor carries none of them. An anchor has no purpose and no lifecycle, which is why the family table's nuclearity cells are blank for governance and evidence. What an anchor endpoint does carry is **identity**. Each anchor type is declared in `anchors` and owned by exactly one resolver. Anchor strings are normalized before comparison, so two spellings of one target are one node. An anchor that no resolver claims is a finding. Write-time impact detection ([spec 5](05-ai-integration.md)) fires on these identities, so anchor resolution is a correctness root, not an edge case. **A resolver reads repository content or a committed snapshot, and never a live service.** That keeps check time offline ([spec 0](00-vision-and-scope.md#non-negotiables)) and it makes a resolution result reproducible. Two cases already rest on the rule: a requirements snapshot ([Q19](09-decisions.md#q19--inbound-integration-an-external-system-of-record)) and a pinned corpus export at the federation tier ([spec 7](07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it)). **Resolution has three outcomes.** An anchor resolves, or it fails to resolve, or the source that owns it withheld the target under a declared export filter. The third outcome is `withheld`, and it is never counted as the second. One is somebody's declared decision, and the other is a defect ([Q17](09-decisions.md#q17--governed-access-and-the-solution-layer)).

### Disagreement is adjudicated, not ranked

Two documents can both be current, both be well-formed, and disagree on a fact — a standard says one thing, a specification another. An earlier draft answered this with a per-kind **authority rank** consumed by an `on_disagreement` rule. It is cut, for three reasons that survive any rewording:

- **The trigger is unobservable.** To detect that two documents disagree *on a fact* is to reason about what prose asserts. [spec 1](01-conceptual-model.md#two-layers-terminology-and-assertions) explicitly forswears that, and [spec 4](04-assurance-model.md#declaration-moves-the-boundary) confirms that it is undecidable structurally. The rule could only ever fire *after* a human or a coherence-sweep finding already identified the disagreement.
- **At that point, the adjudication is the data.** Whoever identified the disagreement knows which side is right for this case. Spec 4's design rule says to record that judgment — as a declared edge, a correction, or a succession. It says not to pre-answer the judgment with a scalar that someone chose before the question existed.
- **A scalar cannot express the semantics that the prose demanded.** "A specification outranks a standard about its own component and is silent about anything else" is scoped precedence. `authority: 20` is a global ordering, which is exactly the backwards behavior that the old prose warned against.

What survives needs no numbers: where sources conflict and no declared adjudication exists, an agent cites both and flags the conflict ([spec 5](05-ai-integration.md)).

**The positive half is settled, and it needs no declaration either** ([Q18](09-decisions.md#q18--recording-adjudicated-disagreements)). An adjudication is a decision, written by a human, and it carries `overrides` against the document whose effect it displaces. The adjudicator is the name in `accepted_by`, which the engine already enforces. The scope is the pair of endpoints plus the prose. Readers and agents inherit the ruling through derived reading precedence, because the successor governs on a succession edge. A later document may supersede the adjudication, which is what a lifecycle is for.

Three properties follow, and each one was a reason to cut the rank. The judgment happens once, when a human writes the document. It is recorded as data, so everything downstream is ordinary graph work ([spec 4](04-assurance-model.md#declaration-moves-the-boundary)). And the precedence is scoped, because the endpoints scope it.

## Contract sidecars: the specification as oracle

Prose is canonical for meaning and hopeless for precision. A component may therefore carry a **contract sidecar**: machine-verifiable artifacts beside the prose. Examples are schemas, interface descriptions, metric definitions, and structured acceptance criteria that carry stable identifiers.

```
specifications/ingest/parser/
  functional.md          # prose: what it does and why — canonical for meaning
  technical.md           # prose: how it is realized
  contracts/
    input.schema.json    # canonical for shape
    acceptance.yml       # identified, structured criteria
```

The prose stays canonical for intent, and the sidecar is canonical for the exact shape. The prose references the sidecar and does not restate it. This is decomposition by **validation regime** — the two halves are checked by different means — not a new shelf and not a file split for its own sake.

What this unlocks is the more interesting part. Structured acceptance criteria with stable identifiers are a **test oracle**. The engine can derive a conformance check from the specification. Nobody then writes the check alongside the specification, and the check does not drift from it. The check asks the corpus what should be true, so coverage follows the specification automatically, with no parallel maintenance.

That closes the loop that the whole system is built around. A specification that can generate the check that proves it is a specification that cannot quietly become false. That is the strongest available form of "documentation describes what is". It also sets an important boundary: a generated check confirms *structure*, never meaning. A criterion can be present, well-formed, mechanically satisfied, and still describe the wrong behavior. Structural conformance is a floor, and the audit layer ([spec 4](04-assurance-model.md)) is the ceiling.

Sidecars are optional. A component with nothing mechanically checkable carries none, and the taxonomy declares which kinds may have them.

## Participation expectations

A kind may declare that its documents are **expected to participate** in a relation. A document of this kind, in a given state, should acquire the named relation to a document of another kind within a window.

```yaml
kinds:
  decision:
    relations:
      expect:
        - id: decision-realized
          relation: implemented_by
          to_kind: specification
          when: {status: current}
          within: 90d
          since: state_entered
          severity: warn
          rationale: a decision nothing implements is either not a decision or not done
```

An earlier draft declared these as a separate top-level concept, `sequences`, sold as chains. Every declared chain was in fact a single hop: *kind + state ⇒ expected relation, within window*. A chain is three expectations that share endpoints. A single hop is a state-conditional, windowed, detective-posture participation constraint — the `required` end of the cardinality spectrum that a relation already has, plus a clock. So it is declared on the kind, beside the other obligations that a kind carries, and the separate concept is gone. What it models is unchanged: genre theory's *genre system* — proposal → decision → specification → evidence, and incident → postmortem → standard change.

Expectations catch a failure class that nothing else catches. Every check in [spec 4](04-assurance-model.md) validates artifacts that exist. An expectation finds the artifact that **should exist and does not**. Examples are the accepted proposal that nobody implemented, the incident with no postmortem, and the decision that never reached a specification. That is the drift that people actually complain about. It is invisible to link and front-matter validation, because there is nothing malformed to find.

**The window has a declared origin.** `within: 90d` is meaningless until the question *ninety days from what?* has an answer in the graph. The earlier draft had none — no state-entry or creation date existed anywhere, which made the flagship absence check uncomputable from declared data. So the origin is now part of the declaration: `since:` names an engine-significant date role — `state_entered` (stamped by the transition that put the document in the state that triggers the expectation) or `created` for expectations with no state condition. An expectation whose origin facet is not required on the kind that declares it fails `taxonomy validate`. This keeps the check pure ([spec 12](12-check-layer.md)): origin date plus injected clock, no history walk, no git archaeology.

**The origin is maintained, not merely present.** A window computed from a date that nothing defends is a window that silently never starts, so the stamp carries a contract of its own:

- Its correct value is the date that the transition *landed*, not the date that someone noticed. Whatever performs the transition stamps it — scaffold, hook, agent, or author. A generated change-scoped check ([`needs_prior`](12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version)) enforces the pairing. A diff that moves the state facet but not the state-entry date is a finding. A stamp in the future, or earlier than the prior stamp, is also a finding. The contract is checked at the only moment that it is cheaply fixable — while the transition is still in the diff.
- `check --fix` stamps the date only when the transition sits in the same diff. It never reconstructs a missing date after the fact. An invented origin silently rewrites every window measured from it. A human enters a lost entry date, or it stays a finding.
- State changes caused by edges stamp too. `on_target: {set_state: superseded}` sets the target's state-entry date in the same operation. A state change with no stamp is a defect regardless of what caused the change.
- **Re-entry into a state resets the window, deliberately.** The state-entry date describes the current state's entry, so a lifecycle machine with a cycle re-arms any expectation conditioned on the re-entered state. That is usually the right semantics — re-acceptance restarts the clock on realization. But it is a gaming route, so the churn is visible. `taxonomy audit`'s state-dwell and transition-count distributions make a flip-flop to re-arm a window a reportable pattern.
- A document that lacks its origin facet — the normal condition of an adopted corpus at first contact — does not have a window invented for it. Expectation instances against it are skipped with reason `missing-origin` and counted in coverage ([spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)). The missing required facet is already its own finding, so the absence is loud while the window stays honest.

Four constraints keep expectations honest:

- **Detective only.** An expectation is never blocking. The work may legitimately be in flight, deferred, or abandoned for good reason.
- **A window is required.** An expectation with no time bound is a wish. The window is what makes the finding actionable.
- **An origin is required.** A window that cannot say where it starts is not a window. It is a mood.
- **A rationale is required.** If you cannot say why the participation is expected, it is a convention, not an expectation, and it will generate noise.

The engine reports expectation findings against the *originating* document, because that is where the reader who can act will look.

## The immutable core

A taxonomy package declares a **core**: the semantics that overlays may extend but never remove or redefine.

```yaml
core:
  requires:
    - facet_role: state
    - facet_role: freshness
    - facet_role: scent
    - purpose: rationale
    - purpose: behavior
    - relation_family: succession
      lifecycle_sensitive: true
```

Without this, "the same taxonomy" means nothing. If a consumer may override or remove anything, two consumers of one package can share no structure at all. The publisher then has no answer to "are they still using the method?"

**The core is semantic, not lexical.** It constrains *roles and purposes*, never names or paths. An adopter may rename every shelf, relocate every directory, rewrite every identifier pattern, and replace the lifecycle vocabulary — and still satisfy the core. The condition is that after resolution *some* facet carries the state role, *some* kind serves the `rationale` purpose, and lineage remains expressible and lifecycle-sensitive.

The identifier namespace is the one lexical requirement, and the core does not carry it. `identifier integrity` requires one on every resolved scheme, and no core entry can ask for the same thing. `core.requires` has an identifier form, and that form names a scheme an overlay may not remove. An identifier is minted to travel: it appears in commit messages, code comments, tickets, and agent prompts ([spec 3](03-authoring-and-lifecycle.md#identifiers)). A corpus can rewrite its own documents at any time, and it can never rewrite a ticket that somebody else owns. So a namespace is cheap at minting and unrecoverable later, which is the whole of [departure 7](08-design-departures.md#7-identifier-namespacing-arrives-late).

The value is the adopter's and no package states it. A package that named a namespace would give one to every corpus that adopts it, and a stand-in such as `repo` names nobody at all. So a package declares none, and the consumer's overlay declares one. The namespace opens the pattern, because a rendered identifier reads outermost part first, as `PROJ-123` and `owner/repo#n` do. [Q25](09-decisions.md#q25--where-the-namespace-goes-in-an-identifier-and-who-declares-it) is the ruling on both halves.

That is the boundary-object property stated precisely: plastic enough to adapt to local practice, robust enough to keep a common identity across sites. Local form is entirely negotiable. Shared meaning is not.

Core satisfaction is checked **after** overlay resolution, against the resolved taxonomy. The check does not forbid particular operations. An overlay is rejected when the *result* fails to satisfy a core requirement. The error names the requirement and the operation that removed its last satisfier. Conformance ([spec 7](07-distribution-and-federation.md)) checks the core, not the whole taxonomy.

## Mapping between taxonomies

Two divisions with different taxonomies do not need a merged one. They need declared correspondences, and that is a solved problem: SKOS mapping relations.

```yaml
mappings:
  - to: platform/headwater-taxonomy@2.0.0
    kinds:
      decision:     {relation: exactMatch,  target: adr}
      specification:{relation: broadMatch,  target: component_spec}
      runbook:      {relation: closeMatch,  target: operational_procedure}
    facet_values:
      status.current: {relation: exactMatch, target: state.active}
```

Four relations, with their standard meanings: `exactMatch` (interchangeable in practice), `closeMatch` (interchangeable for retrieval, not for inference), `broadMatch` / `narrowMatch` (one is wider than the other), `relatedMatch` (associated, neither wider nor equivalent).

Mappings are what let a cross-repository aggregator answer "show me every decision in the organization" across taxonomies that share no vocabulary. Whoever needs the correspondence declares them, and they are directional, versioned, and validated. A mapping that names a kind that neither taxonomy has is a finding.

Past a handful of taxonomies, "whoever needs the correspondence" is only ever the aggregator tier. That is the normative topology, not a tendency. Pairwise mappings between fifty independent taxonomies permit 1,225 unordered pairs before direction and versions multiply them. Every pair goes stale on every publisher release. Peers do not map to peers at scale. The aggregator owns the correspondences, because it is the only party that reads them.

The engine can also emit the resolved taxonomy as SKOS (`headwater export --format skos`). That is partly interoperability with knowledge-organization tooling that already exists, and partly a sanity check. A taxonomy that no standard concept-scheme vocabulary can express probably contains something idiosyncratic.

## Kind resolution

Given a document path and its front matter, the engine resolves a kind in four steps:

1. It matches the path against shelf patterns — the most specific wins, and ties are a schema-validation error, not a runtime coin-flip.
2. If the shelf is homogeneous, it takes the declared kind.
3. If the shelf is heterogeneous, it reads the discriminator facet. A missing or unrecognized value is a finding whose severity the shelf declares.
4. It applies any path-pattern refinement that the shelf declares (for instance `functional.md` and `technical.md` resolve to different kinds within one component directory).

`headwater explain <path>` prints this derivation — which shelf matched, which rule fired, and the kind's declared purpose. It also prints which facets and sections are consequently required, and which relations are permitted. Classification is never a black box, for a human or an agent.

### Placement is primary; metadata fills the gap

Directory placement carries the primary classification, because it is the signal that a reader sees first and the one that a path glob can act on. Metadata materializes only what placement *cannot* express.

This produces one rule with real teeth: **a homogeneous shelf forbids the discriminator facet.** If the directory already says what a document is, a restatement in front matter creates a second truth that will eventually disagree with the first. The `shelf.placement_is_primary` check enforces the prohibition and does not trust authors to notice.

Heterogeneous shelves carry the inverse risk, named here so that audits watch it. The discriminator is self-asserted, and kind determines every obligation downstream — sections, facets, voice, relations, expectations. An author who types `doc_type: reference` instead of `standard` buys out of the standard's entire contract. No check can cross-examine prose genre unless it crosses the semantic boundary that [spec 1](01-conceptual-model.md) draws.

The mitigation is structural, not detective. Kinds that share a shelf should not diverge so far in obligation cost that arbitrage pays. `taxonomy audit` reports the discriminator distribution and its drift, so a shelf that quietly migrates toward its cheapest kind is visible.

## Facet acceptance tests

"Is this a good facet?" is usually settled by taste. Faceted-classification practice supplies actual tests, and the engine applies them.

| Canon | Test | Where checked |
|---|---|---|
| **Relevance** | The facet is read by at least one check, projection, routing rule, or expectation | schema |
| **Ascertainability** | Every enum value carries guidance that states when it applies | schema |
| **Permanence** | The facet declares `volatility`. A `mutable` facet may not appear in an identifier, a path, or a shelf pattern | schema |
| **Differentiation** | The facet actually partitions the corpus — a value found on nearly every document distinguishes nothing | corpus |
| **Orthogonality** | No two facets are near-perfectly correlated across the corpus | corpus |

The first three are decidable from the schema alone and run under `headwater taxonomy validate`. The last two require documents to measure against, and they run under `headwater taxonomy audit`, which is advisory by construction. A young corpus will fail differentiation simply because it is small.

Orthogonality is the one that deserves attention. If a document's `shelf` tells you its `doc_type` with near-certainty, one of them does no work. The redundant one will eventually disagree with the other. The audit reports correlated facet pairs and does not reject them, because the right fix is a judgment. Sometimes you delete a facet, and sometimes you discover that the shelf split was wrong.

## Abstract kinds

The meta-schema already admitted these, in one clause and with no semantics anywhere. The coverage rule says that every kind is reachable from a shelf, "or it is explicitly marked abstract". Nothing said what an abstract kind was, or what it was for. The [schema-format walkthrough](../evaluations/schema-format-walkthrough.md) found two independent scenarios that need exactly it, so it is defined here.

**An abstract kind is a kind that no document ever is.** It declares what a group of concrete kinds share. A concrete kind names its parent with `is_a`, and inherits from it.

```yaml
kinds:
  governed_document:
    abstract: true
    facets:
      require: [status, status_since, last_verified, summary]
  playbook:
    is_a: governed_document
    purpose: procedure
    lifecycle: standard
    facets:
      require: [owner]                 # added to what the parent already requires
```

Two costs disappear with it, and the walkthrough measured both. To add a kind no longer edits every relation that the kind participates in, because an endpoint may name an abstract kind. That endpoint reaches every concrete kind below it. And a change to a shared facet requirement is one edit rather than one edit for each kind.

The rules are deliberately few.

- **A kind names at most one parent, and a parent may name a parent.** Inheritance is a chain and never a lattice. Several parents bring back the contested-value problem that satellite inheritance already had to solve, and no case yet demands them.
- **No document resolves to an abstract kind.** It may never be a shelf's declared kind, and never a discriminator value. [Kind resolution](#kind-resolution) is unchanged, because it resolves to concrete kinds only.
- **Facet and section requirements union down the chain.** A child adds to what its parent requires. A child may never un-require what a parent requires, because that voids the parent's contract for a reader who trusts it. A parent that requires a facet that the child forbids is a validation error. **`facets.forbid` takes away and it adds nothing.** It removes a requirement that an ancestor stated, and it stops the generated value check from making an instance over that kind. No check reports a document that states a facet its kind forbids. An emitter that carried the prohibition as a constraint would therefore reject a document that this engine accepts. The discriminator rule below is a shelf rule and a different thing, and a check of its own enforces it.
- **Purpose is required on every concrete kind, and it may arrive by inheritance.** An abstract kind may declare the purpose for its group. A concrete kind with no purpose of its own, under no parent that declares one, fails validation exactly as before.
- **Endpoints resolve through the chain.** `supersedes: {from: [governed_document]}` permits every concrete kind that has `governed_document` above it.
- **An abstract kind is rigid.** The [rigidity rules](#kinds-are-rigid-states-are-not) apply to it in full. An abstract kind named `draft_document` is as wrong as a concrete one.

**`is_a` is not `subsumes`.** One is a statement about kinds and the other is a statement about documents, and [spec 1](01-conceptual-model.md#two-layers-terminology-and-assertions) holds those layers apart. `is_a` says that every playbook is a governed document, which is a fact about the schema. `subsumes` says that one decision is wider than another, which is a claim that an author makes about two documents. To confuse them lets the TBox leak into the ABox through a naming accident.

The declaration count does not move. `abstract` and `is_a` are attributes on `kinds`, not a new declaration, and an adopter who needs neither meets neither.

## Kinds are rigid; states are not

A kind is a property that a document cannot lose while it is still the same document — a specification stays a specification. A lifecycle state is a phase that every document passes through. Formal-ontology practice calls the first **rigid** and the second **anti-rigid**, and holds that an anti-rigid class may never subsume a rigid one.

The practical rule: **never model lifecycle state as a kind, a shelf, or a directory.** The validator enforces it. The validator rejects a kind whose name collides with a value in the state vocabulary. It also rejects a kind named with a bare phase adjective (`draft`, `pending`, `proposed`, `deprecated`, `legacy`, `temporary`, `obsolete`).

This is the most common taxonomy mistake there is, and it always looks reasonable at the time (`docs/drafts/`, a `deprecated-standard` kind). It is expensive to undo, because it forces a document to change identity as it matures. A name for the underlying principle gives the argument a resolution instead of a stand-off.

## Customization by composition

An adopter never edits a base taxonomy. They declare an overlay:

```yaml
taxonomy: acme-engineering
extends: headwater/standard@2.1.0

override:
  shelves.decisions.path: docs/adr/**            # we call them ADRs
  identifier_schemes.decision_id.pattern: "{namespace}-ADR-{seq:03d}"   # rename at will, the namespace stays outermost
  vocabularies.lifecycle_state:                  # our lifecycle, our names
    - {value: draft,   role: initial}
    - {value: active,  role: live}
    - {value: retired, role: terminal-retained}

add:
  shelves.playbooks:
    path: docs/playbooks/**
    homogeneous: true
    kind: playbook
  kinds.playbook: {purpose: procedure, ...}

remove:
  - shelves.proposals            # we do not do time-boxed proposals
  - relations.refines
```

Note what the lifecycle override does *not* break. The names change completely, the roles survive, and the core is satisfied. If the overlay dropped the `terminal-retained` role entirely, resolution would fail. The failure would not be because a key went missing, but because succession could no longer retain lineage, which the core requires.

Merge semantics are strict and total:

- **`override`** replaces a value at an addressed path. The path must already exist.
- **`add`** introduces a new key. The key must not already exist.
- **`remove`** deletes a key and everything that depends on it — and the resolver **fails** if a declaration that survives still references the removed key. To remove a shelf that a projection targets is an error at resolve time, not a mystery later. The key must already exist.
- Lists never silently merge. An overlay either replaces a list or uses explicit `add_to` / `remove_from` operations.
- Resolution is **order-independent** for disjoint paths and an **error** for paths in conflict. Two overlays that touch the same path are a conflict to resolve, not a last-writer-wins race.
- Overlay application must be **confluent**: the application of a set of overlays in any legal order yields the same resolved taxonomy. The resolver checks this statically, before it applies anything.
- The resolved taxonomy must satisfy the `core`. The resolver checks this last, on the result.

**Every operation asserts a precondition about the base, and a failed precondition is always an error.** `override` needs the path to exist. `add` needs the key to be absent. `remove` needs the key to be present. The symmetry is deliberate. An overlay states what it believes about the base, and an upgrade that falsifies the belief must say so rather than proceed.

The precondition of an `add` is about the addressed key and about nothing above it. A second overlay that already created the mapping above that key does not falsify the belief. The resolver creates the mappings it needs on the way down. To refuse the operation there would put two overlays in an order that the confluence rule below denies them.

**That rule has a second consequence, and an upgrade is where it shows.** A base release that *drops* the declaration above an addressed key falsifies nothing, so the `add` still resolves. It makes the declaration itself: the addressed key, under a name the base removed, and nothing else. Every reader downstream then finds a declaration that no taxonomy declares. So the resolver records each operation that makes what it addresses, and `taxonomy diff` reads the record as the [`addressability`](#versioning-by-measured-compatibility) dimension. The merge rule stands, because the confluence guarantee below needs it, and the record is what ends the silence.

So an upgrade can break an overlay in three ways, and the third is the one that surprises people. A base release that *adds* a key which the overlay already added is a collision, and the overlay stops resolving. **A collision is always a task for a human, never an automatic promotion to `override`.** The two operations differ in what the consumer inherits. `add` states the whole value. `override` keeps every upstream field that the consumer did not restate, so a silent promotion would import upstream decisions that nobody read.

Confluence is what makes order-independence a guarantee rather than a hope. The resolver builds the set of nodes that each overlay writes, and it checks pairwise commutativity. Two `add` operations commute when no node that one writes lies inside a node that the other writes. An `override` and a `remove` own the whole subtree at their address. So a pair that holds either one, and that reaches one subtree, does not commute. Two `override` operations on one path do not commute either. At resolve time, the resolver rejects any pair that does not commute, and it names both overlays and the contested path.

**The set is the leaves, and the addressed path is the special case of it.** A bundle that declares `kinds.report` and an overlay that writes `kinds.report.identifier` name two paths, and one is a prefix of the other. The two operations meet at no leaf. They write disjoint parts of one mapping, and either order gives one result. A check over the addressed paths alone therefore refuses a pair that commutes. An `add` states a whole value, and the value is where its leaves are.

Delta-oriented software product lines worked this ground thoroughly, and the requirement is theirs. Without it, a three-tier federation ([spec 7](07-distribution-and-federation.md#federation)) has a resolution order that someone must remember. That is a bug that waits for the day when two tiers are upgraded in the wrong sequence.

The resolver writes the resolved taxonomy to a lock file with a content hash. The engine checks the corpus against the lock, so a resolution result is reproducible and reviewable in a diff.

**Two resolutions are the same result when their canonical text is the same text.** The confluence rule above says that any legal order gives "the same resolved taxonomy", and the lock puts a hash over a file. A mapping records the order that its entries arrived in, so two orders can agree on every declaration and disagree on the text. The resolver removes that difference before the hash sees it. A key that the base declares keeps the position of the base, and a key that an overlay contributes follows it, in sorted order. A scalar is written plain where every character permits it, and double-quoted if not, whatever style its author used.

The identity is the text and not the tree, for a reason that the lock supplies. A reviewer reads the lock in a diff, and review is the moment that a lock exists for. So an identity that a reader cannot compute from the artifact in front of them fails there. An identity over the tree also needs a canonical serialization to hash. There is no third thing to hash, so an identity over the tree is this identity with the artifact concealed.

## The `$`-reference sublanguage

The [shape above](#shape) writes `values: $vocabularies.lifecycle_state`. The overlay above writes `shelves.decisions.path`. [To enable a relation](#the-decision-relation-vocabulary) writes `add: {relations.forbids: $package.optional.forbids}`. [Q2](09-decisions.md#q2--schema-format) counted those as three uses of one sublanguage, and it gave the sublanguage no grammar. Here is the grammar. The three uses are one path production, read from two places.

```abnf
address   = segment *( "." segment )
reference = "$" root "." address
root      = "vocabularies" / "package"
segment   = 1*( ALPHA / DIGIT / "_" / "-" )
```

**An address names a place, and a reference names a value.** An address is what an overlay operation takes: the key under `add`, `override`, `add_to` and `remove_from`, and each entry of the `remove` list. A reference stands in a value position, and it reads a value that another declaration holds. The enabling line above holds one of each.

**The sigil marks a reference in every position where a literal is also legal, and in no other position.** A facet's `values` takes a list of values or a reference to one, so the two must be distinguishable. The key under `add` takes an address and nothing else, so no sigil has anything to separate it from. That is a rule rather than a convention, and it settles the next surface that needs one.

**The root set is closed, and `package` is a reserved word.** `vocabularies` is a declaration of the taxonomy under resolution. `package` is not a declaration, and it names the publisher's library of defined but unenabled content. No taxonomy may declare a block called `package`. To add a root is a meta-schema change, exactly as [to add a facet role](#the-meta-schema) is.

The rule binds a taxonomy source and binds nothing else. [Spec 7](07-distribution-and-federation.md#publishing) gives a package manifest a `package:` key at its root, and a manifest is a different file with a different root set. So the two are two files, and a file that plays both parts is refused on its first line.

**Quoting is not an escape, so the sublanguage carries one.** Q2 rules that a scalar takes its type from the meta-schema and never from the YAML resolver. So `"$vocabularies.audience"` and `$vocabularies.audience` are one value, and quotation marks cannot hide a sigil. A corpus whose value starts with a dollar sign writes `$$`, which stands for one literal `$`. Outside a position that admits a reference, `$` is an ordinary character. An identifier scheme `pattern` is typed as a string, so `"^DR-[A-Z]{2,6}-[0-9]{4}$"` is a pattern and not a malformed reference.

**A segment holds letters, digits, `_` and `-`.** The refusal of `.` is permanent, because `.` is the separator. A declared key that holds a dot makes an address ambiguous, and the meta-schema refuses to declare one. Every other refusal is provisional, and the hyphen was one of them until [obligations and controls](#the-thirteen-declarations) arrived keyed by an identifier. Q2 settles the direction to guess in. To relax a rule later costs nothing, and this is the first rule to be relaxed. To add one later is a finding against every source that already used the form.

**An address is a sequence of segments and never a string.** Every question the resolver asks of two addresses is a question about their segments. `kinds.playbook` is a textual prefix of `kinds.playbook_step`, and neither address contains the other. A confluence check over text therefore refuses two `add` operations that commute. Disjointness is decidable on segments, and that is what makes the confluence check of [spec 7](07-distribution-and-federation.md#bundles-are-publisher-overlays-in-the-other-direction) a proof rather than a convention.

**A reference resolves last, over the merged tree.** The [overlay above](#customization-by-composition) overrides `vocabularies.lifecycle_state`, and the facet that reads that vocabulary is expected to follow it. An early resolution freezes the base list before any overlay is read, and the override then changes nothing that a check can see. The order is thus: apply every overlay, resolve every reference, validate the result.

**A reference points at a value and never at a second reference.** A chain admits a cycle, and no use needs one. This is the second provisional rule in this section, and it relaxes under the argument that settled the first.

**An address never travels through a reference.** The node that an address names must be in the merged tree before any reference resolves. To address through a reference makes the result depend on an order that the rule above fixes for a different purpose.

**An address into a list is an error, and the grammar cannot catch it.** `0` is a legal key name, so no lexical rule tells an index from a key. The refusal belongs to the meta-schema, which declares the positions that hold a list and needs no merged tree to answer. A list position does not survive an upstream release, and `add_to` and `remove_from` are what a list takes instead.

What `optional` means under the `package` root is not fixed here. [Spec 7](07-distribution-and-federation.md#publishing) declares no such block in the package manifest, and [13 — Open obligations](13-open-obligations.md) carries the gap. The grammar admits `$package.optional.forbids` under either reading, because the open question is the shape of a package file rather than the shape of a reference.

## The meta-schema

The taxonomy language has a formal schema, published with the engine and versioned with it. `headwater taxonomy validate` checks:

- structural conformance to the meta-schema.
- **reference well-formedness** — every address and every `$`-reference parses under [the sublanguage](#the--reference-sublanguage). A reference names a root from the closed set, and the node it reads is not a second reference. An address names a node that exists, and it reaches no position in a list.
- referential integrity — every referenced vocabulary, regime, kind, facet, and purpose exists. Every relation endpoint is a declared kind or a declared anchor kind.
- **anchor integrity** — every anchor kind names exactly one resolver, and no two anchor kinds claim the same resolver namespace.
- **identifier integrity** — every identifier scheme carries a namespace after resolution, and no two schemes in one namespace admit the same string. This rule requires the namespace and `core` cannot. The identifier form of a core requirement names a scheme rather than a property of one. The engine reads each pattern into its segments and compares the two sets of strings. A `{slug}` stands last, so a pattern is a fixed run of literal and digit positions with an optional free tail. Two such runs meet or they do not, and the answer is a comparison rather than an estimate. So `{namespace}-DR-{seq:04d}` and `{namespace}-DR-{seq:06d}` are disjoint, because no string is four digits and six digits at once. And `{namespace}-SPEC-{slug}` and `{namespace}-SPEC-{seq:04d}` are not, because a free token admits `0042` like any other string. A pattern the engine cannot read is a validation error, because a scheme whose strings are unknown answers no question about a pair. A scheme whose strings another scheme's pattern also matches is a validation error, for the reason that two shelf patterns over one path are.
- coverage — every shelf resolves to at least one kind. Every concrete kind is reachable from at least one shelf. An abstract kind is reachable from none, and says so.
- **kind inheritance** — `is_a` names a declared abstract kind. The chain terminates and holds no cycle. No child un-requires what a parent requires, and no child forbids a facet that a parent requires.
- **purpose completeness** — every concrete kind has a purpose, declared or inherited, and every declared purpose is served by at least one concrete kind.
- determinism — no two shelf patterns can match the same path ambiguously.
- role uniqueness — at most one facet claims each engine-significant role, and the role registry is closed and lives here: `state`, `state_entered`, `created`, `freshness`, `scent`, `name`. Every `role:` in a taxonomy and every list of "special" facets elsewhere in this specification draws from this line. A role outside it is a validation error. To add a role is a meta-schema change, not a taxonomy change. `name` is the sixth, and a projection asked for it. A projection that writes a heading needs the text of that heading, and it may not read a facet by the name of the facet. A facet name means something in one package and nothing in the next, and a role is what the taxonomy states about a value ([spec 6](06-engine-architecture.md#projections)).
- lifecycle soundness — every state machine has an initial state, reaches every state it names, and declares every state it leaves without an exit. Reachability of the *vocabulary* is asked across the family of regimes rather than inside one. A state that no regime reaches is a promise nothing keeps. A state that one regime leaves out is how a kind narrows what it means.
- **relation coherence** — every relation names a valid family. Nucleus–satellite relations name their nucleus. `inherits` names facets that exist on both ends. A family's default is not contradicted without explicit override.
- **expectation well-formedness** — every participation expectation names a declared relation, or the declared inverse of one, and reachable kinds. The kind that declares the expectation is at the source end of the relation that it names. Each expectation carries a window, a rationale, and an origin role that is required on the kind that declares it.
- **core satisfiability** — the resolved taxonomy satisfies every core requirement.
- **facet canons** — relevance, ascertainability, and permanence hold for every facet (the corpus-measured canons run under `taxonomy audit`).
- **kind rigidity** — no kind collides with a lifecycle-state value or is named with a bare phase adjective.
- **edge provenance** — every relation declares a `created_by` from the closed set.
- **attribute well-formedness** — every instance attribute names a value space from the facet set and an owning end. No attribute takes a reference, and no attribute name collides with `to`.
- **warrant integrity** — every `transcription` projection names a declared anchor kind as its pin, and that anchor kind names exactly one resolver. A transcription that writes over an authored path is a projection-target error like any other.
- **context safety** — every agent-facing kind or projection has an applicable size budget, and every facet that carries the freshness role has an applicable staleness policy. The regimes that once wrapped these are gone. The mandates are not.
- **overlay confluence** — the overlay set commutes.
- **mapping integrity** — every mapping names kinds and facet values that exist in both taxonomies, with a valid SKOS relation.
- projection targets — every projection writes inside the corpus and does not collide with an authored path.

The engine never applies a taxonomy that does not validate. There is no partial-load mode. That sentence governs the taxonomy, not the corpus. A corpus may be legitimately between valid states during a major migration, and the tolerance mechanism is the migration state ([spec 7](07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states)).

## Versioning by measured compatibility

A taxonomy is a released, semantically versioned package — but the version number is **derived from measured impact**, not chosen from a table of change categories.

The naive model (additive changes are minor, everything else is major) is straightforwardly wrong for a schema that carries semantics. To add an *optional* facet is additive, and it can still change which documents a projection includes. To widen an enum is additive, and a completeness check that passed can then fail. Structural change and semantic consequence are not the same thing, and only one of them matters to a consumer.

So the engine evaluates compatibility along six dimensions. Five of them measure against a real corpus, and the sixth measures against real overlays. The dimension set is the engine's, fixed and identical for every taxonomy. An earlier draft made it a `compatibility` declaration, which every taxonomy would state identically — and a declaration with one legal value declares nothing:

| Dimension | Question |
|---|---|
| `classification` | Does every existing document still resolve to the same kind? |
| `instance_validity` | Does every existing document still validate? |
| `consequence` | Does every check that passed still pass, and every check that failed still fail? |
| `projection` | Does every projection produce identical output? |
| `identifier` | Does every identifier still resolve to the same document? |
| `addressability` | Does every path that an overlay can address still exist and mean the same thing? |

`headwater taxonomy diff <artifact>` runs all six and reports per dimension. The required version bump is a *consequence* of the result: any dimension broken forces a major version. A run in which a dimension did not happen decides nothing about the version. It says so, rather than report that no dimension is broken.

**Two of the six ask a narrower question than the table's wording, and the engine measures the narrower one.** `projection` asks for identical output. Three fields of two projections carry the package, the version and the taxonomy digest of the run that wrote them. Those three are inputs of a run rather than consequences of a taxonomy. So both plans are built under one identity, and the question becomes whether the *taxonomy* moved a projection. `consequence` asks about every check. Two rules of the check layer reach a verdict over the resolved taxonomy and create no instance ([spec 12](12-check-layer.md#the-five-origins-of-a-check)). A comparison of instances alone is therefore silent about a control that names a mechanism the engine cannot run. So the findings of every rule that creates no instance are compared beside the instances.

**`instance_validity` is the document-grained part of `consequence`, and never a second list of rules.** An instance whose grain is one document reads nothing else, so its verdict states whether that document validates. Every wider grain states something about the corpus between documents. The two dimensions therefore never disagree, because a broken `instance_validity` breaks `consequence` too. The split tells a reader whether the change reached documents or only the graph between them.

**`addressability` is the one dimension whose subject is the schema.** The other five ask what happened to a corpus. This one asks what happened to the surface that an overlay addresses. A rename can leave every document classified, every check unchanged, and every projection identical. It still breaks every consumer overlay that addressed the old path, and all five corpus dimensions report compatible.

**A consumer reads this dimension from one side, and the lock is why.** The question worth asking is whether the candidate drops a declaration that an overlay addresses, and it needs both bases before any overlay applies. The lock holds a *resolved* taxonomy, in which an overlay that put a declaration back is indistinguishable from a base that kept it. So the reading asks instead what each operation does under the candidate. An `add` that creates the declaration it reaches into is reported, whatever the base before it held ([Customization by composition](#customization-by-composition)).

[Spec 7](07-distribution-and-federation.md#upgrading) already reports invalidated overlay entries to the consumer, so half of this existed. What was missing is the half that acts. A report does not force a version bump, and a publisher that measures corpora alone never learns that it broke anyone. Promotion to a dimension fixes both ends. The publisher keeps reference *overlays* beside its reference corpora, and they are cheap to keep.

The publisher and the consumer play different roles here, and both are necessary:

- The **publisher** measures against its own reference corpora and reference overlays, and publishes the result as a compatibility claim attached to the release. That is the best that it can do. It does not have anyone else's documents, and it cannot enumerate every path that a consumer addresses.
- The **consumer** measures against its own corpus before an upgrade. This *verifies* the publisher's claim rather than trusts it. A claim that fails locally is exactly the interesting case. It means that the consumer's corpus uses something that the publisher's reference corpora do not.

A major version ships a **migration payload**: machine-readable steps that declare what moved, what was renamed, and what must be re-stated. [Spec 7](07-distribution-and-federation.md#the-migration-payload) states its form, and the split below runs through one step rather than between two lists of them. The steps are split into what the engine can apply mechanically (`headwater migrate --apply`) and what needs human or agent judgment (emitted as a task list with the affected documents attached). To adopt a new major version without a run of its migration is a hard failure, not a warning. The lock file records the taxonomy version and the measured compatibility result that each corpus was validated against.

**The payload migrates overlays, not only documents.** Every word of the paragraph above was written for documents. The overlay is the artifact most likely to break, and least likely to have a test. The rename map that the payload already carries is exactly what an overlay rewrite needs. `headwater migrate --apply` rewrites overlay addresses from that map, and `overlay_address` is the subject that names one ([spec 7](07-distribution-and-federation.md#the-migration-payload)). Each `add` collision is still owed a judgment task that shows both definitions together ([#195](https://github.com/headwater-ai/headwater/issues/195)). Without that, a consumer reads a resolver error and reconstructs by hand what the publisher already knew.

## Worked example: three taxonomies, one engine

| | Small team | Product suite | Regulated platform |
|---|---|---|---|
| Shelves | `decisions`, `specifications` | + `guides`, `standards`, `proposals`, `evidence` | + `controls`, `audits`, `risk` |
| Purposes | `rationale`, `behavior` | + `procedure`, `constraint` | + `attestation` |
| Lifecycle | `draft` → `current` → `superseded` / `deprecated` | unchanged | + `approved`, with an approver facet |
| Identifiers | decision ids | + requirement ids | + control ids, mapped to an external framework |
| Voice regime | declarative on both kinds | + normative keywords on standards | + mandatory normative keyword usage |
| Relations | the default five | + `verifies`, `implemented_by` | + `mitigates`, `attests`, `forbids`, `does_not_comply_with` |
| Expectations | none | decision → spec, incident → postmortem | + control → audit → attestation |
| Engine changes | none | none | none |

**The first column is derived rather than sketched.** An earlier draft of it gave the small team the `rationale` and `procedure` purposes, no identifiers, and an unconstrained voice. The core requires `behavior`, so `taxonomy validate` rejects that column as written. [Spec 3](03-authoring-and-lifecycle.md#identifiers) also rules that an identifier always carries a namespace, and the base binds the declarative regime to both of its kinds. The column is now the base package of the [first-run walkthrough](../evaluations/default-taxonomy-first-run.md), which derives the smallest legal taxonomy from the core instead of imagining it. Two of the three defects that walkthrough found came from this one cell block, and neither is visible until somebody writes the package out.

The third column is the real test. If a regulated adopter can express control mappings, approval states, and attestation relations with no change to engine code, the model is right. If they cannot, the schema lacks a primitive — and the fix is a new primitive, not a special case.

## Deliberate limits

The taxonomy language is **not** a general programming language. It has no conditionals, no user-defined functions, and no arbitrary expressions. Rules that the taxonomy language cannot express declaratively become **check plugins** with a documented interface (see [engine architecture](06-engine-architecture.md)). That boundary is defended: the moment that the schema grows an `if`, the drift between declared and actual structure comes back.

The relation family set is **closed**. An adopter may declare any number of relation types, but every one must belong to one of the six families. A taxonomy that needs a seventh family tells us something about the model. That conversation should happen upstream, and an escape hatch should not settle it locally.
