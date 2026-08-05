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

### The aggregator authors its own facts

"An aggregator that merges exported graphs" understates it, and the omission matters
once anyone tries to build one.

Some facts belong to no repository. That two services share an interface, that one
system's failure mode is another's operating assumption, that a capability is
implemented across four estates and owned by none of them — these are claims about the
*space between* repositories, and there is no repository whose export could carry
them. An aggregator restricted to merging can never state one.

So the federation layer is not only a merge target. It is a **corpus at a higher
altitude**, authoring the facts that genuinely live there and consuming exports for
everything else. Once that is admitted, the model is unchanged rather than strained:
it is still one corpus per repository, and the solution layer is one more corpus that
happens to be about other corpora. It has a taxonomy, its documents are Markdown, and
its cross-estate edges are declared in front matter like any other.

This also settles where authority sits, in the terms
[principle 2](00-vision-and-scope.md#design-principles) already sets: one source of
truth **per fact**, not per store. A document's content is canonical in its own
repository; a cross-estate edge is canonical in the solution corpus that declares it;
the merged graph is canonical for nothing. The question "is the graph or the Markdown
authoritative?" has no answer because it is the wrong question — nothing is
authoritative *as a store*.

**Leaning:** the solution layer is in scope and is an ordinary corpus, not a new
mechanism. Its access rules, however, are not ordinary — see
[Q17](#q17--governed-access-and-the-solution-layer).

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

### The OKF question is a different layer

The question above is about the **TBox**: whether an external language expresses our
schema and our checks. [OKF](11-adjacent-work.md#i1-okf--the-same-substrate-arrived-at-independently)
— LeanCTX's Markdown-plus-front-matter knowledge format — is about the **ABox**:
whether the *corpus itself* is exportable to something another tool already reads. The
two share the word "export" and nothing else, and conflating them would import the
substrate question's weight onto a decision that does not carry it.

It does not carry it for three reasons. It is **additive**: an emitter that nobody uses
costs a generator and a fidelity test, and deleting it later breaks nothing upstream.
It is **already most of the way done**: OKF is a directory of Markdown files with YAML
front matter, `type` required, and relations as Markdown links, which describes what we
already write. And it is **lossless in the direction that matters** — OKF carries
unrecognised front-matter keys through a parse-emit cycle untouched, so docgov facets
with no OKF meaning ride along under a `docgov_*` prefix rather than being dropped.

Two constraints if it is built. The emitted bundle is a **projection and never
canonical**, on the same terms Q6 sets for any RDF view: the Markdown corpus is the
truth, and a disagreement is the projector's bug. And OKF's own conformance checking
is four advisory warnings, so a consumer validating an exported bundle has verified
almost nothing about it — the export must not be mistaken for a second opinion on the
corpus. That is the same "declare yourself a subset" obligation the emitted shape set
carries above, for the same reason.

**Leaning:** yes, but late — after the graph export format is stable
([Q9](#q9--multi-repository-corpora)) and well after the substrate decision. It is a
day of work at the right moment and a distraction at the wrong one.

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

## Q16 — Public presence

**Blocks:** nothing technical. Blocks adoption entirely, and later than is
comfortable — by the time it obviously matters, the first impressions have been made.

[Q14](#q14--discovery-surface) covers how a *machine* finds a corpus cold. This is the
human half, and it is currently unplanned: there is no site, no sitemap, no positioning
for someone who has heard the name once and has four minutes.

[LeanCTX](11-adjacent-work.md#i5-the-presentation-is-the-lesson) is the standard to
match, and it is a useful standard precisely because it is not a large company — it is
one developer's project with a site that nonetheless assembles, coherently, what most
open specifications never manage:

| Facet | What it answers |
|---|---|
| How it works, architecture | What is this, and what is the shape of it? |
| Benchmarks, metrics | Does it do what it claims, in numbers someone can re-run? |
| Comparisons | Why this and not the adjacent thing I already know? |
| Use cases | Which of these is *me*? |
| Compatibility, integrations | Will it fit what I already run? |
| Docs, getting started | Can I have it working before I lose interest? |
| Pricing, enterprise, consulting | How does this survive, and what does it cost me? |
| Compliance, audits, self-assessment | What can I show the person who has to approve it? |
| Changelog, community, open-source posture | Is it alive, and is anyone else here? |
| `llms.txt`, AI-crawler-friendly `robots.txt` | Can a machine reader find and cite it? |

The last row is where this question touches Q14, and it is the one an ordinary
marketing site would omit. For a project whose entire thesis is that machines are
first-class readers, being unreadable to the machines that would recommend it is a
self-inflicted wound.

Two constraints particular to docgov. The site should be **generated from the corpus
that documents docgov** — anything else is a governance system whose own public
documentation is ungoverned, and that is the first thing a sceptical reader will check.
And the benchmark and self-assessment rows have to be **honest before they are
impressive**: §I.4 records claims that move between README versions as the thing that
made an otherwise strong project harder to trust, and a governance tool caught inflating
its own numbers has nothing left to sell.

**Leaning:** deferred, deliberately, until there is an engine worth visiting a site
about — but the sitemap is worth drafting early, because it is a forcing function for
positioning, and every column above is a question the specification should be able to
answer already. Where it cannot, that is a gap in the design rather than in the
marketing.

## Q17 — Governed access and the solution layer

**Blocks:** the graph export format ([Q9](#q9--multi-repository-corpora)) and the
discovery surface ([Q14](#q14--discovery-surface)), both of which currently assume a
reader entitled to see everything. It becomes urgent the first time an adopter wants a
contractor to read one shelf and not another.

Three proposals arrive bundled and separate cleanly. Keeping them apart is most of the
analysis, because they have very different merits and only one of them is hard.

| Proposal | Verdict |
|---|---|
| A cross-repository **solution layer** | Yes — [Q9](#the-aggregator-authors-its-own-facts)'s aggregator, extended to author its own facts |
| **Access control** over it | Yes — the substantive question, and the one this entry is about |
| **Graph authoritative, Markdown projected** | No — and unnecessary for either of the above |

### Why authority does not move

The case for inverting is that access control cannot be enforced on files someone has
already cloned. That is true and it is the right instinct pointed at the wrong layer.

Inverting costs four things the design currently gets free. **Capture cost**: spec 3's
survival argument is that authoring is a file edit in the same change as the code, and
routing it through a graph store rebuilds the tool-mediated capture step that killed
gIBIS. **Review**: [spec 4](04-assurance-model.md)'s controls trigger on pull requests
because documents diff there; a graph store does not. **Detectability**: Q6 already
warns that a projector nothing can check becomes the most trusted component in the
pipeline — today a projector bug is caught by comparing against the Markdown, and
inverting removes the thing it would be compared against, so the bug corrupts what
humans read instead. **Provenance**: blame, history and signed commits are free from
git and would have to be rebuilt.

Against that, inverting buys nothing the serving boundary does not already give, below.

There is a coherent version of the proposal, and it should be named so it is not
adopted by accident: an organisation for whom a repository clone is *itself* the leak
wants documentation never committed to the repository at all. That is a real market. It
also abandons "documents are files in the repository, next to the code they describe",
which is [spec 0](00-vision-and-scope.md)'s central bet and the reason capture is cheap.
It is a **pivot, not an extension**.

### Access control is a property of the serving boundary

Enforcement belongs where a reader is *served*, not where an author writes. Per-repository
Markdown stays canonical and carries the host platform's repository permissions; the
federated graph is a filtered view, and filtering happens there.

This puts the control exactly where the need is and nowhere else. Someone who can clone a
repository reading that repository is intended behaviour. Every case that motivates the
question — contractor, partner, adjacent business unit, "show the topology but not the
internals" — is cross-corpus, which is the federated layer by definition. Sensitive
material lives in a tightly-permissioned repository and is federated in; the graph serves
filtered views over the union.

The declaration surface already exists: `confidentiality` is a named facet
([spec 1](01-conceptual-model.md)). This promotes it from descriptive metadata to a
load-bearing security control — a small schema change carrying a large change in
obligation, since a mislabelled facet stops being a lint and becomes a leak.

### Four constraints, if it is built

**A filtered view must be legibly filtered.** The doctrine already exists twice — spec 4's
[no silent passes](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)
and Q13's declare-yourself-a-subset rule. Redaction obeys it: tombstones, never silent
omission, so a view reports *3 documents withheld* rather than looking complete. Without
this an agent traverses a redacted graph, finds nothing, and reports absence with
confidence — which is the failure [spec 5](05-ai-integration.md) opens by naming, caused
this time by our own security layer.

**Checks are privileged and total; serving is filtered.** Reciprocity cannot be evaluated
on a partial graph without inventing findings. Validation therefore runs at full
visibility regardless of who triggered it, and only results are filtered on the way out.
That seam is clean, and stating it prevents the obvious mistake of running checks as the
requesting user.

**Topology leaks even when content does not.** Shelf and kind names leak organisational
structure; node counts and edge shapes leak product structure; hiding a document while
keeping an inbound edge leaks its existence, and hiding both changes the graph's shape in
ways a determined reader can difference. This is the multi-level-security inference
problem and it has no clean solution. What the specification owes is honesty that this is
mitigation rather than a guarantee.

**Do not invent an identity system.** Derive from the platform's existing identity and
team model. Two permission systems that disagree means the documentation one is wrong,
and it is the one that leaks. Spec 7 already has the pattern for rules it cannot decide
from the repository tree — degrade to a recorded attestation with an owner and a date.

### The one place "visibility before blocking" cannot apply

[Principle 4](00-vision-and-scope.md#design-principles) says a new rule ships advisory and
earns its way to blocking. Access control is the single mechanism in the system where that
is wrong: shipping it advisory means shipping it broken, and every other control is allowed
to be wrong for a while precisely because being wrong is recoverable. A leak is not.

This exception belongs in the specification rather than in someone's judgement, because
the promotion machinery is otherwise uniform and will happily process a permission check
like any other.

### The sub-question that arrives silently: what may be a node

Access is the loud half of the solution layer. The quiet half is what the layer is
allowed to contain, and it is decided the moment someone writes its schema rather than
when anyone argues about it.

[Spec 11 §A](11-adjacent-work.md#a1-the-solution-layer-presses-on-that-boundary) sets
out the choice. The ABox currently stops at the document boundary: the corpus knows a
document exists, its kind and what it governs, never what it asserts. A solution layer
with a `Service` node describing an actual service has crossed that line and taken on
an obligation to stay true to the estate — which nothing in the design currently
carries, and whose drift is worse than stale prose because a wrong node reads as
structural rather than editorial.

**Leaning:** declared anchors. A solution-layer node carries an identifier, a name and
an owner, and asserts nothing further; every substantive claim stays inside a document
where freshness and the check layer already reach it. This is what
`code_path` already does — an external anchor kind that is referenced and never
described — and generalising it costs no new machinery. Revisit only against a
concrete need the anchor form cannot meet.

### What it changes about the project

Worth stating plainly, because it is a category change rather than a feature. Documentation
tooling with no access model is a developer tool. Documentation tooling with one is security
software: it acquires a threat model, an audit obligation, a disclosure process, and a class
of bug that cannot be fixed forward. That is a defensible business and it is the natural
shape of an enterprise tier ([Q16](#q16--public-presence)) — but it is not a facet someone
adds on a quiet afternoon.

**Leaning:** the solution layer proceeds now as an ordinary corpus; access control is
specified now and built late, after the graph export format is stable, and never as a
side effect of shipping federation. The four constraints above are the acceptance criteria
for the design, not a wish list — a filtered view that does not announce its filtering is
not a partial implementation of this, it is a defect.

## Q18 — Recording adjudicated disagreements

**Blocks:** nothing yet; becomes live the first time an agent must choose between two
current sources that a human has already judged.

Spec 2 formerly assigned kinds a scalar authority rank, consumed by an
`on_disagreement` rule. The core-concepts review cut it
([spec 2](02-taxonomy-model.md#disagreement-is-adjudicated-not-ranked)): the trigger
is a judgement the ABox boundary says the system cannot make, the rank pre-answers a
question that has not been asked, and a global scalar cannot express the scoped
precedence the prose demanded.

What the cut leaves open is the *positive* half: once a human (or a coherence-sweep
finding a human accepted) has adjudicated a specific disagreement, where does that
judgement live so agents and readers inherit it instead of re-deciding? Options: a
resolution recorded on the `conflicts_with` edge itself; a correction or succession
of the losing document; or a dedicated scoped-precedence declaration. The last is the
one to be suspicious of — it re-grows authority rank with more syntax.

This belongs beside [Q15](#q15--a-synthesised-content-tier)'s provenance questions:
both are about recording who vouched for what, and an adjudication without a named
adjudicator is a rank with extra steps.

**Leaning:** record adjudication per-conflict as data on the declared edge, with the
adjudicator named; no per-kind ranks, and no new declaration until a real corpus
shows the edge form failing.
