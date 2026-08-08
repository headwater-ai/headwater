# 7 — Distribution and federation

One organisation defines a documentation method. Many repositories adopt it. It evolves. Everyone must be able to take the evolution without losing what they customised — and the publisher must be able to tell who actually did.

## What is shared, and what is not

| Layer | Shared | Owned locally |
|---|---|---|
| Engine | Yes — a versioned dependency | — |
| Taxonomy schema | Yes — a versioned package | Overlays |
| Doctrine (prose explaining the method) | Yes — vendored or linked | Local method notes |
| Corpus content | No | Everything |
| Router / entry point | No — it describes *this* repository | Yes |
| Control register | Partly — the publisher's obligations are inherited | Local controls and waivers |

The distinction that makes this tractable: **the taxonomy is a package, not a copy.** Prior systems vendored checksummed file trees and gated on byte-identity, which works only while nobody needs to customise. Here, customisation is expressed as an overlay against a versioned base, so an upgrade is a package bump and a re-resolve — not a merge conflict with a file you were never supposed to edit.

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
  plugins: plugins/            # organisation-specific checks
profiles: [service-repo, docs-only, platform]   # named overlays the package ships
migrations: migrations/
```

Publishing is a release: a semantic version, a changelog, an integrity digest, and a migration payload for any major bump. Distribution is over whatever registry or repository the organisation already uses; the engine cares only that it can fetch a version and verify its digest.

## Consuming

A consumer declares what it takes and how it differs:

```yaml
taxonomy:
  package: acme/headwater-taxonomy
  version: 3.2.0
  profile: service-repo
  overlay: .headwater/overlay.yml
