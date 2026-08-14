---
id: EVAL-HW-the-measurement-layer
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: The Q8 and Q20 evidence, which is what a probe costs, when it runs, and whether the promised instruments can produce their measurements.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: evaluate+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cited_by:
    - REG-HW-decisions
    - SPEC-HW-authoring-and-lifecycle
    - SPEC-HW-assurance-model
    - SPEC-HW-ai-integration
    - SPEC-HW-engine-architecture
    - SPEC-HW-theoretical-foundations
    - SPEC-HW-adjacent-work
    - SPEC-HW-check-layer
    - SPEC-HW-glossary
---

# The measurement layer — the Q8 and Q20 evaluation

This evaluation closes [Q8](../spec/09-open-questions.md#q8--probe-cost-and-cadence) and [Q20](../spec/09-open-questions.md#q20--where-scent-lives). The two entries are one question at two radii. Q20's open half asks how anything grades a cue, and the grader is a probe. Q8 decides what probes cost and when they run. A cue that nothing grades is decoration, and a probe budget that does not know what it must grade is a number with no derivation.

[Principle 11](../spec/00-vision-and-scope.md#design-principles) makes this group the place where the whole specification is cashed out. Every earlier evaluation in this sweep deferred a claim to an instrument. This one audits those deferrals first, because a ruling about cost is worthless if the instrument cannot produce what the corpus has charged to it.

## The audit: what the specification has promised to measure

Twenty claims across the specification and the evaluations name an instrument. The table records each one, the class of instrument it names, and whether that instrument can produce the measurement as written.

| Claim | Named instrument | Class | Can it produce the measurement? |
|---|---|---|---|
| An agent with the corpus obeys governing documents more often than one without it ([spec 0](../spec/00-vision-and-scope.md#success-criteria)) | the probe suite, corpus present against absent | probe | Not as written. "Measurably obeys" has no expectation form, and the specification never says what a probe is |
| Routing precision ([spec 5](../spec/05-ai-integration.md#scent-is-the-thing-being-engineered)) | probe transcripts | probe | Yes, once an expectation language exists |
| Abandonment ([spec 5](../spec/05-ai-integration.md#scent-is-the-thing-being-engineered)) | probe transcripts | probe | Yes, on the same condition |
| A descriptor lets a cold agent reach a governing document ([Q14](../spec/09-open-questions.md#q14--discovery-surface)) | Discovery and Navigability, descriptor present and absent | probe | Yes, on the same condition |
| Admitting the asserted tier does not lower routing precision ([evaluation](warrant-and-adjudication.md)) | routing precision split by warrant | probe | Yes, on the same condition |
| An agent that meets the losing document first reaches the adjudication ([Q18](../spec/09-open-questions.md#q18--recording-adjudicated-disagreements)) | which document the transcript cites | probe | Only when the probe task produces an artifact that cites. [Spec 5](../spec/05-ai-integration.md#generated-artifacts-cite-what-licensed-them) already requires the citation, so the fix is to say so |
| A visible warrant stops a reader treating asserted content as vouched ([Q15](../spec/09-open-questions.md#q15--a-synthesized-content-tier)) | a probe "graded on whether the transcript reports the warrant" | probe | **No.** The transcript holds tool calls. The agent's report of the warrant is prose, and the grader may not read prose |
| A `counted` tombstone stops an agent reporting absence with confidence ([Q17](../spec/09-open-questions.md#q17--governed-access-and-the-solution-layer)) | a probe graded on whether the transcript reports the withholding | probe | **No**, for the same reason, and this claim is the whole argument for the tombstone |
| A derived rule teaches the same thing as its source ([spec 5](../spec/05-ai-integration.md#measuring-whether-any-of-this-works), Fidelity) | a probe category | probe | **Not a probe.** A rule file is a generated projection, and `generate --check` proves the equality with no model |
| Distinctiveness, non-restatement, and the length band ([spec 5](../spec/05-ai-integration.md#scent-is-the-thing-being-engineered)) | advisory checks | check | Declared, unbuilt. No engine exists |
| The assisted fraction rises under agent authoring ([spec 3](../spec/03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric)) | scaffolder instrumentation, and a store that trends it | engine metric | **Built.** `headwater new` writes a reading per run to [the store](../spec/03-authoring-and-lifecycle.md#what-the-capture-cost-store-records-and-what-it-refuses-to) and `headwater capture` reads it back. What it holds is one arm: nothing here has ever authored without the scaffolder except the first typing pass |
| Prose-link promotion raises author-attributable edges ([Q4](../spec/09-open-questions.md#q4--relation-storage)) | the assisted fraction, plus edge counts by creator | engine metric | Half built, and the half that runs reads zero on the term the claim is about. Every reading in this corpus carries `edge halves 0 of 0`, because no run of the verb has named a relation. `taxonomy audit` is the other half |
| Asserted content is promoted rather than accumulated ([Q15](../spec/09-open-questions.md#q15--a-synthesized-content-tier)) | promotion rate per quarter in `taxonomy audit` | engine metric | Half built. The denominator runs and reads 6 of 186 here on 2026-08-15, and no component records a promotion. "Per quarter" also needs a stored history, which is Q8's storage half |
| Imported edges do not decay faster than scaffolded ones ([Q19](../spec/09-open-questions.md#q19--inbound-integration-an-external-system-of-record)) | staleness by `created_by` in `taxonomy audit` | engine metric | Declared, unbuilt, and observational rather than controlled |
| Emitted JSON Schema lowers invalid front matter ([Q13](../spec/09-open-questions.md#q13--linkml-and-shacl-as-substrate)) | the coverage report, before and after | engine metric | Declared, unbuilt. Before-and-after across a release is confounded |
| The filter emits exactly the declared set ([Q17](../spec/09-open-questions.md#q17--governed-access-and-the-solution-layer)) | the projection census plus a differential fixture set | fixture set | Declared, unbuilt |
| Harvest keeps a solution-tier route query under 100 ms ([Q9](../spec/09-open-questions.md#q9--multi-repository-corpora)) | route latency at the tier | benchmark | Declared. Needs a real tier |
| Publishing the read set lets a gate skip a re-run ([Q21](../spec/09-open-questions.md#q21--terminological-succession-and-validity-under-merge)) | the fraction of merges whose read set the other side never touched | history measurement | Needs the engine. The exposure prior is measured |
| Declarative voice is detectable at useful precision ([Q5](../spec/09-open-questions.md#q5--voice-checking-depth)) | an adjudicated sample of 50 findings per category | human adjudication | Specified in full, and unrun |
| Working-tree write tools raise the assisted fraction ([Q7](../spec/09-open-questions.md#q7--scope-of-the-mcp-surface)) | the assisted fraction with the tools on and off | engine metric, paired | **No design exists for the pair.** One team cannot author the same corpus twice, and the arms contaminate each other. The metric now has a store, and the store carries no term that names which arm produced a reading. [OBL-repo-0004](../obligations/0004-working-tree-write-tools-have-no-measured-effect.md) records that the issue registering a write tool owes one |
| Transition continuity measures coherence ([spec 10 §G](../spec/10-theoretical-foundations.md#what-the-theory-did-not-settle)) | the distribution across healthy and unhealthy corpora | comparison | **The comparison set does not exist**, and nothing in the specification defines what labels a corpus healthy |

### What the audit shows

**Six claims are charged to the probe layer, and three of the six are misfiled.** Two ask the grader to read the agent's prose, which the grader constraint forbids. One is not a probe at all. The other three are producible, and only after this evaluation supplies an expectation language.

**The larger finding is that the probe layer has no design.** [Spec 5](../spec/05-ai-integration.md#measuring-whether-any-of-this-works) names six categories, two constraints, and a cost envelope. It never says what a probe **is**. There is no declaration for one, no authored form, no owner, and no definition of the "declared expectation" that the grader compares against. So "the grader is never the system under test" is a rule with nothing to bind, and Q8 asks the cadence of a machine that the specification has not built. That is the finding, and it outranks the cadence question.

**Fourteen of the twenty claims are not the probe layer's problem at all.** They belong to `taxonomy audit`, the coverage report, fixture sets, or human adjudication. Every one of them is equally unbuilt, so the probe layer is not the bottleneck that the sweep's language implies. What the probe layer owns is the claim that no other instrument can reach: that the corpus changes what an agent does.

**Two promises name instruments that cannot exist as written.** [Q7](../spec/09-open-questions.md#q7--scope-of-the-mcp-surface) wants an A/B over authoring behavior, where the unit is a team over months and the arms cannot be isolated. [Spec 10](../spec/10-theoretical-foundations.md#what-the-theory-did-not-settle) wants a comparison across healthy and unhealthy corpora, and nothing defines the labels. Both are recorded here rather than repaired, because each belongs to the entry that made it.

## What the specification already fixed

Nine rulings bind this group, and neither entry may revisit them.

**No LLM in the validation path, and no network at check time.** [Spec 0](../spec/00-vision-and-scope.md#non-negotiables) and [spec 5](../spec/05-ai-integration.md#what-we-do-not-do) put both beyond argument. A probe calls a model over a network, so a probe is never a check and never gates.

**The grader is never the system under test, and a published claim carries its counterfactual.** [Spec 5](../spec/05-ai-integration.md#measuring-whether-any-of-this-works) and [spec 11 §M](../spec/11-adjacent-work.md#m--what-the-survey-shows-as-a-whole-convergence-is-not-evidence) settle both, against measured evidence about what a weak grader returns.

**A sampler is not a check.** [Spec 12](../spec/12-check-layer.md#where-the-llm-coherence-sweep-fits) puts the coherence sweep outside the cached reproducible path, marks its provenance `agent`, and forbids it from gating. A probe is the same class of component.

**A verdict is about one state of the corpus, and the read set says which.** [Spec 4](../spec/04-assurance-model.md#a-verdict-is-about-one-state-of-the-corpus) and [spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) settle it for check results.

**Every document carries a warrant, and the value set is closed.** [Spec 1](../spec/01-conceptual-model.md#warrant) and [spec 3](../spec/03-authoring-and-lifecycle.md#the-warrant-and-what-each-value-requires) fix the four values and what each one requires.

**A committed snapshot is how external content enters a corpus.** [Q19](../spec/09-open-questions.md#q19--inbound-integration-an-external-system-of-record) settled that exactly one resolver owns each anchor kind, and that a resolver reads repository content or a committed snapshot which carries its fetch time and its upstream identity.

**Posture comes from fixability, and a rewrite-remediated rule is permanently advisory.** [Spec 4](../spec/04-assurance-model.md#where-promotion-cannot-finish) states it as a general rule.

**A relation instance is an object with declared attributes, and each attribute declares an owning end.** [Q4](../spec/09-open-questions.md#q4--relation-storage) settled it, and made the cue a source-owned attribute. [Spec 2](../spec/02-taxonomy-model.md#instance-attributes-and-which-end-owns-each-one) carries the declaration.

**The declaration count is thirteen, and a use of an existing mechanism earns no name.** [Spec 2](../spec/02-taxonomy-model.md#the-thirteen-declarations) removed three declarations on that rule and refused two more.

## Prior art and observed applications

[Principle 10](../spec/00-vision-and-scope.md#design-principles) asks for the research literature and at least one observed industry application. The web budget for this session was spent before several sources could be re-verified, and each one is marked where it appears.

**Information foraging places scent in a choice set, and that decides Q20.** Pirolli and Card model information seeking as foraging, with scent as the proximal cue that predicts distal value. The part that Q20 needs is how the models operationalize it. The computational descendants — SNIF-ACT and the Bloodhound line — model a link decision as a utility computed over the links available on the current page, and a patch-leaving decision as that utility falling below what another patch offers. Scent is therefore never absolute. It is a comparison among the options at the point of decision. This session could not reach the SNIF-ACT paper itself: two hosts returned 403 and 404, and the general reference that did resolve confirms the per-link and comparative framing without the utility function. The claim is recorded at that strength.

The transfer is exact and it contradicts Q20's own premise. Q20 says that edge cues "have no such place" for a sibling comparison. They do. The comparison set for a document's summary is the other documents a reader is choosing between. The comparison set for a cue on an edge is **the other outbound cues of the document the reader is holding**. That is the choice set at the moment of traversal, and foraging theory says it is exactly where scent is evaluated.

**The CHI work on programmer navigation confirms the placement of the cue.** Lawrance, Bogart, Burnett and colleagues model debugging as foraging over a code base, and their PFIS family of models scores the cues a programmer can see from the current position. The cue is the word or the link in the source, not a property of the destination. This session could not fetch the paper, and the claim rests on memory. It agrees with what [spec 11 §L.3](../spec/11-adjacent-work.md#l3-where-scent-lives--the-first-substantive-disagreement) already recorded from Serena's shipped convention, which is an observed application at the scale of tens of thousands of repositories.

**Information-retrieval evaluation supplies the cost structure of a test collection, and one durable warning.** The Cranfield paradigm builds a reusable collection once — topics, documents, and relevance judgments — and amortizes it over many systems. TREC scaled that with pooling, because judging every document against every topic does not scale. Two lessons transfer. A probe suite is a test collection, so its cost is a large one-time authoring cost and a small per-run cost, which is the opposite of the recurring-spend model that Q8 assumes. And Voorhees measured what happens when the judgments themselves vary between assessors: absolute scores move, and the **relative ranking of systems is stable**. This session could not re-fetch that paper. The transfer stands on its own terms and is the reason the design measures a difference between two arms rather than a level.

**LLM-as-judge validity is measured, and it decides the grader.** Zheng and colleagues name position bias, verbosity bias, self-enhancement bias, and limited reasoning, and report over 80% agreement with human preference for a strong judge. That figure is the case *for* the instrument, and the corpus already records the case against it. [Spec 11 §M.3](../spec/11-adjacent-work.md#m3-the-grader-decides-the-answer-and-the-literature-proves-it) records a systematic comparison whose verdict reversed when the grading method changed, and an order reversal that produced opposite judgments. Read together: an LLM judge is good enough to agree with humans on average and not good enough to carry a claim that its author has an interest in. Headwater's answer is not a better judge. It is to have no judge.

**RAG evaluation practice already splits the two halves that this ruling separates.** RAGAS scores retrieval and generation on different axes: whether the retrieved context is relevant and focused, and whether the answer is faithful to it. The retrieval half is computable against a known answer set. The generation half needs a judge. The design below keeps the first half and refuses the second, which is the same split arrived at from the corpus side rather than the retrieval side.

**Online-experiment methodology supplies the sample size, and the arithmetic is the whole cost model.** A probe outcome is a Bernoulli trial, because sampling is stochastic and [spec 5](../spec/05-ai-integration.md#what-structured-knowledge-buys) says so. A single run of one scenario per arm is one draw per arm and settles nothing. The standard two-proportion sample size is stated below with worked numbers. The practical lesson from the field is Kohavi's: most real effects are small, and underpowered experiments produce confident nonsense. This session could not re-fetch the paper, and the arithmetic below does not depend on it.

**Pinned-model drift is measured, and a model name is not a pin.** Chen, Zaharia and Zou compared two snapshots of nominally identical models three months apart. Accuracy on one task fell from **84% to 51%**, and the same pair moved in opposite directions on different tasks. That is the observed application that decides how a run records its identity. A probe result that pins a model by name has not pinned anything.

**Golden sets and regression suites are the shipped industry pattern, and their shape is the cadence answer.** Every evaluation harness in current practice ships the same two tiers: a small fixed set that runs often against a recorded baseline to catch regressions, and a larger offline evaluation that runs rarely to establish a result. Canary and shadow evaluation in production machine learning is the same shape at a different scale. Nobody runs the powered comparison on every change, because it costs what it costs and because most changes do not move it.

**The observed application for the counterfactual is that almost nobody runs one.** [Spec 11 §M.2](../spec/11-adjacent-work.md#m2-what-the-field-has-actually-measured) tabulates six adjacent projects, and not one compares behavior with its structure present against the same behavior without it. The closest published measurement of a machine-facing documentation surface is the `llms.txt` result that [Q14](../spec/09-open-questions.md#q14--discovery-surface) already carries: about 137,000 domains publish one and 97% of the valid files went unread in a month. That is what an unmeasured instrument surface looks like after it ships.

## The decision — Q8

### A probe is an authored document with a declared expectation

This is the instrument that the specification never defined, and everything else follows from it.

A **probe** is a document in the corpus. It has a kind, a shelf, an identifier, and an acceptance, like anything else. It declares a category, a task statement, and an **expectation**. The expectation is a predicate over the run record, and it comes from a closed set of forms.

| Form | Satisfied when |
|---|---|
| `opened` | The transcript shows the session read one of the named documents |
| `not_opened` | It read none of them |
| `cited` | A produced artifact cites one of the named identifiers ([spec 5](../spec/05-ai-integration.md#generated-artifacts-cite-what-licensed-them)) |
| `answered` | The session's final answer is a named value from a closed set that the probe declares |
| `patched` | A produced patch passes a named check, which is the oracle route ([spec 2](../spec/02-taxonomy-model.md#contract-sidecars-the-specification-as-oracle)) |

**A question whose answer needs a rubric is not a probe.** It is a coherence question, and [spec 4](../spec/04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep) already owns those: the sweep reports findings, never verdicts, and is marked as agent-provenanced. That single rule is what turns "the grader is never the system under test" from a promise into a property. The grader contains no model, because a predicate over an event log needs none.

This is the same move that [Q7](../spec/09-open-questions.md#q7--scope-of-the-mcp-surface) made for the write path. A tool class that is off is not registered, so no handler exists to call. Here, the grader has no prose to read, so no judgment is available to make.

### Self-report and output are different things, and the specification did not separate them

A **self-report** is the agent's claim about its own process. A **produced output** is the artifact that the task asked for. [Spec 5](../spec/05-ai-integration.md#measuring-whether-any-of-this-works) refuses the first and never named the second, which is why two entries wrote probes that cannot run.

Grading an output against a declared expectation is legitimate, and it is the only way that Sufficiency is measurable at all. So the two blocked promises in the audit are restated rather than abandoned. The warrant probe asks a question whose only answer sits in an `asserted` document and requires the session to end with one value from `{answer, unvouched, not found}`. The tombstone probe asks a question whose answer a profile withheld and requires one value from `{answer, withheld, not found}`. Both are then `answered` expectations, and both are gradeable with no judge.

### The category list loses one entry and reshapes another

| Category | Asks | Change |
|---|---|---|
| Discovery | Does the session find the governing document at all? | unchanged |
| Sufficiency | After it finds the document, does it have enough to act correctly? | Needs `patched`, `cited`, or `answered`. Without one of the three it is a rubric question |
| Navigability | Can it get from a code path to the governing document, and back? | unchanged |
| Consistency | Same question, different phrasings, same answer? | "Same answer" means the same read set and the same cited identifiers. A prose equality is not available |
| ~~Fidelity~~ | Does the derived rule teach the same thing as its source? | **Removed.** It is `generate --check` |
| ~~Counterfactual~~ | Does removal of the context change behavior? | **Removed as a category.** It is an arm of every probe |

**Fidelity is not a probe.** A rule file is a generated projection of a canonical document, and [spec 5](../spec/05-ai-integration.md#read-time-rule-loading) already says that CI checks the regeneration. To pay a model to compare a projection with its source is to re-derive what the graph declares, which is the defect that [spec 4](../spec/04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep) names for the sweep. The real question that Fidelity reached for survives inside Sufficiency: a rule file carries only a teaser, so whether the teaser carries enough is a question about behavior and belongs there.

**The counterfactual is an arm, not a category.** Every probe runs in the `present` arm or the `absent` arm, and a claim published under [principle 11](../spec/00-vision-and-scope.md#design-principles) needs the pair. Listing it as a seventh category hid two things. It hid that the counterfactual applies to the other categories rather than sitting beside them, and it hid that the pair is a multiplier on cost. **The arm is a property of a run, and the corpus-absent arm is a declared ablation** — which surfaces the harness withholds — so that "corpus absent" is a recorded fact rather than an improvisation.

### A run produces a committed snapshot and a regenerated document

This is the storage half of Q8, and the mechanism landed one group earlier.

A run emits a **transcript**: the ordered tool-call events with their arguments and result identities, the identifiers of any produced artifact, the final closed-set answer, and the run identity. The run identity is the model and its served version, the corpus tree hash, the taxonomy lock hash, the probe selection hash, the rotation seed, the harness version, the arm, and the time. **The model's prose is not in the transcript.** That omission is the enforcement, in the same way that scope enforcement is the feature in [spec 12](../spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on).

A transcript is a **committed snapshot** that a `probe_run` anchor resolver reads. [Q19](../spec/09-open-questions.md#q19--inbound-integration-an-external-system-of-record) already built this: exactly one resolver owns an anchor kind, and a resolver reads repository content or a committed snapshot that carries its fetch time and its upstream identity. A probe run is an external system of record, and the specification needs nothing new to hold one. The declaration count stays at thirteen.

The **probe result** is a document, and it is a projection over the transcript, the expectations, and the grader version. Its warrant is **`regenerated`**, and `generate --check` proves it. That answers the question that this group inherited.

The alternative is worth stating, because it is what the leaning implied. Suppose the result is committed and the transcript is not. Then nothing inside the repository stands behind the result, so its warrant is `asserted`. [Spec 3](../spec/03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two) then says that an asserted document does not discharge an evidence obligation, and a probe result could never support `evidenced` on any decision record. The measurement layer would produce content that the evidence rules refuse. **So committing the transcript is not a storage preference. It is what makes a probe result usable as evidence at all**, and it is the same sentence that spec 3 already carries for an imported pointer: the pointer resolves offline against a committed snapshot.

### A probe never runs against a pull request, and a pull request still does something

**No probe runs at check time.** [Spec 0](../spec/00-vision-and-scope.md#non-negotiables) forbids the network there, [spec 5](../spec/05-ai-integration.md#what-we-do-not-do) forbids an LLM in the validation path, and a probe is a sampler rather than a check ([spec 12](../spec/12-check-layer.md#where-the-llm-coherence-sweep-fits)). Three further reasons agree. A probe verdict is not reproducible, so it is not a verdict in the sense that a gate needs. The cost is unbounded and falls on whoever opens the change, which is the payer asymmetry that [spec 3](../spec/03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) says kills a system. And a scheduled instrument that anyone can trigger on demand is an instrument whose selection nobody controls.

What a pull request does instead is **report which recorded probe results it voided**, offline and at no marginal cost. A probe run has a read set: the documents that the transcript shows the session opened, plus the corpus tree, the lock hash, the model identity, the probe selection, and the harness version. [Spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) already derives invalidated instances from a diff and a read set. A probe result is a verdict about one state of the corpus, exactly as a check result is, so the same machinery applies with no addition. The finding is advisory, it names the probe result, and it is reported against the result rather than against the change.

That is also what makes the trend readable over quarters. A trend line assembled from results whose staleness nobody tracked is a line drawn through points that are not comparable. The read set makes every point state what it is a point about.

### A rerun that disagrees is drift or variance, and only an interval separates them

[Principle 3](../spec/00-vision-and-scope.md#design-principles) asks that a derived artifact be regenerable, and this layer answers in two halves.

**The grading is deterministic and a disagreement there is a defect.** The result document is a function of the committed transcript, the expectations, and the grader version. `generate --check` proves it on every run. The grader therefore joins the [correctness roots](../spec/12-check-layer.md#the-correctness-roots): a wrong grader produces systematically green probe results, which is the silent-pass failure one level up, and no probe finds it.

**The behavior is not deterministic and a disagreement there is not a defect.** A rerun with an identical run identity can return a different rate, because sampling is stochastic. So a probe result reports an **interval and not a point**. A rerun whose interval overlaps the first is variance. A rerun whose interval does not overlap is drift, and the recorded identity says where to look: the served model moved under a fixed name, the corpus moved, or the harness moved. Without the interval the question has no answer, and every rerun looks like a defect or like nothing.

The model half of that is measured rather than feared. Two snapshots of a nominally identical model, three months apart, moved from 84% to 51% on one task. **A model name is not a pin**, so a run records the served version where the provider exposes one, and states the name as a name where it does not.

### Cadence: the axis is the purpose of the run, not the category

The entry asks which categories run on which cadence. That is the wrong axis, and it is why the question resisted an answer. Categories say what a probe asks. Cadence follows from **whether a run is watching for a change or estimating a difference**, and any category can do either.

| Tier | Purpose | Shape | Cadence |
|---|---|---|---|
| **Regression** | Detect that something moved | A fixed scenario set, one arm, against a recorded baseline | Scheduled. Weekly is a sound default |
| **Campaign** | Estimate a difference for one named claim | Both arms, powered, one batch, one model version | On the claim. When it is published, and again when a change voids it |

Two consequences follow, and both are cost rulings as much as cadence rulings.

**A campaign runs as one batch or it is not one measurement.** A campaign spread over weeks is a campaign whose model may have moved inside it, which the drift measurement above makes concrete. One batch, one model version, one selection.

**The regression tier cannot establish an effect and must not be read as if it does.** It runs one arm, so it has no counterfactual. Its output is a comparison against the previous run of itself, and a claim built on it is exactly the class that [spec 11 §M.2](../spec/11-adjacent-work.md#m2-what-the-field-has-actually-measured) tabulates.

### The envelope, derived

The cost of the layer is `sessions × cost per session`, and `sessions = scenarios × arms × repetitions`. Each factor now has a source.

**The campaign tier's session count comes from statistical power, not from taste.** A probe outcome is a proportion, so the two-proportion sample size applies. Worked, at 80% power and a 5% two-sided level:

| Effect to detect | Sessions per arm | Sessions per campaign |
|---|---|---|
| 0.50 → 0.75 | 58 | 116 |
| 0.70 → 0.90 | 62 | 124 |
| 0.60 → 0.75 | 152 | 304 |

A campaign is therefore **on the order of one hundred to three hundred sessions**, and a smaller effect costs a much larger run. That is the number the entry never had, and it reframes the question: the design decision is not the monthly spend, it is which claims are worth a campaign.

**The per-session cost is small on a corpus of this size.** `docs/spec/` holds about 86,000 words, which is roughly 110,000 tokens as a whole-corpus upper bound. A routing probe loads pointers rather than content, so it reads far less. At current first-party rates for the model tier that this project uses, a session that reads the whole corpus once and produces a short answer costs well under one United States dollar, and repeated scenarios against one corpus share a cached prefix at about a tenth of the input rate. Two levers apply and both follow from the cadence ruling rather than from thrift. Probes are not latency-sensitive, so the batch interface and its 50% discount fit the campaign tier exactly. And a campaign runs many scenarios against one corpus, so the shared prefix is cached across the run.

The order of magnitude that falls out is the finding. **A powered campaign for one claim is a line item in the low hundreds of dollars, and a weekly regression tier of a few dozen scenarios is a small monthly bill.** Money is not the constraint on this layer. The constraint is the authoring of scenarios and expectations, which is human work, and the correctness of the grader.

That does not remove the envelope, because an adopter's corpus is not this one. So the envelope is **declared per tier, and the harness fails closed**: it projects the cost of a run before it starts, and refuses to start a run that exceeds the declared budget. That is [principle 7](../spec/00-vision-and-scope.md#design-principles) read the way that [spec 6](../spec/06-engine-architecture.md#an-export-profile-carries-a-filter) reads it for an exporter. Not running is the cheaper error. And the harness reports realized cost beside the result, so the adaptive layer that reports cost ([spec 4](../spec/04-assurance-model.md#the-adaptive-layer-reports-cost-not-just-coverage)) covers its own instrument.

### The suite has its own health metric, and it is the same rule as every other rule

[Principle 6](../spec/00-vision-and-scope.md#design-principles) applies to the instrument. **A probe whose two arms never differ measures nothing about the corpus** and is a candidate for deletion, in exactly the way that [spec 10](../spec/10-theoretical-foundations.md#what-the-theory-did-not-settle) says a transition-continuity distribution stable across every corpus measures nothing. A probe suite also ages, because a corpus tuned to a fixed suite stops learning from it — the contamination that the benchmark field spends most of its effort on. So the anti-overfitting rules that [spec 5](../spec/05-ai-integration.md#anti-overfitting) already carries gain one constraint: **a paraphrase varies the task statement and never the expectation**, and the rotation seed is part of the run identity so that a selection is reproducible even though a behavior is not.

## The decision — Q20

### Grade a cue against the alternatives it competes with

Q20's standing question is how anything grades an edge cue, and it rests on a premise that foraging theory contradicts. Spec 5 grades distinctiveness by comparing siblings in one place, and Q20 concludes that edge cues have no such place. They do.

> **Grade a cue against the alternatives that it competes with at the moment it is read.**

For a document summary, the alternatives are the other documents the reader is choosing among. The shelf is the static proxy for that set, and the pointer list that routing returns is the real one — which is why the static check and the probe measure the same property at different fidelities. For a cue on an edge, the alternatives are **the other outbound cues of the document that the reader is holding**. That is the choice set on traversal, and the foraging models place the decision exactly there.

This is one rule with two instances rather than two measures, and it corrects the node side as well. Spec 5's distinctiveness measure compares shelf siblings, and a shelf is not the set that a reader is choosing among. The comparison set was never named, so it read as a property of the document rather than of the decision.

### Three static checks, and no new machinery

| Measure | Comparison set | Scope | Posture |
|---|---|---|---|
| **Distinctiveness** | the source document's other outbound cues | `Document` | advisory, permanently |
| **Non-restatement** | the target's own summary and title | `Edge` | advisory, permanently |
| **Length band** | the declared band | `Edge` | advisory, permanently |

Every scope is one that [spec 12](../spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on) already declares, and `Edge` is defined there as one relation instance and both endpoints, which is exactly what non-restatement reads. Posture is settled by the [fixability bar](../spec/12-check-layer.md#fixability) with no new argument: the remediation for a weak cue is a rewrite, so the category is permanently advisory under [spec 4](../spec/04-assurance-model.md#where-promotion-cannot-finish).

Non-restatement is the measure that earns its place first. A cue that only rephrases the target's summary carries nothing that the fallback did not already carry, so it is authoring cost with no scent gain. That is the cheapest and most direct answer to "how does anything grade it".

The checks have no instances in a corpus that declares no cues, so a corpus that declines the feature pays nothing.

### The traversal measures come from the transcripts that already exist

Routing precision and abandonment have edge analogs, and the probe layer produces them with no new instrument.

- **Traversal precision** — of the edges reachable from a document that the session opened, how often it followed one whose target sufficed.
- **Traversal abandonment** — edges offered and never followed.

An edge traversal is visible in a tool-call transcript, because it is the session opening document D and then opening T where D declares an edge to T. Both measures therefore fall out of the same transcripts that Discovery and Navigability already produce.

### The cue's counterfactual is the cheapest in the specification

Because absence falls back to the target's summary, the ablation is a switch in the serving layer rather than a second corpus. `related` and `explain` serve the cue where one exists and the target's summary otherwise, and the arm flag flips that. So the cue is the natural first campaign: one paired run, on one corpus, with no authoring cost for the absent arm.

Q20's leaning said to grade the cue only where a probe records a failed traversal. That is close and it is backwards in one respect. A failed traversal in the present arm alone says nothing, because there is no baseline to compare it against. The cue's claim needs the pair like any other.

### What the cue may do, and what it may not

The decision is the leaning, with the grading question answered and four consequences stated.

**An optional cue on a relation, with the summary still required.** Absence falls back to the target's summary, so no corpus regresses and current behavior stands. It is a source-owned instance attribute on the relation type, which [Q4](../spec/09-open-questions.md#q4--relation-storage) settled, and [spec 2](../spec/02-taxonomy-model.md#instance-attributes-and-which-end-owns-each-one) already shows it declared on `cites`.

**It is not mandatory, and the cost is now measurable rather than asserted.** A required prose field on every edge is the bookkeeping burden that [spec 3](../spec/03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) warns kills a corpus. The instrument for that already exists too: a hand-authored cue is hand entry, so a corpus where cues rise while the assisted fraction falls is a corpus paying a tax.

**Routing never returns a cue.** A routing result has no referring edge, which is the whole reason Q20 exists. Only a traversal surface serves one.

**A cue states its target's warrant.** A cue whose target carries the `asserted` [warrant](../spec/01-conceptual-model.md#warrant) states that beside the cue, which is the rule that [spec 5](../spec/05-ai-integration.md#intent-time-routing) already gives a pointer. An agent that follows a cue has to know before it reads.

**The confidence gate does not reach a cue.** The gate is a scent threshold over a score, and a cue is authored rather than scored. Silence is also not available on a traversal, because the reader already holds the document and can see the edge. So the fallback is the summary and never nothing.

**A withheld target takes its cues with it.** A cue is an edge attribute, and [spec 6](../spec/06-engine-architecture.md#an-export-profile-carries-a-filter) makes the export filter default-deny over classes, including attributes. A tombstone carries a reason from a closed set and never a cue. That is confirmed rather than new.

## What this predicts, and how to measure it

[Principle 11](../spec/00-vision-and-scope.md#design-principles) forbids an inherited claim of efficacy. Five claims here are testable, and every one is unmeasured today.

| Claim | Instrument | Status |
|---|---|---|
| A cue raises traversal precision over the summary fallback | traversal precision, cue-present against cue-absent, one paired campaign | unmeasured, and the cheapest campaign in the specification |
| Non-restatement catches cues that carry nothing the fallback carried | the check's finding rate against an adjudicated sample, on a corpus with cues | unmeasured, and no corpus declares a cue |
| A closed-set expectation makes a probe verdict reproducible under grading | `generate --check` over the result document, against a committed transcript | unmeasured, and no harness exists |
| The regression tier detects a corpus change that a campaign would confirm | a seeded regression against a recorded baseline, followed by a campaign on the same claim | unmeasured |
| A declared envelope keeps a campaign inside its projected cost | projected against realized cost, reported per run | unmeasured |

The first is the one to watch, because it is the whole argument for the cue. If a cue does not beat the summary fallback, the feature is authoring cost with no scent gain, and the honest response is to remove it rather than to make the cue longer.

## What stays open

**Whether any corpus authors enough cues to grade.** The three checks have no instances until an adopter declares a cue and authors some. This repository can supply the first, and no adopter exists.

**Whether the probe kind belongs in the base package.** A probe is a document, so it needs a kind, a shelf, and a purpose. Whether that ships in the minimal base or as a bundle is a [Q3](../spec/09-open-questions.md#q3--how-much-of-the-default-taxonomy-ships-in-the-box) question about clustering, and the answer waits for the same evidence that the bundle set waits for.

**How a probe reaches a corpus that it does not sit inside.** A probe is a document in the corpus that it measures, which is right for a single corpus and unargued for a harvesting tier. The solution-layer route query has its own claim ([Q9](../spec/09-open-questions.md#q9--multi-repository-corpora)) and no tier exists.

**Transcript retention.** A transcript is a committed snapshot, so it grows the repository like any other pin. The obvious policy is to keep every transcript of a published campaign and the last few regression runs, and nobody has measured the size of one.

**Two promises that this evaluation records and does not repair.** [Q7](../spec/09-open-questions.md#q7--scope-of-the-mcp-surface)'s authoring A/B has no runnable design, because the arms cannot be isolated inside one team. [Spec 10](../spec/10-theoretical-foundations.md#what-the-theory-did-not-settle)'s healthy-against-unhealthy corpus comparison names a set that nothing defines. Each belongs to the entry that made it.

**Whether an adopter ever runs a campaign at all.** The regression tier is cheap enough that it will run. A campaign costs a batch and a day, which is small in money and real in attention. If no adopter ever pays it, then principle 11 describes an obligation that nobody discharges, and the honest response is to say so rather than to lower the bar.

## Consequences for the specification

Eighteen changes follow, and all are applied.

| Where | Change |
|---|---|
| [Spec 5](../spec/05-ai-integration.md#measuring-whether-any-of-this-works) | A probe is an authored document with a declared expectation, and the expectation forms are a closed set |
| [Spec 5](../spec/05-ai-integration.md#measuring-whether-any-of-this-works) | A self-report and a produced output are different things, and only the second is gradeable |
| [Spec 5](../spec/05-ai-integration.md#measuring-whether-any-of-this-works) | Fidelity leaves the category list, because it is `generate --check` |
| [Spec 5](../spec/05-ai-integration.md#measuring-whether-any-of-this-works) | The counterfactual is an arm of every probe rather than a category, and the absent arm is a declared ablation |
| [Spec 5](../spec/05-ai-integration.md#measuring-whether-any-of-this-works) | The two tiers, their cadences, and the rule that a campaign runs as one batch at one model version |
| [Spec 5](../spec/05-ai-integration.md#measuring-whether-any-of-this-works) | A run emits a transcript that holds no model prose, plus the run identity, and a model name is not a pin |
| [Spec 5](../spec/05-ai-integration.md#measuring-whether-any-of-this-works) | A result carries an interval, and drift and variance are separated by it |
| [Spec 5](../spec/05-ai-integration.md#measuring-whether-any-of-this-works) | A probe never runs against a change, and a change reports the probe results that it voided |
| [Spec 5](../spec/05-ai-integration.md#measuring-whether-any-of-this-works) | The envelope is declared per tier and the harness fails closed, and it reports realized cost |
| [Spec 5](../spec/05-ai-integration.md#anti-overfitting) | A paraphrase varies the task statement and never the expectation, and a probe whose arms never differ is a deletion candidate |
| [Spec 5](../spec/05-ai-integration.md#scent-is-the-thing-being-engineered) | Scent is graded against the alternatives at the point of decision, and the measures gain a comparison set |
| [Spec 5](../spec/05-ai-integration.md#scent-is-the-thing-being-engineered) | The cue: where it is served, its fallback, the warrant statement, and the confidence gate that does not reach it |
| [Spec 3](../spec/03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed) | A probe result carries the `regenerated` warrant, and the transcript is the committed snapshot behind it |
| [Spec 4](../spec/04-assurance-model.md#the-adaptive-layer-reports-cost-not-just-coverage) | The adaptive layer reports the cost of its own instrument, and efficacy is reported with an interval |
| [Spec 4](../spec/04-assurance-model.md#a-verdict-is-about-one-state-of-the-corpus) | A probe result is a verdict about one state of the corpus, and the read set governs it |
| [Spec 6](../spec/06-engine-architecture.md#cli) | `headwater probe` takes a tier and an arm, and the run identity is what it reports |
| [Spec 10 §E.1](../spec/10-theoretical-foundations.md#e1-information-foraging--routing-has-a-theory) | Scent is a comparison over a choice set, which is what the foraging models compute |
| [Spec 12](../spec/12-check-layer.md#the-correctness-roots) | The probe grader joins the correctness roots, because a wrong grader produces systematically green results |

[Spec 11 §R](../spec/11-adjacent-work.md#r--measuring-whether-the-corpus-works) records the sources above, with what each one confirms, sharpens, or contradicts. Two entries in [spec 9](../spec/09-open-questions.md) are rewritten as closed entries, and the [glossary](../spec/glossary.md) gains **probe**, **probe result**, **probe transcript**, **arm**, **campaign** and **cue**, with **counterfactual probe**, **efficacy** and **probe suite** corrected.
