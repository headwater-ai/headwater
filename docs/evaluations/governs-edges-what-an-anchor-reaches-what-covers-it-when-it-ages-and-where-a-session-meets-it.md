---
id: HW-EVAL-governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it
status: current
status_since: 2026-09-18
summary: "A governs edge reaches one file by equality, 53 of 374 documents declare one, and nothing reports a governed file that changed. One ruling and four designs close the gaps."
last_verified: 2026-09-18
title: "Governs edges: what an anchor reaches, what covers it, when it ages, and where a session meets it"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-fable-5-1
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-ai-integration
    - HW-SPEC-taxonomy-model
    - HW-OBL-0104
    - HW-OBL-0105
    - HW-OBL-0117
  governs:
    - .claude/hooks/write.sh
    - .claude/hooks/intent.sh
---

# Governs edges: what an anchor reaches, what covers it, when it ages, and where a session meets it

[Spec 5](../spec/05-ai-integration.md#write-time-hooks) makes one promise about code. A document can declare that it governs a path, and an edit to that path raises a prompt that names the document. [Spec 2](../spec/02-taxonomy-model.md#the-decision-relation-vocabulary) says that promise is why the base taxonomy enables `governs` at all, because without it a corpus has no edge to code. The [first-run walkthrough](default-taxonomy-first-run.md) calls impact detection the most valuable thing the corpus does for a coding agent. This evaluation measures how much of that promise the engine keeps on 2026-09-18. It measures on one change that exposed the gap, and it carries the designs that close it.

The reader outside this repository is an adopter who runs agents against a corpus of their own. That adopter wants the corpus to reach the agent when the agent touches the code the corpus describes. The reader inside it is the build order, whose agents change hooks and scripts that eleven documents describe and nothing governs.

## The case that exposed it

[#946](https://github.com/headwater-ai/headwater/pull/946) changed four hooks under `.githooks/` and left six prose files stating the behavior it replaced. They are `CLAUDE.md`, `DEVELOPING.md`, `.claude/skills/hw-run-policy/SKILL.md`, `.claude/agents/hw-build.md`, [the isolation how-to](../how-to/diagnose-an-isolation-failure.md) and [the isolation evaluation](one-clone-many-agents-how-this-repository-isolates-the-sessions-that-build-it.md). A later session found the drift by reading, and [#948](https://github.com/headwater-ai/headwater/pull/948) corrected all six.

Nothing in the engine or the harness could have named those six at the time of the change. `.githooks/pre-commit` is named in the prose of eleven documents. A route over its path reports `names the anchor` with no governing document under it, because no document declares a `governs` edge onto it. Four of the six stale files are not documents of this corpus at all, so they can declare nothing. The agent built for this report, `.claude/agents/headwater-maintainer.md`, is dispatched by no stage of the build order. `hw-build.md`, `hw-adjudicate.md`, `next.md` and `next-run.md` name it nowhere. So the one mechanism that reads a change for the documents it falsifies runs only when a person asks.

## What the engine does today, measured

**Reach.** `SourceTree::resolve` in `engine/crates/graph/src/anchors.rs` normalizes the raw value of a `code_path` anchor, asks whether that one path exists, and runs the corpus exclusions over it. `governing_docs_for_path` in `engine/crates/query/src/lib.rs` compares the path it is asked about against the normalized string of each edge. Both are equality. A directory anchor binds, because the directory exists, and matches no file under it. [HW-OBL-0104](../obligations/0104-a-governs-edge-reaches-the-path-it-names-and-nothing.md) recorded this on 2026-08-13 and has waited on a ruling since.

**Capture.** `headwater taxonomy audit` on 2026-09-18 reports `governs` at 144 halves from 53 declaring documents. That is 14.2 percent of 374 eligible documents. `traces_to` stands at 304 halves from 218 documents, at 58.3 percent. Twenty-two interface contracts each name `engine/crates/cli/src/lib.rs` and `engine/crates/cli/src/main.rs` by hand. So 44 of the 144 halves are one fact written 22 times. The audit computes these numbers on demand and commits none of them. [HW-OBL-0001](../obligations/0001-the-promotion-fix-has-no-reading-of-the-assisted-fraction.md) quotes 0.6 percent, which was true on its date, and nothing moved the figure since.

**What the prose names and the front matter does not.** A scan of 350 documents across ten shelves, taken for this evaluation, found 564 pairs of a document and a path. In each pair the document names a repository path in backticks and declares no edge onto it. The pairs cover 449 files and 115 directories across 178 documents. The most-named paths are `.headwater/taxonomy.lock` in 31 documents, and `.headwater/corpus.json`, `.headwater/overlay.yml` and `.headwater/taxonomy.yml` in 20 each. `CLAUDE.md` follows in 16, and `.githooks/pre-commit` in 11. Most single mentions in a decision are a citation and not a governance claim. So the count is an upper bound on the backfill and not its size. It is also the denominator that HW-OBL-0104 says the corpus lacks, taken by hand once.

**Aging.** Nothing reports that a governed file changed after the document that governs it was last verified. `relation.target.suspect` in `engine/crates/check/src/suspect.rs` compares a `verified_revision` on the edge against the revision the resolver reports. It reads only relations with `created_by: import`. `SourceTree` reports no revision at all. `engine/crates/check/src/change.rs` states that the check layer walks no history and runs no version control tool. This evaluation keeps that rule.

**The four moments.** Spec 5 names intent, read, write and review. The harness hooks under `.claude/hooks/` hold two of them for code. `intent.sh` runs `headwater route` over every prompt, and a route names the documents that govern a path the prompt spells out. `write.sh` runs on `PostToolUse` for `Edit` and `Write`. It routes the edited path and prints the governing documents after the edit landed. No position runs when an agent reads a governed file. No position speaks when nothing governs a path, so an ungoverned file and a silent hook read the same. The review position runs the commit gate, which reports an unresolved target and nothing about a changed one.

| moment | what runs today | what it reaches | what it misses |
|---|---|---|---|
| intent | `intent.sh`, `headwater route` over the prompt | documents that govern a path the prompt names | a path the prompt describes and does not spell |
| read | nothing for code | nothing | every governed file an agent opens |
| write | `write.sh`, `PostToolUse` on `Edit` and `Write` | documents that govern the exact path, after the edit | a file under a governed directory, a path nothing governs, a write through `Bash` |
| review | `review.sh`, the commit gate | an edge onto a path that no longer exists | an edge onto a path whose content changed |

## What governs means, and what it does not

A governed file is opaque to the engine. `source-tree` reads whether an entry exists, and no rule reads its bytes for meaning. That stays true under every design below. A digest of the bytes is a fact about change and not about content. It is the one thing this evaluation asks the resolver to read.

The edge runs one way. A document declares that it constrains a file, and an untyped file can declare nothing. So the four stale files of #946 that live outside `docs/` are reached as targets and never as sources. A governed document that restates a rule in `CLAUDE.md` governs `CLAUDE.md`, and an edit to either surfaces the same document. That is the only shape the taxonomy offers for prose outside the corpus root, and this evaluation uses it.

## Four questions, and a design for each

### Reach: an anchor is a pattern, and one edge names a set

[HW-DR-0074](../decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md) is the ruling. The raw value of a `code_path` anchor is a pattern in `headwater_meta::pattern`, the four-form language that shelves and exclusions already use. A bare path matches one entry. `dir/**` matches a subtree. A list of patterns is one anchor over the union of what its members match, for a set that no single pattern names. An edge binds when every pattern matches at least one entry and is unresolved otherwise. A query matches a path against the patterns.

The alternatives each cost more than they return. Implicit descent for a bare directory changes the meaning of the 144 halves on the tree, and a prefix test admits `.claude/hooks-disabled` for `.claude/hooks`. A second relation for a subtree is two relations for one fact. A regular expression admits more than any corpus uses. Naming every file is the state today, and the 22 contracts that each name the same two files are its price.

### Coverage: two numbers with two denominators

The audit already reports the document side: the share of eligible documents that declare at least one `governs` half. The tree side has no reading, and it is the number an adopter asks for. The design adds one declaration and one report.

The declaration is a scope. The overlay names the patterns an author expects to be governed, in the same language as an anchor. This repository's first scope is `engine/crates/**/src/**/*.rs`, `.githooks/**`, `.claude/hooks/**`, `.claude/agents/**`, `.claude/skills/**`, `tools/**` and `site/**`. A scope is a claim by the corpus about itself, and a corpus that declares none reads as it does today.

The report is the fraction of entries in scope that at least one `governs` pattern matches, per scope pattern and in total. The ungoverned entries are listed with it. It lives in `headwater taxonomy audit` beside the relation readings, because that verb already computes the document side. It is computed and never committed, because a committed derived scalar merges wrong under two branches ([HW-DR-0049](../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md)). CI prints it on every pull request. A page for a reader who runs no CLI is [#505](https://github.com/headwater-ai/headwater/issues/505).

A check that errors on an ungoverned file is refused. The remedy is a judgment about which document governs. So under the [fixability](../spec/12-check-layer.md#fixability) bar the finding is advisory at most. An advisory finding per new file is noise that trains an author to ignore the report. One advisory finding per scope pattern, carrying the count and the list, is the shape that stays readable.

### Aging: an edge records a digest of what it reached

The suspect rule is the right mechanism and it reads the wrong set. The design extends `relation.target.suspect` from relations with `created_by: import` to every edge whose resolver reports a revision. It gives `source-tree` a revision to report: a digest over the matched entries, as the sorted list of relative path and content hash. That is the shape of a git tree object, computed from the working tree without git, so the check layer still walks no history.

An author records the digest by verifying the document. `headwater check --fix` writes `verified_revision` onto a `governs` edge whose declaring document carries a `last_verified` at or after the run's clock. That is the mechanical half. The finding on a mismatch is advisory. It names the document, the pattern and the count of entries that changed, because the remedy is to reread. Under this design the four hooks that #946 changed make the isolation evaluation's edge suspect on the next `headwater check`. The commit gate prints every finding.

Three alternatives were measured against this repository's own history and refused. A modification time is not a fact about content, and a fresh worktree gives every file a new one. A `git log` comparison puts a version control tool inside a verdict, which `change.rs` forbids for a reason spec 5 states about hooks. `last_verified` alone measures the age of the document and says nothing about the target.

### Injection: every moment names the governing set, and silence is a report

Spec 5 states that a hook calls a verb that ships and introduces none, and every position below keeps that term. `headwater route <path>` is the verb at each one.

**Read.** A `PreToolUse` position on `Read` routes the path and prints the governing documents before the agent has the bytes. It is silent when nothing governs the path. So it costs one route per file open. That was 35 milliseconds on this corpus when this evaluation measured it, and 180 milliseconds on 2026-09-26. [Spec 16](../spec/16-harness-support.md) keeps the current figure. It adds context only where a document has something to say.

**Write, before.** The `write.sh` advisory moves from `PostToolUse` to `PreToolUse`, so an agent reads the governing documents before it edits and not after. The `PostToolUse` position keeps one line: the edit made an edge suspect, and `headwater check` reports it.

**Write, when nothing governs a path in scope.** This is the position HW-OBL-0105 asks for. Where the path falls under a declared scope and no document governs it, the hook prints the documents a route over the path's terms reaches. It prints the one line of front matter that declares the edge beside them. It proposes and writes nothing, because which document governs which file is a judgment. The [stop rules](../spec/05-ai-integration.md#the-stop-rules) forbid an agent that invents structure. A person or an agent that has read the file types the line.

**Review.** The commit gate already prints every finding, and the suspect finding above reaches it for free. The build order owes one more position. `hw-build.md` dispatches `headwater-maintainer` before it opens a pull request. The agent that reads a change for the documents it falsifies is dispatched by nothing today.

### The actor: `created_by: agent`, and the proposal reaches a person

HW-OBL-0105 records that `governs` declares `created_by: hook` and no hook writes one. The design above makes the write-time position propose and never write, so the honest actor is `agent`, which the closed set holds. The base package owes that value. The first-run walkthrough owes the corrected count of relations with no mechanical creator. This is a change to a published package, and it takes a record of its own when it is built.

## What this change declares

The backfill under equality is bounded on purpose. [HW-OBL-0104](../obligations/0104-a-governs-edge-reaches-the-path-it-names-and-nothing.md) prices a hand edge per file, and the ruling above makes most of them one line. This change declares the edges that #946 needed and the ones the scan shows to be a rule rather than a citation.

- The isolation evaluation governs the four `.githooks/` hooks it describes, `.claude/hooks/lib.sh`, `.claude/hooks/touch.sh`, `tools/hw-cargo`, `tools/cap-run`, `tools/repo/retire-worktree.sh` and `tools/run/run-dir.sh`. It also governs the four untyped files that restate its rules: `CLAUDE.md`, `DEVELOPING.md`, `.claude/skills/hw-run-policy/SKILL.md` and `.claude/agents/hw-build.md`.
- The isolation how-to governs `.githooks/pre-commit`, `.claude/hooks/lib.sh`, `tools/hw-cargo` and `tools/repo/retire-worktree.sh`.
- [HW-OBL-0149](../obligations/0149-the-write-time-hook-still-picks-pointer-lines-out-of-prose-by-their-em-dash.md) governs `.claude/hooks/write.sh`, which is its subject.
- [HW-DR-0064](../decisions/0064-q64-whether-intent-time-routing-gains-an-offline-embedding-path-in-shadow-mode.md) governs `.claude/hooks/intent.sh`, which writes the log it rules on.
- [Spec 16](../spec/16-harness-support.md) governs `.claude/hooks/fixtures.sh` and `.claude/hooks/fixtures-live.sh`, which hold the contract it states.
- This evaluation governs `.claude/hooks/write.sh` and `.claude/hooks/intent.sh`, the two positions its designs change.

The remaining candidate pairs are the backfill issue's evidence. The ones that are rules rather than citations wait for HW-DR-0074. Under patterns the 22 contracts and the `.claude/` set collapse to one anchor each, as a list or a glob.

## What an adopter takes from this

Four things are portable. The ruling that an anchor is a pattern, because every adopter has a directory of forty files. The scope declaration and the tree-side coverage number, because the document-side number answers a question nobody asks. The digest on the edge, because every adopter has a file that changed after its document. The four positions, because spec 5 names the moments for every harness and the verb at each one ships.

Two things are local. The build order's dispatch of the maintainer agent, and the list of paths this repository governs.

## What this evaluation leaves open

Which documents govern the 564 candidate paths is a judgment per pair, and this evaluation took it for one cluster. A `Bash` write reaches no hook, and spec 5 records that bypass. The digest names the count of changed entries and not the lines, and a reader who wants the lines has git. Whether a scope is declared by the base package for an adopter, or only by an overlay, is a question for the library track.
