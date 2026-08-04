# 11 — Adjacent work and tooling

[Spec 10](10-theoretical-foundations.md) tested the design against the research
literature. This document does the same against **practitioner work and existing
tooling** — projects solving neighbouring problems, and standards that may already
implement parts of what spec 2 describes.

The question here is narrower and more uncomfortable: *what have we specified that
someone has already built?*

---

## A. TBox and ABox — the right name for a split we already made

Description logic separates a knowledge base into two parts. The **TBox**
(terminology box) holds the schema: classes, their hierarchy, the relations that may
hold between them. The **ABox** (assertion box) holds instances: individuals, their
attributes, and the actual relations asserted between them.

That is exactly docgov's split, and we have been describing it in invented
vocabulary:

| docgov | Description logic |
|---|---|
| Taxonomy (kinds, facets, relations, regimes) | TBox |
| Resolved taxonomy lock file | Compiled TBox |
| Corpus (documents and their typed edges) | ABox |
| `taxonomy validate` | TBox-internal consistency |
| `taxonomy audit`, `check` | ABox against TBox |

Adopting the standard names costs nothing and buys precision. It also explains why
the `validate` / `audit` split arrived at
[spec 6](06-engine-architecture.md#taxonomy-validate-versus-taxonomy-audit) — that
split is TBox reasoning versus ABox reasoning, which is a real distinction with
decades of theory behind it, not a convenience we invented.

**One boundary worth stating plainly.** docgov's ABox is the graph of *documents and
their declared edges* — not the claims made inside the prose. "This service returns
404 on a missing key" is a sentence in a document; the system knows the document
exists, what kind it is, and what it governs, but not what it asserts. Pretending
otherwise would promise semantic consistency checking we cannot deliver. The ABox
stops at the document boundary, and that limit belongs in the specification rather
than in a footnote someone discovers later.

> **Applied:** TBox/ABox vocabulary adopted in specs 1, 2 and 6, with the ABox
> boundary stated explicitly.

## B. Specification → ontology → implementation (testerstories)

Two posts working a Z-Machine header specification into an RDF/Turtle ontology, then
generating an implementation from it and testing the result. It is the most directly
relevant practitioner work found, because it runs the full pipeline the "structured
knowledge improves machine output" thesis depends on, and reports honestly on what
it did and did not achieve.

What the author does: prose spec → hand-crafted ontology (classes, datatype
properties, object properties) → LLM-generated implementation → tests that **query
the ontology at runtime** to derive what should be true. Version-conditional
behaviour that lived in table footnotes becomes explicit graph edges
(`applicableFrom`, `supersededBy`), and the generated code cites ontology individuals
by name in its comments.

Four things transfer directly.

### B.1 The corpus can be a test oracle

Their tests do not hardcode expectations; they ask the ontology what fields should
exist for the version found in the file. Coverage adapts because the schema is
queried, not copied.

docgov has the ingredients and has not connected them: acceptance criteria carry
stable identifiers ([spec 3](03-authoring-and-lifecycle.md#identifiers)), and
contract sidecars were credited as prior art but never actually specified. Connected,
they give the strongest possible form of "the specification describes what is" — the
specification *generates the check that proves it*.

> **Applied:** contract sidecars specified in [spec 2](02-taxonomy-model.md), with
> identified acceptance criteria as an oracle source.

### B.2 Source authority ordering

Their system prompt states a precedence: where the normative specification and the
conventional practice disagree, prefer the normative one **and note the
discrepancy**. The ontology carries `hasSourceAuthority`; the prompt turns it into a
decision rule.

docgov has no such ordering. `dominance` records whose *purpose* governs, which is
not the same question — when a standard and a specification disagree on a fact,
nothing in the corpus says which one a reader should believe. That is a genuine hole,
and it is precisely the situation where an agent will otherwise pick arbitrarily and
sound confident.

> **Applied:** an `authority` ordering on kinds, with disagreement handling, in
> [spec 2](02-taxonomy-model.md#authority-when-documents-disagree).

### B.3 Generated artefacts cite their source

Comments in the generated code name the ontology individual that justified each
decision (`field_font_width_v5`). Any output can be walked back to the artefact that
licensed it.

> **Applied:** citation-on-generation for agent-produced content, in
> [spec 5](05-ai-integration.md).

### B.4 The honest limit

The author is explicit that automated tests confirm *structure* while semantic
correctness still needs a human — the font-swap test proves the labels are present,
not that the swap is right.

That is [spec 4](04-assurance-model.md)'s cohesion/coherence boundary, reached
independently from an entirely different direction. Independent arrival at the same
line is the strongest evidence available that the line is real.

### B.5 What they do *not* claim

The author states plainly that local models are not deterministic, that identical
prompts produce different outputs, and that the reference implementation exists
*because* of that. Determinism is not claimed for the model; what the pipeline buys
is that deviation becomes **visible and attributable**.

This matters enough to state in our own terms, below.

## C. LinkML — the uncomfortable one

[LinkML](https://linkml.io/) is a schema language authored in YAML that compiles to
JSON Schema, SHACL, RDF/OWL, ShEx, Pydantic classes, SQL DDL, and GraphQL, with a
validation runtime.

Read [spec 2](02-taxonomy-model.md) and then read LinkML's documentation, and the
overlap is substantial: classes with slots, ranges, cardinality, enums with permissible
values, inheritance, and schema-level imports. Spec 2's Q2 leaning — "YAML plus a
published JSON Schema, with the resolved lock in a stricter representation" — describes
something LinkML already implements, along with the multi-format compilation we would
otherwise write.

Where it stops:

| docgov declaration | LinkML |
|---|---|
| Kinds, facets, vocabularies, cardinality | Direct fit — this is what LinkML is |
| Relations with endpoints and cardinality | Direct fit |
| Nuclearity, dominance, family | Expressible as annotations; not native semantics |
| Voice, lifecycle, freshness regimes | Not a data-shape concern; outside its model |
| Sequences, projections, overlays, core | No equivalent |
| Compatibility measurement | No equivalent |

**That table is wrong, and a [worked example](../evaluations/linkml-worked-example.md)
shows why.** Expressing the taxonomy in real LinkML establishes two things the guess
above missed. LinkML already ships three of the twenty research-derived changes —
SKOS mappings, PROV alignment, and `recommended` as advisory severity — and its
`designates_type` is our heterogeneous-shelf discriminator under another name. But
the boundary is not structural-versus-governance: *reciprocity* fails, and reciprocity
is as structural as anything in spec 2. The real line is that LinkML, SHACL, and JSON
Schema all validate **one instance against a shape**, while everything docgov does
that they cannot is a property of the **whole graph, or of the corpus over time**.

That reframes the question from "does LinkML cover enough?" to "is a two-layer
architecture — standard shape layer plus docgov graph layer — better than one custom
layer?" Three readings follow:

1. **Adopt it as the substrate.** Author the structural core as LinkML, layer
   docgov's governance declarations alongside, and inherit the meta-schema, the
   validator, and SHACL/JSON-Schema/OWL output. Less to build; a standard others
   already read; automatic interoperability.
2. **Stay independent, borrow the design.** LinkML's target is data models for
   research and biomedical data; a documentation taxonomy is a different animal, and
   a bolted-together schema — half LinkML, half ours — may be worse to author than
   either alone. The cognitive-dimensions walkthrough (Q2) is the instrument for
   deciding, and "two languages in one file" scores badly on role-expressiveness.

3. **Emit it, do not author in it.** Author in docgov's language and compile the
   shape layer *to* LinkML, which then generates JSON Schema, SHACL, OWL and Pydantic
   through LinkML's own toolchain. One authoring surface, fully validated, with a
   standards-based export. This option only became visible by writing the example out,
   and it is now the leading candidate.

The deciding evidence against option 1 is mundane: everything docgov-specific lands
in LinkML `annotations`, which are untyped pass-through. LinkML carries them and
validates none of them — so for exactly the half that is ours, the meta-schema benefit
disappears, and authors face two languages in one file with no visual cue for which
half is checked.

I am not settling this unilaterally: it changes what gets built, it is close to
irreversible under option 1, and it interacts with the language decision in Q1 —
LinkML's tooling is Python, which pulls against a Rust core. Option 3 dissolves that
tension, which is part of its appeal.

> **Recorded as [Q13](09-open-questions.md#q13--linkml-and-shacl-as-substrate).**

## D. SHACL — the name for schema-derived checks

[Spec 6](06-engine-architecture.md) says schema-derived checks are *generated from
the taxonomy*. SHACL (Shapes Constraint Language, W3C) is the standard that does this
for graphs: declare shapes, validate a graph against them, get structured violation
reports. If the corpus graph is expressible as RDF — and it is, being typed nodes
with typed edges — then SHACL shapes could *be* the check layer rather than something
we hand-write.

A [worked example](../evaluations/shacl-worked-example.md) confirms it reaches the
whole-graph layer LinkML cannot: reciprocity, conflict between two live decisions,
satellite inheritance, and windowed sequence expectations are all expressible — every one
of them by dropping to SPARQL, because SHACL Core can traverse but cannot refer back to
the focus node from the far end of a traversal.

My reservation here — that SHACL's reports are hard to read — was too strong and is
withdrawn: `sh:message` with variable interpolation makes messages as good as they are
authored. The objections that survive are sharper. **Line numbers, remediation and
fixability do not survive the RDF round trip**, and [spec 4](04-assurance-model.md)
requires all three to make a finding actionable. Temporal checks also need evaluation
time injected rather than read from the clock, or determinism breaks — a constraint that
applies to whatever engine we build, not just to this one.

Same conclusion as LinkML, by a different route: a compilation target, not an authoring
surface. Folded into Q13.

## E. OpenGEO — same substrate, opposite direction

[OpenGEO](https://github.com/donhaji/opengeo) is a specification for publishers to
declare canonical meaning to AI systems: Markdown bodies with YAML front matter,
organised as Discovery → Semantic → Context → Execution, with assurance treated as a
concern *around* the chain rather than a layer in it.

Two points of contact:

- **The substrate is identical.** Markdown plus YAML front matter as a semantic
  contract for machine readers, deliberately *not* RDF/OWL/SHACL. That is an
  independent data point that the heavyweight stack is not required for this class of
  problem — useful evidence when weighing Q13, and a caution against assuming the
  standards-based route is obviously correct.
- **Assurance is not a layer.** Their framing — provenance, authorship, freshness,
  auditability and ownership as oversight around the whole chain rather than a stage
  within it — matches [spec 4](04-assurance-model.md) and is a cleaner statement of it
  than ours.

But the direction is opposite. OpenGEO points **outward**: a publisher declaring
meaning to third-party engines it does not control, with execution explicitly out of
scope. docgov points **inward**: an organisation governing its own corpus, for its own
agents, where we control the whole pipeline and can therefore *check* things rather
than merely declare them. Their context layer (tone, persona, interpretation envelope)
follows from not controlling the consumer; we do control it, so we constrain behaviour
directly instead of requesting it.

The transferable gap is **discovery**. OpenGEO takes seriously how a machine reader
arrives cold and finds out what a corpus is. [Spec 7](07-distribution-and-federation.md)
covers distribution to repositories that already know about us, and says nothing about
an agent encountering the corpus for the first time.

> **Recorded as [Q14](09-open-questions.md#q14--discovery-surface).**

## F. Ontology-first, and the LLM-maintained wiki

Two related sources: the r/OntologyEngineering community, and Karpathy's *LLM Wiki*
pattern that the community identified as converging on its own position.

### F.1 The ontology-first thesis

The community's stated position is that ontology should come **first** — that you
build the model of the domain and let agents derive the stack that supports it,
treating implementation as a consequence of the model rather than the other way
round. Documentation generates the system; it does not describe one that already
exists.

That is the testerstories pipeline (§B) generalised into a methodology, and it is
further than docgov currently goes. Our specs *describe* a system that exists;
theirs *generate* one. The two meet at the oracle idea in §B.1 — a specification
precise enough to test an implementation is most of the way to one precise enough to
generate it — and it is worth being clear that docgov's design does not preclude
the stronger position, but does not currently claim it either.

### F.2 Karpathy's LLM Wiki

The pattern: raw sources stay immutable; an LLM incrementally builds and maintains a
wiki of interlinked Markdown between you and those sources; a schema file
(`CLAUDE.md` / `AGENTS.md`) tells the agent how the wiki is structured and what
workflows to follow. Operations are ingest, query, and **lint**.

Four things in it matter here, and one of them is a correction to something I wrote.

**The anti-RAG argument, better than mine.** The objection raised is not that
retrieval is imprecise — it is that retrieval *accumulates nothing*. Every question
re-derives its answer from fragments; the synthesis is thrown away; ask again
tomorrow and the work is done again from scratch. Knowledge should be **compiled
once and kept current**, not reconstructed per query. That is a stronger and more
durable argument than the precision one I gave in
[spec 5](05-ai-integration.md#what-we-do-not-do), and it is the argument docgov's
whole design rests on: validation, relations, and projections are all compilation
steps whose results persist.

Also worth noting as a data point: the pattern reports that a maintained index file
works well into the hundreds of pages *without* embedding infrastructure. A governed
documentation corpus is squarely in that range.

**Independent arrival at the capture-cost thesis.** The stated reason wikis die is
that maintenance burden outgrows value — the bookkeeping, not the thinking, is what
people abandon — and the reason this pattern survives is that an LLM does the
bookkeeping at near-zero cost. That is Grudin's capture bottleneck and
[spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric)'s
assisted-fraction answer, reached from practice rather than from the literature.
Two independent derivations of the same claim is the best support it is going to get
short of measurement.

**Lint as a coherence control.** Their lint pass looks for contradictions between
pages, claims superseded by newer sources, orphans, and concepts referenced but never
defined. Most of that is cohesion and the engine already does it deterministically —
but *contradiction between pages* is not, and it is exactly what
[spec 4](04-assurance-model.md#cohesion-and-coherence-are-different-obligations)
classes as coherence: undecidable structurally, requiring judgement.

An LLM lint pass is therefore a legitimate **coherence control** — sampled,
detective, non-blocking, exactly as spec 4 requires of that class. It does not
violate "no LLM in the validation path", because that rule governs *verdicts*, and a
coherence finding is a prompt for human attention rather than a verdict. This is a
real addition: spec 4 named the coherence class and left it thin.

> **Applied:** LLM-assisted coherence sweep added as a control class in spec 4.

**The gap it exposes: a third content tier.** The pattern's wiki is neither
hand-authored nor mechanically generated. It is *synthesised* — an agent's evolving
interpretation of sources, revised as new ones arrive. docgov recognises only two
tiers: authored documents and deterministic projections. Synthesised content fits
neither, and the difference matters, because a projection can be verified by
regenerating it and a synthesis cannot.

Whether docgov should admit a synthesised tier is a genuine question — it would need
its own provenance, its own staleness rules, and a clear statement that it is never
canonical for anything. It is also how most organisations will actually want to use
this.

> **Recorded as [Q15](09-open-questions.md#q15--a-synthesised-content-tier).**

## G. Knowledge-graph chunking for RAG — a problem we do not have

The TBox/ABox framing (§A) is the valuable half and stands independently of
retrieval. The chunking half — how to slice a knowledge graph into embeddable pieces,
and the trade between class-based and instance-based strategies — is solving a problem
docgov does not have, and the reasoning is worth recording because it also explains
*why* we do not have it.

Chunking exists because embedding retrieval must reconstitute meaning at query time
from fragments chosen by similarity. Every strategy in that article is managing a loss
that the approach introduces: class-based chunking orphans cross-class relations,
instance-based chunking fragments schema reasoning, and hybrid approaches are
recommended because neither loss is acceptable alone.

docgov never incurs the loss. Retrieval returns *identified documents reached along
declared edges*, so structure is not something to be reconstructed — it was never
dissolved. There is nothing to chunk because nothing is being embedded.

This is the position stated properly rather than assumed, and it is recorded in
[spec 5](05-ai-integration.md#what-we-do-not-do).

## H. What structured knowledge actually buys — stated precisely

The goal driving all of this is more deterministic machine behaviour. The claim needs
stating carefully, because the honest version is narrower than the marketing version
and it changes what we should build.

**Structured, governed knowledge does not make a language model deterministic.**
Sampling is stochastic; identical prompts produce different outputs; that is a
property of the model, not of its inputs, and no amount of schema fixes it. The
testerstories author is explicit about this, and keeps a hand-written reference
implementation precisely because of it.

What it does buy, and each buys something we can build toward:

| Mechanism | Effect |
|---|---|
| **Ambiguity removal** | Fewer legitimate readings of the input, so fewer defensible-but-divergent outputs. Variance narrows; it does not vanish |
| **Oracles** | An output can be *checked* against a declared expectation rather than judged by eye |
| **Attribution** | When output deviates, the artefact that licensed it is identifiable — so the fix lands on the corpus or the prompt, not on a hunch |
| **Reproducible comparison** | A pinned corpus plus a pinned model gives a baseline that a later run can be diffed against |

The achievable target is **bounded, auditable non-determinism**: output that varies
within a space the corpus defines, deviations that are visible, and causes that are
attributable. That is a substantially more useful goal than determinism, because it
survives contact with how these models actually work.

It also sets the investment priority. Effort belongs in **oracles and traceability**
— checkable expectations and citation of sources — rather than in prompt engineering
aimed at making the model repeat itself. The first compounds and is measurable; the
second is a treadmill.

> **Applied:** stated in [spec 5](05-ai-integration.md#what-structured-knowledge-buys).

## I. LeanCTX — our file format, none of our TBox

[LeanCTX](https://github.com/yvgude/lean-ctx) is a context-engineering layer for
coding agents: a local Rust binary that sits between an agent and the model and
compresses what passes through. Apache-2.0, ~3.5k stars, created March 2026, shipping
near-daily releases. Its pitch is token economics — read modes, AST-aware
compression, a shell hook that compresses `git` and `docker` output, a proxy that
compresses every request, and a property graph over *code* (imports, calls, exports).

It is not a documentation system, and its own `VISION.md` confirms this rather than
merely omitting it: no documentation corpora, no taxonomy, no schema validation. Where
it says **governance** it means governance *of the agent* — policy over what an agent
may see, signed evidence of what it saw, compliance reports — not governance of a
corpus. That distinction is worth holding onto, because the word is about to be
contested and the two meanings have almost nothing in common.

### I.1 OKF — the same substrate, arrived at independently

The point of contact is the **Open Knowledge Format**, which LeanCTX defines itself
and exports to:

- a directory of Markdown files, one concept per file
- YAML front matter, with `type` as the only required field
- relations as Markdown links — `- depends_on: [category/key](path.md)`
- a relation vocabulary of `depends_on`, `related_to`, `supports`, `contradicts`,
  `supersedes`
- written into the user's repository, byte-deterministic so exports diff cleanly

Typed nodes, typed edges, Markdown in the repository. That is docgov's substrate,
reached from an entirely different starting problem — which makes it a third
independent arrival at the same choice, after OpenGEO (§E) and the LLM Wiki (§F.2).
Three is enough to stop treating it as a preference.

### I.2 The arrow points the other way, and that is the whole difference

OKF is an **export**. The durable store is a `knowledge.json` under the user's config
directory — the only format that round-trips losslessly — and Markdown is a projection
*out* of it, for portability and hand-editing. docgov is the exact inverse: the
Markdown is the corpus, and the graph, indexes and rules are projections out of *that*
([Q6](09-open-questions.md#q6--where-the-corpus-graph-lives-at-rest) exists precisely
to keep it that way).

The inversion explains their validation, which is worth stating concretely because it
is the sharpest available illustration of what a taxonomy is *for*. `lint_okf_bundle`
returns warnings only — its own doc comment says the checks are advisory and "a
partially-malformed bundle should still import what it can" — and the complete set is:
not a directory, unreadable, missing front matter, missing `type`, empty body. Four
checks. The importer then does `get_str(fm, "type").unwrap_or("fact")`.

**`type` is a free string with a default.** There is no closed vocabulary, no
per-type required facets, no cardinality, no reciprocity, nothing whole-graph.

This is not a criticism. For an export format, lenience is correct engineering: the
obligation is to survive a round trip, and a format that rejects its own bundles
serves nobody. But it settles the overlap question. LeanCTX has our file format and
none of our TBox — and since [§C](#c-linkml--the-uncomfortable-one) established that
everything docgov does beyond one-instance-against-a-shape is the interesting part,
sharing a serialisation costs us nothing and threatens nothing.

### I.3 What transfers

**OKF as an export target.** We already emit Markdown with typed front matter; an OKF
bundle is close to free, and it buys interoperation with a tool a large number of
people have already installed. Their `leanctx_*` convention — producer-owned prefixed
keys that a consumer carries but never validates — is the right pattern for the
reverse direction too, and their round-trip test asserts exactly that unknown keys
survive a parse-emit cycle.

> **Folded into [Q13](09-open-questions.md#the-okf-question-is-a-different-layer),
> as a separate and much smaller question than the substrate one.**

**`contradicts` as a declared edge.** [Spec 4](04-assurance-model.md#cohesion-and-coherence-are-different-obligations)
classes contradiction as coherence — undecidable structurally, requiring judgement.
OKF carrying `contradicts` as a first-class relation exposes a middle case we have not
named: a contradiction an author has *declared* is structurally checkable even though
detecting undeclared ones is not. A `contradicts` edge with no resolving decision
record is an ordinary cohesion finding. Worth folding into spec 4's control set when
that document is next opened.

### I.4 Two things to be careful about

**The Context Governance Benchmark.** LeanCTX publishes a self-assessment against a
32-control, 6-family, 3-tier benchmark and claims "C2 — Managed". The structure is
recognisably [spec 4](04-assurance-model.md)'s — named controls, families, maturity
tiers, a published assessment — and if our control catalogue ever wants an existing
numbering to point at, it is a candidate. But the spec lives on a private GitLab
instance belonging to the same author, and its independence is **unverified**; it
should be read as self-published until shown otherwise. The controls themselves govern
agent behaviour rather than corpus quality, so little of the content transfers even if
the framing does.

**The claims move.** The repository description, the README and cached earlier
versions give the MCP tool count as 76, 82 and 62 respectively; the compression
percentages and the "4-layer verification engine" are unmeasured by anyone outside the
project. Roughly nine-tenths of the commits are from one author in under five months.
The OKF specification itself is small, legible and backed by round-trip tests, and can
be depended on directly — that judgement does not extend to the surrounding numbers.

### I.5 The presentation is the lesson

The most transferable thing here is not technical. LeanCTX ships a product site that
covers, coherently and in eighteen languages, what most open specifications never
assemble: how-it-works, architecture, benchmarks, compatibility, competitor
comparisons, six use-case pages, pricing, an enterprise tier, docs, changelog,
community, and a compliance self-assessment. Its `robots.txt` explicitly welcomes AI
crawlers under a "GEO" heading and it serves an `llms.txt` describing itself to
machine readers.

That last detail is not decoration — it is [Q14](09-open-questions.md#q14--discovery-surface)
already shipped by someone else, and it is evidence that the discovery surface has a
human half we have not planned for at all. A corpus nobody can evaluate from the
outside does not get adopted, however well it validates.

> **Recorded as [Q16](09-open-questions.md#q16--public-presence).**

---

## Summary

| Source | Verdict | Outcome |
|---|---|---|
| TBox/ABox framing | Adopt the vocabulary | Applied — specs 1, 2, 6 |
| testerstories, spec → ontology → implementation | Strongest supporting evidence found | Applied — oracle, authority, citation (specs 2, 4, 5) |
| LinkML | May already be half of spec 2 | **Open — Q13**, needs a decision |
| SHACL | May already be the check layer | **Open — Q13** |
| OpenGEO | Different direction, same substrate | Discovery gap recorded — **Q14** |
| KG chunking for RAG | Framing useful, chunking not applicable | Non-adoption reasoned and recorded |
| r/OntologyEngineering | Ontology-first methodology, further than we go | Noted; oracle work is the shared ground |
| Karpathy, *LLM Wiki* | Best anti-RAG argument; independent capture-cost confirmation | Applied — coherence sweep (spec 4); **Q15** raised |
| LeanCTX / OKF | Same substrate, opposite arrow; no taxonomy to collide with | OKF export folded into **Q13**; presentation gap raised as **Q16** |
