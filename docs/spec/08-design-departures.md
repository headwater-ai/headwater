# 8 — Design departures

Documentation governance tooling has a set of recurrent failure modes. Each one is easy to reach, because each one is the cheapest thing to build at the moment that the need appears. This document records the ideas that we adopt. It then records the eight patterns that we design against. Each departure names the pattern, the cost, and the choice that we make instead.

## What we adopt

These ideas work, and we adopt them here on their merits:

| Idea | Why it earns its place |
|---|---|
| Directory placement as the primary classification axis | It is the signal that a reader sees first, and the one that a glob can act on |
| Present-state declarative voice on specifications and standards | The rule with the widest effect. Its precision is **unmeasured**, and [spec 13](13-open-obligations.md#unmeasured-claims) names the run |
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

### 1. Taxonomy as code

The common shape is a set of constants inside each checker: shelf names, type vocabularies, scan roots, and status enumerations. The prose standards restate those values, and the instruction globs restate them again. That makes three copies, and one of them is executable. A team with a different documentation culture cannot express it without a fork of the tooling. A fork then loses every upstream fix.

**Departure:** the taxonomy is a validated, versioned, composable schema and the sole source of structure. Checks are generated from it or configured by it. Customization is an overlay. See [taxonomy model](02-taxonomy-model.md).

### 2. A tool for each concern, a walk for each tool

Governance needs arrive one at a time, so each concern becomes its own executable. Each tool walks the tree again, parses the front matter again, and derives the document types again. Each also brings its own runtime, which the installation must then carry. The result is slow, and two tools can disagree about what a document is.

**Departure:** one engine, one parse, one graph, checks as predicates over it, one finding shape. See [engine architecture](06-engine-architecture.md).

### 3. Structural metadata encoded in filenames

A filename prefix, and a glob that matches it, is a cheap way to record a property such as export scope. It is easy to read in a directory listing, which is a real benefit. But it overloads the name with a facet, and it makes each rename a semantic operation. It also forces literal-path exceptions as soon as a document is coupled to something by path.

**Departure:** provenance and export scope are declared facets. They are checked against the package's declared contents. Naming conventions may still be enforced — as a check over a facet, not as the storage mechanism for one.

### 4. Distribution by checksummed byte-identity

A vendoring model sends consumers verbatim file trees and gates them on byte-identity. This makes drift detectable, and it makes customization nearly impossible. Specialization then leaks into side-channel configuration files. Each new artifact forces a new argument about the boundary between the engine and the vocabulary of the consumer.

**Departure:** versioned packages, explicit overlays, resolved lock files. Identity is a property of the *resolved* taxonomy, not of files on disk. See [distribution](07-distribution-and-federation.md).

### 5. Assurance recorded twice

Coverage lives as a hand-maintained table in a strategy document *and* as a machine-readable register. A checker then holds the two in agreement — a check that exists only to reconcile two copies of one fact. The design, not the domain, makes that check necessary.

**Departure:** one register as data. The human-readable matrix and every coverage claim are generated from it. See [assurance model](04-assurance-model.md).

### 6. Platform coupling

CI templates, work-item conventions, and the hosting model attach themselves to whichever forge and cloud the first adopter uses. They are portable in principle and coupled in practice.

**Departure:** a platform-neutral core with thin adapters. No forge, tracker, or cloud is privileged. More than one adapter exists from the start, and that enforces the adapter boundary.

### 7. Identifier namespacing arrives late

Identifiers start local, because the first corpus is one repository. A namespace per repository is then retrofitted once collisions become foreseeable, and generalized again for each later identifier class. Each step is correct. The sequence is expensive.

**Departure:** every identifier scheme is namespaced and globally resolvable from the first release. A namespace is cheap to apply when an identifier is minted, and expensive to apply later.

### 8. Doctrine and mechanism interleaved

Governance prose, agent instructions, executable tooling, and distribution configuration all live in one repository. Naming conventions and export globs are the only things that separate them. A reader must know those conventions to understand what a change affects.

**Departure:** a clean separation — engine, taxonomy package, doctrine package, corpus — each versioned and consumable independently. A team can adopt the engine and write its own doctrine, or take the doctrine wholesale. One set of opinions becomes a distributable package, not the price of entry.

## What we deliberately do not adopt

- **A generic rule-expression language in the schema.** Anything beyond declarative structure belongs in a plugin with a narrow interface. A schema with conditionals drifts from the corpus that it describes.
- **Blocking gates as the default posture.** Advisory first, promoted on evidence.
- **Documentation profiles as the primary abstraction.** Profiles are a projection convenience over the taxonomy, not a parallel classification system that a team must keep in step with it.
