# Headwater website — design brief

Research input for Claude Design. Written 2026-08-23. Everything here is either measured from a run of this engine, quoted from a source, or marked as a judgment call.

---

## 1. The decisions already made (do not re-open these in design)

| Decision | Value | Source |
|---|---|---|
| Buyer | Platform / DevEx engineering lead — owns the CI gate and the internal docs | Owner, 2026-08-23 |
| Primary CTA | Run the tutorial | Owner, 2026-08-23 |
| Secondary CTA | Book a call (services) | Derived from the offer below |
| Site architecture | Marketing hand-built, docs generated from the corpus | Owner, 2026-08-23 — **this amends Q16 and needs a decision record** |
| Terms | Apache-2.0, engine and base package. DCO, no CLA | Q11, ratified 2026-08-11 |
| Domain | `headwater.tools`, Cloudflare static assets from `site/` | `wrangler.jsonc` |
| Namespace | `https://w3id.org/headwater/` — deliberately *not* the site domain | Q10, Q14 |

**The offer, in the order it can be delivered:**

1. **Corpus audit** — fixed price. Point the engine at their corpus, deliver findings, coverage, and an obligation register. Deliverable today, repeatable, and it is the wedge that becomes (2).
2. **Consulting / implementation** — run the interview, build their taxonomy, wire the gate into CI. Day rate.
3. **Hosted authoring + measurement** — later. Q7 and Q17 both leave "what a hosted server is, operationally" unspecified, so it cannot be priced yet.

---

## 2. Positioning: what the two comparables actually are

The owner named LeanCTX and TrustGraph. **Neither is a competitor**, and the specification already prices both. This matters for the site because it turns the comparison page from a defensive exercise into the strongest page on the site.

### LeanCTX — shares the file format, none of the type system

