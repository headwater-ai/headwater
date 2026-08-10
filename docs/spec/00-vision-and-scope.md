# 0 — Vision and scope

## The thesis

A documentation corpus is a **structured artifact with invariants**, not a pile of prose. When you declare that structure in a form that a machine can read, four things become possible that were not possible before:

- You can **validate** the corpus — not for spelling, but for truthfulness signals, completeness, lineage, and internal consistency.
- You can **navigate** it deterministically. A task, a code path, or a concept resolves to the documents that govern it, without search or luck.
- You can **project** it. The system generates indexes, site navigation, agent instruction files, and reader-facing summaries. No person maintains them by hand.
- The corpus can **hold itself accountable**. The system records which mechanism discharges which obligation, and where it currently has none.

Three qualities are the aim throughout. **Robustness** — the guarantees hold under change, partial adoption, and neglect. **Coherence** — the documents add up to one account of the system, not a set of individually well-formed fragments. **Self-maintenance** — the corpus detects its own drift, then repairs or reports it, and does not wait for a reader to notice.

Everything in this specification follows from that premise.

## Who this is for

**Primary:** an engineering organization that wants documentation that people and tools rely on. Humans use it at review time, and coding agents use it at work time. The organization accepts structure in exchange for trust.

**Secondary:** a single team or a solo maintainer who wants a strong default documentation system, but does not want to invent one. The tool must not dictate a taxonomy that does not match how they think.

**Explicitly served:** organizations whose documentation culture does **not** match ours. The taxonomy is theirs to define. The engine is ours.

## What we build

1. **A taxonomy schema language** — a declarative, versioned, and validated description of the structure of a corpus. That structure includes shelves, document kinds, metadata facets, relations, voice and lifecycle regimes, identifier schemes, and overlays.

2. **An engine** that reads a corpus and its taxonomy, builds a typed graph, and evaluates constraints over that graph. It parses once and checks many times. Its output is machine-readable and human-readable. It is advisory by default and blocking on request.

3. **A projection layer** that generates every derived artifact from that graph: shelf indexes, decision-lineage views, traceability matrices, site navigation, and the AI instruction surface.

4. **An AI integration surface** for the four moments when an assistant touches documentation: planning, reading, writing, and reviewing. A probe harness measures whether the surface works. The authoring half — scaffolding, the information-architecture and authoring skills, and the harness hooks that invoke them — ships with the first release, not after it. Three facts make this necessary. Everything distinctive runs on the edges of the graph. The capture-cost thesis ([spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric)) is not testable without this machinery. And a validator that ships before the machinery would measure a corpus that nothing helps to maintain.

5. **A distribution model** that lets one organization publish a taxonomy and the doctrine that explains it. Many repositories can then consume that taxonomy. They can customize it by overlay, upgrade it deliberately, and prove that they are still conformant.

