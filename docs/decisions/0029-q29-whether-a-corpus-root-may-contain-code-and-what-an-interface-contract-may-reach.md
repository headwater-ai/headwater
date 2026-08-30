---
id: HW-DR-0029
status: current
status_since: 2026-08-30
summary: The corpus root stays `docs`, because a path is corpus content only where a file with no front matter is a defect, and 185 of the 186 Markdown files under `engine/` exist to be defective. An interface contract lives inside the root and reaches a crate by a `governs` edge that binds on existence alone.
last_verified: 2026-08-30
title: "Q29 — Whether a corpus root may contain code, and what an interface contract may reach"
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  governs:
    - .headwater/taxonomy.yml
    - engine/README.md
---

# Q29 — Whether a corpus root may contain code, and what an interface contract may reach

## Context

`.headwater/taxonomy.yml` declares `corpus.root: docs` and excludes `docs/taxonomies/**`. The census walks that root and gives every file under it an outcome. Nothing outside the root is walked at all.

[HW-EVAL-specifying-the-engine](../evaluations/specifying-the-engine.md) finding 2 names what follows. The engine "is the one artifact in this repository that governs nothing and is governed by nothing". [#252](https://github.com/headwater-ai/headwater/issues/252) proposes an `interface_contract` document for each verb of that engine. [#253](https://github.com/headwater-ai/headwater/issues/253) asks where such a document lives and what it may reach.

The question is structural rather than editorial. [Spec 5 stop rule 3](../spec/05-ai-integration.md#the-stop-rules) forbids the invention of a shelf or a kind in place. A corpus root is the same class of fact. A root settled inside an implementation diff is a root settled by precedent.

**The specification already puts five classes of file outside the root, and not one of them is code.** [Spec 3](../spec/03-authoring-and-lifecycle.md#templates-and-scaffolding) puts a taxonomy source outside it, and [spec 3](../spec/03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) puts the capture-cost store outside it. [Spec 5](../spec/05-ai-integration.md#the-hook-contract-and-what-a-hook-cannot-bind) puts `.claude/hooks/` outside it, and [spec 5](../spec/05-ai-integration.md#how-a-skill-reaches-an-agent-and-what-nothing-does) puts a skill outside it. [Spec 6](../spec/06-engine-architecture.md#cli) puts the probe budget outside it. Two of the five state the same three consequences. No census row covers the file, no language regime binds it, and no rule reads it. The hooks passage states two of the three and says nothing about a language regime. The skill passage states one. It raises the language regime as a reason not to move a skill inside, rather than as a consequence of its position. The probe-budget passage states none of the three and gives a different test, which the ruling below has to answer.

**The exclusion this corpus already declares carries the test that decides the question.** The reason on `docs/taxonomies/**` in `.headwater/taxonomy.yml` ends with one sentence. "A fixture corpus under an entry is package content twice over, because it is a corpus that another root is meant to walk." That exclusion covers 36 files. 23 of the 36 sit under a `fixtures/` directory, and 21 of those 23 are the corpus itself. That is 10 files for one entry and 11 for the other. The other two are each a `fixtures/README.md` that documents the set, which is prose a person reads.

**The engine tree is that same sentence at eight times the scale.** `git ls-files engine` reports 608 files on this tree. 186 of them are Markdown. One is `engine/README.md` and the other 185 are inputs to the suite. `engine/crates/census/fixtures/walk/` alone holds `no-front-matter.md`, `unterminated.md`, `invalid-utf8.md` and `not-a-mapping.md`, which the census walker exists to report. [Spec 12](../spec/12-check-layer.md#the-correctness-roots) names that tree as the standing test of the walker.

**The mechanism a narrow reading needs is bound, and it already reaches the engine.** `headwater check` reports 25 distinct `code_path` anchor targets over 29 bindings on the tree this record ships on. Ten of the 25 sit under `engine/` and nine under `.claude/`. One of the ten is an edge this record declares, which is `engine/README.md`. The record's other edge names `.headwater/taxonomy.yml` and sits under neither. The evaluation reports 20 targets, and a claim has to say whether it counts a target or a binding, because those are different denominators.

## Decision

**A path is corpus content when a file arriving there with no front matter is a defect, and no code directory answers that test.** The corpus root of this repository stays `docs`. Four reasons follow, and the first two are measurements rather than arguments.

**First, widening the root to `engine/` admits 185 files that exist in order to be wrong.** The two trees do not compare.

Both rows count the tree this record ships on, which holds this record and the two obligation records it files. The `docs/` row is the census figure of the same run.

| tree | files | Markdown | Markdown that is an input rather than prose of this corpus | Markdown a person reads for what it says |
|---|---|---|---|---|
| `docs/` | 241 | 237 | 21, and every one of them is excluded by name | 216 |
| `engine/` | 608 | 186 | 185 | 1 |

Under a widened root, `engine/crates/census/fixtures/walk/spec/no-front-matter.md` becomes a document of this corpus with no front matter. The census reports it as untyped, against the walker that the file exists to prove. The remedy is an exclusion for every fixture directory of every one of the 22 crates, which is a root defined by what it removes.

**Second, the reach the narrow reading needs is exercised today, by the documents that raise this question.** [Spec 5](../spec/05-ai-integration.md) declares `governs` onto four hook scripts and three skill files. [Q25](../spec/09-decisions.md#q25--where-the-namespace-goes-in-an-identifier-and-who-declares-it) declares it onto `packages/headwater-standard/taxonomy.yml`. [Q26](../spec/09-decisions.md#q26--whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime) declares it onto three source files of the check layer and the resolver. So a document inside the root that governs a file outside it is the shipped shape, in ten places under `engine/` and nine under `.claude/`.

**Third, widening buys one file, and it buys nothing for the crate beside it.** `engine/README.md` is the only Markdown under `engine/` that a person reads for what it says. Every `.rs` file under a widened root takes the census outcome "not a document", which is the outcome it takes today by not being walked. So the reading that widens the root types one file and leaves 180 source files exactly where the narrow reading leaves them.

**Fourth, an anchor is a name and never a subject, and this ruling concedes that rather than hiding it.** `SourceTree::resolve` in `engine/crates/graph/src/anchors.rs` calls `Path::exists` and reads no byte of the file it binds. So a contract may name `engine/crates/check/src/runner.rs`, and nothing holds the bytes at that path to what the contract says about them. Widening the root does not repair that either, because a `.rs` file under a widened root is still not a document.

**What an `interface_contract` may reach.** It lives on a shelf under `docs/`, inside the root, under the language regime and the section contract its kind declares. It reaches the engine by a `governs` edge onto a `code_path`. The edge binds when the path exists on the tree that holds the corpus. So it may name a crate directory, a source file, `engine/Cargo.toml` or `engine/README.md`. It may assert nothing about the bytes at that path that a rule reads, because no rule reads them.

**The sentence applied to seven paths, four of which [#253](https://github.com/headwater-ai/headwater/issues/253) does not name.** Each row states the test, the verdict, and what already stands at that path.

| path | a file arriving there with no front matter | verdict | what already stands |
|---|---|---|---|
| `engine/README.md` | ordinary, beside `Cargo.toml` and 180 source files | outside | typed by nothing, reachable as an anchor, and its crate table is [#188](https://github.com/headwater-ai/headwater/issues/188) |
| `.claude/hooks/*.sh` | ordinary | outside | spec 5 states it, and four of the scripts are anchor targets of spec 5 |
| `packages/headwater-standard/taxonomy.yml` | ordinary | outside | an anchor target of Q25, and a taxonomy source that spec 3 puts outside the root |
| `docs/taxonomies/decision-record/bundle.yml` | ordinary, and the tree beside it is a corpus another root walks | outside | excluded by name, and the run reports the anchor as sitting inside that exclusion |
| `engine/crates/census/fixtures/walk/spec/no-front-matter.md` | the whole point of the file | outside | the standing fixture of the census walker |
| `.headwater/capture-cost.jsonl` | ordinary | outside | spec 3 states it |
| `docs/probe-runs/<name>.md` | a defect, because the shelf types a transcript | inside | spec 5 rules a transcript corpus content, on the test of what regenerates it |

The last row is the one that shows the test is not a restatement of the root path. A probe transcript is machine-recorded rather than authored, and it is corpus content.

**What reports such a file, measured rather than assumed.** A Markdown file with no front matter was put on `docs/probe-runs/` in a scratch copy of this corpus. The census counts it and names it, and `headwater conformance` moves `corpus.classified` from a two-file gap to a three-file gap and names it there too. No check finding moves at all, the count holds at 523, and `headwater check --strict` exits 0. So "a defect" above is what this corpus declares about such a file, and the enforcement is a report rather than a refusal. That is the shape [OB-COV-1](../../packages/headwater-standard/taxonomy.yml) asks for. Its statement is that every file is classified **or reported as unclassifiable**, and the control that discharges it produces facts rather than findings. [HW-OBL-0129](../obligations/0129-spec-12-calls-two-phase-a-outcomes-structural-findings-and-the-engine-emits-none.md) records the one place that reading is not written down.

**This corpus already carries a second test, and this ruling says how the two stand.** [Spec 5](../spec/05-ai-integration.md#a-probe-result-is-citable-because-each-of-its-three-inputs-is-a-committed-artifact) states it in one sentence. "The test is whether anything regenerates the artifact from a committed source." [Spec 6](../spec/06-engine-architecture.md#cli) then puts the probe budget outside the root on that same test, so it is cited and used twice. The `docs/probe-runs/` row above reaches its verdict by it.

The two agree on every path either one was written for. A probe result and the transcript under it are inside. The capture-cost store and the probe budget are outside.

**They part on two paths, and the older sentence is the one that fails.** `.headwater/taxonomy.lock` is regenerated from a committed source by `taxonomy resolve`, and `taxonomy resolve --check` holds it. `.headwater/corpus.json` is written by `headwater generate`, and `generate --check` holds it. Both are regenerated from a committed source, both sit outside the root, and neither is corpus content. Read as a test of membership, the older sentence pulls both inside. The test above puts both outside, because a file arriving in `.headwater/` with no front matter is no defect of anything.

**So the regeneration test is not a membership test, and this record fixes its reading.** It decides, for an artifact whose kind a shelf already declares, whether that artifact is committed content or a fact about a run that ended. Spec 5 asks it of a probe result against a capture-cost reading, which are two artifacts of settled kinds. The test above is asked one step earlier, of a directory, before any kind is in play. Where a later reader has to choose, the test above governs, and nothing in spec 5 or spec 6 is edited to say so.

## Consequences

**What an author meets.** An `interface_contract` is authored under `docs/`, on a shelf the taxonomy declares, and it names its subject by a `governs` edge. `headwater check` reports the edge as an anchor binding and reports nothing about the file at the other end. A contract whose subject is deleted reports an unresolved anchor, which is the one thing the resolver can see.

**What remains unanswered about finding 2, and it is filed rather than left here.** Nothing holds a crate to having a contract. A participation expectation runs between two documents, and an anchor target is no node with a kind. So "every verb of this engine has a contract" is inexpressible as a check under this ruling. [HW-OBL-0128](../obligations/0128-nothing-holds-a-crate-to-having-a-contract-under-a-root-that-excludes-it.md) records the gap and states what would discharge it. The candidate the record names is a projection whose rows are derived from a path outside the corpus root. [Spec 6](../spec/06-engine-architecture.md#projections) does not permit that today, and it is the shape [#257](https://github.com/headwater-ai/headwater/issues/257) asks for.

**What this ruling gives up.** Two things, and both are the same asymmetry read from two ends. `engine/README.md` stays typed by nothing, so no section contract and no language regime reads its 293 lines. And a crate with no contract is invisible to every rule, so an undescribed verb costs a reader everything and costs a run nothing.

**What reopens this, and each condition is an event.**

The first is an adopter whose source files each carry a header block that the adopter's own toolchain requires. A regulated corpus under a safety standard makes a source file with no header a defect. A repository with a license-header gate does the same. The test above then admits that source tree, and the ruling has to be argued against a real corpus rather than against this one.

The second is the first check-layer rule whose subject is the bytes of an anchor target. The reading exists in this tree and it is not a rule. `engine/crates/check/src/fragment.rs` holds a `comment_links` module that walks the comments of every `.rs` file of the engine and resolves their links through the corpus slugger. It is a test of the engine's own suite, and its own comment states that nothing else reads a `.rs` comment. On the day that reading becomes a rule a corpus runs, an anchor stops being a name. The fourth reason above then stops holding.

**What this record does not settle.** Whether an agent may write the acceptance stamp of a document it drafted is [HW-OBL-0108](../obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md). Whether `warrant: proposed` is a value of that facet at all is [HW-OBL-0125](../obligations/0125-nine-documents-state-a-warrant-the-closed-set-does-not-hold-and-no-check-reads-one.md). Which kind, shelf, facet and relation an `interface_contract` declares is [#254](https://github.com/headwater-ai/headwater/issues/254), and this record declares none of them.
