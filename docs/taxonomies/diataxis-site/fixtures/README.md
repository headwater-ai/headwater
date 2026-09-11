# Fixtures for the Diátaxis documentation-site bundle

`tools/repo/diataxis-fixtures.sh` runs every case below. Until 2026-09-11 this page was prose and nothing executed it, and the cost of that is recorded under *What the runner caught on its first run*.

The worked shape corpus under `corpus/` holds one document of each concrete kind. The external corpus of admission criterion 4 is not in the tree: the runner assembles it from the pinned files under `sources/` at each run, adding front matter and nothing else.

## The case table

| Case | Expected result |
|---|---|
| Isolated selection | `taxonomy validate` and `taxonomy resolve` exit 0 after the fixture overlay supplies the identifier namespaces the selection leaves without one |
| Entry selection with three library entries | resolution exits 0 |
| Constructor sweep | `headwater new tutorial`, `how_to`, `reference`, and `explanation` succeed |
| Overlay collision | resolution exits 1 and first names `purposes.procedure.intent` |
| Criterion 6 | the entry's address set is disjoint from every other entry of this library |
| Criterion 3 | the bundle writes no `override` and no `remove` |
| Worked shape corpus | a strict run over `corpus/` exits 0, and one document stands at each of the four kinds |
| Planted defect | the runner writes `kind: reference` onto the document on the `tutorials` shelf, and the run reports `shelf.placement_is_primary` |
| Byte identity | each assembled external document, below its front-matter block, is its pinned source byte for byte |
| External corpus typing | each of the four kinds types one assembled document, and every document of that corpus is typed |

The isolated run on 2026-09-09 validated and resolved successfully, then constructed all four kinds successfully. The collision run exited 1 and named four collisions: `purposes.procedure.intent`, `identifier_schemes.tutorial_id.pattern`, `kinds.tutorial.is_a`, and `shelves.tutorials.title`. The first one is the discriminator.

The collision is with this repository's own adopter overlay and not with an entry of the library, which is why it is no failure of criterion 6. The criterion binds an entry against the entries already admitted, and the runner measures that population separately: `diataxis-site` shares no address with `design-spec`, `decision-record`, `diataxis`, `evidence-and-obligation`, `standards-spec` or `brd-prd`.

## The external corpus, and where each document comes from

The runner reads this table. Each row names a concrete kind, the shelf its documents stand on, the pinned file the runner assembles, and the path the assembled document takes. A kind the bundle declares with no row here fails the run, and a row naming a kind the bundle does not declare fails it too.

| Kind | Shelf | Pinned source | Assembled path | SHA-256 of the pinned file |
|---|---|---|---|---|
| `explanation` | `docs/explanations/**` | `sources/cockroach/life_of_a_query.md` | `docs/explanations/life-of-a-query.md` | `d34dc409343a38e6234cd3f2fa45452a57f5124d3c03f47ed04212b14d23ed27` |
| `how_to` | `docs/how-to/**` | `sources/sysl/best-practices-intro.md` | `docs/how-to/best-practices-intro.md` | `52e91d22214a6c94e8ea8472433396dacfd623cb9745e90d8970ec1f17fc97ef` |
| `reference` | `docs/reference/**` | `sources/sysl/lang-spec.md` | `docs/reference/lang-spec.md` | `2d96f55d9897a2af9f8b3b10aa3bb368d56ae476b14cce68a28751aebc286ee0` |
| `tutorial` | `docs/tutorials/**` | `sources/sysl/tutorial.md` | `docs/tutorials/tutorial.md` | `8b1d338dd4a99310724860042c37f11bc7e0e259ec2e5dafa31047c32705bf69` |

`sysl` commit `d34f35389ac019db7c37300ac62c2306bfa766b2` supplies `docs/docs/tutorial.md`, `docs/docs/best-practices/intro.md` and `docs/docs/lang-spec.md`. `cockroach` commit `8812064a015d2faf99d3fc7e15880f94042954b0` supplies `docs/tech-notes/life_of_a_query.md`.

