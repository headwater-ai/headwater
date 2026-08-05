# 0 — Vision and scope

## The thesis

A documentation corpus is a **structured artefact with invariants**, not a pile of
prose. Once its structure is declared in a form a machine can read, four things
become possible that are impossible otherwise:

- the corpus can be **validated** — not for spelling, but for truthfulness signals,
  completeness, lineage, and internal consistency;
- it can be **navigated deterministically** — a task, a code path, or a concept
  resolves to the documents that govern it, without search or luck;
- it can be **projected** — indexes, site navigation, agent instruction files, and
  reader-facing summaries are generated, never hand-maintained;
- it can **hold itself accountable** — the system records which mechanism discharges
  which obligation, and where it currently has none.

Everything in this specification follows from taking that seriously.

## Who this is for

**Primary:** an engineering organisation that wants its documentation to be
load-bearing — used by humans at review time and by coding agents at work time —
and is willing to accept structure in exchange for trust.

**Secondary:** a single team or solo maintainer who wants a strong default
documentation system without inventing one, and without the tool dictating a
taxonomy that does not match how they think.

**Explicitly served:** organisations whose documentation culture does **not** match
ours. The taxonomy is theirs to define; the engine is ours.

## What we are building

1. **A taxonomy schema language** — a declarative, versioned, validated description
   of a documentation corpus's structure: shelves, document kinds, metadata facets,
   relations, voice and lifecycle regimes, identifier schemes, and overlays.

2. **An engine** that reads a corpus plus its taxonomy, builds a typed graph, and
   evaluates constraints over it — one parse, many checks, machine- and
   human-readable output, advisory by default and blocking on request.

3. **A projection layer** that generates every derived artefact from that graph:
   shelf indexes, decision-lineage views, traceability matrices, site navigation,
   and the AI instruction surface.

4. **An AI integration surface** spanning the four moments an assistant touches
   documentation — planning, reading, writing, reviewing — plus a probe harness
   that measures whether the surface actually works.

5. **A distribution model** so one organisation can publish a taxonomy (and the
   doctrine that explains it) and many repositories can consume it, customise it by
   overlay, upgrade it deliberately, and prove they are still conformant.

6. **A doctrine starter kit** — an opinionated default taxonomy and the prose that
   explains it, shipped as a package that a new adopter can take wholesale, take
   partially, or ignore.

## What we are explicitly not building

| Not building | Why | What to use instead |
|---|---|---|
| A documentation renderer | Static-site generation is solved | Emit navigation config for MkDocs / Docusaurus / Astro |
| A wiki or an editor | Documents are files in the repository, next to the code they describe | Any editor; the corpus is Markdown |
| A code-analysis tool | We check documents and their declared links to code, never the code's meaning | Language-specific tooling |
| A prose style checker | Voice and structure are in scope; grammar and readability are not | Vale, textlint — composable alongside |
| A ticketing or workflow system | We reference work items; we do not manage them | Whatever tracker is in use |
| An LLM product | The engine is deterministic. LLMs are consumers of the corpus, and one optional authoring surface | — |
| A general knowledge base | The corpus documents a system, for people who change that system | — |

## Design principles

These are the tie-breakers when a design decision is genuinely contested.

1. **Configuration over code.** Anything an adopter might reasonably want different
   belongs in the schema. If a change to taxonomy requires a change to the engine,
   the design has failed.

2. **One source of truth per fact, many paths to it.** A fact — including a fact
   *about the corpus's structure* — is declared once. Prose that restates the schema
   is generated from it or validated against it, never maintained in parallel.

3. **Derived artefacts are always regenerable.** If a human can hand-edit a
   generated file without the system noticing, it will drift and be trusted while
   wrong. Generation is checked, not merely offered.

4. **Visibility before blocking.** A new rule ships advisory, accumulates evidence
   about its false-positive rate, and is promoted to blocking on that evidence. The
   promotion criteria are recorded, not improvised.

5. **Explicit incompleteness.** The system records what it does not check. A gap
   that is registered is triageable; a gap that is invisible compounds.

6. **Every rule earns its place.** Context is finite for humans and metered for
   agents. A rule with no enforcement, no observed violation, and no stated cost of
   failure is removed, not tolerated.

7. **Fail open at the edges, closed at the core.** Agent-facing helpers degrade
   silently when they cannot answer (a missing hint is better than a wrong one);
   corpus validation does not.

8. **The system governs itself.** docgov's own documentation is a docgov corpus,
   validated by docgov in its own CI. A change that is painful to dogfood is a
   change we have not finished designing.

## Success criteria

The system is working when:

- a new document's correct location, template, metadata, and required links are
  determined by the schema, not by asking someone;
- an organisation with a different taxonomy adopts it by writing a schema, not by
  patching code;
- an agent given a task retrieves the governing documents before reading source,
  and measurably follows them more often than an agent without the corpus;
- a taxonomy change propagates to consumers as a reviewable, verifiable migration
  rather than a broadcast request;
- every stated invariant either names the control that discharges it or appears in
  the gap register, with no third category.

## Non-negotiables

- **Markdown with YAML front matter** is the document format. No proprietary store.
- **The corpus is valid without the tool.** Everything degrades to readable
  Markdown in a browser or a text editor.
- **No network dependency at check time.** Validation runs offline, in a container
  or on a laptop, with the same result as CI.
- **Deterministic core.** Given the same corpus and schema, the engine's output is
  byte-identical. LLM involvement is confined to authoring assistance and to the
  efficacy probes, and is never load-bearing for a verdict.
