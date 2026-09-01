# Fixtures for the design-spec taxonomy — the n8n corpus

Four real architecture documents from [`n8n-io/n8n`](https://github.com/n8n-io/n8n), typed against this entry. Nothing here is invented. The [other fixtures of this entry](../README.md) hold a miniature corpus for an imaginary system called Beacon, which is what criterion 4 calls a *realistic* corpus. This one is what criterion 4 calls a *real* one.

Every number below came from the run this file quotes. Nothing here is predicted.

## Modification notice

**These four files are modified copies of n8n software.** The modification is the addition of one Headwater front-matter block at the top of each file, and nothing else. **The body of every copy, from the blank line after the closing `---` to the last byte, is identical to the pinned upstream file.** No word, no heading, no link, no spelling, no contraction and no line break was changed.

n8n's [Sustainable Use License](LICENSE.md) requires a prominent notice on a modified copy, and this section is it. The same license requires that anyone who receives any part of the software also receives a copy of the terms, so [`LICENSE.md`](LICENSE.md) beside this file is a verbatim copy of n8n's own, and not a link to it.

## The pin

| | |
|---|---|
| Fork | https://github.com/headwater-ai/n8n (forked 2026-09-01, default branch only) |
| Upstream | https://github.com/n8n-io/n8n |
| Branch | `master`. n8n's license states that content of branches other than `master` is not licensed, so every citation here names `master` and nothing else |
| Commit | `b0550cb3cb4d1752546a69056c55eccfb9111a12`, committed 2026-09-01T16:27:59Z |

The fork carries `LICENSE.md` and `LICENSE_EE.md` untouched and holds no commit of this project's. It is a mirror, and its only job is to keep the pin readable after upstream moves.

**The pin is not caution, and there is evidence for that.** Between a survey of this repository on 2026-08-31 and an adjudication of it on 2026-09-01, n8n's `.agents/review-rules/` corpus gained a whole category. One day, one new directory of five governing documents.

## The four files

Each row names the upstream path, which is also the path the copy sits at under `corpus/`.

| Upstream path at the pin | Bytes | Kind |
|---|---|---|
| `packages/@n8n/instance-ai/docs/architecture.md` | 23,914 | `design_spec` |
| `packages/@n8n/expression-runtime/ARCHITECTURE.md` | 16,796 | `design_spec` |
| `packages/@n8n/instance-ai/evaluations/ARCHITECTURE.md` | 5,454 | `design_spec` |
| `packages/@n8n/local-gateway/docs/ARCHITECTURE_CONNECTION_VS_SETTINGS.md` | 1,692 | `design_spec` |

The byte counts are of the upstream body, and the copies carry that many bytes plus their front-matter block.

**None of the four is under the Enterprise License.** n8n excludes a file with `.ee.` in its name and a file under a directory with `.ee` in its name from the Sustainable Use License. The tree at the pin holds 196 paths of the first form and 1,064 of the second. None of them is one of these four, and none of them is anywhere under `packages/@n8n/instance-ai/`, `packages/@n8n/expression-runtime/` or `packages/@n8n/local-gateway/`, which are the three directories this fixture read.

## How to run this corpus

`headwater check` reads one corpus root, and this tree sits inside a directory that this repository excludes from its own corpus. Assemble a scratch root out of three committed things and run the engine over it:

    ROOT=$(mktemp -d)
    cp -R packages "$ROOT/packages"
    cp -R docs/taxonomies/design-spec/fixtures/n8n/corpus/packages/@n8n "$ROOT/packages/@n8n"
    cp -R docs/taxonomies/design-spec/fixtures/n8n/.headwater "$ROOT/.headwater"
    headwater taxonomy resolve --root "$ROOT"
    headwater check --root "$ROOT" --no-cache --now 2026-09-01

Unlike the [Beacon fixtures](../../../brd-prd/fixtures/README.md#how-to-run-this-corpus), this one commits its own `.headwater/taxonomy.yml` and `.headwater/overlay.yml` rather than describing them in prose, because the shelf declaration is the measurement rather than a detail of the harness.

The entry sits outside the corpus root on purpose, the same reason the Beacon fixtures give. Nothing in CI runs this corpus.

## What the taxonomy needed before it could read one file

**The design-spec entry, exactly as it ships, types none of these documents.** Its one shelf is `spec_series` at `docs/spec/**`. Not one file of this corpus is under `docs/`. Run the assembly above with the shelf removed from `overlay.yml` and the run reports **83 files under the corpus root, 0 typed, 4 untyped, 0 checked, 2 check instances, 0 findings**, and `headwater check --strict` exits **0**.

A green strict run over four real governing documents that no rule read is the shape of failure the census crate names *systematically green*. The engine reports it honestly in the census, as four rows saying `no shelf pattern claims this path`, and no finding anywhere says the corpus went unchecked.

**n8n does have a `docs/` directory, and it is not a counter-example.** It holds 272 files at the pin. 271 of them sit under `docs/generated/` and are `tbls` output produced from the database migrations. The 272nd is `docs/db.md`, an authored page whose subject is how that output is generated and where to read it. No architecture document, no review rule and no skill is under `docs/`. The governed prose is elsewhere, one document per package, which is the claim this fixture tests.

**A path pattern is the wrong instrument for a corpus that carries its signal in the filename.** The four documents share no directory. What they share is a name: `architecture.md`, `ARCHITECTURE.md`, `ARCHITECTURE.md`, `ARCHITECTURE_CONNECTION_VS_SETTINGS.md`. The pattern language admits `**` as a whole segment, so `packages/**/ARCHITECTURE.md` is legal. It claims **2 of the 4**: the run reports 2 typed, 2 untyped, 22 check instances and 4 findings, and the two it misses report nothing at all. The pattern that reaches all four is `packages/**`, which fixes one segment out of a corpus of 27,688 files and would claim every Markdown file in the monorepo if the corpus held them.

**The shelf has to be heterogeneous with one admitted kind, and that reads as a contradiction.** `kinds.design_spec` requires the facet `doc_type`, because this entry's own shelf is heterogeneous and needs a discriminator. A homogeneous shelf refuses a document that restates the kind its placement already states. So a `design_spec` on a homogeneous shelf reports an error whichever way the front matter is written, and both arms were run:

| The shelf, and the front matter | Findings |
|---|---|
| `homogeneous: true`, `doc_type: design_spec` declared | 12: `shelf.placement_is_primary` on all four, plus the eight below |
| `homogeneous: true`, `doc_type` removed | 12: `facet.required.missing` for `doc_type` on all four, plus the eight below |
| `homogeneous: false`, `discriminator: doc_type`, `kinds: [design_spec]` | 8 |

The third row is what this fixture commits. A one-kind heterogeneous shelf makes the discriminator carry no information: every document writes the one value the shelf admits, and the front-matter key exists to satisfy a facet requirement rather than to tell two kinds apart.

## What a run reports

`headwater taxonomy resolve` then `headwater check --no-cache --now 2026-09-01` over the assembled root, with the committed `.headwater/`: **83 files under the corpus root, 4 typed, 79 excluded, 4 checked, 42 check instances, 8 findings, all 8 of them errors.** The census reads 4 `design_spec`. The graph reads 0 nodes, 0 declared edge halves, and 1 prose link that did not resolve. `headwater check --strict` exits 1.

The 79 excluded files are the vendored taxonomy package, and that count moves when the package does. The other counts move only when this corpus moves.

Two findings per document, and the same two on every one.

**Finding 1, four times.** `facet.required.missing`, an error, under `OB-FACET-1`:

    `design_spec` requires the facet `sequence`, and it is not declared

`sequence` is an integer, and this entry's shelf layout is `{sequence:02d}-{slug}.md`. It is the numbering of a specification series: RFC 2119 comes after RFC 2118. n8n's architecture documents are one per package, and a package is not a position in a series. There is no number to write. **This is the finding, not a problem to design around**, and no fixture here invents a value to make the run pass. What the entry assumed is that a design specification belongs to a numbered series. A monorepo that keeps one architecture document per package is a corpus where that assumption is simply false, and criterion 4 exists to produce exactly this.

**Finding 2, four times.** `identifier.unusable`, an error, under `OB-ID-2`:

    a typed `design_spec` that declares no identifier, so no edge can name it

The design-spec entry declares five kinds and no identifier scheme for any of them. This repository's own overlay mints `spec_id` for its own use, and an overlay is not an admitted library entry. So a corpus that takes the entry alone gets four documents that cannot stand at either end of a relation. The `diataxis` fixture reported the same rule six times for the same reason, and the two together are one finding about the library rather than two about two corpora.

## The prose link that no rule reports

`packages/@n8n/expression-runtime/ARCHITECTURE.md:426` writes the link `[n8n workflow package]` with the target `../workflow/`. From that document's directory that resolves to `packages/@n8n/workflow/`, and the tree at the pin holds no such path. The package it means is `packages/workflow/`, which holds 244 files, and the link would have to be `../../workflow/`. **The link is broken upstream, and this was checked against the whole tree at the pin rather than against the four-file slice.**

`headwater check` sees it. It appears in the graph section as `1 prose links that did not resolve`, with the file, the line and the reason. **No rule turns it into a finding.** `link.fragment.unresolved` reads a fragment inside a document, ran four instances, and reported nothing. So the run states the fact and the report says 8 findings, none of which is this one.

Neither of n8n's own two tools reports it either. This is the one defect in these four files that a reader would call a defect, and three tools looked at it and none of them said so.

## What the entry reports nothing about

**Not one finding is about n8n's writing.** The base package declares `language: default: {tag: en-US, controlled: none}`, so no admitted entry of this library holds prose to a controlled language, a source form, a sentence length or a retired term. `voice.forbidden_construction` ran four instances and reported nothing: no sentence in these four documents states a future intent, narrates a change, or announces a phase. `section.required.missing` never instantiated, because this entry's `design_spec` declares no required sections.

The eight findings are about the metadata the typing added and the metadata it could not add. That is worth stating plainly, because a large finding count over somebody else's corpus is easy to mistake for a result.

## What this repository's own house regime would have reported

The measurement below is a probe and not a declaration, on the precedent the [brd-prd fixtures](../../../brd-prd/fixtures/README.md#what-the-bases-voice-regime-would-have-reported) set. Append `regimes.language.ste_house` from `.headwater/overlay.yml` of this repository to the fixture overlay, add `kinds.design_spec.language: ste_house`, and change nothing else. That is one regime and one line, and no line of any document.

The run reports **162 findings, 145 error and 17 warn**, against 8 findings and 8 errors before it. The 154 extra findings are:

| Rule | Count | Severity |
|---|---|---|
| `language.source_form.not_met` | 136 | error |
| `language.controlled.not_met`, a contraction (`doesn't`) | 1 | error |
| `language.controlled.not_met`, a semicolon in running prose | 9 | warn |
| `language.controlled.not_met`, a sentence past 25 words | 8 | warn |
| `language.retired_term.used` | 0 | — |

Per document:

| Document | Under the entry alone | Under the entry plus `ste_house` |
|---|---|---|
| `packages/@n8n/instance-ai/docs/architecture.md` | 2 | 122 |
| `packages/@n8n/instance-ai/evaluations/ARCHITECTURE.md` | 2 | 33 |
| `packages/@n8n/expression-runtime/ARCHITECTURE.md` | 2 | 3 |
| `packages/@n8n/local-gateway/docs/ARCHITECTURE_CONNECTION_VS_SETTINGS.md` | 2 | 4 |

**Three things in that table are worth more than the total.**

**The 136 hard-wrap findings come from two documents and not four.** `instance-ai/docs/architecture.md` carries 108 and `instance-ai/evaluations/ARCHITECTURE.md` carries 27. The other two documents carry one between them, because they are written one line per paragraph already. A house rule that reads as a verdict on a project turns out to be a verdict on the editor two of its authors used.

**There is no British spelling anywhere in the four documents.** The rule that catches one ran on every document and found nothing. n8n writes `organisation` and `behavioural` elsewhere in the repository, and not here.

**One finding of 162 is mechanically fixable, and it is the contraction.** `headwater check --fix` writes a British spelling, a contraction whose expansion is one word, a retired term that names a replacement, and a missing reciprocal link. Of the 162, exactly one is in that set, and the report marks it: it is the only line that reads `fix (mechanical)`. The 136 hard wraps are mechanical to a reader and carry no patch, and `engine/crates/check/src/source_form.rs` states why in its own comment. Nothing here was ever run with `--fix`.

## What ages, and what does not

**The pin does not move and the upstream does.** Every count above is of the four blobs at `b0550cb`. Re-run the assembly at a later commit of `master` and every number is a different measurement.

**The excluded count moves with the vendored package.** It is 79 today and it counts files of `packages/headwater-standard/`, which this corpus does not describe. The four typed rows are the corpus.

**Two findings are about declarations rather than about documents.** `facet.required.missing` on `sequence` stops the day the entry stops requiring it or the day somebody writes a number, and neither is a change to n8n. `identifier.unusable` stops the day the entry mints a scheme. Both are recorded in [the evaluation](../../../../evaluations/n8n-worked-example.md) as findings against the library.
