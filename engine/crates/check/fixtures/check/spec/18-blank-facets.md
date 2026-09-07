---
id: SPEC-FIX-blank-facets
doc_type: design_spec
status: current
status_since: 2026-01-05
summary: ~
title: ""
oracle: []
---

# Blank facets

The failing fixture for `facet.value.blank`, and it carries one document per family rather than three documents.

`summary` is plain null. The text after the colon is `~`, and a rule written as `trim().is_empty()` reads that as content, so this arm is the one that decides whether the implementation reads the core schema.

`title` is empty text. The author wrote a string and the string holds nothing.

`oracle` is a sequence where the declaration says `string`.

`summary` is required and the other two are not, so the report reads the remediation in both of its forms.

Every other document on this shelf declares a `summary` a person wrote, and those are the passing instances.
