# 9 — Open questions

Decisions deliberately deferred. Each blocks something; each is listed with the
options actually on the table and a current leaning, so the design phase argues
rather than rediscovers.

## Q1 — Implementation language

**Blocks:** everything downstream of the engine.

| Option | For | Against |
|---|---|---|
| Rust | Single static binary, fast, no runtime, good WASM story | Slower to iterate; smaller pool of contributors |
| Go | Single binary, fast enough, simple, easy plugins via subprocess | Weaker ergonomics for schema and graph work |
| TypeScript | Best editor/LSP and MCP ecosystem; agent tooling is native there | Runtime dependency; distribution is heavier |
| Python | Fastest prototyping; strongest text-processing library set | Distribution and performance are the known pain |

**Leaning:** Rust for the core with a WASM build for editor and browser embedding,
if the schema and graph work proves ergonomic enough. Go is the pragmatic fallback.
Decide via a spike that implements the parse-classify-graph path in two candidates.

## Q2 — Schema format

YAML is the default expectation, but the meta-schema, the diff experience, and the
overlay merge semantics all get better with a stricter format. Options: YAML with a
published JSON Schema; a typed configuration language (CUE, Dhall, KCL) that gives
validation and composition natively; or a small purpose-built DSL.

**Leaning:** YAML plus JSON Schema for the authored surface, because adopters must
be able to read and write it without learning a language — with the resolved lock in
a stricter representation. Investigate whether CUE can be an *optional* authoring
front-end for organisations that want it.

**How this gets settled.** Not by preference. The cognitive-dimensions framework is
the standard instrument for evaluating a notation, and this question is exactly what
it is for. Walk each candidate through five authoring scenarios — add a kind; rename
a shelf; split one facet into two; add a relation type to an existing family;
upgrade across a major version with a live overlay — and score each on:

| Dimension | Question |
|---|---|
| Viscosity | How much editing does a small conceptual change require? |
| Hidden dependencies | When I change this, what else changes that I cannot see? |
| Premature commitment | What must I decide before I have enough information? |
| Role-expressiveness | Can a reader tell what each part is *for*? |
| Error-proneness | Which mistakes does the notation invite? |
| Progressive evaluation | Can I check partial work, or only a complete schema? |
| Abstraction gradient | What must a beginner learn before writing anything at all? |

Viscosity and hidden dependencies will decide it, and the framework makes explicit
something the design has already chosen: **overlays deliberately trade viscosity for
hidden dependencies.** Customising by overlay makes change cheap (low viscosity) at
the cost of a resolved result no one authored directly (hidden dependencies). That
trade is defensible — but it means the schema format must claw back visibility, so
`explain`, `resolve`, and a readable lock file are not conveniences here. They are
the mitigation.

## Q3 — How much of the default taxonomy ships in the box

A taxonomy that is too opinionated repels adopters with an existing culture; one
that is too thin leaves them facing a blank schema. Options: minimal core plus
optional packages; one batteries-included default; or a `docgov init` interview that
composes a taxonomy from answers.

**Leaning:** a small core (decisions, standards, guides) plus optional packages
(specifications, evidence, operations, compliance), with an interview that composes
them. The interview matters more than the packages — the blank-schema problem is a
first-run problem.

## Q4 — Relation storage

Relations are declared in front matter today. Alternatives: inline in prose with
extractable syntax, or a sidecar edge file per document. Front matter is simple and
diffs well; prose links are where humans actually write references and are easy to
leave dangling.

**Leaning:** front matter is authoritative; prose links are *extracted* and checked
for resolvability but do not carry relation semantics unless annotated. Revisit if
authors find declaring the same link twice tedious — that friction is a real signal.

## Q5 — Voice checking depth

Lexical pattern matching is cheap, explainable, and imprecise. A small local
classifier would be more accurate and much less explainable — and a finding an
author cannot understand is a finding they suppress.

**Leaning:** lexical, with a well-curated pattern set, per-category severities, and
reasoned escape hatches. Revisit only with measured false-positive data.

## Q6 — Where the corpus graph lives at rest

Options: rebuilt from cache each run (simplest, no sync problem); persisted to a
committed JSON artefact (reviewable in diffs, enables tooling without the engine);
or an embedded database (fast queries, another thing to keep in step).

