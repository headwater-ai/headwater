---
"headwater:generated": "shelf_sections. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-REG-open-questions
doc_type: decision_register
title: Open questions, closed and redirected
relations:
  superseded_by:
    - HW-REG-decisions
    - HW-REG-open-obligations
---

# Decision records

60 documents on this shelf, in the reading order this corpus derives. Each heading below is the name that the document declares, so a citation of a heading is a citation of a document.

## Q1 — Implementation language

[HW-DR-0001](../decisions/0001-implementation-language.md) — The language is Rust, with a WebAssembly build of the same crate for editor and browser embedding.

## Q2 — Schema format

[HW-DR-0002](../decisions/0002-schema-format.md) — YAML 1.2 is the concrete syntax, and Headwater owns the schema language, the reference sublanguage, the overlay language and the meta-schema.

## Q3 — How much of the default taxonomy ships in the box

[HW-DR-0003](../decisions/0003-how-much-of-the-default-taxonomy-ships-in-the-box.md) — The base package is minimal and derived from the core, and optional content ships as add-only bundles.

## Q4 — Relation storage

[HW-DR-0004](../decisions/0004-relation-storage.md) — Front matter is authoritative, and a relation instance is an object rather than a pointer.

## Q5 — Voice checking depth

[HW-DR-0005](../decisions/0005-voice-checking-depth.md) — Voice checking stays lexical, with a curated pattern set, a per-category posture and a reasoned escape hatch.

## Q6 — Where the corpus graph lives at rest

[HW-DR-0006](../decisions/0006-where-the-corpus-graph-lives-at-rest.md) — The graph never rests. Every run rebuilds it, and no derived artifact is canonical for anything.

## Q7 — Scope of the MCP surface

[HW-DR-0007](../decisions/0007-scope-of-the-mcp-surface.md) — Three classes of tool, and a landed write never ships.

## Q8 — Probe cost and cadence

[HW-DR-0008](../decisions/0008-probe-cost-and-cadence.md) — A probe is a document with a declared expectation, and cadence follows the purpose of the run.

## Q9 — Multi-repository corpora

[HW-DR-0009](../decisions/0009-multi-repository-corpora.md) — A repository holds one or more corpora, the tier above harvests pinned exports, and no merged graph exists.

## Q10 — Naming

[HW-DR-0010](../decisions/0010-naming.md) — The name is Headwater, capitalized in prose and lower case as an identifier.

## Q11 — License and distribution posture

[HW-DR-0011](../decisions/0011-license-and-distribution-posture.md) — Apache-2.0 for the engine, the library, the base package and the bundles, ratified by the owner on 2026-08-11.

## Q12 — Migration path for an existing corpus

[HW-DR-0012](../decisions/0012-migration-path-for-an-existing-corpus.md) — Adoption is a migration from no taxonomy, and `headwater infer` computes the adoption payload.

## Q13 — LinkML and SHACL as substrate

[HW-DR-0013](../decisions/0013-linkml-and-shacl-as-substrate.md) — Headwater owns the language, emitters never chain, and LinkML is the last of six siblings.

## Q14 — Discovery surface

[HW-DR-0014](../decisions/0014-discovery-surface.md) — The corpus descriptor is a generated projection at `.headwater/corpus.json`, and registration went to Q16.

## Q15 — A synthesized content tier

[HW-DR-0015](../decisions/0015-a-synthesized-content-tier.md) — Every document carries a warrant from a closed set of four, and `asserted` content is admitted with limits.

## Q39 — How a figure reaches a hand-built page

[HW-DR-0039](../decisions/0039-q39-how-a-figure-reaches-a-hand-built-page.md) — The interpolating form that HW-DR-0037 admits runs before the commit rather than after it, and a script writes every figure on a hand-built page from a run.

## Q50 — Where the visual register of the hand-built pages lives

[HW-DR-0050](../decisions/0050-q50-where-the-visual-register-of-the-hand-built-pages-lives.md) — The hand-built pages read as a specification rather than as a product page, and one file holds the color tokens and type stacks that carry it. A script writes that file into each hand-built page before the commit, because the content policy of those pages admits no linked stylesheet, and the generated half links the same file because its own policy admits one. (asserted, and no human has accepted it)

## Q37 — Which parts of the site are hand-built and which are a projection of this corpus

[HW-DR-0037](../decisions/0037-q37-which-parts-of-the-site-are-hand-built-and-which-are-a-projection-of-this-corpus.md) — Everything under `site/` is hand-built and the corpus is the generated projection. A hand-built page states no figure a person typed, and nothing checks that.

## Q16 — Public presence

