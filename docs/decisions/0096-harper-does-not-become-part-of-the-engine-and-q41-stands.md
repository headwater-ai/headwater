---
id: HW-DR-0096
status: current
status_since: 2026-09-27
summary: "harper-core found no error in a 25-document sample of this corpus, and it needs a newer compiler than the engine allows. So the engine does not link it, and Vale stays refused."
last_verified: 2026-09-27
title: "Harper does not become part of the engine, and Q41 stands"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0041
    - HW-DR-0005
    - HW-DR-0024
---

# Harper does not become part of the engine, and Q41 stands

## Context

**[#1013](https://github.com/headwater-ai/headwater/issues/1013) asks whether `harper-core` should become a deterministic grammar and autofix layer in this engine.** Harper is a Rust library, so it answers the four reasons of [HW-DR-0041](0041-q41-whether-vale-becomes-a-declared-regime-backend.md) differently from Vale. That record refused Vale as a regime backend. The measurements are in [the Harper spike results](../evaluations/harper-spike-results.md).

**The owner ruled on scope before the spike ran.** The ruling of 2026-09-27 is posted on #1013, and it reads:

> the vision's exclusion of grammar (spec 00, line 70) was made because no tool like Harper existed, not as a principle. It is not a constraint on this decision.

So this record weighs Harper on what it measured, and not on what [spec 00](../spec/00-vision-and-scope.md#what-we-do-not-build) excludes.

## Decision

**The engine does not link `harper-core`.** No Harper rule becomes a control, and no regime names Harper as a backend. Spec 00 keeps its line about grammar, because the ruling licensed a change to it only for a record that integrates Harper.

**Q41 stands unchanged.** Nothing in this spike shows a capability that Vale gives and the engine lacks.

**The decision rests on three measurements, and any one of them is enough:**

- **Harper found no error on this corpus.** Of 187 findings read by hand on 25 documents, 0 were errors, 57 were preferences and 130 were wrong.
- **Harper needs a newer compiler than the engine allows.** `harper-core` 2.11.0 does not compile on rustc 1.93.1, and it declares no minimum. The engine's floor is 1.91.
- **Harper costs more than the engine's whole check.** It adds 496 crates, and one document takes 254 to 382 ms, against a budget of 200 ms for a change-scoped check.

## Consequences

**Each of the four mismatches of HW-DR-0041, for Harper:**

1. **Plugin shape: answered.** Harper is a library. A pure `DocumentCheck` over the parsed body is possible, and the spike is one.
2. **Closed control registry: open.** Harper compiles its rule set in, so the set is closed at a pinned version. But the engine would have to name each Harper rule it does not own, or map a Harper category to a control. The spike did not need to choose, and this record does not.
3. **Scoping: answered, with a correction.** The spike feeds Harper one sentence, masks what the author did not write, and maps each finding to file bytes. The decisive fixture holds. But `Sentence::authored` is the wrong input for grammar: it removes code spans, and the removal wrote 602 false findings on the sample. A grammar rule needs the words a reader sees, with the foreign runs masked.
4. **Determinism: answered.** Two runs were byte-identical. The crate version pins the dictionary, and the dialect and rule set are code. So the four inputs of spec 12 hold if the Harper version counts as part of the check version.

**The two answers that fail are cost and yield, which Q41 did not ask about.** Harper's shape fits this engine better than Vale's. Its output on this corpus does not.

**[HW-DR-0005](0005-voice-checking-depth.md) and [HW-DR-0024](0024-q24-readability-and-what-a-sweep-can-be-asked-about.md) are unchanged.** Harper has no voice rule and no readability score that reaches this corpus. Its `LongSentences` rule fired once in the raw mode and never on the authored text.

**An adopter can still run Harper alongside.** It can run as a separate step, and nothing in this engine stops it. That is the same place spec 00 puts Vale.

**What would reopen this.** Two things together reopen it. The first is a Harper release that declares a `rust-version` at or below the engine's floor. The second is a sample that shows at least one true error for each rule proposed. The seeded set in `tools/engine/harper-spike/tests/seeded.rs` shows that the doubled-word rule and the article rule are correct on text that holds the error. A corpus with those errors would give a different answer, and `tools/engine/harper-spike/build.sh` measures it again.
