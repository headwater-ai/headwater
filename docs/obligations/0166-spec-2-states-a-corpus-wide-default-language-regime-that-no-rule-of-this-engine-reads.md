---
id: HW-OBL-0166
status: current
status_since: 2026-09-06
summary: "Spec 2 tells an adopter that a corpus declares one default language regime. The engine resolves a regime through a kind and implements no fallback, so the first half of that sentence describes nothing."
last_verified: 2026-09-06
title: "Spec 2 states a corpus-wide default language regime that no rule of this engine reads"
waiting_on: ruling
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# Spec 2 states a corpus-wide default language regime that no rule of this engine reads

## Context

[Spec 2](../spec/02-taxonomy-model.md#language-is-declared-not-assumed) tells a reader that the corpus declares one default language regime, and that a kind may bind a different profile. The first half of that sentence describes nothing this engine implements.

`Shape::language_of` in `engine/crates/check/src/shape.rs` walks the ancestry of a kind for a `language` declaration, and it answers with nothing where it finds none. There is no fallback to a regime named `default`. The name `default` is an ordinary regime name, and it carries no meaning beyond being the only name the base package uses.

Nothing binds it either. The base package at `taxonomy-source/headwater-standard/taxonomy.yml` declares `language` on no kind, and neither does any of the six bundles under `docs/taxonomies/`. The count of `kinds.<kind>.language` declarations across the published library is zero. The ten bindings this corpus runs under are all in its own overlay.

**Measured end to end over this corpus on 2026-09-06.** An override of `regimes.language.default.controlled` to `ste-house` was appended to `.headwater/overlay.yml`, where an override is legal. `headwater taxonomy resolve` and `headwater check` each exited 0. The two reports differ on two lines of 73786 bytes, and both of those lines carry the lock digest. Each run reported 395 seen, 283 classified, 5194 check instances and 7 findings.

**This is adopter-facing prose that misdescribes the model.** Spec 2 is what an adopter reads in order to configure a corpus of their own. A reader who takes the sentence at its word sets one default and expects every kind to answer to it. No kind answers to it, and no finding reports the gap. That is more than housekeeping, and it is why this record exists rather than a note in a pull request.

[HW-DR-0056](../decisions/0056-a-language-regime-reaches-prose-through-a-kind-so-the-starter-kit-s-doctrine-carries-the-writing-profile.md) rules who carries the writing profile of the starter kit, and it rests on the measurement above. It rules nothing here, because the repair is a product decision about the taxonomy model.

## Obligation

**The corpus owes a ruling on whether the model has a corpus-wide default language regime.** Two futures follow, and they are not the same product.

In the first, the engine gains the fallback. `Shape::language_of` answers with the regime named `default` where a kind binds none. The sentence at spec 2 becomes true, and the base declaration becomes a value with a reader. That change makes checks that passed fail for an adopter whose kinds bind nothing today. It breaks the `consequence` dimension, so it forces a major version and a migration payload.

In the second, spec 2 loses the claim. The model then says plainly that a regime binds for each kind or binds for nothing. The base declaration is a template an adopter copies rather than a default that applies.

Neither is derivable from the measurement, and the second forecloses the first in silence.

## Discharge

**A decision record that takes one of the two futures.** That is the whole discharge, and the work that follows it is small in either direction. The first costs a change to one function, a major version and a migration payload. The second costs one sentence of spec 2 and one comment beside the declaration in the base package.

**Nothing discharges this by making the sentence vaguer.** A sentence that names no mechanism leaves an adopter with the same question, and it removes the evidence that the question was asked.
