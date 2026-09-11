# The maturity ladder

A new adopter faces three dials and no guidance about which way to turn them. This file names the ordered positions that most adopters will want, and it states what a position is allowed to mean.

The short form: **a maturity level is a named, ordered subset of the conformance rule set, shipped as package data.** It adds no declaration, changes no posture, and weakens no core. It is the [starter kit](../spec/07-distribution-and-federation.md#the-starter-kit-is-an-assembly) pattern applied to conformance instead of to bundles.

## The ladder invents nothing

An adopter turns four dials. Three of them already have mechanisms, and the ladder is a selection over those mechanisms rather than a fourth one.

| Dial | The mechanism that already exists |
|---|---|
| How much taxonomy | The bundle selection that [`headwater init`](../spec/07-distribution-and-federation.md#the-interview) composes from an interview |
| How much debt is deferred | The [adoption payload](../spec/07-distribution-and-federation.md#first-contact-adoption-is-a-migration-from-no-taxonomy), with owners, expiries, and a remaining-pair count on every run |
| Which capabilities are wired up | The [conformance](../spec/07-distribution-and-federation.md#conformance) rule set, which ships with the package |
| Which checks block | [Promotion](../spec/04-assurance-model.md#promotion-advisory-to-blocking), and the ladder must not reach for this one |

The third dial already carries the ratchet that a maturity model would otherwise have to build. Conformance rules ship with the package, so a pin advance brings newly added requirements into force on every consumer at once. Improve the method, and the next upgrade surfaces the new gap. A ladder is the ordering over those rules that tells an adopter which gap to close first.

`headwater conformance --level L2` reports the distance between here and there. Nothing else is added.

## Where the ladder actually lives, and which rungs ship

This file is the argument. `.headwater/packages/headwater-standard/conformance.yml` is the ladder, and where the two disagree the package is right.

**Three rungs ship, and they are L0, L1 and L2.** Each one names rules that the engine holds a reading for: `pin.current` and `lock.current` at L0, `corpus.classified` at L1, `projections.current` at L2. A rung arrives with the rules that earn it or it does not arrive, and a level naming no rule is refused at read time.

The four rungs above ship nothing yet, and each waits on a different thing.

| Rung | What it waits on |
|---|---|
| L3 Linked | A reading of relation participation and of installed hooks. The second is per-clone configuration, so it waits on the attestation record below |
| L4 Gated | An attestation record. Branch protection lives in the administration surface of a forge, and no reading of a tree decides it |
| L5 Publishing | A reading of whether the local overlay is itself a published package |
| L6 Measured | The measurement layer, which is [HW-OBL-0023](../obligations/0023-no-corpus-has-authored-enough-cues-to-grade.md) and the probe tiers |

**Three rules ship with no rung at all.** `checks.wired`, `gates.required` and `hooks.installed` each declare an attestation in place of a reading. `headwater conformance` names each one, states what would decide it, and counts it as neither met nor missing. No rung names one of them, because a rung that names a rule nothing can meet is a rung nobody reaches, and a ladder with an unreachable rung is worse than a ladder that stops.

**A level is never claimed, and there is no key to claim one in.** The third open question below asked whether an adopter declares a target level. The answer the engine gives is narrower than either option: an adopter may ask about a rung with `--level <name>` and no file records an answer. The report derives the level from the rules that pass. A [waiver](../spec/07-distribution-and-federation.md#waivers) moves the exit status of a `--level` run and never the level the report states, so an adopter who accepts a deviation gets a green gate and the same rung as before.

## A level never sets posture

This is the rule that the rest of the file exists to protect.

**A level says what an adopter has wired up. It never says how hard a rule bites.**

A rung that reads "at level 4 these checks block" launders promotion. [Spec 4](../spec/04-assurance-model.md#promotion-advisory-to-blocking) requires a stated observation window, a false-positive rate under a declared threshold, an unambiguous mechanical remediation, and an adjudicated sample before any rule blocks. A ladder that promotes by declaration skips all four while looking rigorous. It would also be unenforceable in two cases that the specification already names:

- A control whose error classes are not both recoverable [ships at its final posture](../spec/04-assurance-model.md#where-promotion-does-not-apply) and never walks the path at all. The withholding rule of an export profile is the instance today.
- A control whose remediation needs judgment is [permanently advisory](../spec/04-assurance-model.md#where-promotion-cannot-finish), whatever its measured precision. The voice categories and the shift ratio are the instances today. No level can ever promote them, so no level may claim to.

Level 4 below is therefore phrased as a fact about process rather than about rules. It states that promotion has run. It does not state which rules it promoted.

## A level is never a second vocabulary

[Spec 8](../spec/08-design-departures.md) refuses documentation profiles as a primary abstraction, because a profile that names its own concepts becomes a parallel classification system that a team must hold in step with the taxonomy. A maturity ladder can fail in exactly that way.

The guard is a rule about what a rung may contain. **A rung is a set of conformance rules that already exist.** Where a level seems to need a concept that is not a conformance rule, that is a finding against the conformance rule set, and the remedy is a rule rather than a level vocabulary.

## The seven levels

Each level includes every level below it. The "needs" column names the engine capability that has to exist before an adopter can reach the rung at all, in the terms of the build order.

### L0 — Pointed at

**The adopter can** get a green build on the first run, and read a coverage number that is honest about what it has not checked.

**Conformance facts:** an overlay exists, from `init` or from `infer`. Checks run in continuous integration, advisory. The adoption payload is committed, and its open-pair count is reported on every run.

**Needs:** the check layer and the onboarding path.

This is the state that [first contact](../spec/07-distribution-and-federation.md#first-contact-adoption-is-a-migration-from-no-taxonomy) already defines. The corpus was never valid, so every finding it raises is the payload of the taxonomy's first version. Nothing here asks the adopter to fix a document.

### L1 — Classified

**The adopter can** ask what a document is and what it is for. `explain` answers, and the coverage number starts to mean something.

**Conformance facts:** the census holds no unclassified file. Every document carries a kind, a shelf, a warrant, and a lifecycle state. The classification pairs in the adoption payload are closed.

**Needs:** the check layer.

A document with zero check instances is a finding ([spec 12](../spec/12-check-layer.md#instances-and-why-coverage-needs-them)). Classification is the floor that makes every later number a fraction of something real.

### L2 — Regenerated

**The adopter can** stop maintaining derived artifacts by hand: indexes, navigation, agent rule files, the coverage report.

**Conformance facts:** projections are declared, and `generate --check` runs in continuous integration. No committed derived file lacks a generated-file marker.

**Needs:** the projection layer.

[Principle 3](../spec/00-vision-and-scope.md#design-principles) is the reason this rung is early. A derived file that a person can edit unseen will drift, and a reader will then trust a file that is wrong.

### L3 — Linked

**The adopter can** route a task to the documents that govern it, read lineage, and detect impact. This is the rung at which the system starts to pay.

**Conformance facts:** relations are declared and participation expectations are in force. The write-time hooks are installed, so an edge is created where its cost is lowest.

**Needs:** the authoring surface.

The hooks are part of the rung rather than an optional extra. Everything distinctive runs on the edges of the graph, and author-maintained links decay because the payer is not the beneficiary ([spec 5](../spec/05-ai-integration.md)). A corpus that declares relations and installs no scaffolding reaches this rung on paper and not in fact.

### L4 — Gated

**The adopter can** rely on the corpus at review time, because a rule that fires has earned the right to.

**Conformance facts:** promotion has run. Observation windows are closed, adjudicated samples are taken, and the rules that the evidence promoted are required on the default branch. Waivers carry owners and expiries.

**Needs:** the authoring surface, plus calendar time.

Read the first sentence again. The rung is that the process ran, and the outcome of that process belongs to the evidence.

### L5 — Publishing

**The adopter can** give the same method to a second team without giving them a fork.

**Conformance facts:** the local overlay is a published package with its own core, doctrine, and conformance rules. Consumers hold pins. Upstream awareness raises a draft change proposal rather than a notification.

**Needs:** distribution.

### L6 — Measured

**The adopter can** answer [principle 11](../spec/00-vision-and-scope.md#design-principles) for their own corpus, instead of inheriting a claim that this project has not yet measured for its own.

**Conformance facts:** probe campaigns run on a cadence. Capture cost is tracked. `taxonomy audit` and the accuracy audit are scheduled, and somebody acts on them.

**Needs:** the measurement layer.

## What a level does not mean

- **Not a score for the corpus.** The rungs measure what the adopter has wired up, and a small corpus at L4 is a normal thing rather than a contradiction.
- **Not a certification.** Nobody audits a claim to a level. `headwater conformance` evaluates a repository, and its report is the only artifact.
- **Not an order of quality.** An adopter who stops at L2 has made a reasonable choice, and the ladder is not an argument that they should go further.
- **Not a gate on support.** A published package serves a consumer at L0 exactly as it serves one at L5.

## What is settled here, and what is a guess

The **axes** are settled, because each one is forced by a mechanism that the specification already fixed. A ladder cannot set posture, cannot invent a vocabulary, and has nothing to select over except the conformance rule set.

The **rung contents are a guess**, and this file says so rather than waiting to be corrected. [13 — Open obligations](../spec/13-open-obligations.md#what-waits-on-a-first-adopter) records the bundle set as a guess about how adopters cluster, held open because no adopter exists yet. A ladder is the same class of guess one level out, and it gets the same treatment. It is data in a package, so the first real adopter revises it at the cost of a release.

Three questions are open and none of them blocks the rest:

- **Whether the levels are a chain or a partial order.** L2 and L3 are independent in principle. An adopter who wants routing before regeneration is not doing anything wrong, and the chain above is a claim about the common case rather than about necessity.
- **Whether an adopter declares a target level.** Closed by the shape of the verb rather than by an argument. `--level <name>` asks about a rung and writes nothing, so a run reports the distance when somebody asks and no file holds an answer that could fall out of step. A key that recorded a target would also be a key that an adopter could write a level into, and the section above states why no such key exists.
- **Whether a rung ever removes a rule.** Every rung above adds. If a real ladder needs a level that relaxes a requirement of the level below, the ordering is wrong, and the same argument applies that makes a bundle add-only.
