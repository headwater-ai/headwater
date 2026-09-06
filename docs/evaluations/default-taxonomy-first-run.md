---
id: HW-EVAL-default-taxonomy-first-run
status: current
status_since: 2026-08-10
last_verified: 2026-08-10
summary: The Q3 evidence, which wrote the base package out as real YAML and ran five adopters through their first day against it.
title: "What ships in the box — a first-run walkthrough"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: evaluate+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cited_by:
    - HW-REG-decisions
    - HW-REG-open-obligations
    - HW-SPEC-vision-and-scope
    - HW-SPEC-taxonomy-model
    - HW-SPEC-distribution-and-federation
    - HW-SPEC-glossary
---

# What ships in the box — a first-run walkthrough

Evidence for [Q3](../spec/09-open-questions.md#q3--how-much-of-the-default-taxonomy-ships-in-the-box). Q3 named three options and a leaning, and it argued them on adoption feel: too opinionated repels, too thin leaves a blank schema. Feel is not a method. This walkthrough replaces it with one. Author the candidate base package as real YAML, and run five adopters through their first day against it. Count what each one types and what each one deletes.

It produced four kinds of result.

**A decision, and the three options turn out not to compete.** The base package is minimal and derived from the core. The batteries-included taxonomy is a named bundle selection over that base. The interview composes a different selection. Q3 read these as three answers to one question. They are three layers, and each one needs the layer below it.

**A reason for the minimal base that is not about adoption feel.** A bundle that only adds is confluent with every other bundle, so any subset of bundles resolves. A bundle that has to remove is not. Only a minimal base lets every bundle be add-only. The size of the base is thus a property of the resolver, not a matter of taste.

**A correction to Q3's own list.** The leaning named decisions, standards, and guides as the small core. That list serves the `rationale`, `constraint`, and `procedure` purposes, and the immutable core requires `behavior`. The leaning's core fails the core. What the list actually describes is the starter kit, which is a different artifact.

**Six design defects that no packaging decision fixes.** Three of them sit in [spec 2](../spec/02-taxonomy-model.md), and the walkthrough found each one by writing the base package out and reading what it says. They are recorded at the end with the section that owns each.

## The question decomposes into three

Q3 as written bundles three decisions that have different answers.

1. **What does the base enable** — the taxonomy an adopter gets when they declare nothing.
2. **What does the package define but not enable**, and how does an adopter turn a definition on.
3. **How does an adopter reach their own taxonomy** — by interview, by inference from a tree that already exists, or by hand.

The second question is nearly answered already, and the answer was written for one declaration only. [Spec 2](../spec/02-taxonomy-model.md#the-decision-relation-vocabulary) rules that a package *defines* a full relation vocabulary and a taxonomy *enables* a subset of it. An overlay enables one more by reference, in a line, with no restatement. Everything Q3 calls an optional package is that mechanism, applied to more than relations.

So there is no packaging mechanism left to design, and this is the first result. [Spec 7](../spec/07-distribution-and-federation.md#profiles-are-publisher-overlays) already ruled that a profile is a publisher-shipped overlay and not a mechanism of its own. An optional package is the same thing pointed the other way. A profile carries `remove` operations for an archetype with fewer shelves, and an optional package carries `add` operations for an adopter that needs more. One mechanism, two conventional directions. Spec 2 removed the `profiles` declaration for exactly this reason, and to introduce `packages` now would be the same mistake with a new name.

That leaves two real questions: how big is the base, and how does an adopter move off it.

## The base, derived rather than chosen

The core states what every conformant taxonomy must have. So the base is derivable: it is the smallest taxonomy that satisfies the core and keeps every part of itself reachable. Nothing here was chosen for taste.

**The block below is the derivation, and the source is now a file.** For as long as this was the only committed copy, every tool that resolved a taxonomy parsed Markdown to reach it. The overlay resolver ended that, and [`packages/headwater-standard/`](../../packages/headwater-standard/) holds the package: `taxonomy.yml` is the source, and `package.yml` is the manifest. One key differs between the two. The block opens `package:`, which is spec 7's manifest key. And a taxonomy source may not declare a `package` block, because `package` is a reserved reference root. The split of the two files is what settled that collision.

```yaml
package: headwater/standard
version: 1.0.0

purposes:
  rationale:
    intent: explain why a choice was made and what it forecloses
    answers: ["why is it this way", "what was rejected", "what does this constrain"]
  behavior:
    intent: state what the system does, as it is now
    answers: ["what does this component do", "what may I rely on"]

vocabularies:
  lifecycle_state:
    - {value: draft,      role: initial}
    - {value: current,    role: live}
    - {value: superseded, role: terminal-retained}
    - {value: deprecated, role: terminal-retained}

facets:
  status:        {role: state,         values: $vocabularies.lifecycle_state, required: true}
  status_since:  {role: state_entered, type: date,   required: true}
  last_verified: {role: freshness,     type: date,   stale_after_days: 180}
  summary:       {role: scent,         type: string, required: true}

regimes:
  lifecycle:
    standard:
      initial: draft
      transitions: {draft: [current, deprecated], current: [superseded, deprecated]}
      retain_terminal: true
  voice:
    declarative: {forbid: [future_intent, change_narration, phased_rollout]}
  language:
    default: {tag: en-US, controlled: none}

anchors:
  code_path: {resolver: source-tree}

kinds:
  governed_document:
    abstract: true
    facets: {require: [status, status_since, last_verified, summary]}
  decision:
    is_a: governed_document
    purpose: rationale
    identifier: {scheme: decision_id}
    voice: declarative
    lifecycle: standard
    sections: {require: [Context, Decision, Consequences]}
  specification:
    is_a: governed_document
    purpose: behavior
    voice: declarative
    lifecycle: standard
    sections: {require: [Scope, Behavior]}

relations:
  supersedes:
    family: succession
    from: [governed_document]
    to:   [governed_document]
    inverse: superseded_by
    reciprocal: required
    nuclearity: multinuclear
    on_target: {set_state: superseded}
    created_by: scaffold
  governs:
    family: governance
    from: [governed_document]
    to:   [code_path]
    cardinality: many
    created_by: hook
  constrains:
    family: governance
    from: [decision]
    to:   [decision]
    created_by: agent
  conflicts_with:
    family: association
    from: [decision]
    to:   [decision]
    reciprocal: symmetric
    nuclearity: multinuclear
    invalid_when: {both: {status: current}}
    created_by: agent
  traces_to:
    family: evidence
    from: [governed_document]
    to:   [governed_document, code_path]
    created_by: hook

shelves:
  decisions:      {path: docs/decisions/**,      homogeneous: true, kind: decision}
  specifications: {path: docs/specifications/**, homogeneous: true, kind: specification}

identifier_schemes:
  decision_id: {pattern: "{namespace}-DR-{seq:04d}", namespace: repo, allocation: reconcile-first}

core:
  requires:
    - facet_role: state
    - facet_role: freshness
    - facet_role: scent
    - purpose: rationale
    - purpose: behavior
    - relation_family: succession
      lifecycle_sensitive: true

projections:
  - {kind: shelf_index, for: [decisions, specifications], output: "{shelf}/README.md"}
```

Four facets, one abstract kind, two concrete kinds, two shelves, five relations, one anchor. That is the whole base.

### Three things the derivation contradicts

**The smallest column of spec 2's worked example does not satisfy spec 2's core.** The example gives the small team the `rationale` and `procedure` purposes. The core requires `rationale` and `behavior`. A taxonomy with no behavior-serving kind has nowhere to say what the system does. And [spec 0](../spec/00-vision-and-scope.md) promises that a task or a code path resolves to the documents that govern it. Such a corpus has nothing for a code path to resolve to. The core is right and the column is wrong, which is worth stating in that order. The core came from [theory](../evaluations/theoretical-foundations.md), and the column was a sketch.

**No default relation attaches the corpus to code.** Spec 2 enables four by default, and it selected all four from the Kruchten decision-relation vocabulary. That vocabulary runs between decisions by construction, so the count was taken over a set that could not contain the edge in question. Add a behavior kind to that base and every document of that kind is unlinked. That is an orphan finding under the base's own generated checks. `governs`, which ends on the `code_path` anchor, is what fixes it. It also supplies write-time impact detection ([spec 5](../spec/05-ai-integration.md)), which is the most valuable thing the corpus does for a coding agent. No default edge carried it.

**Two of the default four have no mechanical creator.** Spec 2 states that every relation the default enables is creatable by scaffold, generator, or hook, and it explains why. A default that works only if assisted authoring raises edge capture is a bet rather than a design. Write the base out and the claim fails on its own list. A scaffold can propose `supersedes`, and a hook can propose `traces_to` and `governs` from the change. Nothing mechanical proposes `conflicts_with` or `constrains`. Both come from the coherence sweep, which is an agent.

The rule that does the intended work is narrower than the one spec 2 states: **no relation in the base is `created_by: author`.** That preserves the point, because the claim under test is that unassisted human capture decays. It also stops the base from asserting a creator it does not have. It also makes the bet visible instead of hiding it. Two of the base's five edges depend on the agent-authoring claim, which [HW-EVAL-theoretical-foundations](../evaluations/theoretical-foundations.md#what-the-theory-did-not-settle) records as the least-tested claim in the system. `taxonomy audit` already reports edge counts and staleness by creator, so the instrument to measure the dependence exists.

## Bundles, and why one line per relation does not generalize

Spec 2's enabling line reads `add: {relations.forbids: $package.optional.forbids}`. That is genuinely one line, and it is one line because a relation in the decision vocabulary references only kinds the base already has. The generalization fails at the first kind.

`kinds.control` needs an `attestation` purpose, a `control_id` identifier scheme, and the facets that carry a framework reference. It also needs a shelf to make it reachable, and the `mitigates` and `attests` relations that make it more than a shape. To enable it in one line produces five dangling references and a taxonomy that does not validate. So the package ships **bundles**: named add-only overlays, each with a declared dependency closure, each of which resolves as a single operation.

| Bundle | Adds | Requires |
|---|---|---|
| `procedure` | `guide` kind, its shelf, the `procedure` purpose | — |
| `standards` | `standard` kind, its shelf, the `constraint` purpose, `forbids`, `enables` | — |
| `evidence` | `evidence` kind, `implemented_by`, the decision-realized expectation | — |
| `proposals` | `proposal` kind, the narrative voice regime, the proposal-decided expectation | — |
| `operations` | `runbook`, `incident`, `postmortem`, the incident-postmortem expectation | `procedure` |
| `compliance` | `control`, `risk`, `attestation`, `mitigates`, `attests`, `does_not_comply_with`, control identifiers, an approval state | `standards`, `evidence` |

Two rules make this cheap, and both are already enforced by machinery that exists.

**A bundle may contain no `override` and no `remove`.** If a bundle needs to change the base, the base declared something it should not have. Add-only overlays over disjoint addresses commute, so the resolver's static confluence check ([spec 2](../spec/02-taxonomy-model.md#customization-by-composition)) proves that every subset of bundles resolves. The publisher runs that check pairwise once per release, and no adopter can select a combination that fails. That is the argument that fixes the size of the base, and it is stronger than the adoption argument Q3 gave. A large base forces bundles and profiles to remove. `remove` is the operation with dependent-key deletion, and it has no specified answer when the upstream target is already gone. The [Q2 walkthrough](schema-format-walkthrough.md) left that failure mode open.

**A bundle declares its closure, and the closure is checked at publish time.** The dependency column above is data, not documentation. `compliance` requires `standards` because a control that complies with nothing is a shape with no meaning.

Abstract kinds pay for themselves here, and the payment was not predicted. They arrived as defect 2 of the Q2 walkthrough, to stop a new kind from editing every relation endpoint. In the base, `governed_document` means that a bundle's new kind inherits the four facets and joins `supersedes`, `governs`, and `traces_to` with no endpoint edit anywhere. Without the abstract kind, every bundle would have to `override` three relation endpoint lists, and no bundle could be add-only. The rule that makes bundles composable rests on a primitive that a different walkthrough added for a different reason.

## The starter kit is a selection, not a third option

[Spec 0](../spec/00-vision-and-scope.md) promises a doctrine starter kit: an opinionated default taxonomy and the prose that explains it. Spec 2 refers to a `headwater/standard` base package. Q3 reads as if these are one artifact, and they cannot be. The base has to be minimal so that bundles stay add-only. The starter kit has to be opinionated so that a new adopter does not face a blank schema.

They are two artifacts over one mechanism. `headwater/starter` is the base, a bundle selection, and the doctrine prose that explains the selection. It is what `headwater init` produces when the adopter accepts every default. It is exactly Q3's batteries-included option, expressed as a selection rather than as a base.

This also settles a smaller inconsistency. Spec 2 states that the doctrine starter kit declares `en-US` with the STE house profile. A controlled-language profile that the *base* turns on meets an adopted corpus of two hundred documents with a wall of findings on the first run. That is the outcome Q3 exists to avoid. The declaration belongs to the starter kit, which is where spec 2 put it, and the base declares `controlled: none`. The two artifacts were never in conflict. The names were.

## Five first runs

Each run asks the same two questions. How many lines does the adopter author before the first check, and how many lines does the adopter delete? The counts come from the drafted package above and are honest estimates of authored YAML, not measurements from a running tool.

**A — solo maintainer, empty repository.** Takes `headwater/starter` whole. `headwater new decision` works in the first minute. Authors nothing, deletes nothing.

**B — small team, sixty documents, no conventions.** `headwater infer` ([Q12](../spec/09-open-questions.md#q12--migration-path-for-an-existing-corpus)) reads the tree and proposes three shelves from three directories. The interview asks which of the three hold how-to material, and selects `procedure`. The adopter authors about twelve lines, all of them shelf paths, and deletes nothing. Every document lacks `status_since`, so windowed expectations skip with reason `missing-origin` and the corpus stays honest while the missing facet is its own finding.

**C — product suite, four hundred documents, an ADR numbering scheme already in use.** Selects `procedure`, `standards`, `evidence`, and `proposals`. Two lines override the identifier pattern so that inference does not renumber anything that already exists. About thirty authored lines, no deletions.

**D — regulated platform.** Selects `compliance`, which pulls `standards` and `evidence` through its closure. One line enables the bundle. About fifteen more declare the mapping to the external control framework. No deletions, and no engine change, which is the test spec 2 sets for the third column of its worked example.

**E — Headwater's own corpus.** [Principle 8](../spec/00-vision-and-scope.md#design-principles) makes this a real test rather than a courtesy, and it is the run that produced the most.

The base plus the `evidence` bundle covers it. `docs/spec/` holds specification documents, `docs/evaluations/` and `docs/reviews/` hold evidence documents, and `docs/reviews/` binds the narrative voice regime because those files are point-in-time records. The language regime absorbs the American-spelling ruling and the STE house profile with no authored line, because the starter kit already declares both. That ruling currently lives in an instruction file, which is the failure spec 2 confesses in its own words: configuration expressed as convention. About twenty-two authored lines, no deletions.

Two findings came out of that run, and neither is fixed here.

**The open-questions register is one document that contains eighteen documents.** Each entry has a state, and three of them are closed. Each entry has a state-entry date that the prose states and no facet records. Entries link to each other: Q13 blocks Q1, and Q11 affects Q3 and Q7. Entries cite their evidence: Q1 cites the language evaluation and the spike results. Read structurally, every open question is a `decision` in the `draft` state, closing one is the ordinary transition to `current`. The blocking lines are `constrains` edges, and the citations are `traces_to` edges. The corpus needs no new kind for its most-edited artifact, which is real corroboration for a two-kind base. But it cannot express that artifact today, because the artifact is a single file.

**The specification documents serve two purposes at once.** Spec 2 states a model and argues for it in the same file. Spec 2 also states that a kind serving two unrelated purposes is a signal to split the kind. The repository is most of the way to its own answer already. `docs/evaluations/` carries the argument, `docs/spec/` carries the result, and the revision note at the top of each spec is the seam between them. What is left is the argument that stayed inline.

### What the counts show

| Adopter | Authored lines | Deleted lines | Deleted under a batteries-included base |
|---|---|---|---|
| A — solo | 0 | 0 | 4 shelves and their dependents |
| B — small team | ~12 | 0 | 2 shelves |
| C — product suite | ~30 | 0 | 0 |
| D — regulated | ~16 | 0 | 0 |
| E — Headwater | ~22 | 0 | 3 shelves |

The right-hand column is the whole argument in one place. A batteries-included *base* costs three of five adopters a first experience that consists of `remove:` lines. Each one of those removals is the overlay operation with the most failure modes. Spec 2 reached the same conclusion for relations and stated the reason plainly: friction spent to delete things that nobody asked for. The bundle set makes the same ruling for shelves and kinds, and the confluence property makes it a guarantee instead of a preference.

## The interview

Q3's leaning says the interview matters more than the packages, and the runs above agree. Adopter A types nothing and adopter D types one line, so the packaging is nearly free in both directions. What is not free is the moment between them. Five requirements fall out of the runs, and each one is derived rather than preferred.

**It emits an overlay, never a resolved taxonomy.** A tool that writes a complete taxonomy file has forked the adopter from the base before they write their first document. Every later upgrade then becomes a merge. Spec 2's requirement list says customization by overlay and never by fork. The command that creates the first taxonomy is the one place where breaking that rule would be invisible.

**It is declared as package data, not written as engine code.** [Principle 1](../spec/00-vision-and-scope.md#design-principles) says that anything an adopter might reasonably want different belongs in the schema. An interview compiled into the engine cannot be shipped by a third-party publisher, and a publisher who defines their own bundles needs their own questions. The interview is a package artifact beside profiles and templates. It is not a fourteenth taxonomy declaration, because it describes the package rather than the corpus. The count of declarations stays at thirteen.

**It is `headwater infer` with a second evidence source.** Q12 makes `infer` propose a taxonomy from an existing tree. Both commands emit the same artifact, so they are one command with two inputs: what the tree shows, and what the answers say. On an empty repository the tree contributes nothing and the interview asks everything. On adopter C's four hundred documents the tree answers most of it and the interview settles what the tree could not. One code path, and the blank-schema case is the degenerate one rather than the special one.

**Every question is about the corpus, and no question is about the taxonomy.** "Do you write runbooks?" needs no model in the reader's head. "Do you want a `procedure` purpose?" needs the whole of spec 2 first. Abstraction gradient decided Q2, and it decides the question set here. Each answer selects a bundle, and no answer exposes a declaration name.

**A question that an existing ruling answers is deleted rather than asked.** Spec 3 rules that identifiers are always namespaced, because retrofitting a namespace is expensive, so the interview never asks whether the adopter wants identifiers. Applied across the question set this removes most of what a first draft would ask. It is the cheapest way to keep the interview short.

The bound on length follows from the same test. The interview asks only what changes the bundle selection, and everything else waits until a corpus exists for `taxonomy audit` to measure. A day-one guess about facet orthogonality is worse than a day-thirty measurement of it, and the audit already reports the distributions.

**The honest cost is the one overlays already carry, one level up.** The resolved taxonomy is an artifact that nobody authored directly, and an interview adds a step where nobody authored the *answers* as configuration either. Spec 2 names the mitigation for the first case: `explain`, `resolve`, and a readable lock. The second needs one more thing, and it is cheap. The generated overlay carries a comment above each block that names the question and the answer that produced it. YAML preserves comments through a round trip, which the Q2 walkthrough scored as secondary notation. Re-running `init` re-asks with the current answers as defaults and rewrites the same blocks. An adopter who changes their mind edits an answer, not a taxonomy.

## The decision

**The base package is minimal and derived from the core.** Two concrete kinds under one abstract kind, four facets, five relations, one anchor, two shelves. It is not a taxonomy anyone is expected to run bare, and it is the thing that everything else composes over.

**Optional content ships as bundles.** A bundle is a publisher-shipped, add-only overlay with a declared dependency closure. It is the profile mechanism pointed the other way, and it is not a new concept.

**The doctrine starter kit is a named bundle selection over the base, plus the prose that explains the selection.** That is Q3's batteries-included option, and it is what an adopter gets when they answer nothing.

**The interview composes a selection, and it is `infer` with a second evidence source.** It emits an overlay, it is declared in the package, its questions are about the corpus, and it is bounded to what changes the selection.

Q3 presented three options as alternatives. All three survive as layers, and the layering is what makes each one correct. The minimal core is the base. Batteries-included is a selection. The interview reaches every other selection. What does not survive is the framing that made them compete.

## Consequences for the specification

| # | Finding | Landed in |
|---|---|---|
| 1 | The core requires the `behavior` purpose, and the smallest column of the worked example does not serve it. | [Worked example](../spec/02-taxonomy-model.md#worked-example-three-taxonomies-one-engine) |
| 2 | All four default relations run between decisions, so no default edge attaches the corpus to code. And a behavior kind added to that base is a permanent orphan. Enable `governs`. | [The decision-relation vocabulary](../spec/02-taxonomy-model.md#the-decision-relation-vocabulary) |
| 3 | "Creatable by scaffold, generator, or hook" is false for two of the default four. The rule that does the work is that no base relation is `created_by: author`. | [Who creates each edge](../spec/02-taxonomy-model.md#who-creates-each-edge) |
| 4 | Enabling by reference does not generalize past relations. Optional content ships as add-only bundles with declared closures, and add-only is what makes any subset resolve. | [Defined and enabled](../spec/02-taxonomy-model.md#the-decision-relation-vocabulary) and [spec 7](../spec/07-distribution-and-federation.md#bundles-are-publisher-overlays-in-the-other-direction) |
| 5 | The base package and the doctrine starter kit are two artifacts, and spec 0 and spec 2 name them as if they were one. | [Spec 0 item 6](../spec/00-vision-and-scope.md#what-we-build) and [spec 7](../spec/07-distribution-and-federation.md#the-starter-kit-is-a-selection) |
| 6 | The interview is package data rather than engine code, and it is `infer` with a second evidence source. | [Spec 7](../spec/07-distribution-and-federation.md#the-interview) and [Q12](../spec/09-open-questions.md#q12--migration-path-for-an-existing-corpus) |

Findings 1, 2, and 3 have one cause between them. The smallest column of the worked example was drawn as an impression of a small corpus. It was never derived from the core beside it. To write the base package out as YAML is what makes the three visible at once. None of them is visible from reading the prose.

## What this walkthrough did not settle

**The bundle set is provisional.** Six bundles is a guess about how adopters cluster, and no adopter exists yet. The set is data in a package, so it costs a release rather than an engine change. The first real adopter is the evidence that revises it.

**The base kind names are placeholders that carry weight anyway.** The core is semantic, so `specification` renames freely. But a name in the starter kit is what every new adopter reads first, and `specification` is heavier than what a solo maintainer writes. That is a doctrine question, and doctrine ships in the same package.

**Two findings against this repository's own corpus stay open.** The open-questions register cannot be expressed while it is one file, and the specification documents serve two purposes at once. Both are ordinary corpus work rather than design defects, and both are the kind of thing that the system exists to report.

**The license half of Q3 stays with [Q11](../spec/09-open-questions.md#q11--license-and-distribution-posture).** Whether the base, the bundles, and the doctrine ship under the terms of the engine is a licensing decision. Nothing here depends on the answer.
