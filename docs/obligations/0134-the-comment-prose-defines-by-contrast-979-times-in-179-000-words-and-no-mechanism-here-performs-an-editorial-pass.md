---
id: HW-OBL-0134
status: draft
status_since: 2026-08-26
summary: "The comment prose passes every rule this engine has, so the readability problem is a judgment call no check here reaches."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
last_verified: 2026-08-26
title: "The comment prose defines by contrast 979 times in 179,000 words, and no mechanism here performs an editorial pass"
waiting_on: adopter
---

# The comment prose defines by contrast 979 times in 179,000 words, and no mechanism here performs an editorial pass

## Context

Issue #191 ran the check layer's own prose rules over `engine/**/*.rs`, about 179,000 words of comment prose and more than `docs/spec` carries. The two rules with a mechanical remedy, contractions and British spelling, were already met by habit, with zero violations across the whole corpus. The one rule the prose misses, the 25-word sentence limit, stays advisory because the remedy is a rewrite rather than a substitution. A passage from `check/src/scope.rs` stayed under that limit throughout and still read as hard, which is the case the issue built its argument on.

## Obligation

The corpus owes an editorial pass over the worst of that comment prose, because the prose defines by contrast rather than by statement. `rather than` appears 979 times, about 5.5 times per 1,000 words. `is what` appears 397 times, `and it is` 237 times, and `and never` 196 times. `which is what` appears 123 times, and `it is not` appears 73 times. The prose also carries 428 inline quotations stitched into host sentences, 1,082 clause-introducing colons, and 215 em-dash asides. Almost every sentence carries a correction inside it, and each instance is defensible on its own. The aggregate is the problem, and no single instance is a finding a lexical rule could report. A worked rewrite of the `check/src/scope.rs` module header cut 211 words to 187, without dropping a claim. Mean sentence length dropped from 17.6 words to 11.7, and the longest sentence dropped from 36 words to 22. [Q24](../decisions/0024-q24-readability-and-what-a-sweep-can-be-asked-about.md) ruled readability out as a sweep class, because every sweep class names two things that disagree and a readability finding names one. The tic counts point to four files as the worst of it: `cli/src/main.rs`, `check/src/scope.rs`, `check/src/cache.rs`, and `check/src/lib.rs`.

## Discharge

The `docs/` half of this is already closed. #204 fixed the broken relative links, and `headwater_check::fragment::comment_links` now holds that rule, covering `check/src/scope.rs` and `check/src/cache.rs`. Three issues carry the rest: #201 for rustdoc-relative links, #202 for `neighbourhood` and `centre` against American spelling, and #203 for the remaining pass. This document closes when those three land, because the pass itself discharges the debt, not a new mechanism. No sweep class or check rule performs this work, since Q24 already ruled it outside what a sweep can confirm. `ste-editor` remains the only route to a rewrite that keeps every citation and every reason intact.
