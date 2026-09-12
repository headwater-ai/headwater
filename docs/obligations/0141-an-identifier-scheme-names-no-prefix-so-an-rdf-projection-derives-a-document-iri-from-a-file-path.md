---
id: HW-OBL-0141
status: current
status_since: 2026-09-06
summary: "The gap `HW-OBL-0046` left open, a document IRI built from a file path rather than a name, is gated on a consumer for `rdf` or `skos` that has not appeared."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
last_verified: 2026-08-26
title: "An identifier scheme names no prefix, so an RDF projection derives a document IRI from a file path"
waiting_on: adopter
---

# An identifier scheme names no prefix, so an RDF projection derives a document IRI from a file path

## Context

The OWL and SKOS worked example records what an `rdf` emitter did in place of a declared document identifier. It minted an IRI for each document from that document's repository path instead. That IRI holds only while the file stays where it is, and a rename breaks every reference to it.

`HW-OBL-0046` names the cause. `identifier_scheme` declares `pattern`, `namespace` and `allocation`. The prefix that discriminates a kind lives inside `pattern`, as a run of literal characters with no name of its own. Issue #215 discharged the disjointness half of that obligation, because `taxonomy validate` already reads disjointness out of the parsed segments. This record carries the half that is left, the member that names the prefix.

## Obligation

The corpus owes a meta-schema that declares the prefix, the namespace and the local-part form as three separate members of `identifier_scheme`. Today all three live inside one `pattern` string, and `Template::parse` in `headwater_meta::identifier` recovers the prefix only by position, not by name. A named member lets the prefix be read directly, in place of a `Template` built by counting characters in a string. `mint`, `admits`, `sequence_of`, `render`, `needs`, `refusal` and `disjoint` keep their current behavior, pinned by tests in `engine/crates/meta/src/identifier.rs`.

The rewrite must not move a single rendered identifier. `engine/crates/check/fixtures/identifiers.mint`, taken before issue #215, records all nine schemes and stays byte-identical once the members are declared. All nine schemes, across `.headwater/packages/headwater-standard/taxonomy.yml`, `docs/taxonomies/decision-record/bundle.yml` and `.headwater/overlay.yml`, owe the new members, and `.headwater/taxonomy.lock` regenerates from them.

The change costs a major version on two artifacts that have already shipped. The meta-schema moves past 0.2.0, to 0.3.0 or 1.0.0, and the package that reads it moves past 2.0.0, to 3.0.0. Nine schemes across three sources rewrite to pay that cost. `docs/spec/03-authoring-and-lifecycle.md` and the identifier passages of `docs/spec/02-taxonomy-model.md` owe a description of the members the taxonomy then declares.

## Discharge

Q13 puts every unbuilt emitter format, `rdf` and `skos` included, behind a named external consumer, and none exists today. This record waits on that consumer, not on engineering effort. A declaration written before a consumer tests it is a declaration nothing exercises.

A named consumer for `rdf` or `skos`, and the three declared members built alongside the emitter that consumer justifies, discharge this. `HW-OBL-0046` reaches `discharged` at that point too, with a discharge that names this record.
