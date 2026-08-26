---
id: HW-OBL-0139
status: draft
status_since: 2026-08-26
last_verified: 2026-08-26
title: "An identifier scheme glues three parts into one string, so nothing can decide that two schemes are disjoint"
summary: "A pattern string glues the prefix, the namespace, and the local part, so disjointness is undecided and a published address is invented from its path."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
waiting_on: adopter
---

# An identifier scheme glues three parts into one string, so nothing can decide that two schemes are disjoint

## Context

HW-OBL-0046 has asked for this since 2026-08-12. It traces to spec 2 and spec 3, which say a grammar with separate fields makes disjointness decidable by construction.

Pull request #207 moved the namespace to the front of every identifier pattern and expanded `literal_prefix` to expand the declared namespace before comparing. That left the underlying grammar untouched, and the obligation stayed open.

`engine/crates/meta/meta-schema.yml` still declares `identifier_scheme.pattern` as one string holding the prefix, the namespace placeholder, and the local-part form together. Five places in the corpus name this gap, and none of them names a work item that owns it.

## Obligation

The corpus owes a grammar for `identifier_scheme` that declares the prefix, the namespace, and the local-part form as separate members. Today `pattern` in the meta-schema carries all three glued into one string, and the engine cannot read the prefix it mints.

`Template::parse(pattern, namespace)` is the sole constructor that three production sites depend on. Those sites mint identifiers, generate the `identifier.pattern.not_met` finding, and read schemes during `taxonomy audit`. A grammar change is a constructor swap, because `Template` still exposes `mint`, `admits`, `sequence_of`, `render`, and `needs` unchanged.

Disjointness is undecidable from a pattern string, and `headwater taxonomy validate` reports as much on every run. `literal_prefix` in `engine/crates/resolve/src/rules.rs` expands the declared namespace, then compares only the text before the first placeholder left. That decides a pair of schemes only when the prefix sits in front of every free placeholder. Two schemes whose free placeholders meet each other remain undecided.

The same gap reaches identifier publication. `engine/crates/generate/src/profile.rs` declares seven emitters, and only `json` and `jsonschema` are built. `rdf` and `skos` parse and refuse, because Q13 gates each unbuilt format behind a named external consumer, and none exists. The OWL and SKOS worked example shows what an emitter does without a grammar. It mints an IRI from a file path, and a rename breaks every reference to it.

## Discharge

Discharge requires `identifier_scheme` to declare the prefix, the namespace, and the local-part form as separate members, with the `# gap:` comment removed rather than reworded. `Template` must be built from those members, with `Template::parse` gone or kept only as a shim pinned by a test.

The existing tests in `check/src/identifier.rs` must still pass, and `mint`, `admits`, `sequence_of`, `render`, and `needs` must keep their current behavior. `identifier_integrity` must decide disjointness from declared prefixes, `literal_prefix` must be gone, and `headwater taxonomy validate` must no longer print a not-decided clause.

All nine schemes must declare the new members, `.headwater/taxonomy.lock` must regenerate, and every string `mint` produces must stay byte-identical to what it produces today. The taxonomy major version must bump, per the cost that HW-OBL-0046 states, and spec 2 and spec 3 must describe the new members.

HW-OBL-0046 reaches a terminal state naming this obligation, and this obligation waits on the grammar change landing.
