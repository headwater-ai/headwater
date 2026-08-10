# 4 — Assurance model

How the system knows that it works, and admits where it does not.

## Assurance, not enforcement

Enforcement implies a gate that stops bad things. Real documentation systems need four different postures, because no single gate catches enough:

| Class | Acts | Example |
|---|---|---|
| **Preventive** | Before the mistake lands | Scaffolding, editor validation, agent behavior, commit hooks |
| **Detective** | After it lands | CI checks, scheduled drift scans, staleness sweeps |
| **Corrective** | Repairs what was found | Auto-fix, generated remediation tasks, agent-raised change proposals |
| **Adaptive** | Retunes the mechanisms | Efficacy probes, false-positive tracking, promotion decisions |

Most systems build the first two and claim the set. The adaptive layer decides if the other three are worth their cost. Here it is specified as a binding obligation, not an aspiration.

## Cohesion and coherence are different obligations

Linguistics draws a distinction that this model needs. **Cohesion** is the set of surface ties that bind a text — references that resolve, links that land, vocabulary that is used consistently. **Coherence** is the reader's experience that the whole holds together. Cohesion is a property of the artifact. Coherence is a property of the encounter.

Almost everything that the engine checks is cohesion: links resolve, relations are reciprocal, identifiers bind, enums are respected, sections are present. All of it is decidable, deterministic, and blocking-eligible.

**A corpus can pass every one of those checks and still not add up.** Each document can be well formed while the set is incoherent. Two standards can teach different things. A shelf's documents can share a schema and no purpose. A specification can be technically accurate and unusable by the person who needs it. Cohesion is necessary but far from sufficient, and — this is the important part — it is the only half that is mechanically decidable.

So each obligation declares which it is, and different mechanisms discharge the two:

| | Cohesion obligations | Coherence obligations |
|---|---|---|
| Decidable | Yes, deterministically | No, it needs judgment |
| Mechanism | Engine checks | Sampled audit, efficacy probes, reader feedback |
| Coverage | Total (every document, every run) | Sampled (a subset, at intervals) |
| Posture | Advisory → blocking | Detective, always |
| Failure mode | False positives | Sampling misses things |

The system does not pretend that one covers the other. A green check run is never reported as "the corpus is coherent". It means that the corpus is *cohesive*. That is a real and useful thing to know, and a smaller claim.

### Declaration moves the boundary

The line between the two is not fixed by subject matter. It is fixed by one question: is the claim *declared in the graph*? That means that an author can move the line.

Contradiction is the clearest case. To detect that two documents disagree is structurally undecidable — someone must read both and judge. But a `conflicts_with` edge between two decisions is an assertion already in the front matter. Everything downstream of it is ordinary graph work.

