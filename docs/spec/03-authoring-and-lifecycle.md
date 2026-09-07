---
id: HW-SPEC-authoring-and-lifecycle
status: current
status_since: 2026-08-01
last_verified: 2026-08-11
summary: How a document starts, declares itself in front matter, records its provenance, and dies without loss of its lineage.
doc_type: design_spec
sequence: 3
title: "Authoring and lifecycle"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: [claude-fable-5, claude-opus-5]
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cites_evidence:
    - HW-EVAL-the-measurement-layer
    - HW-EVAL-warrant-and-adjudication
    - HW-EVAL-what-a-check-can-know
---

# 3 — Authoring and lifecycle

This section tells how a document starts and how it declares what it is. It also tells how the document stays trustworthy, and how it dies without loss of its history.

## Front matter is the contract

Every document opens with YAML front matter that contains the facets that its kind requires. Front matter is the machine's only guaranteed read of a document, so the schema is strict about it. Required facets are required, unknown facets are reported, and enum values outside a controlled vocabulary are findings.

Four facets do structural work in the default taxonomy:

- **state** (`status:`) — where the document sits in its lifecycle.
- **state-entry date** (`status_since:`) — the date when the document entered that state. The tool that does the transition stamps the date, and the same diff enforces the stamp. A change that moves `status` but not `status_since` is a finding, and so is a stamp later than the current date or before its predecessor. Windowed participation expectations are measured from this origin, under a declared maintenance contract ([spec 2](02-taxonomy-model.md#participation-expectations)). The date also makes dwell time observable. A document that stays in `draft` for a year is a fact that the corpus can now state.
- **freshness** (`last_verified:`) — the date when a human last confirmed that the document agrees with reality. This is deliberately *not* the last-edited date. Git already knows the last-edited date, and that date says nothing about truth.
- **summary** — one sentence, for machine consumption. It is what the query layer, the generated indexes, and agent-facing pointers show. The summary is not optional decoration. It is the public interface of the document.

Everything else — owner, scope, audience, domain — is a taxonomy choice.

**The provenance block is the exception, and its shape belongs to the engine.** An earlier draft listed provenance beside the taxonomy choices above. A taxonomy chooses which facets a kind carries. It does not choose whether `accepted_by` exists, because that field is what enforces the boundary between a draft and an acceptance. It does not choose the [warrant](01-conceptual-model.md#warrant) vocabulary either, because four engine rules turn on the value.

## Lifecycle

The lifecycle is a state machine, declared in the taxonomy and interpreted by the engine. A taxonomy declares one machine per regime, and a kind binds one regime. The default taxonomy declares two:

```
standard    draft ──┬──▶ current ──┬──▶ superseded
                    │              └──▶ deprecated
                    └──▶ deprecated

obligation  draft ──┬──▶ current ──┬──▶ superseded
                    │              ├──▶ deprecated
                    │              └──▶ discharged
                    └──▶ deprecated
```

The three terminal states answer three different questions. `superseded` names a successor. `deprecated` says that nothing replaced this one. `discharged` says that the document recorded something the corpus owed, and that the corpus paid it.

**What makes a state terminal is the role on its value.** A state ends a regime when the vocabulary gives its value a `terminal-` role, and when the machine gives it no exit. The two readings name one set, because `lifecycle soundness` refuses a regime where they differ. A regime that reaches a state with no exit and no such role is refused. A regime that reaches a role-terminal state and gives it an exit is refused too. A regime declares no list of terminal states of its own.

Both readings are read. `lifecycle.dependency.on_terminal` asks the role, and `lifecycle.deletion.not_permitted` and `lifecycle.transition.not_permitted` ask the machine. The role is one word for the whole state vocabulary, and no rule that reads it asks which regime. So a state that one regime ends and another lets move on is refused. Terminality is a property of a state, and never of a state and a regime together. [Q26](09-decisions.md#q26--whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime) is the ruling, and the reasons for it are there rather than here.

**A taxonomy that wants both readings of one word writes two words.** A regime that continues past a state gives it a value with the `live` role and its own exits. A regime that ends there keeps a value with the `terminal-` role. `lifecycle.state.not_admitted` then reports a document authored at the value its own regime does not name. A rule holds the split, and not the care of an author. `engine/crates/resolve/fixtures/validate/terminality-by-rename/` is the worked shape, beside the case that is refused.

**A regime is where a kind says which states it means.** The state vocabulary is one list for a whole taxonomy, and a machine names the part of that list its own documents move through. `standard` names four of the five states and `obligation` names all five, so only a kind that binds `obligation` stands at `discharged`. That is the whole of the per-kind restriction, and no member on `kinds` carries it.

Rules that the engine enforces from the declaration alone:

- the engine rejects transitions that are not in the declared machine **when they land**. `lifecycle.transition.not_permitted` is the rule, and it reads the machine that the resolved taxonomy declares rather than a set of states written into the engine. An illegal transition is only visible against the prior state, and the prior state lives in the diff, not in the graph. Thus hooks and change-scoped CI receive the prior version as a declared check input ([spec 12](12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version)). A full-corpus run sees only current states. It reports transition instances as change-scoped, and it does not silently pass them.
- a document stands only at a state that the regime of its kind names. `lifecycle.state.not_admitted` is the rule, and it reads one declared value against one declared state set. It needs no prior version, so it reaches the case the rule above it cannot. A document **authored** at a state its kind has no use for made no movement, and a full-corpus run reports it anyway. Where a movement lands on such a state, this rule owns the finding and the transition rule stands down. One defect stays one finding.
- a change does not delete a document that stands at a terminal state of a regime that declares `retain_terminal: true`. `lifecycle.deletion.not_permitted` is the rule, and it reads the change rather than a document. A deletion is only visible against the version that stood before it. The corpus holds no row where the document was, so the version the caller named is the only account of it. That is why the rule has no document to instantiate over. Its grain is the corpus, and it is the first rule of that grain to read a prior version. A rename is not a deletion, because the caller names the path the document arrived at and the census holds a row there. Lineage is the point. A terminal state is where a document is kept, and a corpus that deletes the record loses the only account of what it said.
- a live document may not depend on a terminal one through a relation whose family a core requirement declares `lifecycle_sensitive`. `lifecycle.dependency.on_terminal` is the rule, and it is the first one that reads a state at both ends of one edge. Live and terminal are the `role` on each state value rather than a list of state names in the engine. A taxonomy that renames every state is read unchanged, which is what the worked overlay of [spec 2](02-taxonomy-model.md#the-immutable-core) asks of a reader of a state. A relation that writes a state onto its target is exempt where the target stands at that state, because the edge put it there. The base declares `succession` sensitive, so a correct supersession is exempt and a successor that points at a document some other history deprecated is not. **The flag is declared on a family and never on a relation.** The meta-schema writes `lifecycle_sensitive` on an entry of `core.requires` and nowhere else. A taxonomy names the sensitive families, and every relation of one carries it. [#226](https://github.com/headwater-ai/headwater/issues/226) holds whether a relation may declare it for itself, which is what a rule about a citation of a superseded decision would need.
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

- **Declarative regime** (specifications, standards, architecture): present tense, present state. No future-tense claims about later behavior, no change narration, no phased-rollout language, no comparatives against a prior state. If a reader cannot tell whether a sentence describes today or last quarter, the document failed.
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

**A template is derived, and no file holds one.** The rule above says so, and one consequence follows for a package. [Spec 7](07-distribution-and-federation.md#publishing) puts a `templates/` directory in the anatomy of a package, and a file there teaches an author rather than feeding the scaffolder. A source that the scaffolder read would be a second authoring surface. [#130](https://github.com/headwater-ai/headwater/issues/130) refused one on a projection declaration, and the reason reaches a scaffolder unchanged. A taxonomy sits outside the corpus root. No census row covers it, no language regime binds it, and no rule reads its links. Prose that the corpus governs would move to the one place that the corpus cannot see.

**A scaffolded document is authored, and every check reads it.** A projection carries a generated-file marker, and no check reads the file. [Spec 6](06-engine-architecture.md#projections) gives two reasons for that. The content is a function of the emitter, and an author cannot repair it in the file. Neither clause holds for a scaffolded document. Nothing writes the file a second time. So an author repairs it where it stands, and the repair survives. The test between the two is regeneration, and it is the whole of the difference. So a scaffolded document carries no marker, and the rule that a generated output is exempt reaches it nowhere.

**What the taxonomy determines, the scaffolder writes. What it does not, the scaffolder asks.** The shelf that claims the kind decides the directory, and the shelf layout decides the file name. The lifecycle regime decides the opening state. The injected clock decides the two dates. The discriminator of a heterogeneous shelf takes the kind. The section contract decides the headings. A facet that no declaration determines carries a prompt instead of a value, and the run reports it as hand entry.

**A shelf layout reads three sources, and a placeholder it cannot fill is a refusal.** `{slug}` takes the slug of the title. A placeholder that names a facet takes the value that the run wrote into that field. `{seq}` takes the sequence that the identifier of the run carries. A numbered shelf therefore names a file from the number it already holds, rather than from a second copy of that number in a facet. A namespace is a constant of its scheme, so no layout names one. Such a placeholder is the same characters in every file name on the shelf. A placeholder that none of the three fills is a refusal, because a name with a hole in it names nothing.

**No rule reads a layout, and the reason is measured rather than deferred.** A layout renders a name from a title, and a title is an argument to one run. So a check has to re-derive the slug from a facet, and `design_spec` requires no facet in the `name` role at all. `taxonomy audit` takes that measurement over every shelf that declares a layout, and it holds apart the shelf that no declaration lets it measure. [HW-OBL-0106](../obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md) states what a rule of this shape would need. A layout is therefore a convention at birth that the scaffolder obeys, and a rename after that birth is a fact that no declaration states.

**A required facet that nothing determines and no prompt fits is a refusal.** A closed value set admits no prompt, and neither does an integer or a date. A value the engine picked there would be a state that nobody chose. That is the fabricated fact that [spec 12](12-check-layer.md#the-correctness-roots) holds a scaffolder to account for. So the run writes nothing and names the facet and the kind.

**Allocation reads a tree, and a tree holds no history.** The rule below says that a deleted document does not free its number. The corpus is what the engine reads, and a deleted document is not on it. So the highest value that a run finds is a lower bound on every value ever allocated. The report of each minted identifier states that bound.

**A tree also holds no concurrency, and the claim store is the answer.** Every branch cut from one state reads the same corpus, so two runs on two branches find the same highest value and mint the same number. Each branch is correct on its own, the two file names differ, and the merge is silent. The pair first exists after the second merge, when no run is looking at either change. So a run writes one file for each identifier it mints, at `.headwater/ids/<scheme>/<identifier>`, and that file holds the path of the document that minted it. The allocator takes the highest value of the corpus and of the store together.

**Two declarations decide which identifiers the store covers.** A shelf that declares a layout writes each file name from a template. Two documents that hold one identifier then differ at a segment the identifier does not carry. A scheme that allocates reconcile-first mints a sequence that every branch reads alike. Either state means that the file name does not determine the identifier, which is when a merge needs a shared path to refuse it. A shelf with neither names each file from the slug that its identifier already patterns, so the document itself is the conflict ([HW-DR-0057](../decisions/0057-a-shelf-layout-is-the-second-half-of-what-the-identifier-claim-store-covers.md)).

**The store does not report the collision. It makes the two branches meet.** Two branches that claim one value add one path with two different contents, which the version control system refuses to merge. To resolve that refusal, the second branch takes the first one into its own tree, and the rule that no two documents claim one identifier then reports the pair on the branch, before the merge. The path inside the file is what makes the two sides differ, so a claim file is never empty.

**The verb reports where every value came from, and a reader needs that more than the file.** A tool that printed only its output would ask for trust on one ground: that a tool produced it. This specification refuses that reading everywhere else. Each field names the declaration behind it, and the [assisted fraction](#capture-cost-is-a-tracked-metric) is the count of those origins rather than a second number beside them.

## Identifiers

Some artifacts need stable names that survive a move, a rename, or a read out of context. These artifacts are decisions, requirements, acceptance criteria, controls, and obligations.

The resolved taxonomy declares the pattern, the namespace, and the allocation policy of each identifier scheme. The namespace is the one of the three that no published package declares. A package that named one would give it to every corpus that adopts it, so the corpus declares its own in an overlay ([Q25](09-decisions.md#q25--where-the-namespace-goes-in-an-identifier-and-who-declares-it)). Three properties matter:

1. **Globally unique.** An identifier is namespaced at minting — by repository or organization. The namespace goes first, so `^ACME-` matches everything one corpus owns and a lexical sort groups foreign identifiers by their owner. Thus, when a corpus is vendored into another repository, identifiers from two sources cannot collide. To retrofit a namespace later is expensive. The default is to always have one.

2. **Resolvable without its document.** Given `ACME-DR-0042` and nothing else, the engine resolves it to a path. The graph contains an identifier index, so identifiers work in commit messages, code comments, tickets, and agent prompts.

3. **Never reused.** Allocation is reconcile-first. The allocator scans the corpus (terminal-state documents included) and the claim store for the highest allocated value before it mints a new one. A deleted document does not free its number, and its claim stands after it. A claim that names a document the corpus no longer holds is therefore correct, and no rule reports one.

## Evidence has three honest states, not two

A decision recorded without evidence outside the document that records it is a rationalization. The system asks for a pointer — a work item, a substantive commit, a recorded discussion, a measurement.

Where none exists, the earlier design offered two outcomes: evidenced, or a registered gap. That is one short. The real design process is never as rational as the record makes it look. To document it *as if* it were rational is both legitimate and valuable — if the reconstruction is labeled as one. To force every after-the-fact account into "gap" pushes authors to overstate what they have. That is the failure that the rule existed to prevent.

| `evidence_basis` | Means | Obligation |
|---|---|---|
| `evidenced` | An external, auditable artifact supports this | The pointer resolves |
| `reconstructed` | Written after the fact from memory and inference | Must state what it was reconstructed from, and by whom |
| `unevidenced` | No evidence exists and none is claimed | Appears in the gap register |

`reconstructed` is not a soft `evidenced`. It never silently promotes. To move a document to `evidenced`, you must add a resolving pointer, and the transition is recorded. A corpus where most rationale is reconstructed tells you something real about how decisions are made there. To hide that behind a binary would waste the signal.

The semantic judgment — *is this evidence actually about this decision?* — stays with the author and the agent stop rules ([spec 5](05-ai-integration.md)). The mechanical parts are these. The facet is present and valid, pointers resolve, `reconstructed` contains its basis, and the gap register accounts for every `unevidenced` document.

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

The failure mode is bulk. A script that stamps forty documents writes bytes that are identical to forty real acceptances, and no check can separate them. So the engine does not try to tell one stamp from another. What it can do is count them in a change. Forty in one change is a finding about the review rather than about the documents. The posture is advisory permanently, for the reason that the [shift ratio](04-assurance-model.md#measuring-coherence-where-we-can-continuity-across-links) is advisory permanently. A threshold teaches people to promote in smaller batches, and nothing declares one.

**The count is a check that reads the version each document stood at before the change.** A promotion is a lifecycle transition, from the `asserted` warrant to `accepted`. [Spec 12](12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version) supplies the input that a transition needs. `warrant.promoted` declares `needs_prior`, so it receives that version of each changed document. `headwater check --change` is where a caller names the set of documents one change carries. A run with no change reports every instance of the rule as skipped, because the input is available only in change-scoped evaluation. The rule reports one document at a time and the run states the count, and neither carries a verdict.

**A caller of this repository writes the manifest, and it is a hook and a job rather than a verb.** `.githooks/change-manifest` turns a base revision and the working tree into the manifest that `headwater check --change` reads. `.githooks/pre-commit` names the committed `HEAD` as the base, which is the anchor a working-tree gate has. The continuous integration job names the base of the proposed change. Every version control command on that path is in the producer, and the engine holds none.

**The producer filters nothing, and the noise is the instrument.** A change carries files that are no document of any corpus. A run holds every path of a manifest against the corpus it walks, and it names each one that reaches no row. A producer that filtered to the corpus root first would drop a path written in a form the census does not use. The run would then report zero promotions over that change, and report success. That is the one defect a producer can have, and the report is where it becomes visible.

**The count over this repository is zero, and the zero is a measurement.** The producer ran over the last 120 commits of this repository, each commit against its parent. No change moved a warrant from `asserted` to `accepted`. No change made a state movement that a regime refuses, and that includes the one change where four obligation records reached `discharged`. A person checked those four by hand, and the rule that decides such a movement now agrees with the person.

**It cannot live in `taxonomy audit`, and an earlier draft of this section said it would.** That verb reads one working tree, and spec 12 makes the prior version available only in change-scoped evaluation. So the mis-assignment was the verb rather than the shape of the reading. What `taxonomy audit` reports instead is the population standing at `asserted`, which is the denominator a promotion rate divides by. That figure is a stock rather than a flow. A bulk stamp lowers it and forty separate acceptances lower it by the same amount. So it answers whether anything is ever promoted, and it does not answer whether one change was a review. [HW-OBL-0119](../obligations/0119-an-audit-reading-carries-no-declared-bar-so-a-distribution-cannot-become-a-finding.md) holds the missing bar, with the other readings that carry none.

Promotion never rewrites history. `drafted_by` and `agency` stay as they were, so the corpus can still say which parts an agent drafted.

## Capture cost is a tracked metric

Every design-rationale system of the last fifty years — IBIS, gIBIS, QOC — produced a rich model and almost no sustained adoption. There is one reason: **capture costs the author and benefits someone else, later.** Our evidence rules increase author cost. That trade may be correct, but it is the trade that historically kills these systems. Thus the trade is measured, not assumed.

The engine records the **assisted fraction** for each document created. Of the required front matter, sections, identifiers, and relations, how much was scaffolded, derived, or agent-drafted, and how much was hand-entered? The fraction is cheap to compute — the scaffolder knows what it supplied — and it trends.

The fraction is used in two ways:

- **As a design budget.** A hand-entered fraction that rises means that the taxonomy demands more than the tooling supports. The remedy is to derive more, or to require less. To add a lint that nags authors is the wrong move, and the metric makes that visible. An assisted fraction that *declines* is an assurance finding in its own right, not a trend line to glance at. It shows that the adoption model fails, measurably.
- **As the test of the agent-authoring claim.** [Spec 5](05-ai-integration.md) argues that an agent may draft from evidence already present in the commit, the ticket, and the conversation. Doing so shifts capture cost off the author. That is the first genuinely new answer to that objection in thirty years. Either the assisted fraction rises when agent authoring is enabled, or the claim is wrong. This is how we find out.

The metric is reported in the adaptive layer of the [assurance model](04-assurance-model.md), alongside efficacy results.

**The scaffolder reports the fraction of one run, and a run is all that it can report.** The denominator is the four terms above: the required front matter, the required sections, the identifier, and the halves of each proposed edge. A section counts as its heading and never as its prose. The heading is what the section contract reads, and the prose is what a person owes. Each edge counts two halves, and the reciprocal one is the half that follows from the target that an author named.

**No reader of a committed corpus can derive the fraction, and one ruling is why.** [Q4](09-decisions.md#q4--relation-storage) keeps `created_by` on the relation type, so the value states who a taxonomy expects to pay for an edge rather than who wrote one. A scaffolded edge and a hand-typed edge of the same relation are one string on disk. So the reading has to be taken where the work happens, and a trend needs a store that the run writes to.

## What the capture-cost store records, and what it refuses to

The store is `.headwater/capture-cost.jsonl`. A run of `headwater new` that writes a document appends one line to it, and no line is ever rewritten. A run whose document landed and whose reading did not exits non-zero. A silent hole in the denominator is the one state that no later reading can report.

**What a reading holds.** The four terms above, the kind, the path the run wrote, and the identifier it minted. Beside them go the date of the injected clock and the digest of the taxonomy the run resolved. The digest is there because the denominator is a count of declarations. A required facet that the scaffolder can fill raises the fraction with no change in what a person types. So two readings taken under two digests are two measurements, and `headwater capture` states how many digests the readings span rather than averaging them. [HW-OBL-0109](../obligations/0109-the-capture-cost-denominator-is-set-by-declaration-and-not-by-work.md) holds the measurement of what that denominator does and does not count.

**What a reading refuses to hold, and the reason for each.** Telemetry about capture cost is a measurement of people and of agents, so what is left out is a ruling rather than an omission.

| not recorded | why |
|---|---|
| any person or agent | The remedy this section names for a falling fraction is to derive more or to require less. It is aimed at the taxonomy and never at the author, and a per-author number is a performance measure |
| wall-clock time, and any duration | Neither is reproducible, and both would change the file on a run that measured the same thing |
| the prose, and the title | The store holds counts. The document holds the words |
| a run that refused | A refusal wrote no document, so there is nothing to attribute a reading to. A count of refusals is the first row under another name |
| whether a hook or a skill ran | [Spec 5](05-ai-integration.md#the-hook-contract-and-what-a-hook-cannot-bind) says a hook binds nothing. A disabled hook and a hook that stayed silent produce one reading. The count would then fall when a harness changed, and read as a fall in authoring |

**Where it lands, and who reads it.** Under `.headwater/`, which is outside the corpus root. No census row covers it, no language regime binds it, and no rule reads it. That is the boundary a taxonomy source already sits on, and it is there for the same reason. The file is committed plain text, so every reader of the repository recomputes each aggregate from the lines. A capture-cost number that a reader cannot recompute is a number that nobody should cite.

**What the store cannot be made to overstate.** `headwater capture` reports the reach of the verb as the classified documents that a reading names, against every classified document. A document that arrived by any other route is classified and named by no reading, so it raises the denominator and lowers the fraction. A reading whose document is gone is named in the report and counted nowhere. The join reads the identifier first and the path second, so a rename after the reading is a move rather than a loss.

**The reach figure carries the age of the store with it.** Every document that a corpus wrote before the store existed carries no reading and never could. So the report states the date of the first reading beside the fraction, and a reader holds the two together. A number that started at zero on the day the store shipped is not a finding about how those documents were written.

[HW-OBL-0001](../obligations/0001-the-promotion-fix-has-no-reading-of-the-assisted-fraction.md) carries what the store still does not settle.

## Authoring surfaces

| Surface | Use |
|---|---|
| `headwater new <kind>` | Scaffold a document with correct placement, metadata, sections, identifier |
| `headwater check --fix` | Apply mechanical corrections: format front matter, add missing reciprocal links, regenerate projections |
| Editor integration | Schema-driven completion and inline validation via a language server over front matter |
| Agent-assisted authoring | The judgment-bearing path: drafting, evidence checks, cross-linking (spec 5) |

All four converge on the same schema. There is no path into the corpus that skips it.

**Two of the three corrections in that row have a writer, and one does not.** A missing reciprocal link and a substitution inside prose each carry a patch. [Spec 12](12-check-layer.md#what-a-patch-may-say) states the two shapes a patch takes. A projection is regenerated by `headwater generate` rather than by a patch, which is one verb for one artifact. A correction inside front matter needs a read-back over a mapping, and the engine has none. So every rule whose remedy is a facet reports remediation prose, and [HW-OBL-0103](../obligations/0103-the-front-matter-half-of-a-patch-has-no-writer.md) holds the remainder.
