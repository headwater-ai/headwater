---
id: HW-DR-0036
status: current
status_since: 2026-08-30
summary: "MkDocs is the pick. Its `nav:` is data and not code, and Astro has no native nav format to emit at all."
last_verified: 2026-08-30
title: "Q36 — Which of MkDocs, Docusaurus, or Astro this corpus emits navigation for, and why"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-sonnet-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-vision-and-scope
    - HW-SPEC-taxonomy-model
    - HW-SPEC-engine-architecture
---

# Q36 — Which of MkDocs, Docusaurus, or Astro this corpus emits navigation for, and why

## Context

**Spec 0 already closes the candidate set to three.** [Spec 0](../spec/00-vision-and-scope.md#what-we-do-not-build) refuses to build a documentation renderer and names the alternative: "Emit navigation config for MkDocs / Docusaurus / Astro." No fourth generator is named anywhere else in this corpus. This record picks among the three, and states why.

**`site_nav` is declarable and unbuilt today.** [Spec 6](../spec/06-engine-architecture.md#projections) lists it among the ten declarable projection kinds. `engine/crates/generate/src/lib.rs` reports it under `Unwritten`, with the reason: `spec 5 and Q16 name the artifact and no document states its form, so an emitter here would be this engine inventing a schema for somebody else's consumer`. This record removes that reason. `.headwater/overlay.yml` declares no `site_nav` entry yet.

**A reading order is already derived, and a shelf index already consumes it.** `Relation::governs()` in `headwater-graph` returns `Governs::Source`, `Governs::Target`, or `Governs::Neither` from a relation's nucleus and family, per [spec 2](../spec/02-taxonomy-model.md#reading-precedence-is-derived)'s four clauses. `Surface::by_precedence` in `headwater-query` walks that derivation to order a list of documents, and `headwater generate`'s `shelf_index` emitter already calls it. A `site_nav` emitter needs the same call over each of this corpus's thirteen shelves, and not a new derivation.

**This corpus's shape is one output file over flat groups, and not a deep tree.** `.headwater/taxonomy.lock` resolves thirteen shelves under `docs/`, each a flat glob (`docs/decisions/**`, `docs/spec/**`, and so on). A shelf holds no shelf of its own. So the nav a generator needs is two levels: a shelf name, and the ordered documents under it. Nothing here asks for arbitrary nesting.

## Decision

**MkDocs is the pick.** `mkdocs.yml`'s `nav:` is a nested list of one-key mappings (`Title: path/to/file.md`). This corpus's shape above is exactly that: one entry per shelf, holding an ordered list of `{title, path}` pairs that `governs()`-derived precedence already produces. Four reasons carried the pick past the other two.

**First, `nav:` is data and `sidebars.js`/`.ts` is code, and [spec 0](../spec/00-vision-and-scope.md#design-principles)'s first design principle is the tie-breaker for exactly this case.** "Configuration over code… if a change to the taxonomy requires a change to the engine, the design is wrong." MkDocs's native format is plain YAML with no execution semantics. Docusaurus's native format is a JavaScript or TypeScript module: `module.exports = {...}` or `export default {...}`. Even where its content is JSON-shaped by convention, the file itself admits arbitrary code. Nothing here would ever write that code, but a hand edit could. A generated file this engine holds to exact regeneration should not sit in a format whose native shape is a program.

**Second, MkDocs costs this repository no new toolchain, and the other two cost the same one.** `.githooks/` and `.claude/hooks/` already assume `python3`: `fixtures.sh`, `fixtures-live.sh`, `intent.sh`, `write.sh`, and `ci.yml` all read it. So a Python-based static-site generator adds no toolchain class this repository does not already use for its own tooling. `git grep` over `.github/workflows/` finds no `node`, `npm`, or `package.json` anywhere. Docusaurus and Astro both require Node.js and `npm`, a toolchain category this repository has never once needed, for its engine or its hooks.

**Third, MkDocs's own default matches this corpus's layout with no configuration at all.** MkDocs resolves `docs_dir` to `docs/`, relative to `mkdocs.yml`, when the key is absent. This corpus's root is already `docs/`. A `mkdocs.yml` at the repository root points at this corpus's content with nothing stated about where the content is.

**Fourth, and decisively against Astro: there is no single native nav-config format for Astro to emit.** MkDocs's is `nav:` in `mkdocs.yml`. Docusaurus's is the sidebar object in `sidebars.js`/`.ts`. Plain Astro has neither. Navigation there is hand-written inside a layout component. The one Astro-ecosystem convention with a declarative sidebar config is the Starlight theme, and this record never raises that unrelated third-party choice. Picking Astro now forces one of two outcomes. Either this engine invents a schema, the exact refusal the `unbuilt()` reason for `site_nav` already states. Or it silently adopts Starlight as a second undecided dependency. Astro also needs real page components to render anything. Picking it now would commit piece C, which is not built, to a component-authoring shape before piece C has been scoped at all. MkDocs and Docusaurus both render Markdown files directly, from a nav entry that names a path. Either one leaves piece C exactly as unconstrained as it is today: write Markdown at the declared paths.

**The cost this pick does carry, stated honestly: no crate of this engine writes YAML today.** `headwater-yaml` parses YAML and it writes JSON, through the hand-rolled `Json` struct in `crates/yaml/src/json.rs` that `graph_export` calls. It writes no YAML anywhere. `engine/crates/generate/src/site_nav.rs` is new code either way. The shape it needs is narrow: a nested list of plain-string keys and values, two levels deep. It sits over titles this corpus already writes in Markdown, and paths this corpus already carries. That is no larger a task than `Json` already was to write. It is smaller than a general-purpose YAML serializer would be, because nothing here asks this emitter to round-trip an arbitrary document. Docusaurus's JSON-shaped sidebar object would have let this emitter reuse `Json` directly, which was the one implementation convenience in the other direction. It does not outweigh the three reasons above.

## Consequences

**`.headwater/overlay.yml` declares `site_nav` at `.headwater/nav.yml`.** That path is not new to this corpus. [Q16](0016-public-presence.md#decision) already named `.headwater/nav.yml` as the illustrative output, when it ruled that Headwater emits the navigation and a third-party generator renders it. [Spec 2](../spec/02-taxonomy-model.md#the-thirteen-declarations)'s own worked example for a `site_nav` projection uses the same path. `graph_export`'s `profile: site` entry already sits at `.headwater/export.json` beside it, for the same reader.

**The generated file carries `nav:` alone, and nothing a human would add to a working `mkdocs.yml`.** `site_name`, `theme`, and any plugin configuration are not stated anywhere in this corpus. Writing them here would be the same invention the `unbuilt()` reason refuses, aimed at a different field. A generated file's whole content comes from what `plan()` derives, the same rule every other emitter in this engine already holds to. A file that mixed generated `nav:` with hand-authored `site_name` would break under `generate --check`, the moment either side changed alone. A complete, buildable `mkdocs.yml` that folds `.headwater/nav.yml` in is downstream work, after piece C exists, and is not this record's concern.

**The emitter writes a small YAML routine that did not exist before this record.** `engine/crates/generate/src/site_nav.rs` writes the nested-list text by hand, in the way `crates/yaml/src/json.rs` writes JSON by hand, because no shared YAML writer exists to call. Escaping is the one detail worth naming ahead of time. A document title may carry a colon or a quote, and a plain YAML scalar breaks on either. So a title is written as a quoted scalar, and never assumed plain.

**This reopens if a later record needs what only Docusaurus or Astro supplies.** Versioned documentation, or interactive, componentized pages, are needs this corpus does not have today. If piece C or a successor states either need in writing, this ruling is the record to revisit. The four reasons above are what a replacement has to outweigh.
