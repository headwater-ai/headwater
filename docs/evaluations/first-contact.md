---
id: HW-EVAL-first-contact
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: The Q11, Q12 and Q16 evidence, which is the license, the migration path, and the public presence that a first reader meets.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: evaluate+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cited_by:
    - HW-REG-decisions
    - HW-SPEC-vision-and-scope
    - HW-SPEC-assurance-model
    - HW-SPEC-engine-architecture
    - HW-SPEC-distribution-and-federation
---

# First contact — the Q11, Q12 and Q16 evaluation

This settles [Q11](../spec/09-open-questions.md#q11--license-and-distribution-posture), [Q12](../spec/09-open-questions.md#q12--migration-path-for-an-existing-corpus) and [Q16](../spec/09-open-questions.md#q16--public-presence) together. It is the sixth and last of the grouped evaluations that closed the register, and after it every entry is closed or explicitly pending.

One of the three closes differently from every other question in this specification. Q11 is a decision about what the owner of this project intends for it, and no argument from the design settles that. What the design *can* do is narrow the field, and it narrows it further than the entry expected. This evaluation lays out the options, shows which of them existing rulings already refuse, states a recommendation with its reasoning, and stops there. Q11 is marked pending the owner's ratification, and the last section of the Q11 decision says exactly what ratifying it would mean.

## Why the three are one question

The three entries describe one path, walked by one person who does not work here.

They hear the name (Q16). They read enough to decide whether it is worth an hour (Q16). They check the terms, because an engineering organization that cannot answer "what is the license?" does not start (Q11). Then they point it at a corpus that was never written to any of this, and the first run tells them whether the next step is an afternoon or a quarter (Q12).

Each entry fails on its own. A site with nothing to try is a brochure. A permissive license that nobody has heard of buys nothing. An on-ramp that arrives after the first impression arrives late. Rogers' account of what predicts the adoption rate of an innovation names the two attributes that split cleanly across these three entries: **trialability** — can I try it before I commit? — and **observability** — can I see that anyone else is using it, and what it did for them? Q11 and Q12 are the whole of trialability. Q16 is the whole of observability. The theory says the two lower uncertainty by different routes and that neither substitutes for the other, which is the reason to settle them in one place.

Q11 goes first inside that, because Q16 cannot fill two of its ten rows until Q11 is ratified, and because Q12's on-ramp is worth nothing to somebody who may not run the tool.

## What the specification already fixed

### For Q11: eight rulings, and together they refuse two of the four options

Q11's entry names four postures: open source, source-available, internal-only, and the separate question of whether the taxonomy package ships under the terms of the engine. The entry treats all four as open. Rulings already made close more than that.

**1. Spec 6 requires an embeddable library, and [Q1](../spec/09-open-questions.md#q1--implementation-language) chose the language for that requirement alone.** [Spec 6](../spec/06-engine-architecture.md#library) says that editor integrations, the MCP server and CI adapters consume the library in-process and never start a subprocess. Q1's decisive argument was that Rust reaches a Node, Python or JVM host as a shared library and reaches a browser through WebAssembly. Each of those hosts belongs to the adopter. A license whose obligations extend to a work that links the library therefore reaches into software the adopter owns, and it makes the embedding that the whole language choice bought into a legal event. That refuses GPL-3.0 and AGPL-3.0 for the library. It does not refuse MPL-2.0, whose copyleft is per file and survives linking, which is why Terraform carried MPL-2.0 for a decade.

**2. [Principle 1](../spec/00-vision-and-scope.md#design-principles) puts the adopter's asset in the taxonomy and not in the engine.** Configuration over code means that an adopter who wants something different writes a schema and never patches the engine. So the engine's license governs almost nothing that an adopter builds. Three things must stay unobstructed: run the engine in CI, publish a taxonomy package, and consume an emitted artifact. All three are use and distribution, and every candidate license permits all three. This is why the license question is smaller than it looks for the engine, and larger than it looks for the package.

**3. The base package is data that every adopter's taxonomy resolves over, and that decides its terms.** [Q3](../spec/09-open-questions.md#q3--how-much-of-the-default-taxonomy-ships-in-the-box) ships a minimal base plus add-only bundles, and [Q2](../spec/09-open-questions.md#q2--schema-format) makes an overlay a patch whose resolution contains base content. An adopter's resolved taxonomy and lock therefore contain the base. A copyleft or share-alike term on the base propagates into that resolved artifact, and [spec 0](../spec/00-vision-and-scope.md#who-this-is-for) promises the opposite in its own words: the taxonomy is theirs to define, and organizations whose documentation culture does not match ours are explicitly served. **The base package and the bundles must impose nothing on a derived taxonomy.** That is the one part of Q11 that the specification decides on its own, and it decides it against the entry's framing, which treated the package half as a preference.

**4. [Q13](../spec/09-open-questions.md#q13--linkml-and-shacl-as-substrate)'s staging order makes an external consumer the trigger for four of six emitters.** SHACL, RDF and SKOS, OKF and LinkML each ship when a named external consumer asks. Under internal-only there is never one. [Principle 9](../spec/00-vision-and-scope.md#design-principles) then has nothing to be better together with, four emitters are dead by construction, and the staging order is a fiction rather than a plan. **Internal-only is inconsistent with rulings already made.** It is not merely unattractive.

**5. [Q17](../spec/09-open-questions.md#q17--governed-access-and-the-solution-layer) made this security software with a published claim.** [Spec 6](../spec/06-engine-architecture.md#what-a-filtered-export-claims-and-what-it-does-not) states one claim and five non-claims, and Q17 takes on a coordinated-disclosure process and a class of defect that nobody can fix forward. Two consequences follow. A disclosure process needs a reporting channel and a route by which a fix reaches adopters, which is a distribution obligation. And a tool whose output enters other people's compliance pipelines is exactly where a patent claim would be expensive. MIT is silent on patents. Apache-2.0 grants a patent license and terminates it for a contributor who sues. That is a reason to prefer one permissive license to another, and it comes from Q17 rather than from taste.

**6. [Q8](../spec/09-open-questions.md#q8--probe-cost-and-cadence) put real money in exactly one place, and [Q7](../spec/09-open-questions.md#q7--scope-of-the-mcp-surface) and Q17 left the same hole open twice.** A campaign run costs a line item in the low hundreds of dollars, and the harness declares a budget and fails closed. Q7 and Q17 both record the same open point: what a hosted server is, operationally, and who runs it. Those two places — a hosted server, and a hosted probe harness — are the only points in the whole specification where a commercial tier could sit, and neither is specified. A license chosen to protect a hosted business would be protecting a business that this specification has not described.

**7. [Q10](../spec/09-open-questions.md#q10--naming) fixed `https://w3id.org/headwater/` as the namespace.** A namespace URI inside an emitted artifact is a promise about a name that outlives every release. Control of the name is a governance instrument and not a license one, and Apache-2.0 says so explicitly by granting no trademark rights. This is the one lever a permissive license does not give away, and it is the lever that matters most for [Q16](../spec/09-open-questions.md#q16--public-presence).

**8. [Principle 8](../spec/00-vision-and-scope.md#design-principles) makes this repository the demonstration.** The system governs itself, and today this corpus is the only Headwater corpus in existence. Under internal-only, the half of principle 8 that anyone outside can check cannot be shown at all.

### For Q12: the migration state already has the shape, and one field blocks it

[Spec 7](../spec/07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states) records a migration state in the lock: a from-version, a to-version, an owner, an expiry, and the open task list. While tasks remain open, checks run against the new schema, and a finding whose `(document, rule)` pair the migration payload expects to fail is reported as `migration-pending`. Such findings are counted, visible in coverage, never blocking, and never suppressed individually. The expiry is the anti-parking device, renewable only by an explicit move of the date.

Read that list against first contact and only one item refers to a known-good starting point: the from-version. Every other property — the pair grain, the owner, the expiry, the counted-and-visible posture, the task list — is defined against the *new* schema and says nothing about what came before.

Three further rulings bear on the on-ramp.

- [Spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for) states three coverage obligations. Every file under the corpus root is classified or reported as unclassifiable, every classified document is routed to at least one check, and every run reports what it saw, classified, checked and skipped, with reasons. A document that is not checked is a finding, never an omission.
- [Spec 6](../spec/06-engine-architecture.md#cli) already carries `headwater check --changed-only`, and it exists for the 200 ms commit-hook budget. It is a performance scope.
- [Spec 6](../spec/06-engine-architecture.md#cli) also states the posture in one sentence: the CLI is advisory by default, because a tool that blocks on first contact is removed, and a removed tool catches nothing.
- Q3 already applied the same instinct to content. The base package declares `controlled: none`, because a controlled-language profile turned on by the *base* meets an adopted corpus of two hundred documents with a wall of findings on the first run.

### For Q16: Q14 handed this entry a definition, not only a task

[Q14](../spec/09-open-questions.md#q14--discovery-surface) split discovery into resolution and registration and closed the first. Resolution is a machine that holds a location and learns what governs it, and the corpus descriptor at `.headwater/corpus.json` answers it. Registration is how a machine learns that a corpus exists when it holds no pointer at all, and Q14 refused it on a general ground: no file inside a corpus answers it, and every convention in the field presumes a client that already resolved a name. Three package ecosystems put a capability document inside an index and discover the index in none of them.

Two more rulings constrain the site itself.

- [Spec 0](../spec/00-vision-and-scope.md#what-we-do-not-build) refuses to build a documentation renderer, and names the alternative: emit navigation configuration for a static-site generator. [Spec 0](../spec/00-vision-and-scope.md#what-we-build) item 3 already lists site navigation among the projections.
- [Principle 11](../spec/00-vision-and-scope.md#design-principles) publishes an unmeasured claim as unmeasured, and [spec 4](../spec/04-assurance-model.md#the-systems-own-assurance) already says that the same command an adopter runs generates the coverage numbers this project publishes.

## Prior art and observed applications

[Principle 10](../spec/00-vision-and-scope.md#design-principles) asks for the research literature and at least one observed industry application. The sources below are recorded in [HW-EVAL-adjacent-work §S](../evaluations/adjacent-work.md#s--terms-on-ramps-and-being-found). What each one confirmed, sharpened or contradicted is stated here.

### What a relicensing costs, measured in forks

Three episodes are available and two of them were read directly.

HashiCorp announced on 10 August 2023 that Terraform and its other products moved from MPL-2.0 to the Business Source License v1.1. The announcement permits "copying, modification, redistribution, non-commercial use, and commercial use under specific conditions", and excludes anyone "providing a competitive offering to HashiCorp". Fifteen days later a public fork appeared. The OpenTofu manifesto states the reason in the language of uncertainty rather than of principle: "every company, vendor, and developer using Terraform has to wonder whether what they are doing could be construed as competitive with HashiCorp's offerings." OpenTofu then joined the Linux Foundation, in its own words so that the project stays "truly open source and neutral and not at the whim of any one company."

Elastic supplies the other half of the arc, and it is the more useful half because the company published a retrospective. Its founder wrote on 29 August 2024 that Elasticsearch and Kibana would add AGPL beside the existing Elastic License and SSPL. On the 2021 change he wrote: "We had issues with AWS and the market confusion their offering was causing." On the result: "Amazon is fully invested in their fork, the market confusion has been (mostly) resolved, and our partnership with AWS is stronger than ever." The post declines to call the original change a mistake. It does not have to. A relicensing whose stated goal was to prevent a competitor's offering produced a competitor's offering that is now permanent, and the licensor added an open-source license back three years later.

Valkey is the third, and the site confirms the shape rather than the history: a BSD-licensed key-value datastore "backed by the Linux Foundation, ensuring it will remain open source forever", carrying BSD-licensed code from the original project and noting that the original name is a registered trademark of the original company.

Two things transfer, and one of them contradicts a common reading.

**What transfers.** A restriction on a competing offering does not read to a user as a restriction on a competitor. It reads as a question that the user cannot answer about themselves, and the OpenTofu manifesto is the primary evidence for that, in the words of the people who forked. This bears directly on [principle 9](../spec/00-vision-and-scope.md#design-principles). Headwater's stated plan is to build integrations with complementary projects rather than replacements for them, and [Q13](../spec/09-open-questions.md#q13--linkml-and-shacl-as-substrate) names LeanCTX as the consumer that would trigger the fifth emitter. Under a source-available license, every adjacent project that considers an integration has to ask a lawyer whether it competes. That is a cost paid by exactly the parties principle 9 wants.

**What it contradicts.** The reading that a relicensing simply destroys a project is not what these cases show. Terraform and Elasticsearch both still exist and both still sell. What the cases show is narrower and more useful: the license change bought the licensor a permanent, well-funded fork, transferred stewardship of the open version to a foundation controlled by others, and cost the name. Valkey and OpenTofu both live under the Linux Foundation. The trademark was the only asset that stayed.

Three details could not be verified within this evaluation's budget and are marked rather than presented as read: the number of companies and engineers pledged to OpenTofu, the dates and license identifiers of the 2021 Elastic change and the OpenSearch fork, and Redis's own relicensing sequence. The web-search budget for this session was exhausted, and the pages above were fetched directly. This is [HW-EVAL-adjacent-work §R](../evaluations/adjacent-work.md#r--measuring-whether-the-corpus-works)'s convention, applied to a source that was not reachable rather than to one that was paywalled.

### The contribution agreement is the relicensing lever

The mechanism that makes a unilateral relicensing possible is not the license. It is the inbound agreement.

The Developer Certificate of Origin, version 1.1, is a certification and not a grant. A contributor certifies that they wrote the contribution or have the right to submit it under the project's license, that they understand the contribution is a public record, and nothing else. It transfers no additional rights to anybody.

A contributor license agreement can transfer more, and the open-source guidance is explicit about when one is worth its friction. It lists four cases, and the fourth is the relevant one: the project "uses copyleft licensing but needs a proprietary version". The same guidance records the default: "For the vast majority of open source projects, an open source license implicitly serves as both the inbound (from contributors) and outbound (to other contributors and users) license." It also records a cost with a named example — Node.js removed its CLA to lower the barrier to entry and broaden the contributor base.

This sharpens the recommendation below rather than merely informing it. The unique power a CLA gives a project is the power to change the terms later without asking. The evidence above is that exercising that power is what produced the forks. A project that does not want the power should not collect it, and the DCO is the instrument that says so credibly, because it makes the promise structural rather than stated.

### Where the closest analog draws the commercial line

[Spec 0](../spec/00-vision-and-scope.md#what-we-do-not-build) names Vale as the closest observed analog to this engine. Vale is MIT-licensed. Its author asks for sponsorship, and sells two hosted products beside the tool: a hosted authoring platform for building and maintaining style guides, and a hosted layer for managing configuration and rules in a browser. The command-line checker is not the product. The place where a person cannot easily self-host is.

That is the same line that Q7 and Q17 both left open — what a hosted server is, operationally — and the same line Q8 drew when it found that money is not the constraint on the measurement layer. The engine is cheap to run and expensive to build. A hosted authoring and measurement layer is expensive to run and is where a per-seat or per-run charge would land. The observed case draws the line where this specification's own open points already sit, which is stronger evidence than an argument would have been.

### Gradual adoption is a declaration in the tree, never a flag on the run

The typed-language migrations are the closest analog to a corpus that was never valid, and the strongest of them is Sorbet, because it is the one that made the strictness level a property of the file.

Sorbet reads a `# typed:` sigil at the top of each Ruby file, with five levels: `ignore`, `false`, `true`, `strict` and `strong`. The default for a file with no sigil is `# typed: false`, at which only syntax, constant resolution and signature correctness are reported. The purpose of the levels is stated as gradual adoption: a team strengthens checking file by file rather than everywhere at once.

Two properties of that design are the finding, and neither is about types.

**The level is committed, reviewed and diffable.** It is in the file, so two runs over one tree agree, a change of level appears in a pull request, and no invocation of the tool can change a verdict. **The default is the level at which almost nothing fires.** An untouched file is not exempted from the tool. It is checked at a level that a never-typed codebase already satisfies, which is a different thing, because the file stays in the denominator.

Headwater already has both properties and reached them from another direction. The regimes are declared per kind and per shelf in the taxonomy, which is committed and reviewed, and Q3 already made the base package the level at which almost nothing fires by declaring `controlled: none`. Sorbet's sigil and Headwater's base package are the same device. Nothing new is needed on this axis, and the contribution of the source is to refuse the alternative: Q12's leaning proposed `--since <ref>`, which is the same intent expressed as a flag on the run.

### Grandfathering has a scale at which it lies, and one tool names the number

Three grandfathering mechanisms were read, and they differ in exactly the property that matters at first contact.

| Tool | What is recorded | What happens when a violation is fixed |
|---|---|---|
| RuboCop `.rubocop_todo.yml` | A raised `Max` for a metric cop, a list of excluded files, or the cop disabled entirely | The documentation states no report. `--regenerate-todo` rewrites the file when a person runs it |
| ESLint `eslint-suppressions.json` | A count of suppressed violations per rule per file | The run **fails**: "There are suppressions left that do not occur anymore." `--prune-suppressions` removes them |
| This repository's `.ste-lint-baseline.json` | A hash of file, rule and the offending text | The hash stops matching, so the entry silently becomes dead. Nothing reports it |

RuboCop supplies the sharper finding, and it is a number. `--auto-gen-config` excludes offending files one by one until `--exclude-limit` is reached, and the default limit is 15. Past 15 files, the cop is **disabled entirely** rather than carrying a longer exclusion list. That is a reasonable default for a mature codebase adding one rule. It is precisely wrong for first contact, because first contact is the one moment at which every rule exceeds any such threshold. A mechanism that degrades from per-instance accounting to whole-rule silence converts an on-ramp into the failure that [spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for) exists to forbid, and it does it exactly when the corpus is least trustworthy.

ESLint supplies the property this repository's own baseline lacks: a suppression that is no longer needed is an error, not dead weight. Headwater's `migration-pending` already has the stronger version, because a payload declares a `(document, rule)` pair set and a pair that now passes is a task that closes. What the source adds is the argument for reporting the remaining count on every run rather than only at the expiry.

### What predicts adoption, and which attribute each entry supplies

Rogers' account of diffusion names five attributes of an innovation that predict its rate of adoption: relative advantage, compatibility, complexity, observability and trialability. Trialability is the ability to experiment before committing, and observability is whether the benefits are visible and whether a potential adopter can see somebody else using it. Both work by reducing uncertainty rather than by increasing benefit.

Read against the three entries, the mapping is exact and it explains the sequencing.

| Attribute | What supplies it here | State |
|---|---|---|
| Relative advantage | Spec 0's thesis, and the comparison against every adjacent tool in [HW-EVAL-adjacent-work](adjacent-work.md) | Argued, unmeasured ([principle 11](../spec/00-vision-and-scope.md#design-principles)) |
| Compatibility | Principle 1 — the adopter's taxonomy, not ours. Q13's emitters | Specified |
| Complexity | Q3's minimal base and the interview | Specified |
| Trialability | Q11's terms, and Q12's on-ramp | Q11 pending, Q12 closes here |
| Observability | Q16's site, and the benchmark rows | Q16 closes here, and one row must stay empty |

The theory's contribution is the last row, and it is uncomfortable. Observability is supplied by *visible results*, and [principle 11](../spec/00-vision-and-scope.md#design-principles) forbids publishing a result that no run produced. So the attribute the site exists to supply is the one this project has decided it may not fabricate. That is the correct trade and it is worth naming, because the pressure to relax it will arrive precisely when the site does.

### Four documentation modes, and the site is a corpus

Diátaxis names four kinds of documentation on two axes — practical against theoretical, and specific against general — yielding tutorials, how-to guides, reference and explanation. Its claim is that content, style and organization all follow from keeping the four apart.

This lands on Q16 through [principle 8](../spec/00-vision-and-scope.md#design-principles) rather than as advice. If the site is generated from the corpus that documents Headwater, then the site's sections are shelves and its page types are kinds. The four modes are a candidate kind set for a documentation-site bundle, and the Q3 walkthrough's own finding applies: the specification documents here serve two purposes at once, stating a model and arguing for it in the same file, with `docs/evaluations/` already carrying the argument. That split is reference against explanation, found independently in this repository before the framework was consulted.

## The decision — Q11

### The options, and what each one forecloses

| Option | What it buys | What it forecloses | Which ruling refuses it |
|---|---|---|---|
| **Permissive** (Apache-2.0, MIT, BSD) | Trialability with no legal review. Embedding under spec 6. A taxonomy publisher may ship a plugin. Distribution packaging | Any later restriction on a hosted competitor, unless every contributor agrees | None |
| **Weak copyleft** (MPL-2.0) | Engine improvements return. Still links into a host the adopter owns | Little, and it adds a compliance question at every embedding site | None, but it taxes constraint 1 |
| **Strong or network copyleft** (GPL-3.0, AGPL-3.0) | Improvements return, including from a hosted service | The in-process embedding that Q1 selected the language for | Spec 6's library requirement (constraint 1) |
| **Source-available** (BUSL, ELv2, SSPL) | Protects a hosted business | OSI status, distribution packaging, and principle 9's integrations. The SSPL specifically is not an open-source license, on the OSI's stated ground that restricting a field of endeavor violates OSD 6 | Principle 9 and Q13's staging order (constraint 4); and it protects a business Q7 and Q17 have not specified (constraint 6) |
| **Internal-only** | Nothing that this specification asks for | Q13's emitters 3 through 6, principle 9 entirely, principle 8's public half, Q17's disclosure channel, and Q16 | Constraints 4, 5 and 8 |

Two of the five are refused by rulings already made, and one is refused for the library. The specification narrows Q11 to permissive or weak copyleft, and it does not choose between them. **That remaining gap is the owner's, and it is why this entry is pending rather than closed.**

### The recommendation

**Engine and library: Apache-2.0.** It is permissive, so constraint 1 is satisfied and there is nothing for an adopter's legal review to resolve before a trial. It carries an express patent grant with a termination clause, which MIT lacks and which constraint 5 argues for directly. It reserves trademark rights explicitly, which is what keeps the name and the `w3id.org/headwater` namespace under the project's control while the code stays free — the one asset the observed relicensing episodes show a licensor keeps. It is also what the nearest neighbors use: LeanCTX is Apache-2.0, and Sorbet is Apache-2.0.

**Base package and bundles: the same terms as the engine.** Constraint 3 requires only that they impose nothing on a derived taxonomy, and Apache-2.0 does not. A separate public-domain dedication would also satisfy the constraint and would cost a second license to explain. One set of terms for everything a machine reads is the simpler position, and simplicity is a Q11 criterion because the adopter reads this before they read anything else.

**Doctrine prose: Creative Commons Attribution 4.0.** The doctrine is prose that explains a method, vendored into consumers under [spec 7](../spec/07-distribution-and-federation.md#what-is-shared-and-what-is-not). A software license applied to prose is a category error that a reviewer will notice, and [HW-EVAL-adjacent-work §P.7](../evaluations/adjacent-work.md#p7-debian-settles-redistribution-by-segregating-the-archive) already records the precedent for separating terms by content class. The boundary is mechanical rather than a matter of judgment: `contents.doctrine` in the package declaration is one of the six content paths, so the line is already drawn in the schema.

**Contributions: the Developer Certificate of Origin, and no CLA.** State the reason positively. The unique power a CLA confers is the power to relicense later without asking, and the recommendation is not to want that power. A DCO makes that promise structural. This forecloses a future dual-licensing business, and that is the intent.

**Governance: a stated commitment now, a foundation as a named upgrade path.** OpenTofu and Valkey both used a foundation to make "this will not be relicensed" credible, and both did so *after* an incident, with a community already in place. At zero adopters a foundation is process with nobody to protect. What is available now and costs nothing is a dated statement of the terms, the DCO, and a written commitment that the license will not narrow — with the foundation named as what would be done if adoption made the commitment worth more than one person's word.

### What ratifying this would mean, concretely

Q11 stays pending until these exist. Each is a file or a public statement, and together they are the answer an outsider is looking for.

1. A `LICENSE` file at the repository root, and an `SPDX-License-Identifier` convention for source files.
2. A `NOTICE` file, and a `CONTRIBUTING.md` that states the DCO and requires `Signed-off-by`.
3. A `SECURITY.md` with the coordinated-disclosure process that [Q17](../spec/09-open-questions.md#q17--governed-access-and-the-solution-layer) already obliges, and which nothing currently discharges.
4. The content license for `docs/`, marked where a reader will find it.
5. A statement of the trademark position for the name and the domain — including, if that is the answer, that there is none.
6. The date. A term with no date attached cannot later be shown to have changed, and the whole value of the commitment is that a change would be visible.

### What the specification cannot decide, stated plainly

Every constraint above rules an option *out*. Not one of them rules exactly one option *in*. Between MIT and Apache-2.0 the design is indifferent except for the patent and trademark clauses, and constraint 5 is an argument rather than a proof. Between Apache-2.0 and MPL-2.0 the design is nearly indifferent, and the choice turns on whether the owner wants engine improvements to return, which is a question about intent and not about architecture. Choosing a source-available license remains possible, and it would require reopening [principle 9](../spec/00-vision-and-scope.md#design-principles) and Q13's staging order and saying so. That is the honest map. A confident answer here would be a worse document.

## The decision — Q12

### First contact is a migration from no taxonomy, and one field is all that blocks it

The entry states the problem correctly: the migration state is defined against a known-good starting point, and a corpus that was never valid has none. The conclusion drawn from that — that first contact needs its own mechanism — does not follow. Only the from-version refers to the prior state. The pair grain, the owner, the expiry, the task list, and the counted-visible-never-blocking posture are all defined against the *new* schema.

**So the from-version becomes optional, and everything else is unchanged.** Adoption is a migration whose source is the empty taxonomy. Before `headwater init` a corpus is governed by nothing and every document is trivially valid, so the set of findings that the new taxonomy produces over the existing tree *is* the migration payload of that taxonomy's first version. The publisher computes a payload from a diff between two majors. At first contact, `infer` computes it from a diff between nothing and one.

That gives one mechanism where the entry expected two, and it gives first contact four properties that no baseline file has: a pair grain that keeps yesterday's defect loud while the declared debt stays patient, an owner, an expiry that is the anti-parking device, and a place in the coverage report.

### `headwater infer` emits three artifacts, and the third is the one that closes this

Q3 settled that `infer` and the `init` interview are one command with two evidence sources, and that it emits an overlay rather than a resolved taxonomy. The entry adds a report of what does not fit. The ruling above adds a third output, and it comes free because all three come from one read of the tree.

| Artifact | What it is | Settled by |
|---|---|---|
| The overlay | The bundle selection and the shelf paths, as a patch over the base | [Q3](../spec/09-open-questions.md#q3--how-much-of-the-default-taxonomy-ships-in-the-box) |
| The misfit report | What the tree contains that no proposed shelf or kind explains | The entry |
| The **adoption payload** | The `(document, rule)` pairs that the proposed taxonomy expects to fail, with an owner and an expiry | Here |

`infer` cannot make the corpus valid by weakening the base, and that is a structural property rather than a rule anyone has to enforce. Q3 made bundles add-only over disjoint addresses, which is what makes every subset resolve. An add-only overlay has no operation that removes a base rule. So an inference that optimized for zero findings — which would encode the corpus's accidents as if they were intentions — is not expressible in the artifact `infer` is permitted to write.

### `--since <ref>` as a gate is refused, and the flag it would have duplicated already exists

The leaning proposes an incremental adoption mode "where checks apply only to newly touched documents", as a first-release feature. Three arguments refuse it, and one of them is that the word already means something else in this specification.

**It makes two runs over one tree disagree.** [Spec 6](../spec/06-engine-architecture.md#implementation-constraints) requires that the same corpus and the same lock produce byte-identical output. A flag that decides which findings count is a second input to the verdict that no reviewer sees, and [Q6](../spec/09-open-questions.md#q6--where-the-corpus-graph-lives-at-rest) already refused the same shape for the cache: an artifact that can change a verdict is a store under another name.

**It converts an unchecked document into an unreported one.** Spec 4's coverage obligations require every classified document to reach a check and every run to account for what it skipped, with reasons. A run scoped to touched documents either reports the rest as skipped — in which case nothing was gained, since `migration-pending` already reports them better — or it does not, which is the silent pass that the whole coverage doctrine exists to forbid.

**The name is taken.** `headwater check --changed-only` already exists, for the 200 ms commit-hook budget. It is a performance scope over a verdict that a full run would reach identically. Giving a second flag posture semantics would put two mechanisms with opposite properties behind nearly the same word, and the one that reviewers already trust would be the one that gets confused.

**What the adopter actually wanted is the payload.** The desired experience is a green build on day one with the debt visible. `migration-pending` delivers exactly that, and it delivers more: each item has a name against it, an expiry, and a line in coverage. The difference is that the debt is a fact about the corpus rather than a property of how somebody invoked the tool.

### No threshold ever converts an accounting into a silence

RuboCop disables a cop entirely past 15 excluded files. First contact is the moment every rule exceeds any such limit, so a threshold of that shape would fire on the whole rule set at once, and the adopter would receive a green run over a corpus in which most rules had been switched off without anybody choosing that.

**Headwater declares no such threshold.** The adoption payload holds `(document, rule)` pairs however many there are, and the honest cost is stated rather than avoided: on a large adopted corpus that payload is large, and it sits in the lock, which is committed and reviewed. That is the same trade the projection census makes — a full accounting is bigger than a summary, and being bigger is what makes it an accounting.

**And the remaining count is reported on every run.** ESLint fails a run that carries a suppression which no longer matches, which is the property `.ste-lint-baseline.json` lacks. Headwater does not need the failure, because a pair that passes is a task that closes and the expiry already forces the conversation. What it needs is the number, reported beside coverage on every run, so that a payload which is not shrinking is visible long before its expiry rather than at it.

### What this repository can already say about it, at its real strength

This repository ran the pattern in miniature, and the numbers are real. `.ste-lint-baseline.json` grandfathered **65** violations when the check landed (commit `3665182`). It holds **14** today (commit `290a84d`). That is a 78% reduction over the design phase.

Stated at the strength the evidence supports, which is lower than the number suggests:

- The reduction is genuine and the mechanism is the hash key. An entry is keyed to file, rule and offending text, so editing the sentence invalidates its entry. The count cannot go backwards without somebody noticing.
- The reduction is **not** evidence that anyone works a debt list. Most of it came from a commit that fixed the sentence splitter and then rewrote the sentences that were genuinely too long — that is, from prose touched for other reasons, and from a defect in the checker rather than in the corpus.
- The baseline has no owner and no expiry, which is exactly [Q21](../spec/09-open-questions.md#q21--terminological-succession-and-validity-under-merge)'s finding about it. So this measurement says nothing about whether an owned, expiring payload gets closed, which is the claim the adoption payload actually makes.

That claim is therefore **unmeasured**, as [principle 11](../spec/00-vision-and-scope.md#design-principles) requires. The instrument is the remaining pair count of an adoption payload over time, per adopter, and the fraction of payloads that reach zero before their expiry. No adopter exists.

## The decision — Q16

### Registration is publication into a channel whose reader is already obliged

Q14 refused registration on the ground that no file inside a corpus performs it. That is right, and it is a symptom rather than the reason. The reason is that **registration is an act of publication, and a publication needs a channel whose reader is already obliged to read it.** A file inside a corpus fails because it is not in anybody's channel. `llms.txt` fails for the same reason at a larger radius: about 137,000 domains, 97% of valid files unread in a month, no provider obliged to read one ([HW-EVAL-adjacent-work §O.3](../evaluations/adjacent-work.md#o3-llmstxt-is-the-measured-failure-of-a-descriptor-with-no-obliged-reader)). It is not a discovery surface, however cheap it is to write.

Once registration is defined that way, the machine half closes with no new machinery, because Headwater already uses two channels that have obliged readers.

- **The package channel.** A taxonomy package is published to a registry that a resolver must read to install it ([spec 7](../spec/07-distribution-and-federation.md#publishing)). A resolver is an obliged reader by construction. So a package that carries the location of the publisher's own corpus registers that corpus with every consumer who installs the taxonomy.
- **The served page.** [Spec 7](../spec/07-distribution-and-federation.md#arriving-at-a-corpus-cold) already requires a rendered page to carry a link relation to the served descriptor. A page that a search engine or an agent already fetches is a channel with a reader.

**Registration is therefore the publisher's own act, in a channel that already exists, and Headwater builds nothing for it.** What Headwater supplies is the payload — the descriptor — which Q14 already settled.

### A registry or directory of Headwater corpora is refused

Not deferred. Three rulings converge, and each one already refused a smaller version of the same thing.

[Q9](../spec/09-open-questions.md#q9--multi-repository-corpora) refused query fan-out, in part because a fan-out that meets an unreachable source returns a smaller answer with no notice. Q14 refused a reserved path at the root of an origin, because it fixes one service to one site and needs control of the apex. A central directory is both failures at the largest radius, and it adds one that neither has: it would be the single piece of Headwater infrastructure that must stay online for discovery to work, in a system whose [non-negotiables](../spec/00-vision-and-scope.md#non-negotiables) include running offline with the same result as CI.

The cost of the refusal is real and is stated. There is no way to enumerate Headwater corpora, and there will not be one. An organization that wants its corpora enumerated builds a solution corpus and pins them ([Q9](../spec/09-open-questions.md#q9--multi-repository-corpora)), which is enumeration at the scale where somebody owns the list.

### The site is a projection of this corpus, and no generator is built

[Principle 8](../spec/00-vision-and-scope.md#design-principles) applied to the public surface: a governance system whose own public documentation is ungoverned fails the first check a skeptical reader runs. The entry states that constraint, and the ruling makes it cheap rather than aspirational.

**Spec 0 already refuses to build a renderer, and already lists site navigation as a projection.** So the division is settled before the question is asked. Headwater emits the navigation configuration and the content from the corpus. A third-party static-site generator renders it. No site generator is a Headwater component, and this entry does not create one.

That matters for a reason beyond scope. The owner's standing position is that the STE enforcement scaffolding in this repository is interim and should retire into Headwater's own check layer rather than grow. A Q16 ruling that quietly commissioned a bespoke site generator would repeat that mistake at a larger size. **Nothing here is built until the engine exists**, and when it does the site is an emitter target and a projection, which are two mechanisms that already have owners.

### The sitemap, run now, against the specification

The entry says to draft the sitemap early because it is a forcing function, and that every facet is a question the specification should already answer. That instruction was followed. The result is a measurement of the specification rather than a plan for a site.

| Facet | What answers it today | Verdict |
|---|---|---|
| How it works, architecture | [Spec 6](../spec/06-engine-architecture.md), [spec 1](../spec/01-conceptual-model.md), [spec 12](../spec/12-check-layer.md) | Answered |
| Benchmarks, metrics | Nothing. Every efficacy claim is marked unmeasured | **Empty, and must stay empty** |
| Comparisons | [HW-EVAL-adjacent-work](../evaluations/adjacent-work.md), [spec 8](../spec/08-design-departures.md), spec 0's table of what we do not build | Answered, and the strongest row |
| Use cases | The five adopters of the [Q3 walkthrough](default-taxonomy-first-run.md#five-first-runs) | Answered, from an evaluation |
| Compatibility, integrations | [Q13](../spec/09-open-questions.md#q13--linkml-and-shacl-as-substrate)'s six emitters, of which two ship | Answered, and the answer is "two" |
| Docs, getting started | [Spec 3](../spec/03-authoring-and-lifecycle.md) and the [interview](../spec/07-distribution-and-federation.md#the-interview) | Partial. No quickstart, because there is nothing to start |
| Pricing, enterprise, consulting | [Q11](../spec/09-open-questions.md#q11--license-and-distribution-posture) | Blocked, pending ratification |
| Compliance, audits, self-assessment | [Spec 4](../spec/04-assurance-model.md)'s obligation and gap registers, [spec 6](../spec/06-engine-architecture.md#what-a-filtered-export-claims-and-what-it-does-not)'s claim and five non-claims | Answered, better than most projects manage |
| Changelog, community, open-source posture | [Q11](../spec/09-open-questions.md#q11--license-and-distribution-posture). There is no changelog and no community | Blocked |
| `llms.txt`, AI-crawler `robots.txt` | Cheap to emit, measurably unread | Ship it, and count it as nothing |

**The getting-started row above is the measurement as it stood, and it has since moved.** *No quickstart, because there is nothing to start* was true on the day this sitemap ran. M1 and M2 shipped the verbs, and [the tutorial](../tutorials/your-first-governed-corpus.md) takes a reader from an empty directory to a passing strict run. [Q16](../decisions/0016-public-presence.md) carries the row that holds now, and the table above stays as it was measured.

Four rows are answered from the specification, one from an evaluation, one is partial, two are blocked on Q11, one is empty by principle 11, and one is worth approximately nothing. **The forcing function worked, and what it found is that the largest hole in the public story is the one the project has decided it may not fill.** The benchmark row is empty not because the work is pending but because principle 11 forbids a number that no run produced, and it will stay empty until a campaign runs.

The second finding is quieter and more useful. The comparison row is the strongest asset this project has, because [HW-EVAL-adjacent-work](adjacent-work.md) already carries counter-evidence against Headwater — OpenGEO declining the stack, TrustGraph shipping the opposite mechanism, Vale being the closest analog and being in another language, and §M stating that the survey licenses no efficacy conclusion at all. A comparison page that includes the arguments against the thing is unusual enough to be the differentiator, and it costs nothing because it is already written.

### Honest before impressive, made mechanical

The entry's constraint is that the benchmark and self-assessment rows must be honest before they are impressive. As stated it is an intention, and intentions are what §I.4 records failing at a neighboring project, where the claims moved between README versions.

Because the site is generated from the corpus, the constraint can be a mechanism instead. **Every number on the site is generated from the evidence register, and a claim with no instrument is generated as unmeasured.** [Spec 4](../spec/04-assurance-model.md#the-systems-own-assurance) already states that the same command an adopter runs produces the coverage numbers this project publishes. Extending that to the public surface costs nothing new: a figure can appear only where a run put it, and `generate --check` holds the page to its source exactly as it holds every other projection.

Two consequences. A hand-written number on the site is a finding, in the same way that a hand-edited shelf index is. And the self-assessment states that it is self-published — the standard §I.4 applied to a neighbor, applied here to ourselves, which is what principle 8 means when it is inconvenient.

### Timing: the leaning survives, with the trigger derived

The leaning defers the site until an engine makes it worth a visit. That is right, and the reason is stronger than "worth a visit". A site supplies observability. It cannot supply trialability, and the diffusion literature is clear that neither substitutes for the other. A site that describes a tool nobody can run produces interest with no path to adoption, and it spends the first impression — which the entry itself identifies as the thing that arrives before anyone notices it matters.

So: **the site ships when the engine ships.** The sitemap is drafted above, three of its rows are blocked on Q11, and one is blocked on a campaign run that has no corpus to run against. Those are the dependencies, and they are all recorded.

## What stays open

**Q11 is pending the owner's ratification.** The six items in "what ratifying this would mean" are the work. Until they exist, this repository has no stated license, which is itself the first thing an outsider checks.

That sentence was true when this evaluation ran, and it is no longer true. The owner ratified Apache-2.0 on 2026-08-11, and all six artifacts exist. The paragraph stays as written, because this document records what the evaluation found rather than what the project later did. [Q11](../spec/09-open-questions.md#q11--license-and-distribution-posture) carries the current state.

**What a hosted server is, operationally.** Q7 and Q17 both left this open and this evaluation does not close it. What it adds is that the two hosted surfaces — a server and a probe harness — are the only places in this specification where a commercial tier could sit, and that the closest observed analog draws its line there.

**Whether any adopter ever pays for a campaign.** Q8's open question is a business-model question, and Q11's recommendation does not answer it. It only says that the answer does not belong in the engine's license.

**Whether the four documentation modes are the right kind set for a site bundle.** Q3's bundle set is a guess about how adopters cluster, and a documentation-site bundle would be another. Data in a package, revisable at the cost of a release.

**Three claims are unmeasured**, as [principle 11](../spec/00-vision-and-scope.md#design-principles) requires.

- An adoption payload should shrink. The instrument is its remaining `(document, rule)` count over time, per adopter, and the fraction of payloads that reach zero before the expiry. This repository's own baseline fell from 65 entries to 14, and the section above states why that is not the same measurement.
- A permissive license should remove a step before a trial. The instrument is a count of adopters who report the terms as the reason they stopped, and it needs a public channel that does not exist yet.
- A generated site should let a reader answer "is this for me?" without reading the specification. The Discovery probe category is the instrument for the machine half, and the human half has none, which is worth saying rather than hiding.

## Consequences for the specification

| # | Finding | Landed in |
|---|---|---|
| 1 | Q11 is pending ratification, and the field is narrowed rather than open. Internal-only and source-available are refused by rulings already made; strong copyleft is refused for the library | [Q11](../spec/09-open-questions.md#q11--license-and-distribution-posture) |
| 2 | The base package and the bundles may impose nothing on a derived taxonomy, because an overlay resolution contains base content and spec 0 promises the taxonomy is the adopter's | [Spec 7](../spec/07-distribution-and-federation.md#publishing) and [Q11](../spec/09-open-questions.md#q11--license-and-distribution-posture) |
| 3 | The from-version of a migration state is optional, and adoption is a migration from no taxonomy | [Spec 7](../spec/07-distribution-and-federation.md#first-contact-adoption-is-a-migration-from-no-taxonomy) |
| 4 | `infer` emits three artifacts: the overlay, the misfit report, and the adoption payload | [Spec 7](../spec/07-distribution-and-federation.md#the-interview) |
| 5 | An adoption payload declares no threshold past which a rule is switched off, and every run reports the remaining pair count | [Spec 7](../spec/07-distribution-and-federation.md#first-contact-adoption-is-a-migration-from-no-taxonomy) and [spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for) |
| 6 | `--since <ref>` as a gate is refused. No flag decides which findings count, and `--changed-only` is a performance scope | [Spec 6](../spec/06-engine-architecture.md#cli) and [Q12](../spec/09-open-questions.md#q12--migration-path-for-an-existing-corpus) |
| 7 | Registration is publication into a channel with an obliged reader, and the package channel and the served page are the two that exist | [Spec 7](../spec/07-distribution-and-federation.md#arriving-at-a-corpus-cold) |
| 8 | A registry or directory of Headwater corpora is refused, not deferred | [Spec 0](../spec/00-vision-and-scope.md#what-we-do-not-build) |
| 9 | The public site is a projection of this corpus, rendered by a third-party generator, and every published number comes from the evidence register | [Spec 4](../spec/04-assurance-model.md#the-systems-own-assurance) |
| 10 | The prior art for terms, on-ramps and being found | [HW-EVAL-adjacent-work §S](../evaluations/adjacent-work.md#s--terms-on-ramps-and-being-found) and its summary table |
| 11 | The register is no longer a list of deferrals, and what remains live across it is stated once rather than reconstructed | [Spec 9](../spec/09-open-questions.md) |
| 12 | The repository has no stated license, and a reader should learn that from the README rather than from its absence | [README](../../README.md) |

All twelve are applied.
