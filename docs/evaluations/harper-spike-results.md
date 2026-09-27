---
id: HW-EVAL-harper-spike-results
status: current
status_since: 2026-09-27
summary: "harper-core 2.11.0, fed only the prose an author wrote, found 0 errors in 187 hand-read findings on 25 documents. It adds 496 crates and does not compile at rustc 1.91."
last_verified: 2026-09-28
title: "Harper over this corpus finds no error on a 25-document sample, and version 2.11.0 does not build at the engine's floor of rustc 1.91"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - to: tools/engine/harper-spike/**
      verified_revision: sha256:432b2bb624a36c45c456b9a88d5af8ce8644f1cf26ab32c744ef7e0fe09161ca
  traces_to:
    - tools/engine/harper-spike/build.sh
---

# Harper over this corpus finds no error on a 25-document sample, and version 2.11.0 does not build at the engine's floor of rustc 1.91

[#1013](https://github.com/headwater-ai/headwater/issues/1013) asks whether `harper-core` is worth a place in this engine as a deterministic grammar and autofix layer. This page holds the measurements. The ruling that reads them is [HW-DR-0096](../decisions/0096-harper-does-not-become-part-of-the-engine-now-and-q41-stands.md). The code is in [`tools/engine/harper-spike/`](../../tools/engine/harper-spike/), and `tools/engine/harper-spike/build.sh` reproduces every number.

## The setup

**The spike is its own cargo workspace, and `engine/Cargo.lock` does not change.** It pins `harper-core = "=2.11.0"`, the newest version on crates.io on 2026-09-27. It takes `headwater-doc` by a path dependency, so the input is `Body::sentences()` and the runs of each block.

**Harper reads one sentence at a time, through its plain-English constructor, and never through its own Markdown parser.** The text of a sentence keeps its code spans and quotations. A Harper finding that touches a run the author did not write is dropped. A `--raw` mode gives Harper the whole file through its Markdown parser, for comparison.

**The first version gave Harper the text of `Sentence::authored`, and that text made errors of its own.** `Sentence::authored` removes code spans and quotations. So "on a `conflicts_with` edge" became "on a edge", and the article rule fired. The spaces on each side of a removed span became a double space. On the sample, that version reported 2,182 findings, and the masked version reports 1,580. The net fall is 602. Five rules account for 579 of it: 178 comma findings, 257 capitalization, 111 space, 29 article and 4 doubled-word findings. Twelve other rules fell by 28 in all, and `ToTwoToo` and `WayTooAdjective` rose by 5. The table is `results/v1-drop-authored.tsv`. A grammar checker needs the words a reader sees, and the engine's `authored` text is correct only for a voice rule.

**The decisive fixture is `tests/scoping.rs`.** One file holds the doubled word "the the" in six places. Five are not the author's prose: an inline code span, an inline quotation, a block quote, a fenced code block and an HTML block. The sixth is one authored sentence. The authored mode reports one finding, at the file bytes of the authored occurrence, and the `expect` bytes read from the file match. The raw mode reports the doubled word more than once. Before the offset map existed, the authored mode reported three findings and no file range.

## The sample

**25 documents, 10 of them with a `language.controlled.not_met` finding.** The sample reads five shelves: spec, decisions, evaluations, obligations and how-to. On each, `sample.py` takes up to three documents with a language finding and up to three without, in path order. The evaluations shelf had one flagged document and how-to had none, so the sample holds 25 and not 30. The corpus had 441 checked documents that day. The list is `sample.txt`.

**Harper reported 1,580 findings under 37 rules, and 1,567 of them map back to file bytes.** The table is `results/authored.tsv`, one row for each finding. SpellCheck is 1,069 of them. The raw mode reported 1,839 under 37 rules, of which 234 are title-case findings on headings.

## Precision, read by hand

**I read 187 findings: every finding of a rule with ten or fewer, and ten drawn with a fixed seed from each larger rule.** `draw.py` writes the draw, and `results/verdicts.tsv` holds the verdict for each rule with its n. A verdict is TP where the text is an error, and FP where Harper is wrong. It is PREF where the text is correct and Harper prefers another form.

| verdict | findings | share of 187 |
|---|---|---|
| TP | 0 | 0% |
| PREF | 57 | 30% |
| FP | 130 | 70% |

**No rule had a true positive.** There are 57 PREF findings. Of these, 38 are closed compounds (9), closed prefixes (10), a comma after "Thus" (10) and the serial comma (9). A further 16 expand an abbreviation such as "ms", "GB" or "config". This corpus writes no serial comma by choice. The other 3 are one each from `Codebase`, `OrthographicConsistency` and `WouldNeverHave`: "codebase" for "code base", a capitalized proper name, and a word order. The FP findings have four causes:

- Identifiers and domain terms that the dictionary does not hold: "Q19", "HW-", "SHACL", "fixability", "reachability". This is every drawn SpellCheck and SplitWords finding.
- Correct grammar that a pattern reads wrong: "every one of them", "a ruling", "a finding", "whose effect it displaces", "one of the two".
- List items and table cells, which are not sentences. This is 9 of the 10 drawn `SentenceCapitalization` findings. The tenth is a prose sentence in `docs/evaluations/adjacent-work.md`, at line 277, that opens with the link text "spec 2" in lower case.
- The text of a masked code span, which Harper still reads for context. A span that opens with a period, such as `` `.headwater/` ``, reads as the end of a sentence. The word "to" before a code span reads as "too".

## Misses, against a seeded set and against the engine

**`tests/seeded.rs` holds one sentence for each category.** Harper found three of seven categories.

| seeded defect | Harper | the engine today |
|---|---|---|
| a contraction, "doesn't" | missed | error, with a patch |
| a British spelling, "organisation", "colour", "catalogue" | found, and suggests "catalog's" for "catalogue" | error, with a patch |
| a doubled word, "the the" | found, with a correct replacement | nothing |
| a wrong article, "a element", "an rule" | found, with correct replacements | nothing |
| a retired term, "leverage" | missed | error, with a patch |
| a semicolon in running prose | missed | warning |
| a sentence of 31 words | missed, with a false mass-noun finding in it | warning |

**Against the engine's own findings on the same 25 documents, Harper found none of them.** The 10 flagged documents carry sentence-length and semicolon findings. Harper's `LongSentences` rule fired once on the whole sample, and only in the raw mode.

## The three classes

**The bar for safe autofix is [spec 12](../spec/12-check-layer.md#fixability).** The remedy must be mechanical and total, and it must fit a `Patch::Text` whose `expect` bytes match the file.

- **Safe autofix: RepeatedWords only, and only as a candidate.** Its suggestion replaces the pair with one word, and the fixture shows that the range and the `expect` bytes match the file. It fired on no document of the sample.
- **Advisory with a suggestion:** AnA, SpellCheck for a regional spelling, CompoundNouns, DisjointPrefixes, DiscourseMarkers, OxfordComma, Codebase, WouldNeverHave, and the three expansions ExpandTimeShorthands, ExpandMemoryShorthands and ExpandConfiguration. AnA is not safe, because it reads the letters of a code span and not the sound, so "an `<h2>`" is wrong to it.
- **Detection only:** every other rule that fired. The verdicts table names the class of each.

## What each category of `language.rs` gets

| category today | Harper | why |
|---|---|---|
| contraction | do not touch | Harper missed the seeded one |
| British spelling | complement at most | Harper found it, with one wrong suggestion. Its dictionary flagged 1,069 words on the sample, which are 285 distinct byte strings. The 10 of them read by hand were all correct terms |
| sentence length, 25 words | do not touch | Harper's threshold is higher, and it missed a 31-word sentence |
| semicolon | do not touch | Harper missed it |
| retired terms | do not touch | Harper has no such list, and `retired_terms` is the mechanism |
| voice categories | do not touch | Harper has no voice rule. It reaches no part of [HW-OBL-0134](../obligations/0134-the-comment-prose-defines-by-contrast-979-times-in-179-000-words-and-no-mechanism-here-performs-an-editorial-pass.md), [HW-OBL-0002](../obligations/0002-declarative-voice-is-called-detectable-at-useful-precision.md) or [HW-OBL-0205](../obligations/0205-hw-obl-0002-s-sample-counts-and-voice-rs-s-own-census-predate-the-scent-role-fix-and-need-retaking.md) |

The only category that Harper adds and the engine lacks is grammar: a doubled word and a wrong article. On the authored text of this sample, the doubled-word rule did not fire. The article rule fired 4 times, and all 4 are false. In "a `SKILL.md`", "A SKOS", "§A" and "an `<h2>`", Harper read the letters and not the sound. So the conclusion stands: neither rule found an error in this corpus.

## Runtime and footprint

| measurement | value |
|---|---|
| compilers tried, for `harper-core` 2.11.0 | fails on rustc 1.93.1, the host compiler, with four `E0308` errors in its own `linting/` modules. Fails the same way on rustc 1.91 in `rust:1.91` (the verification of this page). Builds on rustc 1.98.1 in `rust:latest`. The crate declares no `rust-version`, and no image between 1.93.1 and 1.98.1 was tried |
| compilers tried, for older versions | 2.8.0 fails on rustc 1.93.1. The verification of this page ran `cargo check` on rustc 1.91: 2.4.0 fails, and 2.0.0, 1.12.0 and 1.0.0 pass |
| crates `harper-core` adds to what `headwater-doc` reaches | 496, from the resolve graph of every target. It brings the `burn` machine-learning crates for part-of-speech tagging |
| licenses of those crates | all permissive except 4 MPL-2.0 and 2 that offer LGPL-2.1 as one choice. `harper-core` is Apache-2.0 |
| spike binary, release, not stripped | 18,670,680 bytes |
| engine binary, dev-release, without Harper | 55,606,192 bytes |
| linter construction | 305 to 397 ms |
| 25 documents, cold | 9.3 to 9.6 s, which is 373 to 382 ms a document |
| 25 documents, warm, same process | 6.4 to 6.8 s, which is 254 to 271 ms a document |

**Two numbers the issue asked for are not here.** The engine pins no toolchain, and it declares `rust-version = "1.91"` as its floor. To measure the engine binary with `harper-core` 2.11.0 linked, the whole engine must build on rustc 1.98.1 or a compiler near it. This spike built only itself in that image. A `wasm32-unknown-unknown` build was not tried, because the container has no such target installed. Harper ships a WebAssembly package of its own, but this page did not measure it.

**For scale, the engine's whole budget for a change-scoped check is 200 ms** ([spec 6](../spec/06-engine-architecture.md#performance-targets)). One document through Harper takes more than that.

## Determinism

**Two runs over the sample wrote byte-identical output.** Both have the SHA-256 `fbd3460f4b2eb007e4f71c8c0af1d313659b6e6b70687947f89517de951b24eb`. In one process, the warm pass also matched the cold pass row for row.

**Four things must be pinned for that to hold:** the crate version, the dictionary, the dialect and the rule set. The dictionary is compiled into the crate, so the version pins it. The dialect is `Dialect::American` in `src/lib.rs`, and the rule set is `LintGroup::new_curated`. The linter keeps a cache between documents, and the output did not depend on it. So the four inputs of spec 12 hold if the Harper version counts as part of the check version.

## Configuration

**Harper takes a user dictionary and pattern rules of its own, called Weir.** An adopter could add terms and house rules through them without a new Headwater rule language. But the word list of this corpus would have to be written first. On 25 documents, SpellCheck made 1,069 findings over 285 distinct byte strings, counted with `LC_ALL=C sort -u`. So a list for the sample alone holds at most 285 entries. If the list ignores case, 279 strings are distinct, and this page did not test which one Harper does. Some of those strings can be real errors, because only 10 findings were read by hand. This is a smaller list than the first version of this page stated. The recommendation of [HW-DR-0096](../decisions/0096-harper-does-not-become-part-of-the-engine-now-and-q41-stands.md) does not rest on this number. `retired_terms` in the overlay already does what a Weir rule for a retired term would do, and the engine reads it.
