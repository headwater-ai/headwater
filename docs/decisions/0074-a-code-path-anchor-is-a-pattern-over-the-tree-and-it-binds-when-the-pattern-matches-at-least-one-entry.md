---
id: HW-DR-0074
status: current
status_since: 2026-09-20
summary: "A `code_path` anchor is a pattern in the shelf language or a list of them, and a bare path matches one entry. An edge binds when every pattern matches at least one entry, and a query matches a path against the patterns."
last_verified: 2026-09-20
title: "A code path anchor is a pattern over the tree, and it binds when the pattern matches at least one entry"
relations:
  constrains:
    - HW-DR-0029
  governs:
    - engine/crates/graph/src/anchors.rs
    - engine/crates/graph/src/edges.rs
    - engine/crates/meta/src/pattern.rs
    - engine/crates/query/src/lib.rs
    - engine/crates/query/src/route.rs
  traces_to:
    - HW-OBL-0104
    - HW-OBL-0105
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-fable-5-1
  accepted_by: j.baxter
  activity: measure+draft
  evidence_basis: evidenced
---

# A code path anchor is a pattern over the tree, and it binds when the pattern matches at least one entry

## Context

[Spec 2](../spec/02-taxonomy-model.md#the-decision-relation-vocabulary) declares the `code_path` anchor with the `source-tree` resolver. It states nothing about what the string on the edge denotes. [Spec 5](../spec/05-ai-integration.md#write-time-hooks) states the consequence. `governing_docs_for_path` answers by equality against the string each edge reached, so a document reaches the paths it names and nothing under them. [HW-OBL-0104](../obligations/0104-a-governs-edge-reaches-the-path-it-names-and-nothing.md) has waited on this ruling since 2026-08-13.

The engine does what spec 5 states. `SourceTree::resolve` in `engine/crates/graph/src/anchors.rs` normalizes the raw value, asks whether that one path exists, and runs the corpus exclusions over it. `governing_docs_for_path` in `engine/crates/query/src/lib.rs` compares the asked path against the normalized string of each edge. A route that names a path in a task does the same. Neither position reads a pattern.

The cost is on the tree. On 2026-09-18 `headwater taxonomy audit` reports 144 `governs` halves from 53 documents, which is 14.2 percent of 374 eligible documents. Twenty-two interface contracts each name `engine/crates/cli/src/lib.rs` and `engine/crates/cli/src/main.rs` by hand. Spec 5 names nine files under `.claude/` one by one. A new file in a governed directory is governed by nothing, and no rule reports it, because the set has no denominator.

One pattern language exists in this engine, and it lives in one crate on purpose. `headwater_meta::pattern::Pattern` admits four forms: `**` as a whole segment, `*` inside a segment, `?`, and a literal. It carries a specificity order and an overlap test. Shelf declarations, corpus exclusions and `headwater explain` read it. The exclusion test inside `SourceTree::resolve` already runs that matcher over the normalized path. The [evaluation](../evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md) that raised this record measured what each position reaches and what it misses.

## Decision

**The raw value of a `code_path` anchor is a pattern in the language of `headwater_meta::pattern`, or a list of such patterns.** Each pattern is normalized as a path is normalized today. A value with no wildcard is a pattern that matches one entry. So every edge this corpus declares keeps the meaning it has.

**A list is one anchor, and its matched set is the union of what its members match.** An author writes a list where one document governs one thing that no single pattern names, such as `engine/crates/cli/src/lib.rs` and `engine/crates/cli/src/main.rs`. Separate entries under the relation stay separate anchors, one for each fact. In front matter a list is a YAML sequence in the place of the string. The reader admits a sequence only where the endpoint is an anchor, because a document target is one identifier. No taxonomy declaration changes, because the endpoint is still `code_path`.

**An anchor binds when every pattern it holds matches at least one entry of the source tree, and it is unresolved otherwise.** An entry the corpus excludes is not a match. A pattern that matches nothing is an unresolved target, and `relation.target.unresolved` reports it by name, as it reports a missing path. A list with one dead member is reported for that member. An edge onto nothing is the stale edge that rule exists to raise.

**A value with no wildcard is the one exception, and it answers a different question.** That value already confirmed the target exists: the check is a filesystem `exists`, not a search. So an exclusion there is metadata about a real file, and never grounds to call the edge broken. It resolves and carries the exclusion as a note. That is what "every edge this corpus declares keeps the meaning it has" requires, above. A pattern with a wildcard, or a list, answers a coverage question instead. Does some real, included entry back this claim at all. There "an entry the corpus excludes is not a match" holds with no exception. A set with only excluded entries in it is the stale, over-broad claim `relation.target.unresolved` exists to raise.

**A directory named without a wildcard matches the directory entry and nothing under it.** A subtree is written with `**`, as a shelf is. There is no implicit descent. `.claude/hooks` reaches `.claude/hooks` alone, and `.claude/hooks/**` reaches every entry under it. Neither reaches `.claude/hooks-disabled/write.sh`.

**The identity of an anchor node is its normalized patterns, sorted.** Two documents that write one pattern, or one list in any order, share one node. Two anchors that both match one file are two nodes, and a query about that file answers with both.

**A query about a path matches the path against every pattern of every anchor and never against the string.** That is `governing_docs_for_path`, the anchor half of `headwater route`, and every position that calls either one. The matched set is a fact the resolver holds. So `headwater explain` reports how many entries an anchor reaches, in total and for each member. A report over the corpus then has the denominator that HW-OBL-0104 records as absent.

**Refused.** Implicit descent for a bare directory, because it changes the meaning of every edge on the tree, and a prefix test admits `hooks-disabled` for `hooks`. A second relation for a subtree, because one fact takes one relation. A brace alternation inside a pattern, because a list already names a set and a fifth form is a second copy of that. A regular expression or a second glob dialect, because a language that admits more than the corpus uses carries untested cases. Two matchers disagree the day one of them learns a construct. A pattern with no literal segment before its first wildcard, such as `**/README.md`. The one subtree no path it admits can lie outside of is then the whole tree the resolver was given. No repository this ruling was measured against has ever written a pattern of its own that opens that way.

## Consequences

**The fixtures this record owes are the ones HW-OBL-0104 names, plus three.** A path equal to the anchor, a path under a `**` anchor, and a path that shares a prefix with the anchor and leaves it. The third is `.claude/hooks/**` against `.claude/hooks-disabled/write.sh`, and a naive prefix test passes it wrongly. The fourth is a pattern that matches no entry. The fifth is a list with one member that matches nothing, reported for that member and not for the list. The sixth is one list written in two orders, which is one node. Acceptance of this record discharges HW-OBL-0104, and moving that record is a human act.

**Four more fixtures came out of building the above, and each one is now in the suite.** A wildcard pattern whose only matching entries are all excluded resolves to nothing. That is the same "not a match" rule a matched-but-excluded entry answers to. A pattern with no literal segment before its first wildcard is refused rather than walked. Two patterns that share a literal prefix, asked of one resolver, walk the tree once. This is proven by mutating the tree between the two calls and showing the second answer does not move. Two different lists whose members would collide under a plain `, `-joined identity encode to two different identities instead. One of the two holds the join text itself.

**Three sentences state equality and answer to this record.** Spec 5 under write-time hooks, the glossary entry for impact detection, and the `governs` paragraph of the `headwater-authoring` skill. Each is a second copy of the rule. The change that builds this ruling rewrites all three. A skill that instructs the opposite of a decision record is a defect the skills fixture does not catch.

**The read set of a cached verdict has to hold the matched set.** [HW-OBL-0117](../obligations/0117-a-cached-verdict-about-an-anchor-survives-the-change-that-falsifies-it.md) records that a target outside the corpus is in no cache key. Under this ruling a file added under a governed subtree changes the matched set of an edge and no document. A key that holds only the declaring document then serves a stale verdict for every rule that reads the set. The sorted list of matched entries enters the key.

**The resolver reads the tree once, not once per edge.** `SourceTree` caches the walk of a wildcard pattern's literal prefix, keyed by that prefix, for its own lifetime. Two edges whose patterns share a prefix, or one prefix a list anchor names twice, walk the filesystem once between them. The cache never crosses a run. One `headwater check` process builds one `Resolvers`, and so one `SourceTree`. `.headwater/cache/checks` is the cache that answers across runs ([HW-OBL-0117](../obligations/0117-a-cached-verdict-about-an-anchor-survives-the-change-that-falsifies-it.md)).

**A list's identity is not a joined string.** `["a", "b, c"]` and `["a, b", "c"]` are two different lists. Both sort and join on `, ` to `"a, b, c"`. So `Target::Anchor.normalized` is a length-prefixed encoding of the sorted patterns instead, injective over their content. That field is the identity a duplicate-edge check, `anchor_nodes()` and an exported node key all read. A single pattern's identity stays the plain string it always was, unaffected. `Target::anchor_display` is the separate, human-facing join every renderer reaches for.

**One edge names a set, so the capture cost of governing forty files is one line.** [Spec 3](../spec/03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) names that cost as the thing that killed every prior design-rationale tool. Nothing here forces an author to rewrite an existing edge. An interface contract that names two files by hand keeps both edges until its author chooses one anchor that lists both, or `engine/crates/cli/src/*.rs`. The digest that ages an edge, where the evaluation's design for it is built, is one digest over the union. That is why a set is one anchor rather than several.

**This record rules on denotation alone.** Which actor proposes a `governs` edge, and whether `created_by: hook` on the relation is true, is what [HW-OBL-0105](../obligations/0105-nothing-plays-the-hook-role-that-two-relations-name.md) waits on. The evaluation above carries a design for it that is not a ruling.
