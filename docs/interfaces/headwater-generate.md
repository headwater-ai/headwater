---
id: HW-IFACE-headwater-generate
status: current
status_since: 2026-09-06
summary: "How to write the projections declared by the taxonomy and detect stale generated files."
last_verified: 2026-11-25
title: "headwater generate"
relations:
  governs:
    - [engine/crates/cli/src/lib.rs, engine/crates/cli/src/main.rs]
    - engine/crates/generate/src/lib.rs
    - .headwater/corpus.json
  traces_to:
    - HW-DR-0115
---

# headwater generate

## Synopsis

    headwater generate [--check] [--root <path>]

The command writes every generated projection that the resolved taxonomy declares.

## Description

Without `--check`, it computes projections and writes marked generated files. With `--check`, it writes nothing and compares committed projections with the same plan.

It refuses to overwrite a file that lacks the generated-file marker. A stale projection or a marked file that the plan does not write is an error.

**A run that refuses writes nothing.** The command finds the verdict of every output before its first write. When one of these six refusals applies, the command writes no file, and it exits 1:

- more than one transcript of one arm of one tier in one run, for the present arm, the absent arm or a component arm
- a pair of arms that holds a refused session that the recorder or the probe declaration caused
- a refused transcript whose state holds the refusal
- a marked file that no declaration writes
- an unmarked file at an output path
- an output whose marker the census does not read

The report lists each output that it did not write with the line `not written, because the run refused before it wrote this file`. A refused transcript whose state releases the refusal is not a refusal of the run, so the command writes its result and continues.

Three failures can still come after a write, because no check before the first write can find them. The operating system can refuse a write. A projection can read a value that another projection writes in a cycle, and only a write shows the cycle. A later pass can find a new refusal. The first pass writes no transcript, so only a defect in this engine or a file that another writer adds during the run causes that. In that case the report lists the files that the earlier passes wrote, and it does not say that the run wrote nothing.

**A `graph_export` that declares `committed: false` is not written and not compared.** The tree does not hold that file, because `headwater export` builds it at publish time. The run names the path with the line `built at publish time by headwater export, and not written or compared here`, and that line is not an error. The command does not read a copy that a local export left at the path, and it does not report that copy as orphaned.

**The command also refuses a declaration whose kind requires what the generated file cannot carry, and it writes no file for that declaration.** Two refusals have this shape. The first reads the `facets.require` list of the declared kind. The front-matter block states the identifier, the discriminator of a heterogeneous shelf, and the facet in the `name` role. The command derives the state, the two dates and the summary. A required facet outside that set is one that no author can add, because the only writer of a generated document is this engine.

The second reads the `sections.require` list of the same kind against the body that the emitter composes. No emitter of this engine reads a section contract. The headings of a generated body are the name of a shelf and the names of the documents on it. So a required section that no heading answers is also one that no author can add. Each refusal names the kind and the requirement, and the run prints it under *what this verb does not write, and why*.

**With `--check`, the command reads the producer identity before it compares the bytes of any projection.** The corpus descriptor at `.headwater/corpus.json` records the emitter set of the engine that wrote it. When that number is not this engine's, the two runs do not agree about what the projections are. A byte difference then has two possible causes. A corpus moved, or an emitter moved, and the command cannot tell which. It reports both numbers, it withholds the instruction to regenerate for every projection in the run, and it exits non-zero. When the numbers agree, or when the committed descriptor records no emitter set, the command behaves as it always did.

A descriptor that records no emitter set is a descriptor an earlier engine wrote. Absence is not disagreement, so the command reports no producer difference for it. The first run of this engine over such a repository writes the member, and the pair agrees from then on.

**An emitter set is a number a person raises, and the raise moves `.headwater/corpus.json` in every repository.** An emitter that changes the bytes it produces for a corpus it produced other bytes for before raises the number. The next `headwater generate` in an adopter's repository then writes a new descriptor. That cost is the point: a regenerate is deliberate, and the taxonomy lock format already behaves this way.