[HW-DR-0016](../decisions/0016-public-presence.md) — Registration needs a channel with an obliged reader, a directory of corpora is refused, and the site is a projection of this corpus.

## Q17 — Governed access and the solution layer

[HW-DR-0017](../decisions/0017-governed-access-and-the-solution-layer.md) — The serving boundary is the export step of each publishing corpus. A profile filters for an audience, and Headwater has no principals.

## Q18 — Recording adjudicated disagreements

[HW-DR-0018](../decisions/0018-recording-adjudicated-disagreements.md) — The edge does not become a node. An adjudication is a decision document that carries `overrides`.

## Q19 — Inbound integration: an external system of record

[HW-DR-0019](../decisions/0019-inbound-integration-an-external-system-of-record.md) — Imported content is `transcribed` against a committed pin, and an imported edge carries full weight from the first release.

## Q20 — Where scent lives

[HW-DR-0020](../decisions/0020-where-scent-lives.md) — An optional source-owned cue sits on a relation instance, graded against the alternatives in view at the point of decision.

## Q21 — Terminological succession, and validity under merge

[HW-DR-0021](../decisions/0021-terminological-succession-and-validity-under-merge.md) — A retired-term lexicon sits in the language regime, and a verdict reports the read set that a merge may void.

## Q22 — The integrity posture of a published package

[HW-DR-0022](../decisions/0022-q22-the-integrity-posture-of-a-published-package.md) — A package digest checks a fetched artifact against a pin that a person committed, and it is never a signature. The in-house SHA-256 is held against a second implementation rather than replaced by a dependency.

## Q23 — The engine lint floor

[HW-DR-0023](../decisions/0023-the-engine-lint-floor.md) — A published Rust guideline set is not installed as a skill, because a skill carries no rule of its own. Twenty-two lints are declared once in the workspace manifest, and each was chosen by measuring the corpus rather than by adopting a list.

## Q24 — Readability, and what a sweep can be asked about

[HW-DR-0024](../decisions/0024-q24-readability-and-what-a-sweep-can-be-asked-about.md) — Readability is not a sweep class and gets no verb, because every class of a sweep names two things that do not fit and a readability finding names one. A source file is not a slice member, because admitting one degrades the membership refusal for every class.

## Q25 — Where the namespace goes in an identifier, and who declares it

[HW-DR-0025](../decisions/0025-q25-where-the-namespace-goes-in-an-identifier-and-who-declares-it.md) — The namespace opens every identifier, and a published source declares no namespace at all, because any constant it wrote would be minted by every adopter of it at once.

## Q26 — Whether terminality belongs to a state, or to a state and a regime

[HW-DR-0026](../decisions/0026-q26-whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime.md) — Terminality stays a property of a state alone, because the per-regime reading is available at both call sites and costs the deferral that keeps one defect one finding. No published tradition wants the collision, and two that met it minted a second state name.

## Q27 — Whether a decision record is governed prose

[HW-DR-0027](../decisions/0027-q27-whether-a-decision-record-is-governed-prose.md) — A decision record is governed prose, and the overlay binds the house language regime to it. The shelf carries a prose problem rather than a quotation problem, and collapsing all 37 inline quotations moves 64 findings to 63.

## Q28 — Whether an evaluation is governed prose

[HW-DR-0028](../decisions/0028-q28-whether-an-evaluation-is-governed-prose.md) — An evaluation is governed prose, and the overlay binds the house language regime to it. The shelf carries 432 advisory findings, and the measurement that explains them is whether a rule was reading the prose when it was written rather than who wrote it.

## Q29 — Whether a corpus root may contain code, and what an interface contract may reach

[HW-DR-0029](../decisions/0029-q29-whether-a-corpus-root-may-contain-code-and-what-an-interface-contract-may-reach.md) — The corpus root stays `docs`, because a path is corpus content only where a file with no front matter is a defect, and 185 of the 186 Markdown files under `engine/` exist to be defective. An interface contract lives inside the root and reaches a crate by a `governs` edge that binds on existence alone.

## Q30 — Whether what an obligation waits on is a state or a property, and how many values it takes

[HW-DR-0030](../decisions/0030-q30-whether-what-an-obligation-waits-on-is-a-state-or-a-property-and-how-many-values-it-takes.md) — What an obligation waits on is a state and not a property, and the closed set holds four values. A required `waiting_on` facet on `obligation_record` takes `ruling`, `build`, `measurement` or `adopter`, and the decision-record bundle declares it.

## Q31 — Whether this repository becomes public, and when

[HW-DR-0031](../decisions/0031-q31-whether-this-repository-becomes-public-and-when.md) — This repository is public. The owner set 2026-09-08 and opened it on 2026-09-06, and the record stops each run from raising the schedule.

