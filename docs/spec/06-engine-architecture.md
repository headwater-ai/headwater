---
id: HW-SPEC-engine-architecture
status: current
status_since: 2026-08-01
last_verified: 2026-08-11
summary: One parse, one typed graph, and many consumers, with the pipeline, the library boundary, and the performance targets.
doc_type: design_spec
sequence: 6
title: "Engine architecture"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: [claude-fable-5, claude-opus-5]
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cites_evidence:
    - HW-EVAL-adjacent-work
    - HW-EVAL-first-contact
    - HW-EVAL-graph-export-and-federation
    - HW-EVAL-language-spike-results
    - HW-EVAL-shacl-worked-example
    - HW-EVAL-the-measurement-layer
    - HW-EVAL-the-serving-boundary
    - HW-EVAL-warrant-and-adjudication
    - HW-EVAL-what-a-check-can-know
---

# 6 — Engine architecture

One parse, one graph, many consumers.

## Why one engine

The obvious decomposition — a separate linter per concern — is the wrong one. Each tool re-walks the tree, re-parses front matter, re-implements path matching, and re-derives what kind each document is. That is slow, and worse, it is *divergent*. When two tools disagree about what a document is, the result is contradictory findings and an unfixable bug report.

Headwater parses once, builds one typed graph, and runs every check against it. Checks become small predicates over a shared model instead of programs.

## Pipeline

```
┌──────────────┐   ┌───────────┐   ┌────────────┐
│ taxonomy pkg │──▶│  resolve  │──▶│   lock     │
│  + overlays  │   │  + verify │   │ (hashed)   │
└──────────────┘   └───────────┘   └─────┬──────┘
                                         ▼
┌──────────────┐   ┌───────────┐   ┌────────────┐   ┌──────────┐
│ corpus files │──▶│   parse   │──▶│   graph    │──▶│  cache   │
└──────────────┘   │ + classify│   │ build/link │   └──────────┘
                   └───────────┘   └─────┬──────┘
                                         │
        ┌────────────┬───────────────────┼──────────────┬────────────┐
        ▼            ▼                   ▼              ▼            ▼
     checks       queries          projections       export      explain
        │            │                   │              │            │
     findings    pointers          derived files    JSON / MCP   derivation
```

**Resolve** merges the base taxonomy and overlays, validates against the meta-schema, and writes a content-hashed lock. Everything downstream reads the lock, never the sources. Thus a check result depends on a hash that a reviewer can see in a diff.

**Parse** reads each file once: front matter, headings, links, code fences. It does not interpret. Classification assigns a kind by the declared resolution rules and records the derivation for `explain`.

**Graph build** resolves relations into edges, indexes identifiers, binds external anchors (code paths, work items, URLs), and reports what it could not resolve. It also emits a **census**: every file under the corpus root and the outcome for each. The census fixes the denominator for coverage before any check runs. Thus a document that failed to classify is visibly unchecked, not silently absent.

**Cache** is content-addressed per file plus taxonomy hash, so incremental runs are proportional to the change, not the corpus. That first sentence is the promise [HW-OBL-0072](../obligations/0072-a-cache-of-check-results-does-not-make-a-run-proportional.md) holds open. The change-scoped mode that CI and hooks use is the same code path over the same corpus, and it narrows nothing. It supplies the version each named document stood at before the change, so the rules that read one reach a verdict rather than a skip. It therefore evaluates more instances than a full-corpus run and never fewer.

## Nothing stores the graph