Under *what this verb does not write, and why*, a run reports every projection kind this engine does not emit. It reports a kind the taxonomy declared at the output path of that declaration, and a kind no declaration named at `no declaration names one`. A reason is a property of this engine rather than of the corpus, so a reader gets it before writing the declaration. The run also reports each shelf of the taxonomy that no `shelf_index` declaration names, at `` shelf `<name>` ``. Its reason gives the declaration that adds an index. Name the shelf in the `for` of a `shelf_index` declaration, or add a `shelf_index` declaration for the shelf under `add_to: projections:` in the overlay. A shelf that a `shelf_index` declaration names gets an index or one of the reasons above, and never this line.

Under *unreadable documents*, a run names each file under the corpus root that the census could not read. Each entry gives the reason, and the line and column where the reader gives them. No projection holds such a file, so without this section a run would drop the file from every index and say nothing. Front matter that nests its collections more than 256 deep is one such file. The section is a report and not a refusal: it changes no exit status, and `headwater check` is the gate on an unreadable file.

A `consumer_surface` declaration writes one page from the `surface` block of the taxonomy. The page lists the archive, the integration points with what each one needs, the prerequisites, the companions and the commands. It does not list `adopter_documents` or `local_roots`, because they configure a check and an adopter does not receive or run them. A taxonomy with no `surface` block writes no page, and the run reports the declaration under *what this verb does not write, and why*. A change to the block makes the page stale, and `--check` reports it.

### What a projection writes, and why

**A shelf index writes a bullet for each document, and a shelf sections file writes a heading.** A bullet carries no anchor, so nothing outside the index cites one row. A heading is an address, so a reader cites the row rather than the file. That difference is the reason for two kinds. A member that changed the shape of one output would be read by one kind and ignored by the other.

