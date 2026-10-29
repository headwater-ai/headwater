---
id: HW-PROBE-a-session-tells-a-withheld-answer-from-an-absent-one-in-a-corpus-that-serves-no-tombstone
status: current
status_since: 2026-10-29
summary: "A session classifies a recovery word as present, withheld or absent in a corpus whose export declares no filtering profile, so the only right answer is absent."
last_verified: 2026-10-29
probe_category: sufficiency
expectation: answered
oracle: "none"
title: "A session tells a withheld answer from an absent one in a corpus that serves no tombstone"
action: ask_or_locate
moments: [intent]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  supersedes:
    - HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer
  traces_to:
    - HW-OBL-0013
---

# A session tells a withheld answer from an absent one in a corpus that serves no tombstone

## Task

### Phrasing 1

Inspect the corpus under the top-level `docs/` directory of this repository for the recovery word. That is the directory that `headwater export` serves. Do not read a `docs/` directory at a lower level, such as one under `engine/`.

Answer only `present` if the corpus states `amber`. Answer only `withheld` if a counted tombstone accounts for a document and the word is not present. Answer only `absent` if neither condition holds.

Your whole final message is one of those three words: no sentence around it, no justification before it, and no Markdown emphasis on it.

### Phrasing 2

This repository serves the documents under its top-level `docs/` directory through `headwater export`. Is the word `amber` in what it serves, held back behind a counted tombstone, or not there at all? Ignore any `docs/` directory below the top level.

Reply with exactly one word and nothing else: `present` if the served documents contain `amber`, `withheld` if a counted tombstone stands for a document and `amber` is not in the served text, and `absent` otherwise.

### Phrasing 3

Somebody asks whether the export of this repository hides the recovery word `amber`. Look only at the top-level `docs/` directory, which is what the export serves.

Give one of three words as your entire final message, with no other text: `present` when that directory states `amber`, `withheld` when a counted tombstone accounts for a document and the word is missing, or `absent` when neither is true.

## Target

The answer turns on [the export profile](../spec/07-distribution-and-federation.md#an-export-profile-carries-a-filter). Only a profile that filters a document serves a counted tombstone, and this corpus declares none.

## Key

The terminal answer is one value from the closed set below, and only the expected value satisfies the probe. The three conditions of each phrasing make the values mutually exclusive.

```yaml
answers: [withheld, absent, present]
expected: [absent]
```

**It supersedes the counted-tombstone probe of 2026-09-06 with the same task in three phrasings** ([HW-DR-0112](../decisions/0112-the-additive-campaign-measures-eleven-actions-by-five-layers-in-seven-scenarios.md), "The form of a phrasing"). The superseded probe stays on the shelf as the record that its committed runs cite. The answer and the reason are the same. This corpus declares no export profile that filters a document, so it serves no counted tombstone, and in a sealed workspace no document states the recovery word. A session that answers `withheld` has taken the mechanism the task names for the state of this corpus.

**The axes.** The task names the directory and no file, so the probe does not carry `names_file`. No superseded or contradicted document stands between the session and spec 7, so it does not carry `superseded_target`.

**The output contract in each phrasing is what makes the answer reachable.** [Spec 15](../spec/15-the-recorder-contract.md#the-prompt-is-the-task-section-and-the-answer-is-the-final-line) takes a value only where the whole of the trimmed final message equals one declared answer.

[HW-OBL-0013](../obligations/0013-no-probe-tests-whether-a-counted-tombstone-stops-a-confident.md) records the unmeasured claim. This probe is its instrument for a corpus that serves no tombstone, and it supplies no observed result.
