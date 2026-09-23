---
id: HW-OBL-0204
status: draft
status_since: 2026-09-23
summary: "A sentence in decision 0077 says HW-DR-0049's acceptance still owes a rewrite of itself. #574 already made that rewrite, so the sentence now describes a past event as future."
last_verified: 2026-09-23
title: "docs/decisions/0077 still describes HW-DR-0049's rewrite as pending, after #574 made it"
waiting_on: build
---

# docs/decisions/0077 still describes HW-DR-0049's rewrite as pending, after #574 made it

## Context

[HW-DR-0077](../decisions/0077-the-consumer-surface-is-what-an-adopter-receives-runs-and-must-have-installed-and-it-is-a-closed-and-declared-list.md) narrows HW-DR-0049. Its Consequences section says of HW-DR-0049: "That record is a draft, and its acceptance carries the rewrite of that sentence." [#574](https://github.com/headwater-ai/headwater/issues/574) moved HW-DR-0049 to `status: current` on 2026-09-23 and rewrote the sentence HW-DR-0077 pointed at, in the same change. HW-DR-0077 was out of #574's named scope, so nobody returned to update its own sentence, which now describes as pending an event that already happened.

## Obligation

HW-DR-0077's Consequences section states, about HW-DR-0049, a future event as still open. The sentence needs the same kind of correction #574 already made to HW-DR-0049 itself: state that the acceptance and the rewrite happened, and name what changed.

## Discharge

This discharges when a session next touches HW-DR-0077 and rewrites the sentence to describe HW-DR-0049's acceptance and rewrite as done, or when a reader confirms HW-DR-0049's current text already covers what the sentence promised and HW-DR-0077 is corrected to say so.