**A projection interpolates a path, a facet value and a declared identity, and nothing else.** The heading text is the value of the facet in the `name` role ([spec 2](../spec/02-taxonomy-model.md#the-meta-schema)). An identifier is refused, because a name exists to replace it. A template on the declaration is refused, because it puts authored prose in a taxonomy source. A taxonomy sits outside the corpus root, so no census row, language regime or link rule reads that prose. [Q4](../spec/09-decisions.md#q4--relation-storage) refused a second authoring location for an edge, and this rule applies that refusal to prose.

**Every decline is whole.** A shelf that cannot name every document produces no file, and the run names the document that stopped it. A file with one section missing hides the document that it dropped, and a citation into that section fails with no stated reason. Two documents may not carry one name either. A renderer numbers the second of two equal headings, so a citation from the name resolves to the first document only.

### The generated-file marker

**The marker is the record of the previous run.** The command reads the bytes on disk rather than a manifest, because a manifest beside the file is a second statement of one fact ([principle 2](../spec/00-vision-and-scope.md#design-principles)). It writes a path that holds nothing. It overwrites a file that carries the marker. It refuses anything else and names the path, so a projection cannot silently destroy an authored document.

**The census gives a marked file an outcome of its own, and judges nothing about it.** A shelf index lands on the shelf it indexes and has no front matter. Without its own outcome, each run would report the index as an untyped document of that shelf. The census row names what holds the file, as an excluded path names the rule that excluded it. No check reads a generated document, for two reasons. Its content is a function of the emitter, and an author cannot repair it in the file. `generate --check` holds it to the bytes its emitter produces now.

**The marker sits on the first line, or inside the front-matter block.** A file with no front matter, such as a shelf index, carries it above everything. A generated document that declares an identity opens with the fence. It carries the marker as a member of the block, under the quoted key `headwater:generated`, as a JSON output does. The census reads a member inside the block and nowhere else, so prose that quotes the marker stays prose.

**The marker is a claim, and `--check` tests it.** Anything can write the line, and the census excuses a marked file from every document check. So a run reads the census beside its own plan, and a marked file that no declaration writes is an error. The cause is a removed or repointed declaration, or a marker that somebody wrote by hand. The remedy is to delete the file or to restore the declaration.

**The census reads a marker in each format that a projection writes, and in no other format.** A projection writes Markdown, JSON, YAML and TOML, and a `relation_view` also writes CSV. A `graph_export` can land under the corpus root, and its file stays there when its declaration moves, so the census must read its marker. The census opens no image, archive or other format. The extension is not the whole test, because each kind writes its own content. A `graph_export` at a `.yml` path is JSON, and the census reads a YAML file for a marker on its first line only. So the command refuses an output in another format, and an output that the census would not read back as generated.

**No generated file states when it was generated.** `--check` compares bytes, and a timestamp makes every run differ from the last. [Q17](../spec/09-decisions.md#q17--governed-access-and-the-solution-layer) asks a filtered export to state when it ran. The two rules cannot both hold for an artifact this gate covers, and [spec 13](../spec/13-open-obligations.md) carries that conflict.

**A date that a generated document carries is not a timestamp.** Each date folds the dates of the documents that the projection read. So the value is a function of committed bytes and not of the clock. The date in the `freshness` role is the stalest of those dates, because the file is only as fresh as the oldest thing it carries. Where no edge sets the state, the date in the `state_entered` role is the newest of them.

### A generated document is a document of the corpus

**A generated document that declares an identity is a node of the graph.** The marker exempts the file from checks and from nothing else. Its identifier and kind are functions of the emitter, and a wrong one is repaired in the declaration. The census reads the block, the identifier index holds it, and an edge can name it at either end.

**A declaration states that identity in three scalars: the identifier, the kind and the name.** All three are declarations, so the taxonomy holds them. A summary, a status, a date and a body are prose, and no member of the block carries one. An open mapping of facets would admit prose again, and the refusal of a template would then be about syntax only.

**Every other facet that the kind requires is derived** ([HW-DR-0063](../decisions/0063-every-required-facet-of-a-generated-document-is-derived-and-the-emitter-composes-the-summary.md)). The shelf gives the kind on a heterogeneous shelf. The graph gives the reciprocal half of each incoming edge. The engine reads a facet role and not a facet name, so a taxonomy that renames every facet reads the same. A facet that the shelf `layout` names comes from the output path. The emitter composes the summary. The command refuses a required facet in no role and in no layout, as the refusal above states.

**One test says which files are documents.** The identifier index, the edge builder and a shelf index all read it. So a shelf index holds a row for a generated document on its shelf. Two derivations would disagree about the files that a projection took over. A byte diff would then be the only report of the drop.

**An index row for a generated document carries the cue its emitter composed, where the kind requires one.** A kind that requires no facet in the `scent` role carries no cue, and nothing invents one. A row with no cue still meets [spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for), because the index names the document.

**`kind` names a kind, and never the facet that carries it.** The command reads the shelf that claims the output path. A heterogeneous shelf needs the discriminator. A homogeneous shelf states the kind itself, and [spec 2](../spec/02-taxonomy-model.md#placement-is-primary-metadata-fills-the-gap) forbids a facet that restates it, so the block writes none.

**A generated document declares no edge, and it carries the reciprocal halves that it owes.** [Q4](../spec/09-decisions.md#q4--relation-storage) makes front matter the one place where an author writes an edge. A generated document has no author. So the engine derives each owed half from the edge that another document wrote, and it derives no other edge.

**The warrant does not change.** [Spec 3](../spec/03-authoring-and-lifecycle.md#lifecycle) exempts a generated projection from acceptance and holds it to regeneration, and the census derives that warrant from the marker. A node carries the same value, and its declaration answers who stands behind it.

### A verb index

A `verb_index` carries one row for each verb that the binary dispatches. Each row names the document that describes the verb, or carries a mark where no document does. That mark is the reason the artifact exists, because a missing description is what a reader acts on.

**The rows come from the engine and not from the graph.** The verb list is data of the engine, in the same class as the register and the corpus descriptor. It is neither a taxonomy source nor authored prose, so the interpolation rule above does not close it. The projection reads no path outside the corpus root, so [Q29](../spec/09-decisions.md#q29--whether-a-corpus-root-may-contain-code-and-what-an-interface-contract-may-reach) decides nothing here.

**The join is the whole command line.** A declaration names one shelf, and the emitter matches the facet in the `name` role against the name of a verb. `headwater check` names the verb `check`, and the bare word names nothing.

**Two states decline the whole file.** A document on the shelf that answers to no verb would drop from the index. A document with no name cannot join a verb. Either one loses a description that somebody wrote.

**An empty shelf is not a decline.** An index of verbs that nobody described is this artifact at its most useful. A shelf index of no documents differs, because it only asserts that a shelf exists.

**No projection holds a crate.** [HW-OBL-0128](../obligations/0128-nothing-holds-a-crate-to-having-a-contract-under-a-root-that-excludes-it.md) records that gap. A verb index answers the half that the command surface carries, and leaves the crate tree as that record found it.

### A relation view

A `relation_view` is a traceability matrix ([HW-DR-0114](../decisions/0114-a-relation-view-follows-an-ordered-chain-of-relations-from-every-document-of-one-shelf.md)). It starts from each document of the one shelf that `for` names, and it follows the relations that `relations` names, in order. The first column is the start document, and each later column is one step of the chain.

**A row is one path, and a missing link is an empty cell.** A document that reaches three documents at a step has three rows, and its own cell repeats on each of them. A document that reaches nothing at a step has one row, with an empty cell at that step and at each later step. So no document of the start shelf drops from the matrix. The rows sort by the path in each cell, from the first cell to the last.

**A cell states four things.** It names the document, its lifecycle status, a state and the snapshot entry of `.headwater/observations.yml` that names the document. A verification states `declared`, `observed`, `suspect` or `unknown` against the document before it, as `headwater check` reads it. Any other edge between two documents holds `no stamp`. No cell reads the clock, so a change to a document, a stamp or the snapshot makes the committed view stale, and `--check` reports it.

**A step may reach code, and then it is the last step.** A step whose `to` names only anchor kinds, such as `governs`, writes the code paths it reaches in the last column. A `source-tree` anchor states `stamped`, `baseline`, `suspect` or `unstamped`, from the edge's `verified_revision` and the digest the anchor reaches now.

**`format` is the view's own, and the output's extension follows it.** `markdown` is the default and writes a table under a marker line. `csv` writes four fields for each cell under a first line that opens with `#`. `json` writes the rows as arrays of cells, with `null` for an empty cell and the marker as a member. A view is not an export emitter, so `format: json` does not enroll it in the emitter set of its profile. The output may hold `{shelf}`, which is the directory of the start shelf.

**The command refuses a chain that cannot be walked.** It refuses `relations` on another kind, and a view with no `relations` or with a `for` that does not name one shelf. It refuses a step that names no declared relation. It refuses a step that reaches only code before the last step, and two steps that do not join. Two steps join when a `to` kind of the first is, or inherits from, a `from` kind of the second, or the reverse.

**A Markdown view is a page of the site.** A `site_nav` lists a Markdown view that sits in the directory of a shelf it covers, under the label "The traceability matrix". It lists no CSV or JSON view, because MkDocs serves neither as a page.

### Agent rules

An `agent_rules` projection writes one GitHub Copilot `.instructions.md` file for each document that governs code. [HW-DR-0115](../decisions/0115-an-agent-rules-file-is-a-budgeted-pointer-at-the-copilot-instructions-path.md) states the form, and this section says what the command writes for it. The file is generated, budgeted, checked for regeneration. Its effect on agents is unmeasured.

**A source is a document on a shelf that `for` names, with at least one `governs` edge.** An empty `for` means every shelf, as it does for every other kind. A document with no `governs` edge gets no file, and so does a document whose `governs` targets resolve to no pattern.

**The output must hold `{name}`, which is the file stem of the source.** The stem is the file name without `.md`, so `docs/standards/api-design.md` writes `.github/instructions/api-design.instructions.md` from the output `.github/instructions/{name}.instructions.md`. The command refuses an output with no `{name}` and writes nothing for it. When two sources have one stem, the command writes neither file and names both sources. It does not let one overwrite the other. <!-- headwater allow=surface.local_path.instructed scope=block until=2027-09-30 reason=false_positive note=a path an agent_rules declaration writes into the repository of the adopter -->

**The front matter opens on line 1.** `applyTo` holds the `governs` patterns as the source declares them, joined by a comma with no space, in double quotes. The command writes the pattern and not the paths it matched, because Copilot reads the glob itself. The quotes are necessary because a YAML value that starts with a character such as `@` does not load without them. The marker is a front-matter member, so `--check` holds the file and the orphan rule reads it.

**The body is a pointer to the source.** It holds the label of the source, a link relative to the file, the source path and the summary. When nobody accepted the source, it also says so. It ends with the sentence "Read that document before you edit these files." It copies no other text from the source. An edit of the summary makes the committed file stale, and an edit of the body of the source does not.

**`budget` is taken on an `agent_rules` alone.** It holds `per_file` and `aggregate`, two positive whole numbers of bytes. The command refuses `budget` on every other kind, a figure that is zero or not a whole number, and a budget that omits a figure. `taxonomy resolve` and `taxonomy validate` read the same declaration, so each refuses a misplaced budget before it writes a lock. [#2088](https://github.com/headwater-ai/headwater/issues/2088) refuses an agent-facing projection with no budget.

**The command writes no rule past its budget.** It renders every rule first. A rule file past `per_file` is not written. The remaining rules stand in the derived reading precedence of their sources. A nucleus comes before its satellite, and a governing document comes before the document that it governs. The command keeps the longest head of that order that fits in `aggregate`, and it prunes the rest. So the rule of a satellite goes before the rule of its nucleus. Each rule that is not written is a line of "what this verb does not write, and why", with the size and the figure. A marked file can already be at the path of such a rule. An agent would keep loading it, so `--check` reports it as stale and the command removes it. The command changes no source document to fit a budget. `projection.budget.exceeded` in [`headwater check`](headwater-check.md) reports each rule that the budget withholds ([#2090](https://github.com/headwater-ai/headwater/issues/2090)).

## Preconditions

The repository must have a readable consumer declaration, taxonomy lock, corpus and generated projection declarations. Existing generated targets must satisfy their write preconditions.

## Options

| Option | What it does |
|---|---|
| `--check` | Writes nothing and exits non-zero when a projection is stale, or when the committed corpus descriptor records an emitter set that is not this engine's. |
| `--root <path>` | Selects the repository to load. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

## Exit status

**0** means that all declared projections match or were written. A document that the census cannot read does not change the status.

**1** means that loading, planning or writing failed, that `--check` found drift, or that `--check` found a producer difference. The two `--check` failures print different sentences, because only one of them has a remedy this command can name. Without `--check`, a refusal leaves the tree as the command found it.

**1**, and never 101, when standard output or standard error cannot be written, and one sentence on standard error names a failed standard output.

## Environment

The command reads no environment variable.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.lock`, corpus and configuration | Read to build the projection plan. |
| `.headwater/imports/` | Read where `.headwater/taxonomy.yml` declares an import, for the anchors that an imported snapshot supplies. An `imports` entry that does not read stops the verb with exit 1, and so does an `at` path outside the repository root. |
| the path each `harvests.<name>.at` names | Read where `.headwater/taxonomy.yml` declares a pinned corpus export, for the anchors that export supplies. A `harvests` entry that does not read stops the verb with exit 1, and so does an `at` path outside the repository root. A pin with no digest binds no anchor. An absent file binds no anchor, and neither does a file that fails the pinned digest or is not an export. |
| `.headwater/corpus.json` | Read under `--check` for the emitter set it records, before any projection is compared. Written like any other projection. |
| Generated projection files | Written without `--check` when their marker permits it and the run refuses nothing. A `graph_export` output that declares `committed: false` is not written and not read. |

## See also

[`headwater taxonomy resolve`](headwater-taxonomy.md) refreshes the lock. [`headwater export`](headwater-export.md) emits a selected profile.
