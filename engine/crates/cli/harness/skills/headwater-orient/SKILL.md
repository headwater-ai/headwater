---
name: headwater-orient
description: Find what this corpus already says about a subject, before reading any of it. Use at the start of a task that names a document, a decision, an obligation or a concept of this repository, when a question is about what was already decided, and whenever the alternative is a search across `docs/` or reading a specification part end to end. It answers from the graph the engine already built, and it never replaces reading the document it finds.
---

# Orientation

The corpus is typed, and the engine holds a graph over it. So the cost of finding out what this repository already says about a subject is one command, and the cost of reading a specification part to discover that it was the wrong one is most of a session's attention.

This skill is the first thing in a task, and it is finished as soon as the right document is open. It finds; it does not read for you.

## Start with what the hook already handed you

`.claude/hooks/intent.sh` runs `headwater route` on every prompt, and prints the documents it matched above the first tool call. Those pointers are the answer to "where does this subject live", already paid for. Read them before searching for anything.

The hook is silent when the route matched nothing, which is a result rather than a failure. Silence means the subject is not one a declared purpose answers, and a search is then reasonable.

## The three verbs

    headwater route "<the task, in the words it was asked in>" --root .
    headwater explain <path or identifier> --root .
    headwater check --root .

**`route`** takes a task and returns the documents whose declared purpose answers it. It matches purpose before it matches text, so it is right about subject and indifferent to vocabulary. It reports honestly when nothing matched.

**`explain`** takes one document and prints what the taxonomy knows about it: the kind and the shelf that carries it, the purpose, the summary, the warrant, the facets it must declare, every relation it may declare, and every edge in and out with the summary of the document at the other end. Two of those lines are why this verb replaces reading:

- the **summary** says what the document is for, in its author's words, without opening it
- the **edges** name every document that cites it, supersedes it, governs it or traces to it, each with its own summary

So one `explain` on a wrong guess still names the right document, and a second `explain` lands on it. That is two commands against a search that returns line numbers and no relationships.

**`check`** reports what is currently wrong with the corpus. Run it when the task is to fix something and the finding has not been named yet, rather than looking for the defect by reading.

## Where a cold session starts

`.headwater/corpus.json` names the entry points of this corpus: the document to open first on each shelf. It is written by `headwater generate` and held by `headwater generate --check`, so it is never stale in a way a run would not report.

Open an entry point when the subject is unknown. Use `explain` when a document is already named, even loosely, because a near miss is cheaper than a browse.

## When a search is the right tool

A search over `docs/` is right for a literal string: a spelling, an identifier, a phrase in an error message, a term whose home is what you are trying to find out. It is wrong for a question about subject, status, authority or relationship, because none of those is written in the text of the document that carries them.

The test is whether you could name the file the answer is in. If yes, `explain` it. If no, and the question is about meaning rather than characters, `route` it.

## What orientation does not settle

`explain` reports the warrant of a document and never whether it is right. Contradictions between documents that each pass every rule are what a coherence sweep is for, and [headwater-sweep](../headwater-sweep/SKILL.md) carries it. Whether the corpus owes something that nobody has written is a question for the same sweep, and for `headwater check` when a rule already reads it.
