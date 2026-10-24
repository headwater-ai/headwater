---
name: headwater-orient
description: Load first, before your first Read, Grep, Glob, Edit or Bash, on every task in this repository: a question about what it says or decided, a change to code or a script, a move or rename of files, or an edit to a document. Load it even when the task already gives the exact path, because a named document can be superseded and a file you are told to change can be governed by a contract that forbids the change. Load it also when a file you read cites a specification, a decision or a contract you have not opened. It shows how to run the engine here and the one command that names the documents governing the paths you will read or change.
---

# Orientation

The corpus is typed, and the engine holds a graph over it. So the cost of finding out what this repository already says about a subject is one command, and the cost of reading a long document to discover that it was the wrong one is most of a session's attention.

This skill is the first thing in a task, and it is finished as soon as the right document is open. It finds; it does not read for you.

## Run the engine

`headwater` may not be on PATH. Run `command -v headwater`, and if it prints nothing, run `ls */target/*/headwater` to find this repository's own build. Call the engine by the path that prints in every `headwater` command of this skill. Never fall back to grep because the bare name failed.

## Before you change a file

Run `headwater route "<the task, with every path it names>" --root .` before the first edit. Open each document whose line says `governs <path>` or names the thing you are changing. If that document rules the change out, say so instead of making it.

After the last edit, run `headwater check --fix --root .` and then `headwater check --root .`, and fix each finding on a file you changed. `--fix` writes the missing inverse half of a relation that you declared on one side only.

## When a file cites a document

A comment, a script header or a README that cites a specification, a decision or a contract by number is a pointer and not the ruling. Run `headwater route` with the path of the file that cites it. The line that says `governs <path>` names the document. Open that document before you act on what the comment says, and before you refuse the change because of it.

## Before you answer from a record

Run `headwater explain <path or identifier> --root .` on the record. A `superseded_by` edge, or a line that says another document governs the reading, means that the answer is in the other record. Open that record before you answer, even when the record you already read summarizes it, because a summary of the record in force is not the record.

## Before a procedure

For a move, a rename, a publish or a release, list the how-to shelf that `.headwater/corpus.json` names. Open the guide for the operation, whatever word its title uses for it, and follow it. Do this before `route` and before you load another skill, because a route names the documents that govern the paths and not the order of the steps.

## Start with what a hook already handed you

Where a hook of this repository runs `headwater route` on every prompt, read the documents it printed above the first tool call before you search for anything.

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

A search over the corpus is right for a literal string: a spelling, an identifier, a phrase in an error message, a term whose home is what you are trying to find out. It is wrong for a question about subject, status, authority or relationship, because none of those is written in the text of the document that carries them.

The test is whether you could name the file the answer is in. If yes, `explain` it. If no, and the question is about meaning rather than characters, `route` it.

## What orientation does not settle

`explain` reports the warrant of a document and never whether it is right. Contradictions between documents that each pass every rule are what a coherence sweep is for, and the `headwater-sweep` skill carries it. Whether the corpus owes something that nobody has written is a question for the same sweep, and for `headwater check` when a rule already reads it.
