# 8 — Prior art and departures

This design is informed by studying a prior internal governance framework. That system was unusually complete for its category and got a great deal right; this document records what carries forward as *concept*, and where so much had come to depend on its architecture that it resisted the change we now want.

## What the reference system proved

These ideas work, and are adopted here on their merits:

| Idea | Why it earns its place |
|---|---|
| Directory placement as the primary classification axis | It is the signal a reader sees first and the one a glob can act on |
| Present-state declarative voice on specifications and standards | The single highest-leverage rule; mechanically detectable at useful precision |
| A `last_verified` assertion distinct from the edit date | Git knows when a file changed; only a human knows when it was last true |
| Decision records with retained lineage and reciprocal succession links | Institutional memory at the lowest possible cost |
| Refusing to draft rationale without external evidence | Post-hoc justification is worse than an admitted gap |
| Derived AI rules as thin pointers to canonical sources | Avoids the two-copies-of-every-standard failure |
| Path-scoped, additive rule loading | Context is metered; load what the current file needs |
| Measuring instruction efficacy with probes rather than assuming it | The only evidence that an instruction surface earns its context cost |
| A metadata facet for purpose on shelves that genuinely hold several kinds | Better than fragmenting a shelf whose documents share readers |
| Machine-verifiable contract sidecars beside prose specifications | Prose is canonical for meaning; a schema is canonical for shape |
| Explicit registration of what is *not* covered | Visible debt is triageable; invisible debt compounds |
| Advisory-first, promote on evidence | Keeps the tooling from being switched off in week two |

## What we change, and why

### 1. Taxonomy was code

Shelf names, type vocabularies, scan roots, and status enumerations were constants inside individual checkers, restated in prose standards and again in instruction globs. Three copies, one of them executable. A different documentation culture could not be expressed without forking the tooling — and a fork forfeits upstream fixes.

**Departure:** the taxonomy is a validated, versioned, composable schema and the sole source of structure. Checks are generated from it or configured by it. Customisation is an overlay. See [taxonomy model](02-taxonomy-model.md).

### 2. Ten tools, ten walks of the corpus

Each concern was its own executable, each re-walking the tree, re-parsing front matter, and re-deriving document types — across two languages plus shell, with container fallbacks per tool. Slow, and structurally prone to two tools disagreeing about what a document is.

**Departure:** one engine, one parse, one graph, checks as predicates over it, one finding shape. See [engine architecture](06-engine-architecture.md).

### 3. Structural metadata encoded in filenames

Whether a document was shared or internal was encoded in a filename prefix and a matching glob. It is legible in a directory listing, which is a real benefit, but it overloads the name with a facet, makes renaming a semantic operation, and forces literal-path exceptions the moment a document is coupled to something by path.

**Departure:** provenance and export scope are declared facets, checked against the package's declared contents. Naming conventions may still be enforced — as a check over a facet, not as the storage mechanism for one.

### 4. Distribution by checksummed byte-identity

Consumers received verbatim file trees, gated on byte-identity. This makes drift detectable and customisation nearly impossible — so specialisation leaked into side-channel config files, and the boundary between "engine" and "your vocabulary" had to be re-litigated for each new artefact.

**Departure:** versioned packages, explicit overlays, resolved lock files. Identity is a property of the *resolved* taxonomy, not of files on disk. See [distribution](07-distribution-and-federation.md).

### 5. Assurance recorded twice

Coverage lived as a hand-maintained table in a strategy document *and* as a machine-readable register, kept in agreement by a checker that existed to reconcile two copies of the same fact — a check made necessary by the design rather than by the domain.

**Departure:** one register as data; the human-readable matrix and every coverage claim are generated from it. See [assurance model](04-assurance-model.md).

### 6. Platform coupling

CI templates, work-item conventions, and the hosting model were specific to one forge and one cloud. Portable in principle, coupled in practice.

**Departure:** a platform-neutral core with thin adapters. No forge, tracker, or cloud is privileged, and the adapter boundary is enforced by having more than one adapter from the start.

### 7. Identifier namespacing arrived late

Namespacing identifiers per repository was retrofitted after collisions became foreseeable, then generalised again for later identifier classes. Each step was correct; the sequence was expensive.

**Departure:** every identifier scheme is namespaced and globally resolvable from the first release. Identifiers are cheap to namespace at minting and expensive to namespace later.

### 8. Doctrine and mechanism were interleaved

Governance prose, agent instructions, executable tooling, and vendoring configuration all lived in one repository, distinguished by naming conventions and export globs. Understanding what a change affected meant knowing the conventions.

**Departure:** a clean separation — engine, taxonomy package, doctrine package, corpus — each versioned and consumable independently. A team can adopt the engine and write its own doctrine, or take the doctrine wholesale. The reference system's opinions become one distributable package, not the price of entry.

## What we deliberately do not adopt

- **A generic rule-expression language in the schema.** Anything beyond declarative structure belongs in a plugin with a narrow interface. A schema with conditionals drifts from the corpus it describes.
- **Blocking gates as the default posture.** Advisory first, promoted on evidence.
- **Documentation profiles as the primary abstraction.** Profiles are a projection convenience over the taxonomy, not a parallel classification system that must be kept in step with it.

## Legal and ethical position

This specification was written from a structural study of a prior system's design — its concepts, its architecture, and its friction points. It reproduces no source code, no configuration, no prose, and no organisation-specific content. The ideas credited above are, in the main, general practice in the technical-documentation and architecture-decision-record communities; the credit here is for the specific combination and for demonstrating that the combination holds together in production.
