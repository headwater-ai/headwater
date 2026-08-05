# 3 — Authoring and lifecycle

How a document comes into existence, how it declares what it is, how it stays
trustworthy, and how it dies without taking its history with it.

## Front matter is the contract

Every document opens with YAML front matter carrying the facets its kind requires.
Front matter is the machine's only guaranteed read of a document, so the schema is
strict about it: required facets are required, unknown facets are reported, and
enum values outside a controlled vocabulary are findings.

Four facets do structural work in the default taxonomy:

- **state** (`status:`) — where the document sits in its lifecycle;
- **state-entry date** (`status_since:`) — when it entered that state, stamped by
  the transition (scaffolding and `check --fix` maintain it). This is the origin
  that windowed participation expectations are measured from
  ([spec 2](02-taxonomy-model.md#participation-expectations)), and it makes dwell
  time observable: a document parked in `draft` for a year is a fact the corpus
  can now state;
- **freshness** (`last_verified:`) — the date a human last confirmed the document
  matches reality, deliberately *not* the last-edited date, which git already knows
  and which says nothing about truth;
- **summary** — one sentence, machine-consumed. It is what the query layer, the
  generated indexes, and agent-facing pointers show. Writing it is not optional
  decoration; it is the document's public interface.

Everything else — owner, scope, audience, domain, provenance — is taxonomy choice.

## Lifecycle

A state machine declared in the taxonomy, interpreted by the engine. The default
taxonomy uses:

```
draft ──▶ current ──┬──▶ superseded
                    └──▶ deprecated
```

Rules the engine enforces from the declaration alone:

- transitions not in the declared machine are rejected **when they land**: an
  illegal transition is only visible against the prior state, which lives in the
  diff rather than the graph, so hooks and change-scoped CI receive the prior
  version as a declared check input
  ([spec 12](12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version)).
  A full-corpus run sees only current states and reports transition instances as
  change-scoped rather than silently passing them;
- terminal states marked `retain: true` may never be deleted — lineage is the point;
- a live document may not depend on a terminal one through a relation declared
  `lifecycle_sensitive`, so a current specification citing a superseded decision is
  a finding, automatically, for every such relation;
- entering a state may require facets (a `superseded` document must name its
  successor) or trigger reciprocal updates on the target;
- dwell in a non-terminal state is observable, not policed: `status_since` makes
  "parked in `draft` for a year" a fact, `taxonomy audit` reports the dwell
  distribution per state, and a shelf whose documents sit indefinitely in the
  state that carries the fewest obligations is a finding about the shelf. A
  document evading live-state obligations by never going live is the same
  absence class participation expectations catch, one level down.

**Correction versus succession** is a distinction the system takes seriously. A
document is edited in place when it was wrong about the present. A *successor* is
written when the decision itself changed. The first preserves truth; the second
preserves lineage. Conflating them destroys the record, so the lifecycle regime
makes the second path cheap and the first path honest.

## Freshness and staleness

`last_verified` is an assertion by a human: *on this date I checked that this
document is true*. The freshness facet's declared policy turns that into signal:

- format and plausibility are validated (no future dates);
- documents past the staleness threshold are reported, weighted by the shelf's
  criticality;
- a document whose backing code changed since its last verification is flagged
  ahead of one that is merely old — staleness is a function of drift risk, not
  only of calendar time.

Staleness is **detective, never blocking**. Blocking on it teaches authors to bump
the date, which converts the corpus's most valuable signal into noise.

## Voice

Some kinds describe the world as it is; some narrate change. Mixing them is the
most common failure in a documentation corpus, and it is mechanically detectable at
useful precision.

- **Declarative regime** (specifications, standards, architecture): present tense,
  present state. No future intent ("will be", "planned"), no change narration ("we
  moved from X to Y"), no phased-rollout language, no comparatives against a prior
  state. If a reader cannot tell whether a sentence describes today or last quarter,
  the document has failed.
- **Narrative regime** (proposals, evidence, incident records): time-bound by
  nature, and exempt.

Enforcement is lexical and therefore imperfect: a curated pattern set per forbidden
category, with per-file and per-block escape hatches that must state a reason. The
escape hatch is itself a signal — a shelf accumulating exemptions is a shelf whose
kind assignment is wrong, and the engine reports that concentration.

## Normative language

Where a document states requirements, the strength of each statement is explicit
(RFC 2119 keywords, or whatever set the taxonomy declares) and formatted
distinctly. The checkable parts: keywords appear in the declared casing; documents
using them carry the interpretation boilerplate; hedged pseudo-requirements
("should probably", "ideally must") are flagged.

This exists because unmarked requirements are the ones that get argued about later —
and because an agent reading the corpus needs to know the difference between a rule
and a suggestion.

## Templates and scaffolding

Each kind declares a template. The template is not a suggestion file for humans to
copy; it is generated from the kind declaration, so the required sections and the
required front matter always match what will be validated. A template that has
drifted from its kind is impossible by construction.

```
docgov new decision --title "Adopt overlay-based taxonomy customisation"
```

resolves the kind, allocates an identifier, seeds front matter, emits required
sections with prompts, places the file where the shelf layout dictates, and prints
which relations the new document is expected to declare.

## Identifiers

Some artefacts need stable names that survive being moved, renamed, or read out of
context: decisions, requirements, acceptance criteria, controls, obligations.

The taxonomy declares each identifier scheme's pattern, namespace, and allocation
policy. Three properties matter:

1. **Globally unique.** An identifier is namespaced at minting — by repository or
   organisation — so that when a corpus is vendored into another repository,
   identifiers from two sources cannot collide. Retrofitting a namespace later is
   expensive; the default is to always carry one.

2. **Resolvable without its document.** Given `DR-ACME-0042` and nothing else, the
   engine resolves it to a path. The graph carries an identifier index, so
   identifiers work in commit messages, code comments, tickets, and agent prompts.

3. **Never reused.** Allocation is reconcile-first: scan the corpus (including
   terminal-state documents) for the highest allocated value before minting. A
   deleted document does not free its number.

## Evidence has three honest states, not two

A decision recorded without evidence outside the document that records it is a
rationalisation. The system asks for a pointer — a work item, a substantive commit,
a recorded discussion, a measurement.

Where none exists, the earlier design offered two outcomes: evidenced, or a
registered gap. That is one short. The real design process is never as rational as
the record makes it look, and documenting it *as if* it were is both legitimate and
valuable — provided the reconstruction is labelled as one. Forcing every
after-the-fact account into "gap" pushes authors to overstate what they have, which
is the failure the rule existed to prevent.

| `evidence_basis` | Means | Obligation |
|---|---|---|
| `evidenced` | An external, auditable artefact supports this | The pointer resolves |
| `reconstructed` | Written after the fact from memory and inference | Must state what it was reconstructed from, and by whom |
| `gap` | No evidence exists and none is claimed | Appears in the gap register |

`reconstructed` is not a soft `evidenced`. It never silently promotes: moving a
document to `evidenced` requires adding a resolving pointer, and the transition is
recorded. A corpus where most rationale is reconstructed is telling you something
real about how decisions get made there, and hiding that behind a binary would waste
the signal.

The semantic judgement — *is this evidence actually about this decision?* — stays
with the author and the agent stop rules ([spec 5](05-ai-integration.md)). What is
mechanical: the facet is present and valid, pointers resolve, `reconstructed`
carries its basis, and the gap register accounts for every `gap`.

## Provenance is recorded, not assumed

Documents are now drafted by humans, by agents, and by both, and "who wrote this and
who accepted it?" should be a query rather than an archaeology exercise.

Every document carries provenance aligned with W3C PROV:

```yaml
provenance:
  agency: agent            # human | agent | mixed
  drafted_by: claude-opus-5
  activity: scaffold+draft
  accepted_by: j.baxter    # a human is always named here
  evidence_basis: reconstructed
  reconstructed_from: "commit 4a2f1c, ADO 1441575, design session 2026-07-15"
```

`accepted_by` is the load-bearing field. An agent may draft; acceptance is a human
act, and the record says who performed it. Generated projections are exempt — they
are `wasGeneratedBy` a tool and are checked against regeneration rather than
accepted.

This makes real questions answerable: which parts of the corpus are agent-drafted,
whether agent-drafted documents drift faster than hand-written ones, and whether
reconstruction correlates with agency. None of those can be asked of a corpus that
does not record the answer.

## Capture cost is a tracked metric

Every design-rationale system of the last fifty years — IBIS, gIBIS, QOC — produced
a rich model and almost no sustained adoption, for one reason: **capture costs the
author and benefits someone else, later.** Our evidence rules increase author cost.
That trade may be right, but it is the trade that historically kills these systems,
so it is measured rather than assumed.

The engine records, per document created, the **assisted fraction**: of the required
front matter, sections, identifiers, and relations, how much was scaffolded, derived,
or agent-drafted versus hand-entered. It is cheap to compute — the scaffolder knows
what it supplied — and it trends.

Two ways it is used:

- **As a design budget.** A rising hand-entered fraction means the taxonomy is
  demanding more than the tooling supports. The remedy is to derive more, or to
  require less; adding a lint that nags authors is the wrong move and the metric
  makes that visible. A *declining* assisted fraction is an assurance finding in
  its own right — the adoption model failing, measurably — not a trend line to
  glance at.
- **As the test of the agent-authoring claim.** [Spec 5](05-ai-integration.md)
  argues that an agent drafting from evidence already present in the commit, the
  ticket, and the conversation shifts capture cost off the author — the first
  genuinely new answer to that objection in thirty years. Either the assisted
  fraction rises when agent authoring is enabled, or the claim is wrong. This is how
  we find out.

Reported in the adaptive layer of the [assurance model](04-assurance-model.md),
alongside efficacy results.

## Authoring surfaces

| Surface | Use |
|---|---|
| `docgov new <kind>` | Scaffold a document with correct placement, metadata, sections, identifier |
| `docgov check --fix` | Apply mechanical corrections: format front matter, add missing reciprocal links, regenerate projections |
| Editor integration | Schema-driven completion and inline validation via a language server over front matter |
| Agent-assisted authoring | The judgment-bearing path: drafting, evidence checks, cross-linking (spec 5) |

All four converge on the same schema. There is no path into the corpus that skips
it.
