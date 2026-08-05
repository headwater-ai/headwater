# Core-concepts review: findings, second run

Instrument: [core-concepts-review-prompt.md](core-concepts-review-prompt.md), run
2026-08-06 against revision `58dc640` — the specification after the integration of
[run 1](core-concepts-review-findings.md), the
[independent review](core-concepts-independent-review.md), and the
[assessment](core-concepts-findings-assessment.md). Single sequential reviewer
(Claude Fable 5); no fan-out.

**Conflict declared:** this reviewer performed run 1 *and* authored the integration
now under review. The pass was therefore conducted adversarially against the new
text in particular — the changes I made are the ones examined hardest, and the most
consequential findings below are defects in the integration, not residue it missed.
Claims marked *(inferred)* rest on absence of text or predicted behaviour; the rest
is read directly.

## Verdict table

| Concept | Verdict | Reason |
|---|---|---|
| corpus | keep | Unchanged from run 1; still the coverage denominator and federation unit. |
| document | keep | Unchanged. |
| shelf | keep | Unchanged; discriminator-drift audit now watches the heterogeneous risk. |
| kind | keep | Unchanged; arbitrage named and audited. |
| external anchor | **define edges** | Identity is now defined, but anchor *kinds* (`code_path`) appear in relation endpoints and have no declaration home among the ten declarations (argument 2). |
| relation | keep | Limits are now declared; one rule rests on a false premise (argument 7). |
| relation family | keep | Defaults plus audited overrides — the narrowed run-1 position, correctly applied. |
| nuclearity | keep | Family-defaulted, nucleus named where direction demands it. |
| participation expectation | **define edges** | The merge is right; the origin's integrity, re-entry semantics, and missing-origin behaviour are not defined (argument 1). |
| `state_entered` role | **define edges** | New in this revision, and it shipped without a maintenance contract — the single weakest datum in the design (argument 1). |
| facet | keep | Scalar-only value space closes run 1's ungoverned-edge hole. |
| purpose | keep | Unchanged. |
| voice regime | keep | Unchanged. |
| lifecycle regime | keep | Transition checking now has its input; the input itself is under-anchored (argument 4). |
| obligation | keep | Unchanged. |
| control | keep | Unchanged. |
| register | **merge into projection** | It *is* one now — spec 1 still spends a glossary entry on what spec 4 defines as a mandatory generated projection. The last free concept cut (closing 1). |
| projection | keep | Unchanged. |
| identifier scheme | keep | Unchanged. |
| core | keep | Unchanged. |
| overlay | keep | Unchanged; now also carries profiles, correctly. |
| mapping | keep, defer | Unchanged; aggregator topology now normative. |
| contract sidecar | keep, defer | Unchanged; still severable, still unreferenced by anything else. |
| `created_by` | keep | Now backed by the scaffold-first default stance. |
| migration state | **define edges** | New in this revision; per-document blanket attribution masks unrelated regressions, and its own staleness rule has no parameters (argument 5). |

Honest accounting of the concept count: the integration removed six author-facing
concepts (authority, dominance, sequences, profiles, compatibility, the freshness
and size wrappers) and added two system-facing ones (`state_entered`, the
migration state) plus one check input (`needs_prior`). Net movement is real and in
the right direction — the learnability gradient for extending a taxonomy is now
roughly eight concepts, down from ~12 — but the additions are exactly where this
run's defects concentrate. A cut concept cannot break; a new one can.

## Delta from run 1

Every run-1 verdict was applied or consciously narrowed; re-checking each against
the current text found no regression *except* where the fixes introduced new
undefined edges, reported below. Resolved and verified as resolved: authority cut
(Q18 opened), dominance derived, sequences merged with origins, reference facets
removed, relation limits declared, declarations at ten (the table now counts
honestly), register generated, suppression expiry mandatory, migration state
exists, default relations minimal and scaffold-created, machinery elevated to
first-release. The comparison the review directory exists for: **run 1's findings
are closed; run 2's findings are new, smaller, and mostly about the closures.**

