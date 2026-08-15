---
id: DR-repo-0024
status: draft
status_since: 2026-08-15
last_verified: 2026-08-15
summary: Readability is not a sweep class and gets no verb, because every class of a sweep names two things that do not fit and a readability finding names one. A source file is not a slice member, because admitting one degrades the membership refusal for every class.
title: "Q24 — Readability, and what a sweep can be asked about"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - engine/crates/sweep/src/lib.rs
    - engine/crates/sweep/src/intake.rs
    - .claude/skills/headwater-sweep/SKILL.md
---

# Q24 — Readability, and what a sweep can be asked about

## Context

`engine/**/*.rs` carries about 179,000 words of comment prose, which is more than the governed prose under `docs/spec/`. [#191](https://github.com/headwater-ai/headwater/issues/191) measured it and reports that it is hard to read. The habit it names is definition by contrast: the prose states what a thing is not and then corrects to what it is. `rather than` appears 979 times, which is once every 183 words, on top of 428 inline quotations stitched into host sentences and 1,082 clause-introducing colons. Every instance is defensible and the aggregate is the problem, so no single instance is a finding.

**The check layer is the wrong instrument, and the measurement says so.** Every prose rule this engine has was run over the comments. They hold zero contractions and zero British spellings across 179,000 words that no rule polices. The two rules whose remediation is mechanical and total, which is the [fixability](../spec/12-check-layer.md#fixability) bar, are already met by habit, so extending them to `engine/` buys a rule that lands green and stays green. The one rule the comments miss is the advisory sentence limit, and it is advisory because the remedy is a rewrite.

**That leaves the one mechanism here that no engine performs.** [Spec 4](../spec/04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep) declares the assisted sweep, and its shape is the shape this work wants: the engine writes a briefing, a model reads the documents, and `headwater sweep report` confirms what it can while leaving the judgment to a person. Two things stop it reaching a comment. The five classes are coherence classes, and the intake refuses a class outside them. The slice is classified documents under a path, `corpus.root` is `docs`, and a `.rs` file has no front matter and therefore no kind.

Whether either is an accident of scope or a property of the mechanism is the question this record answers.

## Decision

**Readability is not a class of the sweep, and it gets no verb of its own. The work is unmechanized, and a person does it with the `ste-editor` skill.**

The discriminator is not the intake. Three of the five classes — `undefined_concept`, `audience_mismatch` and `unwritten_section` — already return `None` from `Class::implies`, so the novelty leg is vacuous for them, and a readability class would sit beside them with the same four confirmations available. The argument that the intake refuses it is available and it is wrong.

What separates them is the shape of the claim. **Every class of this sweep names two things and says they do not fit.** Two documents that contradict each other. A newer claim against an older one. A term used across the corpus against the place its definition is owed. A document against its declared audience. A heading against the prose beneath it. A reader holds one against the other and adjudicates in seconds, which is the standard spec 4 sets. `Class::remediation` reads the same way: every remedy declares an edge, defines a term, narrows an audience or writes a section, and each one changes what the corpus states.

**A readability finding names one thing.** There is no second term to hold it against, the corpus states the same claims on either side of the rewrite, and the quotation is therefore the subject of the finding rather than evidence for it. That is what makes the engine's confirmation empty: for a relational class the intake proves that a passage a person is about to weigh is real, and for readability it proves only that a passage exists, which is what `grep` proves. A sweep whose verification reduces to `grep` is a sweep in name.

**A source file is not a sweep slice. A slice member is a classified document, and that requirement is not a formality.**

`intake::classified` refuses a finding whose path is not a typed row of the census, and the crate states what that refusal is for: "a path that is not there at all is the plainest sign that a model invented it". Over a tree where any file may be a member, that refusal weakens to "the file exists", which a model satisfies by naming any path in the repository. **Admitting a source file therefore degrades an existing refusal for every class, and not only for the class that wanted it.** That cost is paid by `undeclared_conflict` and the other four, which asked for nothing.

Two smaller consequences point the same way. `Plan::over` reports its extent as classified documents in the slice against classified documents in the corpus, and the plan carries that number so that no report can lose it; a slice that may hold unclassified files makes "12 of 169" two populations under one ratio. And `Member::kind` is not an `Option`. A member with no kind is a path and a title, and a kind is what tells the agent what the document is for.

**The tic counts are a map and never a rule.** No count in #191 becomes a `retired_terms` entry, an added rule or a threshold. `rather than` is correct English and this record uses it. The counts locate the work and judgment performs it, which is the whole distinction the question turns on.

## Consequences

**What reopens the first ruling.** A second term. If a readability finding can name something in the corpus that the passage must cohere with, and the engine can name that thing too, then the finding is relational and the argument above no longer holds. A declared house-style profile would be such a term. Note where that lands: a profile the taxonomy declares is read by a check, and spec 12 already carries the check that reads one, advisory because its remedy is a rewrite. So the reopening path leads back to the check layer rather than to a sixth class, and the person who reopens it should say why the check layer is the wrong place a second time.

**What reopens the second ruling.** A census that types something other than a document. Today `Outcome::Typed` requires front matter, and a `.rs` file cannot carry any, because `split` reads the first line of the file. A kind whose instances are source files — declared, placed on a shelf, and reaching the census by some route other than front matter — makes membership a real test again, and the refusal above stops being degraded. Nothing proposes one.

**What the editorial pass covers, and what it must not lose.** These comments carry citations into the specification and the reasons behind decisions, so a pass that trades a reason for a shorter sentence makes the corpus worse. The standard is the worked example in #191: fewer words, same claims. The moves are the house profile's, and no check reads any of them — cut the navigation, unnest the quotation, name the actor, split the causal tail.

**One defect of the comments is mechanical, and it is now held.** Twenty-nine relative links from comments into `docs/` did not resolve: twenty at the wrong `../` depth, six naming a heading that was retitled, and three naming a document that was. Nothing read a `.rs` comment, so they rotted in silence. `headwater_check::fragment::comment_links` holds the whole set now, and it calls the slugger the `link.fragment.unresolved` rule calls, so a corpus link and a comment link resolve by one rule or by neither.

**A count taken outside the engine undercounts this population.** The script in #191 reported 448 links and a second one written for this change reported 452, and both match a link label on one line. The suite reports 456, because a Markdown reader accepts a label wrapped across two comment lines and a regular expression over one line does not. None of the four is broken today. Had one been, both scripts would have reported green.
