---
id: HW-DR-0084
status: current
status_since: 2026-09-26
summary: "A language regime lists the paths outside the corpus root that it binds, and only the three language rules read them. Such a path is not a document, so no facet, voice, link or relation rule reaches a README."
last_verified: 2026-09-26
title: "A language rule reaches front-door prose outside the corpus root, and no other rule does"
relations:
  constrains:
    - HW-DR-0029
    - HW-DR-0056
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# A language rule reaches front-door prose outside the corpus root, and no other rule does

## Context

The prose that a stranger reads first is usually outside the corpus root. In this repository that is `README.md` at the root, `.github/CONTRIBUTING.md`, `.github/SECURITY.md` and `.github/ISSUE_TEMPLATE/issue.md`. The corpus root is `docs`, and no rule reads any of the four. [#615](https://github.com/headwater-ai/headwater/issues/615) asks whether a rule may reach such a path, and by what mechanism.

**The measurement that raised the question.** While #560 was built, eight defects went into the root `README.md`. They were a British spelling, a contraction, a hard-wrapped block, four retired terms, a dead link and a dead fragment. `headwater check` exited 0, and its report was the same byte for byte. `tools/repo/readme-fixtures.sh` now holds the dead link and the dead fragment for this repository alone. It holds nothing for an adopter.

**Measured again on `main` at f0a4882f.** `headwater check --root .` reports 594 files under the corpus root and 413 checked, with 8373 check instances. The root `README.md` is in none of those counts.

**Moving the files under the root is not the answer.** [HW-DR-0029](0029-q29-whether-a-corpus-root-may-contain-code-and-what-an-interface-contract-may-reach.md) gives the test for the root: a path is corpus content only where a file with no front matter is a defect. A `README.md` that GitHub renders on the first screen has no front matter, and that is correct. So the test keeps it out of the root, and the test still holds. The census also has a cost. [#600](https://github.com/headwater-ai/headwater/pull/600) put the hero image of the README under `assets/` and not under `docs/`. A file under the root moves the census count as a document does. Two branches that each move it can merge to a wrong total.

**A regime reached prose through one route until now.** [HW-DR-0056](0056-a-language-regime-reaches-prose-through-a-kind-so-the-starter-kit-s-doctrine-carries-the-writing-profile.md) measured that `Shape::language_of` binds a regime through `kinds.<kind>.language` and through nothing else. A path outside the root has no kind, so no regime can reach it.

**The owner ruled on the issue on 2026-09-26** ([comment](https://github.com/headwater-ai/headwater/issues/615#issuecomment-5844324547)):

> A language rule may read front-door prose outside the corpus root, such as `README.md`, through a path list that the overlay declares. No other rule kind reads outside the root. 0.4's accepted statement already promises this, and the ruling unblocks the front-door clause of that statement.

This record writes that ruling down in the terms that an implementation reads.

## Decision

**1. A language regime lists the paths outside the root that it binds.** The key is `outside_root` on a regime in `regimes.language`. Its value is a list of patterns in the pattern language of [HW-DR-0074](0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md). A pattern resolves against the repository root, and not against the corpus root. An adopter writes the list in their own overlay, because the list is a fact about one repository:

    regimes.language.ste_house:
      outside_root:
        - README.md
        - .github/*.md

The list sits on the regime and not on a separate key, for two reasons. Each path then has exactly one regime, with no second field to agree with the first. And no fallback to a regime named `default` exists in this engine ([HW-OBL-0166](../obligations/0166-spec-2-states-a-corpus-wide-default-language-regime-that-no-rule-of-this-engine-reads.md)), so an entry with no regime would have nothing to take.

**2. Three kinds of entry are refused when the taxonomy resolves.** A pattern that matches a path inside the corpus root is refused. A path inside the root is placed, and placement is the one way it gets a kind. A pattern that leaves the repository, by `..` or by an absolute path, is refused. A path that two regimes list is refused, because it would then answer to two regimes. A pattern that matches no file is reported by name, on the terms HW-DR-0074 states for an anchor that matches nothing.

**3. Such a path is not a document.** It is not a kind and not a member of the census. It owes no front matter, no identifier and no lifecycle state. It is a path that carries a language regime, and nothing more. This is the second route by which a rule reaches text. Spec 2 stated one route until this record.

**4. Three rules reach such a path, and they are named here.** Each reads the regime that lists the path, on the same terms as for a document that a kind binds:

- `language.controlled.not_met`, which reads a contraction, a spelling against the regime's variant, sentence length and a semicolon in running prose.
- `language.retired_term.used`, which reads the `retired_terms` of the regime.
- `language.source_form.not_met`, which reads a hard-wrapped block.

**5. Every other rule stays inside the root, and these are named so that nobody re-derives them.**

- `facet.required.missing` does not reach such a path. The path has no kind, so it has no required facets. A `README.md` with no front matter is correct.
- `voice.forbidden_construction` does not reach such a path. A voice regime binds per kind, and the path has no kind.
- `link.path.unresolved` and `link.fragment.unresolved` do not reach such a path. The owner ruled language rules only. For this repository, `tools/repo/readme-fixtures.sh` still holds the links of the root `README.md`.
- No structural rule, relation rule, surface rule, freshness rule or lifecycle rule reaches such a path. Each of them reads front matter, a section contract or an edge, and the path has none.

**6. The report keeps the census separate.** Such a path does not move `seen` or `classified`, because it is not a census member. The report counts it on its own line, with the number of paths each regime binds outside the root. `headwater check --fix` writes the same corrections to such a path that it writes to a document. The allow directive applies on the same terms, because both read the bytes of the file and not its kind.

**7. The root does not move, and HW-DR-0029 still holds.** Its test decides what the census admits, and this record admits nothing to the census. A language regime is a promise about text, and the promise does not depend on front matter. So the root question and the language question are separate, and this record answers only the second.

## Consequences

**The engine implements this since [#1159](https://github.com/headwater-ai/headwater/issues/1159).** The census keeps the listed paths in a list of its own, beside its rows and never in them. The three rules read that list through a second runner, and only a rule that declares the second route can use that runner. Decision 2 holds in two halves. The refusals that need no tree are a pattern with `..`, an absolute pattern, and one literal path that two regimes list. `taxonomy resolve` makes these under the projection-targets rule, so no lock that carries one is written, and `resolve --check` fails on it. Three refusals need the tree. The first is a pattern that matches a path under the root. The second is a path that two regimes reach through a wildcard. The third is a symlink. `headwater check` reports each of these as an error of `language.outside_root.refused`, and so does a pattern that matches no file. So `check --strict` fails on each. A refused pattern reads nothing. A symlink is never followed, in either direction, which is the rule the census walk states. The first fixture is `README.md` beside the root of the `outside-root` tree in `engine/crates/check/fixtures/`, with one contraction in it.

**This repository declares its list** under `regimes.language.ste_house` in `.headwater/overlay.yml`: the four files that the Context names. The list does not use `.github/*.md`, because that pattern also matches `.github/copilot-instructions.md`. That file is a copy of `CLAUDE.md` for an agent, and it quotes every retired term in the section that retires them.

**HW-DR-0056 is amended and not superseded.** Its decision stands: the doctrine page of the starter kit carries the writing profile, and no package does. Its premise, that a regime reaches prose only through a kind, becomes one of two routes. The doctrine page names `outside_root` when the engine reads it.

**Spec 2 states the second route** under [Language is declared, not assumed](../spec/02-taxonomy-model.md#language-is-declared-not-assumed), and it cites this record.

**The overlay comment on `adopter_documents`** now says that the three language rules hold the root `README.md`, and that no surface rule does.

**What would reopen this record.** A rule kind other than language that an adopter needs on front-door prose, measured on a real corpus, reopens decision 5. The owner's ruling closes it until then.
