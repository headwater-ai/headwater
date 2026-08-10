#!/usr/bin/env python3
"""Self-test for ste-lint. Run: python3 tools/test_ste_lint.py

No test framework: this must run anywhere the pre-commit hook runs.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import importlib.util

spec = importlib.util.spec_from_file_location(
    "ste_lint", Path(__file__).resolve().parent / "ste-lint.py")
ste = importlib.util.module_from_spec(spec)
sys.modules["ste_lint"] = ste  # dataclasses resolves annotations through sys.modules
spec.loader.exec_module(ste)

FAILURES = []


def check(name, markdown, expected, forbidden=(), profile="house"):
    rules = {f.rule for f in ste.lint("docs/spec/t.md", markdown, profile)}
    missing = set(expected) - rules
    present = set(forbidden) & rules
    if missing or present:
        FAILURES.append(
            f"{name}: missing={sorted(missing) or '-'} unexpected={sorted(present) or '-'} "
            f"(got {sorted(rules) or 'nothing'})")


# --- things that must be caught ---------------------------------------------

check("hard wrap in a paragraph",
      "A first line of prose.\nA wrapped continuation.\n", ["hard-wrap"])
check("hard wrap in a list item",
      "- An item of a list.\n  continued on the next line.\n", ["hard-wrap"])
check("hard wrap in a blockquote",
      "> A quoted paragraph.\n> wrapped onto a second line.\n", ["hard-wrap"])
check("semicolon in prose",
      "The check runs; the finding appears.\n", ["semicolon"])
check("contraction",
      "The engine doesn't read the file.\n", ["contraction"])
check("british spelling",
      "The organisation records its behaviour.\n", ["british-spelling"])
check("british spelling in an inflected -our word",
      "OpenGEO solves a neighbouring problem on Markdown.\n", ["british-spelling"])
check("american -or words are left alone",
      "The neighboring color of the labor it favors.\n", [], ["british-spelling"])
check("stock phrase",
      "The identifier is load-bearing for the corpus.\n", ["stock-phrase"])
check("leverage as a verb",
      "The engine leverages the graph to find conflicts.\n", ["stock-phrase"])
check("sentence at the limit passes",
      "This one sentence runs past the twenty-five word limit because it keeps "
      "adding clauses and more clauses and yet more clauses until it finally stops.\n",
      [], ["sentence-length"])
check("over-long sentence",
      "This one sentence runs past the twenty-five word limit because it keeps "
      "adding clauses and more clauses and yet more clauses and still more "
      "clauses until it finally stops.\n",
      ["sentence-length"])
check("paragraph with too many sentences",
      "One. Two. Three. Four. Five. Six. Seven.\n", ["paragraph-sentences"])
check("progressive verb form",
      "The engine is checking the corpus.\n", ["progressive"])
check("passive voice",
      "The finding is recorded by the engine.\n", ["passive"])
check("auxiliary construction",
      "The document must be reviewed.\n", ["auxiliary"])

# --- things that must NOT be caught ------------------------------------------

check("fenced code is skipped",
      "Prose line.\n\n```python\nx = 1; y = 2  # doesn't wrap\nprint(x)\n```\n",
      [], ["semicolon", "contraction", "hard-wrap"])
check("indented code is skipped",
      "Run the command:\n\n    pdftotext -layout in.pdf out.txt; echo done\n",
      [], ["semicolon", "hard-wrap"])
check("front matter is skipped",
      "---\nname: thing\ndesc: it's fine\n---\n\nProse line.\n",
      [], ["contraction", "hard-wrap"])
check("consecutive list items are not wraps",
      "- First item.\n- Second item.\n- Third item.\n", [], ["hard-wrap"])
check("nested list items are not wraps",
      "- First item.\n  - Nested item.\n", [], ["hard-wrap"])
check("table rows are not wraps",
      "| A | B |\n|---|---|\n| one | two |\n", [], ["hard-wrap"])
check("semicolons in table cells are punctuation",
      "| Editor | Any editor; the corpus is Markdown |\n", [], ["semicolon"])
check("semicolons in headings are punctuation",
      "## Kinds are rigid; states are not\n", [], ["semicolon"])
check("semicolons in a citation are punctuation",
      "The work (Gotel et al., 2012; revisited 2017) carries the lesson.\n",
      [], ["semicolon"])
check("a verbatim quotation keeps its own spelling",
      'They define them as "plastic enough to adapt, yet robust enough to '
      'maintain a common identity".\n',
      [], ["stock-phrase", "british-spelling"])
check("hyphenated leverage is a noun",
      "| The single highest-leverage rule | detectable |\n", [], ["stock-phrase"])
check("possessives are not contractions",
      "The corpus's identifier and the engine's output.\n", [], ["contraction"])
check("abbreviations do not end a sentence",
      "Use a short name, e.g. a slug, i.e. one word, for the identifier.\n",
      [], ["paragraph-sentences"])
check("a bibliography line is a list, not a sentence",
      "Discourse and coherence — Halliday & Hasan, *Cohesion in English* (1976) · "
      "Mann & Thompson, *Rhetorical Structure Theory* (1988) · Hobbs, *On the "
      "Coherence and Structure of Discourse* (1985) · Kehler, *Coherence, "
      "Reference, and the Theory of Grammar* (2002)\n",
      [], ["sentence-length", "paragraph-sentences"])
check("a bibliography line still gets the word-level checks",
      "Sources — Smith, *Organisation of Things* (1990) · Jones, *More* (1991) · "
      "Patel, *Even More* (1992)\n",
      ["british-spelling"])
check("one interpunct is not a reference list",
      "The engine reads the view · and this one long sentence must still be "
      "counted because it keeps going and going and going well past the "
      "stated limit.\n",
      ["sentence-length"])
check("a parenthetical counts as one word",
      "The engine reads the view (which carries the scoped subset of the corpus "
      "graph, plus every declaration that applies to it) and returns.\n",
      [], ["sentence-length"])
check("an inline allow comment suppresses its rule",
      "The engine doesn't read the file. <!-- ste-lint: allow contraction -->\n",
      [], ["contraction"])
check("an inline allow comment takes a reason and a list",
      "The engine doesn't record its behaviour. "
      "<!-- ste-lint: allow contraction, british-spelling # quoting the source -->\n",
      [], ["contraction", "british-spelling"])
check("an inline allow comment suppresses only the named rule",
      "The organisation doesn't read it. <!-- ste-lint: allow contraction -->\n",
      ["british-spelling"], ["contraction"])
check("a deliberate hard break is not a wrap",
      "A first line of prose.\\\nA deliberate second line.\n", [], ["hard-wrap"])

# --- sentence and word counting ----------------------------------------------

if ste.count_words(ste.split_sentences(
        "The engine reads (a long parenthetical here) and stops.")[0]) != 6:
    FAILURES.append("count_words: parenthetical should count as one word")
if len(ste.split_sentences("One thing. Two things. Three.")) != 3:
    FAILURES.append("split_sentences: should find three sentences")
if len(ste.split_sentences("Version 2.5 ships, e.g. in Q3. Then it stops.")) != 2:
    FAILURES.append("split_sentences: decimals and abbreviations must not split")
if len(ste.split_sentences("It fails on maintenance cost. Links decay anyway.")) != 2:
    FAILURES.append("split_sentences: 'St.' must not match inside 'cost.'")
if len(ste.split_sentences("It documents the problems. Nobody reads them.")) != 2:
    FAILURES.append("split_sentences: 'Ms.' must not match inside 'problems.'")
if len(ste.split_sentences("Families exist first. The rest came later.")) != 2:
    FAILURES.append("split_sentences: 'St.' must not match inside 'first.'")
if len(ste.split_sentences("B exists in service of A. That is the relation.")) != 2:
    FAILURES.append("split_sentences: a single letter can end a sentence")
if ste.restore(ste.protect("the maintenance cost.")) != "the maintenance cost.":
    FAILURES.append("protect/restore: must not corrupt the casing of 'cost.'")
if len(ste.split_sentences("does it classify? does it pass? do they match?")) != 3:
    FAILURES.append("split_sentences: a run of lower-case questions must split")
if len(ste.split_sentences('It answers "does this prevent?" — which is a filter.')) != 1:
    FAILURES.append("split_sentences: a quoted question must not split")

check("a sentence may open with a lower-case link",
      "To detect that two documents disagree is to reason about what prose asserts. "
      "[spec 1](01-conceptual-model.md) forswears that, and [spec 4](04.md) confirms "
      "that it is undecidable structurally.\n",
      [], ["sentence-length"])
check("a sentence may open with a section mark",
      "Every integration recorded so far points out of the corpus — TrustGraph "
      "ingestion, the OKF bundle, and the LinkML and SKOS emissions. §J prices "
      "TrustGraph as cheap on exactly that ground: nothing flows back in.\n",
      [], ["sentence-length"])

if FAILURES:
    print(f"{len(FAILURES)} test(s) failed:")
    for failure in FAILURES:
        print(f"  - {failure}")
    sys.exit(1)
print("ste-lint self-test: all checks pass")
