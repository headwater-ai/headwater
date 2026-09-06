---
id: HW-DR-0056
status: current
status_since: 2026-09-06
summary: "No kind in the base package or in any bundle binds a language regime. A package that set the default value would change nothing, so the doctrine page of the starter kit carries the promise."
last_verified: 2026-09-06
title: "A language regime reaches prose through a kind, so the starter kit's doctrine carries the writing profile"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# A language regime reaches prose through a kind, so the starter kit's doctrine carries the writing profile

## Context

[Spec 2](../spec/02-taxonomy-model.md#language-is-declared-not-assumed) states that the doctrine starter kit declares `en-US` with the house profile as its default. [#578](https://github.com/headwater-ai/headwater/issues/578) asks which artifact keeps that promise. It offers three answers: `headwater init` writes the value, the doctrine page of the starter kit carries it, or spec 2 retires the promise.

The issue states three facts, and all three re-measure true. An assembly overlay cannot write the value. `assembly::validate_glue` refuses an operation that restates a path the base package owns, and it reads the path rather than the value. `headwater init` reads one version scalar out of `package.yml` and opens no taxonomy. On the first-run path there is no package on the tree to read.

**A fourth fact settles the question, and the issue does not state it.** `Shape::language_of` resolves a language regime through `kinds.<kind>.language` and through nothing else. The name `default` carries no meaning to this engine, and no fallback to a regime of that name exists. Neither the base package nor any of the six bundles declares `language` on a kind, so the count of such bindings is zero.

**Measured end to end over this corpus on 2026-09-06.** An override of `regimes.language.default.controlled` to `ste-house` was appended to the adopter overlay of this repository, where an override is legal. `headwater taxonomy resolve` and `headwater check` each exit 0 afterward. The two reports differ on two lines of 73786 bytes, and both of those lines carry the lock digest. Each run reports 395 seen, 283 classified, 5194 check instances and 7 findings.

**So the remedy the first answer names writes a value that nothing reads.** An adopter who opens the file `headwater init` wrote would read `controlled: ste-house` and conclude that a controlled language holds over their corpus. It would not hold, and no finding would tell them. That is the defect [#276](https://github.com/headwater-ai/headwater/issues/276) already recorded against this verb, which wrote a route an adopter cannot take into the file the adopter commits.

The form that would turn the profile on is a full regime declaration plus one `language` line for each kind. This repository writes exactly that in its own overlay: a regime named `ste_house`, eighteen retired terms each with a reason, and ten kind bindings. Six of the ten sit on kinds the package declares, and four sit inside kinds the overlay itself adds. The retired-term list is editorial judgment about one corpus. [Spec 7](../spec/07-distribution-and-federation.md#the-interview) rules that the interview is package data and not engine code. An engine that authored that list would breach the rule that defines the verb.

## Decision

**The doctrine page of the starter kit carries the promise, and no package carries it.** A language regime reaches prose only through the kinds that bind it. The kinds a corpus wants under a controlled profile are a fact about that corpus. The declaration therefore belongs to the adopter, and it lands in the adopter overlay. The doctrine page is where the adopter reads how to write it, and `taxonomy-source/headwater-standard/doctrine/starter.md` says so under *Your writing profile*.

**`headwater init` writes no language declaration.** That holds on a tree carrying the package under `packages/` and on a tree carrying none. The verb reads one version scalar and asks its questions in the file it writes, and this ruling adds nothing to either half.

**Spec 2 names the doctrine as the carrier.** Its paragraph on the two packages says where an adopter turns the profile on. The base package keeps `controlled: none` for the reason [Q3](../spec/09-decisions.md#q3--how-much-of-the-default-taxonomy-ships-in-the-box) gives.

**This record rules nothing about the sentence of spec 2 that names a corpus default.** That sentence describes a fallback this engine does not implement. The repair is a change to the engine, or a change to the model the specification states. [HW-OBL-0166](../obligations/0166-spec-2-states-a-corpus-wide-default-language-regime-that-no-rule-of-this-engine-reads.md) holds that question for the owner.

## Consequences

**An adopter who takes the starter kit gets `controlled: none`, and the doctrine page tells them so.** Their first run meets the finding count their own prose earns, and the wall of findings that Q3 exists to prevent stays prevented.

**The closing sentence of the shipped doctrine page moves with this record.** It said that which artifact carries the promise is an open question, and this record answers the question. The page states the ruling and names the two lines an adopter writes.

**Two live sentences of an evaluation move with it.** [The first-run walkthrough](../evaluations/default-taxonomy-first-run.md) states that the declaration belongs to the starter kit. It also states that the kit already declares both the spelling ruling and the house profile. The second claim is measurably false against the published artifact, which carries `controlled: none` at `taxonomy.yml`.

**The doctrine page sits inside the package, so the change costs a publish cycle.** Publish, then pin the printed digest, then vendor, then resolve. That order matters, because a resolve before a publish moves the lock with the source and a comparison meant to fail passes.

**The third answer of the issue stays available and this record does not take it.** Retiring the promise altogether leaves an adopter with a question and no address. Naming a carrier that already exists and already says the right thing costs one publish cycle and about six sentences.

**One test now reads what `headwater init` writes.** `engine/crates/cli/tests/init.rs` holds both written files byte for byte. It is a characterization of the verb rather than a statement of what the verb should write. It carries no language declaration of its own, which is what makes it the record of this ruling in the suite.
