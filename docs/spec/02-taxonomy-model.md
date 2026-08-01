# 2 — The taxonomy model

**This is the central design of the system.** Everything else is downstream of it.

## The problem being solved

In the systems this replaces, the taxonomy exists in at least three places at once:
as prose in a standards document, as constants in each linter, and as path globs in
the AI instruction files. Those three copies drift. Worse, the third copy is
*executable*, so a customer whose documentation culture differs — different shelf
names, an extra document kind, a lifecycle with an extra state — must fork the
tooling to express it, and then loses the ability to take upstream fixes.

The taxonomy must therefore be:

- **declarative** — data, not code;
- **complete** — nothing structural known only to the engine;
- **validated** — the schema itself has a schema;
- **composable** — customisation by overlay, never by fork;
- **versioned** — a change is a release, with a migration path;
- **explainable** — the engine can justify every classification decision.

## Shape

A taxonomy is one logical document, assembled from a base package plus zero or more
overlays, resolving to a single validated object. The illustrative form below is
YAML; the binding decision is deferred (see [open questions](09-open-questions.md)).

```yaml
taxonomy: acme-engineering
version: 3.2.0
extends: docgov/standard@2.1.0        # base package, or null for from-scratch

vocabularies:                          # named, reusable value sets
  lifecycle_state: [draft, current, superseded, deprecated]
  audience:        [engineer, operator, integrator, auditor]

facets:
  status:
    role: state                        # engine-significant role
    values: $vocabularies.lifecycle_state
    required: true
  last_verified:
    role: freshness
    type: date
    stale_after_days: 180
  owner:
    type: string
    required: false
  audience:
    values: $vocabularies.audience
    required: false
    severity: warn

regimes:
  voice:
    declarative:
      forbid: [future_intent, change_narration, phased_rollout]
    narrative: {}
  lifecycle:
    standard:
      initial: draft
      transitions:
        draft:      [current, deprecated]
        current:    [superseded, deprecated]
        superseded: []
      retain_terminal: true            # superseded documents are never deleted

relations:
  supersedes:
    from: [decision]
    to:   [decision]
    inverse: superseded_by
    reciprocal: required
    on_target: {set_state: superseded}
  governs:
    from: [standard, specification]
    to:   [code_path]                  # an external anchor kind
    cardinality: many
  verifies:
    from: [control]
    to:   [obligation]
    reciprocal: derived

shelves:
  decisions:
    path: docs/decisions/**
    homogeneous: true
    kind: decision
    group_by: domain                   # sub-directory is a facet value
  specifications:
    path: docs/specifications/**
    homogeneous: true
    kind: specification
    layout: "{domain}/{component}/{aspect}.md"
  governance:
    path: docs/governance/**
    homogeneous: false
    discriminator: doc_type            # facet naming the kind
    kinds: [standard, methodology, runbook, reference, register, values]

kinds:
  decision:
    identifier: {scheme: decision_id}
    voice: declarative
    lifecycle: standard
    facets:
      require: [status, last_verified, domain, summary]
      forbid:  [doc_type]              # placement already states the kind
    sections:
      require: [Context, Decision, Consequences]
      optional: [Alternatives considered, Related]
    relations:
      may: [supersedes, superseded_by, refines]

identifier_schemes:
  decision_id:
    pattern: "DR-{namespace}-{seq:04d}"
    namespace: repo                    # globally unique when vendored
    allocation: reconcile-first        # never reuse; scan before minting

projections:
  - kind: shelf_index
    for: [decisions, governance]
    output: "{shelf}/README.md"
  - kind: agent_rules
    output: .agent/rules/
  - kind: site_nav
    output: .docgov/nav.yml

profiles:                              # named subsets for repo archetypes
  service-repo:
    shelves: [decisions, specifications, governance]
  docs-only:
    shelves: [governance, architecture, domain]
```

## The eight declarations

| Declaration | Answers |
|---|---|
| `vocabularies` | What controlled value sets exist, reusable across facets |
| `facets` | What metadata documents carry, its shape, and how hard it is enforced |
| `regimes` | Reusable rule bundles: voice, lifecycle, freshness, size |
| `relations` | What typed links exist, their endpoints, cardinality, and reciprocity |
| `shelves` | How the corpus is partitioned, and what each partition means |
| `kinds` | What each species of document is, requires, and may link to |
| `identifier_schemes` | How stable identifiers are shaped, namespaced, and allocated |
| `projections` | What derived artefacts the corpus emits |

Plus `profiles`, which name subsets for repositories that hold only part of the
taxonomy.

## Kind resolution

Given a document path and its front matter, the engine resolves a kind by:

1. matching the path against shelf patterns — most specific wins, ties are a
   schema-validation error, not a runtime coin-flip;
2. if the shelf is homogeneous, taking its declared kind;
3. if heterogeneous, reading the discriminator facet; a missing or unrecognised
   value is a finding whose severity the shelf declares;
