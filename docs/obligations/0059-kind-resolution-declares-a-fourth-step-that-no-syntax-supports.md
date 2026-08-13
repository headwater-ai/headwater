---
id: OBL-repo-0059
title: "Kind resolution declares a fourth step that no syntax supports"
status: current
status_since: 2026-08-12
last_verified: 2026-08-13
summary: "The fourth step of kind resolution applies a path-pattern refinement, and nothing gives that declaration a form."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - SPEC-HW-taxonomy-model
---

# Kind resolution declares a fourth step that no syntax supports

## Context

The fourth step of [kind resolution](../spec/02-taxonomy-model.md#kind-resolution) applies "any path-pattern refinement that the shelf declares". Nothing gives that declaration a form. Spec 2 illustrates the step in prose, the base package declares no refinement, and the design-spec bundle declares none either.

## Obligation

The census walker therefore runs three steps and omits the fourth, because an engine cannot read a declaration that has no shape.

## Discharge

Either the meta-schema gives the refinement a form, or kind resolution has three steps and spec 2 says so.
