---
id: SPEC-HW-authoring-and-lifecycle
status: current
status_since: 2026-08-01
last_verified: 2026-08-11
summary: How a document starts, declares itself in front matter, records its provenance, and dies without loss of its lineage.
doc_type: design_spec
sequence: 3
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: [claude-fable-5, claude-opus-5]
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cites_evidence:
    - EVAL-HW-the-measurement-layer
    - EVAL-HW-warrant-and-adjudication
    - EVAL-HW-what-a-check-can-know
---

# 3 — Authoring and lifecycle

This section tells how a document starts and how it declares what it is. It also tells how the document stays trustworthy, and how it dies without loss of its history.

## Front matter is the contract

Every document opens with YAML front matter that contains the facets that its kind requires. Front matter is the machine's only guaranteed read of a document, so the schema is strict about it. Required facets are required, unknown facets are reported, and enum values outside a controlled vocabulary are findings.

Four facets do structural work in the default taxonomy:

- **state** (`status:`) — where the document sits in its lifecycle.
- **state-entry date** (`status_since:`) — the date when the document entered that state. The tool that does the transition stamps the date, and the same diff enforces the stamp. A change that moves `status` but not `status_since` is a finding, and so is a stamp in the future or before its predecessor. Windowed participation expectations are measured from this origin, under a declared maintenance contract ([spec 2](02-taxonomy-model.md#participation-expectations)). The date also makes dwell time observable. A document that stays in `draft` for a year is a fact that the corpus can now state.
- **freshness** (`last_verified:`) — the date when a human last confirmed that the document agrees with reality. This is deliberately *not* the last-edited date. Git already knows the last-edited date, and that date says nothing about truth.
- **summary** — one sentence, for machine consumption. It is what the query layer, the generated indexes, and agent-facing pointers show. The summary is not optional decoration. It is the public interface of the document.

Everything else — owner, scope, audience, domain — is a taxonomy choice.

**The provenance block is the exception, and its shape belongs to the engine.** An earlier draft listed provenance beside the taxonomy choices above. A taxonomy chooses which facets a kind carries. It does not choose whether `accepted_by` exists, because that field is what enforces the boundary between a draft and an acceptance. It does not choose the [warrant](01-conceptual-model.md#warrant) vocabulary either, because four engine rules turn on the value.

## Lifecycle

The lifecycle is a state machine, declared in the taxonomy and interpreted by the engine. The default taxonomy uses:

```
draft ──▶ current ──┬──▶ superseded
                    └──▶ deprecated
```

Rules that the engine enforces from the declaration alone:

- the engine rejects transitions that are not in the declared machine **when they land**. An illegal transition is only visible against the prior state, and the prior state lives in the diff, not in the graph. Thus hooks and change-scoped CI receive the prior version as a declared check input ([spec 12](12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version)). A full-corpus run sees only current states. It reports transition instances as change-scoped, and it does not silently pass them.
- terminal states marked `retain: true` may never be deleted. Lineage is the point.
- a live document may not depend on a terminal one through a relation declared `lifecycle_sensitive`. Thus a current specification that cites a superseded decision is a finding, automatically, for each such relation.
- entry into a state can require facets (a `superseded` document must name its successor). Entry can also cause reciprocal updates on the target.
- dwell in a non-terminal state is observable, not policed. `status_since` makes "parked in `draft` for a year" a fact. `taxonomy audit` reports the dwell distribution per state. A shelf whose documents sit indefinitely in the state with the fewest obligations is a finding about the shelf. A document that evades live-state obligations because it never goes live is in the same absence class that participation expectations catch, one level down.

**Correction versus succession** is a distinction that the system takes seriously. A document is edited in place when it was wrong about the present. A *successor* is written when the decision itself changed. The first preserves truth. The second preserves lineage. A conflation of the two destroys the record, so the lifecycle regime makes the second path cheap and the first path honest.

**Adjudication is the third case, and the pair above does not cover it.** Two documents can both be right and still disagree, and a human then settles which one governs. Neither was wrong about the present, so a correction misstates it. Neither decision changed, so a succession misstates it too. What happens instead is a new decision that `overrides` the one whose effect it displaces, and the loser stays in place ([spec 2](02-taxonomy-model.md#disagreement-is-adjudicated-not-ranked)).

## Freshness and staleness

`last_verified` is an assertion by a human: *on this date I checked that this document is true*. The declared policy of the freshness facet turns that assertion into a signal:

- the engine validates format and plausibility (no future dates).
- the engine reports documents past the staleness threshold, weighted by the criticality of the shelf.
- the engine flags a document whose backing code changed since its last verification ahead of one that is only old. Staleness is a function of drift risk, not only of calendar time.

Staleness is **detective, never blocking**. A block on staleness teaches authors to bump the date, and that changes the most valuable signal of the corpus into noise.

**Content with the `asserted` warrant carries no freshness value, and the engine never reports it as stale.** `last_verified` records that a human confirmed a document, and nobody confirmed this one. It cannot acquire the date without becoming `accepted`, which is what promotion is. What the corpus reports instead is drift: the sources changed after the date on which the content was asserted. The remedy is to produce the content again or to delete it, and there is no date to bump ([Q15](09-decisions.md#q15--a-synthesized-content-tier)).

## Voice

Some kinds describe the world as it is. Some narrate change. A mix of the two is the most common failure in a documentation corpus. It is mechanically detectable at useful precision.

- **Declarative regime** (specifications, standards, architecture): present tense, present state. No future intent ("will be", "planned"), no change narration ("we moved from X to Y"), no phased-rollout language, no comparatives against a prior state. If a reader cannot tell whether a sentence describes today or last quarter, the document failed.
- **Narrative regime** (proposals, evidence, incident records): time-bound by nature, and exempt.

Enforcement is lexical and thus imperfect. It uses a curated pattern set for each forbidden category, with per-file and per-block escape hatches that must state a reason. The escape hatch is itself a signal. A shelf that collects exemptions is a shelf whose kind assignment is wrong, and the engine reports that concentration.

**The escape hatch takes a reason from the closed set that [spec 4](04-assurance-model.md#suppression) declares**, and free prose beside it. `false_positive` and `accepted_deviation` are what the promotion statistics run on. A reason field that only holds prose collects no statistic at all.

### What a lexical rule gets wrong, and where posture comes from

Enforcement is lexical, and the measured errors of a lexical checker do not fall where intuition puts them ([evaluation](../evaluations/what-a-check-can-know.md)). Three rulings follow, and two of them constrain the parser rather than the rules.

**A voice check reads author-owned text only.** A quotation, a code span, a citation line, and a generated block are outside every voice rule by construction. This is not an exemption that a rule declares. It is a property of what the parser hands a `Document`-scoped check, and [spec 12](12-check-layer.md#the-correctness-roots) holds it there with sentence segmentation.

**Posture per category comes from fixability and never from precision.** A category may become blocking only when its remediation is mechanical and total, which is the bar that [spec 12](12-check-layer.md#fixability) already sets for a patch. A category whose remediation is a rewrite stays advisory permanently, whatever its false-positive rate turns out to be ([spec 4](04-assurance-model.md#where-promotion-cannot-finish)). Phased-rollout language is the most likely candidate for a mechanical fix. Nobody has built one.

**The precision of the pattern set is not the thing to tune first.** Segmentation and span produced most of the observed errors of the checker that this repository runs on itself. A pattern set that a better tokenizer feeds is a different rule from the same pattern set on raw lines.

## Normative language

Where a document states requirements, the strength of each statement is explicit (RFC 2119 keywords, or the set that the taxonomy declares). Each statement also has a distinct format. Three parts are checkable. Keywords appear in the declared casing. Documents that use them contain the interpretation boilerplate. The engine flags hedged pseudo-requirements ("should probably", "ideally must").

This exists because unmarked requirements are the ones that people argue about later. It also exists because an agent that reads the corpus must know the difference between a rule and a suggestion.

## Templates and scaffolding

Each kind declares a template. The template is not a suggestion file for humans to copy. It is generated from the kind declaration. Thus the required sections and the required front matter always agree with what the engine validates. A template that drifted from its kind is impossible by construction.

```
headwater new decision --title "Adopt overlay-based taxonomy customization"
```

resolves the kind, allocates an identifier, seeds front matter, and emits required sections with prompts. It places the file where the shelf layout dictates. It also prints the relations that the new document is expected to declare.

## Identifiers

Some artifacts need stable names that survive a move, a rename, or a read out of context. These artifacts are decisions, requirements, acceptance criteria, controls, and obligations.

The taxonomy declares the pattern, the namespace, and the allocation policy of each identifier scheme. Three properties matter:

1. **Globally unique.** An identifier is namespaced at minting — by repository or organization. Thus, when a corpus is vendored into another repository, identifiers from two sources cannot collide. To retrofit a namespace later is expensive. The default is to always have one.

2. **Resolvable without its document.** Given `DR-ACME-0042` and nothing else, the engine resolves it to a path. The graph contains an identifier index, so identifiers work in commit messages, code comments, tickets, and agent prompts.

3. **Never reused.** Allocation is reconcile-first. The allocator scans the corpus (terminal-state documents included) for the highest allocated value before it mints a new one. A deleted document does not free its number.

## Evidence has three honest states, not two

A decision recorded without evidence outside the document that records it is a rationalization. The system asks for a pointer — a work item, a substantive commit, a recorded discussion, a measurement.

Where none exists, the earlier design offered two outcomes: evidenced, or a registered gap. That is one short. The real design process is never as rational as the record makes it look. To document it *as if* it were rational is both legitimate and valuable — if the reconstruction is labeled as one. To force every after-the-fact account into "gap" pushes authors to overstate what they have. That is the failure that the rule existed to prevent.

| `evidence_basis` | Means | Obligation |
|---|---|---|
| `evidenced` | An external, auditable artifact supports this | The pointer resolves |
| `reconstructed` | Written after the fact from memory and inference | Must state what it was reconstructed from, and by whom |
| `unevidenced` | No evidence exists and none is claimed | Appears in the gap register |

`reconstructed` is not a soft `evidenced`. It never silently promotes. To move a document to `evidenced`, you must add a resolving pointer, and the transition is recorded. A corpus where most rationale is reconstructed tells you something real about how decisions are made there. To hide that behind a binary would waste the signal.

The semantic judgment — *is this evidence actually about this decision?* — stays with the author and the agent stop rules ([spec 5](05-ai-integration.md)). The mechanical parts are these: the facet is present and valid, pointers resolve, `reconstructed` contains its basis, and the gap register accounts for every `unevidenced` document.

**Two rulings about which pointers count** ([Q15](09-decisions.md#q15--a-synthesized-content-tier), [Q19](09-decisions.md#q19--inbound-integration-an-external-system-of-record)). A pointer to a document with the `asserted` [warrant](01-conceptual-model.md#warrant) does not support `evidenced`, because such a document is neither external nor auditable. A fabricated *why* with a file name is the failure that this table exists to prevent. A pointer that an importer created does support `evidenced`. A work item in a system of record is an external auditable artifact, and the pointer resolves offline against a committed snapshot.

The value is named `unevidenced`, and not `gap`, because `gap` is already the [disposition](04-assurance-model.md#every-obligation-has-exactly-one-disposition) of an obligation that no control discharges. One word for two mechanisms hid a real question, and this document does not settle it. Is the gap register above the same artifact as the obligation gap register, or a second one that shares its name?

## Provenance is recorded, not assumed

Humans, agents, and the two together now draft documents. The question "who wrote this and who accepted it?" should be a query, not an archaeology exercise.

Every document contains provenance aligned with W3C PROV:

```yaml
provenance:
  warrant: accepted        # accepted | regenerated | transcribed | asserted
  agency: agent            # human | agent | mixed
  drafted_by: claude-opus-5
  activity: scaffold+draft
  accepted_by: j.baxter    # a human is always named here
  evidence_basis: reconstructed
  reconstructed_from: "commit 4a2f1c, ADO 1441575, design session 2026-07-15"
```

`accepted_by` is the field that enforces the boundary. An agent may draft. Acceptance is a human act, and the record says who did it. Generated projections are exempt. They are `wasGeneratedBy` a tool, and they are checked against regeneration, not accepted.

This makes real questions answerable. Which parts of the corpus are agent-drafted? Do agent-drafted documents drift faster than hand-written ones? Does reconstruction correlate with agency? None of these questions can be asked of a corpus that does not record the answer.

### The warrant, and what each value requires

`warrant` states what stands behind the document ([spec 1](01-conceptual-model.md#warrant)). It is required, its value set is closed, and an absent value is a finding rather than a default. Each value requires a different part of the block, and the engine checks the pairing.

| `warrant` | Requires | Forbids |
|---|---|---|
| `accepted` | `accepted_by`, naming a human | — |
| `regenerated` | The generated-file marker, and a source inside the repository | `accepted_by` |
| `transcribed` | The generated-file marker, and the snapshot pin that it copies | `accepted_by` |
| `asserted` | `drafted_by`, `activity`, and the sources that produced the content | `accepted_by` |

**`asserted` forbids `accepted_by` for the reason that runs through this specification.** A document that names an acceptor is `accepted`. To let the two coexist gives one fact two authoring locations, and the reader then has to decide which one wins.

**Agency is not the warrant.** An agent that drafts a document which a human then accepts produces an `accepted` document, and that is the ordinary case in this design. `asserted` marks content that nobody accepted, whoever or whatever wrote it.

**The engine derives `regenerated` from the marker, and never from a declaration.** A generated file that declares no front matter has no block in which to state a warrant. A generated document that declares an identity has one, and a declared warrant there would be a fact that a hand edit can falsify. The census would then have to choose between the declaration and the marker. It reads the marker instead, and reports the file under an outcome of its own ([spec 6](06-engine-architecture.md#projections)). So nobody accepts a generated document, and no run reports one as untyped.

**A measurement is `regenerated`, and its transcript is what makes that true.** An efficacy [probe](05-ai-integration.md#a-run-produces-a-snapshot-and-a-document) run emits a transcript, which the corpus commits as a snapshot. The probe result is generated from that snapshot, the declared expectations and the grader version, so `generate --check` proves it. Without the committed transcript the result would be `asserted`, and the [evidence rules above](#evidence-has-three-honest-states-not-two) would then refuse it. A measurement that no decision may cite is not worth running.

### Promotion is one human, one document, one diff

An `asserted` document becomes `accepted` when a person reads it, sets the warrant, and names themselves. That is the act the whole model rests on, and it needs no second mechanism.

The failure mode is bulk. A script that stamps forty documents writes bytes that are identical to forty real acceptances, and no check can separate them. So the engine does not try. `taxonomy audit` reports promotions per change instead, and a change that promotes forty documents is a finding about the review rather than about the documents. The posture is advisory permanently, for the reason that the [shift ratio](04-assurance-model.md#measuring-coherence-where-we-can-continuity-across-links) is advisory permanently. A threshold teaches people to promote in smaller batches.

Promotion never rewrites history. `drafted_by` and `agency` stay as they were, so the corpus can still say which parts an agent drafted.

## Capture cost is a tracked metric

Every design-rationale system of the last fifty years — IBIS, gIBIS, QOC — produced a rich model and almost no sustained adoption. There is one reason: **capture costs the author and benefits someone else, later.** Our evidence rules increase author cost. That trade may be correct, but it is the trade that historically kills these systems. Thus the trade is measured, not assumed.

The engine records the **assisted fraction** for each document created. Of the required front matter, sections, identifiers, and relations, how much was scaffolded, derived, or agent-drafted, and how much was hand-entered? The fraction is cheap to compute — the scaffolder knows what it supplied — and it trends.

The fraction is used in two ways:

- **As a design budget.** A hand-entered fraction that rises means that the taxonomy demands more than the tooling supports. The remedy is to derive more, or to require less. To add a lint that nags authors is the wrong move, and the metric makes that visible. An assisted fraction that *declines* is an assurance finding in its own right, not a trend line to glance at. It shows that the adoption model fails, measurably.
- **As the test of the agent-authoring claim.** [Spec 5](05-ai-integration.md) argues that an agent that drafts from evidence already present in the commit, the ticket, and the conversation shifts capture cost off the author. That is the first genuinely new answer to that objection in thirty years. Either the assisted fraction rises when agent authoring is enabled, or the claim is wrong. This is how we find out.

The metric is reported in the adaptive layer of the [assurance model](04-assurance-model.md), alongside efficacy results.

## Authoring surfaces

| Surface | Use |
|---|---|
| `headwater new <kind>` | Scaffold a document with correct placement, metadata, sections, identifier |
| `headwater check --fix` | Apply mechanical corrections: format front matter, add missing reciprocal links, regenerate projections |
| Editor integration | Schema-driven completion and inline validation via a language server over front matter |
| Agent-assisted authoring | The judgment-bearing path: drafting, evidence checks, cross-linking (spec 5) |

All four converge on the same schema. There is no path into the corpus that skips it.
