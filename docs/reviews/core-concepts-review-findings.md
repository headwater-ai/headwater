# Core-concepts review: findings

Instrument: [core-concepts-review-prompt.md](core-concepts-review-prompt.md), run 2026-08-04 against the specification at commit `6057872`. Single sequential reviewer (Claude Fable 5); no fan-out. Everything below is anchored to a file and, where it matters, a section. Claims marked *(inferred)* rest on absence of text or on reasoning about behaviour no corpus yet exists to measure; everything else is read directly from the specs.

## Verdict table

| Concept | Verdict | Reason |
|---|---|---|
| corpus | keep | The coverage denominator (OB-COV-1, spec 4) and the federation unit; nothing else fixes "every file accounted for". |
| document | keep | The node type everything else types. |
| shelf | keep | Heterogeneous shelves and unclassifiable-file findings make placement do work kind cannot — but `group_by` and `layout` are mini-language creep worth watching. |
| kind | keep | The type system; carries purpose, contracts, regimes. Deleting it deletes the product. |
| external anchor | keep | Without it `governs` has no target and write-time impact detection (spec 5) has no trigger. |
| relation | define edges | Healthy and load-bearing, but acyclicity, multi-nucleus inheritance, and anchor-endpoint semantics are all undefined (argument 7). |
| relation family | keep | Should carry *more*: it is the one classifier an author must choose, and it can supply the rest (argument 1). |
| nuclearity | merge into family | Family defaults determine it in every example spec 2 gives; keep it engine-side, remove it from the authoring surface except as an override. |
| dominance | merge into family | Spec 2's own revision note says families carry dominance; the body then declares it per relation. On every nucleus–satellite relation it equals the nucleus; it carries information only as a multinuclear tiebreak. |
| authority rank | cut | Fires only on a judgement spec 1's ABox boundary says the system cannot detect; the declared edge already embodies the resolution; a scalar rank cannot express the scoped semantics the prose demands (argument 2). |
| `created_by` | keep | Cheap, closed-set, measurable; the traceability lesson made operational. |
| sequence expectation | merge into relations | Every declared sequence is one windowed hop — a state-conditional participation constraint — and as specified its window start is uncomputable (argument 3). |
| facet | define edges | Keep scalar/enum/date. Reference-valued facets are untyped edges that dodge every relation check (argument 5). |
| vocabulary | merge into facets | A reusable enum is authoring sugar; inlining values loses only a single edit point for shared sets, restorable later as a `$ref`. |
| purpose | keep | Deleting it reverts routing to lexical ranking and the core to lexical identity, which destroys the rename-freedom that makes conformance meaningful (spec 7). |
| voice regime | keep | A genuinely reusable named rule bundle with a real check behind it. |
| lifecycle regime | keep | The state machine earns its place, but transition enforcement needs an input — the prior state — that the check scope model cannot supply (argument 6). |
| freshness regime | merge into facets | Spec 2's own YAML already puts `stale_after_days` on the facet. One parameter is not a regime. |
| size regime | merge into kinds/projections | A token budget is a number on the thing budgeted, not a regime. |
| obligation | keep | The gap/unverifiable dispositions are the product; nothing else forces explicit incompleteness. |
| control | keep | Distinct from checks because it covers non-engine mechanisms, and the severity/posture split (spec 12) depends on it. |
| register | merge into obligation + control | Spec 4 puts the disposition on the obligation itself; the register is a generated projection of the other two, not a third concept. |
| projection | keep | `generate --check` is the drift-killer; no substitute exists. |
| identifier scheme | keep | Resolvable-without-document is a real capability (commit messages, agent prompts) nothing else provides. |
| core | keep | Without it conformance regresses to byte-identity — the named failure mode in spec 7. |
| overlay | keep | The confluence rules are the best-defined edge behaviour in the whole specification. |
| profile | merge into overlays | A profile is a publisher-named removal overlay; the resolver already implements every part of it. |
| compatibility declaration | cut | The five dimensions are engine constants; a declaration every taxonomy states identically declares nothing. Keep the measurement, delete the schema key. |
| mapping | keep, defer | Real capability (cross-taxonomy aggregation) whose only consumer is an aggregator that does not exist yet. |
| contract sidecar | keep, defer | Spec-as-oracle is real, but nothing else references sidecars; severable from the first release at zero cost to the rest. |

