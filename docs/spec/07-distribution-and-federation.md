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
profiles: [service-repo, docs-only, platform]   # named overlays the package ships
migrations: migrations/
```

Publishing is a release: a semantic version, a changelog, an integrity digest, and a migration payload for any major bump. Distribution is over the registry or repository that the organization already uses. The engine requires only that it can fetch a version and check its digest.

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

## The invariant core

A package declares a **core**: the semantics that an overlay may extend but never remove or redefine ([spec 2](02-taxonomy-model.md#the-immutable-core)). Without one, "the same taxonomy" is not a meaningful claim. If a consumer may override anything, two consumers of one package can share no structure at all.

The core is **semantic, not lexical**. It constrains roles and purposes, never names or paths. A consumer may rename every shelf, relocate every directory, and replace every identifier pattern and every lifecycle value, and still satisfy the core. The condition: after resolution, some facet still has the state role, some kind still serves the `rationale` purpose, and lineage remains expressible and lifecycle-sensitive.

That is what makes the package a workable boundary object. It is plastic enough to adapt to local practice, and strong enough to keep a common identity across sites. Local form is fully negotiable. Shared meaning is not.

Satisfaction is evaluated on the **resolved** taxonomy. The engine does not forbid particular overlay operations. The engine rejects an overlay when the result fails a core requirement. The rejection names that requirement and the operation that removed its last satisfier.

**Conformance checks the core, not the whole taxonomy.** A consumer that renamed and rearranged everything, but kept the core, is conformant, and the report should say so. This is the difference between a method and a monoculture.

## Upgrading

```
headwater taxonomy diff --to 4.0.0
```

reports, against the *local* corpus rather than in the abstract:

- what changed in the base.
- **measured compatibility across the engine's five dimensions** — classification, instance validity, consequence, projection, identifier ([spec 2](02-taxonomy-model.md#versioning-by-measured-compatibility)).
- which overlay entries the change invalidates (an override that addresses a removed path is an error, not a silent no-op).
- whether the new base still satisfies the core under the local overlay.
- which local documents violate the new schema.
- which migration steps apply, split into mechanical and judgment-bearing.

The publisher measures compatibility against its own reference corpora and attaches the result to the release as a claim. The consumer's run **verifies that claim against documents that the publisher never saw**. A claim that holds upstream but fails locally is the interesting case, not an anomaly. It means that the local corpus exercises something that the reference corpora do not.

`headwater migrate --to 4.0.0` applies the mechanical steps. It emits the rest as a task list with the affected documents attached, ready for a human or a coding agent. The distinction is the whole point. To move files is mechanical. To rewrite a document to fit the section contract of a new kind is not. To pretend that the second is automatable produces plausible, wrong documents at scale.

### Between majors, the corpus is legitimately between valid states

A migration with judgment-bearing tasks creates a period in which the corpus fully satisfies neither the old schema nor the new one. That is an ordinary major upgrade, not an anomaly. The upgrade is atomic for the *taxonomy*. The lock points at 4.0.0 or it does not, and the no-partial-load rule of spec 2 governs the schema alone. The upgrade is not atomic for the *corpus*. A spec written as if it were would make every real upgrade a lie.

Thus the migration state is recorded in the lock: from-version, to-version, an owner, an expiry, and the open task list. While tasks remain open, checks run against the new schema. A finding is reported as `migration-pending` when its **(document, rule) pair is one that the migration payload expects to fail**. The payload declared what moved and what must be re-stated. Thus it knows which rules it broke for which documents, and each open task records that pair set.

A label by document alone would blanket every finding on a named document for the whole migration. Defects introduced yesterday would then read as expected breakage. The pair grain keeps yesterday's regression loud while the declared debt stays patient. `migration-pending` findings are counted, visible in coverage, never blocking, and never suppressed individually.

When the last task closes, the state ends. The expiry is the anti-parking device, on the same terms as the expiry of a waiver. A migration state past its expiry is a finding against the owner. It is renewable only by an explicit move of the date — a decision with a paper trail, not a timeout that nobody notices. Waivers are per-rule, and suppressions are per-file. Neither fits a corpus that is half-way across, and that is why the state is its own mechanism, not a pile of either.

## Conformance

Vendoring content is not adoption. A consumer can hold a perfect copy of the taxonomy and wire none of it. Conformance is a separate, evaluated question:

```
headwater conformance
```

evaluates the repository against rules that the taxonomy package ships — checks wired in CI, gates required on the default branch, projections regenerated, hooks installed, pin current. It reports gaps with remediation. The rules ship *with the package*. Thus a pin advance brings newly-added requirements into force automatically. Improve the method, and the next upgrade of every consumer surfaces the new gap. That loop is what turns a published method into an adopted one.

Some rules cannot be decided from the repository tree (for instance, a permission granted in the admin console of a platform). These rules degrade to a recorded attestation with an owner and a date. They are not silently dropped.

### Waivers

A consumer may deviate deliberately. A waiver names the rule, the reason, the owner, and an expiry. Waivers appear in the coverage report of the consumer, and they are visible to the publisher in aggregate. Deviation is fine. Invisible deviation is not.

## Federation

Larger organizations use layers: a generic method, a divisional taxonomy that extends it, and a repository overlay that extends that. Two rules keep the stack coherent:

1. **References run upward.** A repository may reference its own tier or a higher one, never a sibling or a lower one. A downward reference makes the upper tier depend on something that it does not control, and the abstraction inverts. The legal reference set is derived from what a repository actually consumes, so it needs no hand-maintained registry.

2. **Overlays compose in one direction.** Each tier may override, add, or remove against the tier above it. A tier never reaches past its parent. Conflicts are resolution errors, not precedence puzzles. Overlay application must be confluent ([spec 2](02-taxonomy-model.md#customization-by-composition)). Thus a three-tier stack has no resolution order that anyone must remember.

### Across taxonomies, not under them

A layered stack only helps organizations that share a root. Two divisions that adopted different taxonomies independently — after an acquisition, or simply because they arrived separately — have no common ancestor to build an overlay against. A merge of the two is a political project, not a technical one.

They do not need to merge. They need **declared correspondences**: SKOS-style mapping relations between their concept schemes ([spec 2](02-taxonomy-model.md#mapping-between-taxonomies)). `exactMatch` where two kinds are interchangeable, `closeMatch` where they are interchangeable for retrieval but not inference, `broadMatch` / `narrowMatch` where one is wider.

With mappings declared, an aggregator answers "every decision in the organization" across taxonomies that share no vocabulary. Neither division gives up its own. Neither taxonomy changes. A third artifact records how they correspond, and the tier that aggregates owns it — normatively, not conveniently. Pairwise mappings between peers grow quadratically, and they go stale on every publisher release ([spec 2](02-taxonomy-model.md#mapping-between-taxonomies)). That is the standard answer to this problem in knowledge organization, and there is no reason to invent a worse one.

## Upstream awareness

A scheduled check compares the pinned version against the latest release of the publisher. It raises a change proposal, with the diff report and the migration assessment attached. It does not raise a notification that nobody acts on. The default is a draft change request that an agent can complete. A pin that only a human can advance is a pin that goes stale.
