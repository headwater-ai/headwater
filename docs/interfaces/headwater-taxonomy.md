---
id: HW-IFACE-headwater-taxonomy
status: current
status_since: 2026-09-06
summary: "How to validate, resolve, audit, publish, vendor, compare, migrate, draw and list the kinds of taxonomy packages."
last_verified: 2026-09-25
title: "headwater taxonomy"
relations:
  governs:
    - [engine/crates/cli/src/lib.rs, engine/crates/cli/src/main.rs]
    - engine/crates/cli/src/taxonomy_graph.rs
    - engine/crates/query/src/kinds.rs
    - engine/crates/resolve/src/lib.rs
    - engine/crates/audit/src/lib.rs
    - engine/crates/audit/src/reading.rs
    - engine/crates/compat/src/lib.rs
    - .headwater/taxonomy.lock
    - .headwater/adoption.jsonl
---

# headwater taxonomy

## Synopsis

    headwater taxonomy <validate|resolve|audit|publish|vendor|diff|migrate|graph|kinds> [options] [--root <path>]

The grouped command validates taxonomy sources, resolves the lock, measures schema use, publishes or vendors packages, compares versions and applies migration payloads. It also draws the resolved taxonomy, and it lists the kinds that a document can be.

## Description

`validate` checks taxonomy sources without writing. `resolve` writes the validated `.headwater/taxonomy.lock`, or checks that the committed lock is current with `--check`. `audit` reports schema measurements and remains non-gating. `audit --record` also appends this run's adoption reading to `.headwater/adoption.jsonl`, which is the one write this word performs.

