# Fixtures for the standards-spec taxonomy — the n8n corpus

Seven real governing documents from [`n8n-io/n8n`](https://github.com/n8n-io/n8n), typed against this entry. Nothing here is invented. The [other fixtures of this entry](../README.md) hold a miniature corpus for an imaginary system called Beacon, which is what criterion 4 calls a *realistic* corpus. This one is what criterion 4 calls a *real* one.

This is the second corpus of the same repository at the same pin. The [design-spec fixture](../../../design-spec/fixtures/n8n/README.md) types n8n's architecture documents, and it shares this pin and this fork. It is a separate tree rather than a wider one, and the reason is mechanical: `corpus.root` is a single scalar string, n8n's architecture prose is under `packages/` and its review rules are under `.agents/`, and one root cannot reach both.

Every number below came from the run this file quotes. Nothing here is predicted.

## Modification notice

**These seven files are modified copies of n8n software.** The modification is the addition of one Headwater front-matter block at the top of each file, and nothing else. **The body of every copy, from the blank line after the closing `---` to the last byte, is identical to the pinned upstream file.** No word, no heading, no link, no spelling, no contraction and no line break was changed.

n8n's [Sustainable Use License](LICENSE.md) requires a prominent notice on a modified copy, and this section is it. The same license requires that anyone who receives any part of the software also receives a copy of the terms, so [`LICENSE.md`](LICENSE.md) beside this file is a verbatim copy of n8n's own, and not a link to it. It is a second copy rather than a link to the design-spec fixture's, because a second set of copied files is a second receipt of part of the software.

## The pin

| | |
|---|---|
| Fork | https://github.com/headwater-ai/n8n (forked 2026-09-01, default branch only) |
| Upstream | https://github.com/n8n-io/n8n |
| Branch | `master`. n8n's license states that content of branches other than `master` is not licensed, so every citation here names `master` and nothing else |
| Commit | `b0550cb3cb4d1752546a69056c55eccfb9111a12`, committed 2026-09-01T16:27:59Z |

**The pin is not caution, and the evidence for it is this corpus.** A survey on 2026-08-31 recorded 13 rule files across five directories. A survey on 2026-09-01 recorded 22 across six: a whole `db-migrations/` category of five governing documents arrived in one day. That jump is what the pin holds still, and reading it as one burst rather than as a trend is itself a measurement: the subtree is byte-for-byte identical between this pin and `master` at `1b87f4f003f6cbfe833e044751c344c23c6422e3`, committed later the same day.

## The seven files

Each row names the upstream path, which is also the path the copy sits at under `corpus/`.

| Upstream path at the pin | Bytes | Identifier |
|---|---|---|
| `.agents/review-rules/README.md` | 5,331 | `N8N-STD-review-rules` |
| `.agents/review-rules/testing/coverage.md` | 558 | `N8N-STD-coverage` |
| `.agents/review-rules/db-migrations/conventions-and-tests.md` | 1,430 | `N8N-STD-conventions-and-tests` |
| `.agents/review-rules/db-migrations/data-safety.md` | 1,056 | `N8N-STD-data-safety` |
| `.agents/review-rules/security/credentials-and-secrets.md` | 744 | `N8N-STD-credentials-and-secrets` |
| `.agents/review-rules/qa-dx/workflow-safety.md` | 2,316 | `N8N-STD-workflow-safety` |
| `.agents/review-rules/frontend/design-system.md` | 705 | `N8N-STD-design-system` |

The byte counts are of the upstream body, and the copies carry that many bytes plus their front-matter block.

Every one of the seven is a `standard`. Five of the six directories are represented, and `backend/` is not. Six are rule files and the seventh is the README that files them, which is here because a sweep finding may only name a classified document and the reach model this corpus asserts is restated in that README.

**Byte identity was verified mechanically, twice.** The front-matter block was stripped from each copy and the remainder diffed against the file at the pin: seven empty diffs. The file at the pin was then confirmed to be the pin, by comparing each `git hash-object` against the blob the fork's tree API reports at that commit: seven matches.

**None of the seven is under the Enterprise License.** n8n excludes a file with `.ee.` in its name and a file under a directory with `.ee` in its name from the Sustainable Use License. The tree at the pin holds 91 paths under `.agents/` and **zero** of them match `.ee` in any form. `db-migrations/` only talks about `packages/@n8n/db/`; the licensed artifact is the rule file, which is not under `packages/` at all. The check was run rather than assumed.

## How to run this corpus

`headwater check` reads one corpus root, and this tree sits inside a directory that this repository excludes from its own corpus. Assemble a scratch root out of three committed things and run the engine over it:

    ROOT=$(mktemp -d)
    cp -R docs/taxonomies/standards-spec/fixtures/n8n/corpus/.agents "$ROOT/.agents"
    cp -R docs/taxonomies/standards-spec/fixtures/n8n/.headwater "$ROOT/.headwater"
    cp -R packages "$ROOT/packages"
    headwater taxonomy resolve --root "$ROOT"
    headwater check --root "$ROOT" --no-cache --now 2026-09-01
    headwater sweep plan --root "$ROOT" --under .agents/review-rules

The entry sits outside the corpus root on purpose, the same reason the Beacon fixtures give. Nothing in CI runs this corpus.

## What the taxonomy needed before it could read one file

**This entry, exactly as it ships, types none of these documents.** Its shelf for `kinds.standard` is `standards` at `docs/standards/**`, and n8n keeps these files at `.agents/review-rules/`. Run the assembly above with the shelf removed from `overlay.yml` and the run reports **7 files under the corpus root, 0 typed, 7 untyped, 0 findings**, every row reading `no shelf pattern claims this path`, and `headwater check --strict` exits **0**. That is the design-spec fixture's first result arriving a second time from a second kind of the same repository.

**A leading-dot corpus root walks like any other.** `corpus.root: .agents` was the one plausible blocker before the run and it is not one: the census walker uses a plain directory read with no hidden-directory filter. Shelf patterns are matched against paths that carry the corpus root segment, so the shelf writes `.agents/review-rules/**` and not `review-rules/**`.

**No exclusion is needed, and the design-spec fixture's exclusion was forced.** That fixture roots at `packages`, which is also where a vendored taxonomy package is found, so it had to declare `exclude: packages/headwater-standard/**` and its census reports 101 excluded files that are not corpus content. With `root: .agents` the package sits outside the corpus root entirely. The exclusion was a property of where that kind's prose lives, not of scattered corpora in general.

**The shelf can be homogeneous here, and it could not there.** `kinds.design_spec` requires the facet `doc_type`, and a homogeneous shelf refuses a document that restates the kind its placement already states, so the design-spec fixture reports an error whichever way the front matter is written and settles for a heterogeneous shelf with one admitted kind — a shape its own README calls a contradiction. `kinds.standard` requires no facet and forbids `spec_layer`, so `homogeneous: true, kind: standard` resolves with nothing to declare. All three arms were run:

| The shelf, and the front matter | Result |
|---|---|
| `homogeneous: true`, `kind: standard` | 7 typed, 21 findings. This is what the fixture commits |
| `homogeneous: false`, `discriminator: spec_layer`, `spec_layer: standard` declared | 7 typed, 21 findings, identical |
| `homogeneous: false`, `discriminator: spec_layer`, the facet omitted | 0 typed, 7 untyped, 0 findings, strict exits 0 |

**The second row is worth a sentence, because the value it declares is illegal twice over and no rule says so.** `spec_layer` admits `functional_spec` and `technical_spec` and the arm writes `standard`, and `kinds.standard` forbids the facet outright. `facet.value.not_permitted` and `facet.required.missing` both instantiate seven times and report nothing — not because the run skips them, but because both pass silently, for two distinct reasons. `facet_value.rs` filters a forbidden facet out of the kind's admitted set before generation runs at all, so `facet.value.not_permitted` never sees the value. `shape.rs`'s required-facet computation removes a forbidden facet from what a kind owes, so `facet.required.missing` never owes one either. (`shelf.placement_is_primary` is the rule that instantiates seven times with the *"the shelf is heterogeneous, so the discriminator is the gap metadata fills, and kind resolution already read it"* reason — a third, unrelated rule, not either facet rule.) The engine states both rulings deliberately, and a stronger fact follows from them: **no rule in this engine reports a forbidden facet's mere presence at all.** A heterogeneous shelf's discriminator answers to no facet declaration, `forbid` included, and there is no check anywhere for a forbidden facet showing up regardless of shelf shape.

**The overlay declares four namespaces and the corpus uses one.** `taxonomy validate` refuses a resolved scheme that carries none, and it refuses every scheme of the closure rather than every scheme the corpus mints under. So a corpus of standards alone declares a namespace for `functional_spec_id` and `technical_spec_id`, which no document here is, and for the base's `decision_id`, which nothing here selects a kind for. Only `standard_id` is live. The [Beacon fixtures](../README.md#how-to-run-this-corpus) record the same four, and this run confirms the cost is paid by a corpus that holds one of the three kinds as much as by one that holds all three.

**And these identifiers are usable, which the design-spec ones are not.** That entry declares no identifier scheme for any of its five kinds, so its four typed documents each report `identifier.unusable` and the graph reads **0 nodes**. This entry mints one scheme per kind, so all seven documents here carry an `N8N-STD-{slug}` identifier, the graph reads **7 nodes**, `identifier.unusable` instantiates seven times and reports nothing, and both ends of a proposed edge can be named. Same library, two entries, and one of them can be pointed at.

## What a run reports

`headwater taxonomy resolve` then `headwater check --no-cache --now 2026-09-01` over the assembled root, with the committed `.headwater/`: **7 files under the corpus root, 7 typed, 0 excluded, 7 checked, 93 check instances, 21 findings, all 21 of them errors.** The census reads 7 `standard`. The graph reads 7 nodes, 0 declared edge halves, and 0 prose links that did not resolve. `headwater check --strict` exits 1.

Every count moves only when this corpus moves. There is no excluded count to drift with the vendored package.

**Three findings per document, and the same three on every one.** `section.required.missing`, an error, under `OB-SECT-1`, once for each of `Scope`, `Requirements` and `Conformance`:

    `standard` requires the section `Conformance`, and no heading of this document says so

**This is the first time in the n8n work that this rule instantiated at all.** The design-spec fixture recorded that it "never instantiated, because this entry's `design_spec` declares no required sections." Here it produces every finding of the run.

**The finding is sharper than a heading count, and it is the result this corpus was typed for.** Not one heading at any level of any of the 23 upstream files carries the word `Scope`, `Requirements` or `Conformance`. The corpus is not missing the content. Every one of the 22 rule files opens with a line `Applies to: …`, which is the scope, written as a labeled paragraph; 16 of them use the imperative `Flag` somewhere to state what to flag, though only six write it as a literal `Flag:` block. The contract is lexical over headings and the tradition writes the same content as a labeled paragraph, so the rule reports 100% absence over 100% presence. `Conformance` is the honest third: no rule file says how conformance is judged, because the AI reviewer judges it, and that section is genuinely absent rather than differently written.

**The expectation on `standard-applied` is not due, and the run says so rather than staying silent.** `kinds.standard` expects a `regulates` edge to a `functional_spec` within 180 days of `state_entered`, at severity `warn`. `relation.participation.overdue` instantiates seven times and reports nothing, because `status_since` is the pin date and `--now` is the pin date. The window opens on 2026-09-01 and its deadline falls on 2027-02-28. The rule fires only once the clock passes that date, so a run of this fixture at `--now 2027-03-01` or later reports seven warnings that this run does not, and neither reading is about n8n.

## The sweep

**This is the first coherence sweep run and recorded anywhere in this repository.** There was no committed briefing, no return file and no report before this one. Two documents state the absence outright: [spec 4](../../../../spec/04-assurance-model.md) records that this register declares no coherence obligation, so a sweep discharges none, and [HW-OBL-0114](../../../../obligations/0114-a-control-that-names-a-sweep-marks-its-obligation-verified-with-nothing-run.md) says the same from the obligation's side. All three artifacts are committed under [`sweep/`](sweep/):

| File | What it is |
|---|---|
| [`sweep/briefing.md`](sweep/briefing.md) | the bytes `headwater sweep plan --under .agents/review-rules` printed |
| [`sweep/findings.yml`](sweep/findings.yml) | what a model wrote back after reading all 24 documents |
| [`sweep/report.txt`](sweep/report.txt) | the bytes `headwater sweep report` printed over that file |

The briefing is committed because `plan` is byte-reproducible: two runs over one tree write the same bytes, so the briefing is a measurement in the same sense as the check counts above. The return file is committed because it is the only thing that lets a later reader re-run `report` and get the same verdict.

**What the sweep confirms, and what it cannot.** `report` confirmed four things: the return file names the lock this tree carries, every path it names is a typed row of the census, every quotation is really in the document it is attributed to, and no proposed edge restates one the graph already declares. It confirms nothing about whether the reading is right. **3 findings carried, 0 refused, of 3 the file held.**

**Finding 1, `undefined_concept`.** The README's instruction to an author of a new rule opens "Pick the level of reach first." The term is defined nowhere in the slice. Its only definition is a header comment in `cubic.yaml`, and see below for why that file can never appear here.

**Finding 2, `undeclared_conflict`.** The README rules that "A directory maps to one agent unless, like `testing/`, the policy is identical across domains — then it is one file listed in several agents' `file_paths`, never a copy per directory." `testing/coverage.md` holds the shared coverage policy. `db-migrations/conventions-and-tests.md` holds a second coverage policy under its own `## Tests` heading. **A defender would call this specialization rather than duplication**, since the README's own test is "word-for-word the same" and a migration test is not word-for-word a service-method test. Both readings are honest and the sampler adjudicates neither, which is what a sampler is for.

**Finding 3, `quiet_supersession`.** The README names two agents that deliberately do not link `testing/`. Read against `cubic.yaml`, three do not: Security, QA & DX, and the DB migrations agent, whose `file_paths` holds only its own five files. The README's own Layout table is already consistent with three, mapping `testing/` to "Backend + Frontend". So the table was updated when `db-migrations/` arrived and the exception prose two paragraphs below it was not, and the rationale that prose gives — "coverage nagging on a credential fix or a Dockerfile" — does not describe a migration at all. **This is a genuine corpus defect, and it is causally tied to the drift the pin exists to capture.**

**The two exceptions the README declares are exceptions and not contradictions.** `testing/coverage.md` is one file listed in two agents' `file_paths`, Backend and Frontend, exactly as the model declares. Security and QA & DX not linking it is stated with a reason. Neither is a finding, and the answer to the question this fixture was typed for is that the three-level reach model holds for the two cases the corpus declares and is broken by a third the corpus never noticed.

### Three things the sweep could not do, each of which is a result

**`cubic.yaml` can never be a finding path.** `report`'s membership test requires a typed row of the census, and `cubic.yaml` is a configuration file that no taxonomy here types. So the document that *asserts* the three-level model is structurally unable to appear in the sweep that checks it, and every quotation from it lives in a finding's `message` where nothing verifies it. The sampler can confirm a contradiction between two governed documents and cannot reach the configuration file that governs them.

**`conflicts_with` cannot join two standards, and the intake carried the proposal anyway.** The base package declares it `from: [decision] to: [decision]`. Finding 2 proposes `N8N-STD-coverage conflicts_with N8N-STD-conventions-and-tests` deliberately, to find out. The intake's four confirmations do not include endpoint-kind validation, so the proposal **passed**, and `report` printed the front matter to write. Declaring that front matter and re-running `headwater check` reports **2 `relation.endpoint.not_permitted` errors**, one for each end:

    `conflicts_with` declares `from: [decision]`, and `N8N-STD-review-rules` at that end has the kind `standard`

So a sweep proposes an edge that the gate then refuses, and nothing between the two says so. This is a finding about the library, not about n8n.

**And the front matter `report` prints names the wrong document.** The proposal is `from: N8N-STD-coverage`, and the line beneath it reads "to declare it, in the front matter of `.agents/review-rules/README.md`", which is the first path in the finding's `documents` list. A reader who follows the instruction declares `N8N-STD-review-rules conflicts_with N8N-STD-conventions-and-tests`, which is not the edge the sweep proposed. Both arms were run and both report the same 23 findings, so the gate catches the substitution for the wrong reason and never names it.

**`unwritten_section` does not apply.** That class is for a required heading present over empty prose. Here the headings are absent entirely, which `section.required.missing` already reports 21 times, and stretching the class would be the sampler restating a check.

### `class: coherence` gets its first member

`obligations.OB-SS-1` of this entry carries `class: coherence`. Spec 4 and HW-OBL-0114 both record that this repository's own register has no coherence-class obligation, so a sweep here discharges nothing. Resolving this entry over this corpus gives that class its first member in any real corpus of this library. It still discharges nothing: OB-SS-1's disposition is `unverifiable`, and no control names a `sweep:` mechanism. The gap is now visible rather than hypothetical, and this fixture is where it is visible.

## What this repository's own house regime would have reported

The measurement below is a probe and not a declaration, on the precedent the [brd-prd fixtures](../../../brd-prd/fixtures/README.md#what-the-bases-voice-regime-would-have-reported) set. Append `regimes.language.ste_house` from `.headwater/overlay.yml` of this repository to the fixture overlay, add `kinds.standard.language: ste_house`, and change nothing else.

The run reports **143 findings, 114 error and 29 warn**, against 21 findings and 21 errors before it. The 122 extra findings are:

| Rule | Count | Severity |
|---|---|---|
| `language.source_form.not_met` | 87 | error |
| `language.controlled.not_met`, a contraction | 5 | error |
| `language.controlled.not_met`, a British spelling | 1 | error |
| `language.controlled.not_met`, a semicolon in running prose | 19 | warn |
| `language.controlled.not_met`, a sentence past 25 words | 10 | warn |
| `language.retired_term.used` | 0 | — |

Per document:

| Document | Under the entry alone | Under the entry plus `ste_house` |
|---|---|---|
| `.agents/review-rules/README.md` | 3 | 58 |
| `.agents/review-rules/qa-dx/workflow-safety.md` | 3 | 25 |
| `.agents/review-rules/db-migrations/conventions-and-tests.md` | 3 | 19 |
| `.agents/review-rules/db-migrations/data-safety.md` | 3 | 17 |
| `.agents/review-rules/frontend/design-system.md` | 3 | 10 |
| `.agents/review-rules/security/credentials-and-secrets.md` | 3 | 7 |
| `.agents/review-rules/testing/coverage.md` | 3 | 7 |

**Three things in that table are worth more than the total.**

**The hard wraps are spread across all seven documents, and in the design-spec corpus they were not.** There, 136 of 136 came from two documents of four, and the other two were written one line per paragraph already — a house rule that read as a verdict on a project turned out to be a verdict on the editor two of its authors used. Here every rule file is hard-wrapped at about 80 columns and every one of the seven reports between 2 and 42. Same repository, same pin, one editorial convention per corpus.

**This is a direct correction to what the design-spec fixture recorded about British spelling, in both directions.** That fixture reported "there is no British spelling anywhere in the four documents". There are four in this one — `behaviour`, `defence`, `colours` and `denormalised` — and three of them are in the typed set. **The rule reports one.** The engine's spelling table is closed at 24 words and holds `behaviour` and not `defence`, `colour` or `denormalise`. So the corpus-level claim and the rule-level claim differ by a factor of three, and only the second is what a run measures.

**Six findings of 143 are mechanically fixable, against one of 166 in the design-spec corpus.** `headwater check --fix` writes a British spelling, a contraction whose expansion is one word, a retired term that names a replacement, and a missing reciprocal link. Five contractions and one spelling are in that set, and the report marks each with `fix (mechanical)`. The 87 hard wraps are mechanical to a reader and carry no patch. **Nothing here was ever run with `--fix`.**

## What ages, and what does not

**The pin does not move and the upstream does.** Every count above is of the seven blobs at `b0550cb`. Re-run the assembly at a later commit of `master` and every number is a different measurement.

**One count is a function of the clock rather than of the corpus.** `relation.participation.overdue` reports nothing at `--now 2026-09-01` and seven warnings from `--now 2027-03-01` (the day after the 180-day deadline of 2027-02-28 passes). Both are correct and neither is about n8n.

**Two findings are about declarations rather than about documents.** The 21 `section.required.missing` errors stop the day this entry stops requiring headings a labeled paragraph could satisfy, and that is a change to the entry rather than to n8n. The sweep's refused edge stops the day `conflicts_with` admits a kind that is not a decision. Both are recorded in [the evaluation](../../../../evaluations/n8n-worked-example.md) as findings against the library.