The graph is a function of the corpus and the lock, and every run rebuilds it. Three artifacts derive from it, and none of them is canonical for anything ([Q6](09-decisions.md#q6--where-the-corpus-graph-lives-at-rest)).

| Artifact | Lives for | Committed | Canonical for |
|---|---|---|---|
| The in-memory graph | one run | never | nothing |
| The cache | until its inputs change | never, and version control ignores it | nothing |
| An export | until a run regenerates it | when the taxonomy declares an output path | nothing |

**The cache is disposable, and a test says so.** `headwater check --no-cache` produces output byte-identical to `headwater check`. A cache that can change a verdict is a store under another name. The engine carries that test beside the fixtures for its other correctness roots ([spec 12](12-check-layer.md#the-correctness-roots)).

**There is no embedded database, and the trigger to add one is named.** The refusal rests on measurement rather than on taste. A warm change-scoped pass over 1,000 documents costs about 2 ms against a 200 ms budget ([spike results](../evaluations/language-spike-results.md)). Nothing in the design asks a question that the in-memory graph cannot answer inside the targets below. A named query workload that misses a target reopens this, and nothing else does.

The industry does not agree, and the disagreement belongs in view. CodeQL ships a derived database as its query surface, at very large scale. The refusal here follows from spec 0's non-negotiables: offline, deterministic, and reviewable in a diff. It does not follow from a claim that a derived store cannot work ([evaluation](../evaluations/graph-export-and-federation.md)).

## Checks

A check is a pure function from a **scoped view** of the graph to findings. Checks come from five origins:

| Origin | Comes from | Exportable as |
|---|---|---|
| **Shape** | the taxonomy, generated | JSON Schema, and SHACL or LinkML when either emitter arrives |
| **Graph** | relation declarations, generated | SHACL, when that emitter arrives |
| **Corpus** | declarations that need many documents at once | — |
| **Document** | regimes applied to the body, which is not in the graph | — |
| **Plugin** | adopter code | — |

The first two are *generated*: a new facet or relation brings its checks with no code. That is the point of taxonomy-as-data, and most of the check count is there. The last three are why a native engine exists at all. They are exactly what LinkML and SHACL cannot express.

The last column is a **set of emitter targets**, not one format. A check exportable to SHACL need not be exportable to JSON Schema, and most checks export to nothing. The engine generates both the exported set and the unexported set from one registry. A target may appear only when the emitted constraint is equivalent to the native check. [Spec 12](12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule) owns the partition rule and the equivalence bar.

The column above states what an origin can reach. The declaration is per rule, and it lives on the check as `EXPORTABLE_AS`. Two rules declare a target today. `facet.required.missing` and `facet.value.not_permitted` both name `jsonschema`, and a differential test established each one.

Every check declares its **scope** (document, edge, neighborhood, shelf, corpus), and the engine enforces it. A check sees only what it declared. Scope is what makes change-scoped evaluation exact, cache keys sound, and parallelism safe.

The design — scope semantics, instances and coverage, the two-phase census, fixability, determinism, and the plugin contract — is [spec 12](12-check-layer.md).

## Projections

Projections are generated artifacts. Each one has the same contract:

```
headwater generate            # write
headwater generate --check    # fail if any committed output differs
```

**Twelve projection kinds exist, and this engine emits seven of them.** The block below names all twelve, as a taxonomy writes each name. A kind under `runs` has an emitter here. A kind under `waits` has a slot that a declaration opens and no emitter fills.

```
runs
  shelf_index
  shelf_sections
  site_nav
  graph_export
  probe_result
  verb_index
  corpus_descriptor

waits
  relation_view
  agent_rules
  template
  transcription
  coverage_report
```

A shelf index and a shelf sections file each carry the documents of one shelf. A probe result carries the verdicts of one graded run, and [spec 5](05-ai-integration.md#a-run-produces-a-snapshot-and-a-document) declares it. A verb index carries the command surface of the binary, and [the paragraph below](#a-verb-index-reads-the-command-surface-of-the-engine) declares it. Site navigation, a graph export and the corpus descriptor each carry one file that a reader outside this corpus opens.

A relation view carries decision lineage and a traceability matrix. A template carries the permitted relations, facets and sections of one kind. An agent rule file is the artifact [the glossary](glossary.md#projection) names. A transcription reads a pinned external snapshot, and [Q19](09-decisions.md#q19--inbound-integration-an-external-system-of-record) leaves it to the first adopter who asks. `engine/crates/generate/src/lib.rs` states beside each of these four what no document says, and `headwater generate` prints all four over any corpus. `engine/crates/generate/tests/spec_six_projections.rs` holds the block above against that statement.

**A run over any corpus names every waiting kind, and the coverage report is the one of the five that no taxonomy declares.** [Spec 4](04-assurance-model.md#every-obligation-has-exactly-one-disposition) makes the register engine-defined, so no taxonomy declares it and every run reaches it. The other four are declarable, and a run states each one at `no declaration names one` where the corpus declared none of them. A reason is a property of this engine rather than of the reader's corpus, which the [ruling on #596](https://github.com/headwater-ai/headwater/issues/596#issuecomment-5562802450) records. `headwater generate --check` prints the register under *what this verb does not write, and why*:

```
coverage_report the register
  its content is a function of the clock as well as of the corpus and the lock, because a migration task lapses and a suppression expires on a date. A committed copy would fail this check on a morning when nothing changed. Spec 13 carries it
```

[Spec 13](13-open-obligations.md) carries the register, and no committed copy of it exists.

**Ten of the twelve are declarable, and two are not.** A taxonomy names the kind and the output path, and [principle 1](00-vision-and-scope.md#design-principles) makes that path a schema decision. The coverage report and the corpus descriptor are the exceptions. [Spec 4](04-assurance-model.md#every-obligation-has-exactly-one-disposition) makes the register engine-defined and non-optional, and [Q20](09-decisions.md#q20--where-scent-lives) fixes the descriptor at `.headwater/corpus.json`. Both hold that standing for one reason. A reader who must consult the taxonomy to find an artifact already knows what it would tell them. So a declaration of either would put a second copy of one artifact at a path the engine did not fix. The meta-schema therefore closes the declarable ten as a value set, and `headwater generate` writes the other two under no declaration at all.

**A shelf index writes a bullet for each document, and a shelf sections file writes a heading.** A bullet carries no anchor, so nothing outside the index cites one row of it. A heading is an address, so a reader, an evaluation and an agent each cite the row rather than the file. That is the whole of the difference between the two kinds, and it is the reason they are two kinds. A member that changed the shape of one output would be read by one kind and ignored by the rest.

**A projection interpolates a path, a facet value and a declared identity, and nothing else.** The heading text is the value of the facet in the `name` role ([spec 2](02-taxonomy-model.md#the-meta-schema)). Two other sources are refused. An identifier is the stable choice and it is the artifact that a name exists to replace. A template on the declaration puts authored prose in a taxonomy source, and a taxonomy sits outside the corpus root. No census row covers it, no language regime binds it, and no rule reads its links. So prose that the corpus governs would move to the one file that the corpus cannot see. [Q4](09-decisions.md#q4--relation-storage) refused a second authoring location for an edge, and this is that refusal applied to prose.

**Every decline is whole.** A shelf that cannot name every document produces no file, and the run names the document that stopped it. A file with one section missing hides the document that it dropped. A citation into that section then fails, and nothing anywhere states why. Two documents may not carry one name either, and the reason is worse than a failed citation. A renderer numbers the second of two headings with one text, so a citation from the name resolves to the first document alone.

**A projection carries a generated-file marker, and the marker is the record of the previous run.** The engine refuses to overwrite a file that lacks it. Read as two permissions, that rule asks the engine to remember what it wrote last time, and it does not need to. A manifest beside the file is a second statement of one fact, which is the drift [principle 2](00-vision-and-scope.md#design-principles) rules against. So the engine reads the bytes on disk. A path that holds nothing is written. A file that carries the marker is overwritten. Anything else is refused, and the run names the path. Thus a projection can never silently destroy an authored document.

**A projection lands inside the corpus root, so the census gives it an outcome of its own.** A shelf index has to land on the shelf it indexes. The next run walks that file and finds no front matter. It would otherwise report the file as an untyped document of that shelf, against every contract the kind requires. The artifact the engine wrote would become a finding against the corpus, on every run.

**The census accounts for a marked file and judges nothing about it.** The row names what holds the file, in the way an excluded path names the rule that excluded it. No check reads a generated document, because its content is a function of the emitter and an author cannot repair it in the file. `generate --check` holds the file to the bytes its emitter produces now. Two reports of one fact would send a reader to two places.

**The marker sits on the first line, or inside the front-matter block.** A generated file with no front matter carries it above everything, and a shelf index is such a file. A generated document that declares an identity opens with the fence, so the first line is taken. It carries the marker the way a JSON output does, as a member of the block under the quoted key `headwater:generated`. The two positions are disjoint, and a member is read inside the block and nowhere else, so prose that quotes the marker is still prose.

**A generated document that declares an identity is a node of the graph.** The exemption above is about checks, and it says nothing about identity. The content of a generated file is a function of the emitter, and so are its identifier and its kind. A wrong one is repaired in the declaration, which is what `generate --check` already holds. The census reads the block of such a file, the identifier index holds it, and an edge names it at either end. Without this a projection can write only a file that no document points at.

**A declaration states that identity in two scalars, and it states nothing else.** The block names the identifier and the kind. An identifier is minted once and a kind is a declaration, so the taxonomy is where both belong. A summary, a status and a body are prose, and no member of the block carries one. A member that took an open mapping of facets would admit that prose again under a different syntax. The refusal of a template above would then be a ruling about punctuation rather than about substance.

**A generated document is a document of the corpus, and one test says which files are.** The exemption above is about checks, and it reaches nothing else. So a shelf index holds a row for a generated document on its shelf, and a read offers one. The engine derives the set of documents once, from the test that the identifier index and the edge builder already read. Two derivations disagree about one class of file, which is the class that a projection took over. The byte diff of a generated file is then the only report of the drop.

**An index names a generated document and says nothing about it.** The identity block holds two scalars, so such a document declares no facet in the `scent` role. A row for it carries the identifier and no cue. A row with no cue is still what [spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for) asks for. A document that no index names is one that a reader cannot know about.

**`kind` names a kind, and never the facet that carries it.** The engine reads the shelf that claims the output path, and writes what that shelf needs to resolve the kind. A heterogeneous shelf needs the discriminator. A homogeneous shelf states the kind itself, and [spec 2](02-taxonomy-model.md#placement-is-primary-metadata-fills-the-gap) forbids a facet that restates it, so the block writes none.

**A generated document declares no edge, and it carries the reciprocal halves that it owes.** [Q4](09-decisions.md#q4--relation-storage) makes front matter the one place an edge is authored, and a taxonomy source is not front matter. A relation that requires reciprocity obliges the far end of each pair to declare its half. A generated document has no author to write one. Each such half is the inverse of an edge that another document already wrote. So the engine derives it from the graph, and it derives no other edge. A projection that needed one would be stating a fact that exists nowhere else, which is the second authoring location that Q4 refused.

**Nothing about the warrant moves.** [Spec 3](03-authoring-and-lifecycle.md#lifecycle) exempts a generated projection from acceptance and holds it to regeneration, and the census derives that warrant from the marker. A node carries the same derived value. The graph holds what a document is, and the warrant states who stands behind it. A generated document answers the second question with its declaration rather than with a person.

**The marker is a claim, and `generate --check` tests it.** Anything can write the line, and the census excuses a marked file from every document check. So the line would otherwise exempt any document from every rule, permanently and in silence. A run therefore reads the census beside its own plan, and a marked file that no declaration writes is an error. Somebody removed a declaration or repointed it and left the output behind, or somebody wrote the marker by hand. The remedy for both is to delete the file or to restore the declaration.

**A generated file in another format stays outside this rule.** The census reads a marker on a Markdown file, and it already accounts for every other file as one that is not a document. To read a marker in any format asks the walk to open every image and every archive under the corpus root. The answer would change no verdict.

**No generated file states when it was generated.** `generate --check` compares bytes. A timestamp inside an output makes every run differ from the last one, so the gate would report drift over a corpus that nobody touched. [Q17](09-decisions.md#q17--governed-access-and-the-solution-layer) asks a filtered export to state when it ran, and the two rules cannot both hold for an artifact this gate covers. [Spec 13](13-open-obligations.md) carries the conflict.

### A verb index reads the command surface of the engine

A verb index carries one row for every verb the binary dispatches. It names the document that describes each verb, and a verb that no document describes carries a mark in place of a link. That cell is the reason the artifact exists. An index of the descriptions that exist reports work already done, and the missing description is what a reader acts on.

**The rows come from the engine and not from the graph.** The rule above closes the sources a projection may interpolate, and it closes them against a taxonomy source and against authored prose. The dispatch table is neither. It is data of the engine, in the class the register and the corpus descriptor already project. No reader has to consult a taxonomy to learn what the binary carries. This projection reads no path outside the corpus root, so [Q29](09-decisions.md#q29--whether-a-corpus-root-may-contain-code-and-what-an-interface-contract-may-reach) decides nothing here.

**The join is the whole command line.** A declaration names one shelf, and the emitter matches the facet in the `name` role against the name of a verb. `headwater check` names the verb `check`. The bare word names nothing, because a description of a command is titled by the command line.

**Two states decline the whole file, in the way every other decline here is whole.** A document on the shelf that answers to no verb would be dropped from the index. A document with no name could not be joined to a verb at all. Either one loses a description that somebody wrote.

**An empty shelf is not a decline, and that is the difference from a shelf index.** An index of verbs that nobody has described is this artifact at its strongest. An index of no documents is a file that asserts a shelf is there.

**What no projection holds is a crate.** [HW-OBL-0128](../obligations/0128-nothing-holds-a-crate-to-having-a-contract-under-a-root-that-excludes-it.md) records that gap. A verb index answers the half of it that the command surface carries, and it leaves the crate tree where that record found it.

### An export is a projection, and it declares what it dropped

A graph export is a projection like the others. The taxonomy declares its output path, so whether an export is committed is a schema decision and not an engine default ([principle 1](00-vision-and-scope.md#design-principles)). A declared export is held to regeneration by `generate --check`, exactly as a shelf index is.

**A committed export regenerates on every edit to a facet it carries, and that is the `Cargo.lock` regime.** The native export is lossless, so it carries every facet of every document, and `summary` is one of them. An edit to one summary changes the export, and `generate --check` then fails until somebody regenerates it. A committed copy that survived a source edit would be a committed copy that had drifted, which is what the gate exists to catch. This corpus already pays the same cost on its shelf index, which carries the summary of every document on the shelf.

**An adopter who wants a committed artifact that a summary edit leaves alone has two controls, and neither one drops a facet.** The first is which corpus the profile carries. The second is whether the taxonomy declares an output path at all, because an export with no declared path is never committed. `headwater export --format` writes such an artifact to standard output on demand. A filter over facet values is not a third control, and [the filter section](#an-export-profile-carries-a-filter) below states why.

Exports fall into two classes, and only one class preserves fidelity.

- **The native graph export** carries the property graph with no loss, and that includes the instance attributes on edges ([Q4](09-decisions.md#q4--relation-storage)). It is what the federation tier reads ([spec 7](07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it)).
- **An interoperability export** is lossy by construction. RDF, SKOS, LinkML, SHACL, JSON Schema and OKF each speak a vocabulary that cannot carry everything in the graph.

So every emitter declares a **loss set**: the node classes, edge classes, and attributes that its target cannot carry, each with a reason. Every export run then emits a **projection census**. Every node and every edge in the graph is either present in the output, or accounted for by a declared loss reason. An omission that no reason covers is a projector defect, and it fails the run.

**A node of that census is a classified document or an external anchor, and an identifier is not what makes one.** A typed document that declares no identifier never reaches the identifier index, and it is still content that leaves a corpus or does not. A census over the index alone would let an emitter drop every unidentified document and report itself complete.

That is the coverage doctrine of [spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for), applied one layer out. It answers the trust problem that the [SHACL evaluation](../evaluations/shacl-worked-example.md#problem-one-everything-downstream-trusts-the-projection-and-shacl-does-not-check-it) found. The projector was the component that everything downstream trusted and nothing could check. A round trip is the wrong instrument for the lossy class, and an earlier draft of [Q6](09-decisions.md#q6--where-the-corpus-graph-lives-at-rest) asked for one. The native export keeps its round-trip test, because an empty loss set is exactly what a round trip proves.

**An emitter that cannot carry the warrant does not carry the content.** Every node carries a [warrant](01-conceptual-model.md#warrant), and the native export carries it with no loss. A target vocabulary that has no place for it produces an artifact in which unwarranted content is indistinguishable from accepted content. A loss-set entry reaches the consumer who reads the loss set and nobody else. The field also reports that ordinary tooling strips a mark which travels beside content ([HW-EVAL-adjacent-work §P](../evaluations/adjacent-work.md#p--provenance-endorsement-and-the-record-of-a-judgment)). So such an emitter **withholds** every `asserted` and `transcribed` node, at the profile's declared tombstone grain, with its own inability to mark as the reason. That is [principle 7](00-vision-and-scope.md#design-principles) read the way that a filtered exporter reads it. An unmarked assertion is unrecoverable, and a withholding is visible and cheap ([Q15](09-decisions.md#q15--a-synthesized-content-tier)).

**A transcription that leaves carries its pin.** A `transcribed` node exports the identity of the snapshot that it copies. A consumer who holds the copy can then return to the authority and ask whether it is current. Scholarly publishing solved the same problem in that direction, rather than by a flag that has to survive every copy.

**Emitters never chain.** Every emitter reads the resolved lock and the graph directly. A pipeline that routes one standard format through another inherits every loss of every hop, and declares none of them. LinkML's own SHACL generator is the observed case, because it drops constructs that LinkML itself expresses ([Q13](09-decisions.md#q13--linkml-and-shacl-as-substrate)).

### An export profile carries a filter

The export is the point where a corpus meets a reader that it does not control, so it is where a corpus decides what leaves. An **export profile** is an entry under `projections` ([spec 2](02-taxonomy-model.md#the-thirteen-declarations)). It names an audience, an emitter target, an output path, a **filter** over facet values, and a **tombstone grain**. A corpus with one audience declares one profile with no filter, which is the first release ([Q17](09-decisions.md#q17--governed-access-and-the-solution-layer)).

**The audience is the name, and the name groups the entries.** The fifth rule below asks every projection inside a profile to regenerate from the filtered graph, so a profile holds more than one artifact. Two entries that write one audience name are two artifacts for it. **Two entries of one profile may not declare two filters, and the engine refuses the pair.** One audience has one answer about what it may see. Otherwise one of the two filters wins. The winner is whichever the reader reaches last, and the artifact that lost carries more than the profile permits. An entry that names no profile is in the profile called `default`. A name that nobody can type is a name that `--profile` cannot select.

**The filter runs at export, and there is no reader to identify.** A profile filters for a destination and never for a person. So the engine holds no principals, evaluates no permission at request time, issues no credential, and records no read. The bytes of a filtered export live in a repository. The permissions of the hosting platform on that repository decide who reads them, exactly as they decide who reads the Markdown. One permission system, and it is not ours.

Six rules make the filter honest, and three of them already hold elsewhere.

- **Carried and withheld partition the corpus, and the engine generates both.** This is the [partition rule](12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule) that `exportable_as` obeys, applied to documents instead of to checks. Neither list is authored, so neither can drift from the other.
- **A withholding is a loss reason.** The projection census already accounts for every node and edge that the output does not carry. A withheld document is one more accounted absence.
- **A document is withheld whole.** The unit is the document, and no filter reaches inside a body. A redaction inside prose is how a reader ends up with a rectangle drawn over text that is still there.
- **The filter is default-deny over classes.** A node class, an edge class, or an attribute that no profile names does not travel. So a later release that adds a class does not widen a profile that nobody re-read. A filter stated as a list of exclusions grows a hole every time the schema grows.
- **Every projection inside a profile regenerates from the filtered graph.** Take a shelf index, a lineage view, or a navigation file. Built at full visibility and then shipped inside a filtered profile, each one carries what the filter removed. A count, a sort order, or an index of terms is enough. That failure is observed, and it is the one that survives a correct redaction.
- **The declaration travels with the artifact.** A filtered export states that it is filtered, and it states when it was generated. A copy of an artifact carries neither of those unless the artifact does.

**The class half of that fourth rule waits on an emitter that needs it.** The declaration reaches facet values, and the document is the unit that a filter withholds. A class filter acts on an emitter that carries some classes and not others. Neither of the two emitters that exist is one. The native export carries every class, and a JSON Schema carries no instance at all. So the first emitter that partitions by class is the one that gives a class filter something to act on.

**The attribute half waits on a consumer, and until then a filter reaches no attribute.** The engine evaluates a clause over a document's facet values and withholds that document whole. It never projects an attribute off a document that it carries. So no profile can emit a graph that holds a document and omits its summary. What would ask for one is a consumer that wants a committed artifact which a summary edit leaves alone. That work reaches past the filter, because the projection census would have to account for a withheld attribute as well as a withheld node. And [`exportable_as`](12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule) would have to rule whether an export that drops a declared facet still conforms.

**The generation time is injected, and that is what lets it coexist with a byte gate.** The rule above and the regeneration gate look incompatible. A time inside an output moves on every run, so `generate --check` reports drift over a corpus that nobody touched. They hold together because they cover two artifacts. A **committed** export is held to regeneration and carries no time at all. An export that **leaves** the repository is the artifact the rule above is about, and `headwater export --at <date>` supplies its time. The clock is thus a value that a caller injects. [Spec 12](12-check-layer.md#determinism-concretely) injects it into a check for the same reason, rather than let one read a syscall. Same corpus, same lock, same injected clock, byte-identical output, under both.

**The tombstone grain is declared, because the two things that a filtered view owes a reader are in tension.** A view must not look complete, and a report of what it withheld is itself a disclosure. Both cannot hold in full. The freedom-of-information statutes reached this exact conditional from the other direction, and so did the multilevel-security literature ([HW-EVAL-adjacent-work §O](../evaluations/adjacent-work.md#o--the-serving-boundary-descriptors-redaction-and-the-write-path)).

| Grain | What the reader learns | When it fits |
|---|---|---|
| `counted` | A placeholder sits where each withheld node or edge would have been, and it carries the identifier of the rule that withheld it | The default. The reader is a tier under a contract, and the existence of the item is not the secret |
| `sealed` | The view is filtered. Nothing else | The existence of the item is itself the disclosure |

**A withholding reason comes from a closed set that the taxonomy declares.** Free prose in a tombstone is a channel, and a reason that quotes the document is a leak wearing a label. The rule identifier is what a reader needs to ask for access, and it is all that they get.

**No profile may produce a view that presents as total.** That is the invariant, and it holds under both grains because it leaks nothing. Under `sealed` a reader still knows to stop drawing conclusions from absence, which is the harm that the rule exists to prevent. An agent that traverses a filtered graph, finds nothing, and reports absence is the failure that [spec 5](05-ai-integration.md) names at its start. Here our own filter causes it.

**An exporter fails closed, and that is [principle 7](00-vision-and-scope.md#design-principles) read correctly.** An exporter that cannot evaluate its filter emits nothing and fails the run. It never emits an unfiltered artifact, and it never emits a partly filtered one. The principle says "fail open at the edges", and its own gloss gives the rule underneath: degrade toward the cheaper error. For an agent-facing hint, silence is cheaper than a wrong pointer. For an exporter with a filter, an empty output is cheaper than one document too many.

**A withholding rule never ships advisory.** Its two error classes are not both recoverable, so the promotion machinery measures the wrong one ([spec 4](04-assurance-model.md#promotion-advisory-to-blocking)). It is not suppressible and it is not waivable.

### What a filtered export claims, and what it does not

A tool acquires a security obligation when it publishes a claim that a boundary holds, and not before. So the claim is stated here, narrowly, and the things that are **not** claims are stated beside it. A reader who treats a non-claim as a boundary has been misled by us rather than by an attacker.

**The claim.** A filtered export contains no document that its declared filter withholds, and no artifact inside the profile derives from one.

**Not claims, and each one is a channel that the design accepts rather than removes.**

- **The tombstone under `counted` is a declared channel.** It reports that something exists and does not say what. That is deliberate, and an adopter who cannot accept it declares `sealed`.
- **Shape is not hidden.** Shelf and kind names, node counts, and edge degrees describe organizational and product structure. No filter removes what the remaining graph implies.
- **A reader who can also read the publishing repository is not separated from anything.** The export is not a boundary against a party that holds a clone.
- **Revocation is not immediate.** A tier reads a pinned export, so a document withheld today stays in the tier's copy until the next harvest ([spec 7](07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it)). The lag is bounded by the export cadence, and the export carries its generation time so that a reader can compute it.
- **The platform's repository permission is the enforcement, and it has its own limits.** Repository history, forks, and a change of visibility are governed by the hosting platform and not by us.
- **No claim about a license reaches any content.** Headwater never reads an upstream's terms, and it cannot decide whether an adopter may redistribute a requirement that the adopter imported. A profile that carries `transcribed` content republishes somebody else's material, and the adopter owns that decision. The filter defaults to deny over classes, so the decision is a line in a taxonomy that a reviewer reads ([Q19](09-decisions.md#q19--inbound-integration-an-external-system-of-record)).

## Interfaces

### CLI

```
headwater check       [--strict] [--fix] [--no-cache] [--now <date>]
                      [--read-set <path>] [--register <path>]
                      [--format text|json|sarif|markdown | --json]
headwater gate        --read-set <path> [--now <date>] [--json]
headwater derived
headwater generate    [--check]
headwater new         <kind> --title <text> [--summary <text>]
                      [--relates <relation>=<identifier>] [--facet <facet>=<value>]
                      [--now <date>]
headwater capture     [--format text|json | --json]
headwater sweep       plan [--under <path>]
                    | report <path> [--format text|json | --json]
headwater route       <task description> [--json]
headwater query       <expression>
headwater explain     <path|identifier> [--json]
headwater mcp         [--now <date>] [--write]
headwater import      [<name>] [--expect <digest>] [--write]
headwater export      [--profile ...]
                      [--format json|jsonschema|shacl|rdf|skos|okf|linkml | --json]
                      [--at <date>] [--check]
headwater init        [--corpus <dir>] [--package <name>]
headwater infer       [--owner <name>] [--until <date>] [--write]
headwater conformance [--level <name>] [--now <date>] [--json]
headwater taxonomy    validate | resolve [--check]
                    | migrate <dir> [--to <version>] [--apply] [--now <date>]
                    | diff <dir> [--to <version>] [--now <date>]
                    | audit [--now <date>] [--record]
                    | publish [--package <name> | --from <dir>] [--assembly <name>] --out <dir>
                              [--clear-killed] [--json]
                    | vendor <dir> [--expect <digest>]
headwater coverage    [--format ...]
headwater probe       plan [--tier regression|campaign] [--arm present|absent]
                           [--category <name>] [--seed <n>]
                    | record <path>
                    | grade <path>
                    | stale
headwater json        field <key>...
                    | count [<key>...]
                    | quote
headwater help        [<verb> [<word>]]
headwater completions bash|zsh|fish|powershell
```

**This grammar is a statement of fact about the engine, and a name it declares either runs or waits.** Every verb the engine ships is above. `engine/crates/cli/tests/verbs.rs` holds this claim as a containment against `headwater_verbs::VERBS`. It names a verb the table carries when this block does not carry it. It also names a block entry when the table does not carry it and this paragraph does not declare the entry waiting. A name that the engine has not built stays here when something nameable would make it real. The engine then says what the name waits on when a caller types it. `query <expression>` is such a name, because no document states what an expression is. The verb ships the day one does. The same reading covers `coverage` and the five export targets that no consumer has asked for. Each of those states its wait when a caller types it. `taxonomy migrate` was such a name and it now runs. `--apply` rewrites the facet value of every document that a mechanical step covers, and the address of every overlay entry it covers. It emits the judgment steps as a task list. One of the three writes that the name waited on stays open. Nothing writes the open task set into the lock, and [spec 7](07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states) states why that omission is a decision. A name that nothing could make real has no place here, and `--changed-only` is the one such name this grammar carried. The test between the two is not how far away the work is. It is whether any document or any consumer could turn the name into a verb that runs.

**`--fix` writes the patch that rides with a finding, and the report a caller reads is the run after the write.** A finding carries a patch only under the [fixability bar](12-check-layer.md#fixability). So the flag decides no verdict, and it reaches no finding that carries no patch. It reaches no suppressed finding either, because the runner filters before a reader or a writer sees the list. Every patch is held against the bytes it names, and every result is read back. A document whose shape the engine guessed wrong is refused with nothing written. The batch lands as a set. The run opens every file it will write before it writes a byte. It reads each one back off the tree, and it restores what it wrote when a write fails. So a corrections patch is never applied in part. A refusal is not a finding and no flag softens one: the verb was asked to write and did not. The account of what was written goes to standard error, because `--format` puts one artifact on standard output.

**`new` is the one verb that writes a document, and five flags carry its rules.** `--title` is required, because the file name and the facet in the `name` role both come from it and the engine invents neither. `--summary` fills the facet in the `scent` role directly, exactly as `--title` fills the one in the `name` role. Without it the field carries a prompt for a person to answer. `--facet` states a value for any other facet the kind requires, as `<facet>=<value>`, and it is repeatable. The verb refuses a facet the kind does not require, a value outside a closed set, and a facet a declaration already decides. `--now` injects the clock that the two date facets take, on the terms [spec 12](12-check-layer.md#determinism-concretely) fixes for a check. `--relates` names a relation and the identifier at the other end, and it is repeatable. The verb refuses a relation that the taxonomy assigns to another creator, an end that the relation forbids, and a target that resolves to nothing. Where reciprocity is required, it writes the far half into the target document. [Spec 3](03-authoring-and-lifecycle.md#templates-and-scaffolding) states what it derives and what it leaves to a person.

**`new` also writes one line that is not a document, and `capture` is its only reader.** The capture-cost reading of a run goes to `.headwater/capture-cost.jsonl`, which is outside the corpus root and which no rule reads. [Spec 3](03-authoring-and-lifecycle.md#what-the-capture-cost-store-records-and-what-it-refuses-to) states what the line holds and what it deliberately does not. A run whose document landed and whose reading did not exits non-zero. That is the one place this verb reports over two artifacts at once. `capture` reads the store and the census together, and it writes nothing at all.

**`sweep` is two verbs and neither one reaches a model.** `plan` writes the briefing an agent reads, and `report` reads back the file the agent wrote. The part between them needs a model and no engine code performs it, so no build ever waits for one. Both exit 0 whatever they find, neither writes into the corpus, and the sampler is a crate that `headwater-check` cannot name. Both exit non-zero over a corpus that does not load, which is a fact about the caller rather than about a finding. [Spec 12](12-check-layer.md#four-things-stop-a-sweep-from-gating-and-none-of-them-is-a-rule-that-somebody-keeps) states what each of those four facts enforces.

**`taxonomy vendor` takes a path, and that is what keeps the network out of the engine.** [Spec 7](07-distribution-and-federation.md#consuming) says a consumer fetches a package and checks its digest. The fetch is the caller's, by whatever moves a directory in the organization that runs it, and the verb checks the bytes it is handed. A verb that took a location would need a client, and a client is a crate that opens a socket. So the [non-negotiable](00-vision-and-scope.md#non-negotiables) is a property of the argument rather than a rule that somebody keeps. `publish` is the other half, and it writes the artifact that `vendor` reads.

**`json` is the one verb that reads no corpus.** A harness hands a hook one JSON object on standard input. A hook that read it alone would need an interpreter that nothing else in a session requires. `field` prints one member, addressed by a path of keys. `count` prints how many elements the array or the object at a path holds, which is the read `field` cannot do. `quote` writes standard input back as one JSON string literal, for the object a hook writes to a harness. The reader is the loader this engine already carries, because JSON is a subset of the YAML 1.2 core schema. [HW-DR-0055](../decisions/0055-a-hook-reads-a-wire-format-through-the-engine-and-not-through-an-interpreter.md) rules why a verb answers this and an interpreter does not, and [spec 5](05-ai-integration.md#the-hook-contract-and-what-a-hook-cannot-bind) carries the term that constrains it.

**`conformance` reads a rule set the package ships, and both of its flags carry a rule.** The verb evaluates the repository against those rules and reports the level that the passing ones reach ([spec 7](07-distribution-and-federation.md#conformance)). With no flag it exits 0, on the terms `audit` exits 0: it measures an adoption and it gates nothing. `--level <name>` asks one question — is this repository at that rung — and it exits non-zero on a gap that no waiver covers. `--now` injects the clock that a waiver expiry is read against, so two runs over one tree at one date agree. The verb writes text and no other format, because the reader is a person closing a gap rather than a program.

**`probe` is four verbs, and none of them reaches a model.** `plan` fixes the six members of the run identity that a run inherits. It projects the cost against the tier's declared budget and refuses a run above it. `record` reads back the transcript that a recorder wrote. `grade` evaluates the declared expectations over that transcript, and it is the one verb of this binary that returns a verdict. `stale` holds the read set of every committed transcript against the tree in front of it, and reports which recorded results a change voided. The part between the plan and the record drives a session and observes it. No engine code performs that part, so no build waits for a model. All four verbs exit 0 whatever they find, none writes into the corpus, and the harness is a crate that `headwater-check` cannot name. An exit status that carried a rate would be a build that a model moves. Those are the four facts [spec 12](12-check-layer.md#four-things-stop-a-sweep-from-gating-and-none-of-them-is-a-rule-that-somebody-keeps) fixes for the sampler path.

**The recorder is a separate process for a reason a flag could not carry.** A transcript that an agent writes about its own session is a self-report, and [spec 5](05-ai-integration.md#a-transcript-is-recorded-from-outside-the-session-and-never-written-back-by-the-agent) refuses one. So the sweep's return-file shape does not transfer. No verb here writes a transcript, and there is no flag that makes one.

Neither verb grades. A result is a function of the transcript, the declared expectations and a grader version. The grader is a [correctness root](12-check-layer.md#the-correctness-roots) this engine does not carry.

The budget declaration is `.headwater/probe.yml`, beside the lock. It is outside the corpus root on the [same test](05-ai-integration.md#a-probe-result-is-citable-because-each-of-its-three-inputs-is-a-committed-artifact) that puts a transcript inside it. Nothing recomputes a budget from a committed source, because a budget is a policy rather than a measurement.

`export` is the projection contract under another verb, and `--check` is the same comparison that `generate --check` performs. It carries its own verb because a consumer outside the repository asks for one format at a time. The engine ships `json` and `jsonschema`, and each later format waits for a consumer who asks for it ([spec 13](13-open-obligations.md#what-waits-on-a-first-adopter)). `--profile` selects one declared export profile. With no profile named, the engine writes every declared profile, so a filtered audience is never omitted by accident.

**`--format` names one target, and it writes to standard output.** That is the consumer this verb exists for: somebody outside the repository who holds no clone, wants one vocabulary, and reads bytes on a pipe. No declared output path is involved, so a target that no taxonomy declared is still reachable. Without the flag the verb writes what the taxonomy declared, to the paths the taxonomy names, which is where `--check` has something to compare against.

**A refusal writes nothing to standard output, and it is never a document in the named target.** `--json` and `--format` name the shape of an artifact. Neither one moves the stream a refusal is written on, and neither one moves the grammar it is written in. So a refusal is one English sentence on standard error, which is the same rule that puts the account of `--fix` there. The property is about a refusal and not about a status. Four verbs hold a reason for exit 1 that is decided after the report is already written. Those runs put the whole report on standard output, and `check`, `export`, `gate` and `conformance` are the four. [Q43](09-decisions.md#q43--whether-a-refusal-under---json-is-a-json-document) rules it, and each interface contract states it for its own verb.

The CLI is advisory by default (exit 0 with findings on stdout). Use `--strict` for gates. The default is deliberate: a tool that blocks on first contact is removed, and a removed tool catches nothing.

**No flag decides which findings count, and there is no `--changed-only`.** A flag that took a caller's list of changed documents would put a second input into the verdict that no reviewer sees. It would also report the rest as neither checked nor skipped. A scope derived from content rather than from a list is the cache, and the cache ships. So the one job such a flag has is to pay for a 200 ms hook, and that job is either forbidden or already paid. [HW-OBL-0080](../obligations/0080-changed-only-is-the-content-addressed-cache-under-another-name.md) holds the measurement. An adopter who wants patient debt gets it from the [adoption payload](07-distribution-and-federation.md#first-contact-adoption-is-a-migration-from-no-taxonomy). That is a fact about the corpus, rather than a property of an invocation ([Q12](09-decisions.md#q12--migration-path-for-an-existing-corpus)).

**`--read-set` writes what the report already states.** A run reports the union of its in-scope inputs beside its coverage numbers ([spec 12](12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict)). The flag writes the same bytes to a file. The reader that needs them is a gate, which holds this run against a later tree and reads a file rather than a report. The flag decides no finding and it moves no verdict.

**`gate` is that reader, and spec 12 states the one test it runs.** It hashes the file at each listed path in the tree in front of it, and it reads nothing else. It walks no corpus, it resolves no taxonomy, and it exits non-zero on a verdict that does not carry. The report states the reach of its own answer. A list of the inputs that a run read carries no membership of the corpus. So a document that a merge adds reaches no such list.

### `taxonomy validate` versus `taxonomy audit`

There are two commands because there are two kinds of question. The distinction is the standard one between reasoning over a **TBox** (the terminology) and reasoning over an **ABox** (the assertions) against it ([spec 1](01-conceptual-model.md#two-layers-terminology-and-assertions)).

**`validate`** decides the schema alone: referential integrity, determinism, purpose completeness, kind rigidity, edge provenance, overlay confluence, core satisfiability. It needs no documents, always terminates in a verdict, and gates everything.

**`audit`** measures the schema *against a corpus*. Nine readings run. It measures facet differentiation and orthogonality, edge counts and staleness by `created_by`, and relation drift by family. It also measures the discriminator distribution of a heterogeneous shelf, the state-dwell distribution, and the warrant of every classified document. The eighth reading is the file names on every shelf that declares a layout. The ninth is the adoption payload of this run, against every reading that `.headwater/adoption.jsonl` holds. Its findings are advisory by construction, because a young or small corpus fails differentiation for reasons that are not defects. The findings are about the taxonomy, not the documents. A facet that nothing distinguishes is a schema problem that only documents can show.

Two of the readings this section named do not run. The report evaluates the wait of each one against the corpus in front of it, so a wait that a corpus has ended says so. Each prerequisite that a run does not find states where the absence lives. That matters because a declaration, an authoring pass and a decision are three different acts. Transition continuity waits on a facet in a role that spec 2's closed registry does not hold. Its absence is therefore a column rather than a row. Scent quality waits on a cue that nobody has authored on an edge instance ([HW-OBL-0023](../obligations/0023-no-corpus-has-authored-enough-cues-to-grade.md)). The promotion rate of [Q15](09-decisions.md#q15--a-synthesized-content-tier) is no longer one of them, and it is no longer a wait either. This verb reads one working tree, so it holds the denominator and it can never reach the numerator. `warrant.promoted` counts a promotion in the change that makes one, and the warrant reading names that rule beside the population it reports.

**A wait that a string literal states is a claim that no run re-derives.** The three readings above were three such literals until 2026-08-15. One of them said that no document of this corpus carried `warrant: asserted`, six already did, and the verb printed the sentence anyway. A reading that this verb does not take is now a list of prerequisites, and a run evaluates every one of them.

**A count copied into prose is the same defect at a smaller grain.** The change that built the reading above went on to commit its own count into five documents. Every one of them was false before the branch merged, because the change added a document to the population it counted. So a document that needs one of these figures names the verb that produces it. This specification states none of them.

**The warrant reading walks the closed set of four that [spec 3](03-authoring-and-lifecycle.md#the-warrant-and-what-each-value-requires) declares.** It follows the rule the creator reading follows. A value that no document states keeps its row, so a reader can tell an arm that is empty from an arm that is absent. Two rows stand at zero by construction, because the engine reads `regenerated` and `transcribed` off the generated-file marker rather than out of a declaration. The section states the `asserted` count as the denominator that a promotion rate divides by, and states that nothing declares a bar over it. A value outside the closed set is reported apart from the four rows, and that line is the only report of one. `warrant.promoted` is the one check that reads a warrant, and it reads a movement between two values rather than the values a corpus holds.

**The layout reading holds apart the arm that no declaration lets it measure.** A shelf layout names a file at birth, and no check reads one after that ([spec 3](03-authoring-and-lifecycle.md#templates-and-scaffolding)). The reading renders each name again, through the function that `headwater new` writes one with. A kind that declares no source for a placeholder puts its whole shelf outside the reading, and the row states where the absence lives. A document that the reading could not measure is in no numerator and no denominator. A missing declaration is not a name that drifted, and one figure over both would report the first as the second.

**One bar is declared, and it is what separates a finding from a distribution.** `stale_after_days` on the freshness facet is the one number a taxonomy states about these readings. So the one finding this verb produces is a relation with a half on a document past that window. Nothing states how narrow a facet may get before it separates nothing. Nothing states how low a capture rate may fall before a relation is unmaintained. A bar the engine invented would be a verdict derived from nothing a corpus declared. Every other reading therefore prints its population and carries no verdict, and [HW-OBL-0119](../obligations/0119-an-audit-reading-carries-no-declared-bar-so-a-distribution-cannot-become-a-finding.md) holds the gap.

**`audit` also writes one line that is not a document, and only `--record` writes it.** The adoption reading of a run goes to `.headwater/adoption.jsonl`, which is outside the corpus root and which no rule reads. [Spec 7](07-distribution-and-federation.md#what-the-adoption-store-records-and-what-it-refuses-to) states what the line holds and what it deliberately does not. Without the flag the verb writes nothing. The help text of `--now` promises that two audits of one tree at one date write the same bytes. The store refuses a second reading at one lock and one date, so that promise holds under the flag too. This verb reads one working tree, so the series over time is the one reading here that a run cannot take by itself.

**The grain of the creator reading is a relation, and never an edge.** [Q4](09-decisions.md#q4--relation-storage) keeps `created_by` on the relation type, so a scaffolded `supersedes` and a hand-typed one are one string on disk. A row presented per edge would state a provenance that nothing records. The reading walks the closed set of six creators rather than the values in use. A creator that no relation declares is the arm a comparison needs, and a report of the values in use omits exactly that.

The separation matters. `validate` must stay fast and total because it gates, while `audit` is a periodic design review with a tool attached. `audit` exits 0 whatever it finds, and no gate, no hook and no CI job runs it.

### Library

The CLI is a thin shell over a library API — load, graph, check, query, generate. Editor integrations, the MCP server, and CI adapters all consume the library directly. They do not start a subprocess and parse text.

### MCP server

The MCP server is the agent-facing surface of the same library ([AI integration](05-ai-integration.md)). `headwater mcp` starts it on standard input and output, over the corpus the same lock and the same `corpus:` block describe. It registers the whole query class, and with `--write` the working-tree write class beside it. Spec 5 gives the reason that the registration rather than the annotation is what carries the property.

The server walks the corpus once and reads the clock once, before it accepts a message. So every tool answers about one tree at one date. The `check` tool then answers what `check --format` answers, byte for byte. [What a `check` tool decides](05-ai-integration.md#what-a-check-tool-decides-and-where-each-decision-is-taken) states where each of those values is chosen, and none of them is chosen inside a tool.

**A call that moves a byte of that tree ends the server.** The walk behind every later answer would otherwise describe a tree that is gone. [What a session looks like after a write](05-ai-integration.md#what-the-working-tree-write-class-registers-and-what-a-session-looks-like-after-a-write) states the rule and the two halves that make it exact. The write tools hold the verbs of this binary as functions. So a tool call runs `new` and `check --fix`, rather than a second assembly of the same parts.

### CI adapters

The engine emits findings. Adapters translate them to the native vocabulary of a platform: annotations, check runs, job summaries, review comments. Adapters are thin and swappable so that no forge is privileged in the core. Portability is a requirement, not an aspiration. Coupling of the core to one CI platform is a stated failure mode that we correct ([spec 8](08-design-departures.md)).

**A renderer is what the engine ships, and an adapter is a renderer with a credential.** `check --format` writes four vocabularies and every one of them is neutral. SARIF is an OASIS standard, Markdown is a job summary or a review comment, and JSON is the finding shape [spec 4](04-assurance-model.md#findings) declares. What makes a forge a forge is the call that uploads the artifact, and the credential that the call carries. That call is outside the engine, and this repository's own workflow is one instance of it.

**Two of the four exist to hold the boundary open.** [Spec 8](08-design-departures.md) asks for more than one adapter from the start. One adapter cannot show where the boundary is, because everything a single adapter needs belongs in the core by definition. Two show it, and they show it by losing different things.

**Every format declares a loss set, and a census audits the claim.** This is the rule above, read one layer out. The subject is a field of a run rather than a class of the graph, and the rest of the sentence does not change. Every finding of a run reaches the output, or a declared reason accounts for it. A finding that no reason covers is a defect in the adapter, and it fails the run.

**A loss entry names where the value went, and the census reads it there.** An entry states a path into the artifact and the members under that path. The audit parses the emitted document and resolves each one. An entry is held when the artifact agrees with the run, and adrift when it does not. An entry that names no member of these bytes is counted as unaudited. The three outcomes sum to the entries declared, so nothing is passed over in silence. One entry stood wrong for as long as it existed. SARIF declared that the coverage went to a property bag. Four counts of it went there, and the skip classes went nowhere. An entry names its members now, so a bag that resolves is no longer an answer.

**A finding is still looked for by substring, and a prose format is held by a person.** The audit asks whether the artifact contains a rule name and a path anywhere in it. A format that prints an index of rules and an index of paths meets that test whatever its records hold. `HW-OBL-0110` records the measurement over the text report, and the other three formats print both indexes as well. A prose format declares entries that no member path can reach, so a reader is the whole of what holds that declaration.

**Three severity scales meet at a CI surface, and one of them reaches the output.** A check reports `error`, `warn` or `info` ([spec 12](12-check-layer.md#severity-is-the-checks-posture-is-the-controls)). An obligation carries `high`, `medium` or `low` ([spec 4](04-assurance-model.md#obligations-are-data)). A control carries a posture, which is `advisory` or `blocking`. SARIF adds a fourth scale over `error`, `warning`, `note` and `none`. The map runs from the check's scale alone: `error` to `error`, `warn` to `warning`, and `info` to `note`.

The other two scales are refusals rather than omissions. The obligation's scale describes the invariant and not the finding. It would give one level to every finding of every rule that serves one obligation, which is the control's judgment arriving inside the check's field. The posture says whether a finding stops a gate, and an artifact that stated it would be the engine ordering what lands. Both values still travel, in the members that the target keeps for what its own vocabulary does not name.

**A suppressed finding is in the output, and it is marked.** A live finding, a `migration-pending` finding and a suppressed one are three different things. A surface that showed the first alone reports a suppression that nobody can count. [Spec 12](12-check-layer.md#suppression-is-the-runners) rules that such a suppression is indistinguishable from a rule that never fires. SARIF marks the escaped ones with its own member, so a consumer shows them as dismissed rather than as open. The map is not total. SARIF names two places a suppression lives and spec 4 fixes three escape classes. So the two widest classes share one value, and the loss set records it.

**A run that names a change says so in every format.** `check --change` tells a run which documents one change carries, and the promotion count and every transition finding are about that set alone. The text report opens with the count of documents named, the paths that reached no row of the corpus, and the promotions. The other three carry the same values. `json` writes a member of its own, and `markdown` writes a paragraph and a list. SARIF writes the run's property bag, which its loss set records. A full-corpus run writes none of it, and the absence is what tells the two apart.

**A run emits what it evaluated, and never orders what lands.** Every run reports the corpus tree, the taxonomy lock hash, and its [read set](12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) beside the findings. A gate that holds the merge result can then decide, without a full re-run, whether the verdict still applies ([spec 4](04-assurance-model.md#a-verdict-is-about-one-state-of-the-corpus)). What the engine does not do is hold a queue, choose a landing order, speculate on a future state, or block a merge. A merge queue answers the merge question completely and pays for it with a serialized landing, and that trade belongs to the forge. This is the boundary that [Q7](09-decisions.md#q7--scope-of-the-mcp-surface) drew for the write path, at a second place.

## Performance targets

| Operation | Target |
|---|---|
| Full check, 1,000 documents, cold | < 5 s |
| Full check, warm cache | < 1 s |
| Change-scoped check (hook) | < 200 ms |
| Route query | < 100 ms |

Hooks and agent-facing queries must be fast enough to be invisible. A pre-commit check that costs two seconds is bypassed within a week. The developer who wrote the agent loop will skip a routing call that costs a second.

## Implementation constraints

- **Single binary or single runtime.** Installation of the engine must not require a toolchain per check. A mix of Python, Node, and shell — each with its own container fallback — is a cost that we do not accept.
- **Offline.** No network at check time.
- **Deterministic.** Same corpus, same lock, same output, byte for byte. This is what makes `--check` on projections meaningful.
- **Embeddable.** Usable as a library from an editor plugin or an agent process, with no need to spawn subprocesses.

These constraints made most of the choice. The language is Rust, and the embeddable requirement above is the argument that decided it ([Q1](09-decisions.md#q1--implementation-language)).
