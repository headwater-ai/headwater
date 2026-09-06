<!-- SPDX-License-Identifier: Apache-2.0 -->

# The starter kit, and why it selects these three bundles

You are reading the prose that ships with `headwater/standard` and with every package flattened from it. It explains one recipe: `starter`, which selects `design-spec`, `evidence-and-obligation` and `decision-record` over the base taxonomy.

It is written for the adopter who answers nothing. If you know which bundles you want, take `headwater/standard`, name them in `.headwater/taxonomy.yml`, and stop reading here.

## Why a starter kit exists at all

The base package is minimal on purpose. It is derived from the invariant core rather than chosen, and it is small so that every bundle over it can be add-only. An add-only overlay carries no `remove`, so any subset of bundles resolves and the publisher can prove that once per release rather than per adopter.

That is a good property and it is a bad first experience. An adopter who resolves the base alone gets a taxonomy that admits a governed document, a state, an identifier and a relation, and that says nothing about what their organization writes. The blank schema is the first-run problem, and the starter kit is the answer to it.

So there are two artifacts and they answer opposite requirements. The base has to be minimal. The starter has to be opinionated. Nobody is expected to run the base bare.

## What each bundle gives you

**`design-spec` — a numbered series that states one design.** A specification split into ordered parts, where part 7 is an address a reader can cite and reach. It carries the register that collects the decisions, the review record, and the supersession relations that let part 9 retire part 4 without deleting it. Take it if your organization writes anything that reads in order: an architecture document, a protocol, a standard, a set of numbered internal specifications.

**`evidence-and-obligation` — what a document owes, and what discharges it.** An obligation record states something the corpus owes that nobody has yet paid: a ruling that is waiting, a measurement nobody has taken, an adopter who has not arrived. It carries the register that lists them and the relations that connect a claim to the evidence for it. Take it if you want the gap between what a document asserts and what anybody checked to be a thing you can count.

**`decision-record` — one decision per document, with the argument beside it.** The architecture-decision-record tradition: a context, a ruling, and the consequences, on a shelf where each record supersedes or constrains its neighbors. Take it if your organization already writes ADRs, or wants to.

## Why these three and not another three

**They are the selection this package's own publisher runs.** The repository that maintains `headwater/standard` resolves exactly `design-spec`, `evidence-and-obligation` and `decision-record` over the base, and it checks its own corpus against the result on every commit. The starter is not a designed set. It is the one set that has been run in anger, offered to you as a starting guess.

Say that plainly, because the alternative is worse. The obligation record `HW-OBL-0018` in that corpus states that the bundle set is a guess about how adopters cluster, and that no adopter has revised it. This page is the same admission, addressed to the first adopter who might.

**They do not decompose.** `design-spec` and `decision-record` both name purposes, kinds and a voice regime that `evidence-and-obligation` declares. A selection of the first two without the third leaves seven dangling names, and referential integrity refuses it before a lock is written. So the three arrive together whether you take the recipe or copy the list.

**Three bundles this recipe leaves out, and when to add one.** `standards-spec` models a normative standard with conformance levels and requirement identifiers, which most repositories do not write. `brd-prd` models the business-requirement and product-requirement pair. `diataxis` models the four-way split of tutorial, how-to, reference and explanation, which is a documentation-site shape rather than a governance shape. Each of them is a real tradition and none of them is a guess most adopters would want made for them.

## The two ways to take this

**Take the flattened package.** Pin `headwater/starter` in `.headwater/taxonomy.yml`, name no bundles, and you get one complete taxonomy with the three selections already resolved into it. Your upgrades then arrive as new releases of `headwater/starter`. A selection against it is refused rather than ignored, because a flattened package ships no bundle directory.

**Take the recipe inputs.** Pin `headwater/standard`, and write `bundles: [design-spec, evidence-and-obligation, decision-record]` yourself. You now own the selection: drop one, add `standards-spec`, and advance the source package when you choose rather than when the starter's publisher does.

The two forms differ in who owns the upgrade and in nothing else the engine reads. Neither one forks you from the base, because a bundle is an overlay and customization is always by overlay.

## What this kit does not decide for you

**Your identifier namespace.** No package can hold it. A package that named one would give the same namespace to every corpus that adopts it, and a stand-in such as `repo` names nobody. It is the one value the first-run interview asks for and no bundle supplies.

**Your writing profile.** The base declares `en-US` with `controlled: none`, which turns the controlled-language rules off. This recipe changes that value nowhere, so a corpus that takes the kit gets the same setting. The repository that publishes the kit runs a house profile over its own prose, and it does that in its own overlay. If you want a controlled language, declare one the same way. Spec 2 of that repository currently promises that the starter turns a house profile on for you, and nothing in this package keeps that promise; which artifact should carry it is an open question there.

**What your documents are actually about.** Every bundle here models a tradition of writing, and none of them models your subject. The parts that carry your organization's own vocabulary belong in your overlay, and a day-thirty measurement of what you actually wrote is worth more than a day-one guess.