## Q32 — Which test the self-audit label states

[HW-DR-0032](../decisions/0032-q32-which-test-the-self-audit-label-states.md) — The `self-audit` label states the reader test and not the provenance test. How a finding was found is not the test, and the only question is whether a reader outside this repository is better off.

## Q33 — Whether the command line is derived, and who a flag belongs to

[HW-DR-0033](../decisions/0033-q33-whether-the-command-line-is-derived-and-who-a-flag-belongs-to.md) — `clap` derives the command line for 17 lock entries, and a flag belongs to the verb that reads it rather than to the binary.

## Q34 — Whether acceptance means merged to main, and what an agent may write before that

[HW-DR-0034](../decisions/0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md) — Acceptance is the merge onto main. A provenance block on an unmerged branch is a proposal, so an agent may write `accepted_by` there, and stop rule 5 binds main rather than the byte.

## Q35 — Whether one requirement kind holds an imported requirement and an authored one

[HW-DR-0035](../decisions/0035-q35-whether-one-requirement-kind-holds-an-imported-requirement-and-an-authored-one.md) — The `requirement` and `acceptance_criterion` kinds serve the authored population alone. A transcribed requirement is a marked file that answers no contract, so a separate kind with an empty contract carries the imported case.

## Q36 — Which of MkDocs, Docusaurus, or Astro this corpus emits navigation for, and why

[HW-DR-0036](../decisions/0036-q36-which-of-mkdocs-docusaurus-or-astro-this-corpus-emits-navigation-for-and-why.md) — MkDocs is the pick. Its `nav:` is data and not code, and Astro has no native nav format to emit at all.

## Q38 — Which link relation a rendered page carries to the served corpus descriptor

[HW-DR-0038](../decisions/0038-q38-which-link-relation-a-rendered-page-carries-to-the-served-corpus-descriptor.md) — A rendered page points at the served descriptor with `rel="describedby"`, because the IANA registry carries that token and the Headwater extension address answers 404.

## Q40 — Whether extends, bundle, requires and an overlay's taxonomy key are a mechanism or a label

[HW-DR-0040](../decisions/0040-q40-whether-extends-bundle-requires-and-an-overlay-s-taxonomy-key-are-a-mechanism-or-a-label.md) — All four keys of the family are a label rather than a mechanism today, and the specification and the meta-schema now say so in plain prose.

## Q41 — Whether Vale becomes a declared regime backend

[HW-DR-0041](../decisions/0041-q41-whether-vale-becomes-a-declared-regime-backend.md) — Vale does not become a declared regime backend, and its findings are not translated into this engine's Finding shape. It stays where spec 00 already puts it, composable alongside, because four structural mismatches answer Q24's reopening condition a second time.

## Q42 — What "one screen" means for the first help screen

[HW-DR-0042](../decisions/0042-q42-what-one-screen-means-for-the-first-help-screen.md) — "One screen" is retired as a claim about a terminal and replaced by a claim about content: the first screen carries one line per entry, and the ceiling that follows from it is 60 lines.

## Q43 — Whether a refusal under `--json` is a JSON document

[HW-DR-0043](../decisions/0043-q43-whether-a-refusal-under-json-is-a-json-document.md) — A refusal is an English sentence on standard error and never a JSON document, because `--json` names the shape of an artifact and moves no stream and no grammar.

## Q44 — Whether bundles decompose into capabilities and assemblies compose practices

[HW-DR-0044](../decisions/0044-q44-whether-bundles-decompose-into-capabilities-and-assemblies-compose-practices.md) — Design-spec's evaluation kind and its two purposes move to a new evidence-and-obligation bundle, so decision-record stops requiring the whole specification tradition.

## Coloring the CLI, and where the banner goes

[HW-DR-0045](../decisions/0045-coloring-the-cli-and-where-the-banner-goes.md) — Color is sensed per stream and off by default in a pipe, and turned off everywhere by `--no-color` or `NO_COLOR`. A new masthead banner prints on the root help screen alone, off by `--no-banner` or `HEADWATER_NO_BANNER`.

## Migrating from-version carries a semver and a release digest, kept as separate fields

[HW-DR-0046](../decisions/0046-migrating-from-version-carries-a-semver-and-a-release-digest-kept-as-separate-fields.md) — `adoption.from` is a pair, a semver and the release digest that was pinned when the migration began, and never a hash of the two.

## The served sitemap is derived from the served directory

[HW-DR-0048](../decisions/0048-the-served-sitemap-is-derived-from-the-served-directory.md) — The sitemap a reader gets is derived from the directory that is served, by one walk that both halves call. Nobody types a page list, and the assembled directory is the only place the union exists. (asserted, and no human has accepted it)

