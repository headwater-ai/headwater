---
id: HW-SPEC-distribution-and-federation
status: current
status_since: 2026-08-01
last_verified: 2026-08-17
summary: How a publisher ships a taxonomy, how a consumer overlays it, and what the invariant core requires of the result.
doc_type: design_spec
sequence: 7
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: [claude-fable-5, claude-opus-5]
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cites_evidence:
    - HW-EVAL-default-taxonomy-first-run
    - HW-EVAL-first-contact
    - HW-EVAL-graph-export-and-federation
    - HW-EVAL-the-serving-boundary
    - HW-EVAL-warrant-and-adjudication
---

# 7 — Distribution and federation

One organization defines a documentation method. Many repositories adopt it. It evolves. Everyone must be able to take the evolution without loss of what they customized. Also, the publisher must be able to tell who actually did.

## What is shared, and what is not

| Layer | Shared | Owned locally |
|---|---|---|
| Engine | Yes — a versioned dependency | — |
| Taxonomy schema | Yes — a versioned package | Overlays |
| Doctrine (prose that explains the method) | Yes — vendored or linked | Local method notes |
| Corpus content | No | Everything |
| Router / entry point | No — it describes *this* repository | Yes |
| Control register | Partly — the publisher's obligations are inherited | Local controls and waivers |

This distinction makes the problem tractable: **the taxonomy is a package, not a copy.** Prior systems vendored checksummed file trees and gated on byte-identity. That works only while nobody needs to customize. Here, customization is expressed as an overlay against a versioned base. Thus an upgrade is a package bump and a re-resolve, not a merge conflict with a file that you were never supposed to edit.

## Publishing

A publisher repository declares a **taxonomy package**:

```yaml
package: acme/headwater-taxonomy
version: 3.2.0
requires_engine: ">=1.4 <2"
contents:
  taxonomy: taxonomy.yml
  conformance: conformance.yml # the rules an adopter is evaluated against, and the levels over them
  doctrine: doctrine/          # prose explaining the method, vendored to consumers
  templates: templates/
  plugins: plugins/            # organization-specific checks
profiles: [service-repo, docs-only, platform]   # named overlays that remove
bundles: bundles/            # named overlays that add, selected at init
interview: interview.yml     # the questions init asks, and the bundle each answer selects
migrations: migrations/
```

Publishing is a release: a semantic version, a changelog, an integrity digest, and a migration payload for any major bump. Distribution is over the registry or repository that the organization already uses. The engine requires only that it can check the digest of a version that somebody fetched.

`headwater taxonomy publish` writes the artifact. It is a directory, because the engine carries no archive format and needs none: whatever moves a directory in the organization moves this one. Beside the manifest it writes a **release record**. The record names every file in the artifact with the digest of its bytes, and it carries one digest over that list. The record does not cover itself, so the digest is over what the artifact holds rather than over the file that states it. So the header of the record is outside the digest. `headwater taxonomy vendor` does not read the package identity from it. It takes the name, the version and the engine range from the manifest, which the digest does cover. It refuses a record whose header disagrees with the manifest.

