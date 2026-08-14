---
name: headwater-sweep
description: Run a coherence sweep over a slice of this repository's corpus — read the documents and report contradictions, quiet supersessions, undefined concepts, audience mismatches and unwritten sections that no check can see. Use when asked to sweep the corpus, audit it for coherence, look for documents that disagree, or find out what the linter cannot. It reaches a model, which is you, and its output is a proposal a person accepts rather than a verdict anything gates on.
---

# Headwater coherence sweep

A sweep is the mechanism for what no rule reads. The check layer decides cohesion: links that land, facets that hold, identifiers that bind. Coherence is whether the corpus adds up to one account, and [spec 4](../../../docs/spec/04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep) says that structural checks cannot discharge it.

**You are the middle part of the mechanism, and there is no other.** The engine writes a briefing, you read the documents, and the engine reads your file back. Nothing in the engine reaches a model. If you do not read the documents, nothing does.

**Nothing gates on your output, and that is deliberate.** `headwater sweep report` exits 0 whatever it finds, no gate and no CI job runs it, and a run of it can never fail a build. Your output is a proposal that a person reads and accepts. So do not soften a finding to be safe, and do not pad the list to look productive. A finding nobody can adjudicate in seconds is noise, and noise is what makes a team turn the whole sweep off.

## The three steps

### 1. Ask for the briefing

    headwater sweep plan --under docs/spec

`--under` is the slice, as a path prefix. There is no sampling rule and the engine will not pick one for you: name the corner of the corpus that is worth the reading. Without the flag the slice is the whole corpus, which is usually more than one sweep should claim.

The briefing gives you every classified document in the slice with its identifier, kind, title and summary, plus **every edge the graph already declares between two members**. It also states the extent: how many documents are in the slice against how many are in the corpus.

### 2. Read the documents

Open them. All of them. The briefing carries pointers and no prose, because a sweep whose agent never opened a file cannot cite a passage, and a citation is the only part of your work the engine can confirm.

Report only these five classes:

- **undeclared_conflict** — two documents contradict each other, both are current, and neither says so. A conflict that is already declared is a deterministic check, so the value is entirely in the ones nobody noticed.
- **quiet_supersession** — a newer document has overtaken an older claim, and the older one still reads as live.
- **undefined_concept** — a term is used across the slice and defined in none of it.
- **audience_mismatch** — the audience the document declares could not act on what it says.
- **unwritten_section** — a heading the kind requires, over prose that says nothing about it. [OBL-repo-0113](../../../docs/obligations/0113-every-check-passes-a-document-that-is-still-the-scaffolder-s-placeholder.md) is why this class exists: a decision record whose sections each read the scaffolder's own prompt passed every check.

Two rules bound what you may report.

**Never report an edge the briefing already lists.** A finding that restates the graph is a defect in the sweep rather than a fact about the corpus. The intake refuses one whether or not you read the briefing, so this rule costs you a finding rather than catching one.

**Quote, and never paraphrase.** The engine searches the document for every quotation you attribute to it, with runs of whitespace collapsed and nothing else relaxed. A near-miss is refused and never reaches the person who asked for the sweep. Copy the passage.

### 3. Write the file, then hand it back

Write YAML in the shape the briefing prints, then run:

    headwater sweep report <path>

The report tells you what was carried and what was refused, and it names the test each refusal failed. Read it. A refusal is about your work rather than about the corpus, and the four it can report are a quotation the document does not hold, a path that is not a classified document, a class outside the five, and a proposal the graph already declares.

## The best outcome is an edge, and you do not write it

Spec 4: the best outcome of a sweep is not a finding but a declared `conflicts_with`. After the edge is declared the engine owns the problem permanently and no sweep needs to find it again. So propose one whenever the class implies one, in the `proposal` block of a finding.

Propose it and stop there. `headwater sweep report` prints the front matter that would declare the edge and it writes nothing, and there is no flag that makes it write. Do not apply the proposal yourself either. [OBL-repo-0108](../../../docs/obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md) records what happens when an agent accepts its own work at the scale of a corpus, and a proposal you apply to yourself is that same act.

## What the engine confirms, and what stays yours

It confirms the citation and the novelty. Every quotation is really in the document you named, every path is really a classified document, and the graph does not already carry the edge you proposed.

It confirms nothing about whether you opened the slice. The extent the report states is the slice that was asked for, not the set you read, and nothing can tell the two apart. That is on you.

It confirms nothing about whether you are right. Whether two passages contradict each other is your reading, the report says so on the line that carries it, and a second sweep may not return the same set at all. That is why absence means nothing here: a slice you did not sweep and a slice with no problem produce the same silence.