## Arguments, by consequence

### 1. The window-origin fix relocated the problem into data integrity

Run 1's top finding was that `within: 90d` had no computable start. The fix —
`state_entered`, stamped on transition — makes the window computable and leaves
**who stamps it, and what verifies the stamp,** undefined. Nothing in specs 2, 3,
or 12 declares the check that ties `status_since` to the transition it claims to
record:

- An author (or agent) who edits `status` without touching `status_since` extends
  every window silently. The change-scoped machinery could catch this — `needs_prior`
  sees both versions, and "status changed ⇒ status_since updated" is a mechanical
  rule — but no spec declares that check, and it is not generable from any current
  declaration. *(Inferred from absence; searched specs 2, 3, 12.)*
- What value is correct is also undefined: "stamped by the transition" implies the
  transition date, but `check --fix` run a month late would stamp fix-day, quietly
  extending the window; and a backdated or future-dated `status_since` is validated
  only for being a date. `last_verified` gets a plausibility rule (no future
  dates); the origin facet gets none.
- Spec 2's own `supersedes` declares `on_target: {set_state: superseded}` — an
  edge that mutates the target's state. Whether that mutation stamps the target's
  `status_since` is stated nowhere, and if it does not, every window measured on a
  superseded document is measured from the wrong state's entry.
- A document that re-enters a state (legal in any taxonomy whose machine has a
  cycle) resets `status_since` and thereby restarts every window conditioned on
  that state. Reset-on-re-entry may even be the right semantics — but it is
  currently an accident, not a decision, and it is also a gaming route: flip out
  and back to re-arm a 90-day grace period. *(Inferred.)*
- An adopted corpus has no `status_since` anywhere. The meta-schema requires the
  origin role on declaring kinds, so first contact converts every existing
  document into a findings-flood contributor — and what an expectation does when
  the origin facet is *absent on an instance* (fire? skip-with-reason? treat as
  epoch?) is undefined. Only skip-with-reason is consistent with spec 4's coverage
  honesty, and no spec says so.

Confidence: high on every absence; this is the direct continuation of run 1's
argument 3, one level down. The mechanism is no longer uncomputable — it is now
computable from a datum nothing defends.

### 2. The default taxonomy example fails its own meta-schema, three ways

Spec 2's worked YAML, checked against spec 2's own validation list:

- **The origin facet is not required where the validator demands it.** The
  `decision-realised` expectation declares `since: state_entered`, and the new
  well-formedness rule requires "an origin role that is required on the declaring
  kind" — but `kinds.decision.facets.require` lists
  `[status, last_verified, domain, summary]`. No `status_since`. The example
  fails the rule written three sections below it. (Confidence: high; textual.)
- **The expected relation does not exist.** The expectation names
  `implemented_by`; the `relations:` block declares `supersedes`, `derives_from`,
  `governs`, `conflicts_with`. "Every participation expectation names an existing
  relation" rejects it. This defect predates the integration — the old `sequences`
  block named the same phantom relation — but the integration wrote the validator
  rule that now catches it and did not fix the example. (High; textual.)
- **Anchor kinds have no declaration home.** `governs` declares `to: [code_path]`,
  glossed "an external anchor kind" — and no declaration among the ten declares
  what anchor kinds exist, which resolver owns each, or how their identity is
  normalised. Spec 1 now says "each anchor type is owned by exactly one resolver",
  but *owned where?* Referential integrity ("no dangling relation endpoints")
  either rejects `code_path` or contains an unstated anchor-kind exemption. This
  is not an example bug; it is a missing corner of the model that run 1 and the
  independent review both walked past. The fix is definitional, not additive:
  state where anchor kinds are declared (plausibly inside `facets`-style
  declarations or as a clause of `relations`) and what referential integrity
  means for them. (High; textual absence.)

