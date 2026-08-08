# 4 — Assurance model

How the system knows it is working, and admits where it does not.

## Assurance, not enforcement

Enforcement implies a gate that stops bad things. Real documentation systems need four different postures, because no single gate catches enough:

| Class | Acts | Example |
|---|---|---|
| **Preventive** | Before the mistake lands | Scaffolding, editor validation, agent behaviour, commit hooks |
| **Detective** | After it lands | CI checks, scheduled drift scans, staleness sweeps |
| **Corrective** | Repairs what was found | Auto-fix, generated remediation tasks, agent-raised change proposals |
| **Adaptive** | Retunes the mechanisms | Efficacy probes, false-positive tracking, promotion decisions |

Most systems build the first two and claim the set. The adaptive layer is the one that decides whether the other three are worth their cost, and it is specified here as a binding obligation rather than an aspiration.

## Cohesion and coherence are different obligations

Linguistics draws a distinction this model needs. **Cohesion** is the set of surface ties that bind a text — references that resolve, links that land, vocabulary used consistently. **Coherence** is the reader's experience of the whole hanging together. Cohesion is a property of the artefact; coherence is a property of the encounter.

Almost everything the engine checks is cohesion: links resolve, relations are reciprocal, identifiers bind, enums are respected, sections are present. All of it is decidable, deterministic, and blocking-eligible.

**A corpus can pass every one of those checks and still not add up.** Each document well-formed, the set incoherent: two standards teaching different things, a shelf whose documents share a schema and no purpose, a specification technically accurate and unusable by the person who needs it. Cohesion is necessary, nowhere near sufficient, and — the important part — the only half that is mechanically decidable.

So obligations declare which they are, and the two are discharged by different mechanisms:

| | Cohesion obligations | Coherence obligations |
|---|---|---|
| Decidable | Yes, deterministically | No — requires judgement |
| Mechanism | Engine checks | Sampled audit, efficacy probes, reader feedback |
| Coverage | Total (every document, every run) | Sampled (a subset, periodically) |
| Posture | Advisory → blocking | Detective, always |
| Failure mode | False positives | Sampling misses things |

The system does not pretend one covers the other, and a green check run is never reported as "the corpus is coherent". It means the corpus is *cohesive*, which is a real and useful thing to know, and a smaller claim.

### Declaration moves the boundary

The line between the two is not fixed by subject matter. It is fixed by whether a claim has been *declared in the graph*, and that means an author can move it.

Contradiction is the clearest case. Detecting that two documents disagree is undecidable structurally — it requires reading both and judging. But a `conflicts_with` edge between two decisions is an assertion already in the front matter, and everything downstream of it is ordinary graph work: spec 2's rule that [two `current` decisions joined by `conflicts_with`](02-taxonomy-model.md#the-decision-relation-vocabulary) is an invalid state is deterministic, total, and blocking-eligible. The judgement happened once, when the author declared the edge. The check is cohesion thereafter.

This generalises, and it is worth stating as a design rule rather than an observation about one relation:

> **A coherence obligation becomes a cohesion obligation the moment the judgement it requires is recorded as data.** Where a coherence concern recurs, the question to ask is not "how do we detect this?" but "what could an author declare that would make detecting it unnecessary?"

Two consequences the rest of this document depends on.

**The sweep hunts the undeclared half only.** Anything the graph already asserts is the engine's job, and a sampled LLM pass that re-derives it is slower, dearer, and less reliable than the check that already exists.

**Declared coverage is partial, and stays that way.** `conflicts_with` is `decision`-to-`decision`; two standards that contradict each other have no way to say so, and until they do, that contradiction is coherence work. Widening the relation's endpoints would move more of it across the line — worth doing when there is evidence of the need, and not worth pre-emptively, since a relation authors do not use buys nothing and an unused relation is itself a finding about the taxonomy.

## Obligations are data

An obligation is a stable, identified invariant the corpus commits to, declared as machine-readable data — not a bullet in a strategy document that no tool can read.

```yaml
obligations:
  - id: OB-001
    statement: Behaviour-changing code updates its governing specification in the same change
    rationale: A stale specification actively misleads humans, agents, and auditors
    class: cohesion
    prevents: content.outdated          # observed defect class
    severity: high
  - id: OB-014
    statement: Every live decision is reachable from at least one artefact it constrains
    rationale: Rationale nobody can find from the thing it explains is rationale nobody reads
    class: cohesion
    prevents: process.traceability
    severity: medium
  - id: OB-022
    statement: A document is usable by its declared audience without tacit context
    rationale: Form-correct prose can still be unusable, and nothing structural detects it
    class: coherence
    prevents: content.incomplete
    severity: high
```

### Obligations are derived from observed defects, not invented