## Arguments, by consequence

### 1. The relation type system should collapse onto family

Spec 2's revision note says the applied change was "purpose as a first-class declaration with **dominance carried by relation families**", and spec 10 (§G, row 4) records the same: "relation families carry dominance". The body of spec 2 then does something different: `dominance: source | target | none` is declared per relation type, and the family table (six rows) has columns for default nuclearity and lifecycle-sensitivity but **no dominance column at all**. The spec disagrees with its own revision note. (Confidence: high — this is textual.)

The per-relation declaration is also redundant in the nucleus–satellite half of the space. A satellite "supports the other end and cannot stand without it" (spec 1); a document that cannot stand alone cannot have its purpose govern the reading of the document it depends on. So for every nucleus–satellite relation, dominance = nucleus. Spec 2's examples comply: `derives_from` (nucleus: the standard; the standard is what you read) and composition sidecars behave the same way. Dominance carries independent information **only for multinuclear relations** — `supersedes` (both nuclei, successor dominates) and `conflicts_with` (both nuclei, neither dominates). Both of those are family-typical: succession's whole meaning is that the successor governs now; association's whole meaning is "no stronger claim". (Confidence: medium-high — the entailment argument is conceptual; no corpus exists to find a counterexample in. *Inferred.*)

Nuclearity is in nearly the same position: the family table supplies defaults for succession, derivation, composition, and association, and no example in spec 2 ever overrides one. The blank cells (governance, evidence) are the relations whose targets are typically external anchors, where nuclearity is meaningless anyway (argument 7). (Confidence: medium — *inferred* from the absence of any overriding example.)

Consequence: make family the only relation classifier an author declares. Family supplies nuclearity, dominance, and lifecycle-sensitivity; the meta-schema already contains the enforcement hook ("a family's default is not contradicted without explicit override"). The engine keeps all three properties internally — nothing in lifecycle inheritance, context pruning, orphan detection, or routing order changes. What changes is the glossary: an author writing a relation type learns *one* concept (family + endpoints), not four. This is the single largest simplification available (see closing item 1), and it is the design spec 10 says was already decided.

### 2. Authority ranks should be cut

Spec 2's authority section ("Authority: when documents disagree") is machinery for an event the system cannot observe. Spec 1 draws the ABox boundary explicitly: the system knows what a document governs, "not what it asserts about the world", and "reasoning stops at the document boundary". Detecting that a standard and a specification disagree **on a fact** is exactly the reasoning spec 1 forswears, and spec 4 confirms it: contradiction detection "is undecidable structurally — it requires reading both and judging".

So when does `on_disagreement: prefer_higher_authority + flag` fire? Only when a disagreement has been *declared* — an edge in front matter, or an LLM-sweep finding a human then adjudicates. In both cases the judgement has already been made by someone who knows which side is right, and spec 4's own design rule says the right move is to record *that judgement* as data. A per-kind integer that pre-answers a question nobody has asked yet is the opposite of that rule. (Confidence: high.)

The mechanism as specified also cannot express its own prose. The declaration is a global scalar per kind (`standard: 10, specification: 20`); the prose demands "a specification outranks a standard **about its own component** and is silent about anything else". No scoping input exists anywhere in the declaration — the rank cannot know what "its own component" is. As written, the scalar produces exactly the global ordering the spec calls "backwards". (Confidence: high — textual.)

Cut the ranks. What remains — "cite both, flag the conflict" as the agent instruction — survives unchanged and needs no numbers. Spec 2 treats authority as settled; it should be reopened, and the reopened question is Q-shaped: it belongs next to Q15's provenance questions, not in the taxonomy model.

### 3. Sequence expectations cannot be evaluated as declared