6. **A doctrine starter kit** — an opinionated default taxonomy and the prose that explains it. It is not the base package, and the two answer opposite requirements. The base is minimal and derived from the core. The starter kit is a named bundle selection over that base, which a new adopter can take fully, take partially, or ignore ([spec 7](07-distribution-and-federation.md#the-starter-kit-is-a-selection)).

## What we do not build

| Not building | Why | What to use instead |
|---|---|---|
| A documentation renderer | Static-site generators already solve this | Emit navigation config for MkDocs / Docusaurus / Astro |
| A wiki or an editor | Documents are files in the repository, next to the code they describe | Any editor; the corpus is Markdown |
| A code-analysis tool | We check documents and their declared links to code, never the code's meaning | Language-specific tooling |
| A prose style checker | Voice and structure are in scope. Grammar and readability are not | Vale, textlint — composable alongside |
| A ticketing or workflow system | We refer to work items. We do not manage them | Whatever tracker is in use |
| An LLM product | The engine is deterministic. LLMs are consumers of the corpus, and one optional authoring surface | — |
| A general knowledge base | The corpus documents a system, for people who change that system | — |

## Design principles

These are the tie-breakers when a design decision is genuinely contested.

1. **Configuration over code.** Anything that an adopter might reasonably want different belongs in the schema. If a change to the taxonomy requires a change to the engine, the design is wrong.

2. **One source of truth per fact, many paths to it.** You declare a fact once — including a fact *about the structure of the corpus*. Prose that restates the schema is generated from it or validated against it. No one maintains that prose in parallel.

3. **Derived artifacts are always regenerable.** If a person can change a generated file by hand, and the system does not see the change, that file will drift. Readers will then trust a file that is wrong. Thus, the system checks that generated files agree with their sources. It does not only offer to regenerate them.

4. **Visibility before blocking.** A new rule ships as advisory. It collects evidence about its false-positive rate. Only that evidence promotes the rule to blocking. The promotion criteria are recorded, not improvised.

5. **Explicit incompleteness.** The system records what it does not check. You can triage a gap that the system registers. A gap that stays invisible becomes worse over time.

6. **Every rule earns its place.** Context is finite for humans and metered for agents. We remove a rule that has no enforcement, no observed violation, and no stated cost of failure. We do not tolerate it.

7. **Fail open at the edges, closed at the core.** When agent-facing helpers cannot answer, they degrade silently, because a missing hint is better than a wrong one. Corpus validation never degrades silently.

8. **The system governs itself.** The documentation of Headwater is itself a Headwater corpus, and Headwater validates it in its own CI. If a change is painful to dogfood, its design is not complete.

9. **Better together.** Headwater does not stand alone. Other projects solve adjacent problems, and some solve them well. When a complementary project exists, we build an integration with it before we build a replacement for it. Shared formats and emitted configurations let the corpus serve tools that we do not own. [Spec 11](11-adjacent-work.md) records the current candidates.

10. **Tested against theory and practice.** A structural design decision is not settled on intuition alone. We check it against the research literature, and against at least one observed industry application. We record the confrontation: what the sources confirm, what they sharpen, what they contradict. [Spec 10](10-theoretical-foundations.md) and [spec 11](11-adjacent-work.md) are that record for the current design. A future decision of the same weight owes the same test before it is fixed.

11. **Efficacy is measured, not inherited.** Principle 10 settles whether a design is sound. It never settles whether the design works, and the two are separate claims. Adjacent projects converge on this substrate, and almost none of them measured it ([spec 11 §M](11-adjacent-work.md#m--what-the-survey-shows-as-a-whole-convergence-is-not-evidence)). Their agreement is a shared prior rather than a body of results, so we inherit no efficacy from it. A claim that the corpus changes what a reader or an agent does rests on a counterfactual run. That run compares corpus present against corpus absent, with a pinned model and a recorded probe selection. The grader is never the system under test, because the literature already shows what that instrument returns. Until such a run exists, we publish the claim as unmeasured. Principle 5 governs the gaps in what we check. This principle governs the gaps in what we have shown.

## Success criteria

The system works when:

- The schema determines the correct location, template, metadata, and required links of a new document. No one has to ask.
- An organization with a different taxonomy adopts the system when it writes a schema. It does not patch code.
- An agent that receives a task retrieves the governing documents before it reads source code. It measurably obeys them more often than an agent without the corpus.
- A taxonomy change propagates to consumers as a reviewable, verifiable migration, not as a broadcast request.
- Every stated invariant names the control that discharges it, or appears in the gap register. There is no third category.

## Non-negotiables

- **Markdown with YAML front matter** is the document format. There is no proprietary store.
- **The corpus is valid without the tool.** Everything degrades to readable Markdown in a browser or a text editor.
- **No network dependency at check time.** Validation runs offline, in a container or on a laptop, with the same result as CI.
- **Deterministic core.** For the same corpus and schema, the output of the engine is byte-identical. LLMs only help with authoring and supply the efficacy probes. A verdict never depends on an LLM.