`prevents:` is required, and it names a class in a **documentation-defect taxonomy** — an empirically derived one, from mining real documentation problems and surveying practitioners, rather than a list assembled from our own experience.

The default register is built by walking that taxonomy: content defects (incorrect, incomplete, outdated, inconsistent), presentation defects (readability, organisation), and process defects (maintenance, traceability, contribution friction). For each class, the question is what obligation would prevent it, and whether we can discharge that obligation at acceptable cost.

Two benefits, and the second is the one that matters in practice:

1. Coverage becomes assessable against something external. "Which observed defect classes does our register not address?" has an answer.
2. Every rule can answer *"why are you making me do this?"* with an observed failure rather than an assertion of taste. An obligation whose `prevents:` field cannot be filled in is one nobody has seen go wrong, and it should be cut under "every rule earns its place" rather than kept because it sounds prudent.

A validator check enforces it: an obligation with no cited defect class is a finding.

## Controls are data

A control declares what discharges an obligation, when it runs, and how hard it bites.

```yaml
controls:
  - id: CT-007
    mechanism: check:relation-reciprocity
    discharges: [OB-014]
    trigger: pull_request
    posture: blocking
  - id: CT-021
    mechanism: scheduled:staleness-sweep
    discharges: [OB-003]
    trigger: weekly
    posture: detective
    handoff: task-per-finding
```

Controls are validated like anything else: a control naming a mechanism the engine does not implement, or a pipeline that does not exist, is a finding. A register that claims coverage it does not have is worse than no register.

## Every obligation has exactly one disposition

This is the rule that keeps the register honest:

| Disposition | Meaning |
|---|---|
| **Verified** | One or more controls discharge it |
| **Gap** | No control yet; wanted; tracked with an owner and, ideally, a target |
| **Unverifiable** | No mechanism can exist — accepted, with the reasoning recorded |

There is no fourth state and no silence. An obligation with no disposition is itself a finding. This is the check that stops an assurance model from decaying into a list of good intentions: the register is complete by construction or the build fails.

**The register is generated, never authored.** The binding lives on the control (`discharges:`), the disposition on the obligation, and the register — coverage, control health, suppressions, waivers — is a projection of the two, regenerated and checked like any other. An earlier draft treated it as a third authored artefact; two sources of truth for one binding is exactly the drift this system exists to kill. What does not change is its standing: the register is mandatory and inspectable, and an uncovered or degrading control cannot hide because the view surfacing it is not optional.

The **coverage report** — what fraction of obligations are verified, by severity, with the gap list — is generated from the register. It is never written by hand, because a hand-written coverage claim is a marketing document.

## Feedback targets

Whether a control is *useful* depends on whether its signal reaches someone who can act. The register records the target, and the gaps become visible:

| Target | Timing | Typical mechanism |
|---|---|---|
| Author | Pre-commit | Scaffolding, editor, agent, hooks |
| Author + reviewer | Pull request | CI checks, review annotations |
| Maintainer | Scheduled | Drift scans, efficacy probes, coverage reports |
| Team | Periodic | Conformance audits |
| Next reader | Continuous | *Usually absent* — reader feedback loops |
| Downstream consumer | On release | *Usually absent* — deprecation and breaking-change notice |

The last two rows are where documentation systems consistently fail, and naming them in the model is how they stop being invisible. A reader who cannot tell the maintainer that a document did not answer their question is a control that does not exist.

## Promotion: advisory to blocking

A new check ships **advisory**. It becomes blocking only against evidence:

- a stated observation window;
- a false-positive rate under a declared threshold;
- an unambiguous, mechanical remediation path;
- no unresolved escape-hatch concentration on one shelf.

The criteria are recorded with the control, so promotion is a decision with a paper trail rather than an argument about someone's tolerance for red builds. The inverse is also specified: a blocking check whose false-positive rate rises past the threshold is demoted, not endured.