A fourth option arrived with Q13: an **RDF projection**, which is what SHACL validation
would consume. It carries a question the others do not — *fidelity*. Once checks read a
projection rather than the documents, the projector becomes the most trusted component in
the pipeline and nothing downstream can detect its mistakes; a projector that drops or
mistypes a document yields a graph that validates cleanly and does not represent the
corpus ([evaluation](../evaluations/shacl-worked-example.md#problem-one-the-projection-is-load-bearing-and-shacl-does-not-check-it)).

**Leaning:** cache by default, with `docgov export` producing a committed JSON graph
for anyone who wants to build on it. Avoid a database until a query workload
justifies it. If RDF is emitted, it is a **derived view and never canonical**, and it
ships with round-trip fidelity tests — the Markdown is the corpus, and any projection
that disagrees with it is the projection's bug.

## Q7 — Scope of the MCP surface

Read-only tools are obviously right. Whether an agent may *write* through the MCP
server — creating a decision record, updating a facet — is a trust and workflow
question as much as a technical one.

**Leaning:** read-only in the first release. Writes arrive later, behind explicit
opt-in, producing a change proposal rather than a commit.

## Q8 — Probe cost and cadence

Efficacy probes are the adaptive layer and they cost real money per run. Open:
which categories run on which cadence, what the monthly envelope is, whether probes
run against pull requests or only on a schedule, and how results are stored so that
trends are visible over quarters rather than runs.

**Leaning:** scheduled weekly, pinned model, deterministic rotation, results
committed as evidence records inside the corpus so the trend is itself governed
content.

## Q9 — Multi-repository corpora

The model assumes one corpus per repository. Monorepos with several independent
documentation sets, and organisations wanting a single query surface across many
repositories, both push against that.

**Leaning:** one corpus per repository stays the model. Cross-repository views are a
*federation* concern — an aggregator that merges exported graphs — not a change to
the corpus model. Confirm before the graph export format is frozen, because that
format is the aggregator's input.

The cross-*taxonomy* half of this is now answered: declared SKOS mapping relations
([spec 2](02-taxonomy-model.md#mapping-between-taxonomies)), not a merge. What
remains open is the aggregator itself — whether it is part of this project at all,
where merged graphs live, and how a query fans out across repositories that are not
all checked out at once.

## Q10 — Naming

`docgov` is a working name. The name matters for adoption and for the CLI verb
people type dozens of times a day.

## Q11 — Licence and distribution posture

Open source, source-available, or internal-only; and whether the default taxonomy
package ships under the same terms as the engine. This affects Q3 and Q7 and should
be settled early, since it is easier to open something later than to close it.

## Q12 — Migration path for an existing corpus

An organisation already running a comparable framework needs an on-ramp: a taxonomy
inferred from an existing corpus, a report of what does not fit, and an incremental
adoption mode where checks apply only to newly touched documents. Whether this is a
first-release feature or a follow-on determines how much the schema must tolerate a
half-conformant corpus — which is a design constraint, not a feature request.

**Leaning:** `docgov infer` (propose a taxonomy from an existing tree) and a
`--since <ref>` mode are first-release. Adoption friction is the thing most likely
to kill this, and both of these directly attack it.

## Q13 — LinkML and SHACL as substrate

**Blocks:** Q1 and Q2, and it is close to irreversible once the schema format ships.

[LinkML](https://linkml.io/) is a YAML-authored schema language that compiles to JSON
Schema, SHACL, RDF/OWL, Pydantic, and SQL DDL. It covers a substantial part of what
[spec 2](02-taxonomy-model.md) specifies structurally — classes, slots, ranges,
cardinality, enums, inheritance — and none of the governance half (regimes,
sequences, overlays, core, compatibility). SHACL, similarly, is the standard for
validating a graph against declared shapes, which is what our schema-derived checks
do by hand.

| Option | For | Against |
|---|---|---|
| 1. Author in LinkML | Meta-schema, validator, and multi-format output for free; a standard others already read | Everything docgov-specific lands in untyped `annotations` that LinkML never validates, so the meta-schema benefit disappears for exactly our half; two languages in one file; its Python tooling pulls against a Rust core (Q1) |
| 2. Borrow the design, stay independent | One coherent language; full control of authoring ergonomics | We rebuild a validator and a compiler that already exist, and forfeit interoperability |
| 3. **Author in docgov's language, emit LinkML** | One validated authoring surface; LinkML's generators then produce JSON Schema, SHACL, OWL and Pydantic for free; reversible, since a generator can be changed or dropped; dissolves the Q1 tension | A generator to build and maintain, plus fidelity tests proving the emitted schema accepts exactly what docgov accepts |

A [worked example](../evaluations/linkml-worked-example.md) settles two things that
were guesses when this question was raised.

**LinkML already ships more than expected.** `recommended` is advisory severity;
`designates_type` is our heterogeneous-shelf discriminator; SKOS mapping slots and
PROV `slot_uri` alignment are native — three of the twenty research-derived changes,
for free.

**The boundary is not structural-versus-governance.** *Reciprocity* is not
expressible in LinkML, and reciprocity is as structural as anything in spec 2. The
real line is that LinkML, SHACL and JSON Schema all validate **one instance against a
shape**, whereas everything docgov does that they cannot — reciprocity, satellite
inheritance, cross-endpoint conflict rules, sequence expectations — is a property of
the **whole graph or of the corpus over time**.

**SHACL reaches the layer LinkML cannot.** A second
[worked example](../evaluations/shacl-worked-example.md) shows reciprocity — the
constraint that defeated LinkML — is routine in SHACL, along with cross-node conflict
detection, satellite inheritance, and even windowed sequence expectations. All of them
require dropping to SPARQL, because SHACL Core can traverse but cannot refer back to the
focus node from the far end of a traversal.

Two corrections to what this question originally recorded. The **error-message objection
is withdrawn**: `sh:message` with variable interpolation makes messages as good as they
are authored, and generated shapes would be as good as our generator. The **SPARQL-engine
objection is weaker than stated**: embeddable Rust SPARQL engines exist, so it needs
measuring against the change-scoped budget rather than assuming. The objections that
survive are different and sharper — **line numbers, remediation and fixability do not
survive the RDF round trip**, and those are what make a finding actionable under
[spec 4](04-assurance-model.md#findings).

**Leaning:** option 3, to be confirmed by the Q2 walkthrough — and extended: emit
**LinkML for the shape layer and SHACL for the graph layer**, so an external consumer can
validate a docgov corpus to useful depth without installing docgov. It makes the
shape/graph split an explicit architectural seam rather than an accident, and every
mature validation stack in this space already has that shape.

The decisive argument is the same for both, and stronger for SHACL: every interesting
constraint is embedded SPARQL, which is *less* readable hand-authored than an engine
predicate — but generated, nobody reads it, and readability stops being a cost. What
stays docgov-native either way: schema operations, statistical measures, instrumentation,
and the finding shape. The emitted shape set is deliberately a **subset**, and must
declare itself as one — an external validator reporting a clean run while believing it
checked everything is worse than one that knows what it skipped. Counter-evidence worth keeping
in view: [OpenGEO](11-adjacent-work.md#e-opengeo--same-substrate-opposite-direction)
solves a neighbouring problem on Markdown and YAML while explicitly declining
RDF/OWL/SHACL, so a standards-based route is not self-evidently correct.

## Q14 — Discovery surface

**Blocks:** nothing yet; becomes urgent the moment a corpus is consumed by anything
that did not clone the repository.

[Spec 7](07-distribution-and-federation.md) covers distribution to repositories that
already know about the publisher. Nothing covers an agent or tool encountering a
corpus cold: how it discovers that a corpus exists, what taxonomy governs it, what
version, and where to start reading.

Prior art worth copying from rather than reinventing: a well-known file at a
predictable path, link relations from rendered pages, and an MCP server advertising
the corpus as a capability. All three are cheap, and the first two work without any
docgov installation at all.

**Leaning:** a small machine-readable descriptor at a fixed path — taxonomy identity
and version, corpus root, entry points, and the graph export location — plus the MCP
surface for agents that can use it. Defer until the graph export format is stable,
because the descriptor should point at it.

## Q15 — A synthesised content tier

**Blocks:** the provenance model, if the answer is yes.

docgov recognises two kinds of content: **authored** (a human wrote it; it is
canonical) and **generated** (a projection; verified by regenerating it and
comparing). Karpathy's LLM Wiki pattern
([spec 11](11-adjacent-work.md#f2-karpathys-llm-wiki)) is built on a third:
**synthesised** — an agent's evolving interpretation of sources, revised as new ones
arrive.

It fits neither existing tier, and the difference is not cosmetic: a projection is
verifiable by regeneration, a synthesis is not. Two runs over the same sources
produce different prose, both defensible.

Open: whether docgov admits synthesised content at all; if so, whether it needs its
own staleness rules, how it is prevented from ever becoming canonical for anything,
and whether a human acceptance step promotes it to authored or whether it stays
permanently second-class.

**Leaning:** admit it, permanently non-canonical, clearly marked, never a valid
target for a `governs` or `verifies` relation — with promotion to authored requiring
an explicit human acceptance that changes its provenance record. It is how most
organisations will actually want to use this, and refusing to model it just means it
happens unmarked.
