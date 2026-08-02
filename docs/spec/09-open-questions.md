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

**Leaning:** cache by default, with `docgov export` producing a committed JSON graph
for anyone who wants to build on it. Avoid a database until a query workload
justifies it.

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
