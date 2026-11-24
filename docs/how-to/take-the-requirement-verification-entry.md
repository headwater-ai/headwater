---
id: HW-HOW-take-the-requirement-verification-entry
status: current
status_since: 2026-11-24
summary: "Select the requirement-verification entry with three bundle names and six overlay lines, then write a requirement, its criterion and its verification with headwater new."
last_verified: 2026-11-24
title: "Take the requirement-verification entry"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft
  evidence_basis: evidenced
---

# Take the requirement-verification entry

**Audience:** an adopter of Headwater, in a repository of their own. The consumer surface in `.headwater/overlay.yml` lists this guide, and `headwater check` holds it to that list.

## Before you start

You need three things.

- A corpus that takes `headwater/standard` at 4.25.0 or later, vendored under `.headwater/packages/`. The entry ships inside that package, so no second download is necessary.
- A `.headwater/taxonomy.yml` and an overlay that you can edit.
- A namespace for your identifiers. This guide uses `ACME`. Replace it with your own.

The entry gives you three kinds. A **requirement** states what must hold before a thing is built. An **acceptance criterion** states the measure that decides whether it holds. A **verification** is the thing that applies the measure, usually a test. The [entry's doctrine](../taxonomies/requirement-verification/doctrine.md) says why each declaration is there.

Every command below ran against a scratch corpus on 2026-11-24, and `engine/crates/cli/tests/requirement_bundle.rs` runs the same sequence on every run of the engine's test suite.

## Steps

### Select the entry

Add three names to `bundles:` in `.headwater/taxonomy.yml`.

```yaml
taxonomy:
  package: headwater/standard
  bundles: [evidence-and-obligation, decision-record, requirement-verification]
```

Keep the `version`, `digest` and `overlay` lines that you have. The entry requires `evidence-and-obligation`, and the resolver refuses a selection that holds the entry and not its dependency. `decision-record` is also necessary. `evidence-and-obligation` declares a purpose that only a kind of `decision-record` or `design-spec` serves. The resolver refuses a purpose that no kind serves. If you already select `design-spec`, you can use it in place of `decision-record`.

### Declare the namespaces and the test prefix

Add these lines to the `add` block of your overlay. Keep the lines for any scheme that you already declare.

```yaml
add:
  identifier_schemes.decision_id.namespace: ACME
  identifier_schemes.obligation_record_id.namespace: ACME
  identifier_schemes.requirement_id.namespace: ACME
  identifier_schemes.acceptance_criterion_id.namespace: ACME
  identifier_schemes.verification_id.namespace: ACME
  anchors.test_site.pattern: "ACME-VER-"
```

A bundle declares no namespace, because every adopter would mint under it. `anchors.test_site.pattern` is the prefix that a test comment opens with when it cites a verification. Without it, a `cited_in` edge binds nothing.

You declare no kind, relation, shelf or purpose. The entry carries all of them.

### Resolve the taxonomy

```sh
headwater taxonomy resolve
```

It exits 0 and writes `.headwater/taxonomy.lock`. Requirements go to `docs/verified-requirements/`, acceptance criteria to `docs/acceptance-criteria/` and verifications to `docs/verifications/`. To file requirements under a different path, add `shelves.verified_requirements.path: <your-path>/**` under `override` in your overlay. Do not use `docs/requirements/**` if you also select `brd-prd`, which holds that path.

### Write a requirement

```sh
headwater new requirement --title "Replay a failed webhook"
```

It writes `docs/verified-requirements/0001-replay-a-failed-webhook.md` with the identifier `ACME-REQ-0001`, and the three headings `Context`, `Requirement` and `Verification`.

### Write the acceptance criterion that settles it

```sh
headwater new acceptance_criterion --title "A replay delivers within one minute" --facet verification_method=test --relates verifies=ACME-REQ-0001
```

The verb refuses to guess the method, so `--facet` names it. The four values are `test`, `demonstration`, `inspection` and `analysis`. `--relates verifies=` writes the criterion's half of the edge. The run names the far half, `verified_by`, which the requirement owes once the criterion leaves `draft`.

### Write the verification that proves the criterion

```sh
headwater new verification --title "The replay latency test" --relates proves=ACME-AC-0001
```

It writes `ACME-VER-0001` with `proves: ACME-AC-0001`. If a test file cites the verification, add `--relates cited_in=<path to the test>`. Then put a comment in that file that holds `ACME-VER-0001`.

### Fill the sections and settle the documents

Write the prose under each heading. Then set `status: current` in the front matter of the three documents, and run the fix.

```sh
headwater check --fix
```

It writes the two far halves: `verified_by: ACME-AC-0001` into the requirement and `proven_by: ACME-VER-0001` into the criterion. It reports each file that it changed on standard error. Read the diff before you commit it.

## How to know it worked

```sh
headwater check --strict
```

It exits 0, and the report says `0 findings` for these three documents.

Two warnings can appear later, and each one names a document that is owed an edge. `requirement-verified` names a `current` requirement that has no `verified_by` edge after 90 days. `acceptance-criterion-proven` names a `current` criterion with the method `test` that no verification proves after 90 days. A criterion with any other method is not read, so it never gets this warning.
