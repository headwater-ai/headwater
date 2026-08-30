---
id: HW-DR-0030
status: current
status_since: 2026-08-30
summary: What an obligation waits on is a state and not a property, and the closed set holds four values. A required `waiting_on` facet on `obligation_record` takes `ruling`, `build`, `measurement` or `adopter`, and the decision-record bundle declares it.
last_verified: 2026-08-30
title: "Q30 — Whether what an obligation waits on is a state or a property, and how many values it takes"
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  governs:
    - docs/taxonomies/decision-record/bundle.yml
---

# Q30 — Whether what an obligation waits on is a state or a property, and how many values it takes

## Context

The obligations shelf holds 129 records. 121 stand at `status: current`, 4 at `status: draft`, and 4 at `status: discharged`. Every record states what the corpus owes and what would discharge it. No record states what it is waiting on, and no facet of any entry carries that.

**The shelf therefore answers one question with a word search.** `grep -il ruling docs/obligations/*.md` returns 30 of the 129 records. Read against the reading below, that proxy has a precision of 0.50 and a recall of 0.36. It matches 14 of the 28 open records it returns, and it misses 25 of the 39 open records that wait on a ruling. Two of its 30 hits stand at `status: discharged` and wait on nothing at all.

**The word is a proxy for the wrong tense.** A record that names a ruling usually cites one that already happened. [HW-OBL-0031](../obligations/0031-list-extension-in-an-add-only-overlay.md) and [HW-OBL-0049](../obligations/0049-the-meta-schema-closes-eight-value-sets-that-nothing-states.md) both invoke the ruling of [Q2](../spec/09-decisions.md#q2--schema-format) as a precedent for their own direction. The records that ask for a ruling and use no such word write a plain fork instead. [HW-OBL-0088](../obligations/0088-correcting-an-identifier-is-mechanical-and-it-is-not-local.md) reads *Either spec 12 qualifies its own example, or `check --fix` gains a class of fix that spans documents*.

**[Spec 13](../spec/13-open-obligations.md#a-human-maintains-this-list-by-hand) holds a written argument against a facet over this shelf, and it has to be answered here.** It says that the four editorial classes of that file would read as intrinsic if a facet carried one. It says that the fourth class holds what the first three leave, so its value means none of the other three. It names one casualty: [HW-OBL-0011](../obligations/0011-nothing-has-been-promoted-out-of-the-synthesized-tier.md) and [HW-OBL-0096](../obligations/0096-whether-anything-is-ever-promoted-out-of-the-asserted-tier.md) are one debt filed in two classes, and a facet would hold the duplicate rather than report it.

## Decision

**What an obligation waits on is a state, and the closed set holds four values.** A required facet `waiting_on` on `obligation_record` takes `ruling`, `build`, `measurement` or `adopter`. [The decision-record bundle](../taxonomies/decision-record/bundle.yml) declares both the facet and the requirement.

The four values name the act that comes next. `ruling` means that a person has to choose, and that no build and no measurement settles it. `build` means that the answer is known and somebody has to write the code or the prose. `measurement` means that an instrument exists and that no run of it has been made. `adopter` means that an outside corpus has to exist before anything can move.

**First, a state and not a property, because a property would be wrong about nearly the whole population.** Each of the 129 records was read for the act that comes next, and the reading also recorded the act after that. 38 of the 39 open records that wait on a ruling are followed by a build. One is followed by nothing, and none is followed by a measurement. So a fixed property that had to name one of the two would be wrong about 38 records. A present-tense state reads `ruling` today and `build` on the day the ruling lands. That is one edit, made by the person who is already editing the record.

**A property also has no way to record a record that moved.** The first half of [HW-OBL-0082](../obligations/0082-the-lock-is-half-generated-and-half-authored-and-nothing.md) dissolved on measurement rather than on work. `taxonomy resolve --check` exits 0 over a stale authored block by design, and `resolve` reads package sources and never the corpus. A value fixed at filing carries no such event. A state does.

**Second, four values and not two, because two of the four hold populations that a binary has no cell for.** 22 open records wait on an outside corpus. No build discharges one and no ruling discharges one. 8 open records wait on a run of an instrument that already ships. Under a binary both groups are forced into `build`, which is false, and they then crowd the query that the facet exists to answer.

**Third, no value of the four is a residual, which is the exact ground spec 13 refused a facet on.** A residual value means none of the others, so it changes on a record that nobody edited on the day a new class arrives. Each of these four names an act positively. A record that fits none of the four has a Discharge section that does not say what happens next. That is a defect in the record rather than a missing value. Two records met that test on the first reading, and this change writes the next act into each of them.

**Fourth, spec 13's objection reaches a different thing, and its own casualty comes out the other way.** The four classes of spec 13 are editorial and partly about provenance. The heading *what else each decision left open* names the decision that produced a record. `waiting_on` names neither. The pair that objection rests on takes one value here. HW-OBL-0011 and HW-OBL-0096 both read `measurement`, because no change of this repository has yet moved a warrant. The duplicate stays visible rather than being split across two classes.

**The two partitions are not the same partition, measured.** Each heading of spec 13 was read for the distinct obligation records it links. The largest is *design work that nothing blocks*, which links 91 records, and 37 of them wait on a ruling. The headline heading is *unmeasured claims*, which links 17, and 4 of those wait on a measurement while 8 wait on a build. The hand-maintained partition has drifted from its own criterion, and nothing reports that. Spec 13 keeps its six headings, its framing prose and its authored form, and this record asks for no projection at that path.

**A new value of `status` is refused.** `status` answers whether a reader may rely on this document. A record that waits on a ruling is fully `current`, because it states what holds now. The blocker belongs to the subject of the record and not to its authority, and one facet cannot hold two orthogonal things.

**A relation is refused, because there is no node at the far end.** A ruling that nobody has made is not a document, holds no identifier, and cannot be the target of an edge. A relation in this taxonomy points at a declared kind or a declared anchor and never at a bare string.

**A second document at `status: draft` on the decisions shelf is refused.** Six documents already stand there, and each one is written and awaiting acceptance rather than unanswered. That value would then carry two meanings in the one place they are indistinguishable. It also costs about 30 new documents against 129 front-matter lines, and it holds two documents about one thing.

**The declaration site is the decision-record bundle, and the engine decides that rather than taste.** Two arms were measured on this tree. An overlay that adds `waiting_on` to `add_to.kinds.obligation_record.facets.optional` exits 1. An overlay that adds it to `add_to.kinds.obligation_record.facets.require` exits 1 with the same reason: *does not commute with `add.kinds.obligation_record` in `docs/taxonomies/decision-record/bundle.yml`*. That is [HW-OBL-0031](../obligations/0031-list-extension-in-an-add-only-overlay.md) on a real change. The bundle is a tradition another repository takes whole, so the four values answer to any corpus that keeps obligation records.

## Consequences

**Every record carries a value, and the requirement is an error.** `facet.required.missing` is an error and `.githooks/pre-commit` runs `headwater check --strict`, so a required facet and an unmigrated corpus cannot both sit on a green `main`. The declaration and the 129 front-matter edits are one commit for that reason.

**`headwater new obligation_record` refuses to write a record without a value.** The facet declares a closed value set and carries no role that determines a value. So the scaffolder stops and names the four values. Every record filed from here states what it waits on, or it is not written.

**`headwater taxonomy audit` prints the distribution on every run, at no engine cost.** The reading this change writes down is `build` 59, `ruling` 40, `adopter` 22 and `measurement` 8 over the 129. Over the 125 open records it is `build` 56, `ruling` 39, `adopter` 22 and `measurement` 8.

**A discharged record takes a value and there is no fifth one.** A record leaves the queue by `status` and never by `waiting_on`, so on a record at `status: discharged` the value reads as what discharged it. That is a third instance of [HW-OBL-0123](../obligations/0123-a-facet-that-applies-to-one-value-of-another-facet-has-nowhere-to-say-so.md), which records that a facet applying to one value of another facet has nowhere to say so. This record adds the instance rather than inventing a value to route around it.

**The count is a count and not a list, and the reader that would name the records is defective.** A `shelf_index` projection with a filter over this facet validates, resolves and generates at exit 0, and it writes every document on the shelf. `Filter::admits` reaches one caller in the engine, which is the graph export. Worse, `facets_read` counts a projection filter as a reader for the relevance canon. So declaring the dead filter is what makes a facet pass the guard against an unread facet.

**The authority axis is not encoded, and encoding it would settle an open escalation.** 5 of the 39 open rulings sit beyond what an agent could propose and a reviewer accept. Three of those five are one question asked from three directions, which is who may accept a document, and [HW-OBL-0108](../obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md) is that question. A facet value that said *only the owner may rule this* would answer it by writing a value.

**The value set is closed at four, and closing it is what makes this a decision.** [Q2](../spec/09-decisions.md#q2--schema-format) settles the direction to guess in. A set relaxed later costs nothing, and a set widened later is a finding against every source that used the earlier form. [HW-OBL-0049](../obligations/0049-the-meta-schema-closes-eight-value-sets-that-nothing-states.md) records eight closed sets that nothing states, and this one is stated here rather than becoming the ninth.
