---
id: HW-OBL-0031
title: "List extension in an add-only overlay"
status: current
status_since: 2026-08-12
waiting_on: ruling
last_verified: 2026-08-13
summary: "A bundle holds no `override` and no `remove`, and nothing states whether it may use `add_to` on a list."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0003
---

# List extension in an add-only overlay

## Context

A bundle holds no `override` and no `remove`, and [spec 2](../spec/02-taxonomy-model.md#customization-by-composition) supplies `add_to` for a list. Nothing states whether a bundle may use it.

The confluence argument covers disjoint addresses only, and two `add_to` operations at one address are not disjoint. The wordings disagree as well. Spec 2, [spec 7](../spec/07-distribution-and-federation.md#bundles-are-publisher-overlays-in-the-other-direction) and the [glossary](../spec/glossary.md) permit whatever is not `override` or `remove`, and [Q3](../spec/09-decisions.md#q3--how-much-of-the-default-taxonomy-ships-in-the-box) says that a bundle only adds.

## Obligation

Until this closes, no bundle ships a lifecycle state or a projection, because the base holds both as lists.

**A second entry now names a fact that the collapse loses.** The [decision-record entry](../taxonomies/decision-record/doctrine.md) models the tradition that invented the lifecycle ladder, and it may write no rung of it. Two of the tradition's states arrive at `deprecated`. A rejected record was proposed and refused, so nothing relied on it. A deprecated record was accepted and later abandoned, so things did rely on it. Both values are terminal-retained, so the collapse is silent rather than an error, and a reader cannot tell the two apart. The first entry hit this limit twice and could name no lost fact. This is that fact.

## Discharge

The resolver reads the wordings as they stand. It admits `add_to` in any overlay, and its confluence check refuses two list operations from two sources at one address. What closes this is a ruling in spec 2, and no code can supply it.
