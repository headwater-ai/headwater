---
id: EVAL-HW-schema-format-walkthrough
status: current
status_since: 2026-08-10
last_verified: 2026-08-10
summary: The Q2 evidence, a cognitive-dimensions walkthrough over five authoring scenarios, which chose YAML and found five defects in spec 2.
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
    - SPEC-HW-taxonomy-model
    - SPEC-HW-theoretical-foundations
    - SPEC-HW-adjacent-work
---

# Choosing the schema format — a cognitive-dimensions walkthrough

Evidence for [Q2](../spec/09-open-questions.md#q2--schema-format). Q2 specified the method and left the work undone: walk each candidate notation through five authoring scenarios, and score it on the cognitive dimensions. This is that walkthrough.

It produced three kinds of result, and they are worth separating before the detail.

**A decision.** YAML is the concrete syntax. The schema language, the reference sublanguage, the overlay language, and the meta-schema are Headwater's. JSON Schema is an emitted export and never the validator.

**A correction to Q2's own prediction.** Q2 forecast that viscosity and hidden dependencies would decide it. They did not. The three candidates score near-equal on both, because most of the viscosity is in the *model* and every notation inherits it. What decided it was premature commitment, abstraction gradient, and a dimension that the 1996 framework does not have.

**Five design defects that have nothing to do with notation.** A scenario walkthrough finds what an argument about syntax cannot. All five are recorded at the end, with the spec section that owns each.

## The question decomposes into three

Before the walkthrough could start, the question had to be split. "Schema format" bundles three decisions that have different answers.

1. **Who owns the schema language?** That is, who defines what a taxonomy may say, and what validates it.
2. **What concrete syntax do authors type?**
3. **What can a partially-written taxonomy be checked against, and when?**

[Q13](../spec/09-open-questions.md#q13--linkml-and-shacl-as-substrate) already answered the first: Headwater owns the language, and standard formats are emitted from it. Q2 is really the second and the third.

The separation matters because the leaning as written — "YAML plus JSON Schema for the authored surface" — reads as an answer to the first question when it is not one. [Spec 2](../spec/02-taxonomy-model.md) already contains a Headwater sublanguage that no YAML feature and no JSON Schema keyword can express:

- `values: $vocabularies.lifecycle_state` — a reference into another declaration.
- `add: {relations.forbids: $package.optional.forbids}` — a reference into a package's unenabled definitions.
- `override: {shelves.decisions.path: docs/adr/**}` — a dotted path address into the resolved tree.
- `since: state_entered` — a name drawn from a closed role registry that lives in the meta-schema.

JSON Schema can check that a `$`-reference is *shaped* like one. It cannot check that it *resolves*. It cannot check that an overlay address exists, that a role is unique, or that the overlay set commutes. Every one of those is in spec 2's meta-schema list. So the authored surface was never "YAML plus JSON Schema". It was always YAML plus a Headwater resolver, with JSON Schema covering the part that a text editor can highlight.

This is the same finding that Q13 reached from the other end, and the convergence is the strongest evidence in this document. Q13 found that LinkML and SHACL validate one instance against a shape, and that everything else Headwater does is a property of the whole graph. Q2 finds that JSON Schema validates one document against a shape, and that everything else a taxonomy declares is a property of the whole document. Two questions, opposite directions, one architecture.

## The candidates

| | Authoring surface | Meta-schema | Overlay mechanism |
|---|---|---|---|
| **A. YAML** | YAML 1.2 | Headwater's, with JSON Schema emitted | Headwater path-addressed patches |
| **B. Typed configuration language** | CUE (representative), Dhall, KCL | the language's own type system | the language's own composition |
| **C. Purpose-built DSL** | new syntax | Headwater's, native | Headwater's, native |

CUE represents option B in the detail below, because it is the strongest of the three for this purpose. Dhall and KCL are treated where they differ, and they differ in one place that matters.

## The dimensions

Q2 listed seven. Green and Petre's framework has fourteen, and the seven were a reasonable first cut. Three of the omitted seven earn a place here, and one dimension that the framework does not have earns a place because the framework predates the reader that this project is built for.

**Closeness of mapping.** How directly does a notation express the eleven declarations? This is separable from role-expressiveness, which asks whether a *reader* can tell what a part is for.

**Secondary notation.** What can an author write that the notation does not act on? Comments carry the reason for a declaration. A format that loses them loses the reason.

**Juxtaposability.** Can a reader see two things at once? Base beside overlay, and old beside new, are the two comparisons this system asks for constantly.

**Model-writability.** A new dimension, and the only one invented here. It has two halves. Can a language model write the notation correctly with no examples in front of it, which is a question about how much of the notation is in its training data? And can a program edit the notation structurally, and preserve what it did not touch? Both are requirements rather than preferences. [Spec 5](../spec/05-ai-integration.md) makes agents readers of the corpus. [Q12](../spec/09-open-questions.md#q12--migration-path-for-an-existing-corpus) makes `headwater infer` a program that *writes a taxonomy*. A notation that only humans can author has already failed a stated requirement.

Provisionality — can an author sketch something not yet valid — is folded into progressive evaluation below, because for this notation the two questions have one answer.

---

# Scenario 1 — add a kind

Add `playbook` to the standard taxonomy. It serves the `procedure` purpose, it lives on its own shelf, it can be superseded, and it can govern code paths.

## Edit sites, as spec 2 stands

| # | Site | Why it is required |
|---|---|---|
| 1 | `kinds.playbook` | the concept itself |
| 2 | `shelves.playbooks` | the meta-schema requires every kind to be reachable from a shelf |
| 3 | `relations.supersedes.from` and `.to` | relation endpoints enumerate kinds |
| 4 | `relations.governs.from` | the same |
| 5 | `kinds.playbook.relations.may` | the same fact as 3 and 4, spelled in the other direction |
| 6 | `projections[shelf_index].for` | if the new shelf is indexed |

Six sites. One is the concept. Five are ceremony, and four of the five are somewhere other than the thing being added.

## What the scenario found about the design

**The kind-to-relation permission is declared twice, in opposite directions, and nothing makes the two agree.** `relations.supersedes.from: [decision]` and `kinds.decision.relations.may: [supersedes]` determine the same set of permitted pairs. The meta-schema checks referential integrity — that each name resolves — and does not check that the two directions agree. A taxonomy where `supersedes.from` includes `playbook` while `kinds.playbook.relations.may` omits it passes validation, and the relation is unusable.

This is the failure that spec 2 forbids two sections earlier, for the homogeneous-shelf discriminator: "a second truth that will eventually disagree with the first". The rule is right and it was not applied to itself.

Endpoints should be authoritative. Family, nuclearity, reciprocity, cardinality, and `created_by` all live on the relation, and the endpoint lists are what the generated Graph checks read. `may:` on a kind should be derived. The obvious objection is that a reader asking "what is a decision?" wants the relation list on the kind. That is a presentation need, and `headwater explain` already promises to print which relations are permitted. To keep a declaration in order to serve a reading need is exactly how a second truth gets in.

**`abstract` exists in the meta-schema with no semantics anywhere.** Spec 2's coverage rule reads: "Every kind is reachable from at least one shelf, or it is explicitly marked abstract." Nothing else in the specification says what an abstract kind is, what declares one, or what it is for.

Sites 3, 4, and 5 are what it is for:

```yaml
kinds:
  governed_document:
    abstract: true
    facets:
      require: [status, status_since, last_verified, summary]
  playbook:
    is_a: governed_document
    purpose: procedure
```

With `relations.supersedes.from: [governed_document]`, sites 3, 4, and 5 disappear. Adding a kind becomes two or three sites, and all of them are about the kind.

Three constraints keep this cheap. An abstract kind stays rigid, so the rigidity rule is untouched — inheritance between rigid kinds is ordinary, and the rule forbids an anti-rigid class subsuming a rigid one. An abstract kind may never be a shelf's declared kind or a discriminator value, which is one new meta-schema rule. And kind resolution is unchanged, because it resolves to concrete kinds only.

## What the scenario found about the notations

| | Result |
|---|---|
| **YAML** | JSON Schema checks the shape of each of the six sites and the agreement of none. Four of the six are referential and need the engine. An editor with a schema gives red marks while typing for shape errors, and nothing for meaning errors until `taxonomy validate` runs. |
| **CUE** | Referential integrity is expressible in the notation. `may` can be constrained to a disjunction built by comprehension over the declared relations, so a misspelled relation name fails at the point of the typo. This is a real gain over JSON Schema and it is CUE's second-best showing. |
| **DSL** | Can make the redundancy syntactically impossible by giving the permission exactly one home. So can deleting it from the model. |

That last line is the theme of the whole walkthrough. **The worst viscosity in three of five scenarios comes from the model, not from the notation. No syntax fixes it, and every syntax inherits it.**

---

# Scenario 2 — rename a shelf

Rename `shelves.decisions` to `shelves.adr`, and change its path from `docs/decisions/**` to `docs/adr/**`.

## Edit sites

Inside the schema, a shelf key is referenced in one place: `projections.*.for`. The core is semantic and names no shelves. Mappings name kinds and facet values. Kinds do not reference shelves — shelves reference kinds. So the in-schema cost is trivial, which is the correct design and worth stating.

Outside the schema, the cost is unbounded. **Every consumer overlay that addressed `shelves.decisions.*` breaks.** And the corpus directory moves, which is a change to every document's path.

## What the scenario found about the design

**Overlay path addresses are a published interface, and the compatibility dimensions do not measure them.**

Walk the rename through spec 2's five dimensions. `classification`: every document still resolves to the same kind. `instance_validity`: every document still validates. `consequence`: every check that passed still passes. `projection`: identical output, once `for:` is updated in the same release. `identifier`: unchanged. All five can report compatible, and the release is a minor version.

Meanwhile every downstream overlay that says `override: {shelves.decisions.path: ...}` fails to resolve, because `override` requires the addressed path to exist. The failure is loud, which is right, and it lands on the consumer with no warning from the publisher. The publisher measured against its own reference corpora, and its reference corpora have no overlays.

A sixth dimension closes this: **`addressability` — does every path that an overlay can address still exist and mean the same thing?** It is the only dimension whose subject is the schema rather than the corpus, which is why it was missed. The other five are all corpus-measured, and the publisher's blind spot is exactly the artifact that is not a corpus.

The honest limit is the same limit the other five have. A publisher cannot enumerate every path a consumer might address, so the dimension is measured against reference overlays, in the way that the others are measured against reference corpora. That converts a silent break into a claim the consumer can verify locally, which is what spec 2 already says a consumer's own run is for.

**A second, smaller finding: renaming the key and moving the path are different changes with different consequences, and the overlay syntax makes them look alike.** `override: {shelves.decisions.path: docs/adr/**}` moves a directory, which moves every document. `override` of the key itself is a schema-surface change with no corpus effect. A reviewer sees two similar lines. `taxonomy diff` should report them under different dimensions, which the new dimension makes possible.

## What the scenario found about the notations

| | Result |
|---|---|
| **YAML** | An overlay address is a dotted string in a string position. Nothing checks it until resolution. A rename is a find-and-replace with no compiler behind it. |
| **CUE** | Inside one build, a reference is a language-level identifier and a rename is an error at every reference site. Across a package boundary, a downstream overlay's address is a string in a published artifact again. The advantage shrinks to nearly nothing at exactly the distance where the problem lives. |
| **DSL** | Identical to YAML, unless it ships a rename-aware migration tool. That is a tool, not a syntax, and it can be built for YAML. |

---

# Scenario 3 — split one facet into two

Split `audience` into `audience_role` (engineer, operator, integrator, auditor) and `audience_expertise` (novice, practitioner, expert). The ascertainability canon is what surfaces the need: no guidance for `auditor` can say when it applies without saying "unless you mean the expertise question".

## Edit sites

| # | Site | Count |
|---|---|---|
| 1 | `vocabularies.audience` becomes two value sets | 1 → 2 |
| 2 | `facets.audience` becomes two facet declarations | 1 → 2 |
| 3 | every `kinds.*.facets.require` or `optional` that names it | one per kind |
| 4 | any `expect.when` condition that reads it | zero or more |
| 5 | every projection or routing rule that reads it | at least one, because the relevance canon guarantees a reader |
| 6 | every document that carries it | the corpus |

## What the scenario found about the design

**Facet requirements are enumerated per kind, so the cost of any facet change is linear in the number of kinds.** This is the same amplifier that scenario 1 found in relation endpoints, and the same missing primitive fixes both. Two independent scenarios converging on one absent abstraction is the strongest single result this walkthrough produced, and spec 2 anticipated the shape of it: "the schema lacks a primitive — and the fix is a new primitive, not a special case".

**The split is a major version by `instance_validity`, and the judgment half of its migration payload is the whole of it.** Mapping one old value onto two new ones is mechanical only when the old vocabulary was a product of the two. `senior_engineer` splits mechanically. `auditor` does not. Spec 2 already divides the payload into a mechanical half and a judgment half, so the machinery is correct. What the scenario adds is a requirement on the payload format: it must be able to say *one old value maps to a set, and the author chooses*. That is a task with a closed choice attached, and it is far more useful than remediation prose.

## What the scenario found about the notations

All three notations require the same edits, because the repetition is in the model.

CUE wins one real thing here, and it is instructive. A definition holding the shared facet set makes the split one edit, and unification propagates it. But that is CUE *simulating the primitive that the model lacks*. If the primitive is added, the advantage disappears. **A notation feature that compensates for a missing model primitive is the worst available reason to choose a notation, because it hides the modeling debt behind syntax.**

And the corpus edit — site 6 — is identical under all three notations, and it is larger than the schema edit by two orders of magnitude. Any notation argument about this scenario is an argument about the smaller half of the work.

---

# Scenario 4 — add a relation type to an existing family

Two cases, and the design already made them deliberately different.

**(a) Enable a relation the package defines.** One line: `add: {relations.forbids: $package.optional.forbids}`. This is the cheapest operation in the system, and spec 2 designed it that way so that a minimal default stays nearly free for an adopter who needs more.

**(b) Declare a genuinely new relation.** `mitigates`, governance family, from `control` to `risk`. That requires family, endpoints, `created_by`, cardinality, a nuclearity decision, the `may:` sites from scenario 1, and optionally an expectation with a window, an origin, and a rationale.

## What the scenario found about the design

**The governance and evidence families have no default nuclearity, and the reading-precedence derivation has no clause for them.**

Spec 2 derives reading precedence in three cases. On nucleus–satellite, the nucleus governs. On succession, the successor governs. Other multinuclear relations carry no reading order. A governance relation between two documents falls in none of these. It has no declared nuclearity, because the family table leaves that cell blank, and the prose explains why: a governance edge often ends on an anchor, and an anchor carries no nuclearity.

But `constrains` runs from one document to another, and the family's stated meaning is that the source constrains the target. If the default is "no reading order", the derivation contradicts what the family means. Reading precedence is used in three places — the order of routing results, the document a conflict is reported against, and reading order in generated indexes — and a governance edge needs an answer in all three.

The cut `dominance` field carried exactly this. The core-concepts review removed it as "redundant where nuclearity or succession already determined it, and unused where they did not". It was redundant for succession. It was not unused for governance, and the pre-cut artifact shows it: the [LinkML worked example](linkml-worked-example.md) carries `headwater:dominance: source` on `governs`, where nothing else supplies the order.

The fix is a third derivation clause rather than the restored field: **on a governance relation between two documents, the source governs the reading.** That is what the family means, so nothing needs to declare it.

**`created_by` is a deliberate premature commitment, and the framework is what makes that visible.** It is required at declaration time, and it is a claim about a workflow that may not exist yet. Spec 2 defends the cost explicitly — the field forces the question *what creates this edge, and who pays* at design time. Scoring it as a cost and accepting it is the correct outcome. An expectation's required `rationale` is the same trade, made the same way. The value of the framework here is not that it found a problem. It is that it separated a deliberate cost from an accidental one.

## What the scenario found about the notations

Case (a) is the decisive observation for the whole question, and it is easy to miss because it is one line. `$package.optional.forbids` is not YAML. It is a Headwater reference into a package's unenabled definitions, resolved by a Headwater pass, and a published JSON Schema will type it as a string that starts with a dollar sign.

| | Result |
|---|---|
| **YAML** | The reference sublanguage already exists and already needs an engine. Admitting that is the whole architectural decision. |
| **CUE** | Expresses (a) natively, because selecting a definition from a package is what unification and definitions are for. This is CUE's strongest showing in the walkthrough. |
| **DSL** | Could make (b) a form with required slots and the best error messages available. A good meta-schema does the same with worse messages and no new parser. |

---

# Scenario 5 — upgrade across a major version with a live overlay

Base `headwater/standard` moves 2.1.0 to 3.0.0. The consumer runs overlay `acme-engineering` over a corpus of 400 documents, and the corpus stays live throughout.

## The sequence

1. Fetch 3.0.0.
2. `taxonomy diff --to 3.0.0` against the consumer's own corpus, reporting per dimension.
3. Re-resolve the overlay against the new base.
4. Run the migration payload: mechanical steps by `migrate --apply`, judgment steps as a task list.
5. The corpus sits in the migration state meanwhile, with `migration-pending` findings.
6. Check confluence over the overlay set, statically, before applying anything.
7. Check core satisfaction on the resolved result.

## What the scenario found about the design

**Step 3 can break in three ways, and spec 2 names one.**

- An `override` path no longer exists. Spec 2 makes this an error, loudly. Correct.
- **An `add` key now exists upstream.** Base 3.0.0 adds `shelves.playbooks`, and the consumer's overlay added its own `shelves.playbooks` in 2.1.0. `add` requires the key to be absent, so resolution fails. That is correct and the remedy is unspecified. Switching `add` to `override` resolves cleanly and silently inherits every upstream field the consumer did not restate. **The rule should be stated: a collision is always a task for a human, never an automatic promotion to `override`.**
- A `remove` target is already gone upstream. Spec 2 does not say whether that is a no-op or an error, and the choice decides whether a consumer's overlay survives an upstream deletion.

**The migration payload migrates documents. Nothing migrates overlays.** The payload declares what moved, what was renamed, and what must be re-stated, and every word of that was written for documents. The same payload can drive an overlay rewrite, and it should. The consumer's overlay is the artifact most likely to break and least likely to be tested. `migrate --apply` should rewrite overlay addresses from the payload's rename map, and report each `add` collision as a task that shows both definitions side by side. That is juxtaposability, applied where it earns the most.

## What the scenario found about the notations

This scenario decided the question.

**CUE cannot express overlays, and the reason is structural rather than a gap to be filled.** CUE's unification is a greatest lower bound over a lattice: monotone narrowing, commutative, idempotent. Commutativity and idempotence are exactly the confluence property that spec 2 demands, which makes CUE look like the ideal fit. It is ideal for `add`. It cannot express `override` and it cannot express `remove`. Unifying `path: "docs/decisions/**"` with `path: "docs/adr/**"` yields bottom, not a rename.

The standard workaround is a disjunction with a default, `path: *"docs/decisions/**" | "docs/adr/**"`, which requires the base to pre-enumerate every value a consumer might choose. That is premature commitment in the framework's exact sense, and it lands on the party least able to bear it: the publisher, who would have to guess every adopter's directory layout in advance. CUE's headline feature is unusable for two of the three overlay operations, and the workaround fails a named dimension.

**Dhall can express override, and that is what disqualifies it.** Dhall's record-merge operators give override directly, and an overlay is naturally a total function from taxonomy to taxonomy. Totality means it terminates, so it is safe to run.

But spec 2 requires the resolver to check confluence **statically, before it applies anything**, by building the set of paths that each overlay addresses. For a simple Dhall overlay the normal form does reveal the fields it touches. As soon as the overlay branches on its input, the address set becomes input-dependent and no static reading is available. So the resolver either accepts a restricted subset — at which point it is a path-addressed patch language wearing Dhall syntax — or it gives up static confluence. **A statically checkable confluence property requires the overlay's address set to be readable off its syntax.**

There is a second objection and it is independent. Spec 2 forbids conditionals and user-defined functions in the taxonomy language, and says the boundary is defended. Dhall's contribution is functions. To adopt it means either not using the feature that distinguishes it, or breaking a stated limit.

KCL has real control flow and mutation, so the confluence argument applies with more force, and its ecosystem is smaller. The disqualification does not improve on closer reading, and saying so is more honest than a longer section that reaches the same place.

**The DSL gains nothing here.** It could give the overlay language exactly the operations it needs, and a `rebase` verb for step 3. The Headwater overlay language already has those operations in YAML, because the overlay language is Headwater's under every candidate. A DSL would add syntax for semantics that do not change.

This is the walkthrough's most useful structural result. It does not merely prefer the overlay design that spec 2 sketched. **It derives it.** A first-order, path-addressed patch language with `override`, `add`, and `remove` is the only shape that supports static confluence, and static confluence is what makes order-independence a guarantee instead of a hope.

---

# Scoring

| Dimension | A. YAML | B. CUE | C. DSL |
|---|---|---|---|
| Viscosity | equal — the cost is in the model | equal, with a definition mechanism that masks the missing primitive | equal |
| Hidden dependencies | overlay addresses are unchecked strings | the same across a package boundary | the same |
| Premature commitment | low, and what remains is deliberate | **high** — override needs a pre-enumerated disjunction | low |
| Role-expressiveness | moderate — eleven named declarations carry it | good — definitions read differently from fields | **best** |
| Error-proneness | YAML's own traps, and a strict loader removes them | **best** — closed structs and in-language referential integrity | good in principle, untested in fact |
| Progressive evaluation | shape in the editor, meaning at `taxonomy validate` | **both in one tool** | whatever gets built, and nothing on day one |
| Abstraction gradient | **lowest** — no new language to learn | **highest** — lattice, closedness, definitions, comprehensions | high, and idiosyncratic rather than transferable |
| Closeness of mapping | good — declarations map onto the eleven concepts | moderate — concepts arrive dressed as CUE constructs | **best** |
| Secondary notation | comments, and they survive a round trip | comments, and they survive | as designed |
| Juxtaposability | base and overlay diff in every tool a reader already has | the same | the same, minus the tools |
| Model-writability | **best** — models write YAML fluently, `infer` emits it, editors round-trip it | poor — thin training data, and generated CUE loses its comments | **worst** — no training data exists at all |

Q2 predicted that viscosity and hidden dependencies would decide it. They came out near-equal across all three, for the reason that runs through every scenario: the expensive part is the model. The dimensions that actually decided it were premature commitment, which rules out CUE on scenario 5; abstraction gradient and progressive evaluation, which rule out the DSL on its first day; and model-writability, which the framework does not contain.

That is worth recording as a result about the method. The framework earned its place by contradicting the prediction that chose it.

# The decision

**YAML 1.2 is the concrete syntax. The schema language, the reference sublanguage, the overlay language, and the meta-schema are Headwater's. JSON Schema is emitted and never authoritative.**

The relationship between Headwater's meta-schema and the emitted JSON Schema is exactly the one Q13 established for LinkML and SHACL, for the same reason and with the same obligation. The export declares itself a subset. An external validator that reports a clean run while believing it checked everything is worse than one that knows what it skipped.

## The loader rulings

YAML's traps are the error-proneness cost of this choice, and a strict loader removes nearly all of them. These are rulings, not suggestions.

- **YAML 1.2 core schema only.** No YAML 1.1 type resolution. `no` is the string `no`, not the boolean false, which matters the first time a taxonomy carries a country code or a two-letter language subtag.
- **Duplicate keys are rejected.** Most loaders take the last one silently.
- **Anchors and aliases are forbidden in taxonomy sources.** The `$`-reference is the sanctioned reuse mechanism, and it is addressable by an overlay and attributable by `explain`. An alias would be a second reuse mechanism that no overlay path can address. That is the second-truth rule, applied to the notation instead of to the model.
- **Merge keys are forbidden**, for the same reason, and they are a YAML 1.1 extension in any case.
- **Scalar types come from the meta-schema, never from the resolver.** A version reads as a string because the meta-schema says it is one.

## The implementation cost, and why it is already retired

Choosing YAML in a Rust engine has one obvious cost, and [Q1](../spec/09-open-questions.md#q1--implementation-language) named it while closing: the YAML crate ecosystem is in poor repair. The reason it matters less than it looks is the same reason in both entries. [Spec 12](../spec/12-check-layer.md) requires findings to anchor to a line, and no convenient deserializer retains spans. The engine writes its own parse whatever format it reads.

The [language spike](language-spike-results.md) built that parse for document front matter, with ten assertions covering line and column for every key. A taxonomy file is the same problem at a different scale. So the format decision here adds no implementation risk that the engine had not already accepted.

This is also the reason the loader rulings above cost nothing to enforce. A parser that the engine owns can reject a duplicate key, refuse an anchor, and take a scalar's type from the meta-schema. A parser that the engine merely calls can do none of those.

## The lock is a different artifact with different criteria

Q2's leaning said the resolved lock belongs in "a stricter representation" without saying what strict means. The lock is generated, so authorability is not one of its criteria. Determinism and diffability are.

Strict therefore means a **canonical serialization**, not a typed configuration language: JSON with sorted keys, exactly one representation of each value, every reference resolved, and a content hash over the bytes. YAML admits several spellings of one value, and that alone disqualifies it for a hashed artifact. The lock carries the resolved taxonomy, the base and overlay identities and versions, the measured compatibility result, and the address map that overlay migration reads.

## CUE as an optional front-end

Q2 asked whether CUE could be an optional authoring front-end. It can, in one direction and outside the engine. An organization that wants CUE writes CUE, runs its own build, and commits generated Headwater YAML. The engine never reads CUE.

Two reasons hold that line. The engine keeps one input format, so there is one thing to validate and one thing to explain. And the committed artifact stays what review, blame, and the pre-commit hook all see.

This costs Headwater nothing, because it is a consequence of the format being ordinary YAML rather than a feature to build. What Headwater does owe such an organization is the JSON Schema emission that Q13 already schedules first. That is what makes a CUE, TypeScript, or Python front-end possible with no Headwater dependency at all.

# Consequences for the specification

Each of these is a design defect that the walkthrough surfaced. None of them is a notation choice, and none is fixed by changing syntax. All five are now applied, and the last column says where each landed.

| # | Finding | Landed in |
|---|---|---|
| 1 | Kind-to-relation permission is declared twice and nothing makes the two agree. Endpoints become authoritative and `may:` is derived. | [Endpoints are the only permission](../spec/02-taxonomy-model.md#endpoints-are-the-only-permission) |
| 2 | `abstract` appears in the meta-schema with no semantics. Define abstract kinds and `is_a`. Scenarios 1 and 3 both need exactly it. | [Abstract kinds](../spec/02-taxonomy-model.md#abstract-kinds) |
| 3 | Reading precedence has no clause for the governance family, and the derivation contradicts what the family means. On governance between two documents, the source governs. | [Reading precedence is derived](../spec/02-taxonomy-model.md#reading-precedence-is-derived) |
| 4 | Compatibility has no dimension for the overlay address surface. Add `addressability`, measured against reference overlays. | [Versioning by measured compatibility](../spec/02-taxonomy-model.md#versioning-by-measured-compatibility) |
| 5 | Migration migrates documents and not overlays. The payload drives an overlay rewrite, and an `add` collision is always a human task. | [Customization by composition](../spec/02-taxonomy-model.md#customization-by-composition) and versioning |

Two of them turned out to be half-present rather than absent, which is worth recording because it changes what the fix is. Spec 7 already reported invalidated overlay entries to a consumer, so defect 4 was a missing *consequence* rather than a missing report. And spec 2 already named the `override` precondition, so defect 5 was an incomplete symmetry rather than an unconsidered case. A walkthrough finds these where a reading does not, because it forces the whole sequence rather than one paragraph.

# What the walkthrough did not settle

The `$`-reference sublanguage now has three uses — a vocabulary reference, a package reference, and an overlay address — and no grammar. It has been written three times in three shapes. Before the meta-schema is published, that grammar needs one definition, because it is the part of the authored surface that no external tool will ever check.
