---
id: HW-DR-0046
status: current
status_since: 2026-08-31
summary: "`adoption.from` is a pair, a semver and the release digest that was pinned when the migration began, and never a hash of the two."
last_verified: 2026-08-31
title: "Migrating from-version carries a semver and a release digest, kept as separate fields"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-OBL-0082
    - HW-DR-0012
---

# Migrating from-version carries a semver and a release digest, kept as separate fields

## Context

[HW-OBL-0082](../obligations/0082-the-lock-is-half-generated-and-half-authored-and-nothing.md) leaves a ruling open beside its third question. A `from` written by `taxonomy migrate` would be an assertion about a prior version. No digest covers it, no rule reads it, and no run can falsify it. The alternative is a from-version carried by something the engine can verify: the release digest of the artifact migrated from. [Issue #78](https://github.com/headwater-ai/headwater/issues/78) carries the rest of the migration payload work. It waits on exactly this ruling for the one write it still owes, `adoption.from` and the to-version in the lock.

[Spec 7](../spec/07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states) already names the field a semver. It writes: "the migration state is recorded in the lock: from-version, to-version, an owner, an expiry, and the open task list." Dropping the semver for a bare digest would lose the label every message about a migration already reads by, such as "migrating from 3.2.0". It would also abandon a plain reading of the document register.

## Decision

`adoption.from` is a pair, kept as two plain fields rather than folded into one:

```yaml
adoption:
  from:
    version: 3.2.0
    digest: sha256:...
  to: 4.0.0
```

`version` stays the human-readable label spec 7 already names. `digest` is the verification anchor. It is the release digest that `.headwater/taxonomy.yml` pinned at the moment `taxonomy migrate` ran, read through `headwater_resolve::package::consumer(root).digest`, before that pin moves to the new release. It is not a value `migrate` invents. It is the pin a person already authored and committed, carried forward into the lock rather than re-asserted.

The two fields are never combined into one hash. Checking `hash(version, digest)` needs both inputs already in hand. Holding both is exactly the state in which the digest is verified directly against a fetched artifact. The hash gives a caller nothing beyond what the digest alone already gives. It also costs the value the version alone was carrying. Nothing can recover `3.2.0` from a hash to print it in a report or compare it against `--to`. A hash is the right tool for a commitment nobody may read before it opens. It is also right for one compact identifier standing in for values a reader does not otherwise hold. Neither situation applies here. The reader always holds `from.version` and `from.digest` as soon as `migrate` writes them. They sit in plain sight, in a file that is authored and committed for exactly that reading.

Nothing has to reconcile the two fields against each other. They were never two independent claims. `headwater_resolve::release::at` already reads `package.yml` as part of the hashed member set, and refuses a header whose declared version disagrees with it (spec 7, [`release.rs`](../../engine/crates/resolve/src/release.rs)). A `Release`'s `version` and `digest` are therefore internally consistent by construction at the moment they are read. Writing both into `adoption.from` carries that consistency forward. It does not create a second place where the two could drift apart.

The shape is not new to this engine. Package identity is already kept as plain, uncombined fields elsewhere. `conformance::Identity` (`engine/crates/conformance/src/lib.rs`) and `generate::Identity` (`engine/crates/generate/src/lib.rs`) both carry `package`, `version` and `digest` as three fields rather than a digest of the three. `adoption.from` reuses that shape rather than inventing one.

## Consequences

**What this settles for #78.** `taxonomy migrate` may write `adoption.from` as `{version, digest}` and a `to` version into the lock's `adoption` block. The remaining work is the write itself. It reads the pin out of `.headwater/taxonomy.yml` before `migrate` overwrites it. It then commits the block through the batch writer #183 already built for the other migration writes.

**What this does not settle.** The digest is only re-checkable against an artifact that is still reachable, such as a registry or a vendor cache. No crate of this engine archives an old release or walks history ([spec 12](../spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version)). It is a truer claim than a bare version string the moment it is written, and it is not an eternal proof. Documentation of the field states that distinction rather than implying more.

**What no rule reads yet.** Nothing in `headwater-check` verifies `from.digest` against anything after it is written. There is, structurally, nothing left on disk to verify it against once a migration completes. The field is provenance for a person or a later investigation to use, and not an input a check rule consumes. A rule that wanted to check it would need an archived artifact to check it against. That is a separate and larger question, and this decision does not open it.
