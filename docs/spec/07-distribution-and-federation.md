---
id: SPEC-HW-distribution-and-federation
status: current
status_since: 2026-08-01
last_verified: 2026-08-12
summary: How a publisher ships a taxonomy, how a consumer overlays it, and what the invariant core requires of the result.
doc_type: design_spec
sequence: 7
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: [claude-fable-5, claude-opus-5]
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cites_evidence:
    - EVAL-HW-default-taxonomy-first-run
    - EVAL-HW-first-contact
    - EVAL-HW-graph-export-and-federation
    - EVAL-HW-the-serving-boundary
    - EVAL-HW-warrant-and-adjudication
---

# 7 — Distribution and federation

One organization defines a documentation method. Many repositories adopt it. It evolves. Everyone must be able to take the evolution without loss of what they customized. Also, the publisher must be able to tell who actually did.

## What is shared, and what is not

| Layer | Shared | Owned locally |
|---|---|---|
| Engine | Yes — a versioned dependency | — |
| Taxonomy schema | Yes — a versioned package | Overlays |
| Doctrine (prose that explains the method) | Yes — vendored or linked | Local method notes |
| Corpus content | No | Everything |
| Router / entry point | No — it describes *this* repository | Yes |
| Control register | Partly — the publisher's obligations are inherited | Local controls and waivers |

This distinction makes the problem tractable: **the taxonomy is a package, not a copy.** Prior systems vendored checksummed file trees and gated on byte-identity. That works only while nobody needs to customize. Here, customization is expressed as an overlay against a versioned base. Thus an upgrade is a package bump and a re-resolve, not a merge conflict with a file that you were never supposed to edit.

## Publishing

A publisher repository declares a **taxonomy package**:

```yaml
package: acme/headwater-taxonomy
version: 3.2.0
requires_engine: ">=1.4 <2"
contents:
  taxonomy: taxonomy/
  doctrine: doctrine/          # prose explaining the method, vendored to consumers
  templates: templates/
  plugins: plugins/            # organization-specific checks
profiles: [service-repo, docs-only, platform]   # named overlays that remove
bundles: bundles/            # named overlays that add, selected at init
interview: interview.yml     # the questions init asks, and the bundle each answer selects
migrations: migrations/
```

Publishing is a release: a semantic version, a changelog, an integrity digest, and a migration payload for any major bump. Distribution is over the registry or repository that the organization already uses. The engine requires only that it can fetch a version and check its digest.