An example that fails its own validator is exactly the drift between declared and
actual structure the system exists to kill, in the document that defines the
system.

### 3. Promotion still lacks its data source — where it matters most

The integration defined false-positive labelling as riding the suppression reason
(`false_positive` vs `accepted_deviation`), which closes run 1's "who labels, and
where" for blocking checks. It does not close it for **advisory** checks — and
promotion decisions are made *about advisory checks by definition*.

An advisory finding does not block anything, so nothing forces an author to
suppress it; the rational response to advisory noise is to ignore it, which spec
4's own first-contact analysis predicts. Ignored findings carry no label, so the
false-positive rate of exactly the population being considered for promotion is
computed over the sparse, biased subset of findings that annoyed someone enough to
suppress. A check can sit at a catastrophic real FP rate and a clean measured one.
The demotion direction works (blocking checks force engagement); the promotion
direction measures selection bias. (Confidence: medium-high; the mechanism is
textual, the behavioural claim *inferred* — but it is the same
capture-cost-shaped inference the spec itself relies on elsewhere.)

This needs defining, not inventing: state what the promotion criteria's
"false-positive rate under a declared threshold" is computed over, and either
accept the bias explicitly or gate promotion on a sampled adjudication of
unsuppressed findings during the observation window.

### 4. `needs_prior` is defined against an undefined baseline

Spec 12 says the prior version "arrives from the diff" and joins the cache key.
Which diff? A pre-commit hook's prior is the working tree's HEAD; a CI run's prior
is presumably the merge-base — but a rebase, a squash, or a force-push changes the
merge-base, so the same tree state can pass in one push and fail in another, and a
multi-commit branch can hide an illegal transition inside itself (draft →
superseded via an intermediate commit that CI never diffs against). Hook and CI
can legitimately disagree, and the spec that made determinism a headline
constraint does not say which answer is the deterministic one. Define: prior =
the last version on the target branch (merge-base), transitions inside a branch
are evaluated pairwise across its commits, or transitions are hook-enforced only —
any of the three, stated. (Confidence: high that it is unstated; the divergence
scenarios are *inferred* but mechanical.)

### 5. `migration-pending` is a blanket over the wrong shape

Spec 7 labels findings by *document* — "a finding attributable to a document named
in an open task". Migrations name documents; findings attach to documents; so for
the duration of a migration, **every** finding on a named document is soft-labelled,
including defects introduced yesterday that have nothing to do with the migration.
A three-month major upgrade turns its task list into a set of documents where new
breakage reads as expected breakage. The right grain is the (document, rule) pair
the migration payload already knows — it declared what moved and what must be
re-stated, so it can declare which rules it expects to fail. Also undefined: "a
migration state with no recent activity is itself a staleness finding" names no
clock, no threshold, and no owner — the same "a window is a mood" defect the
integration fixed for expectations, reproduced in the mechanism it added.
(Confidence: high; textual.)

### 6. The engine-significant role inventory exists in three places and agrees in none

- Spec 1 lists **three** structurally special roles: state, state-entry,
  freshness.
- Spec 2's YAML declares **four**: `state`, `state_entered`, `freshness`, `scent`
  — and routing, indexes, and the scent measures all reason about `scent`, so
  spec 1's list is short by one.
- Spec 2's participation-expectations prose names a **fifth**, `created`, "for
  expectations with no state condition" — and no facet with that role exists in
  the default YAML, in spec 1's list, in the core, or anywhere else. It is a
  phantom: an expectation with no state condition (run 1's
  `incident-learned-from`, deleted from the example rather than migrated) is
  currently undeclarable in the default taxonomy.
- The meta-schema enforces "role uniqueness — at most one facet claims each
  engine-significant role" against a set that is enumerated nowhere.

One registry, one place, and the other lists derived from or deleted in favour of
it. (Confidence: high; entirely textual.)

