---
id: HW-DR-0073
status: current
status_since: 2026-09-18
summary: "A verification takes a minted document identifier, because a rename must not break the link and one criterion may be proved in several repositories. The anchor option loses that identity, and it would also be blind to the participation expectation until #855 lands."
last_verified: 2026-09-18
title: "A verification is a kind, and its identity is minted rather than found in the code that cites it"
relations:
  constrains:
    - HW-DR-0029
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
---

# A verification is a kind, and its identity is minted rather than found in the code that cites it

## Context

A requirement states what a system must do. An acceptance criterion states what would settle it. The artifact that settles most criteria is a test, and a test has no identity in this corpus.

[#514](https://github.com/headwater-ai/headwater/issues/514) asked four questions about that gap and recorded a leaning on each. This record answers all four. The question came from a practice the owner has run elsewhere. In that practice a requirement identifier, a criterion identifier and a verification identifier carry one coverage matrix.

**Two of the three objects already exist.** [#398](https://github.com/headwater-ai/headwater/issues/398) declared `kinds.requirement` and `kinds.acceptance_criterion`. The overlay declares `facets.verification_method` with the 29148 set of `test`, `demonstration`, `inspection` and `analysis`. A `current` requirement with no `verified_by` edge is reported after 90 days under the `requirement-verified` expectation.

**The third object has a path and no name.** [#411](https://github.com/headwater-ai/headwater/issues/411) dispositioned a test as `traces_to` onto a `code_path`. `SourceTree::resolve` binds such a target by `Path::exists` and carries `revision: None`. So the link survives no rename. A criterion proved in three repositories cannot say so, and no identifier exists for a person to write into a test.

**The live instance is on the tree.** `HW-AC-0002` declares `verification_method: test` and reaches its verifier by `traces_to: engine/crates/check/src/sections.rs`. That is a path and never an identity.

**#411 shipped a verifier that is not a document, and it chose an anchor.** The overlay declares `anchors.check_rule: {resolver: check-rule}`. `relations.verified_by` now reaches `[acceptance_criterion, probe, check_rule]`. The reason recorded there is that a rule identifier is a string this engine ships, so the identity is the string itself. That reasoning is sound for a rule. It does not carry to a test, because no engine ships the tests of an adopter.

**One measurement settles the disposition question that #514 left open.** A scratch corpus at `788885a9` took two added obligations. One carried no control. The other carried a single control with `mechanism: ci:nightly-suite`. The register moved from `33 verified` to `34 verified`, and the obligation with the external mechanism appeared nowhere in the report by name. So the register already grades a declaration as verified with no observation behind it. [#934](https://github.com/headwater-ai/headwater/issues/934) owns that finding.

## Decision

**1. A verification is a kind.** It is a governed document on its own shelf, with an identifier minted by `headwater new` under a scheme of the `{namespace}-VER-{seq:04d}` shape.

The ground is identity rather than the value of the prose. #514 asks for three properties. A rename must not break the link. One criterion must be provable by tests in several repositories. A person must have something to write into a test. An anchor bound by a scan of the code satisfies the third property alone. Such an anchor has no identity apart from the place its bytes were found. So it survives no rename, and it cannot name one verification across two repositories. A minted identifier is held by a claim under [HW-DR-0054](0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md), and it resolves in any repository under the namespace-first rule of Q25. The prose that states the approach of a test, and what the test does not cover, is the second reason and not the first.

**This ruling does not rest on [#855](https://github.com/headwater-ai/headwater/issues/855), and a reader should not read it that way.** `Adjacency::of` skips every edge whose target is not a document. So no anchor is a neighbor, and no participation expectation counts one. Under the anchor option a criterion with a verification would therefore read as a criterion with none, until #855 lands. That is a second argument against the anchor option, and never the first. A defect that somebody will fix cannot carry a ruling. Issue #855 completes #411, and it belongs beside that change.

**2. A verification mirrors the assurance register and does not unify with it.** Two properties drive the wedge. A control is a declaration in the `controls:` block of a taxonomy, and a verification under ruling 1 is a document on a shelf. The register also counts facts about the corpus, and a criterion states a fact about the product, so one headline number would mix two populations.

The measurement above is the stronger reason. The register grades `verified` from a declaration and never from an observation, which is the loss #514 names for a test. A unified register today would import that loss into the object this record exists to protect. **This ruling flips if [#934](https://github.com/headwater-ai/headwater/issues/934) gives the register an observation dimension.** On that day a control and a verification grade evidence the same way, and one register serves both. A later reader tests that condition before a mirror is proposed again.

**3. The corpus records observation and freshness, and never an outcome.** A verification carries three states, which are `declared`, `observed at commit X` and `suspect`. Pass and fail stay in the build, which is the system of record for them and which already blocks on a failure.

A snapshot names each verification identifier and the commit it ran against, with no outcome column. The engine reads it offline against a pin, which holds the rule of spec 0 that the engine reaches no network at check time. `headwater_import::anchors::over` is the transport and it already ships. A verification is `suspect` when the criterion it proves changed after the snapshot commit, which is the rule DOORS states as P.6. The shape of a declaration on one shelf and an observation on another follows `kinds.probe` and `kinds.probe_result`.

**The build that writes the snapshot is a system of record, and never an integration point of the governed loop.** [HW-DR-0072](0072-the-binary-is-the-only-interface-an-adopter-must-run-and-every-integration-point-outside-it-is-declared.md) counts what an adopter must run to read, check and write the corpus. A build runs its tests for its own reasons, and the snapshot is an output of that run. That is the shape of the import snapshot of Q19, and HW-DR-0072 does not list that snapshot as an integration point either.

**Bullet 2 of the Done-when of #514 is answered by citation.** Spec 4 grades such an obligation as `verified`. Line 172 carries the definition. Line 178 carries the exemption, and it reaches only an unimplemented rule or phase. That answer is the defect [#934](https://github.com/headwater-ai/headwater/issues/934) reports, rather than a state this record invents.

**4. A `comment-scan` resolver ships in the binary.** The default reads a pattern declared in the overlay, so the convention stays in configuration and out of the engine.

**A language that the pattern cannot read is a gap against the binary, and never a script that an adopter runs.** [HW-DR-0072](0072-the-binary-is-the-only-interface-an-adopter-must-run-and-every-integration-point-outside-it-is-declared.md) rules that an adopter reaches the whole governed loop through the binary. Its list of integration points outside the binary is closed against growth. A binding out of code enters the graph that `headwater check` reads, so the binding is part of that loop. A scanner that reads the grammar of a language is therefore a resolver to file against the binary, under the necessity test of HW-DR-0072.

**This narrows the fourth reason of [HW-DR-0029](0029-q29-whether-a-corpus-root-may-contain-code-and-what-an-interface-contract-may-reach.md) and reopens nothing.** That reason states that an anchor is a name and never a subject, because `SourceTree::resolve` reads no byte of the file it binds. A `comment-scan` resolver reads bytes at the moment it binds, so the reason stops holding for that one resolver. The second reopening condition of Q29 names the first check-layer **rule** whose subject is the bytes of an anchor target. A resolver is not a rule, so that condition does not fire and the root question stays where Q29 left it.

**[#500](https://github.com/headwater-ai/headwater/issues/500) is not displaced.** It checks the citation of spec 5 in generated code and asks three questions of it. Ruling 4 asks one question of a different comment shape. The two share a scanner and nothing more, and #500 stays in its own milestone.

**The open question of [HW-OBL-0128](../obligations/0128-nothing-holds-a-crate-to-having-a-contract-under-a-root-that-excludes-it.md) is not on this path.** That record asks whether a projection may read outside the corpus root. A resolver that reads outside the root is the shipped shape today, because `source-tree` binds a path under `engine/`.

## Consequences

**Three build issues follow from this record, and [#514](https://github.com/headwater-ai/headwater/issues/514) closes when they exist.** [#935](https://github.com/headwater-ai/headwater/issues/935) declares the `verification` kind, its shelf and its identifier scheme. It also declares a relation from an acceptance criterion, and the expectation that lists every criterion no verification reaches. [#936](https://github.com/headwater-ai/headwater/issues/936) declares the `test_site` anchor kind and the `comment-scan` resolver, and it amends the fourth reason of HW-DR-0029 in the same change. [#937](https://github.com/headwater-ai/headwater/issues/937) adds the observation snapshot. It depends on [#934](https://github.com/headwater-ai/headwater/issues/934), because the two are one transport.

**No expectation over an acceptance criterion exists today.** The only participation expectation in this corpus is `requirement-verified`, which stands on `requirement` and names no `to_kind`. So the first build issue adds an expectation rather than a widened one, whichever way ruling 1 had gone.

**The cadence axis of `verification_method` stays open.** The three states of ruling 3 are the cadence of a `test`. An `inspection` has no such transport, and this record does not give it one.

**A reader outside this repository is the reason for the work.** An adopter with a requirements tradition wants a test to name what it proves. Nobody is stopped today, because `traces_to` onto a `code_path` carries the link at the cost of the identity.
