# 3 — Authoring and lifecycle

How a document comes into existence, how it declares what it is, how it stays
trustworthy, and how it dies without taking its history with it.

## Front matter is the contract

Every document opens with YAML front matter carrying the facets its kind requires.
Front matter is the machine's only guaranteed read of a document, so the schema is
strict about it: required facets are required, unknown facets are reported, and
enum values outside a controlled vocabulary are findings.

Three facets do structural work in the default taxonomy:

- **state** (`status:`) — where the document sits in its lifecycle;
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

- transitions not in the declared machine are rejected;
- terminal states marked `retain: true` may never be deleted — lineage is the point;
- a live document may not depend on a terminal one through a relation declared
  `lifecycle_sensitive`, so a current specification citing a superseded decision is
  a finding, automatically, for every such relation;
- entering a state may require facets (a `superseded` document must name its
  successor) or trigger reciprocal updates on the target.

**Correction versus succession** is a distinction the system takes seriously. A
document is edited in place when it was wrong about the present. A *successor* is
written when the decision itself changed. The first preserves truth; the second
preserves lineage. Conflating them destroys the record, so the lifecycle regime
makes the second path cheap and the first path honest.

## Freshness and staleness

`last_verified` is an assertion by a human: *on this date I checked that this
document is true*. The freshness regime turns that into signal:

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

## Evidence

A decision recorded without evidence outside the document that records it is a
rationalisation. The system asks for a pointer — a work item, a substantive commit,
a recorded discussion, a measurement — and where none exists, the honest outcome is
a registered gap rather than an invented justification.

This is enforced socially and by agent behaviour rather than by a linter (see
[AI integration](05-ai-integration.md)), because the check is semantic. What *is*
mechanical: an evidence facet that is present, well-formed, and resolvable, and a
gap register that accounts for the documents that lack one.

## Authoring surfaces

| Surface | Use |
|---|---|
| `docgov new <kind>` | Scaffold a document with correct placement, metadata, sections, identifier |
| `docgov check --fix` | Apply mechanical corrections: format front matter, add missing reciprocal links, regenerate projections |
| Editor integration | Schema-driven completion and inline validation via a language server over front matter |
| Agent-assisted authoring | The judgment-bearing path: drafting, evidence checks, cross-linking (spec 5) |

All four converge on the same schema. There is no path into the corpus that skips
it.