Spec 2 gives the rule that [two `current` decisions joined by `conflicts_with`](02-taxonomy-model.md#the-decision-relation-vocabulary) is an invalid state. That rule is deterministic, total, and blocking-eligible. The judgment happened once, when the author declared the edge. After that, the check is cohesion.

This generalizes, and we state it as a design rule rather than as an observation about one relation:

> **A coherence obligation becomes a cohesion obligation the moment the judgment that it needs is recorded as data.** Where a coherence concern recurs, do not ask "how do we detect this?" Ask instead: "what could an author declare that would make detecting it unnecessary?"

The rest of this document depends on two consequences.

**The sweep examines only the undeclared half.** Anything that the graph already asserts is the engine's job. A sampled LLM pass that re-derives it is slower, more costly, and less reliable than the check that already exists.

**Declared coverage is partial, and stays that way.** `conflicts_with` is `decision`-to-`decision`. Two standards that contradict each other have no way to say so, and until they do, that contradiction is coherence work. Wider endpoints on the relation can move more of it across the line. That is worth the work when there is evidence of the need, and not pre-emptively. A relation that authors do not use gives no benefit, and an unused relation is itself a finding about the taxonomy.

## Obligations are data

An obligation is a stable, identified invariant that the corpus commits to. It is declared as machine-readable data, not as a bullet in a strategy document that no tool can read.

```yaml
obligations:
  - id: OB-001
    statement: Behavior-changing code updates its governing specification in the same change
    rationale: A stale specification actively misleads humans, agents, and auditors
    class: cohesion
    prevents: content.outdated          # observed defect class
    severity: high
  - id: OB-014
    statement: Every live decision is reachable from at least one artifact it constrains
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

`prevents:` is required, and it names a class in a **documentation-defect taxonomy**. That taxonomy is derived empirically, from real documentation problems and from practitioner surveys. It is not a list that we assembled from our own experience.

The default register is built from a walk of that taxonomy: content defects (incorrect, incomplete, outdated, inconsistent), presentation defects (readability, organization), and process defects (maintenance, traceability, contribution friction). For each class, we ask two questions. What obligation prevents this defect? Can we discharge that obligation at acceptable cost?

This gives two benefits, and the second is the one that matters in practice:

1. Coverage becomes assessable against something external. "Which observed defect classes does our register not address?" has an answer.
2. Every rule can answer *"why are you making me do this?"* with an observed failure rather than an assertion of taste. An obligation whose `prevents:` field cannot be filled in is one with no observed defect behind it. Cut it under "every rule earns its place". Do not keep it because it sounds prudent.

A validator check enforces this: an obligation with no cited defect class is a finding.

## Controls are data

A control declares what discharges an obligation, when it runs, and what posture it has.

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

Controls are validated like anything else. A control that names a mechanism that the engine does not implement, or a pipeline that does not exist, is a finding. A register that claims coverage that it does not have is worse than no register.

## Every obligation has exactly one disposition

This is the rule that keeps the register honest:

| Disposition | Meaning |
|---|---|
| **Verified** | One or more controls discharge it |
| **Gap** | No control yet. Wanted, and tracked with an owner and, ideally, a target |
| **Unverifiable** | No mechanism can exist. Accepted, with the reasoning recorded |

There is no fourth state and no silence. An obligation with no disposition is itself a finding. This is the check that stops the decay of an assurance model into a list of good intentions. The register is complete by construction, or the build fails.

**The register is generated, never authored.** The binding lives on the control (`discharges:`), and the disposition lives on the obligation. The register — coverage, control health, suppressions, waivers — is a projection of the two. The engine regenerates and checks it like any other projection. An earlier draft treated it as a third authored artifact. Two sources of truth for one binding is exactly the drift that this system exists to kill.

What does not change is the register's standing: it is mandatory and inspectable. An uncovered control cannot hide, and neither can one that degrades, because the view that surfaces them is not optional.

The **coverage report** — what fraction of obligations are verified, by severity, with the gap list — is generated from the register. It is never written by hand, because a hand-written coverage claim is a marketing document.

## Feedback targets

A control is *useful* only if its signal reaches someone who can act. The register records the target, and the gaps become visible:

| Target | Timing | Typical mechanism |
|---|---|---|
| Author | Pre-commit | Scaffolding, editor, agent, hooks |
| Author + reviewer | Pull request | CI checks, review annotations |
| Maintainer | Scheduled | Drift scans, efficacy probes, coverage reports |
| Team | Periodic | Accuracy audits |
| Next reader | Continuous | *Usually absent* — reader feedback loops |
| Downstream consumer | On release | *Usually absent* — deprecation and breaking-change notice |

Documentation systems consistently fail on the last two rows. The model names them, and that is how they become visible. A reader who cannot tell the maintainer that a document did not answer their question is a control that does not exist.

## Promotion: advisory to blocking

A new check starts as **advisory**. It becomes blocking only against evidence:

- a stated observation window
- a false-positive rate under a declared threshold
- an unambiguous, mechanical remediation path
- no unresolved concentration of escape hatches on one shelf

The criteria are recorded with the control. So promotion is a decision with a paper trail, not an argument about someone's tolerance for red builds. The inverse is also specified. A blocking check whose false-positive rate rises past the threshold is demoted. It is not endured.

A false-positive rate needs a collection mechanism, or every criterion above is unfalsifiable in practice. The mechanism is the suppression reason ([below](#suppression)). `false_positive` means that the finding is wrong, and `accepted_deviation` means that it is right but tolerated. Only `false_positive` counts toward the promotion and demotion statistics. Nobody is asked to label findings as a separate task. The label attaches to the escape hatch that authors already use, and that is the only place where the judgment occurs.

That mechanism has a blind spot exactly where promotion looks. We name the blind spot, and we do not hide it. An advisory finding blocks nothing, so nobody is forced to suppress it. The rational response to advisory noise is to ignore it. So suppression-derived labels measure a biased subset: the findings that annoyed someone enough to act. So an advisory check can hold a catastrophic real false-positive rate behind a clean measured one.

So the two directions use different evidence. **Demotion** runs on suppression labels alone, because a blocking check forces engagement and its labels are dense.

**Promotion** also needs an adjudicated sample. During the declared observation window, a fixed random sample of the candidate check's unsuppressed findings goes in front of a human. This happens at review, or in the periodic triage that the accuracy audit already schedules. The human applies the same two labels. A check whose sample was never adjudicated did not finish its observation window, however long it was advisory.

### Discharging coherence obligations: the assisted sweep

To name a coherence class is easy. To discharge it is the hard part, and structural checks cannot do it. The mechanism is a periodic **LLM-assisted coherence sweep**. An agent reads a bounded slice of the corpus and reports what no linter can see:

- pages that contradict each other while both remain current *and neither declares it*. A declared conflict is already a deterministic check, and the sweep's value is fully in the contradictions that nobody noticed yet
- claims that a newer source quietly superseded
- concepts that are referenced throughout and defined nowhere
- documents whose declared audience cannot actually use them

Four constraints keep this inside the rules that the rest of the system obeys:

1. **It reports findings, never verdicts.** "No LLM in the validation path" ([spec 5](05-ai-integration.md#what-we-do-not-do)) governs decisions that gate. A coherence finding is a prompt for human attention, and it is marked as one.
2. **It is sampled and detective**, like every coherence control — bounded slices on a schedule, never a blocking gate.
3. **Findings cite evidence.** Each names the documents that it compared, and quotes the passages that it believes to be in conflict. So a human can adjudicate in seconds. An unfalsifiable finding is noise, and noise causes the team to switch the whole sweep off.
4. **It never re-derives what the graph declares.** The sweep runs against the undeclared half by construction. A finding that restates an edge already in the front matter is a defect in the sweep, not a finding about the corpus.

So the best outcome of a sweep is not a finding but an **edge**. The correct end state for a contradiction that the sweep surfaces is a declared `conflicts_with`. After that, the engine owns it permanently, and the sweep never needs to find it again. A coherence control whose findings never convert into declarations does the same work every cycle. That is the accumulation failure that [spec 5](05-ai-integration.md#what-we-do-not-do) rejects RAG for, now in our own assurance layer.

This is the same division that the system draws everywhere. Deterministic tooling handles what is decidable, and judgment handles what is not. There is no pretense that either does the other's job.

## Measuring coherence where we can: continuity across links

Coherence resists mechanization, but one component of it does not.

Centering Theory models local coherence as continuity of focus. Adjacent utterances that keep the same entity in view are easy to follow. Each shift of focus imposes an inference cost on the reader. The corpus analog is direct. For every relation edge, the engine computes what the two endpoints share — a component, a domain, an identifier, a code-path anchor, a facet value. An edge whose endpoints share nothing is a **focus shift**: the reader must reorient on arrival.

Individual shifts are fine and often necessary. The **distribution** is the signal:

- a shelf whose outbound edges are mostly focus shifts forces a re-orientation at every hop
- a relation type that is nearly always a shift is probably in use as a generic "see also". The fix is to narrow it, or to demote it to `association`
- a shift ratio that rises over time means that the corpus fragments faster than its links are maintained

This metric is advisory, permanently. There is no threshold at which a focus shift is *wrong*. A gate on it causes link-padding: authors add shared keywords to satisfy the check, and that destroys the measure. The shift ratio is reported as a corpus-health metric alongside coverage. It is the first thing that we have that measures coherence rather than cohesion.

## Absence is a finding class of its own

Every mechanism above validates something that exists. None of them can see the document that must exist and does not. Examples: the accepted proposal that nobody implemented, the incident with no postmortem, the decision that never reached a specification.

**Participation expectations** ([spec 2](02-taxonomy-model.md#participation-expectations)) close that gap. A kind declares an expectation: its documents, in a given state, acquire a relation to a document of another kind inside a window. The engine reports the ones that did not.

They are constrained deliberately:

- **detective only, never blocking** — the work may legitimately be in flight, deferred, or abandoned for good reason. None of those are defects
- **windowed, from a declared origin** — an expectation with no time bound is a wish, not a control. A window with no origin is not a window
- **reported against the document of origin** — that is where the person who can act will look

This is the drift that readers complain about most. It is invisible to link and front-matter validation, because there is nothing malformed to find.

## No silent passes: every document is accounted for

Every check discussed so far reports what it found. None of them reports what it never looked at. That gap is where an assurance system quietly fails.

The failure mode is general, not specific to any check engine. Some documents have front matter that fails to parse, a kind that cannot be resolved, or a location outside every declared shelf pattern. Such a document is not *checked and passed* — it is **never checked**. A run that reports no findings for it is indistinguishable from a run that cleared it. The most broken document in the corpus is the one most likely to escape, because breakage often prevents classification in the first place.

This is not hypothetical, for standards-based validation in particular. SHACL defines conformance as "no validation results were produced", and it has no notion of completeness at all. A node that no target selects simply conforms ([evaluation](../evaluations/shacl-worked-example.md#problem-two-silent-passes)).

So the engine establishes coverage as a **separate guarantee that comes first**:

| Obligation | Statement |
|---|---|
| OB-COV-1 | Every file under the corpus root is classified, or reported as unclassifiable |
| OB-COV-2 | Every classified document is routed to at least one check |
| OB-COV-3 | Every run reports its coverage: documents seen, classified, checked, and skipped — with reasons |

Three consequences deserve a plain statement:

- **A document that matched no check is a finding**, not a silent success. Usually it means that a shelf pattern is wrong or that a file is misplaced, and both are worth attention.
- **Coverage is reported on clean runs too.** "No findings across 412 of 412 documents" and "no findings across 380 of 412" are very different results. A report that cannot distinguish them is not trustworthy.
- **Parse failures are findings, never omissions.** A file that the engine cannot read is reported as such and counted. It does not drop out of the denominator.

This is the assurance model applied to itself. The system insists that every obligation carries a disposition, and that visible incompleteness beats apparent completeness. It owes the same discipline to its own coverage.

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

Uniform findings make the rest cheap: one renderer per output format (human, Markdown for review comments, JSON, SARIF), one severity model, one suppression mechanism, one path from finding to fix. Every finding names the obligation that it serves. A check that cannot say which invariant it protects did not earn its place.

## Suppression

Suppression is permitted, bounded, and observable. It is scoped to a file or block, it carries an expiry, and it states a reason from a closed set. The reasons are `false_positive` (the finding is wrong) and `accepted_deviation` (the finding is right and tolerated for now). The coverage report includes an inventory of suppressions. A rule with fifty suppressions is not a rule — it is a finding about the taxonomy.

Both constraints were looser in an earlier draft, and each looseness broke something downstream. Expiry was optional here, while waiver expiry ([spec 7](07-distribution-and-federation.md#waivers)) was mandatory. That made the local mechanism — the one that an individual author reaches for at a red check — the leakier of the two, which is backwards. And an undifferentiated reason field conflated "wrong" with "tolerated". That made the false-positive rate unmeasurable, and that is the number that the promotion machinery above runs on.

A finding can now fall under more than one escape mechanism at once — a waived rule, a `migration-pending` document ([spec 7](07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states)), and a file-scoped suppression. Coverage accounting applies one, by fixed precedence — **waiver, then migration-pending, then suppression** — and counts the finding in exactly that bucket. So the three inventories partition the escaped findings, and no finding is counted three times. The wider mechanism wins because it carries the wider accountability. A waiver has an owner and is visible to the publisher. A migration state has an expiry and a task list, and a suppression is one author's local judgment.

## The adaptive layer reports cost, not just coverage

The adaptive class exists to decide if the other three are worth what they cost. That decision needs a measure of the cost. Three metrics are reported together:

| Metric | Source | Question it answers |
|---|---|---|
| **Coverage** | Obligation register | What fraction of obligations are discharged, by severity? |
| **Assisted fraction** | Scaffolder and agent instrumentation | How much of authoring does the tooling carry, and does that rise or fall? ([spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric)) |
| **Efficacy** | Probe suite | Does the instruction surface change agent behavior at all? ([spec 5](05-ai-integration.md)) |

Coverage alone is a number that only goes up. A system that optimizes it will happily add obligations that nobody can satisfy. When coverage is read against capture cost and efficacy, it becomes a trade: this much assurance, at this much author burden, with this much demonstrated effect. A rule that raises cost and moves neither of the others is a rule to delete. Deletion is a success, and it is recorded as one.

## Accuracy audit

The controls above verify *form*. One question is semantic: does a specification still describe the system? At intervals, a human (or a supervised agent) must sample the corpus and answer that question. The name is deliberate. Conformance is what [spec 7](07-distribution-and-federation.md#conformance) evaluates about an adopter, and it is a different question with its own command. The audit makes evidence records with their own kind, lifecycle, and identifiers — inside the corpus, governed like everything else. An audit whose findings are not addressable is theater. An audit whose output is a typed, tracked document is a control.

## The system's own assurance

Headwater's obligations, controls, and gaps live in Headwater's own corpus and are checked in Headwater's own CI. The same command that an adopter runs generates the coverage numbers that we publish for the project. If we exempt ourselves from something, that exemption is visible in the register. That is exactly the property that we ask adopters to accept.
