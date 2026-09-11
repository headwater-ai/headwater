---
id: HW-EVAL-n8n-worked-example
status: current
status_since: 2026-09-01
last_verified: 2026-09-01
summary: Forty-six real governing documents from n8n, typed against three entries of the library, and what three runs and one coherence sweep found in a corpus that keeps its prose beside the code.
title: "n8n as a worked instance — a corpus with no docs root"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# n8n as a worked instance — a corpus with no docs root

Evidence for [criterion 4](../taxonomies/README.md#admission-criteria) of the canonical taxonomy library, which asks each entry to carry a real or realistic corpus that the entry types. The [design-spec entry](../taxonomies/design-spec/doctrine.md) carried one realistic corpus for an invented system. This is its first real one.

It is also the first corpus in this library with no collected documentation root. [#488](https://github.com/headwater-ai/headwater/issues/488), [#489](https://github.com/headwater-ai/headwater/issues/489) and [#491](https://github.com/headwater-ai/headwater/issues/491) each propose a project that keeps its governed writing under one tree. n8n keeps it beside the code it governs. One document per package, across a monorepo of 27,688 files. [#492](https://github.com/headwater-ai/headwater/issues/492) asks whether a taxonomy shaped around one collected root still resolves against that shape.

**The short answer is that it does not, and the correction it needs is one declaration.** The longer answer is below, with the run that produced it.

This document reports two of the three kinds that [#492](https://github.com/headwater-ai/headwater/issues/492) names. The architecture documents are [#492](https://github.com/headwater-ai/headwater/issues/492) itself and the review rules are [#508](https://github.com/headwater-ai/headwater/issues/508), and the two halves are marked below. The third is [#509](https://github.com/headwater-ai/headwater/issues/509), and it is reported here too. The reason for the original split was that the third had no admitted kind to be typed against. The `diataxis-site` entry supplied one, and the skills corpus is typed below.

**The first half of this document reads `packages/`, and [the second half](#the-review-rules-a-second-corpus-of-the-same-repository) reads `.agents/review-rules/`.** They are two corpora at one pin, because `corpus.root` is a single scalar and one root cannot reach both trees.

## The pin, and the fork

| | |
|---|---|
| Fork | https://github.com/headwater-ai/n8n, created 2026-09-01, default branch only |
| Upstream | https://github.com/n8n-io/n8n |
| Branch | `master` |
| Commit | `b0550cb3cb4d1752546a69056c55eccfb9111a12`, committed 2026-09-01T16:27:59Z |

n8n's [Sustainable Use License](../taxonomies/design-spec/fixtures/n8n/LICENSE.md) states that content of branches other than `master` is not licensed. Every citation here names `master` and that commit.

The fork carries no commit of this project. It is a mirror, and its one job is to keep the pin readable after the upstream moves.

**The pin is not ordinary caution.** Between a survey on 2026-08-31 and an adjudication on 2026-09-01, n8n's `.agents/review-rules/` corpus gained a whole category of five governing documents. A report over a repository this active goes stale while it is written, and a pinned one does not.

## What was typed, and as what

Four documents, all typed as `kinds.design_spec` of the design-spec entry:

- `packages/@n8n/instance-ai/docs/architecture.md`
- `packages/@n8n/expression-runtime/ARCHITECTURE.md`
- `packages/@n8n/instance-ai/evaluations/ARCHITECTURE.md`
- `packages/@n8n/local-gateway/docs/ARCHITECTURE_CONNECTION_VS_SETTINGS.md`

The kind is right on its two declared grounds. The purpose of `design_spec` is `behavior`, which is what an architecture document serves. The tradition the entry models is the software design document, which is what each of these is.

**Typing means one front-matter block on a copy, and nothing else.** No word, no heading, no link, no spelling, no contraction and no line break of any body was changed. The [fixture directory](../taxonomies/design-spec/fixtures/n8n/) holds the copies, a verbatim copy of n8n's license, and a modification notice that the license requires. Byte identity was verified mechanically before this document was written. Each copy had its front-matter block stripped and the remainder diffed against `git show <pin>:<path>`. All four diffs were empty.

**None of the four is under n8n's Enterprise License.** That license covers a file with `.ee.` in its name or a directory with `.ee` in its name. The tree at the pin holds 196 paths of the first form and 1,064 of the second. None is under any of the three packages this evaluation read.

### The `sequence` friction, which is a finding

`kinds.design_spec` requires the facets `doc_type` and `sequence`. `sequence` is an integer, and the entry's shelf layout is `{sequence:02d}-{slug}.md`. It is the numbering of a specification series, where RFC 2119 comes after RFC 2118.

**n8n's architecture documents carry no sequence number, because a package is not a position in a series.** There is no number to write. No value was invented to make the run pass, and the run therefore reports `facet.required.missing` on all four documents.

The entry assumed that a design specification belongs to a numbered series. A monorepo that keeps one architecture document per package is a corpus where the assumption is simply false. That is the finding, and criterion 4 exists to produce it.

## What the run reported

The [fixture README](../taxonomies/design-spec/fixtures/n8n/README.md#what-a-run-reports) carries the run in full, with the assembly that reproduces it. The summary:

**42 files under the corpus root, 4 typed, 38 excluded, 4 checked, 46 check instances, 13 findings, all 13 of them errors.** `headwater check --strict` exits 1. The 38 excluded files are the vendored taxonomy package.

Three findings per document, and the same three on each one.

| Document | Rule | Severity |
|---|---|---|
| all four | `facet.required.missing` (`OB-FACET-1`), `sequence` not declared | error |
| all four | `facet.required.missing` (`OB-FACET-1`), `title` not declared | error |
| all four | `identifier.unusable` (`OB-ID-2`), a typed `design_spec` that declares no identifier | error |

The run reported 8 findings until headwater/standard 4.0.0, which requires `title` on `design_spec`. None of these four upstream files declares one. This repository will not write four titles into somebody else's documents to make the number go down.

`identifier.unusable` is a third finding about the entry rather than about n8n. The design-spec entry declares five kinds and no identifier scheme for any of them. This repository mints `spec_id` in its own overlay, and an overlay is not an admitted library entry. So a corpus that takes the entry alone gets documents that can stand at neither end of a relation. The [diataxis fixture](../taxonomies/diataxis/fixtures/README.md) reported the same rule six times for the same reason.

### Not one finding is about n8n's writing

The base package declares `language: default: {tag: en-US, controlled: none}`. So no admitted entry of this library holds prose to a controlled language, a source form, a sentence length or a retired term. `voice.forbidden_construction` ran four instances and reported nothing. `section.required.missing` never instantiated, because `design_spec` declares no required sections.

**The canonical library, pointed at four real governing documents, says nothing at all about the prose in them.** That is worth stating before any count, because a large finding total over somebody else's corpus is easy to mistake for a result.

### The broken link that no rule reported until 4.3.0

`packages/@n8n/expression-runtime/ARCHITECTURE.md:426` writes the link `[n8n workflow package]` with the target `../workflow/`. From that directory the target is `packages/@n8n/workflow/`, and the tree at the pin holds no such path. The package it means is `packages/workflow/`, which holds 244 files. This was checked against the whole tree at the pin and not against the four-file slice.

`headwater check` sees it and reports it in the graph section as `1 prose links that did not resolve`, with the file, the line and the reason. **Until headwater/standard 4.3.0 no rule turned it into a finding.** From 4.3.0 `link.path.unresolved` reports it as an error under `OB-LINK-2`. The near rule is `link.fragment.unresolved`, which reads a fragment inside a document rather than a path. It ran four instances here and reported nothing, then and now.

So the one defect here that a reader would call a defect was a fact and not a finding at 4.0.0. It is now both. That move is the single change in this document's result. Everything the evaluation concluded from the silence is a claim about 4.0.0 rather than about the library today. Neither of n8n's own two tools reports it.

## The scattered-corpus question

This is the third Done-when bullet of [#492](https://github.com/headwater-ai/headwater/issues/492), and it is the reason the issue exists.

**First, a correction to how the issue states it.** n8n does have a root `docs/` directory, and it holds 272 files at the pin. 271 of them sit under `docs/generated/` and are `tbls` output produced from the database migrations. The 272nd is `docs/db.md`. It is an authored page, and its subject is how that output is generated and where to read it. No architecture document, no review rule and no skill is under `docs/`. The issue's claim is right in substance and wrong as literally written. The accurate statement is narrower. n8n has a `docs/` directory that carries generated output and one index page for it. Its governed prose lives elsewhere, scattered.

**The design-spec entry as it ships types none of these documents.** Its one shelf is `spec_series` at `docs/spec/**`, and not one file of this corpus is under `docs/`. Run the same assembly with that shelf alone and the run reports **42 files, 4 untyped, 38 excluded, 0 checked, 6 check instances, 1 finding**. `headwater check --strict` exits **1**, and the one finding is a broken prose link rather than anything about the four documents going unread.

A green strict run over four real governing documents that no rule read is what the census crate calls *systematically green*. The engine is honest about it in the census, which carries four rows saying `no shelf pattern claims this path`. Nothing else says the corpus went unchecked.

### What shape the shelf declaration had to take

Three properties, and each one was measured rather than assumed.

**The corpus root is `packages` and not `docs`.** That forces an exclusion, because a taxonomy package is found under `packages/` and n8n's prose is under `packages/` too. A corpus root that reaches the second reaches the first. The three earlier fixtures of this library never met this, because each of them roots its corpus at `docs`.

**The path pattern has to be `packages/**`, which fixes one segment out of 27,688 files.** The four documents share no directory. What they share is a filename, and the pattern language reads a path rather than a name. `packages/**/ARCHITECTURE.md` is legal and it claims **2 of the 4**, because the other two are named `architecture.md` and `ARCHITECTURE_CONNECTION_VS_SETTINGS.md`. That run reports 2 typed, 2 untyped, 22 check instances and 6 findings, and the two documents it misses report nothing at all.

**The shelf has to be heterogeneous with one admitted kind, and that reads as a contradiction.** `kinds.design_spec` requires `doc_type`, because the entry's own shelf is heterogeneous and needs a discriminator. A homogeneous shelf refuses a document that restates the kind its placement already states. So a `design_spec` on a homogeneous shelf reports an error whichever way the front matter is written:

| The shelf, and the front matter | Findings |
|---|---|
| `homogeneous: true`, with `doc_type: design_spec` | 16, adding `shelf.placement_is_primary` on all four |
| `homogeneous: true`, with `doc_type` removed | 16, adding `facet.required.missing` for `doc_type` on all four |
| `homogeneous: false`, `discriminator: doc_type`, `kinds: [design_spec]` | 12 |

The third row is what the fixture commits. A one-kind heterogeneous shelf makes the discriminator carry no information at all. Every document writes the one value the shelf admits, and the key exists to satisfy a facet requirement rather than to tell two kinds apart.

**The answer to the bullet.** A shelf declared as a path pattern does resolve against a scattered corpus, and it costs three things that no collected corpus pays. An exclusion the collected shape never needed. A pattern so broad that it selects a whole source tree. A heterogeneous shelf with one kind on it. None of the three is a defect of the engine. All three are the shelf model meeting a corpus whose placement carries no information about kind.

## The count

This is the fifth Done-when bullet, scoped to this one kind. It exists so that nobody has to take on faith that a finding is worth anything. A finding that a project's own tooling already catches is not evidence that Headwater helps.

### What `prettier` catches: zero, and not for the reason the issue gives

The issue states that `lefthook.yml` runs `prettier --write` on every staged `.md` file, so Markdown formatting is already enforced. **The first half is true and the conclusion is false.**

`lefthook.yml` does carry a `prettier_check` command with the glob `packages/**/*.{vue,yml,md,css,scss}`, which matches all four of these paths. But `.prettierignore` at the repository root carries the line `**/*.md`, under the comment `# Ignored for now`. Prettier honors its ignore file even for a path named on the command line, so the hook formats no Markdown at all.

That was verified rather than reasoned. A scratch directory with a `.prettierignore` of `**/*.md` and one badly formatted Markdown file: `prettier --check` reports `All matched files use Prettier code style` and exits 0. Remove the ignore file and change nothing else, and the same command reports a style issue on the same file.

**So prettier catches 0 of the 13 findings, and 0 of anything else in these four files.**

The counterfactual is worth one line, because it bounds the overlap. Run prettier over the four files with the ignore lifted and it rewrites three of them. Every change is blank lines around fenced blocks, table column alignment, and indentation inside a fenced block. None of it touches a heading, a facet, a link, a spelling or a sentence. **Even with the ignore lifted the overlap would be zero.**

### What `cubic` catches: zero, and each rule was read

`cubic.yaml` declares five `custom_rules`, and its own trailing comment says all five slots are used. Each one is scoped by an `include` list, and each list was read against these four paths.

| Rule | `include` patterns | Matches any of the four |
|---|---|---|
| Security | `packages/cli/**`, `packages/@n8n/db/**`, `packages/core/**`, `packages/workflow/**`, `packages/nodes-base/**`, `packages/@n8n/nodes-langchain/**` | no |
| Backend | the same six | no |
| DB migrations | `packages/@n8n/db/src/migrations/**`, `packages/cli/test/migration/**` | no |
| Frontend | `packages/frontend/**` | no |
| QA & DX | `.github/**`, `docker/**`, `scripts/**`, `patches/**`, `packages/testing/**`, four `packages/@n8n/*-config/**` entries, and eight file-level patterns | no |

Not one pattern reaches `packages/@n8n/instance-ai/`, `packages/@n8n/expression-runtime/` or `packages/@n8n/local-gateway/`. Two further facts point the same way. Cubic reviews only the lines a pull request adds or modifies, and its `custom_instructions` block states the bar in terms of code rather than prose.

**So cubic catches 0 of the 12.**

### The count itself

| | Under the entry alone | Under the entry plus this repository's house regime |
|---|---|---|
| Findings raised | 12 | 166 |
| Already caught by `prettier` | 0 | 0 |
| Already caught by `cubic` | 0 | 0 |
| Caught by neither | 12 | 166 |
| Of those, written by `headwater check --fix` | 0 | 1 |
| Of those, needing a human rewrite or a declaration change | 12 | 165 |

The second column is a probe and not a declaration, on the precedent the [brd-prd fixtures](../taxonomies/brd-prd/fixtures/README.md#what-the-bases-voice-regime-would-have-reported) set. It binds `regimes.language.ste_house` from this repository's own overlay onto `kinds.design_spec`, changes nothing else, and touches no document. The [fixture README](../taxonomies/design-spec/fixtures/n8n/README.md#what-this-repositorys-own-house-regime-would-have-reported) carries it per rule and per document.

**Three things about that second column matter more than its total.**

136 of the 154 extra findings are hard wraps, and they come from two documents rather than four. One carries 108 and one carries 27. The other two carry one between them, because they are already written one line per paragraph. A house rule that reads as a verdict on a project is a verdict on one editor setting.

**There is no British spelling anywhere in the four documents.** The rule that catches one ran on every document and found nothing. That contradicts what this work expected to find, and it is worth recording as a correction rather than quietly dropping.

**Exactly one finding of the 166 is mechanically fixable.** `headwater check --fix` writes a British spelling, a contraction whose expansion is one word, a retired term that names a replacement, and a missing reciprocal link. One contraction, `doesn't`, is the whole of the intersection, and the report marks it as the only `fix (mechanical)` line. The 136 hard wraps are mechanical to a reader and carry no patch, and `engine/crates/check/src/source_form.rs` states why in its own comment. `headwater check --fix` was never run over this corpus.

**What the count says.** Under the canonical library alone the honest total is 12. Every one of the 12 is about a declaration rather than about n8n, and every one is invisible to both of n8n's tools. Under a house language regime the total is 166, of which one is mechanically fixable and 136 are one editor's line-wrapping habit. Neither number is a claim that Headwater found 12 or 166 problems in n8n. The 12 are what a taxonomy learns about itself by meeting a corpus it did not author.

## The review rules, a second corpus of the same repository

[#508](https://github.com/headwater-ai/headwater/issues/508) types the second of the three kinds that #492 names. It reads the rules that n8n's AI code reviewer loads on every pull request. It also asks a question that no check answers: whether the three-level model n8n publishes about those rules holds.

The fork is the same fork and the pin is the same commit. The corpus is not the same corpus. `corpus.root` is a single scalar string. n8n keeps its architecture prose under `packages/` and its review rules under `.agents/`, and one root cannot reach both. So the work built a second fixture beside the first, under the entry it evidences: [`docs/taxonomies/standards-spec/fixtures/n8n/`](../taxonomies/standards-spec/fixtures/n8n/README.md).

**This is where the headline claim of #492 narrows usefully.** n8n's architecture documents are scattered, one per package, across a source tree of 27,688 files. Its review rules are collected: 23 files under one directory, six subdirectories deep at most. **One repository is a scattered corpus for one of its kinds and a collected corpus for another.** The shelf model met both. The first cost a broad pattern and a forced exclusion. The second cost one ordinary path pattern.

### What was typed, and as what

Seven documents, all typed as `kinds.standard` of the [standards-spec entry](../taxonomies/standards-spec/doctrine.md). Six are rule files across five of the six directories. The seventh is the `README.md` that files them, and it is here because a sweep finding may only name a typed document.

**The kind is right on its declared ground, and the granularity mismatch is the finding.** The purpose of `standard` is `constraint`, which reads "state what must hold across the documents and code it governs, and what it rules out". A rule file decides what a reviewer comments on, which is what a constraint is. #492 guessed that this kind was requirement shaped. That guess points at the `brd-prd` entry, which declares `kinds.brd` and `kinds.prd` and nothing granular. A granular `kinds.requirement` exists in this repository's own overlay and nowhere else. An overlay is not an admitted library entry, so it cannot satisfy criterion 4. **No admitted entry of this library declares a kind for one rule.** `kinds.standard` is the closest admitted target. Every one of the 21 findings below comes from the distance between a whole standard and one rule of one.

**Typing means one front-matter block on a copy, and nothing else.** Byte identity was verified twice. Each copy had its front-matter block stripped and the remainder diffed against the file at the pin, which gave seven empty diffs. Each file at the pin was then compared by `git hash-object` against the blob the fork reports at that commit, which gave seven matches.

The [fixture directory](../taxonomies/standards-spec/fixtures/n8n/) carries its own verbatim copy of n8n's license and its own modification notice. Neither is inherited from the design-spec fixture. A second set of copied files is a second receipt of part of the software. A modification notice is a claim about which files were modified, and these are different files. **None of the seven is under n8n's Enterprise License.** The tree at the pin holds 91 paths under `.agents/`, and zero of them match `.ee` in any form. The check was run rather than assumed.

### What the run reported

The [fixture README](../taxonomies/standards-spec/fixtures/n8n/README.md#what-a-run-reports) carries the run in full. The summary:

**7 files under the corpus root, 7 typed, 0 excluded, 7 checked, 97 check instances, 21 findings, all 21 of them errors.** `headwater check --strict` exits 1. The graph reads 7 nodes and 0 declared edge halves.

Three findings per document, and the same three on every one. `section.required.missing`, an error, under `OB-SECT-1`, once each for `Scope`, `Requirements` and `Conformance`.

**This is the first time in the n8n work that the rule instantiated at all.** The design-spec run recorded that it never did, because `design_spec` declares no required sections. Here it produces the whole run.

**The finding is sharper than a heading count.** Not one heading at any level of any of the 23 upstream files carries the word `Scope`, `Requirements` or `Conformance`. The corpus is not missing the content. Every one of the 22 rule files opens with a line `Applies to: …`, which is the scope written as a labeled paragraph. 16 of them use the imperative `Flag` somewhere to state what to flag, and only six write a literal `Flag:` block. **The required section is present in substance in the whole corpus and absent in form in the whole of it.** The contract is lexical over headings, and the tradition writes a labeled paragraph. `Conformance` is the honest third. No rule file says how conformance is judged, because the AI reviewer judges it, so that section is genuinely absent.

**One expectation was checked and is not due.** `kinds.standard` expects a `regulates` edge to a `functional_spec` within 180 days of `state_entered`, at severity `warn`. `relation.participation.overdue` instantiated seven times and reported nothing, because `status_since` and the injected clock are both the pin date. The window opens on 2026-09-01 and its deadline falls on 2027-02-28. The rule fires only once the clock passes that date, so the seven warnings appear from 2027-03-01. That is a fact about the clock and not about n8n.

### Whether the three-level reach model holds

This is the fourth Done-when bullet of #508, and it is a `headwater sweep` job rather than a check. **It is the first coherence sweep run and recorded anywhere in this repository.** There was no committed briefing, no return file and no report before this one. All three artifacts are committed under [`sweep/`](../taxonomies/standards-spec/fixtures/n8n/sweep/) beside the corpus. `sweep plan` is byte-reproducible, and the return file is the only thing that lets a later reader reach the same verdict.

`headwater sweep report` confirmed four things and no more. The return file names the lock this tree carries. Every path it names is a typed row of the census. Every quotation is really in the document it is attributed to. No proposed edge restates one the graph already declares. **3 findings carried, 0 refused, of 3 the file held.** It confirms nothing about whether a reading is right.

**The answer is that the model holds for the two cases the corpus declares, and a third case broke it that nobody noticed.**

**The two declared exceptions are exceptions.** `testing/coverage.md` is one file listed in two agents' `file_paths`, Backend and Frontend, exactly as the README declares. Security and QA & DX not linking it is stated with a reason in the same paragraph. Neither is a contradiction and the sweep names neither.

**The third case is a genuine defect.** The README names two agents that deliberately do not link `testing/`. Read against `cubic.yaml`, three do not. The DB migrations agent's `file_paths` holds only its own five files. The README's own Layout table is already consistent with three and maps `testing/` to Backend and Frontend. So the table was updated when the `db-migrations/` category arrived and the prose two paragraphs below it was not. The reason that prose gives, coverage nagging on a credential fix or a Dockerfile, does not describe a migration. **The defect is causally tied to the drift the pin exists to capture.**

**A second finding is a real reading and a contestable one.** The README rules that a policy identical across domains is one shared file and never a copy per directory. `testing/coverage.md` holds the shared coverage policy. `db-migrations/conventions-and-tests.md` holds a second coverage policy under its own `## Tests` heading. A defender would call this specialization rather than duplication. The README's own test is "word-for-word the same", and a migration test is not word-for-word a service-method test. Both readings are honest. The sampler adjudicates neither, and that is what a sampler is for.

**A third finding is about the corpus's organizing term.** The README opens its instructions to a rule author with "Pick the level of reach first." The term is defined nowhere in the slice. Its only definition is a header comment in `cubic.yaml`.

### What the sweep could not reach, which is three results

**`cubic.yaml` can never be a finding path.** A sweep finding may only name a typed row of the census, and `cubic.yaml` is a configuration file that no taxonomy here types. **The document that asserts the three-level model cannot appear in the sweep that checks it.** Every quotation from it lives in a message, where nothing verifies it. The sampler can confirm a contradiction between two governed documents and cannot reach the configuration file that governs them.

**`conflicts_with` cannot join two standards, and the intake carried the proposal anyway.** The base package declares it `from: [decision] to: [decision]`. The second finding proposes an edge between two standards on purpose, to find out. The intake's four confirmations do not include endpoint-kind validation, so the proposal passed and the report printed the front matter to write. Writing that front matter and re-running `headwater check` reports **2 `relation.endpoint.not_permitted` errors**, one for each end. **A sweep proposes an edge that the gate then refuses, and nothing between the two says so.**

**The front matter the report prints names the wrong document.** The proposal names `N8N-STD-coverage` as its source. The line beneath it points the reader at the first path in the finding's document list, which is the README. A reader who follows the instruction declares an edge from `N8N-STD-review-rules`, which is not the edge the sweep proposed. Both arms were run and both report the same 23 findings, so the gate catches the substitution for the wrong reason and never names it.

**One result belongs to the register rather than to the corpus.** The standards-spec entry declares `obligations.OB-SS-1` with `class: coherence`. [Spec 4](../spec/04-assurance-model.md) and [HW-OBL-0114](../obligations/0114-a-control-that-names-a-sweep-marks-its-obligation-verified-with-nothing-run.md) both record that this repository's register has no coherence-class obligation, so a sweep here discharges nothing. This corpus gives that class its first member anywhere in the library. It still discharges nothing, because OB-SS-1's disposition is `unverifiable` and no control names a `sweep:` mechanism. The gap moves from hypothetical to visible.

### The count, extended to a third tool

**`prettier` catches 0, and this slice has two independent reasons where the architecture documents had one.** `.prettierignore` carries `**/*.md` at the repository root, which is the reason already recorded above. And `lefthook.yml`'s prettier glob is `packages/**/*.{vue,yml,md,css,scss}`, which never reaches `.agents/` at all. Either one alone gives zero.

**`cubic` catches 0, and the reason is worth one sentence on its own.** Not one `include` pattern of the five `custom_rules` reaches `.agents/`. The five lists cover `packages/`, `.github/`, `docker/`, `scripts/`, `patches/` and eight file-level entries. **cubic reviews its own rule files under no rule at all.** `lefthook.yml` carries a `.agents/skills/**` job and no `.agents/review-rules/**` job, which points the same way.

**A third tool reads exactly these files, and it catches 0 as well.** `pnpm check:cubic-config` runs `.github/scripts/quality/check-cubic-config.mjs`, and CI runs it on every pull request. It validates `cubic.yaml` against a vendored copy of cubic's schema. It then fails on a missing linked path, an agent over the 10,000-character ceiling, or a rule file that no agent links. Its constants name `.agents/review-rules` directly. **It reads no prose.** It counts characters and resolves paths, so the overlap with 21 findings about headings is zero. The zero is measured rather than assumed.

**One detail of that third tool is a result in itself.** Its file list excludes `README.md` by name, because the README is not a rule and no agent links it. **The one document the sweep found three findings in is the one document n8n's own checker declines to open.**

| | Under the entry alone | Under the entry plus this repository's house regime |
|---|---|---|
| Findings raised | 21 | 143 |
| Already caught by `prettier` | 0 | 0 |
| Already caught by `cubic` | 0 | 0 |
| Already caught by `check:cubic-config` | 0 | 0 |
| Caught by none of the three | 21 | 143 |
| Of those, written by `headwater check --fix` | 0 | 6 |
| Of those, needing a human rewrite or a declaration change | 21 | 137 |

**Two things in that second column correct what the design-spec run recorded.**

**The hard wraps are spread across all seven documents, and there they were not.** 87 of the 122 extra findings are `language.source_form.not_met`, and every one of the seven documents carries between 2 and 42. In the architecture corpus 136 of 136 came from two documents of four, and the other two were already written one line per paragraph. Same repository, same pin, one editorial convention per corpus.

**The British-spelling claim was wrong in both directions.** The design-spec run recorded that there is no British spelling anywhere in its four documents. There are four in this corpus: `behaviour`, `defence`, `colours` and `denormalised`. Three of them are in the typed set. **The rule reports one.** The engine's spelling table is closed at 24 words and holds `behaviour` and not `defence`, `colour` or `denormalise`. So a corpus-level count and a rule-level count differ here by a factor of three, and only the second is what a run measures.

**Six findings of 143 are mechanically fixable, against one of 166 before.** Five contractions and one spelling are the whole intersection, and the report marks each with `fix (mechanical)`. The 87 hard wraps are mechanical to a reader and carry no patch. `headwater check --fix` was never run over this corpus.

## What this adds to criterion 4

This is the seventh Done-when bullet of #492 and the sixth of #508. Three entries of the library now carry this repository. The design-spec entry carries the invented Beacon corpus and n8n's architecture documents, and its [doctrine](../taxonomies/design-spec/doctrine.md#worked-instances) cites both. The standards-spec entry carries the invented Beacon ladder and n8n's review rules, and its [doctrine](../taxonomies/standards-spec/doctrine.md#worked-instances) gained a `Worked instances` section that cites both. The diataxis-site entry carries the invented Beacon-style shape fixtures and n8n's skills, and its [doctrine](../taxonomies/diataxis-site/doctrine.md#worked-instances) gained the same section.

**One note, which is not fixed here.** This document is a `kinds.evaluation`, and `evaluation` moved out of design-spec into the `evidence-and-obligation` entry with HW-DR-0044. That entry is not in the admitted-entry table of `docs/taxonomies/README.md`, which still lists five rows while `docs/taxonomies/` holds six directories. Two admitted entries now declare `requires: [evidence-and-obligation]`. That gap is [#510](https://github.com/headwater-ai/headwater/issues/510) and is not touched here.

## What `.agents/skills/` states, and what the tree holds

The third corpus that [#492](https://github.com/headwater-ai/headwater/issues/492) names is `.agents/skills/`, and this section reports one finding in it without typing any of it. The next section types the corpus. The finding below needs no kind either way, because it is a claim the corpus makes about itself.

At the pin, `.agents/skills/spec-driven-development/SKILL.md:8` states this:

> Specs live in `.agents/specs/`. They are the source of truth for architectural decisions, API contracts, and implementation scope.

The tree at the pin holds no such path. `.agents/` holds two entries, `review-rules` and `skills`, and 58 files below them. The count of paths under `.agents/specs/` is zero. Five lines name that path and all five are in this one file: the front-matter `description`, and lines 8, 20, 23 and 66. No other file under `.agents/` names it.

**The directory is absent, and it is not ignored.** `.gitignore` at the pin runs to 96 lines and names `.agents/specs/` on none of them. Line 84 excludes `.claude/specs/`, which is a different path under a different directory. A reader who expects an ignore rule to explain the absence will not find one.

**The skill anticipates a missing spec for one feature, not a missing root.** Step 3 of *Before Starting Work* reads "If no spec exists and the task is non-trivial (new module, new API, architectural change), ask the user whether to create one first." That step handles a feature with no spec. It does not handle a root that holds no specs at all, which is what this tree has. Step 1 gives a literal command, `ls .agents/specs/`, and at this pin that command fails for every feature and not for some.

### What this taxonomy would report, and what it does not

Two relations in the resolved taxonomy have the shape this needs. `governs` runs from a `governed_document` to a `code_path`, and `traces_to` runs from a `governed_document` to a `governed_document` or a `code_path`. Either one could carry the claim that a stated practice makes, and a check could then ask whether the target exists. Neither relation is required on any kind, and no rule asks a document to declare one.

So this taxonomy has the vocabulary for the claim and no obligation to state it. This document already records the near case. The unresolved link at `packages/@n8n/expression-runtime/ARCHITECTURE.md:426` reaches `headwater check` as [a fact in the graph section, and from 4.3.0 as a finding too](#the-broken-link-that-no-rule-reported-until-430). The `.agents/specs/` claim does not reach the engine at all, and typing the corpus does not change that. The run in the next section types every file of `.agents/skills/` and reports 139 findings, and not one of them is this claim. A stated practice that no artifact backs is invisible to a check whether or not the document carrying it has a kind.

Whether a rule should report a stated practice that no artifact backs is [#496](https://github.com/headwater-ai/headwater/issues/496), and this section decides nothing about it.

### What was enumerated here, and what was not

Every number below came from the tree at the pin, measured on 2026-09-08. The `last_verified` date in the front matter covers the rest of this document and not this section.

| fact | number | of what |
|---|---|---|
| entries directly under `.agents/` | 2, and neither is `specs` | the tracked tree at the pin |
| blobs under `.agents/` | 58 | 35 under `skills`, 23 under `review-rules` |
| skill directories under `.agents/skills/` | 22 | 34 files inside them, plus a top-level `AGENTS.md` |
| lines that name `.agents/specs/` | 5, in 1 file | of the 58 files |
| lines that claim a source of truth | 15, in 10 files | of the 58 files |
| those lines that name a repository-root path | 5, in 4 files | of the 15 lines |
| those root paths that the tree does not hold | 1, `.agents/specs/` | of the 5 paths |

The source-of-truth count ignores letter case. A case-sensitive match on the lower-case phrase gives 13 lines in 9 files, because `db-migrations/SKILL.md` opens two of them with a capital.

The last two rows were enumerated and not sampled. Four files name a repository-root path as a source of truth. One of them is `spec-driven-development/SKILL.md`, and the other three each resolve at the pin:

- `db-migrations/SKILL.md` names `packages/@n8n/db/src/migrations/migration-types.ts` and `packages/@n8n/db/src/migrations/dsl/column.ts`.
- `community-pr-readiness-check/reference/checks.md` names `.github/pull_request_template.md`.
- `public-api/SKILL.md` names `packages/@n8n/decorators/src/controller/`, which holds 35 files.

The other six of the ten files point at a symbol, a service, or a file inside a package. None of those six was resolved, and this section claims nothing about them.

To take the finding again, run this against the pin:

```bash
gh api "repos/headwater-ai/n8n/git/trees/b0550cb3cb4d1752546a69056c55eccfb9111a12?recursive=1" \
  --jq '.tree[].path' | grep -c '^\.agents/specs'
```

It prints `0`. Any other number makes this section false.

## The skills corpus, typed

The section above reports one finding in `.agents/skills/` and types none of it. This section types all of it. The premise both #492 and #509 rest on is dead: the [`diataxis-site` entry](../taxonomies/diataxis-site/doctrine.md) landed in `46b86799` and declares `kinds.how_to`, a concrete kind under `purposes.procedure`. That kind is what the third corpus waited for. The vendored slice is [`docs/taxonomies/diataxis-site/fixtures/n8n/`](../taxonomies/diataxis-site/fixtures/n8n/README.md), at the pin the other two corpora use, and its README carries the assembly commands that produced every number below.

**What the pin holds, re-measured.** `.agents/skills/` holds 22 skill directories, 34 files inside them, and one top-level `AGENTS.md` of 1,952 bytes. That is 35 files and 425,316 bytes of upstream body. #492 and #509 each write "22 skill directories and 35 files". Neither one names `AGENTS.md`, so a reader of either issue puts all 35 files inside the 22 directories. The files that are not a `SKILL.md` number 12 and not the ten that #509 enumerates.

**Every file is typed, and that is the result this corpus was vendored for.** One homogeneous shelf at `.agents/skills/**` carries `how_to`. The run reports **35 files under the corpus root, 35 typed, 0 excluded, 35 checked, 426 check instances, 139 findings**. The census reads 35 `how_to`. The graph reads 35 nodes and 0 declared edge halves. Nothing is untyped, and `headwater check --strict` exits 1.

| Rule | Count | Severity |
|---|---|---|
| `section.required.missing` | 104 | error |
| `link.path.unresolved` | 14 | error |
| `voice.forbidden_construction` | 21 | warn |

**104 of a possible 105, and the missing one is the interesting number.** `kinds.how_to` requires `Goal`, `Steps` and `Verification`. Thirty-five documents times three headings is 105. The run reports `Goal` 35 times, `Steps` 34 times and `Verification` 35 times. One file writes a `Steps` heading by name, `.agents/skills/create-pr/SKILL.md`. That is the only heading of the three the corpus supplies anywhere. The review-rules corpus reported this shape from a different kind. A section contract reads headings, and the tradition writes the same content as an imperative paragraph.

**Thirteen of the 14 link findings are a property of the corpus root and not of n8n.** `corpus.root` is `.agents`, and 13 of the targets are relative paths that climb above it, such as `../../../packages/@n8n/instance-ai/evaluations/README.md` and `../../../AGENTS.md`. Each of the 13 was checked against the pin through the contents API and **all 13 exist upstream**. A run whose root reached the whole repository would resolve every one of them.

**The fourteenth is a real dangling link, and this time a rule reports it.** `.agents/skills/design-system/rules/web-animation-guidelines.md:84` links `PRACTICAL-TIPS.md`, and the tree at the pin holds no such path under `.agents/skills/design-system/rules/`. This is the shape of the `.agents/specs/` claim above, written as a link rather than as a sentence. The difference is what a rule can reach. A link resolves or it does not, and a stated practice reaches no rule at all.

**The symlink tree, measured.** `.claude/plugins/n8n/skills/` holds 24 entries at the pin. Twenty-two are symlinks at mode `120000`. The set of 22 link names equals the set of 22 shared directory names, with no difference in either direction. The other two entries are a real directory, `setup-mcps/`, and the `SKILL.md` inside it. A corpus rooted at `.agents` sees none of this, because a symlink tree outside the root is outside the census. So the run types 35 documents where a run over both trees would type 36. The one it cannot reach is the only skill that no shared directory carries.

**The front matter is merged here and prepended in the other two fixtures.** Twenty-two of these 35 files already carry Agent Skills front matter with `name` and `description`. A second block above the first one would not be read, because a parser takes the first block. So the Headwater keys go into the block that is already there, under n8n's own keys, and no upstream key is changed or removed. The 13 files with no upstream front matter get a new block above the body. Under both shapes every byte below the front matter is the file at the pin. **Byte identity was verified mechanically.** The blob hash of each upstream body was recomputed and compared against the tree API's blob at that commit: 35 comparisons and 35 matches.

**The `summary` facet came from the corpus rather than from an author.** `governed_document` requires `summary`, and 35 hand-written summaries would be 35 sentences of invented prose inside a fixture that exists to be byte-identical. For the 22 files with upstream front matter the summary is n8n's own `description` line, copied. For the other 13 it is the document's own first heading, copied. The fixture invents no prose, and a weak summary on 13 documents is what that costs.

**One number in the review-rules fixture has already gone stale, and it is not a number about n8n.** `docs/taxonomies/standards-spec/fixtures/n8n/.headwater/taxonomy.yml` takes `headwater/standard` at `4.0.0`, and the package in this repository is now `4.2.0`. `headwater taxonomy resolve` refuses that mismatch and prints both versions, so the commands that fixture states no longer run as written. The skills fixture takes `4.2.0` and resolves. No check reads either scalar, because `docs/taxonomies/**` is outside this repository's corpus.

### The count, extended to a third kind

The terms are #492's. #511 recorded 8 findings for the architecture documents and #512 recorded 21 for the review rules. This is the third row, and it takes the third tool #509 names.

| | Findings |
|---|---|
| Raised by `headwater check` over the typed slice | 139 |
| Also caught by `prettier` | 0 |
| Also caught by `cubic` | 0 |
| Also caught by `sync-agent-skill-links.mjs --check` | 0 |
| Caught by none of the three | 139 |
| Of those, written by `headwater check --fix` | 0 |
| Of those, needing a rewrite | 139 |

**`prettier` reaches nothing here, and for two independent reasons.** `.prettierignore` at the pin ignores `**/*.md` under a comment that reads "Ignored for now", so no markdown file anywhere in the repository reaches prettier. And `lefthook.yml` scopes its `prettier_check` command to a glob of `packages/**`, which `.agents/` is not under. Either reason alone gives zero, which is the same answer the architecture corpus got for a different reason.

**`cubic` reaches one of the 35 files and reviews none of them.** The `Frontend` agent in `cubic.yaml` names `.agents/skills/design-system/SKILL.md` in its `file_paths`, so cubic loads that one skill as guidance on every frontend review. Every `include` pattern of every agent in that file is under `packages/`. So a change to a file under `.agents/skills/` is reviewed by no agent at all. One of the 35 is an input to the tool, and none of the 35 is a subject of it.

**`sync-agent-skill-links.mjs --check` reaches all 35 and reads none of them.** `lefthook.yml` runs it at pre-commit on a glob of `.agents/skills/**`, so its reach over this slice is total where the other two tools have none. It compares the plugin symlinks against the shared tree and reads no prose. Total reach and zero overlap is the sharpest version of the result #492 asks for.

**Nothing here is mechanically fixable, and the engine states that rather than writing nothing quietly.** A `--fix` arm over the assembled root printed `no finding of this run carries a patch` and changed no file. That is 0 of 139. The review-rules corpus recorded 6 fixable of 143 under this repository's house regime. The difference is the regime rather than the corpus. A contraction and a British spelling carry a patch, and a missing heading, a broken link and a forbidden voice construction do not.

**What ages here.** Every count above is of the 35 blobs at `b0550cb`, and a later commit of `master` is a different measurement. The `last_verified` date in the front matter of this document covers neither this section nor the one above it. Both were measured at the pin on 2026-09-11 and neither moves when the rest of this document is re-verified.

## The three defects, fixed on a fork and sent to nobody

This is the first two Done-when bullets of [#495](https://github.com/headwater-ai/headwater/issues/495). Every number in this section was measured on 2026-09-08 against `n8n-io/n8n@master`, which is the live branch and not the pin.

**The `last_verified` date in the front matter covers neither this section nor [the one above](#what-was-enumerated-here-and-what-was-not), and the carve-out is deliberate rather than deferred.** A claim taken from a moving branch goes stale when the branch moves, so a single date over the whole document would say something false about this section within a week. Moving `last_verified` forward would also assert a re-verification of the two pinned runs that nobody ran.

### Every recorded finding, partitioned

The two runs and the sweep put 38 findings and facts into this document. This is all 38, in three buckets.

| bucket | count | share of 38 |
|---|---|---|
| A genuine defect in n8n's own terms | 3 | 8% |
| An artifact of this library's vocabulary | 33 | 87% |
| Contestable, and this document adjudicates neither | 2 | 5% |

**34 of the 38 were reported by one of the two `headwater check` runs, and 33 of those 34 are in the artifact bucket.** 13 came from the design-spec corpus, out of 46 check instances over 4 typed documents of 42 files under that root. 21 came from the standards-spec corpus, out of 97 check instances over 7 typed documents of 7 files. All 34 are errors. The 34th is the broken link. It sits in the genuine-defect bucket, and a rule started to report it at headwater/standard 4.3.0. The other 33 are about the distance between an admitted entry and n8n's shape: a `sequence` facet that a package has no number for, a `title` facet that upstream does not write, an identifier scheme that the entry never declares, and three headings that this tradition writes as labeled paragraphs. Not one is about n8n's prose, [for the reason recorded above](#not-one-finding-is-about-n8ns-writing).

The 143-finding run under this repository's own house regime stays out of the denominator on purpose. A house regime is not an admitted library entry, and holding somebody else's corpus to this repository's line breaks and contractions produces nothing that n8n would call a defect.

**1 of the 3 genuine defects is reported by a `headwater check` rule, and 0 of them were when this evaluation was recorded.** [The broken link](#the-broken-link-that-no-rule-reported-until-430) reached the 4.0.0 run as a fact in the graph section and never as a finding, and `link.path.unresolved` reports it from 4.3.0. One [reaches no run at all](#what-this-taxonomy-would-report-and-what-it-does-not), because no admitted entry declares a procedure-shaped kind and no run reads `.agents/skills/`. One came out of `headwater sweep`, which is a sampler and not a check. That inversion was the sharpest result here, and it is one third smaller than it was. The library, pointed at eleven real governing documents, raised 33 errors that say nothing n8n would act on. It stays silent on 2 of the 3 things n8n would fix.

The 2 contestable findings are the coverage-policy duplication reading and the undefined term "level of reach", [both recorded above](#whether-the-three-level-reach-model-holds). Neither is fixed and neither goes upstream. They are counted here rather than dropped, because a partition that quietly loses its awkward members is not a partition.

### Each defect, re-measured on the live branch

The failure this section exists to prevent is telling n8n about a defect that is not there. The report is pinned and the fork is a mirror, so nothing in this repository notices upstream fixing one. Each block below runs against `master` and prints the number that would strike the defect. All three were run on 2026-09-08 and all three defects still stand.

**Defect 1, a link that resolves to no path.** `packages/@n8n/expression-runtime/ARCHITECTURE.md:426` writes `- [n8n workflow package](../workflow/)`, which resolves to `packages/@n8n/workflow/`.

```bash
gh api -i "repos/n8n-io/n8n/contents/packages/@n8n/workflow?ref=master" 2>/dev/null | head -1
gh api -i "repos/n8n-io/n8n/contents/packages/workflow?ref=master" 2>/dev/null | head -1
```

The first prints `HTTP/2.0 404 Not Found` and the second prints `HTTP/2.0 200 OK`. A 200 from the first, or a 404 from the second, makes this defect false.

**Defect 2, a source-of-truth claim that no directory backs.** `.agents/skills/spec-driven-development/SKILL.md:8` names `.agents/specs/` as the source of truth for architectural decisions, API contracts and implementation scope.

```bash
gh api "repos/n8n-io/n8n/contents/.agents?ref=master" --jq '[.[].name] | join(" ")'
```

It prints `review-rules skills`. Any output that holds `specs` makes this defect false.

**Defect 3, a count in prose that the configuration contradicts.** `.agents/review-rules/README.md:25` reads "Security and QA & DX deliberately don't link `testing/`", which names two agents. <!-- headwater allow=language.controlled.not_met scope=block until=2027-09-08 reason=false_positive note=quoting n8n's sentence verbatim, because the fix rewrites this exact line -->

```bash
gh api "repos/n8n-io/n8n/contents/cubic.yaml?ref=master" --jq .content | base64 -d | grep -c '^    - name: '
gh api "repos/n8n-io/n8n/contents/cubic.yaml?ref=master" --jq .content | base64 -d | grep -c 'review-rules/testing/coverage.md'
```

The first prints `5` and the second prints `2`, so three agents do not link `testing/` where the prose names two. Any pair whose difference is 2 makes this defect false.

### The fixes, and the rule that found each one

The fixes stand on the branch [`fix-agents-doc-defects`](https://github.com/headwater-ai/n8n/tree/fix-agents-doc-defects) of the fork, cut from the pin, one commit per defect. Titles follow n8n's [pull-request title convention](https://github.com/n8n-io/n8n/blob/master/.github/pull_request_title_conventions.md).

**Two of the three files carry the same bytes on `master` today as at the pin, and the third does not.** `packages/@n8n/expression-runtime/ARCHITECTURE.md` is blob `d4d9c5b` at both, and `.agents/skills/spec-driven-development/SKILL.md` is `57470f1` at both. `.agents/review-rules/README.md` moved from `28140c5` to `b95b525`, in two hunks: the `security/` scope row of the Layout table, and a new paragraph under step 3 of *Adding a rule*. **Neither hunk reaches the three lines that the third fix rewrites**, which are lines 25 to 27 and identical at both. So a fix cut from the pin still applies to all three, and the reason is a line-level comparison rather than a whole-file one. This was measured on 2026-09-08 and it goes stale the same way every other number in this section does.

| defect | commit | what the fix writes | the rule that reported it |
|---|---|---|---|
| 1 | [`c7c3613`](https://github.com/headwater-ai/n8n/commit/c7c3613c5197b4d530383d6a7c0042ba298cdb65) | `../workflow/` becomes `../../workflow/` | none: a graph-section fact, and `link.fragment.unresolved` reads a fragment rather than a path |
| 2 | [`667f926`](https://github.com/headwater-ai/n8n/commit/667f926ca88651d9ef6f38bbfb2e8d2ba2b182f4) | the claim becomes conditional, and the skill says what to do when the directory is absent | none: no admitted entry types `.agents/skills/`, so no run reads the file |
| 3 | [`3d450a8`](https://github.com/headwater-ai/n8n/commit/3d450a89dea6fa917d9778665550a0e094313f98) | the sentence names DB migrations as the third agent | none: `headwater sweep` carried it, and a sweep is a proposal rather than a verdict |

**The rule column reads `none` three times, and that is the result rather than a gap in the table.** Each fix here was found by a person reading, by a graph fact that no rule consumes, or by a sampler. Whether an unresolved prose link should become a finding is one open obligation of this repository. Whether a stated practice that no artifact backs should become a rule is [#496](https://github.com/headwater-ai/headwater/issues/496), and defect 2 is one instance of it.

**No fixture copy was touched.** Two of the three fixed files are also held here as fixture copies, and [the typing note above](#what-was-typed-and-as-what) claims byte identity with the pin for those copies. Editing a copy would make that claim false and no check would report it. The fixes live on the fork and nowhere else.

### Why nothing was sent, and what would reopen it

The third Done-when bullet of [#495](https://github.com/headwater-ai/headwater/issues/495) asks for a pull request against `n8n-io/n8n`. **Nothing was posted, and that is a decision rather than an omission.** n8n's [CONTRIBUTING.md](https://github.com/n8n-io/n8n/blob/master/CONTRIBUTING.md) gates an incoming change three ways, and an autonomous run satisfies none of them.

- **A bug fix needs a linked issue first.** Section 1 states that a bug-fix pull request with no linked issue is returned. Filing that issue is a claim on n8n's tracker, and it belongs to a person.
- **A typo-only pull request is rejected.** Section 5 asks that a small fix go into a larger related change. Defect 1 on its own is exactly the shape that rule names.
- **The description must be the author's own words.** Section 4 requires the author to understand every line and to write the description themselves, and it forbids pasted model output. Section 3 closes a pull request with no real description, and section 6 closes one with no tests after 14 days.

So this run prepared everything and posted nothing. The commits are on the fork. The upstream issue text is in [`.headwater/notes/n8n-upstream/issue.md`](https://github.com/headwater-ai/headwater/blob/main/.headwater/notes/n8n-upstream/issue.md) and the pull-request body is in [`.headwater/notes/n8n-upstream/pull-request.md`](https://github.com/headwater-ai/headwater/blob/main/.headwater/notes/n8n-upstream/pull-request.md). Both are drafts for a person to rewrite in their own words and to send under their own name, with the disclosure that section 4 invites.

**What reopens it:** the owner of this repository says to post, or n8n drops the linked-issue gate and the own-words gate. The upstream half is [#729](https://github.com/headwater-ai/headwater/issues/729), which carries the branch, the two drafts and the bar for sending. It stays open on a person, and this document records why rather than leaving a reader to guess.

## What this did not cover

Sending any finding of the skills corpus upstream, or fixing one, which is [#495](https://github.com/headwater-ai/headwater/issues/495). Typing that corpus is no longer outstanding: [#509](https://github.com/headwater-ai/headwater/issues/509) was blocked on a procedure-shaped kind, the `diataxis-site` entry declares one, and *The skills corpus, typed* above is the run.

A census of all 22 review-rule files as typed documents. Seven is the sample, and #508's own scope note sets that bar.

Sending anything upstream to n8n, which is [#729](https://github.com/headwater-ai/headwater/issues/729). [The three genuine defects are fixed on the fork](#the-fixes-and-the-rule-that-found-each-one) under [#495](https://github.com/headwater-ai/headwater/issues/495), and nothing was reported to n8n. No issue and no pull request was opened against their repository, and the section above says why.

Fixing the 33 findings that the two runs reported. All 33 are artifacts of this library's vocabulary rather than defects in n8n's writing, so there is nothing there for n8n to accept.

Whether the gap between a stated practice and an absent artifact deserves a check rule is [#496](https://github.com/headwater-ai/headwater/issues/496). [One instance of that gap is reported above](#what-this-taxonomy-would-report-and-what-it-does-not), and the rule question stays open.

A census of every architecture document in the monorepo. Four is the sample, and the fourth Done-when bullet of [#492](https://github.com/headwater-ai/headwater/issues/492) asks for one document of this kind at minimum.

## Reproducing this

The [design-spec fixture README](../taxonomies/design-spec/fixtures/n8n/README.md#how-to-run-this-corpus) carries the five commands for the architecture corpus. The [standards-spec fixture README](../taxonomies/standards-spec/fixtures/n8n/README.md#how-to-run-this-corpus) carries six for the review rules, and the sixth is the sweep. Nothing in CI runs either corpus, which is the same posture the earlier fixture corpora of this library have.