`audit --json` writes the run as one JSON document on standard output, in place of the report a person reads (#1573). The document holds the subject of the run and the governed scope, and no other reading of the report. A consumer reads the absence of a reading as "not carried", and never as zero. The members are these:

- `version` names the shape of the document, and its value is `1`. A consumer pins it rather than the version of this engine.
- `subject` holds `package`, `version`, `lock` and `now`, which are the header of the report.
- `scope` holds one element for each declared scope pattern, in declaration order. Each element holds `anchor_kind`, `pattern`, `in_scope`, `governed`, `share` and `ungoverned`. `share` is the percentage to one decimal place, as the report prints it, and `null` for a pattern that admits no entry. `ungoverned` is the sorted list of the entries that no edge reaches.
- `scope_total` holds `in_scope`, `governed` and `share` over the union of the entries. An entry that two patterns admit counts once, so the total is not the sum of the elements.

The report and the document read one total, so they cannot state two figures for one tree. `--json` with `--record` still appends the reading, and the line that says so goes to standard error.

`validate` also counts the governed scope against the tree beside the taxonomy (#951). It refuses each scope pattern that matches no entry, and the run exits 1. A taxonomy published as its own repository has no tree beside it. At such a root the run prints one notice line, skips the count and refuses no pattern. The owner ruled both behaviors on 2026-09-25, on #951.

`validate` also reads the `outside_root` list of each language regime against the tree ([HW-DR-0084](../decisions/0084-a-language-rule-reaches-front-door-prose-outside-the-corpus-root-and-no-other-rule-does.md)). It names each pattern that matches no file. It also refuses four kinds of pattern. The first leaves the repository. The second matches a path under the corpus root. The third reaches a path that a second regime lists. The fourth is a symlink or passes through one. Each one is a line on standard error, and the run exits 1. `resolve` refuses the first kind, and a literal path of the third kind, before it writes a lock. `headwater check` reports the others as errors. At a root with no tree beside it, the run does not name a pattern that matches no file.

`validate` also holds the vendored bytes to the pin (#1186). Where `.headwater/taxonomy.yml` declares `taxonomy.digest`, the run computes the digest over the files in the package directory. Where the two digests differ, the run writes one refusal on standard error and exits 1. The refusal uses the same comparison and the same words as `taxonomy.pin.diverged` in `headwater check`. Where nothing is pinned, the run says nothing about it.

The run asks the root whether a tree is there, and it never asks the patterns. The tree is absent when no directory at the root counts as a tree and no pattern names a path that exists. Four kinds of directory are the taxonomy's own and do not count. They are a dot-directory, a directory that git ignores, and the first directory of each resolution source. The fourth is the first directory of each `contents` value in the root's `package.yml` (#1103). So a value `x: guide/readme.md` makes all of `guide/` the taxonomy's own. A root with no `package.yml` has no fourth kind, and the run says nothing about it. A root `package.yml` that does not read also has no fourth kind. Then the run names the file and the reason, and the taxonomy is not valid (#1123). So an authored package source at the root, with its `assemblies/` and `doctrine/` directories, gets the notice. Every other directory counts as a tree. An `examples/` directory is a tree unless `contents` names it, and a publisher who ships examples as package content names them there. A tree whose every pattern is misspelled is still a tree, so the run refuses each pattern.

**The report takes nine readings of the corpus.** They are facet differentiation and orthogonality, edge counts and staleness by `created_by`, and relation drift by family. They are also the discriminator distribution of a heterogeneous shelf, the state-dwell distribution and the warrant of each classified document. The eighth reading is the file names on each shelf that declares a layout. The ninth compares the adoption payload of this run with each reading that `.headwater/adoption.jsonl` holds. Each reading is about the taxonomy and not about a document. A facet that no document separates from another is a defect of the schema, and only a corpus can show it.

**Two readings that the report names do not run, and each one says what it waits on.** A wait is a list of prerequisites, and each run evaluates that list against the corpus in front of it. So the report says when a corpus has ended a wait, and no string that somebody forgot to edit states one. For each prerequisite that is absent, the report says where the absence is: in a declaration, in an authoring pass or in a decision. Transition continuity waits on a facet in a role that the closed registry of [spec 2](../spec/02-taxonomy-model.md) does not hold. So its absence is a column and not a row. Scent quality waits on a cue on an edge instance, and nobody has authored one ([HW-OBL-0023](../obligations/0023-no-corpus-has-authored-enough-cues-to-grade.md)).

The promotion rate of [Q15](../spec/09-decisions.md#q15--a-synthesized-content-tier) is not a wait. This verb reads one working tree, so it has the denominator of that rate and never its numerator. `warrant.promoted` counts a promotion in the change that makes it, and the warrant reading names that rule beside the population it reports.

**The report states each figure, and no document copies one.** A count in prose goes false at the next change to the population that it counts. So a document that needs a figure of this report names `headwater taxonomy audit` and does not state the figure.

**The warrant reading has one row for each of the four warrant values that [spec 3](../spec/03-authoring-and-lifecycle.md#the-warrant-and-what-each-value-requires) declares.** A value that no document states keeps its row at zero, so a reader can tell an empty arm from an absent arm. The `regenerated` and `transcribed` rows stand at zero by construction. The engine reads those two values from the generated-file marker and not from a declaration. The `asserted` row is the denominator of a promotion rate, and the report says that nothing declares a bar over it. A value outside the four is counted apart from the rows, and `warrant.value.not_permitted` reports each document that states one.

**The layout reading reports a shelf that it cannot measure apart from a name that drifted.** The reading renders each file name again, with the same function that `headwater new` uses to name a file. Where a kind declares no source for a placeholder, its whole shelf is outside the reading, and the row says where the absence is. A document that the reading cannot measure counts in no numerator and no denominator. One figure over both groups would report a missing declaration as drift.

**Two findings carry a verdict, and every other reading is a distribution.** Each finding reads an input that the taxonomy declares. The first is a relation with a half on a document older than `stale_after_days` on the freshness facet. That window is the one bar that a taxonomy states for this report. The second finding is a declared scope pattern with an entry that no `governs` edge reaches. A scope is an input and not a bar, so this finding is advisory. No declaration says how narrow a facet may be, or how low a capture rate may fall. So each other reading prints its population and no verdict, and [HW-OBL-0119](../obligations/0119-an-audit-reading-carries-no-declared-bar-so-a-distribution-cannot-become-a-finding.md) holds that gap.

**`--record` writes one line that is not a document.** The adoption reading of the run goes to `.headwater/adoption.jsonl`, which is outside the corpus root, and no rule reads it. [Spec 7](../spec/07-distribution-and-federation.md#what-the-adoption-store-records-and-what-it-refuses-to) states what the line holds and what it leaves out. Without the flag, `audit` writes nothing. Two audits of one tree at one `--now` date write the same bytes. The store refuses a second reading at one lock and one date, so that is also true with the flag. A run reads one working tree, so the series over time is the one reading that a single run cannot take.

**The creator reading has one row for each relation, and none for each edge.** [Q4](../spec/09-decisions.md#q4--relation-storage) keeps `created_by` on the relation type. So a scaffolded `supersedes` edge and a typed one are the same string on disk. A row for each edge would state a provenance that nothing records. The reading walks the closed set of six creators and not the values in use. A creator that no relation declares is the arm that a comparison needs, and a list of the values in use leaves that arm out.

**`audit` gates nothing.** It exits 0 whatever it finds, and no gate and no hook runs it. A young or small corpus fails differentiation for reasons that are not defects, so each finding is advisory. CI runs `audit` to report the governed scope, and no step of CI gates on that report.

`publish` writes a release artifact and release record. Before it writes anything it resolves the base with every bundle the package ships, and it holds every template the package ships against that resolution. It refuses a bundle set that does not resolve, and a template a person could copy into a document that resolves to no kind.

`publish --json` writes the release record as one JSON document on standard output, in place of the paragraph a person reads. The document names the package, the output directory, the digest a consumer pins, and every member with its own digest. A `delivery` member says how the artifact reached the output directory. The value is `renamed` where one rename moved the assembled artifact into place, and `direct` where the artifact went in file by file. A `direct` publish is not atomic, and a run stopped part way leaves files there with no release record. The member is present under both values, so a consumer tells the two apart. The reason for a `direct` value is one line of prose on standard error, under both output modes. The document names its own shape in a `version` member, so a consumer pins that rather than the version of this engine.

`publish --clear-killed` removes what a publish that was killed part way left at the output directory. It then publishes into that directory in the same run. It removes one state and no other. That state has two halves. The first half is files at the output directory with no release record. The second half is a directory beside it, named for the output directory and the suffix `~staging`. That directory holds two files. One is the file a publish writes to claim the directory. The other is the file a publish writes when it cannot move the artifact into place. Only a killed publish leaves those two files together, and the second file names the output path it was writing. The flag removes nothing where the output directory carries a release record. It removes nothing where those two files are not together beside it. The publish then refuses as it refuses without the flag. The run reports on standard error what it removed.

`publish --from <dir> --check` writes nothing in the tree (#1139). It is for a repository that maintains a package and also consumes it. It publishes the source at `--from` into a private directory outside the tree, and it removes that directory on every exit path. It then compares that artifact with the vendored copy of the package. The vendored copy is the directory under `.headwater/packages/` whose `package.yml` declares the name that the source declares, and that directory must carry a release record. The run prints one line with the digest where the two agree. Where they do not agree, it names each file that moved on standard error, and it names the four steps that republish the source. Every refusal of a publish is also a refusal of the check. The check needs `--from`, because a package that `--package` finds is the vendored copy itself. It refuses `--out`, `--package`, `--assembly`, `--clear-killed` and `--json`.

`vendor` reads a fetched artifact into the package area after digest validation. Where a package is already installed at the same version and a different digest, the run reports the pair. It names both digests and both member counts, and it says that this version number now names two sets of bytes. It installs the artifact and exits 0. Nothing refuses a second artifact under a version already published, so this report is the one place an adopter sees it. `diff` compares a fetched artifact with the current taxonomy. `migrate` reports migration steps and writes them only with `--apply`.

**`vendor` takes a directory or a location, and one crate keeps the network out of the checking loop.** A location is the URL of a published artifact zip. The run fetches it through `headwater-fetch` and unpacks it into a temporary directory. Then it checks those bytes in the same way as a directory that the caller fetched by other means. `headwater-fetch` is the one crate with a client, and only `headwater-cli` links it ([HW-DR-0075](../decisions/0075-the-vendor-verb-may-take-a-location-and-the-fetch-lives-only-in-a-crate-the-checking-loop-never-links.md)). `engine/crates/cli/tests/network_boundary.rs` reads the lock file and fails when any other crate reaches the client. That test holds the crate graph, so no library crate can call the client. It does not hold the binary, where `check` and `vendor` share one crate. So inside `headwater-cli`, the [non-negotiable](../spec/00-vision-and-scope.md#non-negotiables) rests on one call site, the `vendor` arm, and a reviewer reads it. A binary built without the `fetch` feature opens no socket. `publish` writes the artifact that `vendor` reads.

`diff` reads the lock and it re-resolves no source. It states two caveats where they hold, and it gates on neither. The first caveat is a base that resolved to the same text beside a broken `addressability`. A lock that lost its founding record produces that pair. So does a release that moves a declaration between two bundles whose operations commute, and that lock is current. The run separates neither, so the caveat states both readings and refuses nothing. The second caveat names the source files the lock records that have since changed on disk.

`diff` does not apply the test that `resolve --check` applies. That test refuses the publisher who edits a package source in place, which is the correct run this verb is written for. It also passes a lock that a person resolved after the candidate was installed, where `diff` reports the wrong answer.

`graph` prints the resolved taxonomy as a Mermaid flowchart on standard output. It reads `.headwater/taxonomy.lock` and no source, and it writes no file. `--view` selects one of two drawings, and the default is `concrete`.

The concrete view draws each purpose as a lane that holds its concrete kinds. Each anchor is a hexagon, and the drawing shows an anchor that no relation reaches. Each pair of endpoints that a relation declares is one edge, with the name of the relation as its label. A pair whose two ends are one kind is no edge. The drawing writes it on that kind as one line that starts with ↻, in the color that its family gives an edge. A pair with an abstract kind at one end is not an edge in this view. An abstract kind stands for every concrete kind under it, so each of those kinds can take the relation. One comment line at the top of the drawing says that the abstract view draws those pairs.

The abstract view draws each abstract kind as a dashed node, with the facets that the kind requires. Each kind declared under an abstract kind, at any depth, sits in the lane of its purpose. A dotted arrow runs from that kind to the kind it is declared under. Each pair that has an abstract kind at one end is an edge, with the name of the relation as its label. A pair whose two ends are one node is a line on that node, as in the concrete view. An anchor appears only where one of those pairs reaches it. A lock that declares no abstract kind gives one comment line and no drawing.

`--legend` adds a key to either view. The key draws each shape and each edge style that the drawing uses. It names each family that colors an edge or a line on a node. The key takes each color from the list that colors the edges, so the key and the edges cannot disagree. The verb prints no key without the option.

Neither view names a kind or a relation. Each reads `abstract: true` and `is_a` in the lock. Two runs over one lock write the same bytes, because the output is sorted and carries no clock and no digest. A lock that is absent or that does not read stops the run with a non-zero exit and one message on standard error.

`kinds` lists the kinds of the resolved taxonomy for an agent or a person who is about to write a document (#1580). It reads `.headwater/taxonomy.lock` and no source, it walks no corpus, and it writes no file. The first line names the package, its version and the count of kinds. It also counts and names each abstract kind. An abstract kind gets no entry of its own, because no document is of that kind. Its required facets and sections appear in each kind under it.

Each concrete kind gets one block, in the order that the lock declares the kinds. The block names the parent of the kind and its purpose, with the intent of that purpose and each question that the purpose answers. It names each shelf that carries the kind, with the path pattern of the shelf. On a heterogeneous shelf it also names the facet that selects the kind. It lists the facets and sections that the kind requires after inheritance. It gives the [`headwater new`](headwater-new.md#synopsis) command for the kind, in the form `headwater new <kind> --title "<title>"`. A kind that no shelf carries has no such line, and the block says that no shelf can place one. The line is the command and not a promise that it succeeds. `headwater new` decides the rest. It refuses a kind that has no identifier scheme, a kind that requires a closed-set facet that only `--facet` can supply, and a kind that two shelves carry, and it names what it needs.

A taxonomy states when to write a kind in the `write_when` member of the `kind` block. The `when` line of each block prints that sentence. A kind that declares no sentence takes the sentence of its nearest ancestor through `is_a`. So an abstract kind can state it once for each kind under it. `validate` refuses a blank sentence, and a lock that carries one anyway reads as a kind that declares none. Where no kind in the chain declares one, the `when` line is a fixed sentence that states this gap. Where the purpose declares answers, the sentence sends the reader to them. Where it declares none, a second fixed sentence says so. The verb writes no other word about a kind that the lock does not hold. In `--json`, `when.declared` holds the declared sentence, and `when.note` holds the gap sentence. The member that the chain does not fill is `null`. `--json` writes the same content as one JSON document on standard output. The MCP `kinds` tool returns the same bytes, because one renderer in `headwater-query` writes both.

## Preconditions

The consumer declaration and package sources must be readable for source operations. Package and artifact paths must exist for `publish`, `vendor`, `diff` and `migrate`. A write operation must satisfy its directory, digest, version and migration preconditions.

## Options

| Word and options | What it does |
|---|---|
| `validate` | Validates taxonomy sources without writing. |
| `resolve [--check]` | Writes or checks `.headwater/taxonomy.lock`. |
| `audit [--now <date>] [--record] [--json]` | Measures the taxonomy against the corpus. `--record` appends one adoption reading to `.headwater/adoption.jsonl`, and refuses a reading the store already holds at this lock and this date. `--json` writes the subject and the governed scope as one JSON document on standard output. |
| `publish [--package <name>] [--from <dir>] [--assembly <name>] [--out <dir>] [--clear-killed] [--check] [--json]` | Writes a package artifact and release record, or a flattened artifact from the named assembly. `--clear-killed` removes the files a killed publish left at the output directory, and publishes in the same run. `--json` writes the record as one JSON document on standard output. `--check` takes `--from` and no other option. It writes nothing in the tree, and it compares the vendored copy with a fresh publish of the source. |
| `vendor <dir-or-location> [--expect <digest>]` | Installs an artifact after digest validation, from a directory somebody already fetched or from an `https://` location this verb fetches. It reports a second artifact installed under the version already there, with both digests and both member counts. Where `--expect` supplied the digest, the artifact matched it and `.headwater/taxonomy.yml` declares no `taxonomy.digest`, it writes that digest there. It never replaces a declared pin. |
| `diff <dir> [--to <version>] [--now <date>]` | Compares a fetched artifact with the current taxonomy. |
| `migrate <dir> [--to <version>] [--apply] [--now <date>]` | Reports or applies migration steps. It reads the version it migrates from out of the lock header against no pin, and it refuses a transition that is not forward. |
| `graph [--view concrete\|abstract] [--legend]` | Prints the resolved taxonomy as a Mermaid flowchart on standard output, and writes no file. `--view abstract` draws the abstract kinds and the kinds under them. `--legend` adds a key. |
| `kinds [--json]` | Lists each concrete kind of the resolved taxonomy with its parent, purpose, shelves, inherited facets and sections, and the `headwater new` line, and writes no file. `--json` writes the same content as one JSON document. |
| `--root <path>` | Selects the repository to load. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

## Exit status

`validate`, `resolve`, `publish`, `vendor`, `diff` and `migrate` return **0** when their operation succeeds and **1** on refusal or write failure. `publish --json` and `audit --json` move no exit status. A refusal under either writes no document on standard output, and its account is one English sentence on standard error. That is the rule [HW-DR-0043](../decisions/0043-q43-whether-a-refusal-under-json-is-a-json-document.md) states for every `--json` this binary takes. `resolve --check` returns **1** for a stale lock. `publish --check` returns **1** where the vendored copy does not match a fresh publish of the source. It also returns **1** where it finds no vendored copy with a release record. `audit` returns **0** after it reports its measurements, and **1** where `--record` cannot write or read the store. `kinds` returns **0** after it prints the list, and **1** where the lock is absent or does not read.

**1**, and never 101, when standard output or standard error cannot be written, and one sentence on standard error names a failed standard output.

## Environment

The command reads the system date when a subcommand has `--now` and no date is supplied. `vendor <location>` also reads the proxy variables for an `https://` request. The first of `ALL_PROXY`, `HTTPS_PROXY` and `HTTP_PROXY` that is set names the proxy, in upper or lower case. `NO_PROXY` names the hosts that the proxy does not serve. A plain `http://` request goes only to a loopback host, and never through a proxy. So no proxy can send it to another machine. The command reads no other environment variable.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.yml`, package sources and overlay | Read by validation and resolution. |
| `.headwater/taxonomy.lock` | Written by `resolve` without `--check`. Read by `diff`, which also re-hashes the source files it records. Read by `graph` and `kinds`, which read nothing else. |
| `.headwater/adoption.jsonl` | Read by `audit`, and appended to by `audit --record`. |
| `.headwater/imports/` | Read by `audit`, `diff` and `migrate` where `.headwater/taxonomy.yml` declares an import, for the anchors that an imported snapshot supplies. An `imports` entry that does not read stops the verb with exit 1, and so does an `at` path outside the repository root. |
| the path each `harvests.<name>.at` names | Read by `audit`, `diff` and `migrate` where `.headwater/taxonomy.yml` declares a pinned corpus export, for the anchors that export supplies. A `harvests` entry that does not read stops the verb with exit 1, and so does an `at` path outside the repository root. A pin with no digest binds no anchor. An absent file binds no anchor, and neither does a file that fails the pinned digest or is not an export. |
| Package and artifact directories | `publish` and `migrate --apply` write a package directory, and `diff` and `migrate` read one. `vendor` reads the artifact it installs, and it writes `.headwater/packages/<name>`, `.headwater/packages/~staging/<name>` and `.headwater/packages/<name>~aside`. It also writes one line of `.headwater/taxonomy.yml`, only when `--expect` was given, the install succeeded and no pin was declared. It writes no other path in the consumer tree. `publish`, `vendor` and `diff` refuse a file of the package or the artifact that is not a regular file, before they open it. A named pipe is one example. |
| The source at `--from` and its vendored copy, under `publish --check` | Read. The vendored copy is `.headwater/packages/<dir>`, where `<dir>/package.yml` declares the name the source declares. The run writes only a private directory outside the tree, and it removes that directory before it exits. |
| `packages/`, the root before engine 0.2.0 | Read by no verb. A lookup that finds no package, on a tree that carries a directory there, names that directory and says to move what is under it. |

## See also

[`headwater init`](headwater-init.md) creates the consumer declaration. [`headwater generate`](headwater-generate.md) updates projections after resolution. [`headwater import`](headwater-import.md) consumes a pinned external snapshot.
