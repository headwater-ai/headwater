---
id: HW-PROBE-a-session-finds-a-ruling-from-a-task-in-plain-words
status: current
status_since: 2026-10-03
summary: A task in plain words shares no word with the name, path or title of the ruling it needs, so a search by name finds nothing and only a search of the text does.
last_verified: 2026-10-03
probe_category: discovery
expectation: opened
oracle: "none"
title: "A session finds a ruling from a task in plain words"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  examines:
    - HW-DR-0049
    - HW-EVAL-why-corpus-counts-are-derived
  traces_to:
    - HW-SPEC-ai-integration
---

# A session finds a ruling from a task in plain words

## Task

Two branches of this repository each add one page. Each branch also writes a new total into a shared file. Git merges the two branches with no conflict, and the total on the main branch is then wrong. Find the document of this repository that rules on how such a total is kept, and give its path.

## Expectation

`opened` over `HW-DR-0049` or over `HW-EVAL-why-corpus-counts-are-derived`. Either one satisfies the predicate. The decision is the ruling, and the evaluation is the measurement that the ruling cites.

**This probe has the vocabulary-mismatch shape (#1472).** The words of the task occur in no identifier, no file name and no title of the two targets. The #1384 re-run put the absent arm at the ceiling, because a session found each target by a word of its name. Here a search by name finds nothing that rules. A session must search the text of the documents, or follow an index.

**The words that were checked.** The content words of the task are `branches`, `page`, `total`, `shared`, `file`, `git`, `merges`, `conflict`, `main`, `wrong`, `rules` and `kept`. None of them occurs in `HW-DR-0049`, in `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`, or in its title "A corpus-wide fold is derived and never stored". None of them occurs in `HW-EVAL-why-corpus-counts-are-derived`, in `docs/evaluations/why-corpus-counts-are-derived-not-stored.md`, or in its title "Why corpus counts are derived, not stored". The case `every_harder_probe_fails_the_shallow_session_and_passes_the_sound_one` in `engine/crates/probe/tests/corpus.rs` holds seven of these words to the task and to both targets on every change.

**What a search by name reaches.** The tree is a present tree sealed for the five probes of #1472 with `tools/probe/seal.sh`, on 2026-10-03. In that tree, `find . -type f | grep -i -E 'branch|page|total|shared|merge|conflict|wrong' | wc -l` printed 73 paths, and none of them is a target. A search of the text reaches both targets: `grep -r -l -i 'two branches' docs | wc -l` printed 27 paths, and the two targets are among them.

**What a shallow session does.** It searches the names for `merge` or `page`, opens a merge hook, a decision about the merge queue or a decision about the style of the hand-built pages, and then answers. None of these is the ruling, so the grade is not satisfied.

**The words of the task are not the words of the ruling, on purpose.** The ruling says "fold", "derived" and "stored". A person who meets the problem says "total", "merge" and "wrong". So this probe measures whether the layer helps a session cross that gap. The route of `headwater route` and the corpus descriptor are two ways the layer offers.

[Spec 5](../spec/05-ai-integration.md) states the claim that the layer helps a session find the document that governs a task. This probe supplies an instrument for that claim and does not supply an observed result.