A sequence fires when a document "in a given state" fails to acquire a relation "within a window" — `within: 90d`. Ninety days **from what?** No spec records a state-entry timestamp. Spec 3's front matter carries `status`, `last_verified`, and `summary`; entering a state "may require facets", but no date facet is named. The `incident-learned-from` sequence has no state condition at all, so its window can only run from document creation — also recorded nowhere. Spec 12 makes checks pure: the clock arrives as `ctx.now`, but there is no corresponding start-of-window input, and git history (the obvious source) is not in the scope model and would break the vendoring and squash-merge cases anyway. The flagship absence-detection mechanism — "the only construct that finds a document which should exist and does not" — is uncomputable from declared data. (Confidence: high on the gap itself; I searched specs 2, 3, 4, and 12 for a window-start source and found none. *Inferred from absence.*)

This needs defining, not inventing: either the state facet's value records when it was entered, or the graph builder injects a dated census the way it injects the clock. Both are "define the input", and the choice changes the caching story, so it should be made deliberately.

Separately, the concept should merge into relations. The prose sells sequences as chains ("proposal → decision → specification → evidence"), but the declaration is a single hop: *kind + state ⇒ expected relation to kind, within window*. A chain is three declarations that happen to share endpoints. A single hop is precisely a state-conditional, windowed, detective-posture participation constraint — the `required` end of the cardinality spectrum spec 1 already gives relations, plus a window and a rationale. Nothing is lost by declaring it on the relation (or on the kind's `relations:` block, where `may:` already lives); the separate top-level declaration and the separate glossary entry go. The three honesty constraints (detective-only, windowed, rationale-required) attach to the parameters unchanged. (Confidence: high on structural equivalence; the one real difference — cardinality checks are instantaneous, expectations are windowed — is a parameter, not a concept.)

### 4. The twelve declarations are fourteen, and should be ten

Spec 2's table claims twelve and the next sentence adds two more ("Plus `projections` … and `profiles`"). Counted honestly: fourteen. Which are one declaration wearing two hats:

- **`compatibility` declares nothing.** The five dimensions are the engine's measurement framework; spec 2 and spec 7 both treat the set as fixed, every taxonomy would declare the same list, and no example anywhere varies it. A declaration with one legal value is an engine constant. Cut the key, keep the measurement. (Confidence: high, unless per-taxonomy dimension opt-out is intended — no spec text suggests it. *Inferred.*)
- **`profiles` are overlays.** A profile "names the subset a repository archetype owns" (spec 7) — that is a publisher-authored, named overlay consisting of `remove` operations, and the overlay resolver already implements the hard part (dependent-key deletion, core satisfaction after resolution). Two mechanisms exist for "this repository does not carry that shelf"; one is a strict subset of the other. (Confidence: high.)
- **`sequences`** merge into relations (argument 3).
- **`vocabularies`** are authoring sugar over enum facets. Deleting the concept costs one thing: a shared value set edited in one place. That is a `$ref`, not a first-class concept a newcomer must learn before reading a taxonomy. (Confidence: medium-high.)

That leaves ten: purposes, facets, regimes, relations, shelves, kinds, identifier_schemes, core, mappings, projections. The genuinely orthogonal survive; each answers a question no other declaration can.

### 5. Reference-valued facets are a second, ungoverned edge mechanism

Spec 1 gives facets a value space including "reference to another node". A reference-valued facet asserts exactly what a relation asserts — this document is connected to that node — but carries no family, no nuclearity, no reciprocity, no lifecycle interaction, no `created_by`, and generates none of the graph checks. The same fact gets two different levels of governance depending on which syntax the taxonomy author happened to choose, and the ungoverned syntax is the cheaper one, so under deadline pressure it wins. This is the drift the collapse test exists to find: the facet/relation distinction changes outcomes *by accident of declaration*, not by design. Define it away: a reference-valued facet either compiles to an association-family relation internally, or the value type is removed. (Confidence: high on the mechanism duplication; textual.)

### 6. Windowed and transition rules need an input the check model forbids

Spec 3: "transitions not in the declared machine are rejected". Rejected by what, reading what? A pure check over the current graph (spec 12) sees one state value; an illegal transition is only visible against the *prior* value, which lives in git history or in a hook's working memory — neither is in the scope model (`Document | Edge | Neighbourhood | Shelf | Corpus`, plus `needs_body` and `needs_clock`). If transition enforcement is hook-only, CI cannot verify it and the guarantee is soft; if CI verifies it, the scope model is incomplete and every cache key derived from it is silently wrong for those checks. The same hole feeds argument 3's window-start problem — both are "time-dependent rule, no declared temporal input". Spec 12 got `needs_clock` right and stopped one input short. This is a define-edges finding on the check layer, and it is cheap now and a cache-corruption bug later. (Confidence: high. *Inferred* only in the sense that no spec says where prior state comes from.)

### 7. Relation limits are undefined where real corpora will hit them

The instrument asked for edge behaviour per concept; the coverage census (specs 4 and 12) and overlay confluence (spec 2) handle empty, missing, unclassifiable, and conflicting-overlay cases well — credit where due. The relation machinery does not get the same treatment:

- **Cycles.** Nothing forbids `A supersedes B supersedes A`, a `derives_from` loop, or a `comprises` cycle. For succession the result is a corpus with no live end; for derivation it makes satellite facet inheritance non-terminating or order-dependent; for composition it is nonsense. Succession, derivation, and composition need declared acyclicity; association legitimately does not.
- **Self-reference.** `supersedes: self` is expressible and undefined.
- **Multiple nuclei.** A satellite with two nuclei (`derives_from` two standards, one `current`, one `superseded`) inherits contradictory `status` values. Undefined.
- **Inherited versus declared.** A satellite that declares `status: current` while inheriting `superseded` — which wins, and is the disagreement a finding? Undefined.
- **Anchor endpoints.** Nuclearity, dominance, and lifecycle interaction are all defined over documents; `governs` points at anchors, which have no lifecycle and no purpose. The family table's blank nuclearity cells for governance and evidence are this problem showing through, unacknowledged.
- **Mutual succession.** `A supersedes B` and `B supersedes A` with reciprocity satisfied in both directions passes every declared check. *(Inferred: no check in specs 2, 4, or 12 catches it.)*

Every one of these becomes an implementation accident that later becomes the spec — the exact failure mode the review prompt names. (Confidence: high that they are undefined; textual absence.)

### 8. The default taxonomy is tuned to the wrong adopter

Spec 2 ships all twelve Kruchten decision relations enabled by default and says "a small team's overlay will typically remove most of them". That is backwards. Defaults are the learnability surface — spec 0's secondary audience is "a single team or solo maintainer who wants a strong default" — and the design's own sprawl argument ("readers apply them inconsistently once the list passes about a dozen entries") applies to the default it ships. The worked example's small-team column uses exactly one relation. Enable a minimal subset — `supersedes`, `conflicts_with`, `constrains`, `traces_to` keeps the live-conflict and void-constraint checks, at the cost of the `forbids`/`enables` and `does_not_comply_with` checks, which belong to the regulated column anyway — and let the regulated platform's overlay *add*. Removal-by-overlay as the common path also means the common adopter's first taxonomy experience is writing `remove:` lines — friction spent deleting things they never asked for. (Confidence: medium-high; the adoption claim is *inferred*, the internal tension is textual.)

### 9. The adversarial author: four gaps beyond link-padding

Spec 4 catches link-padding; spec 3 catches date-bumping (staleness is detective and drift-weighted); the escape-hatch concentration reporting is genuinely good. What is left open:

- **Cheapest-kind selection.** On a heterogeneous shelf the discriminator facet is self-asserted, and kind determines *every* obligation downstream — sections, facets, voice, relations, sequences. An author who types `doc_type: reference` instead of `standard` buys out of the standard's whole contract, and nothing cross-checks function against claimed genre (that is the ABox boundary again). The mitigation is not detection — it is making sure kind contracts on one shelf do not diverge so far that arbitrage pays. Worth naming in spec 2; currently unnamed. (Confidence: high that it is ungoverned; the incentive claim is *inferred*.)
- **Draft parking.** Sequences fire on `status: current`; obligations attach to live documents. A document parked in `draft` indefinitely evades both, and no dwell-time signal exists for non-terminal states. The lifecycle regime defines transitions but nothing about failing to make them — the same absence class sequences were invented to catch, one level down. (Confidence: high; textual absence.)
- **Weak-family edge choice.** The author chooses which declared relation to use. `cites` (evidence, not lifecycle-sensitive) instead of `implements` dodges lifecycle sensitivity and any sequence keyed to `implements` — and the sequence finding that results is advisory and lands 90 days later. Partial mitigation exists (`taxonomy audit` by `created_by`); per-family usage drift is not reported. (Confidence: medium; *inferred*.)
- **Suppression expiry is optional; waiver expiry is mandatory.** Spec 4: suppressions "may carry an expiry". Spec 7: a waiver "names the rule, the reason, the owner, and an expiry". The local mechanism — the one an individual author reaches for at a red check — is the leakier of the two. The asymmetry looks like an accident, not a decision. (Confidence: high; textual.)

### 10. Scale and mid-flight states

- **Mappings grow pairwise.** Spec 2's mappings are declared per taxonomy-pair and pinned to versions. At 5 taxonomies that is manageable; at 50 in a federation it is up to ~1,225 directed, versioned artefacts that go stale on every publisher release. The hub answer (the aggregating tier declares them) is hinted in "declared by whoever needs the correspondence — usually the aggregating tier" and should be stated as the *only* sane topology at scale. (Confidence: high on the arithmetic; the practical threshold is *inferred*.)
- **The between-majors state is undefined.** Spec 7's upgrade emits mechanical steps plus a judgment-bearing task list; until humans finish the list, the corpus fails the new schema. No mechanism marks "known-failing, migration task open" — waivers are per-rule, not per-migration, and suppressions are per-file. Q12 already names incremental adoption as a design constraint, so this is not a discovery — but spec 7 is written as if upgrade were atomic, which mis-frames what Q12 has already conceded: the schema must tolerate a corpus that is legitimately half-migrated, and the tolerance mechanism is unspecified. (Confidence: high.)
- **First contact is a findings flood.** Running the engine on an existing 500-doc corpus produces unclassifiable files, missing facets, and failed sequences at volume; everything is advisory, so nothing blocks — but spec 4's own suppression logic ("a rule with fifty suppressions is not a rule") applies to advisory noise too: a category that opens with 300 findings is a category that gets ignored wholesale, which then defeats the promotion pipeline that needs clean false-positive data. The `--since` mode (Q12) is the mitigation and is correctly ranked first-release. (Confidence: medium-high; *inferred* from stated behaviour.)
- Everything else scales fine on paper: corpus-scoped checks are O(n) statistics with a visible barrier count (spec 12), reconcile-first minting is a linear scan, and the register is small by construction.

### 11. Single points of failure beyond the two named

Kind resolution and the graph projector are named (Q6). Same shape, unnamed:

- **The overlay resolver and the lock.** Every downstream verdict reads the lock. A resolver bug corrupts every check, projection, and conformance claim at once. The mitigation exists (the lock is committed and diffable) but nothing *checks* the resolver the way projections are checked by regeneration — there is no equivalent of round-trip fidelity tests (Q6 requires them for RDF, not for the resolver itself). (Confidence: high; textual absence.)
- **Scope enforcement.** Spec 12 is explicit that an unenforced scope "would silently corrupt every cache key". That makes the enforcer the correctness root for all caching and all change-scoped CI — worth listing in its own SPOF register, since a leak here produces wrong verdicts that reproduce deterministically.
- **The census enumerator.** Every coverage guarantee (OB-COV-1..3) assumes the census enumerates the corpus root correctly. A glob or symlink bug in the walker quietly shrinks the denominator — the exact silent-pass failure the census exists to prevent, one level up. Nothing checks the checker. (Confidence: medium; *inferred*.)
- **The parser's span retention.** Findings anchor to lines (spec 12); section contracts, voice checks, and prose-link extraction all trust one parse. A heading mis-parse green-lights a section contract with no finding anywhere. Fixture discipline covers checks; no parser conformance corpus is specified. (Confidence: medium; *inferred*.)
- **The scaffolder.** Edges marked `created_by: scaffold` are corpus facts nobody reviews individually; a scaffolder bug manufactures wrong edges at exactly the scale the metric celebrates. Orphan detection distinguishes generation bugs for satellites only. (Confidence: medium; *inferred*.)

### 12. Load-bearing assumptions: mostly instrumented, one pipeline missing

The design is unusually honest here: agent-assisted capture cost, summary scent, and probe efficacy are all named as falsifiable and given metrics (specs 3, 5, 10 §What the theory did not settle). One assumption in the promotion machinery is not: **false-positive rates require someone to label findings false**, and no spec says who does that, where the label lives, or how it is distinguished from "true but suppressed". Suppressions-with-reasons are the nearest proxy and conflate the two. Promotion (spec 4) and demotion both hang off a number with no defined collection mechanism. Define the pipeline or the promotion criteria are unfalsifiable in practice. (Confidence: high; textual absence.)

### 13. Learnability gradient

- **(a) File a conforming document:** with `headwater new`, three concepts — kind (choose it), facets (fill `status`, `summary`, `last_verified`), sections (write them). Within the prompt's budget, *provided the scaffolder exists in the first release*; without it, add shelf, identifier, and relation declaration and the budget is blown. The scaffolder is not optional tooling; it is what makes the concept count claim true. (Confidence: high.)
- **(b) Extend a taxonomy** (add a kind to an existing one): kind, purpose, shelf, facet, regime (voice + lifecycle), relation + family (+ today: nuclearity, dominance, `created_by`, reciprocity, lifecycle interaction), sections, overlay semantics, core. Roughly twelve concepts, and the relation cluster is half the cliff — argument 1 cuts it to family + endpoints and brings (b) near eight.
- **(c) Author from scratch:** the full set, ~24 including mappings, compatibility, profiles, sidecars, authority. After the cuts and merges above, ~16. (c) is rare and can be expensive; (b) is the adoption path and is currently the steep step.

## Closing

**1. The single change that most reduces concept count without losing a capability:** make relation family the only classifier an author declares — family supplies nuclearity, dominance, and lifecycle-sensitivity as defaults, with override possible and flagged (argument 1). It removes three concepts from the authoring surface and one internal inconsistency from the spec, changes no engine behaviour, and is the design spec 10 row 4 already claims was applied. Runner-up, if a second is allowed into the same release: fold `sequences` into relation participation (argument 3), which removes a top-level declaration and forces the window-start definition the mechanism needs anyway.

**2. The single thing most likely to break first in a real adoption:** author-declared relations fail to materialise. The graph is the product, and front-matter edges with `created_by: author` are its supply chain; the traceability literature the spec itself cites (spec 10 §B.5, §C.3) says exactly these links decay because the payer is not the beneficiary. When edge density stays low, everything distinctive degrades together — routing returns thin pointer sets, sequences report absence noise against documents whose authors never declared the upstream edge, lifecycle sensitivity has nothing to propagate through, and impact detection has no `governs` edges to fire on. The design's counter-measure is agent-assisted authoring raising the assisted fraction, which spec 10 names as the strongest and least-tested claim in the system. The instrumentation is in place; the bet is unhedged. If the assisted fraction does not rise in the first real deployment, the right response is prepared in advance by the design itself (`taxonomy audit` says move the edge to `scaffold`/`generator`) — but a first release should assume that outcome, not merely survive it: every relation in the *default* taxonomy should be creatable by scaffold, generator, or hook, with `created_by: author` reserved for overlay additions an adopter explicitly chooses. (Confidence: medium-high; prediction, therefore *inferred* — but from the spec's own cited evidence.)
