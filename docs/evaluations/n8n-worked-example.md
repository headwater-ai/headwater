---
id: HW-EVAL-n8n-worked-example
status: current
status_since: 2026-09-01
last_verified: 2026-09-01
summary: Eleven real governing documents from n8n, typed against two entries of the library, and what two runs and one coherence sweep found in a corpus that keeps its prose beside the code.
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

This document reports two of the three kinds that [#492](https://github.com/headwater-ai/headwater/issues/492) names. The architecture documents are [#492](https://github.com/headwater-ai/headwater/issues/492) itself and the review rules are [#508](https://github.com/headwater-ai/headwater/issues/508), and the two halves are marked below. The third is [#509](https://github.com/headwater-ai/headwater/issues/509). The reason for the split is that the third has no admitted kind to be typed against at all.

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

**83 files under the corpus root, 4 typed, 79 excluded, 4 checked, 42 check instances, 8 findings, all 8 of them errors.** `headwater check --strict` exits 1. The 79 excluded files are the vendored taxonomy package.

Two findings per document, and the same two on each one.

| Document | Rule | Severity |
|---|---|---|
| all four | `facet.required.missing` (`OB-FACET-1`), `sequence` not declared | error |
| all four | `identifier.unusable` (`OB-ID-2`), a typed `design_spec` that declares no identifier | error |

`identifier.unusable` is a second finding about the entry rather than about n8n. The design-spec entry declares five kinds and no identifier scheme for any of them. This repository mints `spec_id` in its own overlay, and an overlay is not an admitted library entry. So a corpus that takes the entry alone gets documents that can stand at neither end of a relation. The [diataxis fixture](../taxonomies/diataxis/fixtures/README.md) reported the same rule six times for the same reason.

### Not one finding is about n8n's writing

The base package declares `language: default: {tag: en-US, controlled: none}`. So no admitted entry of this library holds prose to a controlled language, a source form, a sentence length or a retired term. `voice.forbidden_construction` ran four instances and reported nothing. `section.required.missing` never instantiated, because `design_spec` declares no required sections.

**The canonical library, pointed at four real governing documents, says nothing at all about the prose in them.** That is worth stating before any count, because a large finding total over somebody else's corpus is easy to mistake for a result.

### The broken link that no rule reports

`packages/@n8n/expression-runtime/ARCHITECTURE.md:426` writes the link `[n8n workflow package]` with the target `../workflow/`. From that directory the target is `packages/@n8n/workflow/`, and the tree at the pin holds no such path. The package it means is `packages/workflow/`, which holds 244 files. This was checked against the whole tree at the pin and not against the four-file slice.

`headwater check` sees it and reports it in the graph section as `1 prose links that did not resolve`, with the file, the line and the reason. **No rule turns it into a finding.** `link.fragment.unresolved` reads a fragment inside a document, ran four instances, and reported nothing.

So the one defect here that a reader would call a defect appears as a fact and not as a finding. Neither of n8n's own two tools reports it either.

## The scattered-corpus question

This is the third Done-when bullet of [#492](https://github.com/headwater-ai/headwater/issues/492), and it is the reason the issue exists.

**First, a correction to how the issue states it.** n8n does have a root `docs/` directory, and it holds 272 files at the pin. 271 of them sit under `docs/generated/` and are `tbls` output produced from the database migrations. The 272nd is `docs/db.md`. It is an authored page, and its subject is how that output is generated and where to read it. No architecture document, no review rule and no skill is under `docs/`. The issue's claim is right in substance and wrong as literally written. The accurate statement is narrower. n8n has a `docs/` directory that carries generated output and one index page for it. Its governed prose lives elsewhere, scattered.

**The design-spec entry as it ships types none of these documents.** Its one shelf is `spec_series` at `docs/spec/**`, and not one file of this corpus is under `docs/`. Run the same assembly with that shelf alone and the run reports **83 files, 0 typed, 4 untyped, 0 checked, 2 check instances, 0 findings**. `headwater check --strict` exits **0**.

A green strict run over four real governing documents that no rule read is what the census crate calls *systematically green*. The engine is honest about it in the census, which carries four rows saying `no shelf pattern claims this path`. Nothing else says the corpus went unchecked.

### What shape the shelf declaration had to take

Three properties, and each one was measured rather than assumed.

**The corpus root is `packages` and not `docs`.** That forces an exclusion, because a taxonomy package is found under `packages/` and n8n's prose is under `packages/` too. A corpus root that reaches the second reaches the first. The three earlier fixtures of this library never met this, because each of them roots its corpus at `docs`.

**The path pattern has to be `packages/**`, which fixes one segment out of 27,688 files.** The four documents share no directory. What they share is a filename, and the pattern language reads a path rather than a name. `packages/**/ARCHITECTURE.md` is legal and it claims **2 of the 4**, because the other two are named `architecture.md` and `ARCHITECTURE_CONNECTION_VS_SETTINGS.md`. That run reports 2 typed, 2 untyped, 22 check instances and 4 findings, and the two documents it misses report nothing at all.

**The shelf has to be heterogeneous with one admitted kind, and that reads as a contradiction.** `kinds.design_spec` requires `doc_type`, because the entry's own shelf is heterogeneous and needs a discriminator. A homogeneous shelf refuses a document that restates the kind its placement already states. So a `design_spec` on a homogeneous shelf reports an error whichever way the front matter is written:

| The shelf, and the front matter | Findings |
|---|---|
| `homogeneous: true`, with `doc_type: design_spec` | 12, adding `shelf.placement_is_primary` on all four |
| `homogeneous: true`, with `doc_type` removed | 12, adding `facet.required.missing` for `doc_type` on all four |
| `homogeneous: false`, `discriminator: doc_type`, `kinds: [design_spec]` | 8 |

The third row is what the fixture commits. A one-kind heterogeneous shelf makes the discriminator carry no information at all. Every document writes the one value the shelf admits, and the key exists to satisfy a facet requirement rather than to tell two kinds apart.

**The answer to the bullet.** A shelf declared as a path pattern does resolve against a scattered corpus, and it costs three things that no collected corpus pays. An exclusion the collected shape never needed. A pattern so broad that it selects a whole source tree. A heterogeneous shelf with one kind on it. None of the three is a defect of the engine. All three are the shelf model meeting a corpus whose placement carries no information about kind.

## The count

This is the fifth Done-when bullet, scoped to this one kind. It exists so that nobody has to take on faith that a finding is worth anything. A finding that a project's own tooling already catches is not evidence that Headwater helps.

### What `prettier` catches: zero, and not for the reason the issue gives

The issue states that `lefthook.yml` runs `prettier --write` on every staged `.md` file, so Markdown formatting is already enforced. **The first half is true and the conclusion is false.**

`lefthook.yml` does carry a `prettier_check` command with the glob `packages/**/*.{vue,yml,md,css,scss}`, which matches all four of these paths. But `.prettierignore` at the repository root carries the line `**/*.md`, under the comment `# Ignored for now`. Prettier honors its ignore file even for a path named on the command line, so the hook formats no Markdown at all.

That was verified rather than reasoned. A scratch directory with a `.prettierignore` of `**/*.md` and one badly formatted Markdown file: `prettier --check` reports `All matched files use Prettier code style` and exits 0. Remove the ignore file and change nothing else, and the same command reports a style issue on the same file.

**So prettier catches 0 of the 8 findings, and 0 of anything else in these four files.**

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

**So cubic catches 0 of the 8.**

### The count itself

| | Under the entry alone | Under the entry plus this repository's house regime |
|---|---|---|
| Findings raised | 8 | 162 |
| Already caught by `prettier` | 0 | 0 |
| Already caught by `cubic` | 0 | 0 |
| Caught by neither | 8 | 162 |
| Of those, written by `headwater check --fix` | 0 | 1 |
| Of those, needing a human rewrite or a declaration change | 8 | 161 |

The second column is a probe and not a declaration, on the precedent the [brd-prd fixtures](../taxonomies/brd-prd/fixtures/README.md#what-the-bases-voice-regime-would-have-reported) set. It binds `regimes.language.ste_house` from this repository's own overlay onto `kinds.design_spec`, changes nothing else, and touches no document. The [fixture README](../taxonomies/design-spec/fixtures/n8n/README.md#what-this-repositorys-own-house-regime-would-have-reported) carries it per rule and per document.

**Three things about that second column matter more than its total.**

136 of the 154 extra findings are hard wraps, and they come from two documents rather than four. One carries 108 and one carries 27. The other two carry one between them, because they are already written one line per paragraph. A house rule that reads as a verdict on a project is a verdict on one editor setting.

**There is no British spelling anywhere in the four documents.** The rule that catches one ran on every document and found nothing. That contradicts what this work expected to find, and it is worth recording as a correction rather than quietly dropping.

**Exactly one finding of the 162 is mechanically fixable.** `headwater check --fix` writes a British spelling, a contraction whose expansion is one word, a retired term that names a replacement, and a missing reciprocal link. One contraction, `doesn't`, is the whole of the intersection, and the report marks it as the only `fix (mechanical)` line. The 136 hard wraps are mechanical to a reader and carry no patch, and `engine/crates/check/src/source_form.rs` states why in its own comment. `headwater check --fix` was never run over this corpus.

**What the count says.** Under the canonical library alone the honest total is 8. Every one of the 8 is about a declaration rather than about n8n, and every one is invisible to both of n8n's tools. Under a house language regime the total is 162, of which one is mechanically fixable and 136 are one editor's line-wrapping habit. Neither number is a claim that Headwater found 8 or 162 problems in n8n. The 8 are what a taxonomy learns about itself by meeting a corpus it did not author.

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

**7 files under the corpus root, 7 typed, 0 excluded, 7 checked, 93 check instances, 21 findings, all 21 of them errors.** `headwater check --strict` exits 1. The graph reads 7 nodes and 0 declared edge halves.

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

**Six findings of 143 are mechanically fixable, against one of 162 before.** Five contractions and one spelling are the whole intersection, and the report marks each with `fix (mechanical)`. The 87 hard wraps are mechanical to a reader and carry no patch. `headwater check --fix` was never run over this corpus.

## What this adds to criterion 4

This is the seventh Done-when bullet of #492 and the sixth of #508. Two entries of the library now carry this repository. The design-spec entry carries the invented Beacon corpus and n8n's architecture documents, and its [doctrine](../taxonomies/design-spec/doctrine.md#worked-instances) cites both. The standards-spec entry carries the invented Beacon ladder and n8n's review rules, and its [doctrine](../taxonomies/standards-spec/doctrine.md#worked-instances) gained a `Worked instances` section that cites both.

**One note, which is not fixed here.** This document is a `kinds.evaluation`, and `evaluation` moved out of design-spec into the `evidence-and-obligation` entry with HW-DR-0044. That entry is not in the admitted-entry table of `docs/taxonomies/README.md`, which still lists five rows while `docs/taxonomies/` holds six directories. Two admitted entries now declare `requires: [evidence-and-obligation]`. That gap is [#510](https://github.com/headwater-ai/headwater/issues/510) and is not touched here.

## What this did not cover

The third of the three kinds [#492](https://github.com/headwater-ai/headwater/issues/492) names. `.agents/skills/` is [#509](https://github.com/headwater-ai/headwater/issues/509), and it is blocked on a ruling rather than on effort, because no admitted entry declares a procedure-shaped concrete kind.

A census of all 22 review-rule files as typed documents. Seven is the sample, and #508's own scope note sets that bar.

Fixing any finding, or sending anything upstream to n8n, is [#495](https://github.com/headwater-ai/headwater/issues/495). Nothing here was reported to n8n and no pull request was opened against their repository.

Whether the gap between a stated practice and an absent artifact deserves a check rule is [#496](https://github.com/headwater-ai/headwater/issues/496).

A census of every architecture document in the monorepo. Four is the sample, and the fourth Done-when bullet of [#492](https://github.com/headwater-ai/headwater/issues/492) asks for one document of this kind at minimum.

## Reproducing this

The [design-spec fixture README](../taxonomies/design-spec/fixtures/n8n/README.md#how-to-run-this-corpus) carries the five commands for the architecture corpus. The [standards-spec fixture README](../taxonomies/standards-spec/fixtures/n8n/README.md#how-to-run-this-corpus) carries six for the review rules, and the sixth is the sweep. Nothing in CI runs either corpus, which is the same posture the earlier fixture corpora of this library have.
