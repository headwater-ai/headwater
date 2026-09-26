---
id: HW-DR-0065
status: current
status_since: 2026-09-12
summary: "The relation-level member and the core requirement are a union: either marks a relation, and the published base marks nothing beyond succession."
last_verified: 2026-09-12
title: "A relation declares lifecycle_sensitive for itself and a core requirement demands it of a family"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# A relation declares lifecycle_sensitive for itself and a core requirement demands it of a family

## Context

`lifecycle.dependency.on_terminal` reports a live document that depends on a terminal one, and it reads the relations that the taxonomy marks `lifecycle_sensitive`. Before this change the word had one home. `engine/crates/meta/meta-schema.yml` put it on an entry of `core.requires`, which names a relation *family*. The `relation` block declared thirteen members and none of them this one. `headwater_graph::declarations::Declarations::read` then computed the family set once and wrote the result onto every relation, so a relation body that carried the word was overwritten.

The base package marks one family, `succession`, and that is the one place where a live document that points at a terminal one is correct. `engine/crates/check/fixtures/corpus.checks` records this corpus edge by edge: `relation.target.unresolved` creates one instance for each of 626 edges over 327 classified documents. `lifecycle.dependency.on_terminal` creates 2 instances over the same corpus and reports neither of them. That is because both are the exempt case where the edge wrote the state it points at. So the rule reads two edges of six hundred and says nothing about either.

An adopter who wants a citation of a retired document reported is blocked at both grains. Per relation, the meta-schema has no member to write. Per family, [spec 2](../spec/02-taxonomy-model.md#the-immutable-core) permits an overlay to extend the core, so the adopter may add `{relation_family: evidence, lifecycle_sensitive: true}`. But `headwater_resolve::core::satisfiers` satisfies a lifecycle-sensitive family requirement only with a relation of that family that declares `on_target.set_state`. No relation of the `evidence` family declares one, and none should, because a citation that wrote a state onto the document it cites is absurd. The requirement is then refused as unsatisfied although the adopter removed nothing.

## Decision

**A relation declares `lifecycle_sensitive` for itself.** `relation.lifecycle_sensitive` is an optional boolean of the meta-schema at 0.4.0. Every source that validated under 0.3.0 validates under this one. So the bump is a minor by the rule the version comment of that file already states.

**The two readings are a union and never an override.** A `core.requires` entry is a *requirement over a family*: it demands that the taxonomy marks that family, and an overlay may not resolve it away. The member on a relation body is a *declaration by that relation*: it marks that relation and asserts nothing about the family the relation names. A relation is lifecycle-sensitive when either reading marks it. `Declarations::read` unions the two, and a relation that declares neither is not sensitive.

**The published base marks nothing beyond what the `succession` requirement already marks.** Twelve relations are declared in `.headwater/taxonomy.lock` and one of them is of a marked family. `traces_to` and `cites_evidence` stay unmarked. Marking either of them makes every live document that cites a retired one a finding in every corpus that adopts the published package. That is a ruling for the adopter to make in their own overlay, rather than one for the publisher to impose. An adopter who wants it writes `lifecycle_sensitive: true` on the relation in their own overlay and adds no core requirement at all.

**What reopens this.** A measured corpus where a citation of a retired document is a defect that the adopter wants reported by default. Then the base marks the family, and this record is superseded rather than amended.

## Consequences

**The engine reads a member it never read.** `read_relation` reads `lifecycle_sensitive` through the core schema, exactly as `headwater_resolve::core` reads the flag of the same name. So `True` and `TRUE` are the same declaration and the quoted string the meta-schema refuses is not one. `Declarations::read` unions rather than assigns. `engine/crates/check/fixtures/terminal-dependency.taxonomy.yml` gains `cites`, of the `evidence` family that no core requirement of that fixture names. It also gains one live record that cites the retired one through it. The rule reports that pair and stays silent on `mentions`, which neither reading marks.

**This corpus does not move.** `packages/headwater-standard/taxonomy.yml` is unchanged, so `.headwater/taxonomy.lock` is unchanged, and `headwater check --strict` exits 0 over this corpus, with every finding at `warn` and no error. The finding count moves by the prose this change adds and by nothing else.

**`satisfiers` read a proxy until [#853](https://github.com/headwater-ai/headwater/issues/853) repaired it.** `headwater_resolve::core` tested a lifecycle-sensitive family requirement through `on_target.set_state` alone. It now reads the declared member as well, and either reading satisfies the requirement, as the decision above rules for the check. The `on_target` reading stays, so a taxonomy pinned to headwater-standard 4.4.2 still satisfies its core. The base marks `supersedes` with the member from headwater-standard 4.5.0.

**A finding of the meta-schema is unchanged.** [HW-OBL-0049](../obligations/0049-the-meta-schema-closes-eight-value-sets-that-nothing-states.md) records eight value sets that the meta-schema closes and no specification states. This change adds a boolean scalar, which closes no set. It also adds a member to the `relation` block that `role_is_terminal` cites as one of the eight.