**Two cases hold the pinning, and each one holds only half of it.** The byte-identity case strips the front matter the runner wrote and compares what is left against the pinned file with `cmp`. That case cannot fail on the content of a pinned source, because the runner assembled the document from the same file it compares back to: it holds the stripper and it says nothing about whether the vendored file is still what the cited commit holds. The digest case is the other half. Each digest above is asserted against the file on disk, so an edit to any vendored source reddens the run and the recorded denominators below are never quietly re-measured against different bytes.

**Neither case reaches the upstream commit, and nothing in this repository does.** A digest recorded beside a file it was computed from is a seal against local drift and not a proof of provenance. Confirming that `d34f3538` and `8812064a` still hold these bytes takes a network fetch, which no fixture runner here performs. That gap is real and it is stated rather than papered over.

The identifier of each assembled document comes from the `pattern` of `identifier_schemes.<kind>_id` in `bundle.yml`, with the namespace `DX` and the file name as the slug. No identifier is written into the runner.

## The run record that criterion 4 asks for

Criterion 4 asks for an external corpus "typed by the entry and recorded with its source, revision, paths, and run". It asks for a recorded run. It does not ask for a clean one. Recorded on 2026-09-11, against engine 0.1.2 and `headwater/standard` 4.2.0:

| Denominator | Reading |
|---|---|
| Documents under the corpus root | 4 |
| Typed | 4 of 4, one at each declared kind |
| Findings | 47 |
| Errors | 16 |
| Warnings | 31 |
| Exit status | 0 from `headwater check`, 1 from `headwater check --strict` |
| `section.required.missing` | 11 of the 16 errors, in 4 of the 4 documents |
| `link.path.unresolved` and `link.fragment.unresolved` | 5 of the 16 errors, 3 and 2 |
| `voice.forbidden_construction` | 31 of the 31 warnings |

**The kind assignment holds and the section contracts do not.** Every one of the four real documents types at the kind the entry gives its shelf, with no edit to any body. Not one of them carries the headings that the kind requires: the `tutorial` kind asks for Goal, Prerequisites, Steps and Result, and `sources/sysl/tutorial.md` opens at "Hello World". That is 11 errors across the four documents, and it is the entry's own report against itself. A section contract written beside a kind set is a Headwater invention, and the tradition it models constrains the reader's purpose rather than the writer's headings. The remedy is a ruling on whether a kind of a library entry may declare no section contract at all, and it is not an edit to somebody else's prose. The entry's [doctrine](../doctrine.md#findings) carries that remedy as finding 1, and the [library index](../../README.md) reads criteria 3, 6 and 7 against this entry.

The other five errors are relative links inside the pinned sources that resolve against their own site layouts and not against a Headwater corpus root. They are a property of lifting a file out of its repository, and they are recorded rather than repaired for the same reason.

## What the runner caught on its first run

The planted defect used to live in the tree. `corpus/docs/tutorials/get-started.md` carried `kind: reference` as a committed field, so a strict run over the worked corpus exited 1 and the corpus never demonstrated the clean state it is for. The document was still typed `tutorial`, because the shelf decides the kind and the field is a conflicting restatement rather than a retyping; what the plant cost was the other half of the contract. Nothing reported it either way: `docs/taxonomies/**` is excluded from this repository's census, `engine/crates/check/fixtures/corpus.checks` records 0 findings under this entry, and the case table above was prose that no verb read. The plant now lives in the runner, which writes it into a scratch copy. That is the only arrangement where the clean corpus and the refusal are both held by something that runs.

The planted input is `kind: reference` on the document standing on the `tutorials` shelf. A plain `headwater check` exits 0 over it and `headwater check --strict` exits 1, and both report `shelf.placement_is_primary`: the `tutorials` shelf carries `tutorial`, and the document restates it as `kind: reference`. The rule treats an explicit `kind` field as a conflicting restatement on every homogeneous shelf.

## What the runner enumerates rather than lists

No kind name, shelf path, source path or expected count is written into `tools/repo/diataxis-fixtures.sh`. The concrete kinds come from the `add:` block of `bundle.yml`, the mode map and the digests come from the source table above, and the identifier patterns come from the same bundle. The finding denominators are printed by each run and asserted by nothing, because a finding count is a property of somebody else's prose and a run that asserted it would redden the day a source repository was re-pinned, or the day a rule of the engine was widened.