4. applying any path-pattern refinement the shelf declares (for instance
   `functional.md` and `technical.md` resolving to different kinds within one
   component directory).

`docgov explain <path>` prints this derivation — which shelf matched, which rule
fired, which facets and sections are consequently required, and which relations are
permitted. Classification is never a black box, for a human or an agent.

### Placement is primary; metadata fills the gap

Directory placement carries the primary classification, because it is the signal a
reader sees first and the one a path glob can act on. Metadata materialises only
what placement *cannot* express.

This produces one rule with real teeth: **a homogeneous shelf forbids the
discriminator facet.** If the directory already says what a document is, restating
it in front matter creates a second truth that will eventually disagree with the
first. The schema enforces the prohibition rather than trusting authors to notice.

## Customisation by composition

An adopter never edits a base taxonomy. They declare an overlay:

```yaml
taxonomy: acme-engineering
extends: docgov/standard@2.1.0

override:
  shelves.decisions.path: docs/adr/**            # we call them ADRs
  facets.status.values: [draft, active, retired] # our lifecycle
  identifier_schemes.decision_id.pattern: "ADR-{seq:03d}"

add:
  shelves.playbooks:
    path: docs/playbooks/**
    homogeneous: true
    kind: playbook
  kinds.playbook: {...}

remove:
  - shelves.proposals            # we do not do time-boxed proposals
  - relations.refines
```

Merge semantics are strict and total:

- **`override`** replaces a value at an addressed path; the path must already exist.
- **`add`** introduces a new key; the key must not already exist.
- **`remove`** deletes a key and everything that depends on it — and the resolver
  **fails** if a surviving declaration still references the removed key. Removing a
  shelf that a projection targets is an error at resolve time, not a mystery later.
- Lists never silently merge. An overlay either replaces a list or uses explicit
  `add_to` / `remove_from` operations.
- Resolution is **order-independent** for disjoint paths and an **error** for
  conflicting ones: two overlays touching the same path is a conflict to resolve,
  not a last-writer-wins race.

The resolved taxonomy is written to a lock file with a content hash. The engine
checks the corpus against the lock, so a resolution result is reproducible and
reviewable in a diff.

## The meta-schema

The taxonomy language has a formal schema, published with the engine and versioned
with it. `docgov taxonomy validate` checks:

- structural conformance to the meta-schema;
- referential integrity — every referenced vocabulary, regime, kind, and facet
  exists; no dangling relation endpoints;
- coverage — every shelf resolves to at least one kind; every kind is reachable
  from at least one shelf, or is explicitly marked abstract;
- determinism — no two shelf patterns can match the same path ambiguously;
- role uniqueness — at most one facet claims each engine-significant role;
- lifecycle soundness — the state machine is connected, has an initial state, and
  its terminal states are declared;
- projection targets — every projection writes inside the corpus and does not
  collide with an authored path.

A taxonomy that does not validate is never applied. There is no partial-load mode.

## Versioning and migration

A taxonomy is a released, semantically versioned package.

| Change | Version | Migration |
|---|---|---|
| Add optional facet, add a kind, add a projection | minor | none |
| Widen an enum, relax a requirement | minor | none |
| Rename a shelf, narrow an enum, require a previously optional facet | major | required |
| Change an identifier scheme | major | required, with an identifier map |

A major version ships a **migration payload**: machine-readable steps declaring
what moved, what was renamed, and what must be re-stated, split into what the
engine can apply mechanically (`docgov migrate --apply`) and what needs human or
agent judgment (emitted as a task list with the affected documents attached).
Adopting a new major version without running its migration is a hard failure, not a
warning — the lock file records the taxonomy version each corpus was validated
against.

## Worked example: three taxonomies, one engine

| | Small team | Product suite | Regulated platform |
|---|---|---|---|
| Shelves | `decisions`, `guides` | + `specifications`, `standards`, `proposals`, `evidence` | + `controls`, `audits`, `risk` |
| Lifecycle | `draft` → `current` | + `superseded`, `deprecated` | + `approved`, with an approver facet |
| Identifiers | none | decision + requirement ids | + control ids, mapped to an external framework |
| Voice regime | unconstrained | declarative on specs and standards | + mandatory normative keyword usage |
| Relations | `supersedes` | + `governs`, `verifies` | + `mitigates`, `attests` |
| Engine changes | none | none | none |

The third column is the real test. If a regulated adopter can express control
mappings, approval states, and attestation relations without touching engine code,
the model is right. If they cannot, the schema is missing a primitive — and the fix
is a new primitive, not a special case.

## Deliberate limits

The taxonomy language is **not** a general programming language. It has no
conditionals, no user-defined functions, and no arbitrary expressions. Rules that
cannot be expressed declaratively are implemented as **check plugins** with a
documented interface (see [engine architecture](06-engine-architecture.md)), and
that boundary is defended: the moment the schema grows an `if`, the drift between
declared and actual structure comes back.