**Every `contents` path a publisher writes is read.** `taxonomy`, `bundles`, `conformance` and `migrations` each reach a verb. A key that no verb reads is a claim that a publisher makes and a consumer never sees. That is the defect `requires_engine` refuses from the other side, and `contents.migrations` was it until [the payload](#the-migration-payload) had a reader. A value under `contents` names one path, and the kind of that path is the kind its reader opens. `taxonomy` and `conformance` each name a file, because each one is read as text. `bundles` and `migrations` each name a directory, because each one is read as a listing. A publish refuses a value of the other kind, and it refuses an empty value before either check. An empty value names the package directory, which no key means. A list or a mapping there is refused, because no verb reads one and the rules that hold a path cannot hold it.

**A `contents` key states what the engine reads, and not what the artifact carries.** Publication takes the package directory whole, and it takes the bundle tree as well where `contents.bundles` names one outside the package. So the artifact carries a file that no key names. A key that names a directory holding no file points at nothing the artifact carries. The two readings are separate. The block above is a list of what the engine reads, and not an inventory of the artifact.

**Publication judges a `contents` path by where it resolves, and not by the string a manifest writes.** A path that leaves the package and returns inside it is not refused. A symlink is judged by its target, so a name with no `..` in it can still resolve outside the package. `contents.bundles` may still resolve outside the package, because `publish` copies the bundles into the artifact and rewrites that one scalar. The bound on it is the repository rather than the file system. Every other key is held to the package, symlink or not. A file the walk meets that no `contents` key names is held to the same rule. So an undeclared symlink outside the package is refused as well ([#303](https://github.com/headwater-ai/headwater/issues/303)).

**Nothing is written until everything is read.** `publish` holds every path a manifest declares to the tree before it reads one of them, and before it creates the output directory. A declared path that is not there is a refusal, under every key and not only the keys that publication carries. It names the manifest, the key and the declared value, rather than a file system error out of a copy. A file whose name is not UTF-8 is a refusal too, because the release record names every member as text. A write that fails after that returns the output directory to the state the run found it in.

**The output path is one that the run can observe, or the publish does not start.** It has to be an empty directory or nothing at all. A directory that holds files is refused, and so is a directory that this process cannot read. A link whose target is not there is refused too, because a read of it reports the same result as an absent path. Every state the undo returns to is therefore a state the run observed, and nothing the undo removes belonged to anybody else. The undo removes the output directory and each directory that the run made above it, and nothing else.

**`requires_engine` is read, and a package outside the range is refused before a source is loaded.** The range is a list of comparators over the engine version, written as `">=1.4 <2"`. A range that the engine cannot read is refused rather than ignored. An unreadable range that reads as no range is a claim that the publisher made and the consumer dropped. An engine that resolved a package built for a later one would produce a lock that nobody can reproduce.

**A package states its version in two files, and the two are held to each other.** The manifest carries a `version` key, and the [meta-schema](02-taxonomy-model.md#the-meta-schema) requires a `version` at the root of a taxonomy source. So one value stands in two places, and the artifact carries both, and the release digest covers both. The engine refuses a package whose two declarations disagree, and the refusal names both files and both numbers. It fires when the package loads, so it does not depend on which of the two numbers a consumer pinned. The manifest is what every consumer-facing reader takes the version from: the pin comparison, the lookup that `headwater init` makes, and the release record. Nothing held the two together until the numbers were measured apart. The disagreement reached the lock, where it stood in two blocks of one file.

**A package states its name in two files as well, under two different keys, and the two are held to each other.** The manifest carries a `package` key, and the [meta-schema](02-taxonomy-model.md#the-meta-schema) requires a `taxonomy` key at the root of a taxonomy source. That root may not carry a `package` key at all, because `package` is a reserved reference root there. So one name stands in two places under two spellings, and the artifact carries both, and the release digest covers both. The engine refuses a package whose two declarations name different things, and the refusal names both files, both keys and both names. It fires when the package loads, and before the comparison of the versions. A package that is not the one you asked for makes the question of its version moot. Every consumer-facing reader takes the name from the manifest: the lookup under `packages/`, the release record, the vendor target directory and the corpus descriptor. The key at the root of the source reached the lock and the published artifact, and it decided nothing.

**A package name is one or more segments separated by `/`, and each segment opens and closes with a letter or a digit.** Between those, a segment holds letters, digits, `.`, `-` and `_`. `headwater taxonomy vendor` refuses a name outside that grammar, because the name gives the directory this verb creates under `packages/`. A value such as `..` or `.` names a directory that belongs to the adopter rather than to the package. No registry carries a publisher from a name outside the grammar to one inside it.

**The name becomes the directory by one substitution, and that substitution is not injective.** `vendor` replaces each `/` with a `-`, so `acme/my-taxonomy` and `acme-my/taxonomy` both name `packages/acme-my-taxonomy`. Two publishers reach one directory, and neither of them has to be an adversary. `headwater taxonomy vendor` refuses the second name where the directory holds a different package, and the refusal names both packages and the directory. The lookup under `packages/` reads the `package:` of each manifest and never the name of the directory. So an adopter moves the directory aside, and the engine still finds the package that was there.

**That rename works once, and the specification states the rest of its cost because the refusal does not.** The adopter moves the first package aside, and the second package installs into the cleared path. After that, `headwater taxonomy vendor` refuses every later artifact of the first package, because each one derives the same directory. The adopter can move the second package aside as well. Two directories then declare one name, and the lookup takes the one that sorts first. So an adopter can upgrade exactly one of two packages that contend for a directory ([#354](https://github.com/headwater-ai/headwater/issues/354)).

**A package imposes nothing on a derived taxonomy, and that is a constraint rather than a courtesy.** An overlay is a patch, so a consumer's resolved taxonomy and lock contain the base content of the package. Terms on the package that a derived work inherits therefore reach an artifact that [spec 0](00-vision-and-scope.md#who-this-is-for) promises is the adopter's own. So the terms of a `taxonomy`, `bundles` or `profiles` path may not condition what a consumer does with the resolved result. The `doctrine` path is prose that a consumer vendors, and it takes its own terms. Headwater checks none of this, and [spec 6](06-engine-architecture.md#what-a-filtered-export-claims-and-what-it-does-not) already states that it checks nothing about a license. What the specification states is the requirement that a publisher must meet ([Q11](09-decisions.md#q11--license-and-distribution-posture)).

### The migration payload

`contents.migrations` names a directory inside the package, and each file in it is one transition. A file states the versions it moves between as two ranges, which the one range reader of this engine reads. `headwater taxonomy diff` selects the file whose two ranges hold the version this repository takes and the version the artifact declares.

```yaml
migration:
  format: 1
  from: ">=1 <2"
  to: ">=2 <3"

steps:
  - subject: facet_value
    facet: status
    from: current
    to: [settled, provisional]
    task: Say whether the argument of this document is closed.
    because: One live state held two states that a reader acts on differently.
```

**No key states whether a step is mechanical, and the target list decides it.** The apparent shape of a payload is a rename map with a list of judgment tasks beside it. [The schema-format walkthrough](../evaluations/schema-format-walkthrough.md) found the case that breaks that shape: "one old value maps to a set, and the author chooses". Such a step is a rename in every respect but the one that decides whether a program may apply it. So the split of [spec 2](02-taxonomy-model.md#versioning-by-measured-compatibility) runs through a step, and never between two lists.

One target is mechanical, and the engine applies it. Two or more are a closed choice, and the author picks one value out of the set that the publisher closed. No target is a re-statement, and nothing replaces the old value. A step that leaves a choice or that asks for a re-statement carries a `task`, which is the question the author answers. A `task` beside one target is refused, because a mechanical step asks nobody anything. A key that this engine reads and drops is the defect that a payload exists to stop.

An absent `to` and `to: []` are two different statements. A reader that made them one would turn a forgotten target into a re-statement task. An absent `to` is a publisher that did not say, and it is refused. `to: []` is a publisher that says that nothing replaces the value.

**A step declares no dimension, and its subject fixes one.** A `remedies` key would be a claim that a publisher writes and that nothing measures. The engine holds the mapping instead, and no payload may vary it.

| subject | what it moves | the dimension it is a remedy for |
|---|---|---|
| `facet_value` | one value of one facet, and every document that carries it | `instance_validity`, and `consequence` with it |
| `kind` | one kind, and every document that the census typed as it | `classification` |
| `overlay_address` | one path into the taxonomy, and every entry of the adopter's overlay addressed at or under it | `addressability` |

`consequence` stands in the first row because [spec 2](02-taxonomy-model.md#versioning-by-measured-compatibility) rules that one dimension holds the other: "a broken `instance_validity` breaks `consequence` too".

**Two of the six dimensions have no subject, and each absence is a statement.** No rename is a remedy for `projection`, because a projection that moved is written again by `headwater generate` rather than edited. No rename is a remedy for `identifier`. [Spec 3](03-authoring-and-lifecycle.md#identifiers) makes an identifier a stable name that survives a move and a rename. A payload that renamed one would move the thing the identifier holds still.

**The third subject names a path and not a value.** The `from` of an `overlay_address` step is an address, and so is every entry of its `to`. Both are read by the one path production this engine has ([spec 2](02-taxonomy-model.md#the--reference-sublanguage)). A step covers each operation of the adopter's overlay whose own address it is a prefix of. The prefix is over segments and never over text, so a step over `kinds.play` covers `kinds.play.purpose` and misses `kinds.playbook`. `headwater taxonomy migrate --apply` rewrites the address of each entry it covers, and no byte beside it.

**The overlay it writes is the adopter's own, and never a bundle.** A bundle is package content, and [`headwater taxonomy vendor`](#waivers) replaces a vendored package directory whole. An address rewritten inside a bundle is lost at the next upgrade, and the publisher of the bundle is who moves it. A bundle whose address stopped reaching a declaration reaches the consumer through the `addressability` dimension instead. An `add` over a path the new base no longer declares does not fail to address anything, and [spec 2](02-taxonomy-model.md#customization-by-composition) states what it does instead.

**The two ends of the wire check different halves, and neither one can check both.** `headwater taxonomy publish` reads every payload before it writes a file. It refuses a step whose source the taxonomy under publication still declares, because such a step renames something that did not move. It refuses a step whose target that taxonomy does not declare, and a payload whose ranges do not hold the version under publication. `headwater taxonomy diff` reads the payload out of the artifact and reports what it means for this corpus. A step that names a value which this repository does not hold is reported and never refused. The old taxonomy a consumer holds is the base under its own overlays, and an overlay may have removed that value.

**What the report states is measured, and never asserted.** The documents that a step names come from the census of the taxonomy this repository takes. The documents that moved come from the two dimensions that a subject names. `classification` answers with each document that stopped resolving to the kind it had. `instance_validity` answers with each document that stopped validating. The denominator is the union of the two. Each set comes out of the comparison that decided its own dimension, rather than out of a second pass over it. So the report says how many of the documents that moved lie under a step, and it names every document that lies under none. A payload that names all of them therefore answers to a corpus that its publisher never saw.

## Consuming

A consumer declares what it takes and how it differs:

```yaml
taxonomy:
  package: acme/headwater-taxonomy
  version: 3.2.0
  digest: sha256:6c2f…
  profile: service-repo
  overlay: .headwater/overlay.yml
```

Consuming is two steps, and the split is what keeps the network out of the engine. The caller fetches the artifact, by whatever the organization already uses. `headwater taxonomy vendor` then checks the fetched directory against the `digest` above and installs it. `headwater taxonomy resolve` merges the overlay, validates, and writes the lock. The lock is committed. Thus the corpus is checked against a resolved, reviewable, reproducible taxonomy, and CI needs no network to check anything.

### What a package digest proves, and what it does not

**It proves that the artifact is the one the consumer pinned.** `vendor` recomputes the digest from the bytes on disk and refuses an artifact that does not match. The message names three kinds of divergence. A file that moved, a file the record names that is absent, and a file present that the record names no member for. The third is the one that a comparison over the record alone would miss. A record cannot report a file that it never named.

**It does not prove that the publisher wrote the header of the record.** The digest covers the member files and not the record that lists them. So `vendor` takes the package identity from the manifest, and it refuses a header that disagrees. `taxonomy diff` and `taxonomy migrate` read the header instead, and they read it against no pin. What those two report about a package rests on the artifact and not on a digest.

**The pin is authored, and no verb writes it.** A digest that the engine recorded from whatever it had just received would be a pin against itself. So `vendor` refuses to run when no pin exists, and it names the field to write. The publisher states the digest where a consumer reads it, and the artifact and the digest travel apart.

**It does not prove who published the artifact.** Nothing here is a signature, so the first fetch rests on the channel that carried the digest. A signature needs a key, a route that distributes the key to an adopter who has never met the publisher, and a rule for revocation. None of the three is decided ([Q22](09-decisions.md#q22--the-integrity-posture-of-a-published-package)), and [HW-OBL-0115](../obligations/0115-a-pinned-digest-authenticates-the-pin-and-never-the-publisher.md) holds the question rather than a manifest key that would read as an answer.

**A vendored package is not a maintained one.** `vendor` replaces a directory that carries a release record, and it refuses a directory that carries none. A package that a person maintains is a publisher's source, and a consumer command that overwrote one would delete the thing being published. Spec 2 requires customization by overlay and never by fork, so a vendored directory has nothing in it that an adopter should have edited.

**The replacement never leaves part of a package behind.** `vendor` writes the new package beside the old one, and then puts the new one in place. So `packages/<name>` holds a complete package or nothing at every moment, under a failure and under a run that a person stops. A `vendor` that fails leaves the package that was installed, and the refusal says which state the directory is in.

**A run that a person stops can leave a directory beside the package.** `packages/<name>~aside` holds the package that was installed, and the engine still finds it there. `packages/<name>~staged` holds the copy the run was making, and that copy can be a part of a package ([#357](https://github.com/headwater-ai/headwater/issues/357)). The next `vendor` of the package removes both. `vendor` refuses to install either of them as an artifact, so an adopter who finds one removes it.

### Profiles are publisher overlays

Not every repository holds every shelf. A profile is a **named overlay that the publisher ships**. It contains `remove` operations for the shelves that a repository archetype does not have, and it is selected by name in the consumer declaration above. It is not a separate mechanism. The overlay resolver already implements every part of it (dependent-key deletion, confluence, core satisfaction on the result).

An earlier draft listed profiles as their own declaration, and thus kept two names for a subset of one mechanism. The effect is unchanged. A repository never has a rule, glob, or projection that targets a shelf that is not present. Dead configuration is noise that teaches readers to ignore configuration.

### Bundles are publisher overlays in the other direction

A **bundle** is a named overlay that the publisher ships, which holds `add` operations for optional content. A profile removes what an archetype does not have. A bundle adds what an adopter needs. One mechanism, two conventional directions, and neither one is new.

```yaml
bundles:
  procedure:  {requires: []}
  standards:  {requires: []}
  evidence:   {requires: []}
  proposals:  {requires: []}
  operations: {requires: [procedure]}
  compliance: {requires: [standards, evidence]}
```

Two rules keep this cheap, and both run on machinery that exists.

**A bundle holds no `override` and no `remove`.** If a bundle needs to change the base, the base declared something that it should not have. Add-only overlays over disjoint addresses commute, so the resolver's static confluence check proves that every subset of bundles resolves. The publisher runs that check once per release, and no adopter can then select a combination that fails.

**A bundle declares its closure, and the publisher checks it at release.** Enabling a bundle is one operation for the adopter, whatever it contains. The dependency list above is data, not documentation.

This is what fixes the size of the base package. A large base forces bundles and profiles to remove, and `remove` carries dependent-key deletion and the most failure modes of the three operations. A minimal base lets every bundle stay add-only. The [first-run walkthrough](../evaluations/default-taxonomy-first-run.md) derives the base from the core on those terms, and it measures what each of five adopters authors and deletes.

### The starter kit is a selection

[Spec 0](00-vision-and-scope.md#what-we-build) promises a doctrine starter kit, and [spec 2](02-taxonomy-model.md) refers to a base package. These are two artifacts, and an earlier reading of [Q3](09-decisions.md#q3--how-much-of-the-default-taxonomy-ships-in-the-box) treated them as one. They answer opposite requirements. The base has to be minimal so that bundles stay add-only. The starter kit has to be opinionated so that a new adopter does not face a blank schema.

So `headwater/starter` is the base package, a named bundle selection, and the doctrine prose that explains the selection. Nobody is expected to run the base bare. Everything composes over it.

### The interview

`headwater init` composes a bundle selection from answers. It is the first-run surface, and the blank-schema problem is a first-run problem. Five rules govern it, and the walkthrough derives each one.

- **It emits an overlay, never a resolved taxonomy.** A tool that writes a complete taxonomy file forks the adopter from the base before they write a document. Every later upgrade is then a merge. Spec 2 requires customization by overlay and never by fork, and this is the one place where a breach of that rule stays invisible.
- **It is package data, not engine code.** [Principle 1](00-vision-and-scope.md#design-principles) puts anything an adopter might want different into the schema. An interview compiled into the engine cannot ship with a third-party package, and a publisher with its own bundles needs its own questions. The interview sits beside profiles and templates in the package. It is not a taxonomy declaration, because it describes the package rather than the corpus, so the count of declarations stays at thirteen.
- **It is `headwater infer` with a second evidence source.** [Q12](09-decisions.md#q12--migration-path-for-an-existing-corpus) makes `infer` propose a taxonomy from a tree that already exists. Both emit the same artifact, so they are one command with two inputs. On an empty repository the tree contributes nothing and the interview asks everything. The blank-schema case is thus the degenerate one rather than a special one.
- **It emits three artifacts, and one read of the tree produces all three.** The overlay above. A report of what the tree holds that no proposed shelf or kind explains. And the [adoption payload](#first-contact-adoption-is-a-migration-from-no-taxonomy), which is the set of findings that the proposal expects to fail. `infer` cannot weaken the base to fit the corpus, and that is structural rather than a rule to police. A bundle selection is add-only, and an add-only overlay carries no operation that removes a base rule.
- **Every question is about the corpus, and none is about the taxonomy.** "Do you write runbooks?" needs no model in the reader's head. "Do you want a `procedure` purpose?" needs the whole of spec 2 first. Each answer selects a bundle or supplies a value that the corpus alone holds, and no answer exposes a declaration name.
- **A question that an existing ruling answers is deleted, and a question that no ruling can answer is asked.** [Spec 3](03-authoring-and-lifecycle.md#identifiers) rules that an identifier always carries a namespace, so the interview never asks whether the adopter wants identifiers. It asks what the namespace is. No package can supply that value. A package that named one would give it to every corpus that adopts it, and a stand-in such as `repo` names nobody at all. The interview is the first moment at which the owner of the corpus is present to answer. It is the one answer that selects no bundle, and it lands in the overlay as `identifier_schemes.<scheme>.namespace` on every scheme the selection reaches.

The interview asks only what changes the selection, and the one value that no package can hold. Everything else waits for a corpus that `taxonomy audit` can measure, because a day-one guess about facet orthogonality is worse than a day-thirty measurement of it.

The cost is the one that overlays already carry, one level up. A resolved taxonomy is an artifact that nobody authored directly, and an interview adds a step where nobody authored the answers as configuration either. So the generated overlay carries a comment above each block that names the question and the answer which produced it. Re-running `init` re-asks with the current answers as defaults and rewrites the same blocks. An adopter who changes their mind edits an answer, not a taxonomy.

## The invariant core

A package declares a **core**: the semantics that an overlay may extend but never remove or redefine ([spec 2](02-taxonomy-model.md#the-immutable-core)). Without one, "the same taxonomy" is not a meaningful claim. If a consumer may override anything, two consumers of one package can share no structure at all.

The core is **semantic, not lexical**. It constrains roles and purposes, never names or paths. A consumer may rename every shelf, relocate every directory, and replace every identifier pattern and every lifecycle value, and still satisfy the core. The condition: after resolution, some facet still has the state role, some kind still serves the `rationale` purpose, and lineage remains expressible and lifecycle-sensitive.

The identifier namespace is lexical and the core does not carry it. `identifier integrity` requires one on every resolved scheme ([spec 2](02-taxonomy-model.md#the-meta-schema)), and the core has no form that could ask for the same thing. The value is the consumer's own, because a package that named one would give it to every corpus that adopts it. The rule is lexical because the alternative is unrecoverable rather than merely untidy.

That is what makes the package a workable boundary object. It is plastic enough to adapt to local practice, and strong enough to keep a common identity across sites. Local form is fully negotiable. Shared meaning is not.

Satisfaction is evaluated on the **resolved** taxonomy. The engine does not forbid particular overlay operations. The engine rejects an overlay when the result fails a core requirement. The rejection names that requirement and the operation that removed its last satisfier.

**Conformance checks the core, not the whole taxonomy.** A consumer that renamed and rearranged everything, but kept the core, is conformant, and the report should say so. This is the difference between a method and a monoculture.

## Upgrading

```
headwater taxonomy diff <artifact> --to 4.0.0
```

reports, against the *local* corpus rather than in the abstract:

- what changed in the base.
- **measured compatibility across the engine's six dimensions** — classification, instance validity, consequence, projection, identifier, addressability ([spec 2](02-taxonomy-model.md#versioning-by-measured-compatibility)).
- which overlay entries the change invalidates (an override that addresses a removed path is an error, not a silent no-op). This is the `addressability` dimension, reported here at the grain that a consumer can act on.
- whether the new base still satisfies the core under the local overlay.
- which local documents violate the new schema.
- which migration steps apply, split into mechanical and judgment-bearing. A step names documents or overlay entries, and the report names every document that no step names.

**The verb takes a directory, and `--to` states which version that directory is expected to be.** This engine opens no socket, so the artifact is one the caller already fetched, the way [`taxonomy vendor`](#waivers) takes one. The flag is therefore the assertion rather than the address. A directory that declares another version is a wrong directory rather than a wrong number. The flag accepts a version or a range of them, through the one range reader the engine has.

**The candidate resolves under the local overlays, and every phase then runs twice over one tree.** That is what makes a difference attributable to the schema. Two publishes of one package differ in a version string, in a digest and in a file timestamp. A report that read any of those would fire on every release, and would then carry no information at all. No dimension reads a published byte.

**Two values are held constant across the two runs, because neither is a consequence of a taxonomy.** The first is the identity of the run. The corpus descriptor states the package, the version and the taxonomy digest that wrote it, and a probe result states the digest. A comparison that let those move would report the version number as a change that the version number caused. The second is the path the taxonomy was read from, which reaches a finding about the taxonomy. That path is the artifact directory on one side and the lock on the other. Everything a taxonomy decides about a projection still moves: the exclusions, the entry points, the exports and every declared output.

**A candidate that does not resolve reports five dimensions as unmeasured, and never as preserved.** A refusal that names an overlay address is the `addressability` reading. There is no census, no run, no plan and no graph under a taxonomy that did not resolve. To report the other five as compatible would be the strongest available claim made out of a failure. The run then exits non-zero, because it could not measure rather than because it measured a break. A measured break exits zero, because an upgrade that needs a migration is the ordinary case ([below](#between-majors-the-corpus-is-legitimately-between-valid-states)).

The publisher measures compatibility against its own reference corpora and reference overlays, and attaches the result to the release as a claim. The consumer's run **verifies that claim against documents that the publisher never saw**. A claim that holds upstream but fails locally is the interesting case, not an anomaly. It means that the local corpus exercises something that the reference corpora do not.

`headwater taxonomy migrate <artifact> --apply` applies the mechanical steps. A step is mechanical when it names one new value. That is a property of the target list rather than a claim the publisher makes ([the payload](#the-migration-payload)). The verb rewrites the value in the front matter of every document the step covers. It emits the rest as a task list with the affected documents attached, ready for a human or a coding agent. The distinction is the whole point. To move files is mechanical. To rewrite a document to fit the section contract of a new kind is not. To pretend that the second is automatable produces plausible, wrong documents at scale.

**A run writes every file or none of them.** Each rewrite is composed in memory first, and the result is read again and compared against what it patched. For a document the comparison covers the body byte for byte, and every scalar of the front matter under the key path that carries it. For an overlay it covers the operation set that the resolver reads back. Every operation keeps its position, its kind and its value, and every address this run did not move stays. Then the run opens every target before it writes a byte. A file that no process may write stops the run, and the tree is what it was. It reads each file back off the tree after the write, and it restores what it wrote when a write fails.

**The documents and the overlay are one write set.** An overlay re-addressed beside a document that still holds the old value is a corpus in neither state. So an overlay that no process may write leaves every document of the run untouched, and the refusal names the file. A crash between two writes is not covered, and nothing inside one program covers that without a journal.

**One of the three writes is still open, and it is open for a stated reason.** Nothing writes the lock, for the reason the [next section](#between-majors-the-corpus-is-legitimately-between-valid-states) gives. Beside the three, each `add` collision with the new base is still owed a judgment task that shows both definitions ([spec 2](02-taxonomy-model.md#customization-by-composition), [#195](https://github.com/headwater-ai/headwater/issues/195)).

**A kind that a homogeneous shelf carries has no byte to rewrite.** Placement carries the kind there, so the document declares it nowhere and the remedy is to move the file. The run reports the shelf that carries the kind, rather than nothing about the document. A step that reached no writable byte and printed no line is a step a reader reads as applied.

### Between majors, the corpus is legitimately between valid states

A migration with judgment-bearing tasks creates a period in which the corpus fully satisfies neither the old schema nor the new one. That is an ordinary major upgrade, not an anomaly. The upgrade is atomic for the *taxonomy*. The lock points at 4.0.0 or it does not, and the no-partial-load rule of spec 2 governs the schema alone. The upgrade is not atomic for the *corpus*. A spec written as if it were would make every real upgrade a lie.

Thus the migration state is recorded in the lock: from-version, to-version, an owner, an expiry, and the open task list. The from-version is the one field that names a prior valid state, and it is optional for the reason that the [next section](#first-contact-adoption-is-a-migration-from-no-taxonomy) gives. While tasks remain open, checks run against the new schema. A finding is reported as `migration-pending` when its **(document, rule) pair is one that the migration payload expects to fail**. The payload declared what moved and what must be re-stated. Thus it knows which rules it broke for which documents, and each open task records that pair set.

A label by document alone would blanket every finding on a named document for the whole migration. Defects introduced yesterday would then read as expected breakage. The pair grain keeps yesterday's regression loud while the declared debt stays patient. `migration-pending` findings are counted, visible in coverage, never blocking, and never suppressed individually.

**No verb writes that state, and the omission is a decision.** The state lives in the `adoption` block of the lock, and [HW-OBL-0082](../obligations/0082-the-lock-is-half-generated-and-half-authored-and-nothing.md) records the seam under that block. No document says whether a reviewed artifact may hold a part that no digest verifies. A verb that wrote there would settle that question by precedent rather than by a decision. So `headwater taxonomy migrate` reports the state it would have written, and states on every run that it wrote none of it. `headwater infer --owner <name> --write` stays the one writer of that block.

**`taxonomy resolve --check` passes while the authored half is stale, and that is a measurement rather than a ruling.** The lock of this repository is the instance: its one task holds no finding, and the check exits 0 over it. `resolve` reads the package sources and never the corpus, so no run of it sees whether a pair still raises a finding. `headwater check` is the run that sees it, and it reports the open pairs, the closed pairs and the findings held. `resolve --check` reports which half moved: a source whose bytes changed, or an `adoption` block outside the form the renderer writes. The header of that file invites a person into the block, so the second case is a form and not a stale taxonomy.

When the last task closes, the state ends. The expiry is the anti-parking device, on the same terms as the expiry of a waiver. A migration state past its expiry is a finding against the owner. It is renewable only by an explicit move of the date — a decision with a paper trail, not a timeout that nobody notices. Waivers are per-rule, and suppressions are per-file. Neither fits a corpus that is half-way across, and that is why the state is its own mechanism, not a pile of either.

### First contact: adoption is a migration from no taxonomy

An organization that adopts Headwater points it at a corpus that nobody wrote to any schema. That corpus was never valid, so it appears to fall outside the state above, which names a version that it came from. It does not. Only the from-version refers to a prior state. The `(document, rule)` grain, the owner, the expiry, the task list, and the counted-visible-never-blocking posture are all defined against the **new** schema ([Q12](09-decisions.md#q12--migration-path-for-an-existing-corpus)).

**So the from-version is absent, and nothing else changes.** Before `headwater init` a corpus is governed by nothing, and every document in it is trivially valid. The findings that the proposed taxonomy raises over the existing tree are therefore the migration payload of that taxonomy's first version. A publisher computes a payload from the diff between two majors. At first contact, [`infer`](#the-interview) computes it from the diff between nothing and one. That payload is the **adoption payload**, and it is a migration state like any other.

The adopter thus gets a green build on the first run. Every document that does not yet fit carries a name, an expiry, and a line in the coverage report. That is what a grandfathering file gives, plus the three properties that such files omit.

**No threshold ever converts an accounting into a silence.** One shipped tool excludes offending files one at a time, then disables the rule once the list passes a limit. That is correct for a mature corpus and wrong here. First contact is the one moment at which every rule exceeds any such limit ([spec 11 §S.5](11-adjacent-work.md#s5-grandfathering-has-a-scale-at-which-it-lies)). A payload therefore holds `(document, rule)` pairs however many there are, and [spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)'s coverage obligations stay total. The cost is a large payload in the lock on a large corpus, and the lock is committed and reviewed.

**Every run reports the count that remains.** The expiry is a date, and a date arrives too late to tell anybody that a payload is not shrinking. So the run reports the number of open pairs beside coverage. A payload that does not move is then visible from the second run rather than from the expiry.

**Adoption never runs on a flag.** No invocation of the engine decides which findings count, and [spec 6](06-engine-architecture.md#cli) declares no flag that scopes a run. A mode that gated on newly touched documents would make two runs over one tree disagree. What scopes the work instead is the cache, which derives what moved from content hashes and reaches the same verdict either way.

## Conformance

Vendoring content is not adoption. A consumer can hold a perfect copy of the taxonomy and wire none of it. Conformance is a separate, evaluated question:

```
headwater conformance [--level <name>] [--now <date>]
```

evaluates the repository against rules that the taxonomy package ships — checks wired in CI, gates required on the default branch, projections regenerated, hooks installed, pin current. It reports gaps with remediation. The rules ship *with the package*. Thus a pin advance brings newly-added requirements into force automatically. Improve the method, and the next upgrade of every consumer surfaces the new gap. That loop is what turns a published method into an adopted one.

The verb reports and it gates nothing by itself. `--level <name>` is the one thing that moves its exit status. The section on levels below says what a level is before it says what that flag does.

### The package names a rule, and the engine holds the reading

A conformance rule is two halves in two places. The package declares the name, the text an adopter reads, and the remediation. The engine holds the code that decides the rule against a tree. Neither half is any use alone, and the split is the same one that `requires_engine` already makes one level down.

**A rule this engine cannot read ends the run.** The precedent is exact. A publisher declares `requires_engine`, `headwater_resolve::package::sources` reads it, and an engine outside the declared range is refused before one source loads. A rule name that this engine holds no reading for is refused on the same grounds, and the message names the rule and the package. A publisher who adds a rule that needs a new reading raises the engine floor of the package. That mechanism exists already.

The alternative fails quietly, which is worse. An engine that skipped a rule it could not read would report a level over the wrong rule set. The publisher and the consumer would then disagree about what that level covers. The consumer would hold a green report about a requirement that nothing evaluated, and no line of it would say so.

**Some rules no tree decides.** A permission granted in the admin console of a platform is the standing example, and so is a hook that each clone installs for itself. Such a rule declares an attestation in place of a reading. The report names it, states what would decide it, and counts it as neither met nor missing. No level names such a rule until an attestation record exists, and [13 — Open obligations](13-open-obligations.md) holds that wait. This is what "not silently dropped" means in a report that a person reads.

### The pin is two numbers

"Pin current" was one number while a package was a version. A published artifact is now a digest over every file in it, and `.headwater/taxonomy.yml` carries `taxonomy.digest` beside `taxonomy.version`. A rule that read the version alone would pass a repository whose pinned digest names an artifact that nobody publishes any more.

So the reading compares both and reports each half. A repository whose package directory carries no release record pins nothing, because no published artifact stands behind that directory. The reading calls that a gap with a remediation, rather than a state it looks away from.

**The remediation names five steps, and [Waivers](#waivers) below says why one of them moves the existing package directory aside.** A consumer cannot take the first of those steps from inside this tool, which [13 — Open obligations](13-open-obligations.md) records as `HW-OBL-0085`. So the remediation states which step nobody can do, rather than a route that ends where the reader started.

**No rule names the core, and that is not an omission.** [The section above](#the-invariant-core) says that conformance checks the core rather than the whole taxonomy. `taxonomy validate` already decides core satisfaction, and `taxonomy resolve` writes a lock only when the taxonomy validates. A lock is therefore a validated taxonomy, so the rule that holds the lock to the sources holds the core through it. A second reading of the core here would be a second answer to a question one verb already decides.

### What a level means, and what stops it from becoming a score

A level is a named subset of the conformance rule set, and the package declares it. [The maturity ladder](../doctrine/maturity-model.md) is the ordering over those subsets. **A level states what the adopter wired up. It is not a measurement of how good a corpus is, and this specification does not dress it as one.** A publisher asserts the ordering. The engine measures which rules pass, and it asserts nothing else.

Three properties keep the number honest.

**Nothing declares a level, and a key that tried to would end the run.** The report derives the level from the rules that pass, so there is no field for an adopter to write a larger number into. `waivers` is the only key the `conformance` block of the consumer declaration takes, and any other one is refused by name. A `level` key read and dropped would leave an adopter holding a claim that nothing evaluated. That is the same defect as an engine which skipped a rule it could not read, from the other side. An adopter who wants a different rule set forks the package. A fork moves the package name and the digest that the report prints beside the level. A level with no package identity beside it means nothing, so the report never prints one alone.

**A level with no rule is refused.** The reader rejects a package that declares an empty level, because a rung that names no rule is a rung every repository already stands on. A rung arrives with the rules that earn it or it does not arrive.

**A waiver moves the exit status and never the level.** The next section states what that costs and what it buys.

### Waivers

A consumer may deviate deliberately. A waiver names the rule, the reason, the owner, and an expiry. Waivers appear in the coverage report of the consumer, and they are visible to the publisher in aggregate. Deviation is fine. Invisible deviation is not.

**A waiver lives in the files of the consumer, and the reason is structural.** `headwater taxonomy vendor` replaces a vendored package directory whole, and it refuses to overwrite a directory that carries no release record. A shipped rule is therefore never edited locally, and a waiver written beside the rule it waives would not survive the next upgrade. The consumer declaration holds waivers instead, in a `conformance` block beside the pin. That file is authored, committed and read in a diff, which is where a deviation belongs.

```yaml
conformance:
  waivers:
    - rule: pin.current
      reason: accepted_deviation
      owner: j.baxter
      until: 2027-02-28
      note: an adopter who has not yet vendored a published artifact
```

**The expiry is required, on the terms the local escape hatch already takes.** [Spec 4](04-assurance-model.md#suppression) makes the expiry of a suppression mandatory because the expiry of a waiver was mandatory first. An expired waiver is reported as expired, and the rule under it is then evaluated as though no waiver stood there. A waiver thus fails toward the rule rather than toward the deviation, and the day it expires is the day the gate goes red.

**A waiver against no rule ends the run.** A waiver that names a rule the package does not declare is a deviation that nobody reviews away. The report that would list it has nothing to list it under. The refusal is the same shape as the one above, from the other direction.

**A waived gap is still a gap.** The report states the level that the passing rules reach, and a waived rule does not pass. `--level <name>` exits non-zero on a gap that no waiver covers, so a waiver buys a green gate and never a higher rung. That separates an adopter who accepted a deviation from an adopter who closed it, and it is the whole reason a level cannot be bought.

**A waiver reaches a conformance rule, and [spec 4](04-assurance-model.md#suppression) counts one against a check finding.** One mechanism carries both populations, because the four fields and the mandatory expiry are the same in each. What differs is which rule a waiver may name. `headwater conformance` reads the waivers that name a conformance rule. The coverage account of `headwater check` reads the waivers that name a check rule. That second reader is why the paragraph below needs its exclusion. It does not exist yet, so every waiver in this repository today names a conformance rule, and the coverage line says so.

One rule class is outside the mechanism. A [withholding rule](06-engine-architecture.md#an-export-profile-carries-a-filter) is not waivable. A waiver buys time against an error that a later run corrects, and no later run undoes a disclosure.

### Self-consumption is the vendored-bytes shape

A repository can both publish a package and consume it. This repository does, for `headwater/standard`, and the two roles meet each other at the pin.

**Two shapes reach a consumer, and this one takes the shape already named above.** [Consuming](#consuming) states it: the caller fetches the artifact, `vendor` checks it against the pin, and the lock that names the result is committed. That is vendored bytes under version control. The other shape a consumer could take is a fetch that continuous integration performs on every change. It commits nothing, and it checks the artifact fresh each time. This engine opens no socket anywhere ([spec 0](00-vision-and-scope.md#non-negotiables)). So the second shape needs code no crate of this engine carries, and no crate has ever needed it. The first shape needs none of that code. The artifact already sits in this repository's own history, because `vendor` wrote it there once.

**Publishing and consuming share one lookup, and a publisher that also consumes meets that lookup from both sides.** `taxonomy publish` finds its source under `packages/` by the name a manifest declares. `taxonomy resolve` finds its source the same way. A repository whose authored source and vendored target are one directory meets `vendor`'s own refusal. `vendor` refuses to install over a directory that carries no release record. That directory is the source, and `vendor` never wrote it. Two directories under `packages/` that both declare one name do not solve this either. The lookup returns whichever one sorts first. That is [#354](https://github.com/headwater-ai/headwater/issues/354)'s own defect, met here by the shape of the design rather than by an accident of it.

**So the authored source moves out of `packages/`, and `taxonomy publish` gains a way to read it there.** This repository's own source sits at `taxonomy-source/headwater-standard/`, outside `packages/` entirely. `taxonomy publish --from <dir>` reads the manifest at a directory the caller names. It bypasses the lookup by name. Nobody edits `packages/headwater-standard/` after that. Only `taxonomy publish --from` and `taxonomy vendor` write there. Both run by hand, whenever the source changes. So the directory always carries the release record that `vendor`'s own guard depends on.

## Arriving at a corpus cold

Everything above describes a repository that already knows its publisher. A machine that holds only a location knows none of it. It needs to learn which corpora live there, what taxonomy governs each one, and where to start ([Q14](09-decisions.md#q14--discovery-surface)).

**Two questions arrive together, and they close by different routes.** How a machine learns that a corpus exists, when it holds no pointer at all, is **registration**. No file inside a corpus answers it, and no convention in the field pretends otherwise. Every one of them presumes a client that already resolved a name. What closes here is the other question, **resolution**: a machine holds a location, and it needs to learn what governs it.

**Registration is an act of publication into a channel whose reader is already obliged** ([Q16](09-decisions.md#q16--public-presence)). That definition is what explains the failure of every file-based attempt at it, including the one measured convention that tried ([spec 11 §O.3](11-adjacent-work.md#o3-llmstxt-is-the-measured-failure-of-a-descriptor-with-no-obliged-reader)). Two obliged channels exist already and Headwater builds neither. A taxonomy package goes to a registry that a resolver must read to install it. A package that carries the publisher's corpus location thus registers that corpus with every consumer. And a rendered page carries the link relation below, so a reader that fetches the page reaches the served copy. Registration is thus the publisher's own act, and the descriptor is the whole of what Headwater supplies for it.

**The corpus descriptor is a projection, and it is the one whose path the engine fixes.** `headwater generate` writes it and `generate --check` holds it to regeneration, like any other projection. What it does not take from the taxonomy is its own location, and the reason is the whole point of it. A reader who has to consult the taxonomy to find the descriptor already has what the descriptor would have told them. So the descriptor sits at `.headwater/corpus.json`, relative to the repository root, and it is engine-defined and non-optional. That is the standing that the [register projection](04-assurance-model.md#every-obligation-has-exactly-one-disposition) already has, for a different reason.

Everything else about it is ordinary. It is generated from the roots, so it cannot go stale against them. That property is what lets one document carry per-root identity at all. The conventions that this resembles keep their index files bare and put identity on each collection. They do so because a hand-maintained index describes roots that somebody else edits.

The descriptor carries five things for each corpus root in the repository. The root path. The taxonomy identity and version. The lock hash. The entry points. And each declared [export profile](06-engine-architecture.md#an-export-profile-carries-a-filter), with its output location and its tombstone grain. It carries nothing that those artifacts already state about themselves. An export declares its own coverage ([spec 12](12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule)), and a second copy of that statement would disagree with the first at the next release.

**Four of the five come from a file and one is derived.** The engine reads the root, the identity, the lock hash and each declared export profile. It derives the entry points.

**An entry point is derived, because no declaration in the language holds one.** The entry point of a shelf is the document that the derived reading order puts first ([spec 2](02-taxonomy-model.md#reading-precedence-is-derived)). Spec 2 already names "reading order in generated indexes" as one consumer of that derivation. So a descriptor, a shelf index and a route agree about where a reader starts. Where no relation governs a shelf, the reading order is the path order.

**An entry point carries a path and an identifier, and never a summary.** The gate compares the bytes of this file. A summary is prose in front matter, so a descriptor that held one would fail the gate after an ordinary wording edit. A gate that fires on prose is a gate that everybody bypasses.

**An export row states its name, its target, its output and its grain, and never its filter.** [Spec 6](06-engine-architecture.md#an-export-profile-carries-a-filter) gives a profile an audience, a target, a filter and a tombstone grain. This file is served, and a filter clause names facet values. That sits one step closer to the content than the structure a descriptor may disclose. So a reader learns that a profile is filtered and at what grain, which is what a filtered view owes anybody. The clause itself reaches the reader who receives the export.

An earlier release of the descriptor carried no `exports` member at all. The `projections` reader of the day kept a kind and an output path, so no taxonomy could declare a profile. An empty list would then have said that a corpus exports nothing, rather than that nothing could declare an export. The reader takes the whole declaration now, and an empty list means what it says.

**The descriptor also states each declared exclusion, with its reason.** A root on its own overstates the corpus. This repository declares `docs` and excludes one directory under it. A cold reader that saw only the root would treat package content as governed content. No artifact that such a reader reaches states the exclusion, so the descriptor does.

**JSON carries no comment, so the generated-file marker is a top-level member.** The marker is the rule that protects a file that a person wrote. A path that the engine fixes is not a reason to drop it. An adopter may write a descriptor by hand before the verb reaches them. The member also answers the absence rule below, and one member for two rules is one fact in one place.

Three rules make it usable rather than decorative.

- **A version carries a stated client behavior.** A major version above what the reader understands is a hard failure with a message. A minor mismatch is a warning, and the reader continues. A version field with no rule attached is a string.
- **Absence does not read as presence.** The descriptor declares its own media type and a required shape. A reader that receives a success response which does not parse to that shape treats the descriptor as **absent**, not as malformed. The two have different remedies. A served host that answers every path with a default page is the ordinary case rather than the exotic one.
- **The canonical location is inside the repository, and a served copy is reached by a pointer.** A reserved path at the root of an origin is a poor fit here, for three reasons. A repository holds one or more corpora. A documentation site is often one part of a host that serves other things. And the party who writes the descriptor rarely controls the root. A rendered page therefore carries a link relation to the served copy, and the copy may sit anywhere that the site can put it.

**The descriptor is a served artifact, so a filter reaches it first.** It names roots, entry points and profiles, which is organizational structure. An export profile filters it exactly as it filters anything else, and a filtered descriptor announces that it is filtered.

## Federation

Larger organizations use layers: a generic method, a divisional taxonomy that extends it, and a repository overlay that extends that. Two rules keep the stack coherent:

1. **References run upward.** A repository may reference its own tier or a higher one, never a sibling or a lower one. A downward reference makes the upper tier depend on something that it does not control, and the abstraction inverts. The legal reference set is derived from what a repository actually consumes, so it needs no hand-maintained registry.

2. **Overlays compose in one direction.** Each tier may override, add, or remove against the tier above it. A tier never reaches past its parent. Conflicts are resolution errors, not precedence puzzles. Overlay application must be confluent ([spec 2](02-taxonomy-model.md#customization-by-composition)). Thus a three-tier stack has no resolution order that anyone must remember.

### Across taxonomies, not under them

A layered stack only helps organizations that share a root. Two divisions that adopted different taxonomies independently — after an acquisition, or simply because they arrived separately — have no common ancestor to build an overlay against. A merge of the two is a political project, not a technical one.

They do not need to merge. They need **declared correspondences**: SKOS-style mapping relations between their concept schemes ([spec 2](02-taxonomy-model.md#mapping-between-taxonomies)). `exactMatch` where two kinds are interchangeable, `closeMatch` where they are interchangeable for retrieval but not inference, `broadMatch` / `narrowMatch` where one is wider.

With mappings declared, an aggregator answers "every decision in the organization" across taxonomies that share no vocabulary. The next section says what an aggregator is. Neither division gives up its own. Neither taxonomy changes. A third artifact records how they correspond, and the tier that aggregates owns it — normatively, not conveniently. Pairwise mappings between peers grow quadratically, and they go stale on every publisher release ([spec 2](02-taxonomy-model.md#mapping-between-taxonomies)). That is the standard answer to this problem in knowledge organization, and there is no reason to invent a worse one.

### The tier above a corpus harvests it

A tier that answers questions across many corpora needs their content. Two architectures were available, and only one survives contact with the constraints that this specification already set ([Q9](09-decisions.md#q9--multi-repository-corpora)).

**The aggregator is a solution corpus plus one anchor kind.** The solution layer is an ordinary corpus. It authors the facts that live between repositories, and it consumes exports for everything else. What was unstated is how it reaches the corpora below, and nothing new is needed for that. An anchor kind is declared, and exactly one resolver owns it ([spec 2](02-taxonomy-model.md#behavior-at-the-limits)). That resolver reads pinned corpus exports, in the way that the `code_path` resolver reads a source tree.

**There is no merged graph.** Merging *is* anchor resolution, and anchor resolution leaves nothing behind when a run ends. The solution corpus holds its own documents and its own declared edges. It resolves anchors against the exports that it pinned, and it rebuilds that resolution on every run. A merged graph would be canonical for nothing, would carry no reviewer, and would cost one rebuild to reproduce. Such an artifact does not need to exist.

**The tier harvests, and it never fans out.** Each source corpus carries a pin: an identity, a content hash, and a location. A scheduled job fetches each export out of band and commits it, and the resolver then reads the committed copy. The tier never queries a live endpoint. Three arguments agree, and this specification already made all three.

- [Spec 0](00-vision-and-scope.md#non-negotiables) forbids a network dependency at check time, and a fan-out query is one.
- [Spec 6](06-engine-architecture.md#performance-targets) budgets 100 ms for a route query. A fan-out across estates does not fit inside that, and a call that does not fit is a call that developers remove.
- A fan-out that meets an unreachable source either fails whole, or returns a smaller answer with no notice. The second outcome is the silent pass that [spec 4](04-assurance-model.md#no-silent-passes-every-document-is-accounted-for) exists to forbid.

So a pinned export that the tier cannot read is a **finding that names the pin**. It is never a narrower answer, delivered quietly. The metadata-harvesting aggregators of the digital-library world reached this architecture under the same pressure, and the [evaluation](../evaluations/graph-export-and-federation.md) records what they found.

**A tier pins an export profile, and the publishing corpus decided what is in it.** A filter acts where the export runs and never where a tier reads. A tier that reads holds bytes that already crossed the boundary ([Q17](09-decisions.md#q17--governed-access-and-the-solution-layer)). So a tier never filters what it harvested, and it has nothing to filter: what arrived is what the publisher meant it to have. An anchor whose target the publisher withheld resolves to `withheld` rather than to a dangling reference ([spec 2](02-taxonomy-model.md#behavior-at-the-limits)). The tier reports it at the profile's declared grain.

**A pin makes revocation late, and the specification says how late.** A document that a publisher withholds today stays in the tier's committed copy until the next harvest. That is the price of harvest over fan-out, and it is not removable inside this architecture. So an export carries its generation time, and the harvest schedule is declared. The revocation lag is then a number that an operator can read rather than a surprise. The authorization systems that solve this problem in the other direction carry a freshness token on every answer. A design with no such token owes the reader the cadence instead ([spec 11 §O](11-adjacent-work.md#o--the-serving-boundary-descriptors-redaction-and-the-write-path)).

**A solution-layer node is a declared anchor.** A tier that models the estate is tempted into nodes for services, interfaces and capabilities. A node that asserts a service's properties has left the corpus and started to model the world ([spec 11 §A.1](11-adjacent-work.md#a1-the-solution-layer-presses-on-that-boundary)). Two arguments refuse it. Nothing in the design carries an obligation to keep such a node true, and a wrong node reads as structural rather than editorial. And a filter has nothing to attach to on a node that carries properties. A document has facets that a predicate reads, and an anchor is carried whole or withheld whole. So a solution-layer node carries an identifier, a name and an owner, and every substantive claim stays inside a document. This is what `code_path` already does. A concrete need that the anchor form cannot meet reopens it, argued as the change to the model that it would be.

## Upstream awareness

A scheduled check compares the pinned version against the latest release of the publisher. It raises a change proposal, with the diff report and the migration assessment attached. It does not raise a notification that nobody acts on. The default is a draft change request that an agent can complete. A pin that only a human can advance is a pin that goes stale.

**A proposal channel carries a budget, and the reason is measured.** Studies of automated dependency proposals report about a third merged for undifferentiated version bumps. For security fixes the figure is about two thirds, against roughly four fifths for proposals that a person wrote. The mechanism is the same in all three. What moves the number is how selective the proposer is. So an unbounded channel converts into notification fatigue, and the usual remedy is a cap on open proposals. Headwater declares that cap beside the schedule, so an operator reads it rather than discovers it.

**Whoever opens a proposal needs more than the permission to open one.** On the platforms in common use, the permission to create a proposal does not include the permission to create the branch that it points at. The permission that does create a branch also permits a merge. So a proposer that authors its own branch is not confined by its permissions alone. Two mechanisms confine it, and an operator states which one is in force. Either a branch rule requires review and grants the proposer no exemption, or the proposer owns a separate repository and proposes from there. To leave this unstated is to claim a separation that the credential does not supply ([Q7](09-decisions.md#q7--scope-of-the-mcp-surface)).

**One pattern, three instances.** A taxonomy pin, a requirements snapshot pin ([Q19](09-decisions.md#q19--inbound-integration-an-external-system-of-record)), and a source-export pin all work the same way. Each one fetches out of band, commits the result, checks against the committed copy, and compares on a schedule. Each one raises a change proposal and never a mutation. To state the pattern once is what keeps the third instance from arriving as a new mechanism.

**A pin states the channel that carried it, and an import with no channel is refused.** A digest authenticates the pin and never the publisher, which [§What a package digest proves](#what-a-package-digest-proves-and-what-it-does-not) states for a package and [HW-OBL-0115](../obligations/0115-a-pinned-digest-authenticates-the-pin-and-never-the-publisher.md) records. The consequence is sharper inbound than outbound. An adopter who binds a package reads a lock and a diff. An edge that an import wrote reaches a graph that every later check agrees with. So an imported edge carries the weight of the channel the snapshot arrived on rather than the weight of the digest. The declaration therefore names both, in the words of the person who wrote the digest down, and neither is ever written by a verb. A channel stated beside the payload would certify itself, which is the move the pin exists to refuse.

**A snapshot pin reports drift on each affected edge, and not only on the pin.** A snapshot carries the upstream identity and revision of every item in it, so an advance says which items changed. Every edge into a changed item is then a finding until a person re-verifies it. Requirements practice reached the same mechanism and calls such an edge *suspect*. A proposal against the whole snapshot names a file, and a finding on an edge names the document whose author can act. That is [spec 4](04-assurance-model.md#absence-is-a-finding-class-of-its-own)'s report-at-the-origin rule, applied to a second kind of upstream.
