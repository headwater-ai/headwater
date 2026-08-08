---
name: ste-editor
description: Edit or rewrite technical documentation to ASD-STE100 Simplified Technical English (Issue 9) — controlled vocabulary, short sentences, active voice. Use when asked to simplify docs, apply STE, or run an editorial pass for readability.
---

# STE editor

Rewrite technical text to ASD-STE100 Simplified Technical English (Issue 9,
2025), or a pragmatic subset of it. STE was built for aerospace maintenance
manuals; its goal is text a non-native reader can understand on first read.

Source of truth: `~/Downloads/ASD-STE100_ISSUE9.pdf` (© ASD). Reference files
in `references/` are derived from it for local editorial use — do not
republish them.

## Pick a profile first

Look at the text and decide (or ask) which profile applies:

- **strict** — procedures, runbooks, how-to steps, safety instructions.
  Apply everything: the closed dictionary and all writing rules.
- **house** — descriptive, conceptual, or specification prose. Apply the
  structural rules (sentence/paragraph limits, active voice, verb forms,
  one-term-per-concept) and prefer approved vocabulary, but do not force the
  closed dictionary where it would distort meaning. Domain terms (e.g.
  "taxonomy", "schema", "front-matter") are legal in both profiles as
  *technical nouns* (rules 1.5–1.11) — keep them, but use each one
  consistently and never as a verb.

## Core rules (both profiles)

Full rule text: `references/writing-rules.md`. The load-bearing subset:

1. **Sentence limits**: ≤ 20 words in instructions, ≤ 25 in descriptive text.
   Count numbers, abbreviations, and parenthetical text as one word each.
2. **One thing per sentence**: one instruction (procedures) or one topic
   (descriptive). Build information gradually across sentences.
3. **Paragraphs**: one topic, ≤ 6 sentences.
4. **Verbs**: only infinitive, imperative, simple present/past/future. No
   auxiliary-verb constructions ("must be removed" → "remove" / "you must
   remove"). No "-ing" verb forms except inside technical nouns. Past
   participle only as an adjective.
5. **Active voice**. Passive only in descriptive text when the agent is
   unknown.
6. **Instructions are imperative**, and a condition comes first, then a
   comma, then the command ("When you supply hydraulic pressure, make sure
   that…").
7. **Multi-word nouns ≤ 3 words**; unpack longer ones with prepositions.
8. **One name per thing** — never two terms for the same concept, never the
   same term for two concepts.
9. **No semicolons, no contractions, no omitted words**; use "that" to open
   subordinate clauses; use articles ("the", "a") rather than dropping them.
10. **Vocabulary (strict profile)**: every word must be an approved dictionary
    word (in its approved meaning and part of speech), a technical noun, or a
    technical verb.

## Workflow

1. Read the whole text first; identify its type and profile.
2. Rewrite pass, applying the rules above. When a word-for-word substitution
   changes the meaning, restructure the sentence instead (rule 9.1).
3. Vocabulary check on doubtful words:
   - `grep -i "^word" references/word-substitutions.tsv` — unapproved words
     with their approved alternatives (word-for-word swaps, same meaning
     only). An entry with an empty second column means: restructure.
   - `grep -i "^word " references/approved-words.txt` — the ~800 approved
     words with their part of speech.
   - A word in neither list is unrestricted vocabulary: in **strict**, it must
     qualify as a technical noun/verb of the subject field; in **house**,
     prefer a plainer word if one exists.
4. Mechanical check on the result: sentence word counts, sentences per
   paragraph, semicolons, "-ing" verb forms, passives, terms used
   inconsistently.
5. Report what changed and why, rule by rule, so edits can be reviewed.
   Flag any place where a limit forced a meaning-relevant trade-off.

## Deep lookup

`references/ste-spec-full.txt` (if present) is the full text extraction of the
spec — grep it for a rule number, a dictionary entry's definition and examples,
or the technical-noun/verb category lists. It is git-ignored (ASD's copyright
permits local use, not redistribution); regenerate it from the repo root with:

    pdftotext -layout ~/Downloads/ASD-STE100_ISSUE9.pdf \
      .claude/skills/ste-editor/references/ste-spec-full.txt