A false-positive rate needs a collection mechanism, or every criterion above is unfalsifiable in practice. The mechanism is the suppression reason ([below](#suppression)): `false_positive` labels a finding wrong, `accepted_deviation` labels it right but tolerated, and only the former counts toward promotion and demotion statistics. Nobody is asked to label findings as a separate chore — the label rides on the escape hatch authors already use, which is the only place the judgement is actually made.

That mechanism has a blind spot exactly where promotion looks, and it is named rather than papered over: an advisory finding blocks nothing, so nobody is forced to suppress it — the rational response to advisory noise is to ignore it. Suppression-derived labels therefore measure the biased subset of findings that annoyed someone enough to act, and an advisory check can hold a catastrophic real false-positive rate behind a clean measured one. So the two directions use different evidence. **Demotion** runs on suppression labels alone, because a blocking check forces engagement and its labels are dense. **Promotion** additionally requires an adjudicated sample: during the declared observation window, a fixed random sample of the candidate check's unsuppressed findings is put in front of a human — at review, or in the periodic triage the conformance audit already schedules — and dispositioned with the same two labels. A check whose sample was never adjudicated has not finished its observation window, however long it has been advisory.

### Discharging coherence obligations: the assisted sweep

Naming a coherence class is easy; discharging it is the hard part, and structural checks cannot. A periodic **LLM-assisted coherence sweep** is the mechanism: an agent reads a bounded slice of the corpus and reports what no linter can see —

- pages that contradict each other while both remain current *and neither declares it* — a declared conflict is already a deterministic check, and the sweep's value is entirely in the contradictions nobody has noticed yet;
- claims a newer source has quietly superseded;
- concepts referenced throughout and defined nowhere;
- documents whose declared audience could not actually use them.

Four constraints keep this inside the rules the rest of the system obeys:

1. **It produces findings, never verdicts.** "No LLM in the validation path" ([spec 5](05-ai-integration.md#what-we-do-not-do)) governs decisions that gate. A coherence finding is a prompt for human attention, and is marked as one.
2. **It is sampled and detective**, like every coherence control — bounded slices on a schedule, never a blocking gate.
3. **Findings cite evidence.** Each names the documents it compared and quotes the passages it believes conflict, so a human can adjudicate in seconds. An unfalsifiable finding is noise, and noise gets the whole sweep switched off.
4. **It never re-derives what the graph declares.** The sweep runs against the undeclared half by construction, and a finding restating an edge already in the front matter is a defect in the sweep rather than a finding about the corpus.

The best outcome of a sweep is therefore not a finding but an **edge**: a contradiction it surfaces should end as a declared `conflicts_with`, after which the engine owns it permanently and the sweep never needs to find it again. A coherence control whose findings never convert into declarations is doing the same work every cycle — which is the accumulation failure that [spec 5](05-ai-integration.md#what-we-do-not-do) rejects RAG for, appearing in our own assurance layer.

This is the same division the system draws everywhere: deterministic tooling for what is decidable, judgement for what is not, and no pretence that either does the other's job.

## Measuring coherence where we can: continuity across links

Coherence resists mechanisation, but one component of it does not.

Centering Theory models local coherence as continuity of focus: adjacent utterances that keep the same entity in view are easy to follow, and each shift of focus imposes inference cost on the reader. The corpus analogue is direct. For every relation edge, the engine computes what the two endpoints share — a component, a domain, an identifier, a code-path anchor, a facet value. An edge whose endpoints share nothing is a **focus shift**: the reader must reorient on arrival.

Individual shifts are fine and often necessary. The **distribution** is the signal:

- a shelf whose outbound edges are mostly focus shifts is one a reader cannot traverse without re-orienting at every hop;
- a relation type that is nearly always a shift is probably being used as a generic "see also" and should be either narrowed or demoted to `association`;
- a rising shift ratio over time means the corpus is fragmenting faster than its links are being maintained.

Advisory, permanently. There is no threshold at which a focus shift is *wrong*, and gating on it would produce link-padding — authors adding shared keywords to satisfy a check, which destroys the measure. It is reported as a corpus-health metric alongside coverage, and it is the first thing we have that measures coherence rather than cohesion.

## Absence is a finding class of its own

Every mechanism above validates something that exists. None of them can see the document that should exist and does not — the accepted proposal nobody implemented, the incident with no postmortem, the decision that never reached a specification.

**Participation expectations** ([spec 2](02-taxonomy-model.md#participation-expectations)) close that gap. A kind declares that its documents, in a given state, are expected to acquire a relation to a document of another kind within a window; the engine reports the ones that did not.

They are constrained deliberately:

- **detective only, never blocking** — the work may legitimately be in flight, deferred, or abandoned for good reason, and none of those are defects;
- **windowed, from a declared origin** — an expectation with no time bound is a wish, not a control, and a window with no origin is not a window;
- **reported against the originating document** — that is where the person who can act will look.

This is the drift readers complain about most, and it is invisible to link and front-matter validation because there is nothing malformed to find.

## No silent passes: every document is accounted for

Every check discussed so far reports what it found. None of them reports what it never looked at, and that gap is where an assurance system quietly stops working.

The failure mode is general, not specific to any check engine. A document whose front matter fails to parse, whose kind cannot be resolved, or which sits outside every declared shelf pattern is not *checked and passed* — it is **never checked**, and a run that produces no findings for it is indistinguishable from a run that cleared it. The most broken document in the corpus is the one most likely to escape, because breakage is often what stops it being classified in the first place.

This is not hypothetical for standards-based validation in particular: SHACL's conformance is defined as "no validation results were produced", and it has no notion of completeness at all — a node no target selects simply conforms ([evaluation](../evaluations/shacl-worked-example.md#problem-two-silent-passes)).

So the engine establishes coverage as a **separate, prior guarantee**:

| Obligation | Statement |
|---|---|
| OB-COV-1 | Every file under the corpus root is classified, or reported as unclassifiable |
| OB-COV-2 | Every classified document is routed to at least one check |
| OB-COV-3 | Every run reports its coverage: documents seen, classified, checked, and skipped — with reasons |

Three consequences worth stating plainly:

- **A document that matched no check is a finding**, not a silent success. Usually it means a shelf pattern is wrong or a file is misplaced, and both are worth knowing.
- **Coverage is reported on clean runs too.** "No findings across 412 of 412 documents" and "no findings across 380 of 412" are entirely different results, and a report that cannot distinguish them is not trustworthy.
- **Parse failures are findings, never omissions.** A file the engine cannot read is reported as such and counted, rather than dropping out of the denominator.

This is the assurance model applied to itself: the system that insists every obligation carries a disposition, and that visible incompleteness beats apparent completeness, owes the same discipline to its own coverage.

## Findings

Every finding, from every mechanism, has one shape:

```json
{
  "rule": "relation.reciprocity.missing",
  "severity": "error",
  "obligation": "OB-014",
  "path": "docs/decisions/dr-0042.md",
  "line": 7,
  "message": "DR-ACME-0042 declares supersedes: DR-ACME-0031, which does not link back",
  "remediation": "add 'superseded_by: DR-ACME-0042' to docs/decisions/dr-0031.md",
  "fixable": true
}
```

Uniform findings are what make the rest cheap: one renderer per output format (human, Markdown for review comments, JSON, SARIF), one severity model, one suppression mechanism, one path from finding to fix. Every finding names the obligation it serves — a check that cannot say which invariant it protects has not earned its place.

## Suppression

Suppression is allowed, bounded, and observable: scoped to a file or block, it carries an expiry, and it states a reason from a closed set — `false_positive` (the finding is wrong) or `accepted_deviation` (the finding is right and tolerated for now). Suppressions are inventoried in the coverage report, because a rule with fifty suppressions is not a rule — it is a finding about the taxonomy.

Both constraints were looser in an earlier draft, and each looseness broke something downstream. Expiry was optional here while waiver expiry ([spec 7](07-distribution-and-federation.md#waivers)) was mandatory — making the local mechanism an individual author reaches for at a red check the leakier of the two, which is backwards. And an undifferentiated reason field conflated "wrong" with "tolerated", which made the false-positive rate — the number the promotion machinery above runs on — unmeasurable.

A finding can now fall under more than one escape mechanism at once — a waived rule, a `migration-pending` document ([spec 7](07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states)), and a file-scoped suppression. Coverage accounting applies one, by fixed precedence — **waiver, then migration-pending, then suppression** — and counts the finding in exactly that bucket, so the three inventories partition the escaped findings rather than triple-counting them. The wider mechanism wins because it carries the wider accountability: a waiver has an owner and is visible to the publisher, a migration state has an expiry and a task list, and a suppression is one author's local judgement.

## The adaptive layer reports cost, not just coverage

The adaptive class exists to decide whether the other three are worth what they cost, which requires measuring the cost. Three metrics, reported together:

| Metric | Source | Question it answers |
|---|---|---|
| **Coverage** | Obligation register | What fraction of obligations are discharged, by severity? |
| **Assisted fraction** | Scaffolder and agent instrumentation | How much of authoring is the tooling carrying, and is that rising or falling? ([spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric)) |
| **Efficacy** | Probe suite | Does the instruction surface change agent behaviour at all? ([spec 5](05-ai-integration.md)) |

Coverage alone is a number that only goes up, and a system optimising it will happily add obligations nobody can satisfy. Read against capture cost and efficacy, it becomes a trade: this much assurance, at this much author burden, with this much demonstrated effect. A rule that raises cost and moves neither of the others is a rule to delete — and deletion is a success, recorded as one.

## Conformance audit

The controls above verify *form*. Whether a specification still describes the system is a semantic question, and periodically a human (or a supervised agent) must sample the corpus and answer it. The audit produces evidence records with their own kind, lifecycle, and identifiers — inside the corpus, governed like everything else. An audit whose findings are not addressable is theatre; an audit whose output is a typed, tracked document is a control.

## The system's own assurance

headwater's obligations, controls, and gaps live in headwater's own corpus and are checked in headwater's own CI. Coverage numbers published for the project are generated by the same command an adopter runs. If we exempt ourselves from something, that exemption is visible in the register — which is exactly the property we are asking adopters to accept.
