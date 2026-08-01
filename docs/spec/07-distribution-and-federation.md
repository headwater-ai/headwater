# 7 — Distribution and federation

One organisation defines a documentation method. Many repositories adopt it. It
evolves. Everyone must be able to take the evolution without losing what they
customised — and the publisher must be able to tell who actually did.

## What is shared, and what is not

| Layer | Shared | Owned locally |
|---|---|---|
| Engine | Yes — a versioned dependency | — |
| Taxonomy schema | Yes — a versioned package | Overlays |
| Doctrine (prose explaining the method) | Yes — vendored or linked | Local method notes |
| Corpus content | No | Everything |
| Router / entry point | No — it describes *this* repository | Yes |
| Control register | Partly — the publisher's obligations are inherited | Local controls and waivers |

The distinction that makes this tractable: **the taxonomy is a package, not a copy.**
Prior systems vendored checksummed file trees and gated on byte-identity, which
works only while nobody needs to customise. Here, customisation is expressed as an
overlay against a versioned base, so an upgrade is a package bump and a re-resolve —
not a merge conflict with a file you were never supposed to edit.

## Publishing

A publisher repository declares a **taxonomy package**:

```yaml
package: acme/docgov-taxonomy
version: 3.2.0
requires_engine: ">=1.4 <2"
contents:
  taxonomy: taxonomy/
  doctrine: doctrine/          # prose explaining the method, vendored to consumers
  templates: templates/
  plugins: plugins/            # organisation-specific checks
profiles: [service-repo, docs-only, platform]
migrations: migrations/
```

Publishing is a release: a semantic version, a changelog, an integrity digest, and
a migration payload for any major bump. Distribution is over whatever registry or
repository the organisation already uses; the engine cares only that it can fetch a
version and verify its digest.

## Consuming

A consumer declares what it takes and how it differs:

```yaml
taxonomy:
  package: acme/docgov-taxonomy
  version: 3.2.0
  profile: service-repo
  overlay: .docgov/overlay.yml
```

`docgov taxonomy resolve` fetches, verifies, merges the overlay, validates, and
writes the lock. The lock is committed: the corpus is checked against a resolved,
reviewable, reproducible taxonomy, and CI needs no network to check anything.

### Profiles

Not every repository holds every shelf. A profile names the subset a repository
archetype owns, and the engine prunes accordingly: a repository never carries a rule,
glob, or projection targeting a shelf it does not have. Dead configuration is
noise that teaches readers to ignore configuration.

## Upgrading

```
docgov taxonomy diff --to 4.0.0
```

reports, against the *local* corpus rather than in the abstract:

- what changed in the base;
- which overlay entries the change invalidates (an override addressing a removed
  path is an error, not a silent no-op);
- which local documents violate the new schema;
- which migration steps apply, split into mechanical and judgment-bearing.

`docgov migrate --to 4.0.0` applies the mechanical steps and emits the rest as a
task list with the affected documents attached — ready for a human or a coding
agent. The distinction is the whole point: moving files is mechanical, and rewriting
a document to fit a new kind's section contract is not. Pretending the second is
automatable produces plausible, wrong documents at scale.

## Conformance

Vendoring content is not adoption. A consumer can hold a perfect copy of the
taxonomy and wire none of it. Conformance is a separate, evaluated question:

```
docgov conformance
```

evaluates the repository against rules the taxonomy package ships — checks wired in
CI, gates required on the default branch, projections regenerated, hooks installed,
pin current — and reports gaps with remediation. Because the rules ship *with the
package*, advancing a pin brings newly-added requirements into force automatically:
improve the method, and every consumer's next upgrade surfaces the new gap. That
loop is what turns a published method into an adopted one.

Rules that cannot be decided from the repository tree (a permission granted in a
platform's admin console, for instance) degrade to a recorded attestation with an
owner and a date, rather than being silently dropped.

### Waivers

A consumer may deviate deliberately. A waiver names the rule, the reason, the owner,
and an expiry. Waivers appear in the consumer's coverage report and are visible to
the publisher in aggregate — deviation is fine; invisible deviation is not.

## Federation

Larger organisations layer: a generic method, a divisional taxonomy that extends it,
a repository overlay that extends that. Two rules keep the stack coherent:

1. **References run upward.** A repository may reference its own tier or a higher
   one, never a sibling or a lower one. A downward reference makes the upper tier
   depend on something it does not control, and the abstraction inverts. The legal
   reference set is derived from what a repository actually consumes, so it needs no
   hand-maintained registry.

2. **Overlays compose in one direction.** Each tier may override, add, or remove
   against the tier above it. A tier never reaches past its parent. Conflicts are
   resolution errors, not precedence puzzles.

## Upstream awareness

A scheduled check compares the pinned version against the publisher's latest
release and raises a change proposal — with the diff report and the migration
assessment attached — rather than a notification nobody actions. The default is a
draft change request an agent can complete, because a pin that only a human can
advance is a pin that goes stale.