### 7. The governance-cycle rule rests on a false premise

Spec 2's limits section: "evidence and governance edges are not ordered, so the
question does not arise." Governance edges are *directed* — `constrains` narrows
what the *target* may decide — so `A constrains B constrains A` is expressible,
and the sentence dismissing it is untrue as stated. Mutual constraint may well be
legitimate (two decisions bounding each other is a real design situation), in
which case the rule should say governance cycles are *legal and meaningless to
order*, not that the question cannot arise. As written, an implementer reading
carefully will notice the premise is false and make their own decision — the
precise failure mode the section exists to prevent, inside the section.
(Confidence: high; textual. This is a defect in text I wrote.)

### 8. "Enabled" versus "defined" is now load-bearing and undefined

The default enables four decision relations; spec 1 still says nine relation
types are "all defined in the default taxonomy" including `implements`, `refines`,
`cites`; the example kind's `may:` includes `refines`; the overlay example
*removes* `relations.refines` — from a default where it is unclear whether
`refines` is present, dormant, or absent. Nothing defines what a defined-but-not-
enabled relation *is*: invisible? declarable but warned? valid for overlay
`add_to` without redeclaration? The four-relation default only means something
once "enable" has semantics, and the three documents currently imply three
different ones. (Confidence: high; textual.)

### 9. Smaller defined-edges residue

- **Exception composition.** A finding can now be simultaneously suppressed,
  `migration-pending`, and under a waived rule. Coverage accounting needs a
  stated precedence (suggested: waiver > migration-pending > suppression, each
  counted once); currently each mechanism describes its own inventory as if the
  others did not exist. (Medium; textual absence.)
- **Nuclearity override audit vs. meta-schema.** "A family's default is not
  contradicted without explicit override" (validate) and "audit reports every
  override" are consistent, but no threshold or consequence attaches to "a family
  whose members mostly override it is misassigned" — an audit observation with no
  addressee. Minor. (Medium.)
- **Worked-example drift.** The small-team column shows `supersedes` only while
  the default now enables four; either the small team removed three relations
  (the friction the retune abolished) or the column predates the retune. One
  sentence fixes it. (High; textual.)

## Forced cut, revisited

Run 1's forced-cut list is spent — items 1–8 are done. If a further 30% had to go
today, in order: (1) the register glossary entry (closing 1, free); (2) contract
sidecars (still severable, still unreferenced); (3) mappings (still
aggregator-blocked); (4) the `Neighbourhood(depth)` scope (spec 12 already
suspects it); (5) `group_by`/`layout` shelf mini-language (run 1's watch-item,
still unexercised by any check or projection named in the specs). Items 2–5 all
lose a real capability; the list is now short of free cuts, which is what a
healthy concept inventory looks like.

## Closing

**1. The single change that most reduces concept count without losing a
capability:** retire **the register** as a named concept. Spec 4 already defines
it as a mandatory, generated, checked view; spec 1 still teaches it as a third
thing beside obligations and controls. Make it the engine-defined projection it
is — the obligations-and-controls section then teaches exactly two concepts, and
nothing changes at runtime because nothing about it was ever authored.
(Confidence: high.)

**2. The single thing most likely to break first in a real adoption:** the
absence-detection layer dying **silently** through `state_entered` data quality.
Run 1's leading risk — edges failing to materialise — is now hedged by design
(scaffold-created defaults, audited assisted fraction, machinery in the first
release). The new flagship datum is not: nothing verifies the stamp, `--fix`
can falsify it innocently, migration leaves it absent corpus-wide, and every
failure mode points the same direction — windows that never start. A corpus
whose expectations never fire is indistinguishable from a corpus with nothing
missing, which is precisely the silent pass the census was built to prevent,
recreated one datum deep. It will not break loudly in month one; it will have
quietly never worked. *(Inferred, from the design's own no-silent-passes
doctrine applied to its newest input.)*
