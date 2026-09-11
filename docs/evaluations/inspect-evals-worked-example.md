---
id: HW-EVAL-inspect-evals-worked-example
status: current
status_since: 2026-09-11
summary: "Ten real architecture decision records from a UK government project, typed against the decision-record entry, and what the run found that their own linters cannot."
last_verified: 2026-09-11
title: "inspect_evals as a worked instance — the first ADR log this project did not write"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# inspect_evals as a worked instance — the first ADR log this project did not write

Evidence for [criterion 4](../taxonomies/README.md#admission-criteria) of the canonical taxonomy library. It asks each entry to carry "at least one **external** real or realistic corpus, typed by the entry". The clause after that asks for it "recorded with its source, revision, paths, and run".

**The title of [#488](https://github.com/headwater-ai/headwater/issues/488) is stale, and its subject is live.** The title says that every worked instance corpus in this library is this repository's own writing. That stopped being true the day after the issue was filed. Measured on 2026-09-11 over the seven bundles under `docs/taxonomies/`, three of the seven already carried an external corpus. `design-spec` and `standards-spec` take one from n8n, and `diataxis-site` takes one from `sysl` and `cockroach`. What held is the narrower claim, and it is the one this document closes. The `decision-record` entry carried `fixtures/corpus/` alone. That is a miniature log for an invented system called Beacon, and its own page opens "Everything here is invented, including the names." No ADR that this project did not write had ever been typed against this tradition.

**This is the first external corpus for the `decision-record` entry, and the fourth in the library.** `diataxis`, `brd-prd` and `evidence-and-obligation` still carry none.

## What was typed, and how

| | |
|---|---|
| Upstream | [`UKGovernmentBEIS/inspect_evals`](https://github.com/UKGovernmentBEIS/inspect_evals), a UK government AI evaluation suite, 666 stars on the day of the run |
| License | MIT, so a vendored copy with attribution is clean. This is unlike n8n, whose Sustainable Use License forced the citation discipline that [the n8n evaluation](n8n-worked-example.md) records |
| Fork | [`headwater-ai/inspect_evals`](https://github.com/headwater-ai/inspect_evals), created 2026-09-11, which answers the first box below |
| Pin | `360484a06383f9260279938262d78ed646ddbca1`, committed 2026-09-11T14:42:16Z on `main`, re-read at build time rather than inherited |
| Corpus | the 10 files of `adr/` at that commit, unedited |

The 10 files and the upstream `LICENSE` are vendored under `docs/taxonomies/decision-record/fixtures/sources/inspect-evals/`. No typed copy is in the tree. `tools/repo/decision-record-fixtures.sh` assembles a corpus root from the vendored files at each run: it adds a front-matter block and it changes nothing else. CI runs it on every pull request. The [fixtures page](../taxonomies/decision-record/fixtures/README.md#the-external-corpus-of-criterion-4) carries the source table, the status map, the case table and the run record.

**Two cases hold the seal on somebody else's prose, and each holds half.** A SHA-256 case holds every vendored file against the digest the source table records. A `cmp` case holds each assembled document, below its front-matter block, against the file it was assembled from. The second cannot fail on the content of a source, because the runner assembles out of the file it compares back to. The first is what catches an edit to a vendored file. Measured on 2026-09-11: one `Accepted — implementation deferred` status line rewritten to `Accepted` moves the suite from 37 passed to 36 passed and 1 failed. The failure names the file and both digests.

That pair is the point of the arrangement. The value of an external corpus is that its prose was not written to pass. Somebody may quietly repair a heading, a status line or a link. A run that goes green after that has measured this repository against itself again, which is what #488 was filed about. Nothing else here detects it.

**Neither case reaches the upstream commit, and nothing in this repository does.** A digest recorded beside the file it was computed from seals against local drift and proves no provenance.

## The run

`headwater check --no-cache` over the assembled root, engine 0.1.2, `headwater/standard` 4.3.0, on 2026-09-11:

| Denominator | Reading |
|---|---|
| Files under the corpus root | 10 |
| Typed | 10 of 10, all at `decision` |
| Findings | 8 |
| Errors | 4, all `link.path.unresolved` |
| Warnings | 4, all `voice.forbidden_construction` |
| Exit status | 0 from `headwater check`, 1 from `headwater check --strict` |
| `section.required.missing` | 0 findings, and the rule ran 10 times over the 10 documents |

The front matter the runner writes carries six fields. Five are derived. The identifier comes from the `pattern` of `identifier_schemes.decision_id`, with the sequence off the file name. The title comes from the document's own `#` heading. `status` comes from the status map below, and `status_since` from the document's own `## Date` heading. `last_verified` is the date the pin was taken. **The sixth is a constant, and that is a finding.** `summary` is required on every `governed_document`, and this tradition supplies nothing that derives one. An ADR opens with a title and then a Context section, and it carries no one-sentence scent line. So all ten documents carry the same sentence. A corpus of this tradition cannot satisfy that facet honestly, and no rule reports the fact.

## The six Done-when boxes

**1. A fork exists and the commit is pinned by hash.** [`headwater-ai/inspect_evals`](https://github.com/headwater-ai/inspect_evals), and the pin is `360484a0…` wherever this document or the fixtures page cites it. The `n8n` fork under the same organization is the precedent.

**2. `docs/evaluations/inspect-evals-worked-example.md` exists and types a real slice.** This document, and the slice is the whole of `adr/`. No source prose was rewritten. 10 of 10 typed.

**3. The report names at least one finding.** Four classes below, and the one that matters most is a defect in this engine rather than in their writing.

**4. Every cross-link is checked for resolution.** Below, and the answer upstream is 15 of 15.

**5. One count of what their own stack already catches.** Below.

**6. The entry's doctrine cites this corpus.** [The `decision-record` doctrine](../taxonomies/decision-record/doctrine.md#the-worked-instance-corpora-and-what-criterion-4-now-reads) carries it, in both the authored copy and the vendored one that `tools/repo/library-index-fixtures.sh` holds byte-identical to it.

## The section contract holds in 10 of 10, and that is the result

The base `decision` kind requires Context, Decision and Consequences. All ten records carry all three as `##` headings, with no edit to any body. Every one also carries Status and Date, seven carry Considered Options, five carry Rationale and one carries Related Work.

**`diataxis-site` had to refuse all 4 of its 4 external documents on the same rule**, which is 11 of its 16 errors. The difference is what the two traditions constrain. Nygard's three sections are a convention that ADR authors actually write, and Diátaxis constrains a reader's purpose rather than a writer's headings.

That reading goes against the open finding in [13 — Open obligations](../spec/13-open-obligations.md) that a kind of a library entry cannot decline to state a section contract. **It does not close it.** That finding belongs to `diataxis-site`, and one counter-case from one tradition is not a ruling on the general question.

## Four of the ten statuses say something the state vocabulary cannot

The corpus writes its state in prose under a `## Status` heading. The runner maps that prose to `vocabularies.lifecycle_state` by a table on the fixtures page, and the map is a lossy one:

| `## Status` in the corpus | Records | Mapped state | What the map drops |
|---|---|---|---|
| `Accepted` | 6 of 10 | `current` | nothing |
| `Accepted — implementation deferred` | 3 of 10 (`0003`, `0005`, `0006`) | `current` | that the decision is not built |
| `Accepted — Stage 0→1 active, Stage 2+ deferred` | 1 of 10 (`0007`) | `current` | that one stage of four is live |

**#488 named `0007` alone, and the real figure is 4 of 10.** The five states are `draft`, `current`, `superseded`, `deprecated` and `discharged`, and none of them means "accepted, and not yet built".

**The sharp part is where the dropped clause belongs.** Three of the four say the same thing, "implementation deferred". This entry's own facet `waiting_on` carries the value `build`. Its guidance reads "the answer is known and somebody has to write the code or the prose". A real ADR log invented, in a status line, the exact distinction this entry added as a facet. `waiting_on` is required on `kinds.obligation_record` and declared on no other kind, so a decision record under this entry cannot state it.

**This change takes no declaration on that reading.** To widen `kinds.decision.facets` moves `bundle.yml`, which moves the lock and the package, and that argument belongs to a change of its own. [HW-OBL-0123](../obligations/0123-a-facet-that-applies-to-one-value-of-another-facet-has-nowhere-to-say-so.md) is the adjacent record: a facet that applies to one value of another facet has nowhere to say so. A status that means "accepted, and not yet built" is another instance of it.

## The links resolve upstream, and the engine does not read all of them

**The upstream answer first, because that is the question the fourth box asks.** Measured at the pin over the 10 files, with fenced code stripped: **43 inline markdown links, 27 absolute and 16 relative.** There are no reference-style links and no autolinks. One of the 16 relative links is a pure fragment. Of the remaining 15, **15 resolve against the upstream tree at `360484a0`**, and the one fragment resolves against its own file's `## Related Work` heading.

| Relative link target | Count | Resolves at the pin |
|---|---|---|
| Another ADR in `adr/` | 7 | 7 of 7 |
| `../internal/implementation-plans/external-asset-hosting.md` | 5 | 5 of 5 |
| `../docs/task-configurability.md` | 2 | 2 of 2 |
| `../.github/security/incident-response-runbook.md` | 1 | 1 of 1 |

So nothing in their `ruff` / `mdformat` / `markdownlint` stack resolves a link, and every link resolves anyway.

**A run inside the assembled corpus answers a different question.** 8 of the 15 point out of `adr/`, so a root that holds the 10 files alone cannot resolve them. Those are a property of lifting files out of a repository, recorded rather than repaired, and [the `diataxis-site` run record](../taxonomies/diataxis-site/fixtures/README.md#the-run-record-that-criterion-4-asks-for) already carries that distinction.

**The engine reported 4 of those 8, and the other 4 are a defect.** The 4 it missed are the four `../internal/implementation-plans/…` links inside a blockquote, at line 7 of `0003`, `0005`, `0006` and `0007`. The 4 it reported stand in a paragraph or a list item. A probe run on 2026-09-11 confirms the rule rather than the coincidence. One document carries three broken relative links, one in a paragraph, one in a list item and one in a blockquote. The run reports 2 of the 3, and the blockquote is the one it does not read.

**`link.path.unresolved` does not read a link inside a blockquote.** No fixture of this repository had ever put one there. A real corpus did on its first run, which is the whole argument for typing prose that was not written to pass. [13 — Open obligations](../spec/13-open-obligations.md) carries the finding.

## The cross-reference convention is mostly prose that no tool can follow

#488 describes a "Related ADRs" cross-link convention. **No file carries `Related ADRs` as a heading.** `0007` carries the literal words on one line, as a prose lead-in. Across the set there are 21 textual `ADR-00NN` references, of which 10 are each file's own identifier in its own `#` title. Of the 11 that point at another record, 7 are markdown links and 4 are bare text.

A cross-reference written as bare text is invisible to a link checker, to `markdownlint`, and to this engine alike. It is a weaker statement than #488 makes, and a cleaner one. The convention lives in the reader's head rather than in a form anything reads.

## What their own stack already catches, and what it does not

This is the fifth box, and the honest version of it is narrow. Their `.pre-commit-config.yaml` runs `ruff` over Python, `actionlint` and `zizmor` over workflows, and `mdformat` and `markdownlint-cli` over Markdown. Their `markdownlint.yaml` turns `line-length` and `descriptive-link-text` off, and nothing in the stack resolves a link.

| Of the 8 findings | Count | Would their stack catch it |
|---|---|---|
| `link.path.unresolved` | 4 | No. Nothing there resolves a link. These four are also a property of the trimmed root rather than of their prose |
| `voice.forbidden_construction` | 4 | No. No lexical linter reads a sentence for what tense it narrates in |

**So 0 of the 8 are findings their stack already raises, and 8 of the 8 are findings it cannot raise.** Of those 8, `headwater check --fix` repairs 0. The run prints `no finding of this run carries a patch`, because a dead link and a narrated sentence both need a rewrite.

**That is a count and not a claim of value.** The engine raised four warnings about prose in three of their ten documents, and each one is a judgment a maintainer may decline. What the count does say is that no finding here is a second copy of something they already run.

**The four warnings, in full.** The `declarative` voice that the base binds to `decision` forbids change narration and future intent. `0001` writes `previously`, `0009` writes `previously`, `0007` writes `used to`, and `0007` writes `will eventually`. The reading behind the rule is that a decision record states the position that holds, and leaves the change to the record that made it.

## What this does not settle

**It is one corpus of ten documents.** [#489](https://github.com/headwater-ai/headwater/issues/489) proposes a second, ten times the size, and [#491](https://github.com/headwater-ai/headwater/issues/491) one published by a bank. Each is separate work on the same criterion.

**Nothing was sent upstream.** [#495](https://github.com/headwater-ai/headwater/issues/495) is the work that contacts a project, and [#76](https://github.com/headwater-ai/headwater/issues/76) is the milestone it belongs to. This is a fork typed for this project's own evaluation, with no outside party involved.

**The criterion 5 question the entry waits on is untouched.** [HW-OBL-0185](../obligations/0185-whether-an-admitted-library-entry-may-require-a-bundle-that-admission-refuses.md) and [#520](https://github.com/headwater-ai/headwater/issues/520) stand as they were.

**No taxonomy declaration moved.** The `waiting_on` reading above is recorded and not acted on.
