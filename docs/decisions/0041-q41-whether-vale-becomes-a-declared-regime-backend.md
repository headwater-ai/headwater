---
id: HW-DR-0041
status: current
status_since: 2026-08-30
summary: "Vale does not become a declared regime backend, and its findings are not translated into this engine's Finding shape. It stays where spec 00 already puts it, composable alongside, because four structural mismatches answer Q24's reopening condition a second time."
last_verified: 2026-08-30
title: "Q41 — Whether Vale becomes a declared regime backend"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-sonnet-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0005
    - HW-DR-0024
---

# Q41 — Whether Vale becomes a declared regime backend

## Context

**[#438](https://github.com/headwater-ai/headwater/issues/438) asks whether Vale becomes a declared regime backend.** It names two capabilities. A taxonomy binding would name a Vale style as a language regime, the way `ste_house` names this repository's own controlled-language profile. A finding-shape translation would carry a Vale finding into this engine's own `Finding` record, at parity with a built-in rule. [Spec 00](../spec/00-vision-and-scope.md) already answers a narrower question. A prose style checker stays out of scope. Spec 00 names Vale as a tool an adopter runs alongside this engine, composed rather than folded in. [Q24](0024-q24-readability-and-what-a-sweep-can-be-asked-about.md) named the reopening condition for a question of this shape. A profile the taxonomy declares is read by a check. So a reopening argument must say why the check layer is the wrong place a second time. It must not restate the survey that closed the question the first time.

**[Q5](0005-voice-checking-depth.md) already measured where a lexical checker's errors fall on this repository's own prose.** Of 58 sentence-length findings on one landing, 32 were defects in sentence segmentation rather than genuine problems. The larger error source was which text counts as a sentence, not the pattern set doing the matching. That finding moved segmentation onto the correctness-root list ([spec 12](../spec/12-check-layer.md#the-correctness-roots)). It also built [`sentences.rs`](../../engine/crates/doc/src/sentences.rs), which derives a sentence from this engine's own CommonMark parse, scoped to text this document's own author wrote.

**Nothing has resolved either question quietly since #438 was filed.** A search of every decision record and of [spec 13](../spec/13-open-obligations.md) for the word "Vale" finds three hits outside this record. One is the choice of implementation language ([Q1](0001-implementation-language.md)), where Vale is counter-evidence for a language choice, and two are the licensing and observability analogs ([Q11](0011-license-and-distribution-posture.md), [Q16](0016-public-presence.md)). None of the three touches the check-layer question this record answers. The premise holds exactly as #438 states it.

## Decision

**Vale does not become a declared regime backend.** Its findings are not translated into this engine's `Finding` shape. It stays where spec 00 already puts it: composable alongside, run by an adopter as a genuinely separate step. No taxonomy binds it, and no merged output carries its findings. Four structural mismatches answer Q24's reopening condition, and each one is new relative to Q5, Q24 and the earlier survey. Each falls out of specifying the two capabilities #438 names, rather than restating why the check layer holds prose rules at all.

## Consequences

**The plugin interface this engine already declared does not fit Vale as a plugin.** [Spec 12](../spec/12-check-layer.md#the-plugin-interface) fixes the shape at `DocumentCheck { id(), severity(), evaluate(view: &DocumentView) -> [Finding] }`, pure, with no filesystem, no network, no clock and no graph mutation reaching a plugin. Vale is an external process that reads its own configuration file and style packages from disk, and it would have to run as a subprocess. That is not a plugin in the sense this engine already committed to. It is the kind of integration spec 00 already declined, in favor of composing alongside. Building it now would open a second integration path this repository has never needed.

**The closed control registry cannot hold an open-ended external rule set.** [`Finding.rule`](../../engine/crates/check/src/finding.rs) is typed as a compile-time constant string. Every rule this engine carries reaches its obligation through a control that names that constant by identifier ([spec 4](../spec/04-assurance-model.md#findings)). Vale's own rule set is configured at run time. An adopter's style file can pull in any community package naming any number of checks that this engine never sees at compile time. Merging Vale output at parity with a built-in rule has two options, and both fail. One is inventing an identifier and an obligation for every check in every style package an adopter might choose. The other is reporting no obligation at a scale where that omission stops being an edge case. Spec 4 states the standard this would break: a check that cannot name the obligation it protects did not earn its place.

**Vale's own scoping model reintroduces the false-positive class Q5 already measured and fixed.** `sentences.rs` derives a sentence from this engine's own parsed, ownership-scoped text. It excludes a quotation, a code span and a citation by construction, because raw-text segmentation was the larger source of Q5's measured errors. Vale scopes its own checks with pattern-based ignore rules over raw text, the same class of instrument Q5 already moved away from. Vale has no extension point that accepts a typed, already-parsed text stream in place of a file or raw text. Running Vale over raw Markdown would reopen exactly the false-positive class this engine already paid down.

**Determinism has no answer today for an external tool's own versioned inputs.** [Spec 12](../spec/12-check-layer.md#determinism-concretely) fixes a check's cache key at four inputs. They are the content hashes of the in-scope inputs, the taxonomy lock hash, the check version, and the injected values. A Vale binary's own version and a style package's own content are none of those four today. A style package is also not typically vendored, the way this repository already vendors its own taxonomy package. Admitting Vale into the cache key would reopen the plugin interface's own "no network" boundary in a different place.

**A taxonomy-binding shape alone would have been simple, which is why it is not what stops this.** The shape is parallel to `ste_house`: a named regime carrying a style path, bound to a kind through a language facet. What does not close is severity. [Spec 12](../spec/12-check-layer.md#fixability) rules that posture comes from fixability, never from precision. A category may block only when its remediation is mechanical and total. This engine already holds its own permanently advisory voice categories to that same rule. Vale's own `error`, `warning` and `suggestion` levels state a style-package author's opinion about importance, not a claim about mechanical fixability. The honest mapping sends every Vale finding to advisory, regardless of Vale's own level. The exception is a specific check shown to have a mechanical, total remedy. None is shown today.

**This record forecloses nothing about running Vale at all.** An adopter can already run `vale` as a separate step of continuous integration today, with no engine change and no taxonomy change. That is what composing alongside already means, and spec 00 already says so. The ruling answers only the two capabilities #438 names: a taxonomy-declared binding, and a merged `Finding` stream.