**What catches a stale denominator, then.** The table above is a snapshot that no gate re-derives, so on its own it can drift. **Two edits inside this repository move a denominator, and they are held by different things and to different degrees.**

*Editing a pinned source.* Appending one broken link to a pinned source moves the run from 47 findings to 48 and from 16 errors to 17, and a runner that only printed the new figures would exit 0 over it. The digest case closes that route: every pinned file is sealed against the table above, so the edit reddens this suite.

*Editing the bundle declaration.* The kinds' section contracts produce 11 of the 16 errors, so the declaration moves the denominators as readily as a source does and no seal here reads it. Measured on 2026-09-11, emptying `sections.require` on `kinds.reference` in the vendored copy alone moves the run from 47 findings to 46 and from 16 errors to 15, leaves all four digest cases green, and exits this suite at 0. What catches it is `tools/repo/library-index-fixtures.sh`, whose byte-identity case between the authored library entry and the vendored copy fails at 31 passed and 1 failed. That protection is one-sided by construction: it holds the two copies against each other and neither against the table above.

*The gap, measured rather than assumed.* Making the same edit to **both** copies is caught by nothing on this tree. `tools/repo/library-index-fixtures.sh` returns to 32 passed and 0 failed, this suite stays at 36 passed and 0 failed, `headwater check --strict` exits 0, `headwater taxonomy resolve --check` exits 0, and the denominators above stand at 47 and 16 while the tree produces 46 and 15. A reviewer reading a bundle diff is the only thing between that edit and this page.

*Two more routes leave the table stale without any edit here.* An engine whose rules widened, and a re-pin to a newer upstream commit. Both are deliberate acts by a person already editing this page.

A run that asserted the counts would turn every one of these into a red suite with a number to copy, which teaches a reader to copy numbers rather than re-read the run. So the counts stay recorded, and this paragraph is the statement of what that costs.

## A second external corpus: n8n's skills

[`n8n/`](n8n/README.md) holds a second external corpus, vendored into the tree rather than assembled at run time. It is the 35 files of `.agents/skills/` from `n8n-io/n8n` at `b0550cb3cb4d1752546a69056c55eccfb9111a12` on `master`, every body byte-identical below its front matter, typed by one homogeneous shelf at `.agents/skills/**` carrying `how_to`. That corpus is what #509 was open for, and its README records the assembly commands and every denominator.

| Denominator | Reading |
|---|---|
| Files under the corpus root | 35 |
| Typed | 35 of 35, all `how_to` |
| Excluded, and untyped | 0 and 0 |
| Findings | 139 |
| Errors | 118 |
| Warnings | 21 |
| Exit status | 0 from `headwater check`, 1 from `headwater check --strict` |

**`tools/diataxis-fixtures.sh` does not run it, and `sh tools/taxonomy/n8n-fixtures.sh` does.** The runner above assembles its four documents from `sources/` at each run, and this corpus is 425,316 bytes of vendored prose that no runner reassembles. A second job does: it reads the commands out of [the n8n README](n8n/README.md), runs them in CI as a blocking step, and holds every figure that page states. It holds the [design-spec](../../design-spec/fixtures/n8n/README.md) and [standards-spec](../../standards-spec/fixtures/n8n/README.md) n8n corpora the same way. [What that job holds and what it does not](../../design-spec/fixtures/n8n/README.md#what-that-job-holds-and-what-it-does-not) is stated once, on the design-spec page.

**The table above is not one of the figures it holds.** That job reads `n8n/README.md`, and this is a second copy of the same seven readings on a different page. So is [the evaluation](../../../evaluations/n8n-worked-example.md). Each one can go stale while CI stays green, and the remedy when a number moves is to correct every copy rather than the one that failed. The assembled four-document corpus above is held by no job at all, and its denominators carry the drift the paragraphs above measure.

**One result belongs beside the run record above, because it is the same finding from a second corpus.** 104 of the 139 are `section.required.missing`, which is 104 of a possible 105 over 35 documents and three required headings. The assembled corpus reports 11 section errors over 4 documents. Two unrelated real corpora, one written for a documentation site and one written for an AI agent, and neither carries the headings this entry's kinds require. That is the entry's report against itself a second time, and #349 carries the remedy.
