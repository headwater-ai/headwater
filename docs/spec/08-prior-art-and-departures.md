# 8 — Prior art and departures

This is a clean-room implementation, informed by a study of a prior internal governance framework. That system was unusually complete for its category, and many of its choices were correct. This document records what carries forward as *concept*. It also records where so much depended on its architecture that it resisted the change that we now want. Nothing is carried across as code, prose, or configuration.

## What the reference system proved

These ideas work, and we adopt them here on their merits:

| Idea | Why it earns its place |
|---|---|
| Directory placement as the primary classification axis | It is the signal that a reader sees first, and the one that a glob can act on |
| Present-state declarative voice on specifications and standards | The single highest-leverage rule, mechanically detectable at useful precision |
| A `last_verified` assertion distinct from the edit date | Git knows when a file changed. Only a human knows when it was last true |
| Decision records with retained lineage and reciprocal succession links | Institutional memory at the lowest possible cost |
| Refusal to draft rationale without external evidence | Post-hoc justification is worse than an admitted gap |
| Derived AI rules as thin pointers to canonical sources | Avoids the two-copies-of-every-standard failure |
| Path-scoped, additive rule loading | Context is metered. Load what the current file needs |
| Probes that measure instruction efficacy rather than assume it | The only evidence that an instruction surface earns its context cost |
| A metadata facet for purpose on shelves that genuinely hold more than one kind | Better than a split of a shelf whose documents share readers |
| Machine-verifiable contract sidecars beside prose specifications | Prose is canonical for meaning. A schema is canonical for shape |
| Explicit registration of what is *not* covered | Visible debt is triageable. Invisible debt compounds |
| Advisory-first, promote on evidence | Makes sure that the tooling is not switched off in week two |

## What we change, and why

### 1. Taxonomy was code

Shelf names, type vocabularies, scan roots, and status enumerations were constants inside individual checkers. The prose standards restated those values, and the instruction globs restated them again. That made three copies, and one of them was executable. A different documentation culture could not be expressed without a fork of the tooling. And a fork loses upstream fixes.

**Departure:** the taxonomy is a validated, versioned, composable schema and the sole source of structure. Checks are generated from it or configured by it. Customisation is an overlay. See [taxonomy model](02-taxonomy-model.md).

### 2. Ten tools, ten walks of the corpus

Each concern was its own executable. Each walked the tree again, parsed the front matter again, and derived the document types again. The tools were written in two languages plus shell, with container fallbacks per tool. This was slow, and structurally prone to a disagreement between two tools about what a document is.

**Departure:** one engine, one parse, one graph, checks as predicates over it, one finding shape. See [engine architecture](06-engine-architecture.md).

### 3. Structural metadata encoded in filenames

A filename prefix, and a glob that matched it, encoded whether a document was shared or internal. This is easy to read in a directory listing, which is a real benefit. But it overloads the name with a facet, and it makes each rename a semantic operation. It also forces literal-path exceptions as soon as a document is coupled to something by path.

**Departure:** provenance and export scope are declared facets. They are checked against the package's declared contents. Naming conventions may still be enforced — as a check over a facet, not as the storage mechanism for one.

### 4. Distribution by checksummed byte-identity

Consumers received verbatim file trees, gated on byte-identity. This makes drift detectable, and it makes customisation nearly impossible. So specialisation leaked into side-channel config files. Each new artefact forced a new argument about the boundary between "engine" and "your vocabulary".

**Departure:** versioned packages, explicit overlays, resolved lock files. Identity is a property of the *resolved* taxonomy, not of files on disk. See [distribution](07-distribution-and-federation.md).

### 5. Assurance recorded twice

Coverage lived as a hand-maintained table in a strategy document *and* as a machine-readable register. A checker kept the two in agreement — a check that existed only to reconcile two copies of the same fact. The design, not the domain, made that check necessary.

**Departure:** one register as data. The human-readable matrix and every coverage claim are generated from it. See [assurance model](04-assurance-model.md).

### 6. Platform coupling

CI templates, work-item conventions, and the hosting model were specific to one forge and one cloud. They were portable in principle and coupled in practice.

**Departure:** a platform-neutral core with thin adapters. No forge, tracker, or cloud is privileged. More than one adapter exists from the start, and that enforces the adapter boundary.

### 7. Identifier namespacing arrived late

Namespacing per repository was retrofitted onto identifiers after collisions became foreseeable. It was then generalised again for later identifier classes. Each step was correct. The sequence was expensive.

**Departure:** every identifier scheme is namespaced and globally resolvable from the first release. A namespace is cheap to apply when an identifier is minted, and expensive to apply later.

### 8. Doctrine and mechanism were interleaved

Governance prose, agent instructions, executable tooling, and vendoring configuration all lived in one repository. Naming conventions and export globs distinguished them. To understand what a change affected, a reader needed to know the conventions.

**Departure:** a clean separation — engine, taxonomy package, doctrine package, corpus — each versioned and consumable independently. A team can adopt the engine and write its own doctrine, or take the doctrine wholesale. The reference system's opinions become one distributable package, not the price of entry.

## What we deliberately do not adopt

- **A generic rule-expression language in the schema.** Anything beyond declarative structure belongs in a plugin with a narrow interface. A schema with conditionals drifts from the corpus that it describes.
- **Blocking gates as the default posture.** Advisory first, promoted on evidence.
- **Documentation profiles as the primary abstraction.** Profiles are a projection convenience over the taxonomy, not a parallel classification system that a team must keep in step with it.

## Legal and ethical position

We wrote this specification from a structural study of a prior system's design — its concepts, its architecture, and its friction points. It reproduces no source code, no configuration, no prose, and no organisation-specific content. The ideas credited above are mostly general practice in the technical-documentation and architecture-decision-record communities. The credit here is for the specific combination, and for the demonstration that the combination holds together in production.
