---
id: HW-DR-0062
status: current
status_since: 2026-09-11
summary: "Nine required facets went unwritten on two generated documents. The engine derives seven from committed bytes, the emitter composes the summary, and the declaration block stays closed at three scalars."
last_verified: 2026-09-11
title: "Every required facet of a generated document is derived, and the emitter composes the summary"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-engine-architecture
  governs:
    - engine/crates/generate/src/derived.rs
---

# Every required facet of a generated document is derived, and the emitter composes the summary

## Context

**A generated document's kind requires what every other document of that kind requires.** Two of them answered almost none of it. [#780](https://github.com/headwater-ai/headwater/issues/780) reported the shape and understated the count. `headwater explain` gives the measurement at `640b51ea`. `docs/spec/09-open-questions.md` requires seven facets, and `docs/probe-results/regression-probe-transcript-for-2026-09-11.md` requires five. The `identity` block answered three between them and the shelf answered one, so **nine stayed unwritten**.

Nothing reported those nine. The reason is a gap between two mechanisms rather than a defect in either. The census classifies a marked file as generated, so no document rule creates an instance over it. [Spec 6](../spec/06-engine-architecture.md#projections) states that exemption and names `generate --check` as the reader that holds the file instead. That check compares committed bytes against what the emitter produces now, so it proves the emitter agrees with itself. It cannot report that the emitter produces bytes the taxonomy refuses.

**The absence had one measured consequence and one visible one.** `relations.supersedes` declares `on_target: {set_state: superseded}`, so an edge of it puts that state on its target. Two live registers declare such an edge into `docs/spec/09-open-questions.md`, and that file carried no state facet at all. [Spec 13](../spec/13-open-obligations.md) measured it as the only one of 192 nodes with none. Both instances of the dependency rule then skipped rather than decided, so a relation stated a fact that no rule could reach. That is [#227](https://github.com/headwater-ai/headwater/issues/227). Separately, three rows across two shelf indexes carried a label and no cue, because the documents they name declare no facet in the `scent` role.

**Two shapes could close it, and one of them costs a ruling spec 6 already made.** The block could take a member for each facet. Spec 6 refuses that shape. A taxonomy source sits outside the corpus root, where no census row covers it, no language regime binds it and no rule reads it. That is the refusal the shelf-sections emitter already made against a `template`. A member that took an open mapping of facets would re-admit the same prose under a different syntax.

## Decision

**The block stays closed at three scalars, and the engine writes every other facet the kind requires.** Derivation is not declaration. It is the channel this emitter already used twice. The kind of a document on a heterogeneous shelf comes from the shelf, and the reciprocal half of every incoming edge comes from the graph.

`engine/crates/generate/src/derived.rs` computes five values. Each one is found through the facet role rather than through a facet name. The reason is the one the label emitter already reads a role for: `title` means something in one taxonomy and nothing in the next.

| role | the value, and where it comes from |
|---|---|
| `state` | the state that an incoming edge sets through `on_target.set_state`, and otherwise the state whose own role is `live` |
| `state_entered` | the stalest date on the documents that set the state, or on the documents this projection read |
| `freshness` | the stalest date on the documents this projection read |
| `scent` | composed by the emitter, which is the one value here that nothing derives |
| no role | a facet that the shelf's `layout` names, read back out of the output path |

**The owner ruled on 2026-09-11 that the emitter composes the summary.** An asymmetry decided it. A generated document's body is already prose that its emitter wrote and that no rule reads. A summary beside that body is no more ungoverned than the body. So a refusal of the summary alone rests on nothing that spec 6 states.

**No value comes from a clock.** Spec 6 rules that no generated file states when it was generated. A timestamp makes every run differ from the last over a corpus that nobody touched. A date folded from the dates the inputs already carry is not a statement about the run. So the rule stands untouched and the regeneration gate holds these bytes like every other byte of the file.

**The stalest date answers rather than the freshest.** A file assembled from other documents is only as fresh as the oldest thing it carries. The freshest date would state a confidence that no source supports.

**A facet outside all of that is refused rather than defaulted.** The refusal in `identity::front_matter` now reads the whole required set against everything written into the file. What it guards is a taxonomy that requires a facet in no role this engine reads and in no shelf layout. No author can add such a facet, because a generated document's only writer is this engine. No check reads one either, because of the exemption above. An invented value is worse than a refusal, because somebody will cite it.

## Consequences

**#227 closes.** `docs/spec/09-open-questions.md` states `superseded`, derived from the two incoming edges that put it there. `engine/crates/check/fixtures/corpus.checks` loses the line that recorded both dependency instances as skipped for want of a state there. Neither instance reports a finding. The edge is the cause of the state rather than a dependency on it, which is the reading that rule already took.

**Three rows gained the cue they had none of.** They are the row for the open-questions register on `docs/spec/README.md`, and both rows of `docs/probe-results/README.md`. That closes a remainder [HW-OBL-0044](../obligations/0044-a-generated-index-labels-a-document-with-its-identifier.md) recorded when it discharged, which was one row of sixteen with a label and no summary. That record already stands at `discharged`, and this change does not edit it.

**The exemption does not end, and that is deliberate.** No check reads a generated document, so these nine facets are written and unchecked. They are written for three reasons. A reader meets them, and an index and an export carry them. The third is that a relation which put a state on its target had nowhere to put one.

**A projection now reads another projection's front matter, so the verb reaches a fixed point in two passes.** The shelf index carries the summary of every document on its shelf, and one of those documents is generated. The first pass writes the summary and the second pass carries it into the index. This is not new with this change, because the name that [#627](https://github.com/headwater-ai/headwater/issues/627) admitted had the same property. A committed corpus holds the fixed point, and `generate --check` reproduces a fixed point in one pass. That is why the gate never saw it. A contributor who adds a projection over a shelf that holds a generated document runs `headwater generate` twice.

**One judgment could go the other way.** A probe result over a transcript that stands at `draft` is itself written as `current`. The result is an accurate statement of what the corpus derives now, and its own body says the run was refused. The alternative couples a projection's state to the state of its inputs. `draft` is also the wrong word for a file that no author argues over. Nothing rules on the general question, and this record does not either.

**One adjacent gap stays open and this change does not reach it.** `corpus.checks` still records one skip over the same document. `warrant.evidence.unsupported` cannot say whether an evidenced claim rests on it, because the document declares no warrant. [Spec 3](../spec/03-authoring-and-lifecycle.md#lifecycle) rules that the engine derives `regenerated` from the marker and never from a declaration. So the warrant of a generated document is known, and the rule reads the declaration instead of the derivation.

**Two crate dependencies moved.** `headwater-generate` now names `headwater-resolve` and `headwater-scaffold` as dependencies rather than as dev-dependencies. The first carries the one hardened YAML scalar escaper in this workspace, which had four escape routes found by running the binary against it. The second carries the tokenizer that writes a file name from a shelf `layout`. Reading a path back for the facet it names is the same grammar. A second copy of either would let this engine disagree with itself about bytes it wrote.