[LeanCTX](https://github.com/yvgude/lean-ctx) is a context-engineering layer for coding agents: one local Rust binary between agent and model that compresses what passes through. Apache-2.0, ~3.6k GitHub stars, 261k installs claimed, created March 2026, near-daily releases, roughly nine-tenths of commits from one author.

- **The overlap is the Open Knowledge Format**, which LeanCTX defines and exports to. HW-EVAL-adjacent-work §I already rules: *"LeanCTX has our file format and none of our TBox… a shared serialization costs us nothing and threatens nothing."* Q13 places an OKF emitter **fifth** in the emitter order, triggered by LeanCTX as the named consumer.
- **Their pitch is token economics** (60–90% fewer tokens). Headwater's is whether the documents are *true*. Different layer, and the site should say so in one sentence rather than avoid the name.
- **Commercially**: free open source for individuals, premium team features and cloud sync through Thinkery GmbH. The same free-core-paid-hosted shape Headwater is heading for.
- **Their landing page**, for reference: hero problem statement *"Your agents re-read everything. Every single time."*, a logo wall, a named customer case study with two percentages, four capabilities, three vanity metrics (installs, stars, energy saved), an FAQ, and a one-line install. Eighteen languages. Product / Use Cases / Pricing / Enterprise navigation, 81 docs pages.

### TrustGraph — the same pitch, the opposite mechanism

[TrustGraph](https://trustgraph.ai/) is a containerized context-engineering platform, Apache-2.0, San Francisco, enterprise logos including Nvidia, AstraZeneca, McKinsey and the UK Government. Headline: *"Build a unified semantic context layer with a hypergraph."*

HW-EVAL-adjacent-work §J states the difference exactly, and it is the cleanest sentence available for a comparison page:

> In Headwater, authors **declare** the graph: documents are the nodes, front-matter references are the typed edges, and no LLM issues a verdict. In TrustGraph, an LLM **extracts** the graph: documents are feedstock, dissolved into triples. Nothing in the platform governs the source documents — it mines them.

HW-EVAL-adjacent-work also records TrustGraph as a **candidate integration, not a rival**: a governed corpus is an unusually good input to an extraction platform, because kinds, facets and declared edges arrive as structure the extractor would otherwise guess at.

**Commercially**: no published pricing. Three CTAs in the hero (Get Started → GitHub, a video, and a Calendly "Schedule a Chat"), a gated Playground preview, and inferred enterprise consulting. This is the *services* pattern, and it is the closer analogue to what Headwater wants to sell.

### Vale — the closest analog, and it already drew the commercial line

Spec 0 names [Vale](https://vale.sh/) as the closest observed analog to this engine. MIT-licensed CLI, free for any team of any size, plus **Vale Studio**, a hosted product from the same maintainer adding a web UI, team management and analytics.

The lesson, already recorded in the first-contact evaluation: *"The command-line checker is not the product. The place where a person cannot easily self-host is."* That is exactly where Q7, Q8 and Q17 left a hole — a hosted server and a hosted probe harness. **The site should not sell that yet**, but it should not architecturally preclude it either.

---

## 3. The constraint that becomes the differentiator

This is the single most important thing for the designer to understand, and it inverts normal SaaS landing-page instinct.

**Principle 11 forbids publishing a claim that no run produced.** The Q16 sitemap audit ran against the specification and found the benchmark row **empty, and it stays empty** until a measurement campaign runs. Q16 says the pressure to relax this *"will arrive exactly when the site does."*

So: **no "10× faster", no "90% less drift", no invented percentages, anywhere.** Every efficacy claim in the repository is marked unmeasured, and the site must say so. This is the direct opposite of LeanCTX's landing page, which leads with two customer percentages and three vanity counters.

What replaces the fake metrics is *verifiable machinery*. From a real run of `headwater check` on this repository, 2026-08-23:

| Fact | Value |
|---|---|
| Files under the corpus root | 245 |
| Typed documents | 201 |
| Untyped, and reported as such | 2 |
| Silently unaccounted for | **0** |
| Documents in the read set | 202 |
| Findings reported | 523 |
| Findings suppressed, each by a dated directive with a stated reason | 4 |
| Migration-pending findings | 0 |
| Taxonomy in force | `headwater/standard 3.3.0` |
| CLI verbs shipped | 30, across 14 verb families |
| Milestones | M1–M5 closed; M6 (distribution) and M7 (measurement) open |

**The move: publish the honest number, including the ugly one.** 523 findings on our own corpus is not a failure to hide — it is the proof the thing runs, and every one of them is accounted for. A governance vendor showing a clean board is less credible than one showing a full board with a name against each item.

This also gives the site its best single line, which is already written in Q16:

> A comparison page that carries its own counter-evidence is unusual enough to be the difference, and it is already written.

HW-EVAL-adjacent-work contains arguments *against* Headwater — OpenGEO declining the standards stack for a neighbouring problem, TrustGraph shipping the opposite mechanism, Vale being the closest analog and in another language, and §M stating that the survey licenses no efficacy conclusion at all. Publishing that is the differentiator.

---

## 4. Sitemap

Marketing pages are hand-built. Everything under `/docs` and `/spec` is the generated projection.

### Hand-built

| Page | Job | Notes |
|---|---|---|
| `/` — landing | Answer "is this for me?" in under 30 seconds for a platform lead | Primary CTA: install line + tutorial. Secondary: book a call |
| `/how-it-works` | Taxonomy → graph → checks → projections, in four diagrams | Sources: spec 1, 6, 12 |
| `/compare` | LeanCTX, TrustGraph, Vale, plain linters, static-site generators — *including the arguments against us* | The strongest page. Content already exists in HW-EVAL-adjacent-work |
| `/services` | Audit, consulting, and "hosted, later" | The revenue page. See §6 |
| `/audit` | Fixed-price corpus audit — what you get, what it costs, how to start | The conversion page that is deliverable today |
| `/proof` | Self-assessment. Live numbers from our own run, benchmark row explicitly empty | Marked self-published, per §I.4's standard applied to ourselves |

### Generated projection

| Path | Source |
|---|---|
| `/docs/tutorial` | `docs/tutorials/your-first-governed-corpus.md` — 16 steps, CI-verified |
| `/spec/*` | The 14 typed specification documents, plus the decision register and the obligation register. Index generated by `headwater generate` |
| `/decisions/*` | 29 decision records |
| `/obligations/*` | 129 obligation records — the open-defects register, published |
| `/ns/` | Namespace documentation (exists today) |
| `llms.txt`, `robots.txt` | Q16: *"Ship it, and count it as nothing"* — 97% of llms.txt files go unread in a month |

---

## 5. Landing page — recommended structure and copy direction

**Hero.** The problem sentence from the README is already good and already passes the house language rules:

> Documentation rots because nothing holds it accountable. Specs drift from code, rationale evaporates, and the AI assistants now reading that documentation as context inherit every one of those faults — silently, and at scale.

Then the differentiating question, which is the line no comparable can answer:

> Nothing else can answer *"is this corpus still true?"* — because nothing else knows what kind of document anything is.

Hero CTA: a copyable install line and `headwater init`, leading to the tutorial. Secondary, quieter: *Book a corpus audit*.

**Section 2 — the three commitments.** Taxonomy is data. The corpus is a graph. AI assistants are readers in their own right, and measured ones. Already written, already tight.

**Section 3 — the honesty band.** Where a SaaS site puts logos and metrics, put this instead: a panel of the numbers from our own run, with the benchmark row visibly and deliberately empty, labelled *"unmeasured — principle 11 forbids a number that no run produced."* Nothing else in this category does this. It is the whole brand.

**Section 4 — how it works.** Four steps, four diagrams.

**Section 5 — what it is not.** Spec 0's "what we do not build" table. Refusing scope publicly reads as confidence to a platform lead who has been sold too much.

**Section 6 — comparison teaser** → `/compare`.

**Section 7 — services.** One band, low-key, at the bottom. This audience is repelled by a sales-first page and converts on the tool.

**Section 8 — status, honestly.** M1–M5 shipped; M6 and M7 open. State it. The README already does.

---

## 6. The services page

The buyer is a platform/DevEx lead. They do not buy transformation. They buy a bounded engagement with a named deliverable.

**Lead with the audit, because it is deliverable today and it is a product rather than a rate card.**

> **Corpus audit.** We point the engine at your documentation and give you back what it found: every file classified or explicitly reported as unclassifiable, every finding named and located, a coverage report, and an obligation register of what your corpus owes. Fixed price, fixed scope, and you keep the taxonomy we write.

Then implementation:

> **Implementation.** We run the interview, build the taxonomy that matches how your organization actually writes, and wire the gate into CI so that a document which drifts fails a build rather than a review. Your taxonomy is yours — Apache-2.0, no lock-in, and customizing it never means forking the tooling.

Then the honest forward-look:

> **Hosted authoring and measurement.** Not yet. The engine is cheap to run and expensive to build; the hosted layer is the reverse. We will say what it is when we can say what it costs.

That last paragraph is a sales asset rather than a weakness. It is the same voice as the empty benchmark row, and the consistency is what makes the honesty read as real rather than as positioning.

---

## 7. Visual and tonal direction

**What the current site already gets right** and should be kept and extended rather than replaced: warm off-white ground (`#fbfaf8`), near-black serif body text, deep teal accent (`#1d5c54`), a proper dark theme, and a 40rem measure. It reads like a specification, which is the right credibility register for this buyer. The 🜄 alchemical water glyph is a good, quiet mark.

**Direction:** *engineering document, not SaaS landing page.* The nearest visual references are a well-set standards document or a research paper, given generous space — not a gradient hero. Both comparables sit in minimalist Swiss technical, so competing on that axis is a tie. Competing on *looks like the thing it governs* is a win.

Specifics for the designer:

- **Keep the serif for body prose.** It is the differentiator against every developer-tool site in this space, and it suits documents about documents.
- **Sans for structure only** — labels, table headers, navigation, code annotations. The existing pairing already does this.
- **Terminal output is a primary visual element.** Real `headwater check` output, unedited, is the most persuasive asset available. Set it properly; do not screenshot it.
- **Tables over cards.** This audience reads tables. The Q16 sitemap audit and the spec 0 refusal table are both more convincing as tables than as feature grids.
- **The empty benchmark row must look deliberate** — a rule, a gap, a label. Designed, not broken.
- **No stock illustration, no abstract 3D graph render, no logo wall.** There are no customer logos, and inventing social proof would violate the principle the site is built on.
- **Diagrams:** the graph as nodes and typed edges, drawn as an actual small corpus using real document names from this repository.

**Words to avoid**, because the repository bans them in its own prose and a check enforces it: *load-bearing, first-class, battle-tested, north star, delve, seamless, holistic, deep dive, leverage* as a verb, and *robust* as filler. American spelling throughout. Short sentences — the house rule is 25 words, no contractions, and no semicolons in running prose.

---

## 8. Open items the owner must resolve

1. **Q16 needs an amendment.** The ruling says the site is a projection with every number generated from the evidence register, and that a hand-written number is a finding. The split decision means the marketing pages are hand-built. That is defensible, but it must be *recorded*, with the boundary stated: which paths are hand-built, and the rule that any figure appearing on them is machine-inserted or linked to the run that produced it. Otherwise the front page quietly violates principle 8, which is the first check a skeptical reader runs.
2. **Pricing for the audit.** Not specified anywhere. Needed before `/audit` can convert.
3. **A contact channel that exists.** The license evaluation notes that there is no public channel through which an adopter can report why they stopped. A services site needs one anyway.
4. **No changelog, and no community.** Q16 marks both partial. A platform lead checks release cadence before adopting.
5. **The site ships when the engine ships** — Q16's derived trigger. M1–M5 are done and the tutorial runs, so that condition is met. The remaining question is whether M6 (distribution) has to close first, because a visitor who cannot install the thing has trialability in name only.

---

## Sources

- [LeanCTX](https://leanctx.com/) · [lean-ctx on GitHub](https://github.com/yvgude/lean-ctx) · [LeanCTX docs](https://leanctx.com/docs/)
- [TrustGraph](https://trustgraph.ai/) · [trustgraph on GitHub](https://github.com/trustgraph-ai/trustgraph) · [TrustGraph docs](https://docs.trustgraph.ai/)
- [Vale](https://vale.sh/) · [Vale docs](https://docs.vale.sh/) · [vale on GitHub](https://github.com/vale-cli/vale)
- Internal: `docs/decisions/0016-public-presence.md`, `docs/evaluations/first-contact.md`, `docs/evaluations/adjacent-work.md` §I and §J, `docs/spec/00-vision-and-scope.md`, `README.md`
- Measured: `headwater check` against this repository, 2026-08-23, taxonomy `headwater/standard 3.3.0`
