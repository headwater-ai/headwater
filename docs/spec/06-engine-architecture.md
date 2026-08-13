---
id: SPEC-HW-engine-architecture
status: current
status_since: 2026-08-01
last_verified: 2026-08-11
summary: One parse, one typed graph, and many consumers, with the pipeline, the library boundary, and the performance targets.
doc_type: design_spec
sequence: 6
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: [claude-fable-5, claude-opus-5]
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cites_evidence:
    - EVAL-HW-first-contact
    - EVAL-HW-graph-export-and-federation
    - EVAL-HW-language-spike-results
    - EVAL-HW-shacl-worked-example
    - EVAL-HW-the-measurement-layer
    - EVAL-HW-the-serving-boundary
    - EVAL-HW-warrant-and-adjudication
    - EVAL-HW-what-a-check-can-know
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

**Cache** is content-addressed per file plus taxonomy hash, so incremental runs are proportional to the change, not the corpus. The change-scoped mode that CI and hooks use is the same code path with a smaller working set.

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

The engine implements these projection kinds: shelf indexes, shelf sections, relation views (decision lineage, traceability matrices), agent rule files, site navigation, graph export, coverage reports, and templates. A transcription of a pinned external snapshot is one more ([Q19](09-decisions.md#q19--inbound-integration-an-external-system-of-record)).

**Eight of the ten are declarable, and two are not.** A taxonomy names the kind and the output path, and [principle 1](00-vision-and-scope.md#design-principles) makes that path a schema decision. The coverage report and the corpus descriptor are the exceptions. [Spec 4](04-assurance-model.md#every-obligation-has-exactly-one-disposition) makes the register engine-defined and non-optional, and [Q20](09-decisions.md#q20--where-scent-lives) fixes the descriptor at `.headwater/corpus.json`. Both hold that standing for one reason. A reader who must consult the taxonomy to find an artifact already knows what it would tell them. So a declaration of either would put a second copy of one artifact at a path the engine did not fix. The meta-schema therefore closes the declarable eight as a value set, and `headwater generate` writes the other two under no declaration at all.

**A shelf index writes a bullet for each document, and a shelf sections file writes a heading.** A bullet carries no anchor, so nothing outside the index cites one row of it. A heading is an address, so a reader, an evaluation and an agent each cite the row rather than the file. That is the whole of the difference between the two kinds, and it is the reason they are two kinds. A member that changed the shape of one output would be read by one kind and ignored by the rest.

**A projection interpolates a path, a facet value and a declared identity, and nothing else.** The heading text is the value of the facet in the `name` role ([spec 2](02-taxonomy-model.md#the-meta-schema)). Two other sources are refused. An identifier is the stable choice and it is the artifact that a name exists to replace. A template on the declaration puts authored prose in a taxonomy source, and a taxonomy sits outside the corpus root. No census row covers it, no language regime binds it, and no rule reads its links. So prose that the corpus governs would move to the one file that the corpus cannot see. [Q4](09-decisions.md#q4--relation-storage) refused a second authoring location for an edge, and this is that refusal applied to prose.

**Every decline is whole.** A shelf that cannot name every document produces no file, and the run names the document that stopped it. A file with one section missing hides the document that it dropped. A citation into that section then fails, and nothing anywhere states why. Two documents may not carry one name either, for the same reason: two headings with one text share one anchor.

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

### An export is a projection, and it declares what it dropped

A graph export is a projection like the others. The taxonomy declares its output path, so whether an export is committed is a schema decision and not an engine default ([principle 1](00-vision-and-scope.md#design-principles)). A declared export is held to regeneration by `generate --check`, exactly as a shelf index is.

Exports fall into two classes, and only one class preserves fidelity.

- **The native graph export** carries the property graph with no loss, and that includes the instance attributes on edges ([Q4](09-decisions.md#q4--relation-storage)). It is what the federation tier reads ([spec 7](07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it)).
- **An interoperability export** is lossy by construction. RDF, SKOS, LinkML, SHACL, JSON Schema and OKF each speak a vocabulary that cannot carry everything in the graph.

So every emitter declares a **loss set**: the node classes, edge classes, and attributes that its target cannot carry, each with a reason. Every export run then emits a **projection census**. Every node and every edge in the graph is either present in the output, or accounted for by a declared loss reason. An omission that no reason covers is a projector defect, and it fails the run.

**A node of that census is a classified document or an external anchor, and an identifier is not what makes one.** A typed document that declares no identifier never reaches the identifier index, and it is still content that leaves a corpus or does not. A census over the index alone would let an emitter drop every unidentified document and report itself complete.

That is the coverage doctrine of [spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for), applied one layer out. It answers the trust problem that the [SHACL evaluation](../evaluations/shacl-worked-example.md#problem-one-everything-downstream-trusts-the-projection-and-shacl-does-not-check-it) found. The projector was the component that everything downstream trusted and nothing could check. A round trip is the wrong instrument for the lossy class, and an earlier draft of [Q6](09-decisions.md#q6--where-the-corpus-graph-lives-at-rest) asked for one. The native export keeps its round-trip test, because an empty loss set is exactly what a round trip proves.

**An emitter that cannot carry the warrant does not carry the content.** Every node carries a [warrant](01-conceptual-model.md#warrant), and the native export carries it with no loss. A target vocabulary that has no place for it produces an artifact in which unwarranted content is indistinguishable from accepted content. A loss-set entry reaches the consumer who reads the loss set and nobody else. The field also reports that ordinary tooling strips a mark which travels beside content ([spec 11 §P](11-adjacent-work.md#p--provenance-endorsement-and-the-record-of-a-judgment)). So such an emitter **withholds** every `asserted` and `transcribed` node, at the profile's declared tombstone grain, with its own inability to mark as the reason. That is [principle 7](00-vision-and-scope.md#design-principles) read the way that a filtered exporter reads it. An unmarked assertion is unrecoverable, and a withholding is visible and cheap ([Q15](09-decisions.md#q15--a-synthesized-content-tier)).

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

**The generation time is injected, and that is what lets it coexist with a byte gate.** The rule above and the regeneration gate look incompatible. A time inside an output moves on every run, so `generate --check` reports drift over a corpus that nobody touched. They hold together because they cover two artifacts. A **committed** export is held to regeneration and carries no time at all. An export that **leaves** the repository is the artifact the rule above is about, and `headwater export --at <date>` supplies its time. The clock is thus a value that a caller injects. [Spec 12](12-check-layer.md#determinism-concretely) injects it into a check for the same reason, rather than let one read a syscall. Same corpus, same lock, same injected clock, byte-identical output, under both.

**The tombstone grain is declared, because the two things that a filtered view owes a reader are in tension.** A view must not look complete, and a report of what it withheld is itself a disclosure. Both cannot hold in full. The freedom-of-information statutes reached this exact conditional from the other direction, and so did the multilevel-security literature ([spec 11 §O](11-adjacent-work.md#o--the-serving-boundary-descriptors-redaction-and-the-write-path)).

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
headwater check      [--changed-only] [--strict] [--read-set <path>]
                     [--format text|json|sarif|markdown]
headwater gate       --read-set <path> [--now <date>]
headwater generate   [--check]
headwater new        <kind> [--title ...]
headwater route      <task description>
headwater query      <expression>
headwater explain    <path|identifier>
headwater mcp
headwater export     [--profile ...] [--format json|jsonschema|shacl|rdf|skos|okf|linkml]
                     [--at <date>] [--check]
headwater taxonomy   validate | resolve | diff | migrate | audit
headwater coverage   [--format ...]
headwater probe      [--tier regression|campaign] [--arm present|absent] [--category ...]
```

`probe` is the one verb that reaches the network, so it never runs inside `check` and never gates ([spec 5](05-ai-integration.md#two-tiers-and-the-cadence-follows-the-purpose)). It projects the cost of a run against the tier's declared budget and refuses a run that exceeds it. Every run writes its transcript and reports the run identity, the realized cost, and the interval around each rate.

`export` is the projection contract under another verb, and `--check` is the same comparison that `generate --check` performs. It carries its own verb because a consumer outside the repository asks for one format at a time. Only `json` and `jsonschema` ship in the first release, and each later format waits for a consumer who asks for it ([spec 13](13-open-obligations.md#what-waits-on-a-first-adopter)). `--profile` selects one declared export profile. With no profile named, the engine writes every declared profile, so a filtered audience is never omitted by accident.

**`--format` names one target, and it writes to standard output.** That is the consumer this verb exists for: somebody outside the repository who holds no clone, wants one vocabulary, and reads bytes on a pipe. No declared output path is involved, so a target that no taxonomy declared is still reachable. Without the flag the verb writes what the taxonomy declared, to the paths the taxonomy names, which is where `--check` has something to compare against.

The CLI is advisory by default (exit 0 with findings on stdout). Use `--strict` for gates. The default is deliberate: a tool that blocks on first contact is removed, and a removed tool catches nothing.

**No flag decides which findings count.** `--changed-only` scopes the work and never the verdict, because a full run over the same tree and the same lock reaches the same result. That is what makes it sound for a 200 ms hook. A mode that evaluated only newly touched documents would be a second input to the verdict that no reviewer sees. It would also report the rest as neither checked nor skipped. An adopter who wants patient debt gets it from the [adoption payload](07-distribution-and-federation.md#first-contact-adoption-is-a-migration-from-no-taxonomy). That is a fact about the corpus, rather than a property of an invocation ([Q12](09-decisions.md#q12--migration-path-for-an-existing-corpus)).

**`--read-set` writes what the report already states.** A run reports the union of its in-scope inputs beside its coverage numbers ([spec 12](12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict)). The flag writes the same bytes to a file. The reader that needs them is a gate, which holds this run against a later tree and reads a file rather than a report. The flag decides no finding and it moves no verdict.

**`gate` is that reader, and spec 12 states the one test it runs.** It hashes the file at each listed path in the tree in front of it, and it reads nothing else. It walks no corpus, it resolves no taxonomy, and it exits non-zero on a verdict that does not carry. The report states the reach of its own answer. A list of the inputs that a run read carries no membership of the corpus. So a document that a merge adds reaches no such list.

### `taxonomy validate` versus `taxonomy audit`

There are two commands because there are two kinds of question. The distinction is the standard one between reasoning over a **TBox** (the terminology) and reasoning over an **ABox** (the assertions) against it ([spec 1](01-conceptual-model.md#two-layers-terminology-and-assertions)).

**`validate`** decides the schema alone: referential integrity, determinism, purpose completeness, kind rigidity, edge provenance, overlay confluence, core satisfiability. It needs no documents, always terminates in a verdict, and gates everything.

**`audit`** measures the schema *against a corpus*. It measures facet differentiation and orthogonality, edge counts and staleness by `created_by`, and relation-choice drift by family. It also measures discriminator distribution on heterogeneous shelves, state-dwell distribution, transition-continuity distribution, and scent quality. Its findings are advisory by construction, because a young or small corpus fails differentiation for reasons that are not defects. The findings are about the taxonomy, not the documents. A facet that nothing distinguishes is a schema problem that only documents can show.

The separation matters. `validate` must stay fast and total because it gates, while `audit` is a periodic design review with a tool attached.

### Library

The CLI is a thin shell over a library API — load, graph, check, query, generate. Editor integrations, the MCP server, and CI adapters all consume the library directly. They do not start a subprocess and parse text.

### MCP server

The MCP server is the agent-facing surface of the same library ([AI integration](05-ai-integration.md)). `headwater mcp` starts it on standard input and output, over the corpus that the same lock and the same `corpus:` block describe. It registers the query class alone, and spec 5 gives the reason that the registration rather than the annotation is what carries the property.

### CI adapters

The engine emits findings. Adapters translate them to the native vocabulary of a platform: annotations, check runs, job summaries, review comments. Adapters are thin and swappable so that no forge is privileged in the core. Portability is a requirement, not an aspiration. Coupling of the core to one CI platform is a stated failure mode that we correct ([spec 8](08-design-departures.md)).

**A renderer is what the engine ships, and an adapter is a renderer with a credential.** `check --format` writes four vocabularies and every one of them is neutral. SARIF is an OASIS standard, Markdown is a job summary or a review comment, and JSON is the finding shape [spec 4](04-assurance-model.md#findings) declares. What makes a forge a forge is the call that uploads the artifact, and the credential that the call carries. That call is outside the engine, and this repository's own workflow is one instance of it.

**Two of the four exist to hold the boundary open.** [Spec 8](08-design-departures.md) asks for more than one adapter from the start. One adapter cannot show where the boundary is, because everything a single adapter needs belongs in the core by definition. Two show it, and they show it by losing different things.

**Every format declares a loss set, and a census audits the claim.** This is the rule above, read one layer out. The subject is a field of a run rather than a class of the graph, and the rest of the sentence does not change. Every finding of a run reaches the output, or a declared reason accounts for it. A finding that no reason covers is a defect in the adapter, and it fails the run.

**Three severity scales meet at a CI surface, and one of them reaches the output.** A check reports `error`, `warn` or `info` ([spec 12](12-check-layer.md#severity-is-the-checks-posture-is-the-controls)). An obligation carries `high`, `medium` or `low` ([spec 4](04-assurance-model.md#obligations-are-data)). A control carries a posture, which is `advisory` or `blocking`. SARIF adds a fourth scale over `error`, `warning`, `note` and `none`. The map runs from the check's scale alone: `error` to `error`, `warn` to `warning`, and `info` to `note`.

The other two scales are refusals rather than omissions. The obligation's scale describes the invariant and not the finding. It would give one level to every finding of every rule that serves one obligation, which is the control's judgment arriving inside the check's field. The posture says whether a finding stops a gate, and an artifact that stated it would be the engine ordering what lands. Both values still travel, in the members that the target keeps for what its own vocabulary does not name.

**A suppressed finding is in the output, and it is marked.** A live finding, a `migration-pending` finding and a suppressed one are three different things. A surface that showed the first alone reports a suppression that nobody can count. [Spec 12](12-check-layer.md#suppression-is-the-runners) rules that such a suppression is indistinguishable from a rule that never fires. SARIF marks the escaped ones with its own member, so a consumer shows them as dismissed rather than as open. The map is not total. SARIF names two places a suppression lives and spec 4 fixes three escape classes. So the two widest classes share one value, and the loss set records it.

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