## How the two halves of the site share one host

[HW-DR-0047](../decisions/0047-how-the-two-halves-of-the-site-share-one-host.md) — One directory holds both halves of the site, composed by `tools/assemble-site.sh` with the hand-built half last. The Cloudflare Workers Builds build command runs that script, which puts a build on the deploy path for the first time. (asserted, and no human has accepted it)

## A corpus-wide fold is derived and never stored

[HW-DR-0049](../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md) — A recorded artifact holds one record per entity and derives every total, because two branches that each add one document write the same new total and a merge takes it without a conflict. (asserted, and no human has accepted it)

## Q51 — What licenses a term into the public glossary, and what licenses a sixteenth

[HW-DR-0051](../decisions/0051-q51-what-licenses-a-term-into-the-public-glossary-and-what-licenses-a-sixteenth.md) — A term reaches the public glossary because it recurs in the visitor-facing pages of the site, and for no other reason. The tutorial and the glossary itself are not sources, and the specification glossary answers to a different reader. (asserted, and no human has accepted it)

## A document is proposed at the state it will hold, and the merge activates it

[HW-DR-0052](../decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md) — An author writes the state a document will hold once it lands, and the merge activates it. The state facet answers to the merge, the way HW-DR-0034 made the warrant facet answer to it.

## A package manifest declares no selection, a recipe declares one, and a flattened manifest records provenance

[HW-DR-0053](../decisions/0053-a-package-manifest-declares-no-selection-a-recipe-declares-one-and-a-flattened-manifest-records-provenance.md) — A manifest says what a package carries, a recipe says what a composer selected, and a flattened manifest records where it came from. Three enforcement points already hold the split, and this record names it as a rule rather than as specification prose. (asserted, and no human has accepted it)

## The upper bound of a reconcile-first allocator is the corpus and a claim store

[HW-DR-0054](../decisions/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md) — A tree holds no concurrency, so two branches mint one number in silence. A claim store of one file for each identifier makes the two branches meet, and the rule that already reports a duplicate then fires on the branch. (asserted, and no human has accepted it)

## A hook reads a wire format through the engine and not through an interpreter

[HW-DR-0055](../decisions/0055-a-hook-reads-a-wire-format-through-the-engine-and-not-through-an-interpreter.md) — A harness payload is read by a verb of this engine rather than by an interpreter a session does not otherwise require. The verb answers nothing about a corpus, which is what puts it outside the term that forbids a hook verb. (asserted, and no human has accepted it)

## A language regime reaches prose through a kind, so the starter kit's doctrine carries the writing profile

[HW-DR-0056](../decisions/0056-a-language-regime-reaches-prose-through-a-kind-so-the-starter-kit-s-doctrine-carries-the-writing-profile.md) — No kind in the base package or in any bundle binds a language regime. A package that set the default value would change nothing, so the doctrine page of the starter kit carries the promise. (asserted, and no human has accepted it)

## A shelf layout is the second half of what the identifier claim store covers

[HW-DR-0057](../decisions/0057-a-shelf-layout-is-the-second-half-of-what-the-identifier-claim-store-covers.md) — The claim store covers an identifier where the file name does not determine it: a shelf that declares a layout, or a scheme that reconciles

## A blank facet value is a rule of its own and it reads every string facet

[HW-DR-0058](../decisions/0058-a-blank-facet-value-is-a-rule-of-its-own-and-it-reads-every-string-facet.md) — A declared facet that carries no content is a state that neither the required-facet rule nor the enum rule reaches. A new rule reports it, over every facet a taxonomy types as a string, at error severity. (asserted, and no human has accepted it)

## A transform over a harness session log is an observed transcript when the log arrives by a channel the model cannot write to

[HW-DR-0059](../decisions/0059-a-transform-over-a-harness-session-log-is-an-observed-transcript-when-the-log-arrives-by-a-channel-the-model-cannot-write-to.md) — A filter that keeps the tool calls of a harness session log and drops every block the model wrote is an outside observation, because the type tag it reads is the harness's and not the model's. The condition is the channel the log arrives by, and a file the session can open is not one. (asserted, and no human has accepted it)

## The engine's version stays one number, and a build's exact commit is a separate, unwired fact

[HW-DR-0060](../decisions/0060-the-engine-s-version-stays-one-number-and-a-build-s-exact-commit-is-a-separate-unwired-fact.md) — A binary's `--version` keeps naming one number, the `Cargo.toml` version a `requires_engine` range is read against. Which commit built it is a separate fact a build script now captures, kept out of that comparison and not yet on the command line.
