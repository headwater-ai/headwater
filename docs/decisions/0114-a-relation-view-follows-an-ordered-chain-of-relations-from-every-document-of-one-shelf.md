---
id: HW-DR-0114
status: current
status_since: 2026-11-25
summary: "A relation view is a matrix with one row per path from each document of one shelf through an ordered chain of relations. A missing link is an empty cell, and a cell states the status, the stamp or freshness, and the observation of its document"
last_verified: 2026-11-25
title: "114 — A relation view follows an ordered chain of relations from every document of one shelf"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: evidenced
relations:
  governs:
    - engine/crates/generate/src/relation_view.rs
  traces_to:
    - HW-DR-0073
---

# 114 — A relation view follows an ordered chain of relations from every document of one shelf

## Context

Spec 0 promises a traceability matrix as part of deliverable 3. The projection kind `relation_view` had a slot in the meta-schema and no emitter. Its reason in `unbuilt()` said that no document states which relations a view covers. [#2018](https://github.com/headwater-ai/headwater/issues/2018) is the issue.

The owner ruled six points on 2026-11-22, after a review of two outside adoption trials. The ruling is in the body of #2018 and in a comment on it. The [requirement-verification entry](../taxonomies/requirement-verification/doctrine.md) of #2017 then gave the chain its kinds and its relations: `verified_by`, `proven_by` and `cited_in`.

The ruling leaves four choices open. This record states each of them, so that the emitter in `engine/crates/generate/src/relation_view.rs` implements a statement and does not make one.

## Decision

**The six points of the owner's ruling (2026-11-22).**

1. A view follows an ordered chain of relations, one column for each step. A single relation is a chain of length one.
2. Every document of the start shelf is a row, and a missing link is an empty cell.
3. A cell shows the linked document, its lifecycle status, whether the edge is stamped or suspect, and the latest recorded observation from `.headwater/observations.yml`. No value depends on the clock.
4. The chain may end at code. The last step may follow `traces_to` or `governs` to code paths.
5. The obligation and control register is out of scope. A view covers relations between documents and the optional code step, and nothing else. This statement replaces the reason that `unbuilt()` gave, because the kind now has an emitter and `unbuilt()` gives it no reason.
6. A view is declared with one new projection member, `relations`, which holds the chain in order. It sits beside `for` (the start shelf), `output` and `format`. The default format is a Markdown table, committed in the tree and held by `generate --check`. `csv` and `json` are the other two formats.

**Choice 1: one row for each path.** A requirement that three criteria verify cannot hold three criteria in one cell and still say which verification proves which criterion. So a row is one path through the chain, and the start document repeats on each of its rows. A document that reaches nothing at a step has one row, with an empty cell at that step and at each later step. This keeps point 2: no document of the start shelf is ever dropped. The rows sort by the path of the start document, then by the path in each later cell. So two runs over one tree write the same bytes.

**Choice 2: a code step is any step whose `to` names an anchor kind.** Point 4 names `traces_to` and `governs`. This record reads it as a rule about the declaration and not as a list of two names. So `cited_in` reaches the `test_site` anchor in the same way. An anchor target ends its path in that cell. A step whose `to` names only anchor kinds is the last step, and the engine refuses it at any other position. A step whose `to` mixes kinds, such as `verified_by` with `check_rule`, may stand at any position, and only its anchor targets end a path. This is a reading of point 4, and the owner did not rule it.

**Choice 3: what "stamped or suspect" means for each kind of edge.** An edge between two documents holds no revision, so point 3 reads differently at each kind of end:

- A `verification` states its freshness against the document in the cell before it. The comparison is the one that `headwater check` reads ([HW-DR-0073](0073-a-verification-is-a-kind-and-its-identity-is-minted-rather-than-found-in-the-code-that-cites-it.md) ruling 3): `declared`, `observed`, `suspect` or `unknown`.
- A code path that a `source-tree` anchor reaches compares the edge's `verified_revision` with the digest that the anchor reaches now: `stamped`, `baseline`, `suspect` or `unstamped`. Any other resolver states no digest, and its cell says `no stamp`.
- Every other edge holds no stamp, and its cell says `no stamp`. The start cell of a row is no edge, so it states no stamp.

The observation of a cell is the commit of the snapshot entry that names its document, or none. Only a verification has such an entry in practice, and every cell states it so that a reader never has to ask.

**Choice 4: a CSV file carries the marker on a first line that opens with `#`.** CSV has no comment syntax. A first line `# headwater:generated relation_view. …` is the form that the census already reads for YAML and TOML. The readers that take a comment character, such as `pandas` with `comment="#"` and R with `comment.char = "#"`, skip it. A strict RFC 4180 reader sees it as one record. A marker cell would put the same text in a record of every reader, so this record takes the line. A JSON view carries the marker as the member `"headwater:generated"`, as a `graph_export` does.

**What the engine refuses.** Each refusal is in the reader of `projections`, so that `generate` and `taxonomy publish` refuse one declaration with one sentence. `taxonomy resolve` also refuses a step that names no declared relation.

- `relations` on any kind other than `relation_view`.
- A `relation_view` with no `relations`, or with a `for` that does not name exactly one shelf.
- A step that names no relation by its canonical name. A step walks its relation from the `from` end to the `to` end, whichever end wrote the edge.
- A code step that is not the last step.
- Two steps that do not join, and a start shelf whose kind does not join the first step. Two ends join when one kind is, or inherits from, the other. So an abstract `to` such as `governed_document` joins a concrete `from`, and the reverse also joins.
- A `format` other than `markdown`, `csv` or `json`, and an output whose extension is not the format's extension. A `.csv` output on any other kind.

**A view's `format` is not an export emitter.** The export emitters serialize the graph, and a view serializes its own rows. So `format: json` on a view does not put the entry in the emitter set of its profile. An output path may hold `{shelf}`, which is the directory of the start shelf, as it is for a `shelf_index`.

## Consequences

`unbuilt()` returns `None` for `relation_view`, and spec 6 lists it under `runs`. Nine of the thirteen projection kinds have an emitter, and four wait: `agent_rules`, `template`, `transcription` and `coverage_report`. [The `headwater generate` contract](../interfaces/headwater-generate.md#a-relation-view) states the projection.

The requirement-verification entry declares one view, from `verified_requirements` through `verified_by`, `proven_by` and `cited_in`. This repository generates it, and every verification cell here reads `declared` until a snapshot records one.

Two open findings move. [HW-OBL-0049](../obligations/0049-the-meta-schema-closes-eight-value-sets-that-nothing-states.md) counts value sets that the meta-schema closes and nothing states. The value set of a view's `format` is stated here, so it does not add to that count. [HW-OBL-0172](../obligations/0172-nine-hand-kept-constants-enumerate-an-enum-and-nothing-holds-one-against-the-variants.md) counts hand-kept lists of an enum. `Kind::DECLARABLE` does not change, and `unbuilt()` and the spec 6 block move together, held by `spec_six_projections.rs`.

**What reopens this record.** A reader who needs one row for each start document, with the paths nested inside it, reopens choice 1. That reader is a consumer of the JSON format, and a nested shape would be a fourth format, not a change to these three. An owner ruling on point 4 that limits a code step to `traces_to` and `governs` reopens choice 2.
