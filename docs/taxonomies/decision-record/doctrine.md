# The decision-record taxonomy

One decision, one document, kept forever. A team writes down the choice it made, the forces that produced it, and what the choice costs. It never edits the document afterward. When the choice changes, a new document replaces the old one and says so, and the old one stays where it is.

That is the architecture-decision-record tradition. It is the most widely practiced structured-documentation convention in software teams, and it is the second entry in this library.

This entry is also the remedy that [13 — Open obligations](../../spec/13-open-obligations.md) prescribes under *Identity below the grain of a document*. Two registers here hold their entries as sections of one file, so a decision is an anchor and an obligation is a bold paragraph. Neither is a document, neither reaches the identifier index, and no edge names either. The remedy is to convert each entry into a document, "which is a different tradition with a taxonomy of its own". This is that tradition, and the [migration](#what-a-migration-inherits) is separate work.

The item that names that remedy has no anchor, which is why the sentence above names it in prose. An entry in either register is a paragraph and never a heading, so a link into one does not resolve. That is the defect stated in the coordinate a reader meets it in.

## The prior art

**Nygard, 2011.** [*Documenting Architecture Decisions*](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions) is the post that started it. It supplies the three sections that everything after it copies: Context, Decision, Consequences. It also supplies the two rules that matter more than the sections. One decision per document, and a document is never edited once the decision is taken. A superseding record points back at what it replaced.

**MADR.** [Markdown Any Decision Record](https://adr.github.io/madr/) generalizes the form past architecture and adds the parts that Nygard left to the author: a title that states the decision, a status, the options that were considered, and the outcome with its justification. It is the form that most tooling now writes.

**adr-tools and log4brains.** [adr-tools](https://github.com/npryce/adr-tools) is a shell scaffolder. `adr new` writes a numbered file from a template, and `adr new -s 9` writes the successor and edits both records to name each other. [log4brains](https://github.com/thomvaill/log4brains) does the same and publishes the log as a site. Both are evidence for one claim of this design: the succession edge is written by a scaffold, and the reciprocal half comes free when a tool writes both ends. Nobody maintains that edge by hand at any scale, and neither tool asks anybody to.

**Kruchten.** The relation vocabulary of [spec 2](../../spec/02-taxonomy-model.md#the-decision-relation-vocabulary) comes from Kruchten's ontology of architectural design decisions, and it reached this design from the research literature rather than from the tooling above. The two arrive at the same place from opposite directions. `Obsoletes:` in an RFC, `Superseded by` in an ADR and `supersedes` in Kruchten are one relation under three names. The doctrine reconciles them by adopting the published set and by claiming none of it for this entry.

## What the tradition converges on

**The record is immutable after acceptance.** This is the property that separates the tradition from a wiki page about a decision. An accepted record states what was decided at a date, under the forces of that date. To edit it is to make every citation of it unreliable, because the reader of the citation and the writer of it then read two different texts.

**Succession is the only amendment.** A later record replaces an earlier one and names it. The earlier one stays, and it names its successor. That is what lets a reader who arrives at the old decision learn that it fell.

**One decision per record.** The temptation to violate it is constant, and the cost is exact. Two decisions in one record cannot be superseded apart. The first of them to fall takes the second with it, or it leaves the second alive under a heading that nobody reads.

**The log is read as a log.** A reader scans it by title and by date. That is the property that this entry meets a wall on, and [the title facet](#the-title-facet-and-what-it-does-not-reach) is what it does about it.

## The base package already carries most of this tradition

The base declares `kinds.decision` with `purpose: rationale`, `voice: declarative`, `lifecycle: standard`, an identifier scheme, and `sections: {require: [Context, Decision, Consequences]}`. Those are Nygard's three sections. The shelf `docs/decisions/**` beside it is homogeneous, which is one decision per document expressed as placement.

**So this entry adds no decision kind.** A `decision_record` kind beside the base `decision` would be two live paths for one tradition, and an adopter of both would meet a base kind and a base shelf that nothing fills. The [design-spec entry](../design-spec/doctrine.md#findings) had to do that for `specification`, and it recorded the cost as a finding. This entry does not repeat it, and the reason it can is a property of `add` rather than good luck. An `add` asserts that its addressed key is absent and asserts nothing about the mapping above it ([spec 2](../../spec/02-taxonomy-model.md#customization-by-composition)). The base `decision` declares no `facets` member, so a bundle may write one.

What is left for this entry is what the base does not have: a readable name, the other register's kind, the edge that closes an obligation, and the two invariants that no check enforces.

## The kinds

| Kind | Purpose | Shelf | Where it comes from |
|---|---|---|---|
| `decision` | `rationale` | `docs/decisions/**` | the base, with a `title` requirement added |
| `obligation_record` | `obligation` | `docs/obligations/**` | this entry |

**An obligation record is not a decision, and it needs a kind of its own.** The two registers of this repository sit beside each other and look alike, so the question is worth answering rather than assuming. A decision is closed: somebody chose, and the record states what was chosen and what it forecloses. An obligation is open: the corpus owes something, and the record states what would discharge it. Their reader intents differ, and [spec 2](../../spec/02-taxonomy-model.md#purpose-is-declared-not-implied) declares purpose per kind. A reader who asks "what is missing?" and reaches a register of settled choices got the wrong answer.

The sections say the same thing in the other direction. `Context, Obligation, Discharge` mirrors `Context, Decision, Consequences` deliberately. The middle section states the thing itself, and the third states what follows. What follows from a decision is its cost, and what follows from an obligation is the instrument that would pay it.

**The word `obligation` names two things in this model, and the kind name keeps them apart.** A taxonomy declares `obligations` as one of the thirteen declarations: an invariant that the corpus commits to, with a severity and a disposition ([spec 4](../../spec/04-assurance-model.md#obligations-are-data)). This kind is a document about work that a corpus owes. The two are unrelated, and both are called an obligation in the specification today. So the kind is `obligation_record` and its identifiers read `OBL-`, which leaves `OB-` to the declaration. The finding is below.

### The title facet, and what it does not reach

`docs/spec/README.md` is generated, and it labels each document with `SPEC-HW-vision-and-scope` where the table it replaced read "Vision and scope". [#123](https://github.com/headwater-ai/headwater/pull/123) measured that and recorded the cause: the taxonomy declares `summary`, which carries the scent role, and it declares no name. A summary is a sentence, and a reader who scans a log wants a name.

A tradition whose whole point is a readable log has to answer this, so `facets.title` is here. It is `stable` rather than `mutable`, which is the tradition's own claim: an accepted record is amended by succession, so its name changes by rename and never by edit.

**The facet reaches this entry's kinds and the base `decision`, and it reaches nothing else.** The gap that #123 measured is on the specification shelf, and the design-spec kinds already declare `facets.require` as a list. An `add` cannot reach into a list that exists, and a bundle holds no `override`. So the entry that met the defect cannot fix it where it was found. The general remedy is a `name` role in the base facet registry, which is a meta-schema change, and the finding below carries it.

## The lifecycle ladder maps, and two of its rungs do not

The tradition's ladder is the reason this entry was chosen second. Proposed, then accepted, then superseded or deprecated, is what an ADR log is for. The base declares four states in `vocabularies.lifecycle_state`, and the transitions of `regimes.lifecycle.standard` are the same graph.

| The tradition | The base state | What it means here |
|---|---|---|
| Proposed | `draft` | Written, complete, and awaiting a ruling |
| Accepted | `current` | The team decided, and the record is now immutable |
| Superseded | `superseded` | A later record replaced it, and names it |
| Deprecated | `deprecated` | The decision no longer holds, and nothing replaced it |

**This entry adds no lifecycle state and no lifecycle regime**, for the reason the [design-spec entry](../design-spec/doctrine.md#the-lifecycle-ladder-maps-onto-the-base) found first. The states are a list under `vocabularies.lifecycle_state`, and an `add` introduces a key rather than an element. Two rungs pay for that.

**The names are the tradition's and the corpus cannot write them.** A team that keeps ADRs writes `proposed` and `accepted`. Under this entry it writes `draft` and `current`. The core is satisfied either way, because the core is semantic and the roles survive the renaming ([spec 2](../../spec/02-taxonomy-model.md#the-immutable-core)). What is lost is recognition, which is criterion 1 of the [admission criteria](../README.md#admission-criteria): somebody who works in the tradition is meant to recognize it.

**`rejected` collapses into `deprecated`, and the two are different facts.** A rejected record was proposed and refused, so nothing ever relied on it. A deprecated record was accepted and later abandoned, so things did rely on it and may still. A reader who meets `deprecated` cannot tell "we considered this and said no" from "this used to hold". Both are terminal-retained, which is why the collapse is silent rather than an error.

The fix is one element in one list, and no bundle may write it. The finding is the second data point for an item that [13 — Open obligations](../../spec/13-open-obligations.md) already carries, so it sharpens that item rather than opening a new one.

## Relations, and the one this entry adds

**Succession and adjudication belong to the base, and this entry claims neither.** [Q18](../../spec/09-decisions.md#q18--recording-adjudicated-disagreements) ruled that an adjudication is a decision carrying `overrides` against the document whose effect it displaces, and spec 2 carries the whole Kruchten set with families assigned. The tradition that invented succession does not get to redeclare it.

One correction to the reading that this issue was filed under. It records that `overrides` needs no bundle, because "one line of overlay enables it" through `$package.optional.overrides`. That line does not resolve today. The base package manifest declares no `optional` block, nothing on disk holds the eight unenabled relations, and the resolver refuses a `$package` reference and names the open question as the reason. Enabling `overrides` therefore means writing the declaration out in full, in an adopter overlay. That is a fact about the package library rather than about this entry, and [13 — Open obligations](../../spec/13-open-obligations.md) already carries the shape of `optional` as an open item.

**A decision reaches its evidence through the base `traces_to`.** The relation is in the evidence family and runs from a governed document to a governed document, so a decision that names the evaluation which settled it needs no declaration here. An obligation record reaches the decision that produced it the same way, which is the structure that spec 13 already has in prose: each item sits under the decision that left it behind.

**One relation is new, because one event has no form.** An obligation is discharged when an instrument runs and reports. That is not a citation and it is not a derivation. It is the event that closes the entry, and this corpus has seventeen unmeasured claims waiting for it.

```yaml
relations.discharges:
  family: evidence
  from: [evaluation]
  to:   [obligation_record]
  inverse: discharged_by
  reciprocal: required
  created_by: hook
```

The creator is a hook for the reason the design-spec entry gave for `cites_evidence`. One commit adds the evaluation and closes the obligation, and the commit is what proposes both halves.

**This entry declares no participation expectation, and the absence is the finding.** An expectation states that a document of a kind, in a state, should acquire a named relation inside a window. The two failures this tradition actually has are neither. An accepted record that somebody edited has all its edges. A proposal that nobody ruled on is a document that never left a state, and no edge is missing from it. Spec 2 requires a window on every expectation, and an honest window does not exist for the other candidate either: an obligation that waits on a first adopter waits for as long as it takes, and this corpus says so about nine of its items.

## The two invariants, declared as obligations with no control

Spec 4 makes an obligation data, and it gives every obligation exactly one disposition. An obligation that no control discharges states a gap with an owner, or states that no mechanism can exist. This entry declares three, and not one of them has a check.

**`OB-DR-1`, immutability after acceptance.** The tradition's central rule, and a check class that nothing else in this library needs. It reads a content change against a lifecycle state, so it needs the prior version of the document. [Spec 12](../../spec/12-check-layer.md) already supplies that input as `needs_prior`, which is what makes this a gap with a named target rather than a wish.

**`OB-DR-2`, a proposal reaches a ruling.** The reported failure of ADR practice is a log that fills with proposals. The rule is over state dwell time, which `taxonomy audit` already measures and no check reads.

**`OB-DR-3`, one decision per record.** This one is `unverifiable` rather than a gap, and the reason is the boundary that [spec 1](../../spec/01-conceptual-model.md#two-layers-terminology-and-assertions) draws. To count the decisions in a document is to reason about what its prose asserts. The homogeneous shelf and the section contract each carry part of the rule, and a second decision written under the same `Decision` heading is invisible to both.

Declaring the three costs nothing and buys two things. The register states what this taxonomy holds itself to, which is the whole reason obligations are data. And a later engine that gains the first two rules finds the obligations already written, with the argument beside each.

## The core, declared

Criterion 5 of the [admission criteria](../README.md#admission-criteria) asks each entry to state its relationship to the invariant core. This is a full entry, and it satisfies the core through the base.

- **`state`, `state_entered`, `freshness` and `scent`** are the base facets `status`, `status_since`, `last_verified` and `summary`. Both kinds inherit all four from `governed_document`. `title` carries no engine role, and it claims none.
- **`rationale`** is served by the base `decision`, which this entry reuses rather than replaces.
- **`behavior`** is served by the base `specification`. This entry declares no behavior-serving kind, and it does not need to, because it removes none.
- **Succession** stays lifecycle-sensitive and untouched.

`obligation` is a fourth purpose, and the design-spec entry declares it. This entry uses it and declares `requires: [design-spec]`, which is the next section.

## What this entry deliberately does not declare

**A projection.** The log wants an index by date and by state, and `projections` is a list in the base. The same rule that blocks a lifecycle state blocks this. It also blocks the artifact that made this issue urgent: the redirect map of [#68](https://github.com/headwater-ai/headwater/issues/68) is a projection, and it has to come from an adopter overlay rather than from this entry.

**A second identifier scheme for decisions.** The base declares `decision_id` as `DR-{namespace}-{seq:04d}` and binds it to the kind. A tradition that renumbers is a tradition that breaks citations.

**A language regime, or a voice regime.** The base declares `declarative` and binds it to `decision`, which is right for this tradition: a record states what was decided, and it does not narrate the argument as it unfolds. An adopter that wants a controlled profile binds one in its own overlay.

**A template placeholder syntax.** The templates here write `"{{today}}"` in quotes. Every template of the first library entry writes it unquoted, and [13 — Open obligations](../../spec/13-open-obligations.md) records that a YAML loader reads the unquoted form as a flow mapping and refuses the whole block. The quoted form parses. It also makes the substituted value a quoted scalar, which [Q2](../../spec/09-decisions.md#q2--schema-format) never typed. This entry takes the reading that parses and records the assumption, and it does not edit the other entry's templates, because a finding about somebody else's package is a separate change.

## Findings

These go to [13 — Open obligations](../../spec/13-open-obligations.md), on the route that the [library index](../README.md#where-a-finding-goes) fixes. Nothing here reopens a closed decision. Three of the seven are new items in 13, two sharpen an item that is already there, one belongs in the glossary, and the last is a confirmation rather than a finding.

**1. Composition between two library entries has no add-only form.** This is the finding that a second entry exists to produce, and the entry met it three times in one bundle.

- **A shared purpose.** `obligation_record` serves the `obligation` purpose, and the design-spec entry declares it. Criterion 6 refuses a second declaration at that address, and the resolver refuses it too, because both operations write the same leaves. So the remedies are a rename, which puts two names on one reader intent, or a dependency. This entry declares `requires: [design-spec]` for one declaration, and it inherits six kinds and three shelves of a tradition that an ADR-keeping team need not use.
- **An endpoint list of concrete kinds.** The design-spec `cites_evidence` runs `from: [design_spec, decision_register, obligation_register]`. `obligation_record` is not among them, and no `add` reaches into a list that exists, so a kind of this entry cannot cite evidence at all. An abstract kind at the source end would have kept the endpoint open, and [spec 2](../../spec/02-taxonomy-model.md#abstract-kinds) says that abstract kinds are what make a bundle possible.
- **A facet requirement list.** The same rule stops `title` from reaching the design-spec kinds, which is where #123 measured its absence.

The three share a root. Vocabulary that more than one tradition needs cannot live in an entry, because criterion 6 makes the first entry to claim an address its owner. The base is the only place where shared vocabulary composes. Either a purpose and a name facet belong there, or the resolver admits two identical declarations at one address and the confluence proof takes an exception.

**2. The two checks of this tradition are about content and state over time, and no declaration expresses either.** Immutability after acceptance reads a content change against a lifecycle state. A proposal that never reaches a ruling reads dwell time in a state. Every declaration in the language that carries a rule about time is a participation expectation, and an expectation reports one thing: a missing edge inside a window. Both failures here have every edge they need. The obligations `OB-DR-1` and `OB-DR-2` state the invariants with gap dispositions, and each names an engine input that already exists, which is what makes them gaps rather than requests.

**3. A rejected decision and a deprecated one are one state.** This sharpens the item that 13 already carries about list extension in an add-only overlay. The design-spec entry hit the limit twice and could name no case where the collapse loses a fact. This one can. `rejected` and `deprecated` are both terminal-retained, so the collapse is silent, and a reader cannot tell a refused proposal from an abandoned decision. The tradition also cannot write its own two rung names, which costs criterion 1 the recognition it asks for.

**4. A generated index labels a document with its identifier, and the fix does not reach the shelf that showed the defect.** This sharpens the item that #123 opened. `facets.title` answers it for this entry's kinds and for the base `decision`. It cannot answer it for the specification shelf, for the reason finding 1 states. The general remedy is a `name` role in the closed facet-role registry of [spec 2](../../spec/02-taxonomy-model.md#the-meta-schema), which is a meta-schema change and therefore a taxonomy major version.

**5. `obligation` names two things.** Spec 4 declares obligations as a taxonomy block: invariants that a corpus commits to. Spec 13 holds obligations as owed work. The two are unrelated and the specification calls both an obligation. This entry writes `obligation_record` and `OBL-` to keep them apart in one corpus, and the [glossary](../../spec/glossary.md) is where the general fix belongs.

**6. A declared `invalid_when` reaches no check, so two live contradictory decisions pass in silence.** The base declares `conflicts_with` with `invalid_when: {both: {status: current}}`. [Spec 2](../../spec/02-taxonomy-model.md#the-decision-relation-vocabulary) lists that state first among the four checks that the decision-relation vocabulary brings, and [spec 11](../../spec/11-adjacent-work.md) calls it a deterministic, blocking-eligible check that the design held all along. The engine reads the declaration in one place, where it makes the `status` facet count as read for the relevance canon, and no rule reports the state. The [fixtures](fixtures/README.md#what-a-run-does-not-report-and-should) plant two current decisions that contradict each other, and the run is clean. This is a check that two specifications promise and no code performs, which is a different class of gap from a form that nothing states.

**7. This entry adds no `override` and no `remove`, and it is admissible.** That is not a finding, and it is recorded here because criterion 3 asks the question. Every operation in `bundle.yml` is an `add` at a key that the base and the design-spec entry both leave absent. `headwater taxonomy resolve` and `headwater taxonomy validate` confirm it over the merged result, with this bundle selected beside the first.

## What a migration inherits

This entry is the taxonomy. It is not the conversion of this repository, and the two are separate work for a reason that a count makes plain. Twenty-one decisions and every item of the four lists in 13 become documents, and the largest of those lists holds sixty. Each needs a title, a state, two dates, a summary, a warrant, an identifier, and its edges. The first typing of this corpus took one agent session for 36 documents ([13 — Open obligations](../../spec/13-open-obligations.md#what-the-first-typing-of-this-corpus-found)), and this is several times that at a finer grain.

Four things are ready for it, and three are not.

**Ready.** The kinds, the shelves, the identifier schemes, and the sections. `docs/decisions/**` and `docs/obligations/**` are the base shelf and this entry's shelf, and neither directory exists yet. The lifecycle mapping above says which state each entry is in, and every closed decision is `current`.

**Not ready: the two register documents themselves.** [9 — The decision register](../../spec/09-decisions.md) and 13 stay as documents after the conversion, and they become indexes over the entries rather than the entries. That is a projection, and this entry may not declare one.

**Not ready: the citations.** Every `Qn` in this corpus is a link to a section anchor. Each becomes a link to a document, and `link.fragment.unresolved` is a live rule that reports the ones that fall. Breaking one identifier in this repository produced its own finding plus three dangling references elsewhere, and a migration breaks twenty-one at once.

**Not ready: `09-open-questions.md`.** It is the tombstone of the register that preceded 13, and it is a superseded document that redirects. Under the conversion it becomes generated, which is [#68](https://github.com/headwater-ai/headwater/issues/68)'s first blocked artifact, and generating it needs the projection above.

The conversion is filed as its own issue, and this entry is what unblocks it.
