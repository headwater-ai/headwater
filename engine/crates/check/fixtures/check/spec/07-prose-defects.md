---
id: SPEC-FIX-prose-defects
doc_type: design_spec
status: current
status_since: 2026-01-05
summary: The failing fixture that the prose rules need, with one defect of each kind.
---

# Prose defects

Every defect below is deliberate, and each one is the failing half of the pair that spec 12 requires. Read this file beside `00-both-halves.md`, which is the passing half for the same rules.

## Voice

The resolver will be rewritten once the lock format settles.

We renamed the field, and the old spelling stays valid.

For now the engine reads one profile, and the rest arrive later.

## Language

This sentence is written to run past the limit that the house profile sets for descriptive text, and it keeps going for long enough to make the point that a reader loses the thread of a clause well before the twenty-sixth word arrives.

It doesn't expand the contraction, which the profile refuses.

The behaviour of the resolver is stated in the paragraph above.

The resolver reads one profile; the rest belong to the adopter.

The two sources agree (Smith, 2004; Jones, 2007) about the shape, and a semicolon inside a parenthesis is not a run-on.

## The three spans a patch must not land at

A code span carries one and no rule reads it: `it doesn't matter` is a name rather than prose.

> A quoted author writes what they write, and it doesn't answer to this regime.

A link's text is this author's own prose, so [it doesn't escape](00-both-halves.md) the rule.

## Retired terms

The reference system resolves each name, which is the retirement that names a replacement.

The lock format is robust, which is the retirement that names none.

## A heading is not running prose; nor is a table cell

| operation | what it needs |
| --- | --- |
| add | the path is absent; the parent exists |

## Source form

This paragraph is written over
two lines, and the source form
of this regime forbids that.

## Links

A fragment that lands nowhere: [the missing section](#no-such-section).

A path that lands nowhere: [the part that moved](19-renamed.md).

A path that carries a fragment, which the path rule reads and the fragment rule does not: [one heading of it](19-renamed.md#a-heading).

A path that will not normalize into a repository path at all: [a destination above the root](../../../outside.md).

A query string on a path that lands nowhere: [the part that moved, with a version on it](19-renamed.md?v=2). The query string leaves the destination before the binder reads the path, so the report names the file and never the query.

An address written from the site root, where the root of this corpus holds no such file: [an absolute address](/spec/00-both-halves.md). The binder reads a leading separator against the root of the corpus. Nothing stands there, so the report names the destination as the author wrote it.
