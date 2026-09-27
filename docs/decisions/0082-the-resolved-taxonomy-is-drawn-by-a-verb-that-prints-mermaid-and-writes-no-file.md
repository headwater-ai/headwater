---
id: HW-DR-0082
status: current
status_since: 2026-09-24
summary: "`headwater taxonomy graph` prints the resolved taxonomy from the lock as a Mermaid flowchart, and no projection, explain format or export target draws it, and D2 with TALA is the candidate second format"
last_verified: 2026-09-27
title: "The resolved taxonomy is drawn by a verb that prints Mermaid and writes no file"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft+revise
  evidence_basis: evidenced
relations:
  governs:
    - engine/crates/cli/src/taxonomy_graph.rs
---

# The resolved taxonomy is drawn by a verb that prints Mermaid and writes no file

## Context

[#1009](https://github.com/headwater-ai/headwater/issues/1009) asks for a picture of the resolved taxonomy: its kinds, the purpose of each kind, its anchors, and the relations between them. The first picture came from a script that held its own list of kinds and lanes. That list was correct on the day it was written, and it goes wrong when the lock moves. The issue asks where a picture that follows the lock belongs. It names three places: a projection that `headwater generate` writes, a format of `headwater explain`, and a page of the site.

No verb drew the taxonomy on 2026-09-24. `headwater export` writes the corpus graph as `json` or `jsonschema`, and nothing in `engine/` or `docs/` wrote Mermaid or DOT.

The reader is an adopter who resolved a package or an overlay of their own. That reader wants to see its kinds and relations without reading `.headwater/taxonomy.lock` by hand. `taxonomy diff` and `taxonomy migrate` serve the same reader.

## Decision

`headwater taxonomy graph` prints the resolved taxonomy as a Mermaid `flowchart` on standard output. It reads `.headwater/taxonomy.lock` and no source. It writes no file, and it declares no projection. [The contract](../interfaces/headwater-taxonomy.md) states what the drawing holds.

**The drawing has two views, chosen with `--view`, because one drawing cannot hold both questions.** The concrete view answers which kind can relate to which. The abstract view answers what an abstract kind gives the kinds under it, and which relations name it. The first version of this decision listed the pairs of an abstract kind in a caption. That caption named relations that no edge showed, and the abstract view now draws them. The default is the concrete view.

**It is not a projection.** A projection is a committed derived artifact. It needs a new projection kind in the base package, a change to the lock, and a regenerate after each change to the taxonomy. No reader needs a committed copy of what the lock already holds, and [HW-DR-0006](0006-where-the-corpus-graph-lives-at-rest.md) rules that no derived artifact is canonical.

**It is not a format of `explain`.** [`explain`](../interfaces/headwater-explain.md) answers about one document. A picture of the whole taxonomy takes no document, so it would be a second verb inside a flag.

**It is not a target of `export`.** [HW-DR-0013](0013-linkml-and-shacl-as-substrate.md) ships an emitter target only when a named consumer asks, and each target declares what it drops. A picture for a person is not an interchange format, and it does not go ahead of that queue.

**The page of the site waits, and this decision does not refuse it.** [HW-DR-0036](0036-q36-which-of-mkdocs-docusaurus-or-astro-this-corpus-emits-navigation-for-and-why.md) picked MkDocs, and MkDocs renders a Mermaid block. So a page can embed the output of this verb with no second renderer.

**The output is Mermaid, because the renderer does the layout.** GitHub, MkDocs and most Markdown viewers lay out a Mermaid flowchart. So the engine carries no layout and no position, and the drawing still reads when a release adds kinds or relations. DOT can be a second format later. No reader has asked for it.

**D2 is the candidate second format, and the output stays Mermaid only for now.** [#1107](https://github.com/headwater-ai/headwater/issues/1107) drew both views by hand in D2 v0.9.0. Its TALA layout gave the most compact drawing of the layouts that the issue tried: the abstract view was 1800 by 1161 pixels, and its 19 `is_a` edges merged into a few shared trunks. D2 also has a native key. But GitHub and the VS Code Markdown preview show Mermaid inline and do not show D2. A D2 drawing is an SVG that somebody makes with the `d2` binary, which is a Go program, and then keeps. In the same test, a glob edge in D2 made no edges and gave no warning. So a D2 emitter must write each edge explicitly and count the edges against the lock, as the Mermaid emitter does. The owner [ruled on 2026-09-27](https://github.com/headwater-ai/headwater/issues/1107#issuecomment-5856183225) to keep Mermaid only.

## Consequences

A drawing is a function of the lock. The engine reads every lane, node and edge from the `resolved` block, so no list of kinds or relations in code can drift from it. `engine/crates/cli/tests/taxonomy_graph.rs` edits a copy of the lock and holds the drawing to the edit.

An abstract kind is no node of the concrete view. A pair with an abstract kind at one end is no edge of that view, and one comment line points to the abstract view. The abstract view draws each abstract kind, the kinds declared under it, and those pairs as edges. Each view reads `abstract: true` and `is_a`, and neither names a kind or a relation. A relation from a node to itself is one line on that node and no edge. Mermaid draws such an edge as a detour that two of them turn into a knot. The line takes the color that its family gives an edge. `--legend` adds a key to either view, and the key takes its colors from the list that colors the edges. It names a family that only a line on a node carries.

The question opens again on one of three events. A page of the site embeds the output, a consumer asks for a committed file, or an issue for a taxonomy explorer is filed. The first two make a projection worth its regenerate cost. An explorer can filter by purpose, follow one relation and open the document behind a kind, so the static drawing then matters less, and Mermaid can stay as the plain fallback.

When one of these events makes a second format necessary, D2 with TALA is the first candidate, before DOT. A D2 emitter first names its consumer, who runs `d2`, and where the SVG is kept. [HW-DR-0013](0013-linkml-and-shacl-as-substrate.md) holds an export target to the same test.
