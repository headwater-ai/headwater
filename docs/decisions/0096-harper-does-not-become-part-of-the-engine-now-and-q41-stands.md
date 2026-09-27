---
id: HW-DR-0096
status: current
status_since: 2026-09-27
summary: "harper-core found 0 real errors in 187 hand-read findings on 25 documents of this corpus. So the engine does not link it now, Q41 stands, and a stated precision bar or an adopter request reopens it."
last_verified: 2026-09-27
title: "Harper does not become part of the engine now, and Q41 stands"
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

# Harper does not become part of the engine now, and Q41 stands

## Context

**[#1013](https://github.com/headwater-ai/headwater/issues/1013) asks whether `harper-core` should become a deterministic grammar and autofix layer in this engine.** Harper is a Rust library, so it answers the four reasons of [HW-DR-0041](0041-q41-whether-vale-becomes-a-declared-regime-backend.md) differently from Vale. That record refused Vale as a regime backend. The measurements are in [the Harper spike results](../evaluations/harper-spike-results.md).

**The owner ruled on scope before the spike ran.** The ruling of 2026-09-27 is posted on #1013, and it reads:

> the vision's exclusion of grammar (spec 00, line 70) was made because no tool like Harper existed, not as a principle. It is not a constraint on this decision.

So this record weighs Harper on what it measured, and not on what [spec 00](../spec/00-vision-and-scope.md#what-we-do-not-build) excludes.

## Decision

**The engine does not link `harper-core` now.** No Harper rule becomes a control, and no regime names Harper as a backend. Spec 00 keeps its line about grammar, because the ruling licensed a change to it only for a record that integrates Harper.

**Q41 stands unchanged.** Nothing in this spike shows a capability that Vale gives and the engine lacks.

**The decision rests on precision.** Of 187 findings read by hand on 25 documents, 0 were errors, 57 were preferences and 130 were wrong. The verification of this record read its own sample of 30 findings, drawn another way, and it also found 0 errors. A control that reports no true error on the corpus it runs on costs its readers and gives them nothing.

**Two costs weigh against Harper as well, and neither decides alone:**

- **Crates and time.** It adds 496 crates, and one document takes 254 to 382 ms. The budget for a whole change-scoped check is 200 ms.
- **The compiler.** `harper-core` declares no `rust-version`. Version 2.11.0 fails to compile on rustc 1.91 and on rustc 1.93.1, and it builds on rustc 1.98.1. The verification found that 2.4.0 fails on rustc 1.91 and that 2.0.0 and 1.12.0 pass `cargo check` there. The engine pins no toolchain, and CI builds on current stable, so CI would build 2.11.0. But the engine declares `rust-version = "1.91"`, so linking 2.4.0 or later would raise that floor for every adopter who builds from source.

## Consequences

**Each of the four mismatches of HW-DR-0041, for Harper:**

1. **Plugin shape: answered.** Harper is a library. A pure `DocumentCheck` over the parsed body is possible, and the spike is one.
2. **Closed control registry: open.** Harper compiles its rule set in, so the set is closed at a pinned version. But the engine would have to name each Harper rule it does not own, or map a Harper category to a control. The spike did not need to choose, and this record does not.
3. **Scoping: answered, with a correction.** The spike feeds Harper one sentence, masks what the author did not write, and maps each finding to file bytes. The decisive fixture holds. But `Sentence::authored` is the wrong input for grammar: it removes code spans, and the removal wrote 602 false findings on the sample. A grammar rule needs the words a reader sees, with the foreign runs masked.
4. **Determinism: answered.** Two runs were byte-identical. The crate version pins the dictionary, and the dialect and rule set are code. So the four inputs of spec 12 hold if the Harper version counts as part of the check version.

**The answer that fails is yield, which Q41 did not ask about.** Harper's shape fits this engine better than Vale's. Its output on this corpus does not.

**[HW-DR-0005](0005-voice-checking-depth.md) and [HW-DR-0024](0024-q24-readability-and-what-a-sweep-can-be-asked-about.md) are unchanged.** Harper has no voice rule and no readability score that reaches this corpus. Its `LongSentences` rule fired once in the raw mode and never on the authored text.

**An adopter can still run Harper alongside.** It can run as a separate step, and nothing in this engine stops it. That is the same place spec 00 puts Vale.

**What would reopen this.** Either of two things reopens it, and `tools/engine/harper-spike/build.sh` measures the first again:

- **A precision bar met on a corrected input.** Correct the input first, so that Harper does not read the text of a masked code span as context. Then read by hand at least 50 findings of one Harper rule on any corpus. If at least 4 in 5 of them are true errors, that rule is a candidate control. The seeded set in `tools/engine/harper-spike/tests/seeded.rs` holds text with each error. On it, the doubled-word rule and the article rule are correct, so they are the first to try.
- **An adopter asks.** A person outside this repository can file an issue that asks for grammar findings in the same check. That issue reopens the question, and that adopter's corpus is the sample.