```

`headwater taxonomy resolve` fetches, verifies, merges the overlay, validates, and writes the lock. The lock is committed: the corpus is checked against a resolved, reviewable, reproducible taxonomy, and CI needs no network to check anything.

### Profiles are publisher overlays

Not every repository holds every shelf. A profile is a **named overlay the publisher ships** — `remove` operations for the shelves a repository archetype does not carry — selected by name in the consumer declaration above. It is not a separate mechanism: the overlay resolver already implements every part of it (dependent-key deletion, confluence, core satisfaction on the result), and an earlier draft that listed profiles as their own declaration was maintaining two names for a subset of one. The effect is unchanged: a repository never carries a rule, glob, or projection targeting a shelf it does not have, because dead configuration is noise that teaches readers to ignore configuration.

## The invariant core

A package declares a **core**: the semantics an overlay may extend but never remove or redefine ([spec 2](02-taxonomy-model.md#the-immutable-core)). Without one, "the same taxonomy" is not a meaningful claim — if a consumer may override anything, two consumers of one package can share no structure at all.

The core is **semantic, not lexical**. It constrains roles and purposes, never names or paths. A consumer may rename every shelf, relocate every directory, replace every identifier pattern and every lifecycle value, and still satisfy it — provided that after resolution some facet still carries the state role, some kind still serves the `rationale` purpose, and lineage remains expressible and lifecycle-sensitive.

That is what makes the package a workable boundary object: plastic enough to adapt to local practice, robust enough to keep a common identity across sites. Local form is entirely negotiable; shared meaning is not.

Satisfaction is evaluated on the **resolved** taxonomy, not by forbidding particular overlay operations. An overlay is rejected when the result fails a core requirement, naming which requirement and which operation removed its last satisfier.

**Conformance checks the core, not the whole taxonomy.** A consumer that has renamed and rearranged everything while keeping the core is conformant, and should be told so. This is the difference between a method and a monoculture.

## Upgrading

```
headwater taxonomy diff --to 4.0.0
```

reports, against the *local* corpus rather than in the abstract:

- what changed in the base;
- **measured compatibility across the engine's five dimensions** — classification, instance validity, consequence, projection, identifier ([spec 2](02-taxonomy-model.md#versioning-by-measured-compatibility));
- which overlay entries the change invalidates (an override addressing a removed path is an error, not a silent no-op);
- whether the new base still satisfies the core under the local overlay;
- which local documents violate the new schema;
- which migration steps apply, split into mechanical and judgment-bearing.

The publisher measures compatibility against its own reference corpora and attaches the result to the release as a claim. The consumer's run **verifies that claim against documents the publisher has never seen** — and a claim that holds upstream but fails locally is the interesting case, not an anomaly: it means the local corpus exercises something the reference corpora do not.

`headwater migrate --to 4.0.0` applies the mechanical steps and emits the rest as a task list with the affected documents attached — ready for a human or a coding agent. The distinction is the whole point: moving files is mechanical, and rewriting a document to fit a new kind's section contract is not. Pretending the second is automatable produces plausible, wrong documents at scale.

### Between majors, the corpus is legitimately between valid states

A migration with judgment-bearing tasks creates a period in which the corpus fully satisfies neither the old schema nor the new one. That is an ordinary major upgrade, not an anomaly — the upgrade is atomic for the *taxonomy* (the lock points at 4.0.0 or it does not; spec 2's no-partial-load rule governs the schema alone) and is not atomic for the *corpus*, and a spec written as if it were would make every real upgrade a lie.

So the migration state is recorded in the lock: from-version, to-version, an owner, an expiry, and the open task list. While tasks remain open, checks run against the new schema, and a finding is reported as `migration-pending` when its **(document, rule) pair is one the migration payload expects to fail** — the payload declared what moved and what must be re-stated, so it knows which rules it broke for which documents, and each open task carries that pair set. Labelling by document alone would blanket every finding on a named document for the whole migration, letting defects introduced yesterday read as expected breakage; the pair grain keeps yesterday's regression loud while the declared debt stays patient. `migration-pending` findings are counted, visible in coverage, never blocking, and never suppressed individually.

Closing the last task ends the state. The expiry is the anti-parking device, on the same terms as a waiver's: a migration state past its expiry is a finding against the owner, renewable only by explicitly moving the date — a decision with a paper trail, not a timeout nobody notices. Waivers are per-rule and suppressions per-file; neither fits a corpus that is half-way across, which is why the state is its own mechanism rather than a pile of either.

## Conformance

Vendoring content is not adoption. A consumer can hold a perfect copy of the taxonomy and wire none of it. Conformance is a separate, evaluated question:

```
headwater conformance
```

evaluates the repository against rules the taxonomy package ships — checks wired in CI, gates required on the default branch, projections regenerated, hooks installed, pin current — and reports gaps with remediation. Because the rules ship *with the package*, advancing a pin brings newly-added requirements into force automatically: improve the method, and every consumer's next upgrade surfaces the new gap. That loop is what turns a published method into an adopted one.

Rules that cannot be decided from the repository tree (a permission granted in a platform's admin console, for instance) degrade to a recorded attestation with an owner and a date, rather than being silently dropped.

### Waivers

A consumer may deviate deliberately. A waiver names the rule, the reason, the owner, and an expiry. Waivers appear in the consumer's coverage report and are visible to the publisher in aggregate — deviation is fine; invisible deviation is not.

## Federation

Larger organisations layer: a generic method, a divisional taxonomy that extends it, a repository overlay that extends that. Two rules keep the stack coherent:

1. **References run upward.** A repository may reference its own tier or a higher one, never a sibling or a lower one. A downward reference makes the upper tier depend on something it does not control, and the abstraction inverts. The legal reference set is derived from what a repository actually consumes, so it needs no hand-maintained registry.

2. **Overlays compose in one direction.** Each tier may override, add, or remove against the tier above it. A tier never reaches past its parent. Conflicts are resolution errors, not precedence puzzles — and overlay application must be confluent ([spec 2](02-taxonomy-model.md#customisation-by-composition)), so a three-tier stack has no resolution order anyone needs to remember.

### Across taxonomies, not under them

A layered stack only helps organisations that share a root. Two divisions that adopted different taxonomies independently — after an acquisition, or simply by arriving separately — have no common ancestor to overlay against, and merging them is a political project rather than a technical one.

They do not need to merge. They need **declared correspondences**: SKOS-style mapping relations between their concept schemes ([spec 2](02-taxonomy-model.md#mapping-between-taxonomies)). `exactMatch` where two kinds are interchangeable, `closeMatch` where they are interchangeable for retrieval but not inference, `broadMatch` / `narrowMatch` where one is wider.

With mappings declared, an aggregator answers "every decision in the organisation" across taxonomies that share no vocabulary, without either division giving up its own. Neither taxonomy changes; a third artefact records how they correspond, owned by the aggregating tier — normatively, not conveniently, since pairwise mappings between peers grow quadratically and go stale on every publisher release ([spec 2](02-taxonomy-model.md#mapping-between-taxonomies)). That is the standard answer to this problem in knowledge organization, and there is no reason to invent a worse one.

## Upstream awareness

A scheduled check compares the pinned version against the publisher's latest release and raises a change proposal — with the diff report and the migration assessment attached — rather than a notification nobody actions. The default is a draft change request an agent can complete, because a pin that only a human can advance is a pin that goes stale.
