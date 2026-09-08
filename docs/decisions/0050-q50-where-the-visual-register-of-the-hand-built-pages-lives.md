---
id: HW-DR-0050
status: current
status_since: 2026-09-06
summary: "The hand-built pages read as a specification rather than as a product page, and one file holds the color tokens and type stacks that carry it. A script writes that file into each hand-built page before the commit, because the content policy of those pages admits no linked stylesheet, and the generated half links the same file because its own policy admits one."
last_verified: 2026-09-08
title: "Q50 — Where the visual register of the hand-built pages lives"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  constrains:
    - HW-DR-0037
  governs:
    - tools/site-tokens.css
    - tools/refresh-site-tokens.sh
    - mkdocs-hooks/site_tokens.py
    - mkdocs-overrides/css/headwater.css
  traces_to:
    - notes/website-design-brief.md
---

# Q50 — Where the visual register of the hand-built pages lives

## Context

**Eight hand-built pages carry one visual register, and each of them carried its own copy of it.** [HW-DR-0037](0037-q37-which-parts-of-the-site-are-hand-built-and-which-are-a-projection-of-this-corpus.md) governs the eight by name. Every one of them opened its `<style>` element with the same six color tokens and the same dark-scheme override. Four of the eight declared a seventh token, `--accent-on-ink`, that the other four did not. The three type stacks were written out in full at each use, forty times across the eight files.

**No document of this corpus stated those values.** Section 7 of [the design brief](../../notes/website-design-brief.md) named the ground and the accent in prose. It named neither the four other color tokens nor any type stack. The same section states a measure of 40rem. The eight pages set `max-width` at eleven distinct values, and 40rem is not one of them. So the one description of the register was partial where it was right and wrong where it was specific.

**A linked stylesheet is what the content policy of these pages refuses.** `site/_headers` gives every hand-built path `default-src 'none'` with `style-src 'unsafe-inline'` and no `'self'`. HW-DR-0037 states as a measurement that a hand-built page inlines what it uses and reaches nothing at read time. [HW-DR-0039](0039-q39-how-a-figure-reaches-a-hand-built-page.md) reads the same header and relies on the same property. A served stylesheet needs `'self'` added to `style-src` at all eight paths.

## Decision

**The register is a document rather than a product page, and that is the ruling.** The pages set body prose in a serif face. They use a sans face for structure alone, which is labels, table headers, navigation and code annotations. They set terminal output as text and never as an image. They prefer a table to a card, because the reader [HW-DR-0016](0016-public-presence.md) names reads tables. They carry no stock illustration and no logo wall, because this project has no customer logos and an invented one would break principle 8. Nothing checks any sentence of this paragraph, and it is a ruling so that a later page has something to answer to.

**One file holds the values that carry it, and a script writes that file into each page before the commit.** `tools/site-tokens.css` is the copy a person edits. `tools/refresh-site-tokens.sh` writes the block between the two `headwater:tokens` markers of every `index.html` under `site/`. This is the form HW-DR-0039 ruled for a figure on these same pages, applied to a declaration instead of to a measurement. The interpolation runs on the machine of the person who writes the page, so the committed bytes are the served bytes.

**The register is the seven color tokens, the three type stacks, and the two classes that name the stacks.** Each value was identical in every page that declared it, so no page loses a declaration it had. A page that uses none of `--accent-on-ink`, `.sans` or `.mono` carries them anyway. An unused custom property and an unmatched class rule change no rendering.

**The measure stays out of the file.** `max-width` differs by page for a reason a reader can see, because a comparison table is not set to the width of a prose column. A shared value here would be a figure with no source, which is what HW-DR-0037 forbids on these pages.

**A page under `site/` with no marker pair fails the check.** The script walks the directory rather than reading a list of paths. A ninth page is therefore covered by the commit that adds it. A page that opts out of the register is refused rather than skipped.

**The generated half links the same file, and it never restates a value from it.** The title above names the question as it was asked, and this clause widens the answer to the other half of the site. `mkdocs-hooks/site_tokens.py` registers `tools/site-tokens.css` as a generated file at `css/site-tokens.css`, and `mkdocs-overrides/main.html` links it. The served copy is the source bytes rather than a copy of them. `mkdocs-overrides/css/headwater.css` reads every color and every type stack through `var()`, and it declares none.

**It links where the hand-built half inlines, because the two halves are served under different policies.** `site/_headers` gives each hand-built path `style-src 'unsafe-inline'` with no `'self'`, which is the property HW-DR-0037 states as a measurement. The generated half is served from `/*`, and that policy carries `style-src 'self'`. So the generated half can reach a stylesheet at read time and the hand-built half cannot. Neither policy moves for this record.

**A second copy under `mkdocs-overrides/` fails `tools/refresh-site-tokens.sh --check`.** Before this clause, that script walked `site/` alone. So a theme stylesheet could write out a color of the register, and no reader in this repository would report it. The script now reads each `.css`, `.html` and `.js` file under `mkdocs-overrides/`. It fails on a file that carries a value that `tools/site-tokens.css` declares. It reads the values out of the file rather than from a list, so a new token is covered by the commit that adds it. The first run of this arm failed on a comment that quoted a color to explain the rule.

## Consequences

**The check runs at two places, and a person is neither of them.** `.githooks/pre-commit` refuses a commit whose page disagrees with the file, under `HEADWATER_SKIP_TOKEN_CHECK`. The CI step named "Every hand-built page carries the same visual register" reads the committed tree, which is the half that holds. Five cases of `.githooks/fixtures.sh` drive the stale block, the missing marker pair, the two lines the refusal prints, and the escape hatch.

**Four SVG text elements keep a literal type stack.** `site/index.html` draws a small corpus as a diagram, and its `<text>` elements set `font-family` as a presentation attribute. An attribute takes no `var()`, so those four name `ui-monospace, Menlo, monospace` and this decision does not reach them. The stack there is shorter than the one in the register, and it was shorter before this record.

**A change to the register is one edit and one command.** The cost that moves is the cost of disagreement, which was eight files that nothing compared.

**This is the second check that reads `site/`, and no engine rule is either of them.** `.headwater/corpus.json` names `docs` as the one corpus root, so `headwater check` reads no byte under `site/`. A hand-built page can therefore state something this corpus has falsified while every gate stays green. `tools/refresh-crawler-files.sh --check` was the first shell check written to cover that gap, and this is the second. Both had to be written by hand for the same reason.

**This record states no value that a reader can see on the page.** It names where the values live. What the pages should look like is the first paragraph of the decision, and no rule of this engine reads it.