**A package imposes nothing on a derived taxonomy, and that is a constraint rather than a courtesy.** An overlay is a patch, so a consumer's resolved taxonomy and lock contain the base content of the package. Terms on the package that a derived work inherits therefore reach an artifact that [spec 0](00-vision-and-scope.md#who-this-is-for) promises is the adopter's own. So the terms of a `taxonomy`, `bundles` or `profiles` path may not condition what a consumer does with the resolved result. The `doctrine` path is prose that a consumer vendors, and it takes its own terms. Headwater checks none of this, and [spec 6](06-engine-architecture.md#what-a-filtered-export-claims-and-what-it-does-not) already states that it checks nothing about a license. What the specification states is the requirement that a publisher must meet ([Q11](09-decisions.md#q11--license-and-distribution-posture)).

## Consuming

A consumer declares what it takes and how it differs:

```yaml
taxonomy:
  package: acme/headwater-taxonomy
  version: 3.2.0
  profile: service-repo
  overlay: .headwater/overlay.yml
```

`headwater taxonomy resolve` fetches, verifies, merges the overlay, validates, and writes the lock. The lock is committed. Thus the corpus is checked against a resolved, reviewable, reproducible taxonomy, and CI needs no network to check anything.

### Profiles are publisher overlays

Not every repository holds every shelf. A profile is a **named overlay that the publisher ships**. It contains `remove` operations for the shelves that a repository archetype does not have, and it is selected by name in the consumer declaration above. It is not a separate mechanism. The overlay resolver already implements every part of it (dependent-key deletion, confluence, core satisfaction on the result).

An earlier draft listed profiles as their own declaration, and thus kept two names for a subset of one mechanism. The effect is unchanged. A repository never has a rule, glob, or projection that targets a shelf that is not present. Dead configuration is noise that teaches readers to ignore configuration.

### Bundles are publisher overlays in the other direction

A **bundle** is a named overlay that the publisher ships, which holds `add` operations for optional content. A profile removes what an archetype does not have. A bundle adds what an adopter needs. One mechanism, two conventional directions, and neither one is new.

```yaml
bundles:
  procedure:  {requires: []}
  standards:  {requires: []}
  evidence:   {requires: []}
  proposals:  {requires: []}
  operations: {requires: [procedure]}
  compliance: {requires: [standards, evidence]}
```

Two rules keep this cheap, and both run on machinery that exists.

**A bundle holds no `override` and no `remove`.** If a bundle needs to change the base, the base declared something that it should not have. Add-only overlays over disjoint addresses commute, so the resolver's static confluence check proves that every subset of bundles resolves. The publisher runs that check once per release, and no adopter can then select a combination that fails.

**A bundle declares its closure, and the publisher checks it at release.** Enabling a bundle is one operation for the adopter, whatever it contains. The dependency list above is data, not documentation.

This is what fixes the size of the base package. A large base forces bundles and profiles to remove, and `remove` carries dependent-key deletion and the most failure modes of the three operations. A minimal base lets every bundle stay add-only. The [first-run walkthrough](../evaluations/default-taxonomy-first-run.md) derives the base from the core on those terms, and it measures what each of five adopters authors and deletes.

### The starter kit is a selection

[Spec 0](00-vision-and-scope.md#what-we-build) promises a doctrine starter kit, and [spec 2](02-taxonomy-model.md) refers to a base package. These are two artifacts, and an earlier reading of [Q3](09-decisions.md#q3--how-much-of-the-default-taxonomy-ships-in-the-box) treated them as one. They answer opposite requirements. The base has to be minimal so that bundles stay add-only. The starter kit has to be opinionated so that a new adopter does not face a blank schema.

So `headwater/starter` is the base package, a named bundle selection, and the doctrine prose that explains the selection. Nobody is expected to run the base bare. Everything composes over it.

### The interview

`headwater init` composes a bundle selection from answers. It is the first-run surface, and the blank-schema problem is a first-run problem. Five rules govern it, and the walkthrough derives each one.

- **It emits an overlay, never a resolved taxonomy.** A tool that writes a complete taxonomy file forks the adopter from the base before they write a document. Every later upgrade is then a merge. Spec 2 requires customization by overlay and never by fork, and this is the one place where a breach of that rule stays invisible.
- **It is package data, not engine code.** [Principle 1](00-vision-and-scope.md#design-principles) puts anything an adopter might want different into the schema. An interview compiled into the engine cannot ship with a third-party package, and a publisher with its own bundles needs its own questions. The interview sits beside profiles and templates in the package. It is not a taxonomy declaration, because it describes the package rather than the corpus, so the count of declarations stays at thirteen.
- **It is `headwater infer` with a second evidence source.** [Q12](09-decisions.md#q12--migration-path-for-an-existing-corpus) makes `infer` propose a taxonomy from a tree that already exists. Both emit the same artifact, so they are one command with two inputs. On an empty repository the tree contributes nothing and the interview asks everything. The blank-schema case is thus the degenerate one rather than a special one.
- **It emits three artifacts, and one read of the tree produces all three.** The overlay above. A report of what the tree holds that no proposed shelf or kind explains. And the [adoption payload](#first-contact-adoption-is-a-migration-from-no-taxonomy), which is the set of findings that the proposal expects to fail. `infer` cannot weaken the base to fit the corpus, and that is structural rather than a rule to police. A bundle selection is add-only, and an add-only overlay carries no operation that removes a base rule.
- **Every question is about the corpus, and none is about the taxonomy.** "Do you write runbooks?" needs no model in the reader's head. "Do you want a `procedure` purpose?" needs the whole of spec 2 first. Each answer selects a bundle, and no answer exposes a declaration name.
- **A question that an existing ruling answers is deleted rather than asked.** [Spec 3](03-authoring-and-lifecycle.md#identifiers) rules that an identifier always carries a namespace, so the interview never asks whether the adopter wants identifiers.

The interview asks only what changes the selection. Everything else waits for a corpus that `taxonomy audit` can measure, because a day-one guess about facet orthogonality is worse than a day-thirty measurement of it.

The cost is the one that overlays already carry, one level up. A resolved taxonomy is an artifact that nobody authored directly, and an interview adds a step where nobody authored the answers as configuration either. So the generated overlay carries a comment above each block that names the question and the answer which produced it. Re-running `init` re-asks with the current answers as defaults and rewrites the same blocks. An adopter who changes their mind edits an answer, not a taxonomy.

## The invariant core

A package declares a **core**: the semantics that an overlay may extend but never remove or redefine ([spec 2](02-taxonomy-model.md#the-immutable-core)). Without one, "the same taxonomy" is not a meaningful claim. If a consumer may override anything, two consumers of one package can share no structure at all.

The core is **semantic, not lexical**, with one exception that it names. It constrains roles and purposes, never names or paths. A consumer may rename every shelf, relocate every directory, and replace identifier patterns except the namespace and every lifecycle value, and still satisfy the core. The condition: after resolution, some facet still has the state role, some kind still serves the `rationale` purpose, and lineage remains expressible and lifecycle-sensitive.

The exception is the identifier namespace, which `core` requires and no overlay may remove ([spec 2](02-taxonomy-model.md#the-immutable-core)). It is lexical because the alternative is unrecoverable rather than merely untidy.

That is what makes the package a workable boundary object. It is plastic enough to adapt to local practice, and strong enough to keep a common identity across sites. Local form is fully negotiable. Shared meaning is not.

Satisfaction is evaluated on the **resolved** taxonomy. The engine does not forbid particular overlay operations. The engine rejects an overlay when the result fails a core requirement. The rejection names that requirement and the operation that removed its last satisfier.

**Conformance checks the core, not the whole taxonomy.** A consumer that renamed and rearranged everything, but kept the core, is conformant, and the report should say so. This is the difference between a method and a monoculture.

## Upgrading

```
headwater taxonomy diff --to 4.0.0
```

reports, against the *local* corpus rather than in the abstract:

- what changed in the base.
- **measured compatibility across the engine's six dimensions** — classification, instance validity, consequence, projection, identifier, addressability ([spec 2](02-taxonomy-model.md#versioning-by-measured-compatibility)).
- which overlay entries the change invalidates (an override that addresses a removed path is an error, not a silent no-op). This is the `addressability` dimension, reported here at the grain that a consumer can act on.
- whether the new base still satisfies the core under the local overlay.
- which local documents violate the new schema.
- which migration steps apply, split into mechanical and judgment-bearing.

The publisher measures compatibility against its own reference corpora and reference overlays, and attaches the result to the release as a claim. The consumer's run **verifies that claim against documents that the publisher never saw**. A claim that holds upstream but fails locally is the interesting case, not an anomaly. It means that the local corpus exercises something that the reference corpora do not.

`headwater migrate --to 4.0.0` applies the mechanical steps, and those include the overlay rewrite. Addresses that the payload renamed are rewritten in place, and each `add` collision with the new base becomes a judgment task ([spec 2](02-taxonomy-model.md#customization-by-composition)). It emits the rest as a task list with the affected documents attached, ready for a human or a coding agent. The distinction is the whole point. To move files is mechanical. To rewrite a document to fit the section contract of a new kind is not. To pretend that the second is automatable produces plausible, wrong documents at scale.

### Between majors, the corpus is legitimately between valid states

A migration with judgment-bearing tasks creates a period in which the corpus fully satisfies neither the old schema nor the new one. That is an ordinary major upgrade, not an anomaly. The upgrade is atomic for the *taxonomy*. The lock points at 4.0.0 or it does not, and the no-partial-load rule of spec 2 governs the schema alone. The upgrade is not atomic for the *corpus*. A spec written as if it were would make every real upgrade a lie.

Thus the migration state is recorded in the lock: from-version, to-version, an owner, an expiry, and the open task list. The from-version is the one field that names a prior valid state, and it is optional for the reason that the [next section](#first-contact-adoption-is-a-migration-from-no-taxonomy) gives. While tasks remain open, checks run against the new schema. A finding is reported as `migration-pending` when its **(document, rule) pair is one that the migration payload expects to fail**. The payload declared what moved and what must be re-stated. Thus it knows which rules it broke for which documents, and each open task records that pair set.

A label by document alone would blanket every finding on a named document for the whole migration. Defects introduced yesterday would then read as expected breakage. The pair grain keeps yesterday's regression loud while the declared debt stays patient. `migration-pending` findings are counted, visible in coverage, never blocking, and never suppressed individually.

When the last task closes, the state ends. The expiry is the anti-parking device, on the same terms as the expiry of a waiver. A migration state past its expiry is a finding against the owner. It is renewable only by an explicit move of the date — a decision with a paper trail, not a timeout that nobody notices. Waivers are per-rule, and suppressions are per-file. Neither fits a corpus that is half-way across, and that is why the state is its own mechanism, not a pile of either.

### First contact: adoption is a migration from no taxonomy

An organization that adopts Headwater points it at a corpus that nobody wrote to any schema. That corpus was never valid, so it appears to fall outside the state above, which names a version that it came from. It does not. Only the from-version refers to a prior state. The `(document, rule)` grain, the owner, the expiry, the task list, and the counted-visible-never-blocking posture are all defined against the **new** schema ([Q12](09-decisions.md#q12--migration-path-for-an-existing-corpus)).

**So the from-version is absent, and nothing else changes.** Before `headwater init` a corpus is governed by nothing, and every document in it is trivially valid. The findings that the proposed taxonomy raises over the existing tree are therefore the migration payload of that taxonomy's first version. A publisher computes a payload from the diff between two majors. At first contact, [`infer`](#the-interview) computes it from the diff between nothing and one. That payload is the **adoption payload**, and it is a migration state like any other.

The adopter thus gets a green build on the first run. Every document that does not yet fit carries a name, an expiry, and a line in the coverage report. That is what a grandfathering file gives, plus the three properties that such files omit.

**No threshold ever converts an accounting into a silence.** One shipped tool excludes offending files one at a time, then disables the rule once the list passes a limit. That is correct for a mature corpus and wrong here. First contact is the one moment at which every rule exceeds any such limit ([spec 11 §S.5](11-adjacent-work.md#s5-grandfathering-has-a-scale-at-which-it-lies)). A payload therefore holds `(document, rule)` pairs however many there are, and [spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)'s coverage obligations stay total. The cost is a large payload in the lock on a large corpus, and the lock is committed and reviewed.

**Every run reports the count that remains.** The expiry is a date, and a date arrives too late to tell anybody that a payload is not shrinking. So the run reports the number of open pairs beside coverage. A payload that does not move is then visible from the second run rather than from the expiry.

**Adoption never runs on a flag.** No invocation of the engine decides which findings count. `headwater check --changed-only` is a performance scope over a verdict that a full run reaches identically ([spec 6](06-engine-architecture.md#cli)). A mode that gated on newly touched documents would make two runs over one tree disagree.

## Conformance

Vendoring content is not adoption. A consumer can hold a perfect copy of the taxonomy and wire none of it. Conformance is a separate, evaluated question:

```
headwater conformance
```

evaluates the repository against rules that the taxonomy package ships — checks wired in CI, gates required on the default branch, projections regenerated, hooks installed, pin current. It reports gaps with remediation. The rules ship *with the package*. Thus a pin advance brings newly-added requirements into force automatically. Improve the method, and the next upgrade of every consumer surfaces the new gap. That loop is what turns a published method into an adopted one.

Some rules cannot be decided from the repository tree (for instance, a permission granted in the admin console of a platform). These rules degrade to a recorded attestation with an owner and a date. They are not silently dropped.

### Waivers

A consumer may deviate deliberately. A waiver names the rule, the reason, the owner, and an expiry. Waivers appear in the coverage report of the consumer, and they are visible to the publisher in aggregate. Deviation is fine. Invisible deviation is not.

One rule class is outside the mechanism. A [withholding rule](06-engine-architecture.md#an-export-profile-carries-a-filter) is not waivable. A waiver buys time against an error that a later run corrects, and no later run undoes a disclosure.

## Arriving at a corpus cold

Everything above describes a repository that already knows its publisher. A machine that holds only a location knows none of it. It needs to learn which corpora live there, what taxonomy governs each one, and where to start ([Q14](09-decisions.md#q14--discovery-surface)).

**Two questions arrive together, and they close by different routes.** How a machine learns that a corpus exists, when it holds no pointer at all, is **registration**. No file inside a corpus answers it, and no convention in the field pretends otherwise. Every one of them presumes a client that already resolved a name. What closes here is the other question, **resolution**: a machine holds a location, and it needs to learn what governs it.

**Registration is an act of publication into a channel whose reader is already obliged** ([Q16](09-decisions.md#q16--public-presence)). That definition is what explains the failure of every file-based attempt at it, including the one measured convention that tried ([spec 11 §O.3](11-adjacent-work.md#o3-llmstxt-is-the-measured-failure-of-a-descriptor-with-no-obliged-reader)). Two obliged channels exist already and Headwater builds neither. A taxonomy package goes to a registry that a resolver must read to install it. A package that carries the publisher's corpus location thus registers that corpus with every consumer. And a rendered page carries the link relation below, so a reader that fetches the page reaches the served copy. Registration is thus the publisher's own act, and the descriptor is the whole of what Headwater supplies for it.

**The corpus descriptor is a projection, and it is the one whose path the engine fixes.** `headwater generate` writes it and `generate --check` holds it to regeneration, like any other projection. What it does not take from the taxonomy is its own location, and the reason is the whole point of it. A reader who has to consult the taxonomy to find the descriptor already has what the descriptor would have told them. So the descriptor sits at `.headwater/corpus.json`, relative to the repository root, and it is engine-defined and non-optional. That is the standing that the [register projection](04-assurance-model.md#every-obligation-has-exactly-one-disposition) already has, for a different reason.

Everything else about it is ordinary. It is generated from the roots, so it cannot go stale against them. That property is what lets one document carry per-root identity at all. The conventions that this resembles keep their index files bare and put identity on each collection. They do so because a hand-maintained index describes roots that somebody else edits.

The descriptor carries five things for each corpus root in the repository. The root path. The taxonomy identity and version. The lock hash. The entry points. And each declared [export profile](06-engine-architecture.md#an-export-profile-carries-a-filter), with its output location and its tombstone grain. It carries nothing that those artifacts already state about themselves. An export declares its own coverage ([spec 12](12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule)), and a second copy of that statement would disagree with the first at the next release.

Three rules make it usable rather than decorative.

- **A version carries a stated client behavior.** A major version above what the reader understands is a hard failure with a message. A minor mismatch is a warning, and the reader continues. A version field with no rule attached is a string.
- **Absence does not read as presence.** The descriptor declares its own media type and a required shape. A reader that receives a success response which does not parse to that shape treats the descriptor as **absent**, not as malformed. The two have different remedies. A served host that answers every path with a default page is the ordinary case rather than the exotic one.
- **The canonical location is inside the repository, and a served copy is reached by a pointer.** A reserved path at the root of an origin is a poor fit here, for three reasons. A repository holds one or more corpora. A documentation site is often one part of a host that serves other things. And the party who writes the descriptor rarely controls the root. A rendered page therefore carries a link relation to the served copy, and the copy may sit anywhere that the site can put it.

**The descriptor is a served artifact, so a filter reaches it first.** It names roots, entry points and profiles, which is organizational structure. An export profile filters it exactly as it filters anything else, and a filtered descriptor announces that it is filtered.

## Federation

Larger organizations use layers: a generic method, a divisional taxonomy that extends it, and a repository overlay that extends that. Two rules keep the stack coherent:

1. **References run upward.** A repository may reference its own tier or a higher one, never a sibling or a lower one. A downward reference makes the upper tier depend on something that it does not control, and the abstraction inverts. The legal reference set is derived from what a repository actually consumes, so it needs no hand-maintained registry.

2. **Overlays compose in one direction.** Each tier may override, add, or remove against the tier above it. A tier never reaches past its parent. Conflicts are resolution errors, not precedence puzzles. Overlay application must be confluent ([spec 2](02-taxonomy-model.md#customization-by-composition)). Thus a three-tier stack has no resolution order that anyone must remember.

### Across taxonomies, not under them

A layered stack only helps organizations that share a root. Two divisions that adopted different taxonomies independently — after an acquisition, or simply because they arrived separately — have no common ancestor to build an overlay against. A merge of the two is a political project, not a technical one.

They do not need to merge. They need **declared correspondences**: SKOS-style mapping relations between their concept schemes ([spec 2](02-taxonomy-model.md#mapping-between-taxonomies)). `exactMatch` where two kinds are interchangeable, `closeMatch` where they are interchangeable for retrieval but not inference, `broadMatch` / `narrowMatch` where one is wider.

With mappings declared, an aggregator answers "every decision in the organization" across taxonomies that share no vocabulary. The next section says what an aggregator is. Neither division gives up its own. Neither taxonomy changes. A third artifact records how they correspond, and the tier that aggregates owns it — normatively, not conveniently. Pairwise mappings between peers grow quadratically, and they go stale on every publisher release ([spec 2](02-taxonomy-model.md#mapping-between-taxonomies)). That is the standard answer to this problem in knowledge organization, and there is no reason to invent a worse one.

### The tier above a corpus harvests it

A tier that answers questions across many corpora needs their content. Two architectures were available, and only one survives contact with the constraints that this specification already set ([Q9](09-decisions.md#q9--multi-repository-corpora)).

**The aggregator is a solution corpus plus one anchor kind.** The solution layer is an ordinary corpus. It authors the facts that live between repositories, and it consumes exports for everything else. What was unstated is how it reaches the corpora below, and nothing new is needed for that. An anchor kind is declared, and exactly one resolver owns it ([spec 2](02-taxonomy-model.md#behavior-at-the-limits)). That resolver reads pinned corpus exports, in the way that the `code_path` resolver reads a source tree.

**There is no merged graph.** Merging *is* anchor resolution, and anchor resolution leaves nothing behind when a run ends. The solution corpus holds its own documents and its own declared edges. It resolves anchors against the exports that it pinned, and it rebuilds that resolution on every run. A merged graph would be canonical for nothing, would carry no reviewer, and would cost one rebuild to reproduce. Such an artifact does not need to exist.

**The tier harvests, and it never fans out.** Each source corpus carries a pin: an identity, a content hash, and a location. A scheduled job fetches each export out of band and commits it, and the resolver then reads the committed copy. The tier never queries a live endpoint. Three arguments agree, and this specification already made all three.

- [Spec 0](00-vision-and-scope.md#non-negotiables) forbids a network dependency at check time, and a fan-out query is one.
- [Spec 6](06-engine-architecture.md#performance-targets) budgets 100 ms for a route query. A fan-out across estates does not fit inside that, and a call that does not fit is a call that developers remove.
- A fan-out that meets an unreachable source either fails whole, or returns a smaller answer with no notice. The second outcome is the silent pass that [spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for) exists to forbid.

So a pinned export that the tier cannot read is a **finding that names the pin**. It is never a narrower answer, delivered quietly. The metadata-harvesting aggregators of the digital-library world reached this architecture under the same pressure, and the [evaluation](../evaluations/graph-export-and-federation.md) records what they found.

**A tier pins an export profile, and the publishing corpus decided what is in it.** A filter acts where the export runs and never where a tier reads. A tier that reads holds bytes that already crossed the boundary ([Q17](09-decisions.md#q17--governed-access-and-the-solution-layer)). So a tier never filters what it harvested, and it has nothing to filter: what arrived is what the publisher meant it to have. An anchor whose target the publisher withheld resolves to `withheld` rather than to a dangling reference ([spec 2](02-taxonomy-model.md#behavior-at-the-limits)). The tier reports it at the profile's declared grain.

**A pin makes revocation late, and the specification says how late.** A document that a publisher withholds today stays in the tier's committed copy until the next harvest. That is the price of harvest over fan-out, and it is not removable inside this architecture. So an export carries its generation time, and the harvest schedule is declared. The revocation lag is then a number that an operator can read rather than a surprise. The authorization systems that solve this problem in the other direction carry a freshness token on every answer. A design with no such token owes the reader the cadence instead ([spec 11 §O](11-adjacent-work.md#o--the-serving-boundary-descriptors-redaction-and-the-write-path)).

**A solution-layer node is a declared anchor.** A tier that models the estate is tempted into nodes for services, interfaces and capabilities. A node that asserts a service's properties has left the corpus and started to model the world ([spec 11 §A.1](11-adjacent-work.md#a1-the-solution-layer-presses-on-that-boundary)). Two arguments refuse it. Nothing in the design carries an obligation to keep such a node true, and a wrong node reads as structural rather than editorial. And a filter has nothing to attach to on a node that carries properties. A document has facets that a predicate reads, and an anchor is carried whole or withheld whole. So a solution-layer node carries an identifier, a name and an owner, and every substantive claim stays inside a document. This is what `code_path` already does. A concrete need that the anchor form cannot meet reopens it, argued as the change to the model that it would be.

## Upstream awareness

A scheduled check compares the pinned version against the latest release of the publisher. It raises a change proposal, with the diff report and the migration assessment attached. It does not raise a notification that nobody acts on. The default is a draft change request that an agent can complete. A pin that only a human can advance is a pin that goes stale.

**A proposal channel carries a budget, and the reason is measured.** Studies of automated dependency proposals report about a third merged for undifferentiated version bumps. For security fixes the figure is about two thirds, against roughly four fifths for proposals that a person wrote. The mechanism is the same in all three. What moves the number is how selective the proposer is. So an unbounded channel converts into notification fatigue, and the usual remedy is a cap on open proposals. Headwater declares that cap beside the schedule, so an operator reads it rather than discovers it.

**Whoever opens a proposal needs more than the permission to open one.** On the platforms in common use, the permission to create a proposal does not include the permission to create the branch that it points at. The permission that does create a branch also permits a merge. So a proposer that authors its own branch is not confined by its permissions alone. Two mechanisms confine it, and an operator states which one is in force. Either a branch rule requires review and grants the proposer no exemption, or the proposer owns a separate repository and proposes from there. To leave this unstated is to claim a separation that the credential does not supply ([Q7](09-decisions.md#q7--scope-of-the-mcp-surface)).

**One pattern, three instances.** A taxonomy pin, a requirements snapshot pin ([Q19](09-decisions.md#q19--inbound-integration-an-external-system-of-record)), and a source-export pin all work the same way. Each one fetches out of band, commits the result, checks against the committed copy, and compares on a schedule. Each one raises a change proposal and never a mutation. To state the pattern once is what keeps the third instance from arriving as a new mechanism.

**A snapshot pin reports drift on each affected edge, and not only on the pin.** A snapshot carries the upstream identity and revision of every item in it, so an advance says which items changed. Every edge into a changed item is then a finding until a person re-verifies it. Requirements practice reached the same mechanism and calls such an edge *suspect*. A proposal against the whole snapshot names a file, and a finding on an edge names the document whose author can act. That is [spec 4](04-assurance-model.md#absence-is-a-finding-class-of-its-own)'s report-at-the-origin rule, applied to a second kind of upstream.
